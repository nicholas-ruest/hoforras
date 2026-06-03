//! Mesh / AI-orchestration / operator value objects (DDD-07, DDD-08, DDD-09).

use serde::{Deserialize, Serialize};

use crate::forecast::Anomaly;
use crate::ids::{JunctionId, NodeId};

/// A district membership change handed to the `MeshCoordinator` (DDD-07 §2 / FR-7.2).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeEvent {
    /// A peer Appliance dropped off the mesh.
    Drop(NodeId),
    /// A peer Appliance returned to the mesh.
    Up(NodeId),
}

/// A district-scale signal that triggers collective mitigation behaviour (DDD-07 §2 / FR-7.3).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CollectiveSignal {
    /// A thermal zone is overloaded — rebalance load across it.
    Overload { zone: String },
    /// A cascade failure risk along a pipe path — shed and reroute.
    CascadeRisk { path: Vec<JunctionId> },
    /// A district-wide heat wave — engage emergency routing.
    HeatWave,
}

/// The mitigation action a `CollectiveSignal` dispatches to (DDD-07 §5 / FR-7.3).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum MeshAction {
    /// Rebalance thermal load across an overloaded zone.
    RebalanceLoad { zone: String },
    /// Shed and reroute around a cascade-risk path.
    ShedAndReroute { path: Vec<JunctionId> },
    /// Engage the district emergency routing plan.
    EmergencyRouting,
}

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
