//! The **Appliance** — Completion-phase assembly of every context behind ONE tokio runtime
//! (ADR-0010), with all mocks retired (sparc.md Part 5 §1).
//!
//! This is the DDD-00 context map made concrete: each domain unit is wired to its **real** adapter.
//! The only sanctioned fakes are physical hardware / live network (e.g. [`SimSensorAdapter`] for the
//! 30-day backfill). Latency-critical RVM coherence still runs synchronously off the async path
//! (ADR-0004); everything I/O-bound shares the single runtime owned by the binary's `#[tokio::main]`.
//!
//! ```text
//! Sensing ──ThermalBalance──▶ Broker(MRAP) ──agreement──▶ Consensus(ML-DSA/QuDAG) ──▶ Witness chain
//!    │                            │                                                       ▲
//!    └─ raw stays node-local      └─ Forecast (edge WASM) · Memory (HNSW) · Economy (credits)
//! ```

use hoforras_broker::adapters::{
    DaaEconomyAdapter, DaaOrchestratorAdapter, DaaRulesAdapter, SeedMeshAdapter,
};
use hoforras_broker::BrokerAgent;
use hoforras_domain::{NodeId, Strategy, ThermalBalance};
use hoforras_mesh::consensus::{ConsensusGateway, QuDagAdapter};
use hoforras_ports::PortResult;

use crate::forecast::{ForecastServiceImpl, RuvSwarmAdapter};
use crate::isolation::adapters::RvmWitnessAdapter;

/// The real, post-quantum consensus gateway as wired in the Appliance.
pub type ApplianceConsensus = ConsensusGateway<QuDagAdapter, QuDagAdapter, QuDagAdapter>;

/// The fully-wired broker type — every collaborator is a production adapter (mocks retired).
pub type ApplianceBroker = BrokerAgent<
    SeedMeshAdapter,
    ForecastServiceImpl<RuvSwarmAdapter>,
    DaaRulesAdapter,
    DaaOrchestratorAdapter,
    DaaOrchestratorAdapter,
    DaaOrchestratorAdapter,
    DaaEconomyAdapter,
    RvmWitnessAdapter,
    ApplianceConsensus,
>;

/// One Appliance per building. Owns the node identity and constructs the wired contexts.
pub struct Appliance {
    pub node_id: NodeId,
}

impl Appliance {
    pub fn new(node_id: NodeId) -> Self {
        Self { node_id }
    }

    /// Edge-only forecasting over ephemeral ruv-swarm agents (DDD-03 / ADR-0012).
    pub fn forecast_service() -> ForecastServiceImpl<RuvSwarmAdapter> {
        ForecastServiceImpl::new(RuvSwarmAdapter::new())
    }

    /// The 64-byte hash-chained witness chain (DDD-04 / FR-4.3).
    pub fn witness_chain() -> RvmWitnessAdapter {
        RvmWitnessAdapter::new()
    }

    /// The real QuDAG consensus gateway (ML-DSA-65 + ML-KEM-1024, DDD-05 / ADR-0011).
    pub fn consensus_gateway() -> PortResult<ApplianceConsensus> {
        Ok(ConsensusGateway::new(
            QuDagAdapter::new()?,
            QuDagAdapter::new()?,
            QuDagAdapter::new()?,
        ))
    }

    /// Governance hard limits, fail-closed (DDD-01 / ADR-0015).
    pub fn rules() -> DaaRulesAdapter {
        DaaRulesAdapter::new()
    }

    /// The kWh-equivalent thermal-credit ledger (DDD-01 / ADR-0013).
    pub fn economy() -> DaaEconomyAdapter {
        DaaEconomyAdapter::new()
    }

    /// Build the fully-wired `BrokerAgent` for this node — the DDD-00 context map assembled with
    /// real adapters behind the single runtime (ADR-0010, mocks retired).
    pub fn build_broker(
        &self,
        balance: ThermalBalance,
        strategy: Strategy,
    ) -> PortResult<ApplianceBroker> {
        Ok(BrokerAgent::new(
            SeedMeshAdapter::new(balance),
            Self::forecast_service(),
            Self::rules(),
            DaaOrchestratorAdapter::new(strategy.clone()),
            DaaOrchestratorAdapter::new(strategy.clone()),
            DaaOrchestratorAdapter::new(strategy),
            Self::economy(),
            Self::witness_chain(),
            Self::consensus_gateway()?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hoforras_domain::TradeWindow;

    fn node() -> NodeId {
        NodeId::new("pozsonyi14.thermal.budapest.dark").unwrap()
    }

    #[test]
    fn appliance_wires_a_real_broker_with_no_mocks() {
        let appliance = Appliance::new(node());
        let balance = ThermalBalance {
            node_id: node(),
            surplus_kwh: 100.0,
            deficit_kwh: 0.0,
            window: TradeWindow::new(6).unwrap(),
        };
        let strategy = Strategy {
            offer_threshold_kwh: 5.0,
            bid_threshold_kwh: 5.0,
            reserve_pct: 0.15,
            base_credit_per_kwh: 2.0,
            window_hours: 6,
        };
        // Constructs the entire context map with real adapters — a compile-and-build proof of ADR-0010.
        assert!(appliance.build_broker(balance, strategy).is_ok());
    }
}
