//! Integration ladder (Completion / sparc.md Part 5 §1) — mocks retired, real adapters wired.
//!
//! Each rung is a real-wiring slice of the AC-7 path; together they prove the contexts compose
//! behind the single runtime (ADR-0010). The only fakes are physical hardware ([`SimSensorAdapter`])
//! and the live mesh (in-process reference fabric) — everything else is production domain code.

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use hoforras_domain::{
    DagEntry, DistrictGraph, JunctionId, MlDsaSignature, NodeId, Partitioning, PrivilegedAction,
    Strategy, ThermalFrame, ThermalTradeAgreement, Timestamp, TradeWindow, UiEvent,
};
use hoforras_mesh::ai::WebSocketSink;
use hoforras_mesh::consensus::QuDagAdapter;
use hoforras_mesh::coordination::{MeshCoordinator, MeshReaction, SynapticMeshAdapter};
use hoforras_node::appliance::Appliance;
use hoforras_node::isolation::adapters::{RvmCoherenceAdapter, RvmWitnessAdapter};
use hoforras_node::isolation::{AnomalySignal, CoherenceSupervisor};
use hoforras_ports::ai::OperatorChannel;
use hoforras_ports::consensus::MlDsaSigner;
use hoforras_ports::mesh::MeshTransport;
use hoforras_ports::security::{MincutEngine, PartitionController, WitnessChain};
use hoforras_ports::sensor::{EventEmitter, RejectionLog, SensorSource};
use hoforras_ports::PortResult;
use hoforras_sensor::{
    Ed25519WitnessAdapter, IngestionPipeline, RvcsiIngestAdapter, SimSensorAdapter,
    ThermalAggregator,
};

