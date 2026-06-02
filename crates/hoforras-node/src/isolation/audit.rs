//! `AuditTrail` (FR-4.3 / DDD-04) — thin domain service over a hash-chained witness chain.

use hoforras_domain::{DomainError, PrivilegedAction, WitnessRecord};
use hoforras_ports::security::WitnessChain;

/// Records privileged actions to the witness chain and verifies the chain's integrity.
pub struct AuditTrail<W> {
    chain: W,
}

impl<W: WitnessChain> AuditTrail<W> {
    pub fn new(chain: W) -> Self {
        Self { chain }
    }

    /// Witness a privileged action (FR-4.3).
    pub fn record(&self, action: PrivilegedAction) -> Result<WitnessRecord, DomainError> {
        self.chain.emit(action)
    }

    /// Verify the whole chain. Returns `false` if any record was tampered with.
    pub fn verify_integrity(&self) -> Result<bool, DomainError> {
        self.chain.verify()
    }

    /// Borrow the underlying chain (used by tamper tests to corrupt a record).
    pub fn chain(&self) -> &W {
        &self.chain
    }
}
