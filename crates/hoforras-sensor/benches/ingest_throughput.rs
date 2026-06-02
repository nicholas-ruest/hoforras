//! Criterion bench: end-to-end per-frame `ingest()` throughput (FR-1.1).
//!
//! Wires the real ingestion path — `RvcsiIngestAdapter` (validate + score), `Ed25519WitnessAdapter`
//! (real signing), and no-op in-memory `EventEmitter`/`RejectionLog` sinks — and measures frames
//! per second. Reports throughput via `Throughput::Elements(1)`.

use async_trait::async_trait;
use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use hoforras_domain::{
    DomainError, NodeId, QualityScore, RawReading, ThermalFrame, ValidationStatus,
};
use hoforras_ports::sensor::{EventEmitter, QualityScorer, RejectionLog, Validator};
use hoforras_sensor::{Ed25519WitnessAdapter, IngestionPipeline, RvcsiIngestAdapter};
use std::hint::black_box;
use tokio::runtime::Runtime;

/// No-op sink that discards emitted frames (a real bus would carry them node-local).
struct NullEmitter;

#[async_trait]
impl EventEmitter for NullEmitter {
    async fn emit(&self, frame: ThermalFrame) -> Result<(), DomainError> {
        black_box(frame);
        Ok(())
    }
}

/// No-op rejection log.
struct NullRejections;

impl RejectionLog for NullRejections {
    fn record(&self, _raw: RawReading, _reason: String) {}
}

/// Wrap the rvcsi ACL so the pipeline can own it for both `Validator` and `QualityScorer`.
struct RvcsiPorts(RvcsiIngestAdapter);

impl Validator for RvcsiPorts {
    fn validate(&self, raw: &RawReading) -> ValidationStatus {
        self.0.validate(raw)
    }
}

impl QualityScorer for RvcsiPorts {
    fn score(&self, raw: &RawReading) -> QualityScore {
        self.0.score(raw)
    }
}

fn node() -> NodeId {
    NodeId::new("pozsonyi14.thermal.budapest.dark").unwrap()
}

fn reading() -> RawReading {
    RawReading {
        node_id: node(),
        timestamp_ns: 1_000,
        temperature_celsius: 22.0,
        pipe_vibration_hz: 3.5,
        fluid_pressure_bar: 2.1,
        ground_thermal_gradient: 0.25,
    }
}

fn bench_ingest(c: &mut Criterion) {
    let rt = Runtime::new().expect("tokio runtime");

    let pipeline = IngestionPipeline::new(
        RvcsiPorts(RvcsiIngestAdapter::new(node())),
        RvcsiPorts(RvcsiIngestAdapter::new(node())),
        Ed25519WitnessAdapter::from_seed(&[5u8; 32]),
        NullEmitter,
        NullRejections,
    );

    let mut group = c.benchmark_group("ingest");
    group.throughput(Throughput::Elements(1));
    group.bench_function("frame", |b| {
        b.iter(|| {
            rt.block_on(pipeline.ingest(black_box(reading())))
                .expect("ingest ok")
        });
    });
    group.finish();
}

criterion_group!(benches, bench_ingest);
criterion_main!(benches);
