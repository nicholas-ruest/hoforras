//! Similarity-search benchmark (NFR-4, sub-millisecond query).
//!
//! NFR-4 requires sub-ms top-K similarity search. Production uses `ruvector` HNSW/DiskANN
//! (`~O(log N)`); that substrate is unavailable here, so this measures the **reference** adapter,
//! which is a linear scan over the point set — it is therefore a conservative upper bound, not the
//! production figure. NFR-4 is confirmed on Appliance hardware with real HNSW in Phase C (ADR-0009
//! keeps this node-local). Reported honestly per spec risk R-1.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hoforras_domain::{
    Ed25519Signature, NodeId, QualityScore, Query, ThermalFrame, ValidationStatus,
};
use hoforras_node::memory::{embed, RuVectorAdapter};
use hoforras_ports::memory::VectorIndex;
use serde_json::json;

const N: usize = 1_000;

fn frame(node: &NodeId, i: usize) -> ThermalFrame {
    ThermalFrame {
        node_id: node.clone(),
        timestamp_ns: i as u64,
        temperature_celsius: 10.0 + (i % 30) as f32,
        pipe_vibration_hz: (i % 20) as f32,
        fluid_pressure_bar: 2.0 + (i % 5) as f32 * 0.3,
        ground_thermal_gradient: (i % 7) as f32 * 0.05,
        quality_score: QualityScore(1.0),
        validation: ValidationStatus::Valid,
        witness: Ed25519Signature([0u8; 64]),
    }
}

fn bench_search(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let idx = RuVectorAdapter::new();

    // Pre-populate N surplus-available points in district XIII.
    rt.block_on(async {
        for i in 0..N {
            let node = NodeId::new(format!("b{i}.thermal.budapest.dark")).unwrap();
            let vector = embed(&frame(&node, i)).unwrap();
            idx.upsert(
                node,
                vector,
                json!({"kind":"state","thermal_surplus_available":true,"district":"XIII","timestamp_ns":i}),
            )
            .await
            .unwrap();
        }
    });
    assert_eq!(idx.len(), N);

    let probe = embed(&frame(
        &NodeId::new("probe.thermal.budapest.dark").unwrap(),
        7,
    ))
    .unwrap();

    c.bench_function(
        "ruvector_search_top5_over_1000(reference linear scan)",
        |b| {
            b.iter(|| {
                let query = Query {
                    vector: probe.clone(),
                    top_k: 5,
                    surplus_available: true,
                    district: "XIII".into(),
                };
                rt.block_on(async { black_box(idx.search(black_box(query)).await.unwrap()) })
            });
        },
    );
}

criterion_group!(benches, bench_search);
criterion_main!(benches);
