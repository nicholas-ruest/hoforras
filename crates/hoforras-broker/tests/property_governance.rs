//! Governance-before-execute + fail-closed property (FR-3.7 / ADR-0015 / DDD-01 §5.1).
//!
//! For ANY proposed trade, if the rules engine returns a `Violation` — including the fail-closed
//! case where an *unavailable* engine denies — the broker performs **zero** executions and witnesses
//! exactly one rejection. Driven against the real `BrokerAgent` with a counting market and witness;
//! the rules engine is the real fail-closed `DaaRulesAdapter::unavailable()`.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use hoforras_broker::adapters::DaaRulesAdapter;
use hoforras_broker::{BrokerAgent, TickReport};
use hoforras_domain::{
    AcceptedTrade, Adaptation, Bid, Decision, DemandForecast, Finality, JunctionId, NodeId, Offer,
    PrivilegedAction, ReadingWindow, Reflection, Strategy, ThermalBalance, ThermalTradeAgreement,
    Timestamp, TradeOutcome, TradeWindow, WitnessRecord,
};
use hoforras_ports::broker::{
    EconomyLedger, MarketGateway, SeedMesh, StrategyStore, TradeEvaluator,
};
use hoforras_ports::consensus::ConsensusGateway;
use hoforras_ports::inference::ForecastService;
use hoforras_ports::security::WitnessChain;
use hoforras_ports::PortResult;
use proptest::prelude::*;

fn node() -> NodeId {
    NodeId::new("pozsonyi14.thermal.budapest.dark").unwrap()
}

// --- counting fakes (not mocks — we read the counters after the run) ---

#[derive(Clone, Default)]
struct Counters {
    executes: Arc<AtomicUsize>,
    routes: Arc<AtomicUsize>,
    rejections: Arc<AtomicUsize>,
}

struct CountingMarket(Counters);
#[async_trait]
impl MarketGateway for CountingMarket {
    async fn post_offer(&self, _o: Offer) -> PortResult<()> {
        Ok(())
    }
    async fn post_bid(&self, _b: Bid) -> PortResult<()> {
        Ok(())
    }
    async fn await_acceptance(&self, _d: &Decision) -> PortResult<Option<AcceptedTrade>> {
        Ok(None)
    }
    async fn execute(&self, _t: &AcceptedTrade) -> PortResult<()> {
        self.0.executes.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
    async fn route(&self, _t: &AcceptedTrade, _p: &[JunctionId]) -> PortResult<()> {
        self.0.routes.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

struct CountingWitness(Counters);
impl WitnessChain for CountingWitness {
    fn emit(&self, action: PrivilegedAction) -> PortResult<WitnessRecord> {
        if matches!(action, PrivilegedAction::RuleRejection { .. }) {
            self.0.rejections.fetch_add(1, Ordering::SeqCst);
        }
        Ok(WitnessRecord::new([0u8; 64]))
    }
    fn verify(&self) -> PortResult<bool> {
        Ok(true)
    }
}

struct FakeSeed {
    surplus: f32,
    deficit: f32,
}
#[async_trait]
impl SeedMesh for FakeSeed {
    async fn read_surplus_deficit(&self) -> PortResult<ThermalBalance> {
        Ok(ThermalBalance {
            node_id: node(),
            surplus_kwh: self.surplus,
            deficit_kwh: self.deficit,
            window: TradeWindow::new(6).unwrap(),
        })
    }
}

struct FakeForecast;
#[async_trait]
impl ForecastService for FakeForecast {
    async fn anomalies(&self, _w: ReadingWindow) -> PortResult<Vec<hoforras_domain::Anomaly>> {
        Ok(vec![])
    }
    async fn burst_precursor(
        &self,
        _w: ReadingWindow,
    ) -> PortResult<Option<hoforras_domain::BurstWarning>> {
        Ok(None)
    }
    async fn demand_24h(&self, _w: ReadingWindow) -> PortResult<DemandForecast> {
        Ok(DemandForecast {
            hourly_kwh: vec![10.0; 24],
        })
    }
}

struct FakeStrategy;
impl StrategyStore for FakeStrategy {
    fn load(&self) -> Strategy {
        Strategy {
            offer_threshold_kwh: 1.0,
            bid_threshold_kwh: 1.0,
            reserve_pct: 0.15,
            base_credit_per_kwh: 2.0,
            window_hours: 6,
        }
    }
    fn update(&self, _a: Adaptation) {}
}

struct FakeEvaluator;
impl TradeEvaluator for FakeEvaluator {
    fn evaluate(&self, _o: &TradeOutcome) -> Reflection {
        Reflection {
            value_ratio: 1.0,
            note: String::new(),
        }
    }
}

struct FakeLedger;
#[async_trait]
impl EconomyLedger for FakeLedger {
    async fn debit_credit(&self, _t: &hoforras_domain::ExecutedTrade) -> PortResult<()> {
        Ok(())
    }
}

struct FakeConsensus;
#[async_trait]
impl ConsensusGateway for FakeConsensus {
    async fn finalize(&self, _a: ThermalTradeAgreement) -> PortResult<Finality> {
        Ok(Finality {
            reached_at: Timestamp(0),
        })
    }
}

proptest! {
    /// FR-3.7 / ADR-0015: a fail-closed (unavailable) rules engine ⇒ zero executes/routes and a
    /// witnessed rejection, for ANY surplus/deficit that would otherwise trade.
    #[test]
    fn prop_fail_closed_never_executes(
        surplus in 0.0f32..2000.0,
        deficit in 0.0f32..2000.0,
    ) {
        let counters = Counters::default();
        let agent = BrokerAgent::new(
            FakeSeed { surplus, deficit },
            FakeForecast,
            DaaRulesAdapter::unavailable(), // ADR-0015: fail closed ⇒ always Violation
            CountingMarket(counters.clone()),
            FakeEvaluator,
            FakeStrategy,
            FakeLedger,
            CountingWitness(counters.clone()),
            FakeConsensus,
        );

        let rt = tokio::runtime::Builder::new_current_thread().build().unwrap();
        let report = rt.block_on(async { agent.tick().await.unwrap() });

        // If a trade was proposed at all, it must have been rejected (never executed).
        prop_assert_eq!(counters.executes.load(Ordering::SeqCst), 0); // zero executions
        prop_assert_eq!(counters.routes.load(Ordering::SeqCst), 0);   // zero routing
        match report {
            TickReport::Idle => {
                // Balanced ⇒ Hold ⇒ no proposal, no rejection.
                prop_assert_eq!(counters.rejections.load(Ordering::SeqCst), 0);
            }
            TickReport::Rejected(_) => {
                // A proposal was made and denied — witnessed exactly once.
                prop_assert_eq!(counters.rejections.load(Ordering::SeqCst), 1);
            }
            other => prop_assert!(false, "fail-closed engine must never reach {:?}", other),
        }
    }
}
