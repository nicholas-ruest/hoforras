//! R11 — `ThermalBridgeServer` + `AnomalyExplainer` interaction suite (FR-8.1/8.2).
//!
//! The bridge is an Open Host Service: `boot` registers exactly five tools, each tool delegates to
//! exactly one backing service, and an unknown tool errors with NO delegation. The explainer
//! explains via Claude before surfacing to the operator.

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use hoforras_domain::{Anomaly, AnomalyContext, AnomalyKind, Explanation, NodeId, UiEvent};
use hoforras_mesh::ai::{AnomalyExplainer, ThermalBridgeServer, TOOL_NAMES};
use hoforras_ports::ai::{
    MockAnomalyQueryService, MockClaudeReasoner, MockDistrictStatusService,
    MockForecastQueryService, MockOperatorChannel, MockPipeHealthService, MockTradeQueryService,
};
use mockall::Sequence;
use serde_json::json;

/// A registry mock whose `has` reflects what was `register`ed (so `boot` then `invoke` is coherent).
fn registry_tracking() -> hoforras_ports::ai::MockMcpToolRegistry {
    use hoforras_ports::ai::MockMcpToolRegistry;
    let registered = Arc::new(Mutex::new(HashSet::<String>::new()));
    let mut reg = MockMcpToolRegistry::new();
    let on_register = registered.clone();
    reg.expect_register().returning(move |name| {
        on_register.lock().unwrap().insert(name);
        Ok(())
    });
    let on_has = registered.clone();
    reg.expect_has()
        .returning(move |name| on_has.lock().unwrap().contains(name));
    reg
}

fn all_services() -> (
    MockDistrictStatusService,
    MockTradeQueryService,
    MockPipeHealthService,
    MockAnomalyQueryService,
    MockForecastQueryService,
) {
    (
        MockDistrictStatusService::new(),
        MockTradeQueryService::new(),
        MockPipeHealthService::new(),
        MockAnomalyQueryService::new(),
        MockForecastQueryService::new(),
    )
}

// 1 — boot registers exactly the five named tools (FR-8.1).
#[tokio::test]
async fn boot_registers_five_tools() {
    use hoforras_ports::ai::MockMcpToolRegistry;
    let seen = Arc::new(Mutex::new(Vec::<String>::new()));
    let mut reg = MockMcpToolRegistry::new();
    let on_register = seen.clone();
    reg.expect_register().times(5).returning(move |name| {
        on_register.lock().unwrap().push(name);
        Ok(())
    });
    let (st, tr, pi, an, fo) = all_services();
    let bridge = ThermalBridgeServer::new(reg, st, tr, pi, an, fo);

    bridge.boot().await.unwrap();
    let registered = seen.lock().unwrap().clone();
    assert_eq!(registered.len(), 5);
    for name in TOOL_NAMES {
        assert!(registered.contains(&name.to_string()), "missing {name}");
    }
}

// 2 — each tool delegates to exactly its one backing service (FR-8.1).
#[tokio::test]
async fn each_tool_delegates_to_its_service() {
    let mut st = MockDistrictStatusService::new();
    st.expect_snapshot()
        .times(1)
        .returning(|| Ok(json!({"ok": "status"})));
    let mut tr = MockTradeQueryService::new();
    tr.expect_active().times(0); // not this call
    let pi = MockPipeHealthService::new();
    let an = MockAnomalyQueryService::new();
    let fo = MockForecastQueryService::new();

    let bridge = ThermalBridgeServer::new(registry_tracking(), st, tr, pi, an, fo);
    bridge.boot().await.unwrap();

    let out = bridge
        .invoke("hoforras.thermal.district_status", json!({}))
        .await
        .unwrap();
    assert_eq!(out, json!({"ok": "status"}));
}

// 2b — the anomaly tool delegates only to the anomaly service (a second one-to-one check).
#[tokio::test]
async fn anomaly_tool_delegates_to_anomaly_service() {
    let st = MockDistrictStatusService::new();
    let tr = MockTradeQueryService::new();
    let pi = MockPipeHealthService::new();
    let mut an = MockAnomalyQueryService::new();
    an.expect_recent()
        .times(1)
        .returning(|_| Ok(json!([{"node": "x"}])));
    let fo = MockForecastQueryService::new();

    let bridge = ThermalBridgeServer::new(registry_tracking(), st, tr, pi, an, fo);
    bridge.boot().await.unwrap();

    let out = bridge
        .invoke("hoforras.anomaly.recent", json!({"since": 0}))
        .await
        .unwrap();
    assert_eq!(out, json!([{"node": "x"}]));
}

// 3 — an unknown tool errors with NO delegation (FR-8.1).
#[tokio::test]
async fn unknown_tool_errors() {
    let mut st = MockDistrictStatusService::new();
    st.expect_snapshot().times(0); // never delegated to
    let (_, tr, pi, an, fo) = all_services();
    let bridge = ThermalBridgeServer::new(registry_tracking(), st, tr, pi, an, fo);
    bridge.boot().await.unwrap();

    let result = bridge.invoke("hoforras.nope.unknown", json!({})).await;
    assert!(result.is_err());
}

// 4 — explain THEN push (FR-8.2).
#[tokio::test]
async fn explain_then_push() {
    let mut seq = Sequence::new();
    let mut claude = MockClaudeReasoner::new();
    claude
        .expect_explain()
        .times(1)
        .in_sequence(&mut seq)
        .returning(|_| {
            Ok(Explanation {
                text: "pipe stress rising".into(),
            })
        });
    let mut operator = MockOperatorChannel::new();
    operator
        .expect_push()
        .times(1)
        .in_sequence(&mut seq) // push AFTER explain
        .withf(|e| matches!(e, UiEvent::AnomalyExplained { .. }))
        .returning(|_| Ok(()));

    let explainer = AnomalyExplainer::new(claude, operator);
    let ctx = AnomalyContext {
        node_id: NodeId::new("pozsonyi14.thermal.budapest.dark").unwrap(),
        anomaly: Anomaly {
            kind: AnomalyKind::BurstPrecursor,
            confidence: 0.8,
        },
    };
    explainer.explain_and_surface(ctx).await.unwrap();
}
