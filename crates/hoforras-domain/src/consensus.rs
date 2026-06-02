//! Trade Consensus value objects (DDD-05, FR-5).

use serde::{Deserialize, Serialize};

use crate::ids::{JunctionId, NodeId, Timestamp};

/// A cross-building thermal trade agreement, encoded as a signed DAG entry (FR-5.1).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ThermalTradeAgreement {
    pub seller: NodeId,
    pub buyer: NodeId,
    pub kwh_offered: f32,
    pub duration_hours: u32,
    pub credit_price_per_kwh: f32,
    pub pipe_route: Vec<JunctionId>,
    pub valid_from: Timestamp,
}

/// An ML-DSA (post-quantum) signature over canonical bytes (FR-5.2 / ADR-0011).
#[derive(Clone, Serialize, Deserialize)]
pub struct MlDsaSignature(pub Vec<u8>);

impl PartialEq for MlDsaSignature {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl std::fmt::Debug for MlDsaSignature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MlDsaSignature({} bytes)", self.0.len())
    }
}

/// A DAG entry: a payload plus an optional signature (set once signed; FR-5.1).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DagEntry {
    pub agreement: ThermalTradeAgreement,
    pub signature: Option<MlDsaSignature>,
}

impl DagEntry {
    pub fn new(agreement: ThermalTradeAgreement) -> Self {
        Self {
            agreement,
            signature: None,
        }
    }
}

/// Consensus finality marker (QR-Avalanche). `reached_at` supports the sub-second NFR-2 check.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Finality {
    pub reached_at: Timestamp,
}

/// A resolved peer network address (from `.dark` discovery; FR-5.4).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeAddr(pub String);
