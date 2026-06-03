//! Trade-finality benchmark (NFR-2, <1s finality).
//!
//! NFR-2 requires QR-Avalanche consensus to finalize a trade in under a second. Real finality is a
//! network/consensus figure measured on a **live multi-node mesh** in Phase C — it cannot be
//! produced here (single process, no peers). What this measures is the **local** cost on the
//! finalize path that the Appliance owns: ML-DSA-65 signing of the canonical bytes plus ML-KEM-1024
//! transport key agreement. That local cost is the Appliance's slice of the <1s budget; the
//! consensus round-trip is added on the live mesh. Reported honestly per spec risk R-1/NFR-2.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hoforras_domain::{JunctionId, NodeId, ThermalTradeAgreement, Timestamp};
use hoforras_mesh::consensus::{ConsensusGateway, QuDagAdapter};

fn agreement() -> ThermalTradeAgreement {
    ThermalTradeAgreement {
        seller: NodeId::new("pozsonyi14.thermal.budapest.dark").unwrap(),
        buyer: NodeId::new("pozsonyi22.thermal.budapest.dark").unwrap(),
        kwh_offered: 40.0,
        duration_hours: 6,
        credit_price_per_kwh: 2.3,
        pipe_route: vec![JunctionId(7), JunctionId(12), JunctionId(18)],
        valid_from: Timestamp(0),
    }
}

fn bench_finality(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let gw = ConsensusGateway::new(
        QuDagAdapter::new().unwrap(),
        QuDagAdapter::new().unwrap(),
        QuDagAdapter::new().unwrap(),
    );

    c.bench_function("finalize_local_path(ml-dsa-sign + ml-kem-encap)", |b| {
        b.iter(|| {
            rt.block_on(async { black_box(gw.finalize(black_box(agreement())).await.unwrap()) })
        });
    });
}

criterion_group!(benches, bench_finality);
criterion_main!(benches);
