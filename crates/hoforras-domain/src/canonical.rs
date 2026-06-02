//! Canonical serialization (ADR-0014).
//!
//! Anything that is signed (Ed25519 witnessing, FR-1.3) or hashed (witness chain, FR-4.3) or
//! consensus-signed (ML-DSA, FR-5.1) must have a **deterministic, reproducible** byte encoding so
//! that signatures and hash chains verify across heterogeneous Appliances and software versions.
//! `postcard` provides exactly this; it is the *only* serializer used for those bytes.

use serde::Serialize;

use crate::error::DomainError;

/// Serialize `value` to its canonical byte representation using `postcard`.
///
/// Deterministic: the same value always produces the same bytes (ADR-0014).
pub fn canonical_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, DomainError> {
    postcard::to_allocvec(value).map_err(|e| DomainError::Serialization(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Serialize)]
    struct Sample {
        a: u32,
        b: String,
    }

    #[test]
    fn canonical_bytes_is_deterministic() {
        let s = Sample {
            a: 7,
            b: "hő".into(),
        };
        let first = canonical_bytes(&s).unwrap();
        let second = canonical_bytes(&s).unwrap();
        assert_eq!(first, second, "ADR-0014: encoding must be reproducible");
    }
}
