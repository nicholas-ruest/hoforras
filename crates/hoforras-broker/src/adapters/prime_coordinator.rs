//! `PrimeCoordinatorAdapter` — Byzantine-fault-tolerant gradient aggregation over
//! `daa-prime-coordinator` (DDD-01 / FR-3.8 / ADR-0005).
//!
//! Aggregates peer gradients with a **coordinate-wise trimmed mean**: per dimension, drop the
//! lowest and highest value before averaging, so a minority of Byzantine (malicious/faulty) peers
//! cannot drag the result to an extreme. The port accepts only `Vec<Gradient>` — no raw frame can
//! reach it (ADR-0005). The real `daa-prime-coordinator` substrate is unavailable; this is a
//! reference implementation behind the `GradientAggregator` port.

use async_trait::async_trait;
use hoforras_domain::{DomainError, Gradient};
use hoforras_ports::broker::GradientAggregator;
use hoforras_ports::PortResult;

#[derive(Default)]
pub struct PrimeCoordinatorAdapter;

impl PrimeCoordinatorAdapter {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl GradientAggregator for PrimeCoordinatorAdapter {
    async fn aggregate(&self, gradients: Vec<Gradient>) -> PortResult<Gradient> {
        if gradients.is_empty() {
            return Err(DomainError::Invalid(
                "cannot aggregate zero gradients".into(),
            ));
        }
        let dim = gradients[0].len();
        if gradients.iter().any(|g| g.len() != dim) {
            return Err(DomainError::Invalid(
                "gradients must share dimensionality".into(),
            ));
        }

        let mut result = Vec::with_capacity(dim);
        for i in 0..dim {
            let mut column: Vec<f32> = gradients.iter().map(|g| g.as_slice()[i]).collect();
            column.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            // Trim the most extreme low/high when there are enough samples (BFT robustness).
            let trimmed: &[f32] = if column.len() > 2 {
                &column[1..column.len() - 1]
            } else {
                &column
            };
            let mean = trimmed.iter().sum::<f32>() / trimmed.len() as f32;
            result.push(mean);
        }
        Ok(Gradient::from_local(result))
    }
}
