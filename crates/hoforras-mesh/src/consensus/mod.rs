//! # Trade Consensus (DDD-05)
//!
//! The anti-corruption layer over QuDAG that turns a `ThermalTradeAgreement` into a signed,
//! consensus-final DAG entry. A **generic** subdomain: post-quantum DAG consensus is supplied by
//! QuDAG; Hőforrás contributes only the agreement schema and the boundary validation.
//!
//! * [`ConsensusGateway`] — the domain unit: `finalize` (ML-DSA sign **then** broadcast — FR-5.1),
//!   `resolve_peer` (directory-less `.dark` discovery — FR-5.4), `verify` (tamper-evident — FR-5.5).
//! * [`QuDagAdapter`] — real ML-DSA-65 + ML-KEM-1024 behind the consensus ports (ADR-0011), signing
//!   over `postcard` canonical bytes (ADR-0014).

pub mod adapters;
pub mod gateway;

pub use adapters::QuDagAdapter;
pub use gateway::ConsensusGateway;
