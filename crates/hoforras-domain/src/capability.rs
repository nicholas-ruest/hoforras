//! Node Isolation & Security value objects (DDD-04) — the Shared Kernel for capabilities.

use serde::{Deserialize, Serialize};

use crate::ids::{NodeId, Timestamp};

/// Access rights granted by a capability. The pilot needs only `Read`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rights {
    Read,
}

/// What a capability grants access to. `Raw` MUST NOT cross a partition boundary
/// (FR-4.4 / NFR-7); only `AggregatedThermalAvailability` may (ADR-0008).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Scope {
    Raw,
    AggregatedThermalAvailability,
}

/// Capability expiry. Capabilities are short-lived (e.g. 6h) to bound exposure.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Expiry {
    /// Absolute expiry timestamp (ns). Checked against an injected `Clock` (deterministic in tests).
    At(Timestamp),
}

/// An unforgeable, scoped, expiring capability token (rvm-cap; modeled here vendor-free).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Capability {
    pub rights: Rights,
    pub scope: Scope,
    pub expiry: Expiry,
}

impl Capability {
    /// True if this capability has expired as of `now`.
    pub fn is_expired(&self, now: Timestamp) -> bool {
        match self.expiry {
            Expiry::At(t) => now > t,
        }
    }
}

/// A request to read across a partition boundary (DDD-04).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccessRequest {
    pub requester: NodeId,
    pub target: NodeId,
    pub scope: Scope,
}

/// The capability gate's decision (distinct from the market `Decision`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccessDecision {
    Allow,
    Deny(String),
}
