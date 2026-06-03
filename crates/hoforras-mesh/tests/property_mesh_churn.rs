//! Self-heal-without-state-loss property (NFR-9 / FR-7.2 / DDD-07 §4).
//!
//! For ANY sequence of committed DAG entries, a `Drop(n)` followed by `Up(n)` must leave the
//! survivors' committed state unchanged AND let the returning node converge by replaying exactly the
//! committed entries (eventual consistency over the DAG). Driven against the real
//! `SynapticMeshAdapter` — the same `Arc`-shared fabric is the coordinator's transport and registry.

use hoforras_domain::{
    DagEntry, JunctionId, MlDsaSignature, NodeId, ThermalTradeAgreement, Timestamp,
};
use hoforras_mesh::coordination::{MeshCoordinator, MeshReaction, SynapticMeshAdapter};
use hoforras_ports::mesh::MeshTransport;
use proptest::prelude::*;

fn node(s: &str) -> NodeId {
    NodeId::new(format!("{s}.thermal.budapest.dark")).unwrap()
}

fn signed_entry(i: usize) -> DagEntry {
    DagEntry {
        agreement: ThermalTradeAgreement {
            seller: node("a"),
            buyer: node("b"),
            kwh_offered: i as f32,
            duration_hours: 6,
            credit_price_per_kwh: 2.0,
            pipe_route: vec![JunctionId(i as u32)],
            valid_from: Timestamp(i as u64),
        },
        signature: Some(MlDsaSignature(vec![i as u8])),
    }
}

proptest! {
    /// NFR-9: committed state survives churn and the returning node converges to it.
    #[test]
    fn prop_self_heal_preserves_committed_and_converges(n in 1usize..40) {
        let rt = tokio::runtime::Builder::new_current_thread().build().unwrap();
        let mesh = SynapticMeshAdapter::new();

        rt.block_on(async {
            // Commit n entries across the district.
            for i in 0..n {
                mesh.publish(signed_entry(i)).await.unwrap();
            }
            let committed_before = mesh.committed();

            let coordinator = MeshCoordinator::new(mesh.clone(), mesh.clone());

            // A peer drops …
            let drop_reaction = coordinator
                .on_node_event(hoforras_domain::NodeEvent::Drop(node("a")))
                .await
                .unwrap();
            let reconfigured = matches!(drop_reaction, MeshReaction::Reconfigured { .. });
            prop_assert!(reconfigured);

            // Committed state at survivors is untouched by the drop.
            prop_assert_eq!(mesh.committed(), committed_before.clone());

            // … and returns: it resyncs by replaying every missed entry (converges).
            let up_reaction = coordinator
                .on_node_event(hoforras_domain::NodeEvent::Up(node("a")))
                .await
                .unwrap();
            match up_reaction {
                MeshReaction::Resynced { replayed } => {
                    prop_assert_eq!(replayed, committed_before.clone()); // converged to committed state
                }
                other => prop_assert!(false, "expected Resynced, got {:?}", other),
            }

            // No committed state was lost across the whole churn cycle.
            prop_assert_eq!(mesh.committed(), committed_before);
            Ok(())
        })?;
    }
}
