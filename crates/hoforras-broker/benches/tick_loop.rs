//! MRAP tick-loop benchmark (FR-3, the AC-7 spine).
//!
//! Measures one full `BrokerAgent::tick()` over the reference adapters along the **execute** path
//! (governance Allow → consensus → execute → route → witness×3 → account → reflect → adapt). This is
//! the autonomous-trade hot loop; it is CPU-only here (the real consensus round-trip is a Phase C
//! live-mesh figure).

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hoforras_broker::adapters::{
    DaaEconomyAdapter, DaaOrchestratorAdapter, DaaRulesAdapter, SeedMeshAdapter,
};
use hoforras_broker::BrokerAgent;
use hoforras_domain::{
    AcceptedTrade, DemandForecast, Finality, JunctionId, Kwh, NodeId, Price, PrivilegedAction,
    ProposedTrade, ReadingWindow, Strategy, ThermalBalance, ThermalTradeAgreement, Timestamp,
    TradeWindow, WitnessRecord,
};
use hoforras_ports::consensus::ConsensusGateway;
use hoforras_ports::inference::ForecastService;
use hoforras_ports::security::WitnessChain;
use hoforras_ports::PortResult;

fn node() -> NodeId {
    NodeId::new("pozsonyi14.thermal.budapest.dark").unwrap()
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

struct FakeConsensus;
#[async_trait::async_trait]
impl ConsensusGateway for FakeConsensus {
    async fn finalize(&self, _a: ThermalTradeAgreement) -> PortResult<Finality> {
        Ok(Finality {
            reached_at: Timestamp(0),
        })
    }
}

struct FakeForecast;
#[async_trait::async_trait]
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

fn strategy() -> Strategy {
    Strategy {
        offer_threshold_kwh: 5.0,
        bid_threshold_kwh: 5.0,
        reserve_pct: 0.15,
        base_credit_per_kwh: 2.0,
        window_hours: 6,
    }
}

fn accepted() -> AcceptedTrade {
    AcceptedTrade {
        proposed: ProposedTrade {
            seller: node(),
            buyer: NodeId::new("pozsonyi22.thermal.budapest.dark").unwrap(),
            kwh: Kwh(40.0),
            price: Price {
                credit_per_kwh: 2.3,
            },
            window: TradeWindow::new(6).unwrap(),
            pipe_route: vec![JunctionId(7)],
        },
        accepted_at: Timestamp(1),
        pipe_route: vec![JunctionId(7), JunctionId(12)],
    }
}

fn bench_tick(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();

    let balance = ThermalBalance {
        node_id: node(),
        surplus_kwh: 100.0,
        deficit_kwh: 0.0,
        window: TradeWindow::new(6).unwrap(),
    };

    // Three orchestrator instances (market / evaluator / strategy) — the market one is pre-armed
    // with a counterparty so the full execute path runs.
    let market = DaaOrchestratorAdapter::new(strategy());
    market.set_acceptance(Some(accepted()));

    let agent = BrokerAgent::new(
        SeedMeshAdapter::new(balance),
        FakeForecast,
        DaaRulesAdapter::new(),
        market,
        DaaOrchestratorAdapter::new(strategy()),
        DaaOrchestratorAdapter::new(strategy()),
        DaaEconomyAdapter::new(),
        NoopWitness,
        FakeConsensus,
    );

    c.bench_function("broker_tick_execute_path", |b| {
        b.iter(|| rt.block_on(async { black_box(agent.tick().await.unwrap()) }));
    });
}

criterion_group!(benches, bench_tick);
criterion_main!(benches);
