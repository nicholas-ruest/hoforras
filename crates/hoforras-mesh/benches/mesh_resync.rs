//! Self-heal / resync benchmark (NFR-9).
//!
//! Measures the cost for a returning node to resync by replaying missed DAG entries from the fabric.
//! The reference `SynapticMeshAdapter` drains an in-process inbox; on a live Synaptic-Mesh the
//! replay is network-bound and measured in Phase C. Reported honestly — this is the local replay
//! cost, the Appliance's slice of self-heal time.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hoforras_domain::{
    DagEntry, JunctionId, MlDsaSignature, NodeEvent, NodeId, ThermalTradeAgreement, Timestamp,
};
use hoforras_mesh::coordination::{MeshCoordinator, SynapticMeshAdapter};
use hoforras_ports::mesh::MeshTransport;

const MISSED: usize = 100;

fn node() -> NodeId {
    NodeId::new("a.thermal.budapest.dark").unwrap()
}

fn signed_entry(i: usize) -> DagEntry {
    DagEntry {
        agreement: ThermalTradeAgreement {
            seller: node(),
            buyer: NodeId::new("b.thermal.budapest.dark").unwrap(),
            kwh_offered: i as f32,
            duration_hours: 6,
            credit_price_per_kwh: 2.0,
            pipe_route: vec![JunctionId(i as u32)],
            valid_from: Timestamp(i as u64),
        },
        signature: Some(MlDsaSignature(vec![i as u8])),
    }
}

fn bench_resync(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();

    c.bench_function("mesh_resync_replay_100_missed_entries", |b| {
        b.iter(|| {
            // Fresh fabric with MISSED undelivered entries, then a node returns and resyncs.
            let mesh = SynapticMeshAdapter::new();
            rt.block_on(async {
                for i in 0..MISSED {
                    mesh.publish(signed_entry(i)).await.unwrap();
                }
                let coordinator = MeshCoordinator::new(mesh.clone(), mesh.clone());
                black_box(
                    coordinator
                        .on_node_event(black_box(NodeEvent::Up(node())))
                        .await
                        .unwrap(),
                )
            })
        });
    });
}

criterion_group!(benches, bench_resync);
criterion_main!(benches);
