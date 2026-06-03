//! # Thermal Memory (DDD-06)
//!
//! Node-local vector/GNN memory of a building's thermal behaviour, inside the node's RVM boundary
//! (ADR-0009). A **generic** subdomain: storage, similarity search, and GNN are off-the-shelf
//! (RuVector); the only domain artifact is the deterministic 128-dim embedding scheme.
//!
//! * [`ThermalMemory`] — the domain unit: `record_state`, `find_surplus_matches` (top-K + filter),
//!   `district_inference` (GNN), `store_witness_for_replay`. Raw frames are consumed here and never
//!   cross a port — only embeddings/aggregates do (ADR-0009).
//! * [`RuVectorAdapter`] — reference HNSW search (filtered, similarity-ordered, ≤ topK — FR-6.3) and
//!   GNN inference over the district graph.

pub mod adapters;
pub mod service;

pub use adapters::RuVectorAdapter;
pub use service::{embed, embed_witness, ThermalMemory};
