//! # hoforras-node (library)
//!
//! The Appliance's node-local contexts. **P1 — Node Isolation & Security (DDD-04)** is implemented
//! here: it is the Open Host Service / Shared Kernel that every other context relies on for
//! capability-gated access and witnessed audit.
//!
//! Per ADR-0004, the coherence/isolation logic is **synchronous and runs in-process, off the tokio
//! async path**. The `rvm-*` substrate is not available in this environment, so the adapters are
//! **reference in-process implementations** behind the `hoforras-ports` traits (ADR-0001 makes them
//! swappable for the real RVM crates later, with no change to the domain units).
//!
//! **Forecasting & Anomaly (DDD-03)** is implemented here too (`forecast`): on-device, edge-only
//! inference (ADR-0012) over ephemeral ruv-swarm agents. **Thermal Memory (DDD-06)** is in `memory`:
//! deterministic 128-dim embeddings + RuVector similarity search / GNN. Both are node-local, within
//! the same RVM boundary as ingestion (ADR-0009).

pub mod appliance;
pub mod forecast;
pub mod isolation;
pub mod memory;
