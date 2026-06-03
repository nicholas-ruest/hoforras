//! Gradient-only egress (FR-3.8 / ADR-0005).
//!
//! ADR-0005 is a **compile-time** guarantee: `GradientAggregator::aggregate` accepts only
//! `Vec<Gradient>`, and `Gradient` has no `From<ThermalFrame>` (a frame reaching the aggregator is a
//! type error — the negative is pinned by `hoforras-domain`'s trybuild compile-fail test). This
//! property test exercises the runtime complement: across arbitrary gradients, the `FederatedTrainer`
//! only ever forwards gradients to the aggregator and the result has the same dimensionality — no
//! raw value can flow through because none can be constructed.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use hoforras_broker::FederatedTrainer;
use hoforras_domain::{Gradient, PrivilegedAction, WitnessRecord};
use hoforras_ports::broker::GradientAggregator;
use hoforras_ports::security::WitnessChain;
use hoforras_ports::PortResult;
use proptest::prelude::*;

/// An aggregator that records it only ever sees `Gradient` values (which is all the type permits)
/// and returns their coordinate-wise mean.
struct GradientOnlyAggregator {
    saw_only_gradients: Arc<AtomicBool>,
}
#[async_trait]
impl GradientAggregator for GradientOnlyAggregator {
    async fn aggregate(&self, gradients: Vec<Gradient>) -> PortResult<Gradient> {
        // The argument type is `Vec<Gradient>` — there is no way for a raw frame to be here.
        self.saw_only_gradients.store(true, Ordering::SeqCst);
        let dim = gradients.first().map(Gradient::len).unwrap_or(0);
        let mut out = vec![0.0f32; dim];
        for g in &gradients {
            for (i, v) in g.as_slice().iter().enumerate() {
                out[i] += v;
            }
        }
        let n = gradients.len().max(1) as f32;
        for v in &mut out {
            *v /= n;
        }
        Ok(Gradient::from_local(out))
    }
}

struct NoopWitness;
impl WitnessChain for NoopWitness {
    fn emit(&self, _a: PrivilegedAction) -> PortResult<WitnessRecord> {
        Ok(WitnessRecord::new([0u8; 64]))
    }
    fn verify(&self) -> PortResult<bool> {
        Ok(true)
    }
}

proptest! {
    /// FR-3.8: the trainer forwards only gradients; aggregation preserves dimensionality.
    #[test]
    fn prop_only_gradients_cross_the_federation_boundary(
        dim in 1usize..16,
        peers in 0usize..6,
    ) {
        let saw_only_gradients = Arc::new(AtomicBool::new(false));
        let trainer = FederatedTrainer::new(
            GradientOnlyAggregator { saw_only_gradients: saw_only_gradients.clone() },
            NoopWitness,
        );

        let local = Gradient::from_local(vec![1.0; dim]);
        let peer_gradients: Vec<Gradient> =
            (0..peers).map(|i| Gradient::from_local(vec![i as f32; dim])).collect();

        let rt = tokio::runtime::Builder::new_current_thread().build().unwrap();
        let result = rt.block_on(async { trainer.round(local, peer_gradients).await.unwrap() });

        prop_assert!(saw_only_gradients.load(Ordering::SeqCst));
        prop_assert_eq!(result.len(), dim); // dimensionality preserved; only gradients flowed
    }
}
