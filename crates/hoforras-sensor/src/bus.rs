//! `TypedEventBus` — a node-local channel for validated frames + a rejection sink (ADR-0009).
//!
//! Implements [`EventEmitter`] and [`RejectionLog`]. **Node-local invariant (ADR-0009):**
//! `RawReading` and `ThermalFrame` flow over this bus only *within* the Appliance; they never leave
//! the node. Anything crossing a partition boundary must go through the [`crate::aggregate`]
//! projection into a `ThermalBalance` instead.

use std::sync::Mutex;

use async_trait::async_trait;
use hoforras_domain::{DomainError, RawReading, ThermalFrame};
use hoforras_ports::sensor::{EventEmitter, RejectionLog};
use tokio::sync::mpsc::{self, Receiver, Sender};

/// Aggregated state of recorded rejections, for test assertions / local observability (FR-1.2).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RejectionState {
    pub count: usize,
    pub last_reason: Option<String>,
}

/// A bounded, node-local bus carrying validated [`ThermalFrame`]s, plus a rejection log.
pub struct TypedEventBus {
    tx: Sender<ThermalFrame>,
    rx: Mutex<Option<Receiver<ThermalFrame>>>,
    rejections: Mutex<RejectionState>,
}

impl TypedEventBus {
    /// Create a bus with a bounded channel of `capacity` frames (backpressure when full).
    pub fn new(capacity: usize) -> Self {
        let (tx, rx) = mpsc::channel(capacity);
        Self {
            tx,
            rx: Mutex::new(Some(rx)),
            rejections: Mutex::new(RejectionState::default()),
        }
    }

    /// Take ownership of the receiving half. The frames stay node-local (ADR-0009): the receiver is
    /// consumed *inside the Appliance* (e.g. by the forecaster), never shipped off-node. Returns
    /// `None` if the receiver has already been taken.
    pub fn take_receiver(&self) -> Option<Receiver<ThermalFrame>> {
        self.rx.lock().expect("bus receiver poisoned").take()
    }

    /// Snapshot of the rejection log (count + last reason) for assertions / observability.
    pub fn rejection_state(&self) -> RejectionState {
        self.rejections
            .lock()
            .expect("rejection state poisoned")
            .clone()
    }
}

#[async_trait]
impl EventEmitter for TypedEventBus {
    async fn emit(&self, frame: ThermalFrame) -> Result<(), DomainError> {
        self.tx
            .send(frame)
            .await
            .map_err(|e| DomainError::Adapter(format!("event bus closed: {e}")))
    }
}

impl RejectionLog for TypedEventBus {
    fn record(&self, _raw: RawReading, reason: String) {
        let mut state = self.rejections.lock().expect("rejection state poisoned");
        state.count += 1;
        state.last_reason = Some(reason);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hoforras_domain::{Ed25519Signature, NodeId, QualityScore, ValidationStatus};

    fn frame() -> ThermalFrame {
        ThermalFrame {
            node_id: NodeId::new("a.thermal.budapest.dark").unwrap(),
            timestamp_ns: 1,
            temperature_celsius: 20.0,
            pipe_vibration_hz: 1.0,
            fluid_pressure_bar: 2.0,
            ground_thermal_gradient: 0.1,
            quality_score: QualityScore(1.0),
            validation: ValidationStatus::Valid,
            witness: Ed25519Signature([0u8; 64]),
        }
    }

    #[tokio::test]
    async fn emitted_frame_is_received_node_local() {
        let bus = TypedEventBus::new(4);
        let mut rx = bus.take_receiver().expect("receiver available");
        bus.emit(frame()).await.unwrap();
        let got = rx.recv().await.expect("a frame");
        assert_eq!(got.timestamp_ns, 1);
    }

    #[test]
    fn record_tracks_count_and_last_reason() {
        let bus = TypedEventBus::new(1);
        let raw = RawReading {
            node_id: NodeId::new("a.thermal.budapest.dark").unwrap(),
            timestamp_ns: 0,
            temperature_celsius: 0.0,
            pipe_vibration_hz: 0.0,
            fluid_pressure_bar: 0.0,
            ground_thermal_gradient: 0.0,
        };
        bus.record(raw.clone(), "first".into());
        bus.record(raw, "second".into());
        let state = bus.rejection_state();
        assert_eq!(state.count, 2);
        assert_eq!(state.last_reason.as_deref(), Some("second"));
    }
}
