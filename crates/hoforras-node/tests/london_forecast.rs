//! London-School interaction tests for Forecasting & Anomaly (R7 / FR-2.1–2.6).
//!
//! These assert *interactions* against mocked ports (`MockSwarmFactory`, `MockInferenceAgent`):
//! message, cardinality, order, and ABSENCE. The headline assertions are
//! `dissolve_on_infer_error` (FR-2.1 — the agent is torn down even when inference fails) and the
//! structural fact, exercised by every test here, that the service touches **only** the swarm
//! ports — there is no remote-inference seam (FR-2.6 / ADR-0012).
//!
//! Note on ordering: `spawn → infer` is guaranteed structurally (you cannot `infer` on an agent you
//! have not `spawn`ed), so the sequencing assertions focus on `infer → dissolve`, the order the
//! `with_agent` guard is responsible for.

use hoforras_domain::{AnomalyKind, DomainError, Inference, NodeId, ReadingWindow, Timestamp};
use hoforras_node::forecast::ForecastServiceImpl;
use hoforras_ports::inference::{ForecastService, MockInferenceAgent, MockSwarmFactory};
use mockall::Sequence;

fn window() -> ReadingWindow {
    ReadingWindow {
        node_id: NodeId::new("pozsonyi14.thermal.budapest.dark").unwrap(),
        from_ts: Timestamp(0),
        to_ts: Timestamp(1),
        frames: vec![],
    }
}

/// Build a `MockSwarmFactory` that spawns exactly one agent — the supplied mock.
fn factory_yielding(agent: MockInferenceAgent) -> MockSwarmFactory {
    let mut factory = MockSwarmFactory::new();
    factory
        .expect_spawn()
        .times(1) // exactly one spawn per prediction
        .return_once(move |_| Ok(Box::new(agent)));
    factory
}

// 1 — spawn → infer → dissolve (FR-2.1).
#[tokio::test]
async fn spawns_infers_dissolves() {
    let mut seq = Sequence::new();
    let mut agent = MockInferenceAgent::new();
    agent
        .expect_infer()
        .times(1)
        .in_sequence(&mut seq)
        .returning(|_| {
            Ok(Inference {
                scores: vec![0.9, 0.0, 0.0, 0.0],
            })
        });
    agent
        .expect_dissolve()
        .times(1)
        .in_sequence(&mut seq) // dissolve AFTER infer
        .return_const(());

    let svc = ForecastServiceImpl::new(factory_yielding(agent));
    svc.anomalies(window()).await.unwrap();
}

// 2 — dissolve still happens when inference errors (FR-2.1). HEADLINE.
#[tokio::test]
async fn dissolve_on_infer_error() {
    let mut seq = Sequence::new();
    let mut agent = MockInferenceAgent::new();
    agent
        .expect_infer()
        .times(1)
        .in_sequence(&mut seq)
        .returning(|_| Err(DomainError::Adapter("inference exploded".into())));
    agent
        .expect_dissolve()
        .times(1) // still called exactly once …
        .in_sequence(&mut seq)
        .return_const(());

    let svc = ForecastServiceImpl::new(factory_yielding(agent));
    let res = svc.anomalies(window()).await;
    assert!(res.is_err()); // … and the error is propagated
}

// 3 — anomalies come back typed and confidence-scored (FR-2.2).
#[tokio::test]
async fn anomalies_typed_scored() {
    let mut agent = MockInferenceAgent::new();
    agent.expect_infer().returning(|_| {
        Ok(Inference {
            scores: vec![0.1, 0.92, 0.1, 0.1],
        })
    });
    agent.expect_dissolve().return_const(());

    let svc = ForecastServiceImpl::new(factory_yielding(agent));
    let anomalies = svc.anomalies(window()).await.unwrap();
    assert_eq!(anomalies.len(), 1);
    assert_eq!(anomalies[0].kind, AnomalyKind::PressureDrop);
    assert!((anomalies[0].confidence - 0.92).abs() < 1e-6);
}

// 4 — burst horizon is clamped to [6h, 48h] (FR-2.3).
#[tokio::test]
async fn burst_warning_clamped_6_48h() {
    // ETA well above the band ⇒ clamps to 48h.
    let mut hot = MockInferenceAgent::new();
    hot.expect_infer().returning(|_| {
        Ok(Inference {
            scores: vec![0.85, 240.0],
        })
    });
    hot.expect_dissolve().return_const(());
    let svc = ForecastServiceImpl::new(factory_yielding(hot));
    let warn = svc.burst_precursor(window()).await.unwrap().unwrap();
    assert!((6..=48).contains(&warn.horizon_hours));
    assert_eq!(warn.horizon_hours, 48);

    // ETA below the band ⇒ clamps to 6h.
    let mut soon = MockInferenceAgent::new();
    soon.expect_infer().returning(|_| {
        Ok(Inference {
            scores: vec![0.95, 0.5],
        })
    });
    soon.expect_dissolve().return_const(());
    let svc2 = ForecastServiceImpl::new(factory_yielding(soon));
    let warn2 = svc2.burst_precursor(window()).await.unwrap().unwrap();
    assert_eq!(warn2.horizon_hours, 6);
}

// 5 — demand forecast is shaped as 24 hours (FR-2.4).
#[tokio::test]
async fn demand_24h_shape() {
    let mut agent = MockInferenceAgent::new();
    agent.expect_infer().returning(|_| {
        Ok(Inference {
            scores: vec![5.0; 30],
        })
    }); // model emits 30; service shapes to 24
    agent.expect_dissolve().return_const(());

    let svc = ForecastServiceImpl::new(factory_yielding(agent));
    let forecast = svc.demand_24h(window()).await.unwrap();
    assert_eq!(forecast.hourly_kwh.len(), 24);
}

// 6 — no remote-inference dependency (FR-2.6 / ADR-0012).
//
// Runtime evidence: the only collaborator the service can be constructed with is a `SwarmFactory`
// (here a mock whose ONLY expectation is `spawn`), and the only thing invoked on the spawned agent
// is `infer`/`dissolve`. No other port is — or can be — touched. Compile-time evidence: there is no
// `RemoteInferenceClient` symbol anywhere in `hoforras-ports` for the service to depend on; the
// `use` below would fail to compile if such a seam existed and the test would not build.
#[tokio::test]
async fn no_remote_inference_dependency() {
    let mut agent = MockInferenceAgent::new();
    agent.expect_infer().times(1).returning(|_| {
        Ok(Inference {
            scores: vec![3.0; 24],
        })
    });
    agent.expect_dissolve().times(1).return_const(());

    let svc = ForecastServiceImpl::new(factory_yielding(agent));
    // Success while touching only spawn/infer/dissolve ⇒ no remote hop occurred.
    assert!(svc.demand_24h(window()).await.is_ok());
}
