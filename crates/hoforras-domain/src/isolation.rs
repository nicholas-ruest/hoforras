//! Isolation value objects (DDD-04): the result of a coherence mincut.

use serde::{Deserialize, Serialize};

use crate::ids::NodeId;

/// The outcome of recomputing the coherence mincut over the node graph (FR-4.2).
/// `isolated` lists the nodes split off into their own partition.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct Partitioning {
    pub isolated: Vec<NodeId>,
}
