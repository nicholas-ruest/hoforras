//! `FederatedTrainer` (DDD-01 / FR-3.8) — cross-building model improvement, gradients only.
//!
//! District training deliberately moves data *across* buildings. ADR-0005 makes no-raw-egress a
//! **compile-time** guarantee: `GradientAggregator::aggregate` accepts only `Vec<Gradient>`, and
//! `Gradient` has no constructor from any raw sensor type (no `From<ThermalFrame>`). A raw frame
//! reaching the aggregator is therefore a *type error*, not a test failure. This trainer only ever
//! forwards `Gradient` values, then witnesses the aggregation (FR-4.3).

use hoforras_domain::{Gradient, PrivilegedAction};
use hoforras_ports::broker::GradientAggregator;
use hoforras_ports::security::WitnessChain;
use hoforras_ports::PortResult;

/// Federated training round driver. Generic over its ports (London-School, ADR-0001).
pub struct FederatedTrainer<A, W> {
    aggregator: A,
    witness: W,
}

impl<A, W> FederatedTrainer<A, W>
where
    A: GradientAggregator,
    W: WitnessChain,
{
    pub fn new(aggregator: A, witness: W) -> Self {
        Self {
            aggregator,
            witness,
        }
    }

    /// One federated round: aggregate the local gradient with peers' gradients (BFT), then witness
    /// it. The signature admits **only** `Gradient` — no raw frame can be passed (ADR-0005 / FR-3.8).
    pub async fn round(
        &self,
        local: Gradient,
        peer_gradients: Vec<Gradient>,
    ) -> PortResult<Gradient> {
        let mut gradients = peer_gradients;
        gradients.push(local);
        let aggregated = self.aggregator.aggregate(gradients).await?;
        self.witness.emit(PrivilegedAction::GradientAggregated)?;
        Ok(aggregated)
    }
}
