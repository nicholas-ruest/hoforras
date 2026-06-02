//! `ThermalAggregator` — projects node-local frames into the published `ThermalBalance` (ADR-0009).
//!
//! **Boundary rule (ADR-0009 / NFR-7):** raw [`ThermalFrame`]s stay node-local; the *only* thermal
//! quantity exposed across a partition boundary is the aggregated [`ThermalBalance`]. This
//! projection is where node-local sensing becomes a publishable surplus/deficit figure
//! (DDD-02 → DDD-01 Published Language).

use hoforras_domain::{NodeId, ThermalBalance, ThermalFrame, TradeWindow};

/// Projects a window of frames into a single `ThermalBalance`.
pub struct ThermalAggregator;

impl ThermalAggregator {
    /// Aggregate `frames` into a [`ThermalBalance`] for `node` over `window`.
    ///
    /// Surplus/deficit are derived by treating each frame's temperature relative to a neutral
    /// set-point as a per-frame thermal contribution (kWh-equivalent): above set-point contributes
    /// to surplus, below to deficit. Quality weights each contribution so low-confidence readings
    /// move the balance less. Raw frames are consumed here and never leave the node.
    pub fn aggregate(frames: &[ThermalFrame], node: NodeId, window: TradeWindow) -> ThermalBalance {
        /// Neutral comfort set-point (°C); deviation from it is the tradeable thermal signal.
        const SET_POINT_C: f32 = 21.0;
        /// Conversion from a weighted °C-deviation to a kWh-equivalent contribution.
        const KWH_PER_DEGREE: f32 = 0.05;

        let mut surplus_kwh = 0.0_f32;
        let mut deficit_kwh = 0.0_f32;
        for frame in frames {
            let weight = frame.quality_score.0.clamp(0.0, 1.0);
            let delta = (frame.temperature_celsius - SET_POINT_C) * weight * KWH_PER_DEGREE;
            if delta >= 0.0 {
                surplus_kwh += delta;
            } else {
                deficit_kwh += -delta;
            }
        }

        ThermalBalance {
            node_id: node,
            surplus_kwh,
            deficit_kwh,
            window,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hoforras_domain::{Ed25519Signature, QualityScore, ValidationStatus};

    fn frame(temp: f32, quality: f32) -> ThermalFrame {
        ThermalFrame {
            node_id: NodeId::new("a.thermal.budapest.dark").unwrap(),
            timestamp_ns: 0,
            temperature_celsius: temp,
            pipe_vibration_hz: 1.0,
            fluid_pressure_bar: 2.0,
            ground_thermal_gradient: 0.1,
            quality_score: QualityScore(quality),
            validation: ValidationStatus::Valid,
            witness: Ed25519Signature([0u8; 64]),
        }
    }

    fn node() -> NodeId {
        NodeId::new("a.thermal.budapest.dark").unwrap()
    }

    #[test]
    fn warm_frames_produce_surplus_cold_produce_deficit() {
        let frames = vec![frame(25.0, 1.0), frame(17.0, 1.0)];
        let balance = ThermalAggregator::aggregate(&frames, node(), TradeWindow::new(4).unwrap());
        assert!(balance.surplus_kwh > 0.0);
        assert!(balance.deficit_kwh > 0.0);
        assert_eq!(balance.node_id, node());
    }

    #[test]
    fn empty_window_is_zero_balance() {
        let balance = ThermalAggregator::aggregate(&[], node(), TradeWindow::new(1).unwrap());
        assert_eq!(balance.surplus_kwh, 0.0);
        assert_eq!(balance.deficit_kwh, 0.0);
    }
}
