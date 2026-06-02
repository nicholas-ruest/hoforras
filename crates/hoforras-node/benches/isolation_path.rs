//! Isolation-path benchmark (NFR-3).
//!
//! Vendor (RVM) targets from research §94: partition switch ~6 ns, 16-node mincut ~331 ns,
//! witness emit ~17 ns. These are **reference in-process implementations** (the real `rvm-*`
//! substrate is unavailable here), so the measured numbers WILL differ — reported honestly per
//! ADR-0004 / spec risk R-3. In particular `witness emit` uses SHA-512 over canonical bytes, which
//! is inherently slower than the RVM witness primitive; that is expected and disclosed.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hoforras_domain::{DistrictGraph, JunctionId, NodeId, PrivilegedAction};
use hoforras_node::isolation::adapters::{RvmCoherenceAdapter, RvmWitnessAdapter};
use hoforras_ports::security::{MincutEngine, PartitionController, WitnessChain};

fn graph_16() -> DistrictGraph {
    let nodes: Vec<NodeId> = (0..16)
        .map(|i| NodeId::new(format!("n{i}.thermal.budapest.dark")).unwrap())
        .collect();
    let edges = (0..16)
        .map(|i| (JunctionId(i), JunctionId((i + 1) % 16), (i as f32) + 1.0))
        .collect();
    DistrictGraph { nodes, edges }
}

fn bench_isolation(c: &mut Criterion) {
    let coherence = RvmCoherenceAdapter::new();
    let graph = graph_16();
    let node = NodeId::new("n0.thermal.budapest.dark").unwrap();

    // Partition switch (vendor target ~6 ns).
    c.bench_function("partition_switch(isolate)", |b| {
        b.iter(|| coherence.isolate(black_box(node.clone())));
    });

    // 16-node mincut (vendor target ~331 ns).
    c.bench_function("mincut_recompute_16_nodes", |b| {
        b.iter(|| black_box(coherence.recompute(black_box(&graph))));
    });

    // Witness emit (vendor target ~17 ns; SHA-512 reference is slower — disclosed).
    let witness = RvmWitnessAdapter::new();
    c.bench_function("witness_emit", |b| {
        b.iter(|| black_box(witness.emit(black_box(PrivilegedAction::Exec)).unwrap()));
    });
}

criterion_group!(benches, bench_isolation);
criterion_main!(benches);
