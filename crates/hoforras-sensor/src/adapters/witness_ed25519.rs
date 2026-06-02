//! `Ed25519WitnessAdapter` — Ed25519 frame witnessing (FR-1.3 / ADR-0014).
//!
//! Implements [`WitnessSigner`] using an `ed25519_dalek::SigningKey`. The pipeline signs the
//! `canonical_bytes` (postcard, ADR-0014) of a frame's content; the corresponding
//! [`verifying_key`](Ed25519WitnessAdapter::verifying_key) lets downstream verify the witness.
//! A reference adapter standing in for a hardware/keystore-backed signer — swappable per ADR-0001.

use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use hoforras_domain::{DomainError, Ed25519Signature};
use hoforras_ports::sensor::WitnessSigner;

/// Holds an Ed25519 signing key and signs canonical frame bytes.
pub struct Ed25519WitnessAdapter {
    signing_key: SigningKey,
}

impl Ed25519WitnessAdapter {
    /// Construct from an existing `SigningKey`.
    pub fn new(signing_key: SigningKey) -> Self {
        Self { signing_key }
    }

    /// Construct deterministically from a 32-byte seed (test fixtures, derived keys).
    pub fn from_seed(seed: &[u8; 32]) -> Self {
        Self {
            signing_key: SigningKey::from_bytes(seed),
        }
    }

    /// The public verifying key, for verifying emitted witnesses (contract tests / peers).
    pub fn verifying_key(&self) -> VerifyingKey {
        self.signing_key.verifying_key()
    }
}

impl WitnessSigner for Ed25519WitnessAdapter {
    fn sign(&self, bytes: &[u8]) -> Result<Ed25519Signature, DomainError> {
        let sig = self.signing_key.sign(bytes);
        Ok(Ed25519Signature(sig.to_bytes()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signature, Verifier};

    #[test]
    fn sign_produces_verifiable_signature() {
        let adapter = Ed25519WitnessAdapter::from_seed(&[9u8; 32]);
        let msg = b"thermal frame canonical bytes";
        let witness = adapter.sign(msg).unwrap();
        let sig = Signature::from_bytes(&witness.0);
        assert!(adapter.verifying_key().verify(msg, &sig).is_ok());
    }
}
