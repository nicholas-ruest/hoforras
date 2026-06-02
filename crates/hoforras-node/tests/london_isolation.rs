//! London-School interaction tests for Node Isolation & Security (FR-4.2/4.3/4.4).
//!
//! These assert *interactions* — message, order, cardinality, and ABSENCE — against mocked ports.
//! The headline assertion is `capability_raw_scope_denied_without_consulting_gate` (ADR-0008).

use hoforras_domain::{
    AccessDecision, AccessRequest, Capability, DistrictGraph, Expiry, NodeId, Partitioning,
    PrivilegedAction, Rights, Scope, Timestamp, WitnessRecord,
};
use hoforras_node::isolation::{AnomalySignal, AuditTrail, CapabilityBroker, CoherenceSupervisor};
use hoforras_ports::security::{
    MockCapabilityGate, MockMincutEngine, MockPartitionController, MockWitnessChain,
};
use hoforras_ports::time::MockClock;
use mockall::Sequence;

fn node(s: &str) -> NodeId {
    NodeId::new(s).unwrap()
}

fn request(scope: Scope) -> AccessRequest {
    AccessRequest {
        requester: node("a.thermal.budapest.dark"),
        target: node("b.thermal.budapest.dark"),
        scope,
    }
}

fn cap(scope: Scope, expiry_ns: u64) -> Capability {
    Capability {
        rights: Rights::Read,
        scope,
        expiry: Expiry::At(Timestamp(expiry_ns)),
    }
}

// ───────────────────────── CoherenceSupervisor (FR-4.2) ─────────────────────────

#[test]
fn coherence_benign_signal_noop() {
    let mut mincut = MockMincutEngine::new();
    mincut.expect_recompute().times(0); // benign ⇒ never recompute
    let mut partitions = MockPartitionController::new();
    partitions.expect_isolate().times(0); // benign ⇒ never isolate
    let witness = MockWitnessChain::new(); // emit never set ⇒ must never be called

    let sup = CoherenceSupervisor::new(
        mincut,
        partitions,
        witness,
        DistrictGraph {
            nodes: vec![node("x.dark")],
            edges: vec![],
        },
    );
    assert_eq!(
        sup.on_signal(AnomalySignal::benign(node("x.dark")))
            .unwrap(),
        None
    );
}

#[test]
fn coherence_anomaly_recomputes_then_isolates() {
    let mut seq = Sequence::new();
    let mut mincut = MockMincutEngine::new();
    mincut
        .expect_recompute()
        .times(1)
        .in_sequence(&mut seq)
        .returning(|_| Partitioning::default());
    let mut partitions = MockPartitionController::new();
    partitions
        .expect_isolate()
        .times(1)
        .in_sequence(&mut seq)
        .return_const(());
    let mut witness = MockWitnessChain::new();
    witness
        .expect_emit()
        .times(1)
        .returning(|_| Ok(WitnessRecord::new([0u8; 64])));

    let sup = CoherenceSupervisor::new(
        mincut,
        partitions,
        witness,
        DistrictGraph {
            nodes: vec![node("x.dark")],
            edges: vec![],
        },
    );
    sup.on_signal(AnomalySignal::anomalous(node("x.dark"), "injected reading"))
        .unwrap();
}

#[test]
fn coherence_isolation_is_witnessed_as_node_isolated() {
    let mut mincut = MockMincutEngine::new();
    mincut
        .expect_recompute()
        .returning(|_| Partitioning::default());
    let mut partitions = MockPartitionController::new();
    partitions.expect_isolate().return_const(());
    let mut witness = MockWitnessChain::new();
    witness
        .expect_emit()
        .times(1)
        .withf(|a| matches!(a, PrivilegedAction::NodeIsolated { .. }))
        .returning(|_| Ok(WitnessRecord::new([0u8; 64])));

    let sup = CoherenceSupervisor::new(
        mincut,
        partitions,
        witness,
        DistrictGraph {
            nodes: vec![node("x.dark")],
            edges: vec![],
        },
    );
    sup.on_signal(AnomalySignal::anomalous(node("x.dark"), "fault"))
        .unwrap();
}

// ───────────────────────── CapabilityBroker (FR-4.4 / ADR-0008) ─────────────────────────

#[test]
fn capability_raw_scope_denied_without_consulting_gate() {
    let mut gate = MockCapabilityGate::new();
    gate.expect_authorize().times(0); // ADR-0008: gate NEVER consulted for Raw
    let witness = MockWitnessChain::new(); // emit never set ⇒ never called
    let mut clock = MockClock::new();
    clock.expect_now().times(0); // Raw is rejected before even the clock is read

    let broker = CapabilityBroker::new(gate, witness, clock);
    let res = broker.request_cross_partition(request(Scope::Raw), cap(Scope::Raw, 1_000));
    assert!(res.is_err());
}

#[test]
fn capability_expired_capability_denied_without_gate() {
    let mut gate = MockCapabilityGate::new();
    gate.expect_authorize().times(0);
    let witness = MockWitnessChain::new();
    let mut clock = MockClock::new();
    clock.expect_now().times(1).return_const(Timestamp(2_000)); // now > expiry

    let broker = CapabilityBroker::new(gate, witness, clock);
    let res = broker.request_cross_partition(
        request(Scope::AggregatedThermalAvailability),
        cap(Scope::AggregatedThermalAvailability, 1_000),
    );
    assert!(res.is_err());
}

#[test]
fn capability_aggregate_allow_returns_payload_and_is_witnessed() {
    let mut gate = MockCapabilityGate::new();
    gate.expect_authorize()
        .times(1)
        .returning(|_, _| AccessDecision::Allow);
    let mut witness = MockWitnessChain::new();
    witness
        .expect_emit()
        .times(1)
        .withf(|a| matches!(a, PrivilegedAction::CrossPartitionRead))
        .returning(|_| Ok(WitnessRecord::new([0u8; 64])));
    let mut clock = MockClock::new();
    clock.expect_now().return_const(Timestamp(500)); // not expired

    let broker = CapabilityBroker::new(gate, witness, clock);
    let res = broker.request_cross_partition(
        request(Scope::AggregatedThermalAvailability),
        cap(Scope::AggregatedThermalAvailability, 1_000),
    );
    assert!(res.is_ok());
}

#[test]
fn capability_gate_deny_returns_err() {
    let mut gate = MockCapabilityGate::new();
    gate.expect_authorize()
        .times(1)
        .returning(|_, _| AccessDecision::Deny("nope".into()));
    let witness = MockWitnessChain::new(); // not witnessed on deny
    let mut clock = MockClock::new();
    clock.expect_now().return_const(Timestamp(500));

    let broker = CapabilityBroker::new(gate, witness, clock);
    let res = broker.request_cross_partition(
        request(Scope::AggregatedThermalAvailability),
        cap(Scope::AggregatedThermalAvailability, 1_000),
    );
    assert!(res.is_err());
}

// ───────────────────────── AuditTrail (FR-4.3) ─────────────────────────

#[test]
fn audit_record_and_verify_delegate_to_chain() {
    let mut chain = MockWitnessChain::new();
    chain
        .expect_emit()
        .times(1)
        .returning(|_| Ok(WitnessRecord::new([1u8; 64])));
    chain.expect_verify().times(1).returning(|| Ok(true));

    let audit = AuditTrail::new(chain);
    audit.record(PrivilegedAction::Exec).unwrap();
    assert!(audit.verify_integrity().unwrap());
}
