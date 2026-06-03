//! `SimSensorAdapter` — a deterministic synthetic sensor source (Completion §1).
//!
//! Where real ESP32 SEED hardware is unavailable, this stands in **only** for the 30-day thermal
//! backfill (AC-3) and integration wiring — it implements [`SensorSource`] like the live rvcsi
//! adapter, producing physically-plausible readings on a deterministic diurnal cycle. It is the one
//! sanctioned fake at Completion (sparc.md Part 5 §1: "fake only physical hardware where
//! unavoidable"); everything downstream of it is real domain code.

use std::sync::atomic::{AtomicU64, Ordering};

use async_trait::async_trait;
use hoforras_domain::{NodeId, RawReading};
use hoforras_ports::sensor::SensorSource;
use hoforras_ports::PortResult;

/// A synthetic node sensor. `bias_c` shifts the building warm (surplus) or cold (deficit) so a test
/// can stage a surplus seller and a deficit buyer.
pub struct SimSensorAdapter {
    node_id: NodeId,
    bias_c: f32,
    tick: AtomicU64,
}

impl SimSensorAdapter {
    /// A neutral building around the comfort set-point.
    pub fn new(node_id: NodeId) -> Self {
        Self {
            node_id,
            bias_c: 0.0,
            tick: AtomicU64::new(0),
        }
    }

    /// A building biased warm (`+`, surplus) or cold (`-`, deficit) by `bias_c` degrees.
    pub fn with_bias(node_id: NodeId, bias_c: f32) -> Self {
        Self {
            node_id,
            bias_c,
            tick: AtomicU64::new(0),
        }
    }

    /// Synthesize the reading for sample index `i` (deterministic — same `i` ⇒ same reading).
    pub fn reading_at(&self, i: u64) -> RawReading {
        // A gentle diurnal temperature cycle around the 21 °C set-point, plus the building bias.
        let phase = (i % 24) as f32 * std::f32::consts::PI / 12.0;
        let temperature_celsius = 21.0 + self.bias_c + 3.0 * phase.sin();
        RawReading {
            node_id: self.node_id.clone(),
            timestamp_ns: i * 900_000_000_000, // ~15-min cadence
            temperature_celsius,
            pipe_vibration_hz: 4.0 + (i % 3) as f32,
            fluid_pressure_bar: 3.0,
            ground_thermal_gradient: 0.1,
        }
    }
}

#[async_trait]
impl SensorSource for SimSensorAdapter {
    async fn poll(&self) -> PortResult<RawReading> {
        let i = self.tick.fetch_add(1, Ordering::SeqCst);
        Ok(self.reading_at(i))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node() -> NodeId {
        NodeId::new("pozsonyi14.thermal.budapest.dark").unwrap()
    }

    #[tokio::test]
    async fn poll_is_deterministic_and_advances() {
        let sim = SimSensorAdapter::new(node());
        let a = sim.poll().await.unwrap();
        let b = sim.poll().await.unwrap();
        assert_eq!(a, sim.reading_at(0));
        assert_eq!(b, sim.reading_at(1));
        assert_ne!(a.timestamp_ns, b.timestamp_ns);
    }

    #[test]
    fn warm_bias_runs_hotter_than_cold_bias() {
        let warm = SimSensorAdapter::with_bias(node(), 6.0).reading_at(0);
        let cold = SimSensorAdapter::with_bias(node(), -6.0).reading_at(0);
        assert!(warm.temperature_celsius > cold.temperature_celsius);
    }
}
