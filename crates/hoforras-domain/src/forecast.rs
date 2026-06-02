//! Forecasting & Anomaly value objects (DDD-03).

use serde::{Deserialize, Serialize};

pub use crate::sensor::ReadingWindow;

/// Profile for an ephemeral inference agent (FR-2.1). Mirrors the ruv-swarm spawn profile.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AgentProfile {
    pub specialization: String,
    pub neural_model: String,
    pub analytical: f32,
    pub systematic: f32,
}

impl Default for AgentProfile {
    fn default() -> Self {
        Self {
            specialization: "thermal_timeseries".into(),
            neural_model: "lstm".into(),
            analytical: 0.95,
            systematic: 0.9,
        }
    }
}

/// Raw inference output before it is mapped to typed forecasts/anomalies.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Inference {
    pub scores: Vec<f32>,
}

/// A 24-hour, per-building thermal-demand forecast (FR-2.4).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DemandForecast {
    /// Hourly projected demand in kWh, length 24.
    pub hourly_kwh: Vec<f32>,
}

/// Kinds of thermal anomaly (FR-2.2; taxonomy resolved in `sparc.md` Part 2 §13).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnomalyKind {
    ThermalSpike,
    PressureDrop,
    VibrationPattern,
    BurstPrecursor,
}

/// A typed, confidence-scored anomaly (FR-2.2).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Anomaly {
    pub kind: AnomalyKind,
    pub confidence: f32,
}

/// A pipe-burst precursor warning with a horizon clamped to [6h, 48h] (FR-2.3).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct BurstWarning {
    pub horizon_hours: u32,
    pub score: f32,
}
