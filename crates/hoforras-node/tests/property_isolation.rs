//! Property tests for the two safety invariants of Node Isolation & Security.
//!
//! * FR-4.4 / ADR-0008 — no-raw-egress: for ANY capability and target, a `Scope::Raw` request is
//!   always denied AND the capability gate is never invoked.
//! * FR-4.3 — audit completeness & tamper-evidence: any recorded sequence verifies; corrupting any
//!   record makes verification fail; every record is 64 bytes.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use hoforras_domain::{
    AccessDecision, AccessRequest, Capability, Expiry, NodeId, PrivilegedAction, Rights, Scope,
    Timestamp,
};
use hoforras_node::isolation::adapters::RvmWitnessAdapter;
use hoforras_node::isolation::{AuditTrail, CapabilityBroker};
use hoforras_ports::security::CapabilityGate;
use hoforras_ports::time::Clock;
use proptest::prelude::*;

/// A capability gate that counts how many times it is consulted — to prove it is NOT consulted for
/// raw-scope requests (ADR-0008). NOT a mockall mock: we read the counter after the property run.
struct CountingGate {
    calls: Arc<AtomicUsize>,
}
impl CapabilityGate for CountingGate {
    fn authorize(&self, _cap: &Capability, _req: &AccessRequest) -> AccessDecision {
        self.calls.fetch_add(1, Ordering::SeqCst);
        AccessDecision::Allow
    }
}

/// Fixed clock for the broker (capability never treated as expired here).
struct FixedClock(u64);
impl Clock for FixedClock {
    fn now(&self) -> Timestamp {
        Timestamp(self.0)
    }
}

fn action_of(tag: u8) -> PrivilegedAction {
    match tag % 6 {
        0 => PrivilegedAction::TradeSigned,
        1 => PrivilegedAction::Exec,
        2 => PrivilegedAction::Routing,
        3 => PrivilegedAction::ReadingAccepted,
        4 => PrivilegedAction::CrossPartitionRead,
        _ => PrivilegedAction::GradientAggregated,
    }
}

proptest! {
    /// ADR-0008 / FR-4.4: Raw scope is ALWAYS denied and the gate is NEVER consulted.
    #[test]
    fn prop_raw_scope_always_denied_and_gate_never_called(
        requester in "[a-z]{1,8}",
        target in "[a-z]{1,8}",
        cap_scope_raw in any::<bool>(),
        expiry_ns in 0u64..u64::MAX,
    ) {
        let calls = Arc::new(AtomicUsize::new(0));
        let gate = CountingGate { calls: calls.clone() };
        let broker = CapabilityBroker::new(gate, RvmWitnessAdapter::new(), FixedClock(0));

        let req = AccessRequest {
            requester: NodeId::new(format!("{requester}.thermal.budapest.dark")).unwrap(),
            target: NodeId::new(format!("{target}.thermal.budapest.dark")).unwrap(),
            scope: Scope::Raw,
        };
        let capability = Capability {
            rights: Rights::Read,
            // The capability's own scope is arbitrary — it must not matter.
            scope: if cap_scope_raw { Scope::Raw } else { Scope::AggregatedThermalAvailability },
            expiry: Expiry::At(Timestamp(expiry_ns)),
        };

        let result = broker.request_cross_partition(req, capability);
        prop_assert!(result.is_err());                       // always denied
        prop_assert_eq!(calls.load(Ordering::SeqCst), 0);    // gate NEVER consulted
    }

    /// FR-4.3: a recorded chain verifies, every record is 64 bytes, and corrupting any record
    /// breaks verification.
    #[test]
    fn prop_audit_completeness_and_tamper(
        tags in proptest::collection::vec(any::<u8>(), 1..24),
        corrupt_pick in any::<usize>(),
    ) {
        let audit = AuditTrail::new(RvmWitnessAdapter::new());
        for &t in &tags {
            let rec = audit.record(action_of(t)).unwrap();
            prop_assert_eq!(rec.as_bytes().len(), 64);       // 64-byte records
        }
        prop_assert!(audit.verify_integrity().unwrap());     // intact chain verifies

        let idx = corrupt_pick % tags.len();
        audit.chain().corrupt_record(idx);                   // test-support hook on the reference adapter
        prop_assert!(!audit.verify_integrity().unwrap());    // tamper detected
    }
}
