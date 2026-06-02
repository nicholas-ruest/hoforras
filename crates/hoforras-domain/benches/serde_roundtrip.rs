//! Benchmark trivial canonical-bytes (postcard) roundtrips (ADR-0014).
//!
//! Run with `cargo bench -p hoforras-domain`. Confirms the canonical encoding/decoding of the core
//! signed type (`ThermalTradeAgreement`) is fast and stable.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hoforras_domain::{canonical_bytes, JunctionId, NodeId, ThermalTradeAgreement, Timestamp};

fn sample_agreement() -> ThermalTradeAgreement {
    ThermalTradeAgreement {
        seller: NodeId::new("pozsonyi14.thermal.budapest.dark").unwrap(),
        buyer: NodeId::new("pozsonyi22.thermal.budapest.dark").unwrap(),
        kwh_offered: 40.0,
        duration_hours: 6,
        credit_price_per_kwh: 2.3,
        pipe_route: vec![JunctionId(7), JunctionId(12), JunctionId(18)],
        valid_from: Timestamp(1_700_000_000_000_000_000),
    }
}

fn bench_roundtrip(c: &mut Criterion) {
    let agreement = sample_agreement();

    c.bench_function("canonical_bytes(ThermalTradeAgreement)", |b| {
        b.iter(|| canonical_bytes(black_box(&agreement)).unwrap());
    });

    let bytes = canonical_bytes(&agreement).unwrap();
    c.bench_function("postcard_decode(ThermalTradeAgreement)", |b| {
        b.iter(|| {
            let decoded: ThermalTradeAgreement = postcard::from_bytes(black_box(&bytes)).unwrap();
            black_box(decoded);
        });
    });
}

criterion_group!(benches, bench_roundtrip);
criterion_main!(benches);
