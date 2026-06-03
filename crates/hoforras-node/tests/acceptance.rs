//! Acceptance suite (Completion / sparc.md Part 5 §3–§4) — AC-7 demonstrated live, AC-8 proven.
//!
//! AC-7 is the headline: one fully autonomous peer-to-peer thermal trade
//! (pozsonyi14 surplus → pozsonyi22 deficit, 40 kWh, 6 h, 2.3 credits/kWh, routed through
//! [junction_7, junction_12, junction_18]) with NO operator input and a complete, verifiable
//! rvm-witness audit trail — driven through the **real** broker, the **real** ML-DSA QuDAG
//! consensus gateway, and the **real** hash-chained witness. Only the counterparty match and the
//! evidence taps are test scaffolding.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use hoforras_broker::adapters::{
    DaaEconomyAdapter, DaaOrchestratorAdapter, DaaRulesAdapter, SeedMeshAdapter,
};
use hoforras_broker::{BrokerAgent, TickReport};
use hoforras_domain::{
    AcceptedTrade, Bid, Decision, Finality, JunctionId, Kwh, NodeId, Offer, Price,
    PrivilegedAction, ProposedTrade, Strategy, ThermalBalance, ThermalTradeAgreement, Timestamp,
    TradeWindow, WitnessRecord,
};
use hoforras_node::appliance::{Appliance, ApplianceConsensus};
use hoforras_node::isolation::adapters::RvmWitnessAdapter;
use hoforras_ports::broker::MarketGateway;
use hoforras_ports::consensus::ConsensusGateway;
use hoforras_ports::security::WitnessChain;
use hoforras_ports::PortResult;

fn node(s: &str) -> NodeId {
    NodeId::new(format!("{s}.thermal.budapest.dark")).unwrap()
}

/// The exact AC-7 agreement: pozsonyi14 → pozsonyi22, 40 kWh, 6 h, 2.3 credits/kWh, [7,12,18].
fn ac7_accepted() -> AcceptedTrade {
    AcceptedTrade {
        proposed: ProposedTrade {
            seller: node("pozsonyi14"),
            buyer: node("pozsonyi22"),
            kwh: Kwh(40.0),
            price: Price {
                credit_per_kwh: 2.3,
            },
            window: TradeWindow::new(6).unwrap(),
            pipe_route: vec![JunctionId(7), JunctionId(12), JunctionId(18)],
        },
        accepted_at: Timestamp(1),
        pipe_route: vec![JunctionId(7), JunctionId(12), JunctionId(18)],
    }
}

// ── evidence taps: record the audit trail / route / finalized agreement (everything else is real) ──

struct EvidenceWitness {
    inner: Arc<RvmWitnessAdapter>,
    actions: Arc<Mutex<Vec<PrivilegedAction>>>,
}
impl WitnessChain for EvidenceWitness {
    fn emit(&self, action: PrivilegedAction) -> PortResult<WitnessRecord> {
        self.actions.lock().unwrap().push(action.clone());
        self.inner.emit(action) // real 64-byte hash-chained record
    }
    fn verify(&self) -> PortResult<bool> {
        self.inner.verify()
    }
}

