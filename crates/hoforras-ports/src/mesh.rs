//! District Mesh ports (DDD-07). DAG transport, NOT RPC (ADR-0007).

use async_trait::async_trait;
use hoforras_domain::{DagEntry, NodeId};

use crate::PortResult;

/// Publishes/receives signed DAG entries across the district fabric (FR-7.1). There is
/// deliberately no RPC port (ADR-0007).
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
#[async_trait]
pub trait MeshTransport: Send + Sync {
    async fn publish(&self, entry: DagEntry) -> PortResult<()>;
    async fn next_event(&self) -> PortResult<Option<DagEntry>>;
}

/// Tracks live district membership for self-heal (FR-7.2). Sync.
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait NodeRegistry: Send + Sync {
    fn mark_down(&self, node: NodeId);
    fn mark_up(&self, node: NodeId);
    fn live(&self) -> Vec<NodeId>;
}
