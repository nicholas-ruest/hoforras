//! Sensing & Ingestion ports (DDD-02). Used by `IngestionPipeline` (built in Prompt 2).

use async_trait::async_trait;
use hoforras_domain::{Ed25519Signature, QualityScore, RawReading, ThermalFrame, ValidationStatus};

use crate::PortResult;

/// Source of raw sensor readings (rvcsi/RuView; adapter is an ACL — DDD-02).
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
#[async_trait]
pub trait SensorSource: Send + Sync {
    async fn poll(&self) -> PortResult<RawReading>;
}

/// Validates a raw reading at the ingestion boundary (FR-1.2).
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait Validator: Send + Sync {
    fn validate(&self, raw: &RawReading) -> ValidationStatus;
}

/// Scores reading confidence/quality (FR-1.4).
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait QualityScorer: Send + Sync {
    fn score(&self, raw: &RawReading) -> QualityScore;
}

/// Ed25519 witnessing of a frame's canonical bytes (FR-1.3 / ADR-0014). Synchronous (crypto).
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait WitnessSigner: Send + Sync {
    fn sign(&self, bytes: &[u8]) -> PortResult<Ed25519Signature>;
}

/// Emits validated, witnessed frames downstream (FR-1.1).
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
#[async_trait]
pub trait EventEmitter: Send + Sync {
    async fn emit(&self, frame: ThermalFrame) -> PortResult<()>;
}

/// Records rejected readings with a reason (FR-1.2).
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait RejectionLog: Send + Sync {
    fn record(&self, raw: RawReading, reason: String);
}
