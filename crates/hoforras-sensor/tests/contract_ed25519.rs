//! Contract tests for the real `Ed25519WitnessAdapter` (FR-1.3 / ADR-0014).
//!
//! Unlike the London interaction tests, these exercise the *real* crypto adapter end-to-end: sign
//! with the adapter, then verify with its verifying key. No mocks.

use ed25519_dalek::{Signature, Verifier};
use hoforras_ports::sensor::WitnessSigner;
use hoforras_sensor::Ed25519WitnessAdapter;

#[test]
fn sign_then_verify_roundtrip() {
    let adapter = Ed25519WitnessAdapter::from_seed(&[1u8; 32]);
    let msg = b"canonical thermal frame bytes (postcard)";
    let witness = adapter.sign(msg).unwrap();

    let sig = Signature::from_bytes(&witness.0);
    assert!(adapter.verifying_key().verify(msg, &sig).is_ok());
}

#[test]
fn tampered_message_fails_verify() {
    let adapter = Ed25519WitnessAdapter::from_seed(&[2u8; 32]);
    let msg = b"original frame";
    let witness = adapter.sign(msg).unwrap();

    let sig = Signature::from_bytes(&witness.0);
    let tampered = b"different frame";
    assert!(adapter.verifying_key().verify(tampered, &sig).is_err());
}
