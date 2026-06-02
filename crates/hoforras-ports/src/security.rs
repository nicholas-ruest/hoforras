//! Node Isolation & Security ports (DDD-04). All synchronous: RVM coherence runs in-process,
//! off the async path (ADR-0004); crypto/capability checks are synchronous.

use hoforras_domain::{
    AccessDecision, AccessRequest, Capability, DistrictGraph, NodeId, Partitioning,
    PrivilegedAction, WitnessRecord,
};

use crate::PortResult;

/// Recomputes the coherence mincut over the node graph on an anomaly (FR-4.2). In-process.
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait MincutEngine: Send + Sync {
    fn recompute(&self, graph: &DistrictGraph) -> Partitioning;
}

/// Isolates / rejoins a node's partition (FR-4.2). In-process, no restart.
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait PartitionController: Send + Sync {
    fn isolate(&self, node: NodeId);
    fn rejoin(&self, node: NodeId);
}

/// The hash-chained witness/audit trail (FR-4.3).
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait WitnessChain: Send + Sync {
    fn emit(&self, action: PrivilegedAction) -> PortResult<WitnessRecord>;
    fn verify(&self) -> PortResult<bool>;
}

/// Authorizes cross-partition reads (FR-4.4). The `CapabilityBroker` (Prompt 1) denies `Raw`
/// scope BEFORE calling this (ADR-0008).
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait CapabilityGate: Send + Sync {
    fn authorize(&self, capability: &Capability, request: &AccessRequest) -> AccessDecision;
}
