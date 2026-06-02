//! The federated-learning payload — and the compile-time no-raw-egress wall (ADR-0005 / FR-3.8).
//!
//! [`Gradient`] is the **only** value permitted to cross a building boundary for model training
//! (NFR-7). To make "no raw data crosses" a *compile-time* guarantee rather than a runtime check:
//!
//! * the inner buffer is **private**, so a `Gradient` cannot be fabricated field-by-field from a
//!   [`crate::ThermalFrame`] outside this crate; and
//! * there is intentionally **no** `From<ThermalFrame> for Gradient` (nor any `Into` path).
//!
//! The sole constructor, [`Gradient::from_local`], is named to make the provenance explicit: a
//! gradient is derived from *local* model computation, never from raw frames. The negative
//! guarantee is exercised by `tests/ui/gradient_no_from_thermalframe.rs` (a trybuild compile-fail
//! test).

use serde::{Deserialize, Serialize};

/// A model-update gradient. The only payload that crosses a building boundary (ADR-0005).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Gradient {
    values: Vec<f32>,
}

impl Gradient {
    /// The **only** constructor. A gradient is the product of *local* model computation; there is
    /// deliberately no conversion from any raw sensor type (ADR-0005 / FR-3.8).
    pub fn from_local(values: Vec<f32>) -> Self {
        Self { values }
    }

    pub fn as_slice(&self) -> &[f32] {
        &self.values
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

// NOTE: Do NOT add `impl From<ThermalFrame> for Gradient` or any `From<…sensor type…>`.
// Doing so would defeat ADR-0005 and the trybuild compile-fail test would start failing.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gradient_round_trips_and_only_constructs_from_local() {
        let g = Gradient::from_local(vec![0.1, 0.2, 0.3]);
        assert_eq!(g.len(), 3);
        assert_eq!(g.as_slice(), &[0.1, 0.2, 0.3]);
    }
}
