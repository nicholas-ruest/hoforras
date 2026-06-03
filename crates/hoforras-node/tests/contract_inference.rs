//! Contract test for the on-device inference adapter (R7 contract / ADR-0012).
//!
//! The production contract runs an LSTM under `wasm-bindgen-test` and asserts inference completes
//! (latency measured, but NFR-1 `<100 ms` is confirmed in Phase C on Appliance hardware). The
//! ruv-FANN WASM substrate is unavailable here, so this exercises the **reference** model behind the
//! same `SwarmFactory`/`InferenceAgent` ports, asserting the full spawn→infer→dissolve lifecycle
//! produces well-formed, task-appropriate inferences with the real adapter (no mocks).

use hoforras_domain::{
    Ed25519Signature, NodeId, QualityScore, ReadingWindow, ThermalFrame, Timestamp,
    ValidationStatus,
};
use hoforras_node::forecast::adapters::{ForecastTask, NeuroDivergentAdapter, RuvSwarmAdapter};
use hoforras_node::forecast::{ForecastServiceImpl, SPEC_ANOMALY};
use hoforras_ports::inference::{ForecastService, SwarmFactory};

fn frame(temp: f32, vib: f32, pressure: f32) -> ThermalFrame {
    ThermalFrame {
        node_id: NodeId::new("pozsonyi14.thermal.budapest.dark").unwrap(),
        timestamp_ns: 0,
        temperature_celsius: temp,
        pipe_vibration_hz: vib,
        fluid_pressure_bar: pressure,
        ground_thermal_gradient: 0.1,
        quality_score: QualityScore(1.0),
        validation: ValidationStatus::Valid,
        witness: Ed25519Signature([0u8; 64]),
    }
}

fn window(frames: Vec<ThermalFrame>) -> ReadingWindow {
    ReadingWindow {
        node_id: NodeId::new("pozsonyi14.thermal.budapest.dark").unwrap(),
        from_ts: Timestamp(0),
        to_ts: Timestamp(1),
        frames,
    }
}

#[tokio::test]
async fn ruv_swarm_lifecycle_balances_spawn_and_dissolve() {
    let factory = RuvSwarmAdapter::new();
    let agent = factory
        .spawn(
            // anomaly profile
            hoforras_domain::AgentProfile {
                specialization: SPEC_ANOMALY.into(),
                neural_model: "lstm".into(),
                analytical: 0.95,
                systematic: 0.9,
            },
        )
        .await
        .unwrap();
    let inf = agent
        .infer(window(vec![frame(20.0, 4.0, 3.0)]))
        .await
        .unwrap();
    assert!(!inf.scores.is_empty());
    agent.dissolve();
    assert_eq!(factory.spawned(), factory.dissolved());
}

#[tokio::test]
async fn demand_head_returns_24h_through_service() {
    let svc = ForecastServiceImpl::new(RuvSwarmAdapter::new());
    let forecast = svc
        .demand_24h(window(vec![frame(2.0, 3.0, 3.0), frame(3.0, 3.5, 2.9)]))
        .await
        .unwrap();
    assert_eq!(forecast.hourly_kwh.len(), 24);
    assert!(forecast
        .hourly_kwh
        .iter()
        .all(|v| v.is_finite() && *v >= 0.0));
}

#[test]
fn neuro_divergent_heads_have_expected_shapes() {
    let model = NeuroDivergentAdapter::new();
    let w = window(vec![frame(88.0, 40.0, 4.0)]);
    assert_eq!(model.infer(ForecastTask::Anomaly, &w).scores.len(), 4);
    assert_eq!(model.infer(ForecastTask::Burst, &w).scores.len(), 2);
    assert_eq!(model.infer(ForecastTask::Demand, &w).scores.len(), 24);
}
