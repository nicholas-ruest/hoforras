//! Reference adapter for the Thermal Memory ports (DDD-06).
//!
//! The real `ruvector` substrate (HNSW/DiskANN + native GNN) is unavailable here, so
//! [`RuVectorAdapter`] is a reference in-process implementation behind the `VectorIndex` /
//! `GnnEngine` traits — swappable per ADR-0001. It is node-local and stores embeddings/aggregates
//! only, never raw frames (ADR-0009).

pub mod ruvector;

pub use ruvector::RuVectorAdapter;
