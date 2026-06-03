//! `MeshCoordinator` (DDD-07 / FR-7.1–7.3) — district-scale propagation and self-heal.
//!
//! Cross-node propagation is **always** a signed DAG entry over `MeshTransport.publish` — never RPC
//! (ADR-0007; there is no RPC port). On a node drop it marks the node down and reconfigures over the
//! live set; on return it marks the node up and resyncs by replaying missed DAG entries (FR-7.2 /
//! NFR-9 — committed state survives churn). Collective signals dispatch to their mitigation
//! behaviour (FR-7.3).

use hoforras_domain::{CollectiveSignal, DagEntry, DomainError, MeshAction, NodeEvent, NodeId};
use hoforras_ports::mesh::{MeshTransport, NodeRegistry};
use hoforras_ports::PortResult;

/// The observable outcome of handling a [`NodeEvent`].
#[derive(Clone, Debug, PartialEq)]
pub enum MeshReaction {
    /// A drop was handled: the mesh reconfigured over the surviving live set.
    Reconfigured { live: Vec<NodeId> },
    /// A return was handled: the node resynced by replaying these missed entries.
    Resynced { replayed: Vec<DagEntry> },
}

/// Generic over its two ports so it is unit-tested against mocks (London-School, ADR-0001).
pub struct MeshCoordinator<T, R> {
    transport: T,
    registry: R,
}

impl<T, R> MeshCoordinator<T, R>
where
    T: MeshTransport,
    R: NodeRegistry,
{
    pub fn new(transport: T, registry: R) -> Self {
        Self {
            transport,
            registry,
        }
    }

    /// Propagate committed state across the district as a **signed** DAG entry (FR-7.1). Unsigned
    /// entries are refused — only signed, tamper-evident state enters the fabric (ADR-0007, dovetails
    /// with ADR-0011). Publication is over `MeshTransport`, never RPC.
    pub async fn propagate(&self, entry: DagEntry) -> PortResult<()> {
        if entry.signature.is_none() {
            return Err(DomainError::Verification(
                "refusing to propagate an unsigned DAG entry".into(),
            ));
        }
        self.transport.publish(entry).await
    }

    /// React to a membership change (FR-7.2). `Drop` ⇒ mark down then reconfigure over the live set;
    /// `Up` ⇒ mark up then resync (replay missed entries). Committed state is never lost (NFR-9).
    pub async fn on_node_event(&self, event: NodeEvent) -> PortResult<MeshReaction> {
        match event {
            NodeEvent::Drop(node) => {
                self.registry.mark_down(node); // london: mark_down
                let live = self.registry.live(); // reconfigure over survivors
                Ok(MeshReaction::Reconfigured { live })
            }
            NodeEvent::Up(node) => {
                self.registry.mark_up(node); // london: mark_up
                let replayed = self.resync().await?; // replay missed DAG entries
                Ok(MeshReaction::Resynced { replayed })
            }
        }
    }

    /// Replay every missed DAG entry from the fabric until caught up (eventual consistency, NFR-9).
    async fn resync(&self) -> PortResult<Vec<DagEntry>> {
        let mut replayed = Vec::new();
        while let Some(entry) = self.transport.next_event().await? {
            replayed.push(entry);
        }
        Ok(replayed)
    }

    /// Dispatch a collective signal to its mitigation behaviour (FR-7.3). Pure domain decision.
    pub fn collective_behaviour(&self, signal: CollectiveSignal) -> MeshAction {
        match signal {
            CollectiveSignal::Overload { zone } => MeshAction::RebalanceLoad { zone },
            CollectiveSignal::CascadeRisk { path } => MeshAction::ShedAndReroute { path },
            CollectiveSignal::HeatWave => MeshAction::EmergencyRouting,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hoforras_domain::JunctionId;

    // collective_behaviour is pure — exercise the full dispatch table without any ports.
    fn coordinator() -> MeshCoordinator<DummyTransport, DummyRegistry> {
        MeshCoordinator::new(DummyTransport, DummyRegistry)
    }

    struct DummyTransport;
    #[async_trait::async_trait]
    impl MeshTransport for DummyTransport {
        async fn publish(&self, _entry: DagEntry) -> PortResult<()> {
            Ok(())
        }
        async fn next_event(&self) -> PortResult<Option<DagEntry>> {
            Ok(None)
        }
    }
    struct DummyRegistry;
    impl NodeRegistry for DummyRegistry {
        fn mark_down(&self, _node: NodeId) {}
        fn mark_up(&self, _node: NodeId) {}
        fn live(&self) -> Vec<NodeId> {
            vec![]
        }
    }

    #[test]
    fn collective_dispatch_table() {
        let c = coordinator();
        assert_eq!(
            c.collective_behaviour(CollectiveSignal::Overload {
                zone: "north".into()
            }),
            MeshAction::RebalanceLoad {
                zone: "north".into()
            }
        );
        assert_eq!(
            c.collective_behaviour(CollectiveSignal::CascadeRisk {
                path: vec![JunctionId(7)]
            }),
            MeshAction::ShedAndReroute {
                path: vec![JunctionId(7)]
            }
        );
        assert_eq!(
            c.collective_behaviour(CollectiveSignal::HeatWave),
            MeshAction::EmergencyRouting
        );
    }
}
