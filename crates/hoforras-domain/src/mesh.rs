//! Mesh / AI-orchestration / operator value objects (DDD-07, DDD-08, DDD-09).

use serde::{Deserialize, Serialize};

use crate::forecast::Anomaly;
use crate::ids::{JunctionId, NodeId};

/// Context handed to Claude when explaining an anomaly (FR-8.2 / DDD-08).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnomalyContext {
    pub node_id: NodeId,
    pub anomaly: Anomaly,
}

/// A natural-language anomaly explanation from Claude (FR-8.2).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Explanation {
    pub text: String,
}

/// Events pushed to the operator dashboard over the WebSocket (DDD-09, FR-9.2).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum UiEvent {
    TradeExecuted {
        seller: NodeId,
        buyer: NodeId,
        kwh: f32,
        pipe_route: Vec<JunctionId>,
    },
    AnomalyDetected {
        node_id: NodeId,
    },
    AnomalyExplained {
        node_id: NodeId,
        explanation: Explanation,
    },
}
