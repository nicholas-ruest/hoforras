//! Trade Consensus ports (DDD-05). ACL over QuDAG (ADR-0011). Crypto sync; networking async.

use async_trait::async_trait;
use hoforras_domain::{DagEntry, Finality, NodeAddr, ThermalTradeAgreement};

use crate::PortResult;

/// Post-quantum ML-DSA signing/verification of DAG entries (FR-5.1/5.2). Sync (crypto).
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait MlDsaSigner: Send + Sync {
    fn sign(&self, entry: DagEntry) -> PortResult<DagEntry>;
    fn verify(&self, entry: &DagEntry) -> bool;
}

/// Broadcasts a signed entry and awaits QR-Avalanche consensus (FR-5.3 / NFR-2). Async.
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
#[async_trait]
pub trait DagNetwork: Send + Sync {
    async fn broadcast_and_await_consensus(&self, entry: DagEntry) -> PortResult<Finality>;
}

/// Directory-less peer discovery over `.dark` domains (FR-5.4). Async.
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
#[async_trait]
pub trait PeerDiscovery: Send + Sync {
    async fn resolve(&self, dark_domain: String) -> PortResult<NodeAddr>;
}

/// The broker-facing consensus boundary (DDD-01 §7 — Anti-Corruption Layer over Trade Consensus).
/// `BrokerAgent` finalizes an accepted agreement here **before** executing/routing it (FR-5.1/5.3);
/// the concrete `hoforras-mesh::ConsensusGateway` satisfies this contract (wired in P10).
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
#[async_trait]
pub trait ConsensusGateway: Send + Sync {
    async fn finalize(&self, agreement: ThermalTradeAgreement) -> PortResult<Finality>;
}
