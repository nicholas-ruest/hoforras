//! MCP tool-invocation latency benchmark (FR-8.1).
//!
//! Measures one `ThermalBridgeServer::invoke` over the real `RufloMcpAdapter` registry plus a
//! backing service — the OHS hot path (registry `has` check + one-service delegation). CPU-only
//! reference; the real ruflo MCP transport is a Phase C figure.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hoforras_mesh::ai::{RufloMcpAdapter, ThermalBridgeServer};
use serde_json::json;

mod svc {
    use async_trait::async_trait;
    use hoforras_domain::Json;
    use hoforras_ports::ai::{
        AnomalyQueryService, DistrictStatusService, ForecastQueryService, PipeHealthService,
        TradeQueryService,
    };
    use hoforras_ports::PortResult;
    use serde_json::json;

    pub struct Status;
    #[async_trait]
    impl DistrictStatusService for Status {
        async fn snapshot(&self) -> PortResult<Json> {
            Ok(json!({"district": "XIII", "online": true}))
        }
    }
    macro_rules! empty_svc {
        ($name:ident, $trait:ident, $method:ident) => {
            pub struct $name;
            #[async_trait]
            impl $trait for $name {
                async fn $method(&self) -> PortResult<Json> {
                    Ok(json!({}))
                }
            }
        };
    }
    empty_svc!(Trade, TradeQueryService, active);
    empty_svc!(Pipe, PipeHealthService, scores);

    pub struct AnomalyQ;
    #[async_trait]
    impl AnomalyQueryService for AnomalyQ {
        async fn recent(&self, _a: Json) -> PortResult<Json> {
            Ok(json!({}))
        }
    }
    pub struct Forecast;
    #[async_trait]
    impl ForecastQueryService for Forecast {
        async fn demand(&self, _a: Json) -> PortResult<Json> {
            Ok(json!({}))
        }
    }
}

fn bench_invoke(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let bridge = ThermalBridgeServer::new(
        RufloMcpAdapter::new(),
        svc::Status,
        svc::Trade,
        svc::Pipe,
        svc::AnomalyQ,
        svc::Forecast,
    );
    rt.block_on(async { bridge.boot().await.unwrap() });

    c.bench_function("mcp_tool_invoke(district_status)", |b| {
        b.iter(|| {
            rt.block_on(async {
                black_box(
                    bridge
                        .invoke(black_box("hoforras.thermal.district_status"), json!({}))
                        .await
                        .unwrap(),
                )
            })
        });
    });
}

criterion_group!(benches, bench_invoke);
criterion_main!(benches);
