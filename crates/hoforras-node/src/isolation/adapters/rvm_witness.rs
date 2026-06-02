//! Reference `WitnessChain` adapter — a SHA-512 hash chain (ADR-0014 / FR-4.3).
//!
//! Each record = `SHA-512(prev_record_bytes || canonical_bytes(action))`. SHA-512 yields exactly
//! 64 bytes, matching `WITNESS_RECORD_LEN` (research §93). Because each record commits to the
//! previous one, mutating any stored record makes `verify()` recompute a mismatch — tamper-evident.
//! Stands in for `rvm-witness`/`rvm-proof` (unavailable); swappable per ADR-0001.

use std::sync::Mutex;

use hoforras_domain::{
    canonical_bytes, DomainError, PrivilegedAction, WitnessRecord, WITNESS_RECORD_LEN,
};
use hoforras_ports::security::WitnessChain;
use sha2::{Digest, Sha512};

#[derive(Default)]
struct ChainState {
    entries: Vec<(PrivilegedAction, WitnessRecord)>,
}

/// In-process hash-chained witness store.
#[derive(Default)]
pub struct RvmWitnessAdapter {
    state: Mutex<ChainState>,
}

impl RvmWitnessAdapter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of records in the chain.
    pub fn len(&self) -> usize {
        self.state
            .lock()
            .expect("witness chain poisoned")
            .entries
            .len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn link(
        prev: &[u8; WITNESS_RECORD_LEN],
        action: &PrivilegedAction,
    ) -> Result<[u8; WITNESS_RECORD_LEN], DomainError> {
        let action_bytes = canonical_bytes(action)?; // postcard (ADR-0014)
        let mut hasher = Sha512::new();
        hasher.update(prev);
        hasher.update(&action_bytes);
        let digest = hasher.finalize(); // 64 bytes
        let mut out = [0u8; WITNESS_RECORD_LEN];
        out.copy_from_slice(&digest);
        Ok(out)
    }

    /// Test-support hook: corrupt a stored record so `verify()` must return `false`. Hidden from
    /// docs; exists because this is a reference adapter standing in for the real `rvm-witness`.
    #[doc(hidden)]
    pub fn corrupt_record(&self, idx: usize) {
        let mut state = self.state.lock().expect("witness chain poisoned");
        if let Some(entry) = state.entries.get_mut(idx) {
            let mut bytes = *entry.1.as_bytes();
            bytes[0] ^= 0xFF;
            entry.1 = WitnessRecord::new(bytes);
        }
    }
}

impl WitnessChain for RvmWitnessAdapter {
    fn emit(&self, action: PrivilegedAction) -> Result<WitnessRecord, DomainError> {
        let mut state = self.state.lock().expect("witness chain poisoned");
        let prev = state
            .entries
            .last()
            .map(|(_, r)| *r.as_bytes())
            .unwrap_or([0u8; WITNESS_RECORD_LEN]);
        let record = WitnessRecord::new(Self::link(&prev, &action)?);
        state.entries.push((action, record));
        Ok(record)
    }

    fn verify(&self) -> Result<bool, DomainError> {
        let state = self.state.lock().expect("witness chain poisoned");
        let mut prev = [0u8; WITNESS_RECORD_LEN];
        for (action, stored) in state.entries.iter() {
            let expected = Self::link(&prev, action)?;
            if &expected != stored.as_bytes() {
                return Ok(false); // tamper detected (FR-4.3)
            }
            prev = *stored.as_bytes();
        }
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_are_64_bytes_and_chain_verifies() {
        let chain = RvmWitnessAdapter::new();
        let rec = chain.emit(PrivilegedAction::Exec).unwrap();
        assert_eq!(rec.as_bytes().len(), 64);
        chain.emit(PrivilegedAction::Routing).unwrap();
        assert!(chain.verify().unwrap());
    }

    #[test]
    fn tamper_breaks_verification() {
        let chain = RvmWitnessAdapter::new();
        chain.emit(PrivilegedAction::TradeSigned).unwrap();
        chain.emit(PrivilegedAction::Exec).unwrap();
        assert!(chain.verify().unwrap());
        chain.corrupt_record(0);
        assert!(!chain.verify().unwrap());
    }
}
