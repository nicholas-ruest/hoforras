//! Thermal Memory value objects (DDD-06).

use serde::{Deserialize, Serialize};

use crate::ids::{JunctionId, NodeId};

/// Embedding dimensionality fixed by the spec (FR-6.2). Deterministic so similarity is stable.
pub const EMBEDDING_DIM: usize = 128;

/// A 128-dimensional thermal consumption-pattern embedding (FR-6.2).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Embedding {
    values: Vec<f32>,
}

impl Embedding {
    /// Construct a 128-dim embedding, validating dimensionality at the boundary (FR-6.2).
    pub fn new(values: Vec<f32>) -> Result<Self, crate::DomainError> {
        if values.len() != EMBEDDING_DIM {
            return Err(crate::DomainError::Invalid(format!(
                "embedding must be {EMBEDDING_DIM}-dim, got {}",
                values.len()
            )));
        }
        Ok(Self { values })
    }

    pub fn as_slice(&self) -> &[f32] {
        &self.values
    }
}

/// A similarity query over the vector index (FR-6.3).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Query {
    pub vector: Embedding,
    pub top_k: usize,
    pub surplus_available: bool,
    pub district: String,
}

/// A similarity-search candidate (FR-6.3).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Candidate {
    pub node_id: NodeId,
    pub similarity: f32,
}

/// The district thermal network as a graph: buildings = nodes, pipes = weighted edges (FR-6.4).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DistrictGraph {
    pub nodes: Vec<NodeId>,
    pub edges: Vec<(JunctionId, JunctionId, f32)>,
}

/// Output of GNN inference over the district graph (FR-6.4).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GnnState {
    pub node_scores: Vec<(NodeId, f32)>,
}
