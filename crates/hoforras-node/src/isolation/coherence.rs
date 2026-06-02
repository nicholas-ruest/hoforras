//! `CoherenceSupervisor` (FR-4.2 / DDD-04).
//!
//! On an anomalous signal it recomputes the coherence mincut and isolates the offending node,
//! witnessing the action — with no manual step or restart. Benign signals are a no-op. All
//! synchronous and off the async path (ADR-0004).

use hoforras_domain::{DistrictGraph, DomainError, NodeId, Partitioning, PrivilegedAction};
use hoforras_ports::security::{MincutEngine, PartitionController, WitnessChain};

/// An anomaly notification handed to the supervisor. `anomalous` distinguishes a genuine anomaly
/// (injected bad readings, hardware fault, compromised Appliance) from a benign signal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnomalySignal {
    pub node: NodeId,
    pub reason: String,
    anomalous: bool,
}

impl AnomalySignal {
    /// A genuine anomaly that warrants isolation.
    pub fn anomalous(node: NodeId, reason: impl Into<String>) -> Self {
        Self {
            node,
            reason: reason.into(),
            anomalous: true,
        }
    }

    /// A benign signal — no isolation should occur.
    pub fn benign(node: NodeId) -> Self {
        Self {
            node,
            reason: String::new(),
            anomalous: false,
        }
    }

    pub fn is_anomalous(&self) -> bool {
        self.anomalous
    }
}

/// Supervises a building's coherence domain. Generic over its ports so it is unit-tested against
/// mocks (London-School, ADR-0001).
pub struct CoherenceSupervisor<M, P, W> {
    mincut: M,
    partitions: P,
    witness: W,
    graph: DistrictGraph,
}

impl<M, P, W> CoherenceSupervisor<M, P, W>
where
    M: MincutEngine,
    P: PartitionController,
    W: WitnessChain,
{
    pub fn new(mincut: M, partitions: P, witness: W, graph: DistrictGraph) -> Self {
        Self {
            mincut,
            partitions,
            witness,
            graph,
        }
    }

    /// Handle a signal. On an anomaly: recompute the mincut, isolate the node, witness it — in that
    /// order (FR-4.2). On a benign signal: do nothing. Other nodes keep serving (continuity).
    ///
    /// Synchronous by design (ADR-0004): the RVM coherence path is never placed on the async runtime.
    pub fn on_signal(&self, signal: AnomalySignal) -> Result<Option<Partitioning>, DomainError> {
        if !signal.is_anomalous() {
            return Ok(None); // benign ⇒ neither recompute nor isolate
        }

        let partitioning = self.mincut.recompute(&self.graph); // exactly one recompute
        self.partitions.isolate(signal.node.clone()); // exactly one isolate
        self.witness.emit(PrivilegedAction::NodeIsolated {
            node: signal.node,
            reason: signal.reason,
        })?;
        Ok(Some(partitioning))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hoforras_domain::WitnessRecord;
    use hoforras_ports::security::{MockMincutEngine, MockPartitionController, MockWitnessChain};

    fn node() -> NodeId {
        NodeId::new("pozsonyi14.thermal.budapest.dark").unwrap()
    }

    #[test]
    fn anomaly_triggers_recompute_isolate_witness() {
        let mut mincut = MockMincutEngine::new();
        mincut
            .expect_recompute()
            .times(1)
            .returning(|_| Partitioning::default());
        let mut partitions = MockPartitionController::new();
        partitions.expect_isolate().times(1).return_const(());
        let mut witness = MockWitnessChain::new();
        witness
            .expect_emit()
            .times(1)
            .returning(|_| Ok(WitnessRecord::new([0u8; 64])));

        let sup = CoherenceSupervisor::new(
            mincut,
            partitions,
            witness,
            DistrictGraph {
                nodes: vec![node()],
                edges: vec![],
            },
        );
        assert!(sup
            .on_signal(AnomalySignal::anomalous(node(), "injected reading"))
            .unwrap()
            .is_some());
    }
}
