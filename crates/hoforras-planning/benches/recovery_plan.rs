//! Recovery-planning benchmark (ADR-0016) — the advisory plan is computed off the operational path,
//! so its budget is generous; this confirms a district-scale plan is sub-millisecond.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hoforras_domain::{DistrictGraph, JunctionId, LossCause, NodeId, ThermalLossSite};
use hoforras_planning::RecoveryPlannerService;
use hoforras_ports::recovery::RecoveryPlanner;

fn losses(n: usize) -> Vec<ThermalLossSite> {
    (0..n)
        .map(|i| ThermalLossSite {
            node_id: NodeId::new(format!("b{}.thermal.budapest.dark", i % 10)).unwrap(),
            junction: JunctionId(i as u32),
            estimated_loss_kwh_yr: 500.0 + (i % 50) as f32 * 200.0,
            confidence: 0.3 + (i % 7) as f32 * 0.1,
            cause: LossCause::PipeInsulation,
        })
        .collect()
}

fn bench_plan(c: &mut Criterion) {
    let planner = RecoveryPlannerService::default();
    let graph = DistrictGraph {
        nodes: (0..10)
            .map(|i| NodeId::new(format!("b{i}.thermal.budapest.dark")).unwrap())
            .collect(),
        edges: vec![],
    };
    let sites = losses(200);

    c.bench_function("recovery_plan_200_sites", |b| {
        b.iter(|| black_box(planner.plan(black_box(sites.clone()), &graph).unwrap()));
    });
}

criterion_group!(benches, bench_plan);
criterion_main!(benches);
