//! Witness / audit value objects (DDD-04, FR-4.3).

use serde::{Deserialize, Serialize};

use crate::ids::NodeId;

/// Length of a hash-chained witness record, in bytes (research §93 — "64-byte witness records").
pub const WITNESS_RECORD_LEN: usize = 64;

/// A privileged action that must be witnessed (FR-4.3). Used by the broker (trade steps) and the
/// coherence supervisor (isolation).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum PrivilegedAction {
    TradeSigned,
    Exec,
    Routing,
    ReadingAccepted,
    NodeIsolated { node: NodeId, reason: String },
    CrossPartitionRead,
    RuleRejection { rule: crate::RuleId },
    GradientAggregated,
}

/// A 64-byte, hash-chained audit entry (FR-4.3). The chain is verifiable; tampering breaks it.
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct WitnessRecord(#[serde(with = "serde_big_array::BigArray")] pub [u8; WITNESS_RECORD_LEN]);

impl WitnessRecord {
    pub fn new(bytes: [u8; WITNESS_RECORD_LEN]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; WITNESS_RECORD_LEN] {
        &self.0
    }
}

impl PartialEq for WitnessRecord {
    fn eq(&self, other: &Self) -> bool {
        self.0[..] == other.0[..]
    }
}

impl std::fmt::Debug for WitnessRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WitnessRecord(64 bytes)")
    }
}
