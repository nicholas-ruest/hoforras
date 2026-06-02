//! Identity value objects: `NodeId` (a `.dark` domain), `JunctionId`, `Timestamp`.

use serde::{Deserialize, Serialize};

/// A building/Appliance identity, expressed as a directory-less `.dark` domain
/// (e.g. `pozsonyi14.thermal.budapest.dark`) — FR-5.4 / DDD-05.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(String);

impl NodeId {
    /// Construct from a `.dark` domain string. Validates the namespace suffix at the boundary.
    pub fn new(dark_domain: impl Into<String>) -> Result<Self, crate::DomainError> {
        let s = dark_domain.into();
        if !s.ends_with(".dark") {
            return Err(crate::DomainError::Invalid(format!(
                "NodeId must be a .dark domain, got `{s}`"
            )));
        }
        Ok(Self(s))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A pipe-network junction identifier (used in `pipe_route`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JunctionId(pub u32);

/// Nanosecond wall-clock timestamp. Supplied via an injected `Clock` port for determinism in tests.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Timestamp(pub u64);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_id_requires_dark_domain() {
        assert!(NodeId::new("pozsonyi14.thermal.budapest.dark").is_ok());
        assert!(NodeId::new("pozsonyi14.example.com").is_err());
    }
}
