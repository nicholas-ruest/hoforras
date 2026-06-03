//! `WebSocketSink` — the operator dashboard channel over tokio-tungstenite (DDD-08 / FR-9.2).
//!
//! Reference implementation behind the `OperatorChannel` port. The real sink pushes `UiEvent`s over
//! a WebSocket (`ws://appliance.district-xiii:3001`); tokio-tungstenite is unavailable here, so this
//! records pushed events in memory (the dashboard, P9, conforms to these event shapes). Swappable
//! per ADR-0001.

use std::sync::Mutex;

use async_trait::async_trait;
use hoforras_domain::UiEvent;
use hoforras_ports::ai::OperatorChannel;
use hoforras_ports::PortResult;

/// Reference operator channel. Captures the events that would travel to the dashboard.
#[derive(Default)]
pub struct WebSocketSink {
    pushed: Mutex<Vec<UiEvent>>,
}

impl WebSocketSink {
    pub fn new() -> Self {
        Self::default()
    }

    /// Events pushed so far (test/observability helper).
    pub fn pushed(&self) -> Vec<UiEvent> {
        self.pushed.lock().expect("sink poisoned").clone()
    }
}

#[async_trait]
impl OperatorChannel for WebSocketSink {
    async fn push(&self, event: UiEvent) -> PortResult<()> {
        self.pushed.lock().expect("sink poisoned").push(event);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hoforras_domain::NodeId;

    #[tokio::test]
    async fn push_records_event() {
        let sink = WebSocketSink::new();
        sink.push(UiEvent::AnomalyDetected {
            node_id: NodeId::new("a.thermal.budapest.dark").unwrap(),
        })
        .await
        .unwrap();
        assert_eq!(sink.pushed().len(), 1);
    }
}
