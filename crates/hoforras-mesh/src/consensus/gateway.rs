//! `ConsensusGateway` (DDD-05 / FR-5.1–5.5) — the anti-corruption layer over QuDAG.
//!
//! The market hands it a `ThermalTradeAgreement` and gets back `Finality`; QuDAG's vocabulary never
//! leaks past this boundary. The headline invariant is **sign-before-broadcast** (FR-5.1): an entry
//! is ML-DSA-signed *before* it is broadcast to QR-Avalanche consensus. Peer discovery is
//! directory-less over `.dark` domains (FR-5.4 — there is no directory port), and `verify` is
//! tamper-evident (FR-5.5). Duration ≤ 6h is re-checked here as defense-in-depth against the
//! market's own governance (DDD-05 §3).

use hoforras_domain::{
    DagEntry, DomainError, Finality, NodeAddr, ThermalTradeAgreement, GOVERNANCE,
};
use hoforras_ports::consensus::{DagNetwork, MlDsaSigner, PeerDiscovery};
use hoforras_ports::PortResult;

/// Generic over its three ports so it is unit-tested against mocks (London-School, ADR-0001).
pub struct ConsensusGateway<S, N, D> {
    signer: S,
    network: N,
    discovery: D,
}

impl<S, N, D> ConsensusGateway<S, N, D>
where
    S: MlDsaSigner,
    N: DagNetwork,
    D: PeerDiscovery,
{
    pub fn new(signer: S, network: N, discovery: D) -> Self {
        Self {
            signer,
            network,
            discovery,
        }
    }

    /// Sign the agreement with ML-DSA, **then** broadcast it for QR-Avalanche consensus (FR-5.1/5.3).
    /// The boundary check runs first, so a malformed/overlong trade is rejected *before* signing.
    pub async fn finalize(&self, trade: ThermalTradeAgreement) -> PortResult<Finality> {
        boundary_check(&trade)?; // duration ≤ 6h etc. — defense in depth, before any crypto
        let entry = DagEntry::new(trade);
        let signed = self.signer.sign(entry)?; // ML-DSA sign exactly once (FR-5.1/5.2)
        self.network.broadcast_and_await_consensus(signed).await // broadcast once (FR-5.3 / NFR-2)
    }

    /// Resolve a peer by its `.dark` domain via Kademlia DHT — no directory (FR-5.4).
    pub async fn resolve_peer(&self, dark_domain: impl Into<String>) -> PortResult<NodeAddr> {
        self.discovery.resolve(dark_domain.into()).await
    }

    /// Verify a DAG entry's ML-DSA signature; any mutation of the agreement makes this false
    /// (FR-5.5, tamper-evidence).
    pub fn verify(&self, entry: &DagEntry) -> bool {
        self.signer.verify(entry)
    }
}

/// The broker-facing consensus boundary (DDD-01 §7 → DDD-05 ACL). Implementing the
/// `hoforras_ports::consensus::ConsensusGateway` port lets `BrokerAgent` finalize a trade through
/// the *real* QuDAG gateway in the assembled Appliance (P10 wiring) with no change to the broker.
/// `self.finalize` resolves to the inherent method above (inherent methods take resolution
/// priority), so this is a thin adapter, not a recursion.
#[async_trait::async_trait]
impl<S, N, D> hoforras_ports::consensus::ConsensusGateway for ConsensusGateway<S, N, D>
where
    S: MlDsaSigner,
    N: DagNetwork,
    D: PeerDiscovery,
{
    async fn finalize(&self, agreement: ThermalTradeAgreement) -> PortResult<Finality> {
        self.finalize(agreement).await
    }
}

/// Boundary validation (DDD-05 §3): a trade window may never exceed the governance ceiling, and the
/// economic fields must be sane. Runs before signing so invalid trades never enter consensus.
fn boundary_check(trade: &ThermalTradeAgreement) -> PortResult<()> {
    if trade.duration_hours > GOVERNANCE.trade_window_hours {
        return Err(DomainError::Governance(format!(
            "trade duration {}h exceeds the {}h governance ceiling",
            trade.duration_hours, GOVERNANCE.trade_window_hours
        )));
    }
    if !trade.kwh_offered.is_finite() || trade.kwh_offered <= 0.0 {
        return Err(DomainError::Invalid(
            "kwh_offered must be positive and finite".into(),
        ));
    }
    if !trade.credit_price_per_kwh.is_finite() || trade.credit_price_per_kwh < 0.0 {
        return Err(DomainError::Invalid(
            "credit_price_per_kwh must be non-negative and finite".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use hoforras_domain::{JunctionId, NodeId, Timestamp};

    fn trade(duration_hours: u32) -> ThermalTradeAgreement {
        ThermalTradeAgreement {
            seller: NodeId::new("pozsonyi14.thermal.budapest.dark").unwrap(),
            buyer: NodeId::new("pozsonyi22.thermal.budapest.dark").unwrap(),
            kwh_offered: 40.0,
            duration_hours,
            credit_price_per_kwh: 2.3,
            pipe_route: vec![JunctionId(7), JunctionId(12)],
            valid_from: Timestamp(0),
        }
    }

    #[test]
    fn boundary_accepts_six_hours_rejects_seven() {
        assert!(boundary_check(&trade(6)).is_ok());
        assert!(matches!(
            boundary_check(&trade(7)),
            Err(DomainError::Governance(_))
        ));
    }

    #[test]
    fn boundary_rejects_nonpositive_kwh() {
        let mut t = trade(6);
        t.kwh_offered = 0.0;
        assert!(matches!(boundary_check(&t), Err(DomainError::Invalid(_))));
        t.kwh_offered = f32::NAN;
        assert!(matches!(boundary_check(&t), Err(DomainError::Invalid(_))));
    }
}
