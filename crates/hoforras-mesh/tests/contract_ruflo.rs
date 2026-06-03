//! Contract tests for the AI-orchestration adapters (R11 contract) + the deferred federation stub.
//!
//! Exercises the real `RufloMcpAdapter` (register/has/list/invoke + Claude reasoning) and
//! `WebSocketSink` end-to-end through `ThermalBridgeServer`/`AnomalyExplainer` — no mocks. Federation
//! is asserted to be a disclosed stub that accepts only market signals (FR-8.3 deferred, spec R-5).

use hoforras_domain::{Anomaly, AnomalyContext, AnomalyKind, NodeId, UiEvent};
use hoforras_mesh::ai::{
    AnomalyExplainer, FederationOutcome, FederationStub, RufloMcpAdapter, ThermalBridgeServer,
    WebSocketSink, TOOL_NAMES,
};
use hoforras_ports::ai::McpToolRegistry;
use serde_json::json;

// Minimal real backing services returning canonical Json.
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
    pub struct Trade;
    #[async_trait]
    impl TradeQueryService for Trade {
        async fn active(&self) -> PortResult<Json> {
            Ok(json!([]))
        }
    }
    pub struct Pipe;
    #[async_trait]
    impl PipeHealthService for Pipe {
        async fn scores(&self) -> PortResult<Json> {
            Ok(json!({"junction_7": 0.95}))
        }
    }
    pub struct AnomalyQ;
    #[async_trait]
    impl AnomalyQueryService for AnomalyQ {
        async fn recent(&self, _args: Json) -> PortResult<Json> {
            Ok(json!([]))
        }
    }
    pub struct Forecast;
    #[async_trait]
    impl ForecastQueryService for Forecast {
        async fn demand(&self, _args: Json) -> PortResult<Json> {
            Ok(json!({"hourly_kwh": []}))
        }
    }
}

fn bridge() -> ThermalBridgeServer<
    RufloMcpAdapter,
    svc::Status,
    svc::Trade,
    svc::Pipe,
    svc::AnomalyQ,
    svc::Forecast,
> {
    ThermalBridgeServer::new(
        RufloMcpAdapter::new(),
        svc::Status,
        svc::Trade,
        svc::Pipe,
        svc::AnomalyQ,
        svc::Forecast,
    )
}

#[tokio::test]
async fn ruflo_registers_exactly_five_tools_and_routes() {
    let bridge = bridge();
    bridge.boot().await.unwrap();

    // Each of the five tools routes to its backing service and returns its Json.
    for name in TOOL_NAMES {
        let out = bridge.invoke(name, json!({})).await.unwrap();
        assert!(!out.is_null(), "{name} returned null");
    }
    // An unknown tool errors.
    assert!(bridge.invoke("hoforras.unknown", json!({})).await.is_err());
}

#[tokio::test]
async fn ruflo_adapter_register_has_list_invoke() {
    let adapter = RufloMcpAdapter::new();
    for name in TOOL_NAMES {
        adapter.register(name.to_string()).await.unwrap();
    }
    assert_eq!(adapter.list_tools().len(), 5);
    assert!(adapter.has("hoforras.pipe.health_scores"));
    // The adapter's MCP-facing invoke acknowledges a registered tool and errors on an unknown one.
    assert!(adapter
        .invoke("hoforras.pipe.health_scores".into(), json!({}))
        .await
        .is_ok());
    assert!(adapter.invoke("missing".into(), json!({})).await.is_err());
}

#[tokio::test]
async fn explainer_pushes_explained_event_over_websocket() {
    let claude = RufloMcpAdapter::new(); // also a ClaudeReasoner
    let sink = WebSocketSink::new();
    // Borrow the sink through the explainer, then inspect what was pushed: use an Arc.
    let sink = std::sync::Arc::new(sink);
    // The explainer needs ownership of an OperatorChannel; wrap the Arc in a thin forwarder.
    struct ArcSink(std::sync::Arc<WebSocketSink>);
    #[async_trait::async_trait]
    impl hoforras_ports::ai::OperatorChannel for ArcSink {
        async fn push(&self, event: UiEvent) -> hoforras_ports::PortResult<()> {
            self.0.push(event).await
        }
    }
    let explainer = AnomalyExplainer::new(claude, ArcSink(sink.clone()));
    let ctx = AnomalyContext {
        node_id: NodeId::new("pozsonyi14.thermal.budapest.dark").unwrap(),
        anomaly: Anomaly {
            kind: AnomalyKind::ThermalSpike,
            confidence: 0.9,
        },
    };
    explainer.explain_and_surface(ctx).await.unwrap();

    let pushed = sink.pushed();
    assert_eq!(pushed.len(), 1);
    assert!(matches!(pushed[0], UiEvent::AnomalyExplained { .. }));
}

#[test]
fn federation_is_a_disclosed_stub_accepting_only_market_signals() {
    let federation = FederationStub::new();
    // A tagged market signal validates and is reported as deferred (R-5).
    assert_eq!(
        federation
            .exchange(&json!({"kind": "market_signal", "price": 2.3}))
            .unwrap(),
        FederationOutcome::Deferred
    );
    // Anything that is not a market signal — including any raw payload — is rejected.
    assert!(federation.exchange(&json!({"kind": "raw_frame"})).is_err());
    assert!(federation
        .exchange(&json!({"temperature_celsius": 88.0}))
        .is_err());
}
