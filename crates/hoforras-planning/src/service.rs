//! `StaticRecoveryPlanService` (DDD-10 / ADR-0016 §5) — the advisory read API.
//!
//! Wraps a [`RecoveryPlannerService`] over a fixed snapshot of loss aggregates + the district graph
//! and exposes the resulting [`RecoveryReport`] on demand. This is the boundary the dashboard's
//! Recovery Planner mode consumes — a read-only, advisory plane kept off the operational OHS.

use async_trait::async_trait;
use hoforras_domain::{DistrictGraph, RecoveryReport, ThermalLossSite};
use hoforras_ports::recovery::{RecoveryPlanService, RecoveryPlanner};
use hoforras_ports::PortResult;

use crate::planner::RecoveryPlannerService;

/// Advisory read service over a fixed loss/graph snapshot.
pub struct StaticRecoveryPlanService {
    planner: RecoveryPlannerService,
    losses: Vec<ThermalLossSite>,
    graph: DistrictGraph,
}

impl StaticRecoveryPlanService {
    pub fn new(
        planner: RecoveryPlannerService,
        losses: Vec<ThermalLossSite>,
        graph: DistrictGraph,
    ) -> Self {
        Self {
            planner,
            losses,
            graph,
        }
    }
}

#[async_trait]
impl RecoveryPlanService for StaticRecoveryPlanService {
    async fn report(&self) -> PortResult<RecoveryReport> {
        self.planner.plan(self.losses.clone(), &self.graph)
    }
}
