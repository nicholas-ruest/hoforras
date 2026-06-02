//! Reference `MincutEngine` + `PartitionController` adapter (FR-4.2 / ADR-0004).
//!
//! Stands in for `rvm-coherence`/`rvm-kernel` (unavailable). Runs in-process. The reference mincut
//! is a simple lowest-weight-edge pass over the graph (the real graph-theoretic mincut is in the
//! RVM substrate); isolation tracks a set of isolated nodes under a mutex.

use std::collections::HashSet;
use std::sync::Mutex;

use hoforras_domain::{DistrictGraph, NodeId, Partitioning};
use hoforras_ports::security::{MincutEngine, PartitionController};

#[derive(Default)]
pub struct RvmCoherenceAdapter {
    isolated: Mutex<HashSet<String>>,
}

impl RvmCoherenceAdapter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_isolated(&self, node: &NodeId) -> bool {
        self.isolated
            .lock()
            .expect("coherence state poisoned")
            .contains(node.as_str())
    }
}

impl MincutEngine for RvmCoherenceAdapter {
    fn recompute(&self, graph: &DistrictGraph) -> Partitioning {
        // Reference mincut: pick the node touched by the lowest-weight edge. Exercises graph-sized
        // work for the benchmark; the production graph-theoretic mincut lives in rvm-coherence.
        if graph.nodes.is_empty() {
            return Partitioning::default();
        }
        let mut min_weight = f32::INFINITY;
        let mut cut: Option<NodeId> = None;
        for (a, _b, w) in graph.edges.iter() {
            if *w < min_weight {
                min_weight = *w;
                let idx = (a.0 as usize) % graph.nodes.len();
                cut = Some(graph.nodes[idx].clone());
            }
        }
        Partitioning {
            isolated: cut.into_iter().collect(),
        }
    }
}

impl PartitionController for RvmCoherenceAdapter {
    fn isolate(&self, node: NodeId) {
        self.isolated
            .lock()
            .expect("coherence state poisoned")
            .insert(node.as_str().to_string());
    }

    fn rejoin(&self, node: NodeId) {
        self.isolated
            .lock()
            .expect("coherence state poisoned")
            .remove(node.as_str());
    }
}