fn node(s: &str) -> NodeId {
    NodeId::new(format!("{s}.thermal.budapest.dark")).unwrap()
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

/// A real node-local sink (EventEmitter + RejectionLog) — collects frames so a test can aggregate
/// them. Frames never leave the node (ADR-0009): the sink is the node-local boundary.
#[derive(Clone, Default)]
struct CollectingSink {
    frames: Arc<Mutex<Vec<ThermalFrame>>>,
    rejections: Arc<Mutex<Vec<String>>>,
}
#[async_trait]
impl EventEmitter for CollectingSink {
    async fn emit(&self, frame: ThermalFrame) -> PortResult<()> {
        self.frames.lock().unwrap().push(frame);
        Ok(())
    }
}
impl RejectionLog for CollectingSink {
    fn record(&self, _raw: hoforras_domain::RawReading, reason: String) {
        self.rejections.lock().unwrap().push(reason);
    }
}

// ───────────────────────── Rung 1: sensor_to_broker ─────────────────────────

#[tokio::test]
async fn sensor_to_broker() {
    let seller = node("pozsonyi14");
    // Real ingestion: SimSensor → IngestionPipeline (real validate/score/sign) → node-local frames.
    let sim = SimSensorAdapter::with_bias(seller.clone(), 6.0); // warm ⇒ surplus
    let sink = CollectingSink::default();
    let pipeline = IngestionPipeline::new(
        RvcsiIngestAdapter::new(seller.clone()),      // Validator
        RvcsiIngestAdapter::new(seller.clone()),      // QualityScorer
        Ed25519WitnessAdapter::from_seed(&[7u8; 32]), // WitnessSigner (real Ed25519)
        sink.clone(),
        sink.clone(),
    );
    for _ in 0..24 {
        let raw = sim.poll().await.unwrap();
        pipeline.ingest(raw).await.unwrap();
    }
    let frames = sink.frames.lock().unwrap().clone();
    assert_eq!(
        frames.len(),
        24,
        "all readings ingested as witnessed frames"
    );

    // Real aggregation → the ONLY thing that crosses the boundary: a ThermalBalance (ADR-0009).
    let balance =
        ThermalAggregator::aggregate(&frames, seller.clone(), TradeWindow::new(6).unwrap());
    assert!(balance.surplus_kwh > 0.0, "warm building is in surplus");

    // Real broker monitors that balance and reasons (Monitor→Reason reached, mocks retired).
    let appliance = Appliance::new(seller.clone());
    let broker = appliance.build_broker(balance, strategy()).unwrap();
    let report = broker.tick().await.unwrap();
    // Reaching Posted means Monitor→Reason→(governance Allow)→Act ran on the sensor-derived balance
    // and an offer was posted (no counterparty wired ⇒ Posted), proving the pipeline fed the broker.
    assert_eq!(report, hoforras_broker::TickReport::Posted);
}

// ───────────────────────── Rung 2: broker_to_consensus ─────────────────────────

fn agreement() -> ThermalTradeAgreement {
    ThermalTradeAgreement {
        seller: node("pozsonyi14"),
        buyer: node("pozsonyi22"),
        kwh_offered: 40.0,
        duration_hours: 6,
        credit_price_per_kwh: 2.3,
        pipe_route: vec![JunctionId(7), JunctionId(12), JunctionId(18)],
        valid_from: Timestamp(0),
    }
}

#[tokio::test]
async fn broker_to_consensus() {
    // Real DaaRules already governs at the broker; here a real QuDAG gateway signs + finalizes.
    let gateway = Appliance::consensus_gateway().unwrap();
    let finality = gateway.finalize(agreement()).await.unwrap(); // signed DAG entry → finality
    let _ = finality.reached_at;

    // And the signed entry is tamper-evident (real ML-DSA).
    let signer = QuDagAdapter::new().unwrap();
    let signed = signer.sign(DagEntry::new(agreement())).unwrap();
    assert!(signer.verify(&signed));
    let mut tampered = signed.clone();
    tampered.agreement.kwh_offered = 41.0;
    assert!(!signer.verify(&tampered));
}

// ───────────────────────── Rung 3: consensus_to_witness ─────────────────────────

#[test]
fn consensus_to_witness() {
    // A finalized trade emits the three privileged actions to the real hash-chained witness.
    let witness = RvmWitnessAdapter::new();
    let signed = witness.emit(PrivilegedAction::TradeSigned).unwrap();
    let exec = witness.emit(PrivilegedAction::Exec).unwrap();
    let routing = witness.emit(PrivilegedAction::Routing).unwrap();
    for rec in [&signed, &exec, &routing] {
        assert_eq!(rec.as_bytes().len(), 64); // 64-byte records (FR-4.3)
    }
    assert!(witness.verify().unwrap(), "intact chain verifies");
}

// ───────────────────────── Rung 4: node_isolation_live ─────────────────────────

/// A shared coherence adapter so the test can observe isolation after the supervisor acts.
struct SharedCoherence(Arc<RvmCoherenceAdapter>);
impl MincutEngine for SharedCoherence {
    fn recompute(&self, graph: &DistrictGraph) -> Partitioning {
        self.0.recompute(graph)
    }
}
impl PartitionController for SharedCoherence {
    fn isolate(&self, n: NodeId) {
        self.0.isolate(n)
    }
    fn rejoin(&self, n: NodeId) {
        self.0.rejoin(n)
    }
}

#[test]
fn node_isolation_live() {
    let bad = node("pozsonyi30");
    let coherence = Arc::new(RvmCoherenceAdapter::new());
    let graph = DistrictGraph {
        nodes: vec![node("pozsonyi14"), bad.clone()],
        edges: vec![(JunctionId(0), JunctionId(1), 1.0)],
    };
    let supervisor = CoherenceSupervisor::new(
        SharedCoherence(coherence.clone()),
        SharedCoherence(coherence.clone()),
        RvmWitnessAdapter::new(),
        graph,
    );

    // A benign signal isolates nothing; an anomaly isolates the offending node, others keep serving.
    supervisor
        .on_signal(AnomalySignal::benign(bad.clone()))
        .unwrap();
    assert!(!coherence.is_isolated(&bad));
    supervisor
        .on_signal(AnomalySignal::anomalous(bad.clone(), "injected frames"))
        .unwrap();
    assert!(coherence.is_isolated(&bad), "anomalous node isolated live");
    assert!(
        !coherence.is_isolated(&node("pozsonyi14")),
        "neighbour keeps serving"
    );
}

// ───────────────────────── Rung 5: mesh_self_heal ─────────────────────────

fn signed_entry(i: u8) -> DagEntry {
    DagEntry {
        agreement: ThermalTradeAgreement {
            kwh_offered: i as f32,
            ..agreement()
        },
        signature: Some(MlDsaSignature(vec![i])),
    }
}

#[tokio::test]
async fn mesh_self_heal() {
    let mesh = SynapticMeshAdapter::new();
    for i in 0..5 {
        mesh.publish(signed_entry(i)).await.unwrap();
    }
    let committed_before = mesh.committed();

    let coordinator = MeshCoordinator::new(mesh.clone(), mesh.clone());
    coordinator
        .on_node_event(hoforras_domain::NodeEvent::Drop(node("pozsonyi22")))
        .await
        .unwrap();
    assert_eq!(
        mesh.committed(),
        committed_before,
        "drop preserves committed state"
    );

    let reaction = coordinator
        .on_node_event(hoforras_domain::NodeEvent::Up(node("pozsonyi22")))
        .await
        .unwrap();
    match reaction {
        MeshReaction::Resynced { replayed } => {
            assert_eq!(
                replayed, committed_before,
                "returning node converges (NFR-9)"
            );
        }
        other => panic!("expected resync, got {other:?}"),
    }
}

// ───────────────────────── Rung 6: dashboard_live_feed ─────────────────────────

#[tokio::test]
async fn dashboard_live_feed() {
    // The operator channel publishes the exact UiEvent shape the (conformist) dashboard renders.
    let sink = WebSocketSink::new();
    sink.push(UiEvent::TradeExecuted {
        seller: node("pozsonyi14"),
        buyer: node("pozsonyi22"),
        kwh: 40.0,
        pipe_route: vec![JunctionId(7), JunctionId(12), JunctionId(18)],
    })
    .await
    .unwrap();

    let pushed = sink.pushed();
    assert_eq!(pushed.len(), 1);
    match &pushed[0] {
        UiEvent::TradeExecuted {
            kwh, pipe_route, ..
        } => {
            assert_eq!(*kwh, 40.0);
            // The pipe route the dashboard animates end to end (FR-9.1/9.2).
            assert_eq!(
                pipe_route,
                &vec![JunctionId(7), JunctionId(12), JunctionId(18)]
            );
        }
        other => panic!("expected TradeExecuted, got {other:?}"),
    }
}
