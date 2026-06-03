//! Reference adapter for the District Mesh ports (DDD-07).
//!
//! [`SynapticMeshAdapter`] stands in for the Synaptic-Mesh / QuDAG DAG fabric behind the
//! `MeshTransport` / `NodeRegistry` traits — swappable per ADR-0001. DAG pub/sub only; no RPC seam
//! (ADR-0007).

pub mod synaptic;

pub use synaptic::SynapticMeshAdapter;
