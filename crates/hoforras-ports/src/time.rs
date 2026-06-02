//! Time port — an injected clock for deterministic capability-expiry/timestamp tests (DDD-04).

use hoforras_domain::Timestamp;

/// Source of the current time. Injected so capability expiry (ADR-0008) and timestamps are
/// deterministic under test.
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait Clock: Send + Sync {
    fn now(&self) -> Timestamp;
}
