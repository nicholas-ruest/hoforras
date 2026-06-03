//! Thermal Recovery Planning ports (DDD-10 / ADR-0016) — the advisory plane.
//!
//! Kept off the live operational path: these are decision-support, not control. `LossAnalyzer` runs
//! node-local and emits only aggregated [`ThermalLossSite`]s (no raw frame crosses — ADR-0009);
//! `RecoveryPlanner` assembles a [`RecoveryReport`] from those aggregates + the public district
//! graph; `RecoveryPlanService` is the read API the dashboard's Planner mode consumes.

use async_trait::async_trait;
use hoforras_domain::{DistrictGraph, RecoveryReport, ThermalLossSite};

use crate::PortResult;

/// Node-local thermal-loss analysis (FR-10.1). Implemented inside `hoforras-node` so raw frames stay
/// node-local; the **output** is an aggregate that may cross a boundary.
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait LossAnalyzer: Send + Sync {
    fn analyze_losses(&self) -> PortResult<Vec<ThermalLossSite>>;
}

/// Assemble a recovery report from loss aggregates + the district graph (FR-10.2–10.5). Pure /
/// deterministic — no I/O, swappable estimator (ADR-0001).
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait RecoveryPlanner: Send + Sync {
    fn plan(
        &self,
        losses: Vec<ThermalLossSite>,
        graph: &DistrictGraph,
    ) -> PortResult<RecoveryReport>;
}

/// The advisory read API consumed by the dashboard's Recovery Planner mode (DDD-09 extension).
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
#[async_trait]
pub trait RecoveryPlanService: Send + Sync {
    async fn report(&self) -> PortResult<RecoveryReport>;
}
