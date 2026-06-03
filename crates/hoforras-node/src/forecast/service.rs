//! `ForecastServiceImpl` (DDD-03 / FR-2.1–2.6) — node-local forecasting over ephemeral agents.
//!
//! Every prediction runs through [`with_agent`](ForecastServiceImpl::with_agent): spawn an
//! ephemeral specialist (`SwarmFactory::spawn`), infer **once**, and **always** dissolve — even
//! when inference fails (FR-2.1). The only ports this unit can touch are `SwarmFactory` and
//! `InferenceAgent`; there is intentionally **no** remote-inference seam (ADR-0012), and both
//! ports are node-local (ADR-0009). The mappers below are pure domain functions and are
//! example-tested.

use async_trait::async_trait;
use hoforras_domain::{
    AgentProfile, Anomaly, AnomalyKind, BurstWarning, DemandForecast, Inference, ReadingWindow,
};
use hoforras_ports::inference::{ForecastService, SwarmFactory};
use hoforras_ports::PortResult;

/// Confidence at/above which an anomaly channel is reported (FR-2.2).
const ANOMALY_CONFIDENCE_THRESHOLD: f32 = 0.5;
/// Precursor score above which a burst warning is raised (FR-2.3).
const BURST_THRESHOLD: f32 = 0.5;
/// Burst-horizon bounds in hours (FR-2.3 — invariant 3 / DDD-03 §4).
const BURST_HORIZON_MIN_H: u32 = 6;
const BURST_HORIZON_MAX_H: u32 = 48;
/// Hours in a demand forecast (FR-2.4).
const DEMAND_HOURS: usize = 24;

/// Specialization tags shared with the model host (`NeuroDivergentAdapter`). The tag selects which
/// reference network runs and therefore the `Inference.scores` layout the mappers below expect, so
/// the service and the adapter must agree on these strings.
pub const SPEC_ANOMALY: &str = "anomaly_detection";
pub const SPEC_BURST: &str = "burst_precursor";
pub const SPEC_DEMAND: &str = "demand_forecasting";

/// Fixed channel order of an anomaly inference (must match `NeuroDivergentAdapter`'s anomaly head).
const ANOMALY_CHANNELS: [AnomalyKind; 4] = [
    AnomalyKind::ThermalSpike,
    AnomalyKind::PressureDrop,
    AnomalyKind::VibrationPattern,
    AnomalyKind::BurstPrecursor,
];

/// Build the spawn profile for a task. Mirrors the ruv-swarm analyst profile from `sparc.md`
/// Part 2 §3 (`analytical: 0.95`, `systematic: 0.9`).
fn profile(specialization: &str, neural_model: &str) -> AgentProfile {
    AgentProfile {
        specialization: specialization.into(),
        neural_model: neural_model.into(),
        analytical: 0.95,
        systematic: 0.9,
    }
}

/// Node-local forecasting service. Generic over its `SwarmFactory` so it is unit-tested against
/// mocks (London-School, ADR-0001). The type signature admits **only** a `SwarmFactory`, which is
/// how edge-only (FR-2.6 / ADR-0012) is structurally enforced: there is no port through which a
/// remote inference backend could be injected.
pub struct ForecastServiceImpl<F> {
    factory: F,
}

impl<F: SwarmFactory> ForecastServiceImpl<F> {
    pub fn new(factory: F) -> Self {
        Self { factory }
    }

    /// Resource guard (FR-2.1): spawn exactly once → infer exactly once → **always** dissolve →
    /// map. The dissolve happens *before* the inference error is propagated, so the spawn/dissolve
    /// pairing holds across every outcome (Ok / Err). This is the single place the agent lifecycle
    /// lives — every public method funnels through it.
    async fn with_agent<T>(
        &self,
        profile: AgentProfile,
        window: ReadingWindow,
        map: impl FnOnce(Inference) -> T,
    ) -> PortResult<T> {
        let agent = self.factory.spawn(profile).await?; // london: spawn(profile) once
        let inference = agent.infer(window).await; // london: infer(window) once; capture Ok/Err
        agent.dissolve(); // london: dissolve ALWAYS, even on Err (FR-2.1)
        Ok(map(inference?)) // Err short-circuits AFTER dissolve has run
    }
}

#[async_trait]
impl<F: SwarmFactory> ForecastService for ForecastServiceImpl<F> {
    async fn anomalies(&self, window: ReadingWindow) -> PortResult<Vec<Anomaly>> {
        self.with_agent(profile(SPEC_ANOMALY, "lstm"), window, map_to_anomalies)
            .await
    }

    async fn burst_precursor(&self, window: ReadingWindow) -> PortResult<Option<BurstWarning>> {
        self.with_agent(profile(SPEC_BURST, "lstm"), window, map_to_burst)
            .await
    }

    async fn demand_24h(&self, window: ReadingWindow) -> PortResult<DemandForecast> {
        self.with_agent(profile(SPEC_DEMAND, "n-beats"), window, map_to_demand)
            .await
    }
}

