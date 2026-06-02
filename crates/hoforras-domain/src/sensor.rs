//! Sensing & Ingestion value objects (DDD-02).
//!
//! `RawReading` is transient input that never crosses a partition boundary (NFR-7 / ADR-0009).
//! `ThermalFrame` is the validated, quality-scored, witnessed reading (FR-1.1).

use serde::{Deserialize, Serialize};

use crate::ids::{NodeId, Timestamp};

/// Raw multi-source sensor input prior to validation. **Never** leaves the coherence domain.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RawReading {
    pub node_id: NodeId,
    pub timestamp_ns: u64,
    pub temperature_celsius: f32,
    pub pipe_vibration_hz: f32,
    pub fluid_pressure_bar: f32,
    pub ground_thermal_gradient: f32,
}

/// Confidence score attached to an ingested reading (0.0..=1.0). Mirrors rvcsi's `QualityScore`
/// without depending on rvcsi (the adapter translates — DDD-02 ACL).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct QualityScore(pub f32);

/// Outcome of validation at the ingestion boundary (FR-1.2).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ValidationStatus {
    Valid,
    Invalid(String),
}

/// An Ed25519 signature over `canonical_bytes` of a frame (FR-1.3 / ADR-0014).
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Ed25519Signature(#[serde(with = "serde_big_array::BigArray")] pub [u8; 64]);

impl PartialEq for Ed25519Signature {
    fn eq(&self, other: &Self) -> bool {
        self.0[..] == other.0[..]
    }
}

impl std::fmt::Debug for Ed25519Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Ed25519Signature(..)")
    }
}

/// A validated, quality-scored, cryptographically witnessed thermal reading (`sparc.md` §10).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ThermalFrame {
    pub node_id: NodeId,
    pub timestamp_ns: u64,
    pub temperature_celsius: f32,
    pub pipe_vibration_hz: f32,
    pub fluid_pressure_bar: f32,
    pub ground_thermal_gradient: f32,
    pub quality_score: QualityScore,
    pub validation: ValidationStatus,
    pub witness: Ed25519Signature,
}

/// A window of readings handed to the forecaster (DDD-02 → DDD-03, Customer/Supplier).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReadingWindow {
    pub node_id: NodeId,
    pub from_ts: Timestamp,
    pub to_ts: Timestamp,
    pub frames: Vec<ThermalFrame>,
}
