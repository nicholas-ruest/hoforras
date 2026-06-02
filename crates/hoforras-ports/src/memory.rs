//! Thermal Memory ports (DDD-06). ACL over RuVector (HNSW/GNN). Async (I/O).

use async_trait::async_trait;
use hoforras_domain::{Candidate, DistrictGraph, Embedding, GnnState, Json, NodeId, Query};

use crate::PortResult;

/// Vector store: upsert embeddings, similarity search (FR-6.1–6.3).
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
#[async_trait]
pub trait VectorIndex: Send + Sync {
    async fn upsert(&self, id: NodeId, vector: Embedding, meta: Json) -> PortResult<()>;
    async fn search(&self, query: Query) -> PortResult<Vec<Candidate>>;
}

/// Graph-neural inference over the district graph (FR-6.4).
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
#[async_trait]
pub trait GnnEngine: Send + Sync {
    async fn infer(&self, graph: DistrictGraph) -> PortResult<GnnState>;
}
