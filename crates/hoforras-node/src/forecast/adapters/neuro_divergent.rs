//! `NeuroDivergentAdapter` — reference on-device model host (ADR-0012, edge-only WASM).
//!
//! Production deployments run ruv-FANN's Neuro-Divergent LSTM / N-BEATS networks compiled to WASM,
//! executing **on the Appliance** — no GPU, no cloud, no network hop (ADR-0012 / NFR-5). That crate
//! is not available in this environment, so this is a **deterministic reference network** behind the
//! same boundary, swappable per ADR-0001 with no change to `ForecastServiceImpl`. It is pure CPU and
//! holds no I/O, which is exactly why edge-only holds: there is no seam here through which a remote
//! backend could be reached.
//!
//! The reference network derives physically-motivated features from the `ReadingWindow` and emits
//! an `Inference.scores` vector whose layout depends on the [`ForecastTask`] — matching the layout
//! the service's mappers expect.

use hoforras_domain::{Inference, ReadingWindow};

use crate::forecast::service::{SPEC_BURST, SPEC_DEMAND};

/// Which reference head runs, selected from the spawn profile's specialization tag.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ForecastTask {
    Anomaly,
    Burst,
    Demand,
}

impl ForecastTask {
    /// Resolve a task from an `AgentProfile.specialization` tag (defaults to anomaly detection).
    pub fn from_spec(spec: &str) -> Self {
        match spec {
            SPEC_BURST => Self::Burst,
            SPEC_DEMAND => Self::Demand,
            _ => Self::Anomaly,
        }
    }
}

/// Reference model host. Stateless — one instance can serve any task.
#[derive(Clone, Copy, Default)]
pub struct NeuroDivergentAdapter;

impl NeuroDivergentAdapter {
    pub fn new() -> Self {
        Self
    }

    /// Run on-device inference for `task` over `window`. Deterministic and side-effect-free.
    pub fn infer(&self, task: ForecastTask, window: &ReadingWindow) -> Inference {
        let f = Features::from_window(window);
        let scores = match task {
            ForecastTask::Anomaly => f.anomaly_scores(),
            ForecastTask::Burst => f.burst_scores(),
            ForecastTask::Demand => f.demand_scores(),
        };
        Inference { scores }
    }
}

/// Physically-motivated summary statistics of a reading window — the reference network's "features".
struct Features {
    max_temp: f32,
    mean_temp: f32,
    max_vibration: f32,
    pressure_drop: f32,
}

impl Features {
    fn from_window(window: &ReadingWindow) -> Self {
        let n = window.frames.len();
        if n == 0 {
            return Self {
                max_temp: 0.0,
                mean_temp: 0.0,
                max_vibration: 0.0,
                pressure_drop: 0.0,
            };
        }
        let mut max_temp = f32::NEG_INFINITY;
        let mut sum_temp = 0.0f32;
        let mut max_vibration = 0.0f32;
        let mut min_pressure = f32::INFINITY;
        let mut mean_pressure = 0.0f32;
        for fr in &window.frames {
            max_temp = max_temp.max(fr.temperature_celsius);
            sum_temp += fr.temperature_celsius;
            max_vibration = max_vibration.max(fr.pipe_vibration_hz.abs());
            min_pressure = min_pressure.min(fr.fluid_pressure_bar);
            mean_pressure += fr.fluid_pressure_bar;
        }
        let n_f = n as f32;
        let mean_pressure = mean_pressure / n_f;
        Self {
            max_temp,
            mean_temp: sum_temp / n_f,
            max_vibration,
            // A drop is mean-minus-min pressure; never negative.
            pressure_drop: (mean_pressure - min_pressure).max(0.0),
        }
    }

    /// Anomaly head: per-channel confidence in
    /// `[ThermalSpike, PressureDrop, VibrationPattern, BurstPrecursor]` order.
    fn anomaly_scores(&self) -> Vec<f32> {
        let thermal_spike = unit(self.max_temp, 60.0, 95.0);
        let pressure_drop = unit(self.pressure_drop, 0.5, 4.0);
        let vibration = unit(self.max_vibration, 10.0, 60.0);
        let burst_precursor = (pressure_drop * 0.6 + vibration * 0.4).clamp(0.0, 1.0);
        vec![thermal_spike, pressure_drop, vibration, burst_precursor]
    }

    /// Burst head: `[precursor_score, eta_hours]`. ETA falls as severity rises and is deliberately
    /// allowed outside [6h, 48h] so the service's clamp is the single source of truth (FR-2.3).
    fn burst_scores(&self) -> Vec<f32> {
        let pressure_drop = unit(self.pressure_drop, 0.5, 4.0);
        let vibration = unit(self.max_vibration, 10.0, 60.0);
        let severity = (pressure_drop * 0.6 + vibration * 0.4).clamp(0.0, 1.0);
        let eta_hours = 54.0 - severity * 60.0; // severity 0 → 54h, severity 1 → -6h
        vec![severity, eta_hours]
    }

    /// Demand head: 24 hourly kWh values. Colder mean temperature ⇒ higher heating base load,
    /// shaped by a diurnal sinusoid (FR-2.4).
    fn demand_scores(&self) -> Vec<f32> {
        let base = (21.0 - self.mean_temp).max(0.0) * 2.0;
        (0..24)
            .map(|h| {
                let diurnal = 1.0 + 0.3 * ((h as f32) * std::f32::consts::PI / 12.0 - 1.0).sin();
                (base * diurnal).max(0.0)
            })
            .collect()
    }
}

/// Linear-normalize `x` from `[lo, hi]` into `[0.0, 1.0]`.
fn unit(x: f32, lo: f32, hi: f32) -> f32 {
    if !x.is_finite() || hi <= lo {
        return 0.0;
    }
    ((x - lo) / (hi - lo)).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use hoforras_domain::{
        Ed25519Signature, NodeId, QualityScore, ReadingWindow, ThermalFrame, Timestamp,
        ValidationStatus,
    };

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

    #[test]
    fn empty_window_is_safe_and_low_signal() {
        let inf = NeuroDivergentAdapter::new().infer(ForecastTask::Anomaly, &window(vec![]));
        assert_eq!(inf.scores.len(), 4);
        assert!(inf.scores.iter().all(|s| *s == 0.0));
    }

    #[test]
    fn hot_window_raises_thermal_spike() {
        let w = window(vec![frame(90.0, 5.0, 4.0), frame(92.0, 6.0, 4.0)]);
        let inf = NeuroDivergentAdapter::new().infer(ForecastTask::Anomaly, &w);
        assert!(inf.scores[0] > 0.5, "thermal spike channel should be high");
    }

    #[test]
    fn demand_head_is_24h() {
        let w = window(vec![frame(5.0, 2.0, 3.0)]);
        let inf = NeuroDivergentAdapter::new().infer(ForecastTask::Demand, &w);
        assert_eq!(inf.scores.len(), 24);
        assert!(inf.scores.iter().all(|s| s.is_finite() && *s >= 0.0));
    }

    #[test]
    fn task_resolves_from_spec() {
        assert_eq!(ForecastTask::from_spec(SPEC_BURST), ForecastTask::Burst);
        assert_eq!(ForecastTask::from_spec(SPEC_DEMAND), ForecastTask::Demand);
        assert_eq!(
            ForecastTask::from_spec("anything_else"),
            ForecastTask::Anomaly
        );
    }
}
