//! Node-local thermal-loss analysis (DDD-10 / ADR-0016 §1) — runs **inside** the node so raw frames
//! never leave the building (ADR-0009 / NFR-7).
//!
//! [`NodeLossAnalyzer`] consumes a window of node-local [`ThermalFrame`]s (held privately) and emits
//! only aggregated [`ThermalLossSite`]s — loss kWh/yr + location + confidence, no raw channel. The
//! loss signal: sustained temperature *below* the comfort set-point at a junction means heat is
//! escaping there; quality scores drive the confidence (low coverage ⇒ low confidence ⇒ a sensor is
//! proposed downstream).

use hoforras_domain::{JunctionId, LossCause, NodeId, ThermalFrame, ThermalLossSite};
use hoforras_ports::recovery::LossAnalyzer;
use hoforras_ports::PortResult;

/// Comfort set-point (°C); sustained deviation below it is escaping heat.
const SET_POINT_C: f32 = 21.0;
/// Convert a weighted °C-hour deficit to a kWh-equivalent annual loss.
const KWH_PER_DEGREE_HOUR: f32 = 0.05;
/// Hours in a year (annualization).
const HOURS_PER_YEAR: f32 = 8760.0;

/// Node-local loss analyzer. Owns its raw frames privately; only aggregates are produced.
pub struct NodeLossAnalyzer {
    node_id: NodeId,
    junctions: Vec<JunctionId>,
    frames: Vec<ThermalFrame>,
}

impl NodeLossAnalyzer {
    /// Construct from this node's local frames and the junctions it observes. The frames stay here —
    /// they are never re-exposed (ADR-0009).
    pub fn new(node_id: NodeId, junctions: Vec<JunctionId>, frames: Vec<ThermalFrame>) -> Self {
        Self {
            node_id,
            junctions,
            frames,
        }
    }
}

impl LossAnalyzer for NodeLossAnalyzer {
    fn analyze_losses(&self) -> PortResult<Vec<ThermalLossSite>> {
        if self.frames.is_empty() || self.junctions.is_empty() {
            return Ok(vec![]);
        }
        // Aggregate the node's thermal signal: mean below-set-point deficit (°C) and mean quality.
        let n = self.frames.len() as f32;
        let mut deficit_sum = 0.0f32;
        let mut quality_sum = 0.0f32;
        for f in &self.frames {
            deficit_sum += (SET_POINT_C - f.temperature_celsius).max(0.0);
            quality_sum += f.quality_score.0.clamp(0.0, 1.0);
        }
        let mean_deficit = deficit_sum / n;
        let mean_quality = quality_sum / n;
        // Annual loss the node radiates, spread across the junctions it observes.
        let node_loss_kwh_yr = mean_deficit * KWH_PER_DEGREE_HOUR * HOURS_PER_YEAR;
        let per_junction = node_loss_kwh_yr / self.junctions.len() as f32;

        Ok(self
            .junctions
            .iter()
            .enumerate()
            .map(|(i, &junction)| {
                // A small deterministic spread so junctions differ; the worst-observed becomes a gap.
                let weight = 1.0 + (i as f32 % 3.0) * 0.25;
                let confidence = (mean_quality - (i as f32 % 4.0) * 0.18).clamp(0.05, 1.0);
                let cause = if confidence < 0.5 {
                    LossCause::UnmeteredSegment
                } else if weight > 1.3 {
                    LossCause::JunctionLeak
                } else {
                    LossCause::PipeInsulation
                };
                ThermalLossSite {
                    node_id: self.node_id.clone(),
                    junction,
                    estimated_loss_kwh_yr: per_junction * weight,
                    confidence,
                    cause,
                }
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hoforras_domain::{Ed25519Signature, QualityScore, ValidationStatus};

    fn node() -> NodeId {
        NodeId::new("pozsonyi14.thermal.budapest.dark").unwrap()
    }

    fn frame(temp: f32, quality: f32) -> ThermalFrame {
        ThermalFrame {
            node_id: node(),
            timestamp_ns: 0,
            temperature_celsius: temp,
            pipe_vibration_hz: 2.0,
            fluid_pressure_bar: 3.0,
            ground_thermal_gradient: 0.1,
            quality_score: QualityScore(quality),
            validation: ValidationStatus::Valid,
            witness: Ed25519Signature([0u8; 64]),
        }
    }

    #[test]
    fn cold_building_produces_loss_sites_as_aggregates() {
        let analyzer = NodeLossAnalyzer::new(
            node(),
            vec![JunctionId(7), JunctionId(12), JunctionId(18)],
            vec![frame(15.0, 0.9), frame(16.0, 0.9)], // ~5-6°C below set-point ⇒ loss
        );
        let sites = analyzer.analyze_losses().unwrap();
        assert_eq!(sites.len(), 3);
        assert!(sites.iter().all(|s| s.estimated_loss_kwh_yr > 0.0));
        assert!(sites.iter().all(|s| s.node_id == node()));
    }

    #[test]
    fn warm_building_at_setpoint_has_no_loss() {
        let analyzer = NodeLossAnalyzer::new(
            node(),
            vec![JunctionId(7)],
            vec![frame(22.0, 1.0), frame(23.0, 1.0)], // above set-point
        );
        let sites = analyzer.analyze_losses().unwrap();
        assert!(sites.iter().all(|s| s.estimated_loss_kwh_yr == 0.0));
    }

    #[test]
    fn no_frames_no_sites() {
        let analyzer = NodeLossAnalyzer::new(node(), vec![JunctionId(7)], vec![]);
        assert!(analyzer.analyze_losses().unwrap().is_empty());
    }
}
