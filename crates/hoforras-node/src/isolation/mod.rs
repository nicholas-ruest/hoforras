//! # Node Isolation & Security (DDD-04)
//!
//! Three domain units — all **synchronous** (ADR-0004, off the async path) — plus reference
//! in-process adapters behind the `hoforras-ports` security/time traits:
//!
//! * [`CoherenceSupervisor`] — on an anomaly: recompute the mincut, isolate the node, witness it
//!   (FR-4.2). Benign signals do nothing.
//! * [`CapabilityBroker`] — mediates cross-partition reads; **denies `Scope::Raw` before consulting
//!   the gate** and rejects expired capabilities against an injected clock (ADR-0008 / FR-4.4).
//! * [`AuditTrail`] — record/verify over a 64-byte hash-chained witness chain (FR-4.3).

pub mod adapters;
pub mod audit;
pub mod capability;
pub mod coherence;

pub use adapters::{RvmCapAdapter, RvmCoherenceAdapter, RvmWitnessAdapter};
pub use audit::AuditTrail;
pub use capability::{AggregateAvailability, CapabilityBroker};
pub use coherence::{AnomalySignal, CoherenceSupervisor};
