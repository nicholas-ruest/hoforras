//! # District Mesh Coordination (DDD-07)
//!
//! The district-scale fabric that propagates committed state and enables collective, self-healing
//! behaviour across Appliances — over signed DAG entries, never RPC (ADR-0007).
//!
//! * [`MeshCoordinator`] — the domain unit: `propagate` (publish a signed DAG entry), `on_node_event`
//!   (self-heal on drop/return), `collective_behaviour` (FR-7.3 dispatch).
//! * [`SynapticMeshAdapter`] — reference DAG fabric + node registry with a durable committed log that
//!   survives node churn (NFR-9).

pub mod adapters;
pub mod coordinator;

pub use adapters::SynapticMeshAdapter;
pub use coordinator::{MeshCoordinator, MeshReaction};
