//! # hoforras-planning — Thermal Recovery Planning (DDD-10 / ADR-0016)
//!
//! An **advisory, read-only** context that answers the capital-planning question: where is heat
//! being lost, where should Cognitum devices go to harvest it, what does that cost, what does it
//! save, and how much CO₂e does it avoid. It is decision-support — it never controls or trades, and
//! it operates only on aggregated loss + the public district graph (no raw frame; ADR-0009).
//!
//! * [`RecoveryPlannerService`] — the deterministic, transparent estimator (siting + investment +
//!   savings + carbon).
//! * [`StaticRecoveryPlanService`] — the advisory read API the dashboard consumes.

pub mod planner;
pub mod service;

pub use planner::RecoveryPlannerService;
pub use service::StaticRecoveryPlanService;
