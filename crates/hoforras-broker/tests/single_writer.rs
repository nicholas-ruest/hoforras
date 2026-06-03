//! Single-writer / one-tick-in-flight invariant (ADR-0006 / DDD-01 aggregate).
//!
//! The `TradingNode` is a single consistency boundary: at most one MRAP tick mutates it at a time.
//! This test drives two `tick()`s concurrently against a shared agent and asserts the Monitor read
//! never overlaps — the tick lock serializes them, which is what makes the FR-3.7 ordering
//! impossible to interleave across ticks.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use hoforras_broker::BrokerAgent;
use hoforras_domain::{
    Adaptation, DemandForecast, NodeId, ReadingWindow, Strategy, ThermalBalance, TradeWindow,
};
use hoforras_ports::broker::{
    MockEconomyLedger, MockMarketGateway, MockRulesEngine, MockTradeEvaluator, SeedMesh,
    StrategyStore,
};
use hoforras_ports::consensus::MockConsensusGateway;
use hoforras_ports::inference::ForecastService;
use hoforras_ports::security::MockWitnessChain;
use hoforras_ports::PortResult;

fn node() -> NodeId {
    NodeId::new("pozsonyi14.thermal.budapest.dark").unwrap()
}

/// A Monitor source that records the maximum number of concurrent reads it ever sees.
struct ConcurrencyProbeSeed {
    in_flight: Arc<AtomicUsize>,
    max_seen: Arc<AtomicUsize>,
}
#[async_trait]
impl SeedMesh for ConcurrencyProbeSeed {
    async fn read_surplus_deficit(&self) -> PortResult<ThermalBalance> {
        let now = self.in_flight.fetch_add(1, Ordering::SeqCst) + 1;
        self.max_seen.fetch_max(now, Ordering::SeqCst);
        tokio::time::sleep(Duration::from_millis(20)).await; // widen the overlap window
        self.in_flight.fetch_sub(1, Ordering::SeqCst);
        Ok(ThermalBalance {
            node_id: node(),
            surplus_kwh: 40.0, // ⇒ Hold ⇒ only Monitor/Reason run
            deficit_kwh: 0.0,
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
            offer_threshold_kwh: 5.0,
            bid_threshold_kwh: 5.0,
            reserve_pct: 0.15,
            base_credit_per_kwh: 2.0,
            window_hours: 6,
        }
    }
    fn update(&self, _a: Adaptation) {}
}

#[tokio::test]
async fn two_concurrent_ticks_never_overlap() {
    let in_flight = Arc::new(AtomicUsize::new(0));
    let max_seen = Arc::new(AtomicUsize::new(0));

    let agent = Arc::new(BrokerAgent::new(
        ConcurrencyProbeSeed {
            in_flight: in_flight.clone(),
            max_seen: max_seen.clone(),
        },
        FakeForecast,
        MockRulesEngine::new(),
        MockMarketGateway::new(),
        MockTradeEvaluator::new(),
        FakeStrategy,
        MockEconomyLedger::new(),
        MockWitnessChain::new(),
        MockConsensusGateway::new(),
    ));

    // Fire several ticks at once; the single-writer lock must serialize them.
    let a = agent.clone();
    let b = agent.clone();
    let c = agent.clone();
    let (ra, rb, rc) = tokio::join!(a.tick(), b.tick(), c.tick());
    ra.unwrap();
    rb.unwrap();
    rc.unwrap();

    assert_eq!(
        max_seen.load(Ordering::SeqCst),
        1,
        "at most one MRAP tick may be in flight per node (ADR-0006)"
    );
}
