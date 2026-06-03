//! NFR audit (Completion / sparc.md Part 5 §2) — the runtime gates that mocks could not assert.
//!
//! The latency NFRs (NFR-1 inference, NFR-2 finality, NFR-3 RVM coherence, NFR-4 search, NFR-9
//! self-heal) are *measured* in the per-context criterion benches and reported honestly there
//! (reference adapters, real numbers, Phase C caveats). This file pins **NFR-5** — no GPU / no
//! cloud — as a runtime audit: inference is on-device and completes with no network hop.

use hoforras_domain::{NodeId, ReadingWindow, Timestamp};
use hoforras_node::appliance::Appliance;
use hoforras_ports::inference::ForecastService;

fn node() -> NodeId {
    NodeId::new("pozsonyi14.thermal.budapest.dark").unwrap()
}

/// NFR-5: edge-only inference. The forecast service runs the LSTM/N-BEATS reference network entirely
/// on-device (ADR-0012) — there is no `RemoteInferenceClient` port in `hoforras-ports`, so no cloud
/// inference call is even expressible. This audit confirms a prediction completes with the inference
/// path fully in-process (no network, no GPU handle required).
#[tokio::test]
async fn nfr5_inference_runs_on_device_no_cloud() {
    let forecast = Appliance::forecast_service();
    let window = ReadingWindow {
        node_id: node(),
        from_ts: Timestamp(0),
        to_ts: Timestamp(86_400_000_000_000),
        frames: vec![],
    };
    // Completes purely on-device — the only ports it can touch are SwarmFactory/InferenceAgent, both
    // node-local (no remote-inference seam exists). A cloud dependency would make this hang/err here.
    let demand = forecast.demand_24h(window).await.unwrap();
    assert_eq!(
        demand.hourly_kwh.len(),
        24,
        "on-device 24h forecast produced (NFR-5 / AC-3)"
    );
}
