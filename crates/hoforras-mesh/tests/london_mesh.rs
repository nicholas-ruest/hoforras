//! London-School interaction tests for District Mesh Coordination (R10 / FR-7.1–7.3).
//!
//! Interactions against mocked ports (`MockMeshTransport`, `MockNodeRegistry`). The headline is
//! `propagate_publishes_dag_entry` (FR-7.1): propagation goes through `MeshTransport.publish` — and
//! the *absence* of any RPC port (there is none in `hoforras-ports`) is what makes DAG-not-RPC
//! structural (ADR-0007).

use std::sync::atomic::{AtomicUsize, Ordering};

use hoforras_domain::{
    CollectiveSignal, DagEntry, JunctionId, MeshAction, MlDsaSignature, NodeEvent, NodeId,
    ThermalTradeAgreement, Timestamp,
};
use hoforras_mesh::coordination::{MeshCoordinator, MeshReaction};
use hoforras_ports::mesh::{MockMeshTransport, MockNodeRegistry};
use mockall::Sequence;

fn node(s: &str) -> NodeId {
    NodeId::new(format!("{s}.thermal.budapest.dark")).unwrap()
}

fn signed_entry() -> DagEntry {
    DagEntry {
        agreement: ThermalTradeAgreement {
            seller: node("a"),
            buyer: node("b"),
            kwh_offered: 10.0,
            duration_hours: 6,
            credit_price_per_kwh: 2.0,
            pipe_route: vec![JunctionId(1)],
            valid_from: Timestamp(0),
        },
        signature: Some(MlDsaSignature(vec![1, 2, 3])),
    }
}

// 1 — propagate publishes a signed DAG entry via MeshTransport, never RPC (FR-7.1).
#[tokio::test]
async fn propagate_publishes_dag_entry() {
    let mut transport = MockMeshTransport::new();
    transport
        .expect_publish()
        .times(1)
        .withf(|e| e.signature.is_some()) // a SIGNED entry
        .returning(|_| Ok(()));
    let coordinator = MeshCoordinator::new(transport, MockNodeRegistry::new());

    coordinator.propagate(signed_entry()).await.unwrap();
}

// 1b — an unsigned entry is refused before it ever hits the fabric (FR-7.1 / ADR-0011 dovetail).
#[tokio::test]
async fn propagate_refuses_unsigned_entry() {
    let mut transport = MockMeshTransport::new();
    transport.expect_publish().times(0); // never published
    let coordinator = MeshCoordinator::new(transport, MockNodeRegistry::new());

    let mut unsigned = signed_entry();
    unsigned.signature = None;
    assert!(coordinator.propagate(unsigned).await.is_err());
}

// 2 — Drop marks the node down, then reconfigures over the live set (FR-7.2).
#[tokio::test]
async fn node_drop_marks_down_and_reconfigures() {
    let mut seq = Sequence::new();
    let mut registry = MockNodeRegistry::new();
    registry
        .expect_mark_down()
        .times(1)
        .in_sequence(&mut seq)
        .withf(|n: &NodeId| *n == node("a"))
        .return_const(());
    registry
        .expect_live()
        .times(1)
        .in_sequence(&mut seq) // reconfigure happens AFTER mark_down
        .returning(|| vec![node("b"), node("c")]);

    let coordinator = MeshCoordinator::new(MockMeshTransport::new(), registry);
    let reaction = coordinator
        .on_node_event(NodeEvent::Drop(node("a")))
        .await
        .unwrap();
    assert_eq!(
        reaction,
        MeshReaction::Reconfigured {
            live: vec![node("b"), node("c")]
        }
    );
}

// 3 — Up marks the node up, then resyncs by replaying missed entries (FR-7.2).
#[tokio::test]
async fn node_up_marks_up_and_resyncs() {
    let mut seq = Sequence::new();
    let mut registry = MockNodeRegistry::new();
    registry
        .expect_mark_up()
        .times(1)
        .in_sequence(&mut seq)
        .withf(|n: &NodeId| *n == node("a"))
        .return_const(());

    // Resync drains the fabric: one missed entry, then None.
    let calls = AtomicUsize::new(0);
    let mut transport = MockMeshTransport::new();
    transport
        .expect_next_event()
        .times(2)
        .in_sequence(&mut seq) // resync happens AFTER mark_up
        .returning(move || {
            if calls.fetch_add(1, Ordering::SeqCst) == 0 {
                Ok(Some(signed_entry()))
            } else {
                Ok(None)
            }
        });

    let coordinator = MeshCoordinator::new(transport, registry);
    let reaction = coordinator
        .on_node_event(NodeEvent::Up(node("a")))
        .await
        .unwrap();
    match reaction {
        MeshReaction::Resynced { replayed } => assert_eq!(replayed.len(), 1),
        other => panic!("expected Resynced, got {other:?}"),
    }
}

// 4 — each collective signal dispatches to its mitigation behaviour (FR-7.3).
#[tokio::test]
async fn collective_behaviour_dispatch() {
    let coordinator = MeshCoordinator::new(MockMeshTransport::new(), MockNodeRegistry::new());
    assert_eq!(
        coordinator.collective_behaviour(CollectiveSignal::Overload { zone: "z".into() }),
        MeshAction::RebalanceLoad { zone: "z".into() }
    );
    assert_eq!(
        coordinator.collective_behaviour(CollectiveSignal::CascadeRisk {
            path: vec![JunctionId(7), JunctionId(12)]
        }),
        MeshAction::ShedAndReroute {
            path: vec![JunctionId(7), JunctionId(12)]
        }
    );
    assert_eq!(
        coordinator.collective_behaviour(CollectiveSignal::HeatWave),
        MeshAction::EmergencyRouting
    );
}