// ───────────────────────── pure mappers (FR-2.2/2.3/2.4) ─────────────────────────

/// Map per-channel inference scores to typed, confidence-scored anomalies (FR-2.2). A channel is
/// reported only when its confidence reaches the threshold; missing channels score 0.
fn map_to_anomalies(inf: Inference) -> Vec<Anomaly> {
    ANOMALY_CHANNELS
        .iter()
        .enumerate()
        .filter_map(|(i, &kind)| {
            let confidence = inf.scores.get(i).copied().unwrap_or(0.0).clamp(0.0, 1.0);
            (confidence >= ANOMALY_CONFIDENCE_THRESHOLD).then_some(Anomaly { kind, confidence })
        })
        .collect()
}

/// Map `[precursor_score, eta_hours, ..]` to an optional burst warning with a horizon clamped to
/// [6h, 48h] (FR-2.3). Below threshold ⇒ `None`.
fn map_to_burst(inf: Inference) -> Option<BurstWarning> {
    let score = inf.scores.first().copied().unwrap_or(0.0).clamp(0.0, 1.0);
    if score <= BURST_THRESHOLD {
        return None;
    }
    let eta = inf
        .scores
        .get(1)
        .copied()
        .unwrap_or(BURST_HORIZON_MAX_H as f32);
    Some(BurstWarning {
        horizon_hours: clamp_horizon(eta),
        score,
    })
}

/// Clamp a raw ETA (hours) into the [6h, 48h] burst-horizon band (FR-2.3). Non-finite ⇒ max.
fn clamp_horizon(eta_hours: f32) -> u32 {
    let h = if eta_hours.is_finite() {
        eta_hours.round()
    } else {
        BURST_HORIZON_MAX_H as f32
    };
    (h as i64).clamp(BURST_HORIZON_MIN_H as i64, BURST_HORIZON_MAX_H as i64) as u32
}

/// Map inference scores to a 24-hour demand forecast (FR-2.4). Pads with the last value (or 0) and
/// truncates to exactly 24 entries so the shape is invariant regardless of the model head.
fn map_to_demand(inf: Inference) -> DemandForecast {
    let mut hourly_kwh: Vec<f32> = inf.scores.iter().take(DEMAND_HOURS).copied().collect();
    if hourly_kwh.len() < DEMAND_HOURS {
        let pad = hourly_kwh.last().copied().unwrap_or(0.0);
        hourly_kwh.resize(DEMAND_HOURS, pad);
    }
    DemandForecast { hourly_kwh }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anomalies_reports_only_channels_above_threshold() {
        let out = map_to_anomalies(Inference {
            scores: vec![0.9, 0.1, 0.6, 0.0],
        });
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].kind, AnomalyKind::ThermalSpike);
        assert!((out[0].confidence - 0.9).abs() < f32::EPSILON);
        assert_eq!(out[1].kind, AnomalyKind::VibrationPattern);
    }

    #[test]
    fn anomalies_empty_when_all_below_threshold() {
        let out = map_to_anomalies(Inference {
            scores: vec![0.1, 0.2, 0.3, 0.4],
        });
        assert!(out.is_empty());
    }

    #[test]
    fn burst_none_below_threshold() {
        assert!(map_to_burst(Inference {
            scores: vec![0.4, 12.0]
        })
        .is_none());
    }

    #[test]
    fn burst_horizon_clamped_to_band() {
        // eta above the band clamps to 48h …
        let high = map_to_burst(Inference {
            scores: vec![0.8, 100.0],
        })
        .unwrap();
        assert_eq!(high.horizon_hours, BURST_HORIZON_MAX_H);
        // … and below the band clamps to 6h.
        let low = map_to_burst(Inference {
            scores: vec![0.8, 1.0],
        })
        .unwrap();
        assert_eq!(low.horizon_hours, BURST_HORIZON_MIN_H);
        // A value inside the band is preserved.
        let mid = map_to_burst(Inference {
            scores: vec![0.8, 18.0],
        })
        .unwrap();
        assert_eq!(mid.horizon_hours, 18);
    }

    #[test]
    fn clamp_horizon_handles_non_finite() {
        assert_eq!(clamp_horizon(f32::NAN), BURST_HORIZON_MAX_H);
        assert_eq!(clamp_horizon(f32::INFINITY), BURST_HORIZON_MAX_H);
        assert_eq!(clamp_horizon(-5.0), BURST_HORIZON_MIN_H);
    }

    #[test]
    fn demand_is_always_24h() {
        // Too few ⇒ padded.
        assert_eq!(
            map_to_demand(Inference {
                scores: vec![3.0, 4.0]
            })
            .hourly_kwh
            .len(),
            DEMAND_HOURS
        );
        // Too many ⇒ truncated.
        assert_eq!(
            map_to_demand(Inference {
                scores: vec![1.0; 40]
            })
            .hourly_kwh
            .len(),
            DEMAND_HOURS
        );
    }
}
