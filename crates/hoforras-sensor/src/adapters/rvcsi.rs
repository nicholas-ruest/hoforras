//! `RvcsiIngestAdapter` — anti-corruption layer over @ruv/rvcsi (DDD-02 ACL).
//!
//! @ruv/rvcsi speaks in *channel-state-information* terms — subcarriers, RSSI, doppler — not in
//! the thermal ubiquitous language. This adapter **translates** that vocabulary into a
//! [`RawReading`], keeping rvcsi's concepts from leaking into the domain. @ruv/rvcsi is not an
//! available Rust crate, so this is a **reference** adapter standing in for the real sensor source;
//! it is swappable per ADR-0001. It implements [`SensorSource`], [`Validator`] and
//! [`QualityScorer`].

use async_trait::async_trait;
use hoforras_domain::{DomainError, NodeId, QualityScore, RawReading, ValidationStatus};
use hoforras_ports::sensor::{QualityScorer, SensorSource, Validator};

/// @ruv/rvcsi's frame vocabulary (CSI: channel state information). Private — it must never escape
/// this ACL into the thermal domain.
struct CsiFrame {
    /// Per-subcarrier magnitude samples.
    subcarriers: Vec<f32>,
    /// Received signal strength indicator (dBm).
    rssi: f32,
    /// Doppler shift estimate (Hz) — proxy for mechanical vibration.
    doppler_hz: f32,
    /// Phase offset (radians) — proxy for fluid pressure differential.
    phase_offset: f32,
}

impl CsiFrame {
    /// Mean subcarrier magnitude.
    fn mean_subcarrier(&self) -> f32 {
        if self.subcarriers.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.subcarriers.iter().copied().sum();
        sum / self.subcarriers.len() as f32
    }

    /// Subcarrier spread (max - min) — proxy for thermal gradient steepness.
    fn subcarrier_spread(&self) -> f32 {
        let mut min = f32::INFINITY;
        let mut max = f32::NEG_INFINITY;
        for &s in &self.subcarriers {
            min = min.min(s);
            max = max.max(s);
        }
        if min.is_finite() && max.is_finite() {
            max - min
        } else {
            0.0
        }
    }

    /// RSSI mapped to a 0.0..=1.0 link-confidence weight (stronger signal ⇒ more confidence).
    fn rssi_confidence(&self) -> f32 {
        // Typical indoor RSSI spans roughly -90 dBm (weak) to -30 dBm (strong).
        ((self.rssi + 90.0) / 60.0).clamp(0.0, 1.0)
    }
}

/// Reference rvcsi ingest adapter (ACL). Bound to one Appliance identity.
pub struct RvcsiIngestAdapter {
    node_id: NodeId,
}

impl RvcsiIngestAdapter {
    pub fn new(node_id: NodeId) -> Self {
        Self { node_id }
    }

    /// **Translation step (ACL):** map a vendor `CsiFrame` into the thermal ubiquitous language.
    /// This is the only place rvcsi's vocabulary is allowed.
    fn translate(&self, csi: &CsiFrame, timestamp_ns: u64) -> RawReading {
        RawReading {
            node_id: self.node_id.clone(),
            // Mean subcarrier magnitude (dB-ish) → degrees Celsius via an affine calibration.
            temperature_celsius: 15.0 + csi.mean_subcarrier() * 0.5,
            // Doppler shift maps directly to mechanical vibration frequency.
            pipe_vibration_hz: csi.doppler_hz.abs(),
            // Phase offset → fluid pressure (bar), biased into the positive range.
            fluid_pressure_bar: 1.0 + csi.phase_offset.abs() * 0.25,
            // Subcarrier spread, attenuated by RSSI link confidence → ground thermal gradient.
            ground_thermal_gradient: csi.subcarrier_spread() * csi.rssi_confidence() * 0.01,
            timestamp_ns,
        }
    }

    /// Synthesize one reference reading (stands in for a live rvcsi poll).
    fn synth_reading(&self) -> RawReading {
        let csi = CsiFrame {
            subcarriers: vec![12.0, 13.5, 11.8, 12.4, 13.0],
            rssi: -42.0,
            doppler_hz: 3.7,
            phase_offset: 1.6,
        };
        self.translate(&csi, 0)
    }
}

#[async_trait]
impl SensorSource for RvcsiIngestAdapter {
    async fn poll(&self) -> Result<RawReading, DomainError> {
        Ok(self.synth_reading())
    }
}

impl Validator for RvcsiIngestAdapter {
    fn validate(&self, raw: &RawReading) -> ValidationStatus {
        let fields = [
            raw.temperature_celsius,
            raw.pipe_vibration_hz,
            raw.fluid_pressure_bar,
            raw.ground_thermal_gradient,
        ];
        if fields.iter().any(|v| !v.is_finite()) {
            return ValidationStatus::Invalid("non-finite sensor field".into());
        }
        if raw.fluid_pressure_bar < 0.0 {
            return ValidationStatus::Invalid("negative fluid pressure".into());
        }
        ValidationStatus::Valid
    }
}

impl QualityScorer for RvcsiIngestAdapter {
    fn score(&self, raw: &RawReading) -> QualityScore {
        // Start fully confident; deduct for readings outside physically sane envelopes. The result
        // is clamped to 0.0..=1.0 (mirrors rvcsi's QualityScore range — DDD-02 ACL).
        let mut q: f32 = 1.0;
        if !(-40.0..=120.0).contains(&raw.temperature_celsius) {
            q -= 0.4;
        }
        if !(0.0..=80.0).contains(&raw.pipe_vibration_hz) {
            q -= 0.3;
        }
        if !(0.0..=16.0).contains(&raw.fluid_pressure_bar) {
            q -= 0.3;
        }
        QualityScore(q.clamp(0.0, 1.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node() -> NodeId {
        NodeId::new("pozsonyi14.thermal.budapest.dark").unwrap()
    }

    #[tokio::test]
    async fn poll_translates_csi_into_thermal_language() {
        let adapter = RvcsiIngestAdapter::new(node());
        let reading = adapter.poll().await.unwrap();
        assert_eq!(reading.node_id, node());
        assert!(reading.temperature_celsius.is_finite());
        assert!(reading.fluid_pressure_bar >= 0.0);
    }

    #[test]
    fn validate_rejects_nan_and_negative_pressure() {
        let adapter = RvcsiIngestAdapter::new(node());
        let mut raw = adapter.synth_reading();
        assert_eq!(adapter.validate(&raw), ValidationStatus::Valid);
        raw.temperature_celsius = f32::NAN;
        assert!(matches!(
            adapter.validate(&raw),
            ValidationStatus::Invalid(_)
        ));
        let mut raw2 = adapter.synth_reading();
        raw2.fluid_pressure_bar = -1.0;
        assert!(matches!(
            adapter.validate(&raw2),
            ValidationStatus::Invalid(_)
        ));
    }

    #[test]
    fn score_is_in_unit_range() {
        let adapter = RvcsiIngestAdapter::new(node());
        let q = adapter.score(&adapter.synth_reading());
        assert!((0.0..=1.0).contains(&q.0));
    }
}
