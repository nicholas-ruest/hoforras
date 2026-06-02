//! Domain-level error type. Ports return `Result<_, DomainError>` (see `hoforras-ports`).

use thiserror::Error;

/// Errors expressible purely in domain terms. Adapters map vendor errors into these.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum DomainError {
    /// Input failed validation at a system boundary (NFR-12).
    #[error("invalid input: {0}")]
    Invalid(String),

    /// A cross-partition access was denied (FR-4.4 / ADR-0008).
    #[error("access denied: {0}")]
    Denied(String),

    /// A governance rule was violated (FR-3.7 / ADR-0015).
    #[error("governance violation: {0}")]
    Governance(String),

    /// A serialization / canonical-bytes failure (ADR-0014).
    #[error("serialization error: {0}")]
    Serialization(String),

    /// A signature or consensus verification failed (FR-5.5).
    #[error("verification failed: {0}")]
    Verification(String),

    /// An upstream adapter (Ruv crate) reported a failure.
    #[error("adapter error: {0}")]
    Adapter(String),

    /// A requested resource was not found.
    #[error("not found: {0}")]
    NotFound(String),
}
