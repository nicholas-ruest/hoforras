//! Cross-district federation — **stubbed and deferred** (DDD-08 §7 / FR-8.3 / spec risk R-5).
//!
//! Full cross-district expansion authenticates peers via **mTLS + Ed25519** and exchanges **only
//! market signals** — never raw data, the same no-raw-egress spirit as ADR-0005 / ADR-0007. The
//! pilot ships this **stub**: it enforces the market-signals-only rule (rejecting anything that is
//! not a tagged market signal) and otherwise reports that federation is deferred. Full acceptance is
//! a post-pilot SPARC pass.

use hoforras_domain::{DomainError, Json};
use hoforras_ports::PortResult;

/// The disclosed federation stub. Present so the boundary exists and the invariant (signals-only) is
/// testable; the actual cross-district exchange is deferred (R-5).
#[derive(Default)]
pub struct FederationStub;

impl FederationStub {
    pub fn new() -> Self {
        Self
    }

    /// Exchange a cross-district signal. Enforces **market-signals-only** (no raw data may cross),
    /// then reports the exchange as deferred — the pilot does not perform a real federation round.
    pub fn exchange(&self, signal: &Json) -> PortResult<FederationOutcome> {
        // Reject anything that is not an explicitly tagged market signal — defense against raw data
        // ever crossing a district boundary (FR-8.3, same spirit as FR-3.8 / ADR-0005).
        let kind = signal.get("kind").and_then(Json::as_str);
        if kind != Some("market_signal") {
            return Err(DomainError::Denied(
                "federation accepts only tagged market signals; no raw data crosses districts"
                    .into(),
            ));
        }
        Ok(FederationOutcome::Deferred)
    }
}

/// The outcome of a federation exchange in the pilot: always deferred (R-5), but the signal was
/// validated as a market-signal-only payload.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FederationOutcome {
    /// Validated market signal; real cross-district exchange deferred to a post-pilot SPARC pass.
    Deferred,
}
