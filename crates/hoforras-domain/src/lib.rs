//! # hoforras-domain
//!
//! The pure, dependency-free heart of Hőforrás: the **ubiquitous language** (DDD-00) and every
//! shared value type from `sparc.md` Part 1 §10. No I/O, no async, and — critically — **no
//! dependency on any Ruv crate** (ADR-0002 / ADR-0003), enforced by `scripts/layering-lint.sh`.
//!
//! Anything that is signed or hashed is serialized with [`canonical_bytes`] (postcard), giving a
//! deterministic, reproducible encoding (ADR-0014).

pub mod canonical;
pub mod capability;
pub mod consensus;
pub mod error;
pub mod forecast;
pub mod governance;
pub mod gradient;
pub mod ids;
pub mod isolation;
pub mod market;
pub mod memory;
pub mod mesh;
pub mod sensor;
pub mod witness;

pub use canonical::canonical_bytes;
pub use error::DomainError;

// Flat re-exports so downstream crates can `use hoforras_domain::ThermalFrame;` etc.
pub use capability::{AccessDecision, AccessRequest, Capability, Expiry, Rights, Scope};
pub use consensus::{DagEntry, Finality, MlDsaSignature, NodeAddr, ThermalTradeAgreement};
pub use forecast::{
    AgentProfile, Anomaly, AnomalyKind, BurstWarning, DemandForecast, Inference, ReadingWindow,
};
pub use governance::{GovernanceRules, RuleId, RuleVerdict, GOVERNANCE};
pub use gradient::Gradient;
pub use ids::{JunctionId, NodeId, Timestamp};
pub use isolation::Partitioning;
pub use market::{
    AcceptedTrade, Adaptation, Bid, Decision, ExecutedTrade, Kwh, Offer, Price, ProposedTrade,
    Reflection, Strategy, ThermalBalance, TradeOutcome, TradeWindow,
};
pub use memory::{Candidate, DistrictGraph, Embedding, GnnState, Query, EMBEDDING_DIM};
pub use mesh::{AnomalyContext, CollectiveSignal, Explanation, MeshAction, NodeEvent, UiEvent};
pub use sensor::{Ed25519Signature, QualityScore, RawReading, ThermalFrame, ValidationStatus};
pub use witness::{PrivilegedAction, WitnessRecord, WITNESS_RECORD_LEN};

/// A JSON value, used at the MCP tool boundary (DDD-08). `serde_json` is not a Ruv crate.
pub type Json = serde_json::Value;
