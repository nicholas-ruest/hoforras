//! # hoforras-mesh — Trade Consensus (DDD-05), District Mesh (DDD-07), AI Orchestration (DDD-08).
//!
//! **Trade Consensus (DDD-05)** is implemented in `consensus`: the `ConsensusGateway` ACL over
//! QuDAG, with a real post-quantum `QuDagAdapter` (ML-DSA-65 signing, ML-KEM-1024 transport,
//! directory-less `.dark` discovery). **District Mesh Coordination (DDD-07)** is in `coordination`:
//! the `MeshCoordinator` propagates signed DAG entries and self-heals across node churn. All
//! cross-node propagation is via signed DAG entries, never RPC (ADR-0007) — there is no RPC port.
//! **AI Orchestration (DDD-08)** is in `ai`: the `ThermalBridgeServer` Open Host Service (ADR-0002)
//! publishing exactly five MCP tools, the `AnomalyExplainer`, and a deferred federation stub.

pub mod ai;
pub mod consensus;
pub mod coordination;
