//! `SynapticMeshAdapter` — reference DAG fabric + node registry over Synaptic-Mesh (DDD-07 / ADR-0007).
//!
//! Production rides the Synaptic-Mesh / QuDAG pub/sub fabric. That substrate is unavailable here, so
//! this is a **reference in-process** implementation behind the `MeshTransport` / `NodeRegistry`
//! ports — swappable per ADR-0001. It models the two things NFR-9 depends on:
//!
//! * a **committed DAG log** (append-only) — the durable state that must survive node churn, and
//! * an **inbox** of undelivered entries that a returning node drains to resync (replay).
//!
//! Cloning the adapter shares the same `Arc` state, so the `MeshCoordinator` can hold it as both
//! transport and registry while a test still observes the committed log. There is, by construction,
//! no RPC path — only `publish`/`next_event` (ADR-0007).

use std::collections::{HashSet, VecDeque};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use hoforras_domain::{DagEntry, NodeId};
use hoforras_ports::mesh::{MeshTransport, NodeRegistry};
use hoforras_ports::PortResult;

#[derive(Default)]
struct Inner {
    /// Append-only committed DAG log — the durable state that survives churn (NFR-9).
    committed: Mutex<Vec<DagEntry>>,
    /// Undelivered entries a returning node drains to resync.
    inbox: Mutex<VecDeque<DagEntry>>,
    /// Live district membership.
    live: Mutex<HashSet<String>>,
}

/// Reference Synaptic-Mesh adapter. Cheap to clone — clones share the same fabric state.
#[derive(Clone, Default)]
pub struct SynapticMeshAdapter {
    inner: Arc<Inner>,
}

impl SynapticMeshAdapter {
    pub fn new() -> Self {
        Self::default()
    }

    /// The committed DAG log (test/observability helper) — what survivors retain across churn.
    pub fn committed(&self) -> Vec<DagEntry> {
        self.inner
            .committed
            .lock()
            .expect("mesh state poisoned")
            .clone()
    }

    /// Number of entries still undelivered (awaiting resync).
    pub fn pending(&self) -> usize {
        self.inner.inbox.lock().expect("mesh state poisoned").len()
    }
}

#[async_trait]
impl MeshTransport for SynapticMeshAdapter {
    async fn publish(&self, entry: DagEntry) -> PortResult<()> {
        // Commit durably AND queue for delivery — committed state is never lost on churn (NFR-9).
        self.inner
            .committed
            .lock()
            .expect("mesh state poisoned")
            .push(entry.clone());
        self.inner
            .inbox
            .lock()
            .expect("mesh state poisoned")
            .push_back(entry);
        Ok(())
    }

    async fn next_event(&self) -> PortResult<Option<DagEntry>> {
        Ok(self
            .inner
            .inbox
            .lock()
            .expect("mesh state poisoned")
            .pop_front())
    }
}

impl NodeRegistry for SynapticMeshAdapter {
    fn mark_down(&self, node: NodeId) {
        self.inner
            .live
            .lock()
            .expect("mesh state poisoned")
            .remove(node.as_str());
    }

    fn mark_up(&self, node: NodeId) {
        self.inner
            .live
            .lock()
            .expect("mesh state poisoned")
            .insert(node.as_str().to_string());
    }

    fn live(&self) -> Vec<NodeId> {
        let mut nodes: Vec<NodeId> = self
            .inner
            .live
            .lock()
            .expect("mesh state poisoned")
            .iter()
            .map(|s| NodeId::new(s.clone()).expect("stored node ids are valid"))
            .collect();
        nodes.sort_by(|a, b| a.as_str().cmp(b.as_str())); // deterministic order
        nodes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hoforras_domain::{JunctionId, MlDsaSignature, ThermalTradeAgreement, Timestamp};

    fn node(s: &str) -> NodeId {
        NodeId::new(format!("{s}.thermal.budapest.dark")).unwrap()
    }

    fn signed_entry() -> DagEntry {
        DagEntry {
            agreement: ThermalTradeAgreement {
                seller: node("a"),
                buyer: node("b"),
                kwh_offered: 10.0,
                duration_hours: 6,
                credit_price_per_kwh: 2.0,
                pipe_route: vec![JunctionId(1)],
                valid_from: Timestamp(0),
            },
            signature: Some(MlDsaSignature(vec![1, 2, 3])),
        }
    }

    #[tokio::test]
    async fn publish_commits_and_queues() {
        let mesh = SynapticMeshAdapter::new();
        mesh.publish(signed_entry()).await.unwrap();
        assert_eq!(mesh.committed().len(), 1);
        assert_eq!(mesh.pending(), 1);
        assert!(mesh.next_event().await.unwrap().is_some());
        assert!(mesh.next_event().await.unwrap().is_none());
        // Draining the inbox does NOT lose the committed entry (NFR-9).
        assert_eq!(mesh.committed().len(), 1);
    }

    #[test]
    fn registry_tracks_live_membership() {
        let mesh = SynapticMeshAdapter::new();
        mesh.mark_up(node("a"));
        mesh.mark_up(node("b"));
        mesh.mark_down(node("a"));
        assert_eq!(mesh.live(), vec![node("b")]);
    }
}
