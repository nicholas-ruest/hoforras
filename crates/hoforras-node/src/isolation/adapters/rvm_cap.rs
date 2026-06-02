//! Reference `CapabilityGate` adapter (DDD-04). Stands in for `rvm-security`/`rvm-cap`.
//!
//! Reference policy: grant a `Read` capability scoped to `AggregatedThermalAvailability` when the
//! request asks for exactly that; deny everything else. (The `CapabilityBroker` already rejects
//! `Scope::Raw` and expired capabilities *before* reaching the gate — ADR-0008.)

use hoforras_domain::{AccessDecision, AccessRequest, Capability, Rights, Scope};
use hoforras_ports::security::CapabilityGate;

#[derive(Default)]
pub struct RvmCapAdapter;

impl RvmCapAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl CapabilityGate for RvmCapAdapter {
    fn authorize(&self, capability: &Capability, request: &AccessRequest) -> AccessDecision {
        match (capability.rights, capability.scope, request.scope) {
            (
                Rights::Read,
                Scope::AggregatedThermalAvailability,
                Scope::AggregatedThermalAvailability,
            ) => AccessDecision::Allow,
            _ => AccessDecision::Deny("capability does not grant the requested scope".into()),
        }
    }
}
