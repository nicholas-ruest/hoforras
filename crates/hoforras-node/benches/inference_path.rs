//! Single-building inference benchmark (NFR-1, <100 ms budget).
//!
//! NFR-1 requires a single prediction to complete in <100 ms. The production path runs ruv-FANN
//! LSTM/N-BEATS as WASM on the Appliance; that substrate is unavailable here, so this measures the
//! **reference** NeuroDivergent network behind the same port. The measured numbers WILL differ from
//! Appliance hardware — NFR-1 is confirmed in Phase C on target hardware (ADR-0012), and reported
//! honestly here per spec risk R-3. The full spawn→infer→dissolve path (FR-2.1) is timed, not just
//! the model, so the ephemeral-agent overhead is included.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hoforras_domain::{
    Ed25519Signature, NodeId, QualityScore, ReadingWindow, ThermalFrame, Timestamp,
    ValidationStatus,
};
use hoforras_node::forecast::{ForecastServiceImpl, RuvSwarmAdapter};
use hoforras_ports::inference::ForecastService;

fn window_with_frames(n: usize) -> ReadingWindow {
    let node = NodeId::new("pozsonyi14.thermal.budapest.dark").unwrap();
    let frames = (0..n)
        .map(|i| ThermalFrame {
            node_id: node.clone(),
            timestamp_ns: i as u64 * 900_000_000_000, // ~15-min cadence
            temperature_celsius: 18.0 + (i % 7) as f32,
            pipe_vibration_hz: 4.0 + (i % 3) as f32,
            fluid_pressure_bar: 3.0 - (i % 5) as f32 * 0.1,
            ground_thermal_gradient: 0.1,
            quality_score: QualityScore(1.0),
            validation: ValidationStatus::Valid,
            witness: Ed25519Signature([0u8; 64]),
        })
        .collect();
    ReadingWindow {
        node_id: node,
        from_ts: Timestamp(0),
        to_ts: Timestamp(n as u64 * 900_000_000_000),
        frames,
    }
}

fn bench_inference(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    // 24h of readings at a 15-minute cadence — a realistic single-building window.
    let window = window_with_frames(96);

    c.bench_function("forecast_demand_24h(spawn+infer+dissolve)", |b| {
        b.iter(|| {
            let svc = ForecastServiceImpl::new(RuvSwarmAdapter::new());
            rt.block_on(async {
                black_box(svc.demand_24h(black_box(window.clone())).await.unwrap())
            })
        });
    });

    c.bench_function("forecast_anomalies(spawn+infer+dissolve)", |b| {
        b.iter(|| {
            let svc = ForecastServiceImpl::new(RuvSwarmAdapter::new());
            rt.block_on(async {
                black_box(svc.anomalies(black_box(window.clone())).await.unwrap())
            })
        });
    });

    c.bench_function("forecast_burst_precursor(spawn+infer+dissolve)", |b| {
        b.iter(|| {
            let svc = ForecastServiceImpl::new(RuvSwarmAdapter::new());
            rt.block_on(async {
                black_box(
                    svc.burst_precursor(black_box(window.clone()))
                        .await
                        .unwrap(),
                )
            })
        });
    });
}

criterion_group!(benches, bench_inference);
criterion_main!(benches);