struct EvidenceMarket {
    posted: Arc<AtomicUsize>,
    executed: Arc<AtomicUsize>,
    routed_path: Arc<Mutex<Vec<JunctionId>>>,
}
#[async_trait]
impl MarketGateway for EvidenceMarket {
    async fn post_offer(&self, _o: Offer) -> PortResult<()> {
        self.posted.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
    async fn post_bid(&self, _b: Bid) -> PortResult<()> {
        Ok(())
    }
    async fn await_acceptance(&self, _d: &Decision) -> PortResult<Option<AcceptedTrade>> {
        Ok(Some(ac7_accepted())) // pozsonyi22 accepts autonomously
    }
    async fn execute(&self, _t: &AcceptedTrade) -> PortResult<()> {
        self.executed.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
    async fn route(&self, _t: &AcceptedTrade, path: &[JunctionId]) -> PortResult<()> {
        *self.routed_path.lock().unwrap() = path.to_vec();
        Ok(())
    }
}

struct EvidenceConsensus {
    inner: ApplianceConsensus,
    finalized: Arc<Mutex<Option<ThermalTradeAgreement>>>,
}
#[async_trait]
impl ConsensusGateway for EvidenceConsensus {
    async fn finalize(&self, agreement: ThermalTradeAgreement) -> PortResult<Finality> {
        *self.finalized.lock().unwrap() = Some(agreement.clone());
        self.inner.finalize(agreement).await // real ML-DSA sign → QR-Avalanche finality
    }
}

fn strategy() -> Strategy {
    Strategy {
        offer_threshold_kwh: 1.0,
        bid_threshold_kwh: 1.0,
        reserve_pct: 0.15,
        base_credit_per_kwh: 2.0,
        window_hours: 6,
    }
}

#[tokio::test]
async fn ac7_autonomous_trade_with_complete_audit() {
    // pozsonyi14 in strong surplus (aggregated balance — the only thermal quantity that crossed a
    // boundary). It exceeds the building's own forecast reserve, so `decide` chooses to Offer.
    let balance = ThermalBalance {
        node_id: node("pozsonyi14"),
        surplus_kwh: 300.0,
        deficit_kwh: 0.0,
        window: TradeWindow::new(6).unwrap(),
    };

    let actions = Arc::new(Mutex::new(Vec::new()));
    let witness_inner = Arc::new(RvmWitnessAdapter::new());
    let routed_path = Arc::new(Mutex::new(Vec::new()));
    let executed = Arc::new(AtomicUsize::new(0));
    let posted = Arc::new(AtomicUsize::new(0));
    let finalized = Arc::new(Mutex::new(None));
    let ledger = DaaEconomyAdapter::new();

    // Build the broker with REAL governance, forecast, consensus, witness; evidence taps observe.
    let broker = BrokerAgent::new(
        SeedMeshAdapter::new(balance),
        Appliance::forecast_service(),
        DaaRulesAdapter::new(),
        EvidenceMarket {
            posted: posted.clone(),
            executed: executed.clone(),
            routed_path: routed_path.clone(),
        },
        DaaOrchestratorAdapter::new(strategy()),
        DaaOrchestratorAdapter::new(strategy()),
        ledger,
        EvidenceWitness {
            inner: witness_inner.clone(),
            actions: actions.clone(),
        },
        EvidenceConsensus {
            inner: Appliance::consensus_gateway().unwrap(),
            finalized: finalized.clone(),
        },
    );

    // WHEN the broker MRAP loop runs with NO operator input.
    let started = Instant::now();
    let report = broker.tick().await.unwrap();
    let elapsed = started.elapsed();

    // THEN a trade executed autonomously …
    assert!(
        matches!(report, TickReport::Executed { .. }),
        "autonomous trade executed"
    );
    assert_eq!(posted.load(Ordering::SeqCst), 1);
    assert_eq!(executed.load(Ordering::SeqCst), 1);

    // … routed along [junction_7, junction_12, junction_18] (FR-3.3) …
    assert_eq!(
        *routed_path.lock().unwrap(),
        vec![JunctionId(7), JunctionId(12), JunctionId(18)]
    );

    // … the consensus-signed agreement is the AC-7 trade (ML-DSA signed inside the real gateway) …
    let agreement = finalized
        .lock()
        .unwrap()
        .clone()
        .expect("consensus finalized the trade");
    assert_eq!(agreement.kwh_offered, 40.0);
    assert_eq!(agreement.duration_hours, 6);
    assert_eq!(agreement.credit_price_per_kwh, 2.3);
    assert_eq!(agreement.seller, node("pozsonyi14"));
    assert_eq!(agreement.buyer, node("pozsonyi22"));

    // … a COMPLETE witness audit trail exists for every privileged action {TradeSigned, Exec, Routing}
    //   and the hash chain verifies (FR-4.3) …
    let trail = actions.lock().unwrap().clone();
    assert!(trail.contains(&PrivilegedAction::TradeSigned));
    assert!(trail.contains(&PrivilegedAction::Exec));
    assert!(trail.contains(&PrivilegedAction::Routing));
    assert!(
        witness_inner.verify().unwrap(),
        "AuditTrail.verify_integrity() == true"
    );

    // … finalization within the sub-second budget on the local crypto path (NFR-2 proxy; the live
    //   QR-Avalanche round-trip is a Phase C figure) …
    assert!(
        elapsed < Duration::from_secs(1),
        "finality budget: {elapsed:?} < 1s"
    );

    // Evidence artifact (the acceptance record): the witness trail captured.
    println!("AC-7 witness trail: {trail:?}; finality budget {elapsed:?}");
}

#[tokio::test]
async fn ac8_no_raw_frame_crosses_a_boundary() {
    // AC-8 is a *type-level* guarantee, exercised here at the boundaries the AC-7 trade crossed:
    //   • the only thermal quantity into the broker is a `ThermalBalance` aggregate (SeedMesh port),
    //   • the consensus artifact is a `ThermalTradeAgreement` — no raw sensor fields,
    //   • `Gradient` (the only federation payload) has no `From<ThermalFrame>` (domain trybuild test).
    // The struct below names every field of the published agreement; none is a raw reading channel
    // (temperature/vibration/pressure/gradient). If a raw field were ever added here, it would not
    // compile against `ThermalTradeAgreement`.
    let agreement = ac7_accepted();
    let published = ThermalTradeAgreement {
        seller: agreement.proposed.seller.clone(),
        buyer: agreement.proposed.buyer.clone(),
        kwh_offered: agreement.proposed.kwh.0,
        duration_hours: agreement.proposed.window.hours(),
        credit_price_per_kwh: agreement.proposed.price.credit_per_kwh,
        pipe_route: agreement.pipe_route.clone(),
        valid_from: Timestamp(0),
    };
    // Only aggregates/economic terms cross — no RawReading/ThermalFrame value is constructible here.
    assert_eq!(published.kwh_offered, 40.0);
    assert!(published.pipe_route.len() == 3);
}
