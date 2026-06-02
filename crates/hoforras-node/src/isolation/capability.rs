//! `CapabilityBroker` (FR-4.4 / ADR-0008 / DDD-04) — the no-raw-egress gate.
//!
//! The single most important runtime invariant in the system: raw sensor data never crosses a
//! partition boundary. The broker enforces this by **denying `Scope::Raw` before it even consults
//! the capability gate** (defense in depth — ADR-0008), and by rejecting expired capabilities
//! against an injected clock (deterministic in tests). Synchronous.

use hoforras_domain::{
    AccessDecision, AccessRequest, Capability, DomainError, PrivilegedAction, Scope,
};
use hoforras_ports::security::{CapabilityGate, WitnessChain};
use hoforras_ports::time::Clock;

/// Aggregated thermal availability — the only payload permitted to cross a partition boundary.
/// (Raw frames never do; this is the aggregate referenced by `Scope::AggregatedThermalAvailability`.)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AggregateAvailability {
    pub available_kwh: f32,
}

/// Mediates capability-gated cross-partition reads. Generic over its ports for London-School tests.
pub struct CapabilityBroker<G, W, C> {
    gate: G,
    witness: W,
    clock: C,
}

impl<G, W, C> CapabilityBroker<G, W, C>
where
    G: CapabilityGate,
    W: WitnessChain,
    C: Clock,
{
    pub fn new(gate: G, witness: W, clock: C) -> Self {
        Self {
            gate,
            witness,
            clock,
        }
    }

    /// Authorize and perform a cross-partition read.
    ///
    /// Order is the security contract (ADR-0008):
    /// 1. `Scope::Raw` ⇒ `Err`, and the gate is **never consulted**.
    /// 2. expired capability ⇒ `Err`, gate not consulted.
    /// 3. otherwise delegate to the gate; only `AggregatedThermalAvailability` + `Allow` returns data,
    ///    and that read is witnessed.
    pub fn request_cross_partition(
        &self,
        request: AccessRequest,
        capability: Capability,
    ) -> Result<AggregateAvailability, DomainError> {
        // (1) Hard scope rule — BEFORE the gate. Raw never crosses, regardless of capability.
        if request.scope == Scope::Raw {
            return Err(DomainError::Denied(
                "raw scope must not cross a partition boundary".into(),
            ));
        }

        // (2) Expiry — also before the gate.
        if capability.is_expired(self.clock.now()) {
            return Err(DomainError::Denied("capability expired".into()));
        }

        // (3) Delegate to the capability gate.
        match self.gate.authorize(&capability, &request) {
            AccessDecision::Allow => {
                if request.scope != Scope::AggregatedThermalAvailability {
                    return Err(DomainError::Denied(
                        "only aggregated thermal availability may cross".into(),
                    ));
                }
                self.witness.emit(PrivilegedAction::CrossPartitionRead)?;
                Ok(AggregateAvailability::default())
            }
            AccessDecision::Deny(reason) => Err(DomainError::Denied(reason)),
        }
    }
}
