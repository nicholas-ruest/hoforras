//! Recovery-planning tests (ADR-0016) — example + property + the read-service contract.

use hoforras_domain::{
    DeviceKind, DistrictGraph, JunctionId, LossCause, NodeId, RecoveryReport, ThermalLossSite,
    CAR_CO2E_KG_YR, DEFAULT_RECOVERY_PARAMS, TREE_CO2E_KG_YR,
};
use hoforras_planning::{RecoveryPlannerService, StaticRecoveryPlanService};
use hoforras_ports::recovery::{RecoveryPlanService, RecoveryPlanner};
use proptest::prelude::*;

fn node(s: &str) -> NodeId {
    NodeId::new(format!("{s}.thermal.budapest.dark")).unwrap()
}

fn graph() -> DistrictGraph {
    DistrictGraph {
        nodes: vec![node("pozsonyi14"), node("pozsonyi22")],
        edges: vec![(JunctionId(7), JunctionId(12), 1.0)],
    }
}

fn site(n: &str, j: u32, kwh: f32, conf: f32, cause: LossCause) -> ThermalLossSite {
    ThermalLossSite {
        node_id: node(n),
        junction: JunctionId(j),
        estimated_loss_kwh_yr: kwh,
        confidence: conf,
        cause,
    }
}

fn plan(losses: Vec<ThermalLossSite>) -> RecoveryReport {
    RecoveryPlannerService::default()
        .plan(losses, &graph())
        .unwrap()
}

// ───────────────────────── example tests (FR-10.2–10.5) ─────────────────────────

#[test]
fn high_loss_confident_site_gets_a_recovery_unit() {
    let report = plan(vec![site(
        "pozsonyi14",
        12,
        5000.0,
        0.9,
        LossCause::JunctionLeak,
    )]);
    let kinds: Vec<_> = report.plan.placements.iter().map(|p| p.kind).collect();
    assert!(kinds.contains(&DeviceKind::RecoveryUnit));
    assert!(kinds.contains(&DeviceKind::Appliance)); // one per building
    assert!(report.savings.recoverable_kwh_yr > 0.0);
}

#[test]
fn low_confidence_site_gets_a_seed_sensor_and_is_not_harvested() {
    let report = plan(vec![site(
        "pozsonyi14",
        7,
        5000.0,
        0.2,
        LossCause::UnmeteredSegment,
    )]);
    let kinds: Vec<_> = report.plan.placements.iter().map(|p| p.kind).collect();
    assert!(kinds.contains(&DeviceKind::SeedSensor));
    assert!(!kinds.contains(&DeviceKind::RecoveryUnit));
    // observed (a sensor addresses it) but not yet harvestable.
    assert!(report.plan.observed_loss_kwh_yr > 0.0);
    assert_eq!(report.savings.recoverable_kwh_yr, 0.0);
}

#[test]
fn carbon_equals_recoverable_times_factor_with_equivalents() {
    let report = plan(vec![site(
        "pozsonyi14",
        12,
        10_000.0,
        0.95,
        LossCause::PipeInsulation,
    )]);
    let p = DEFAULT_RECOVERY_PARAMS;
    let expected_co2e = report.savings.recoverable_kwh_yr * p.grid_emission_factor_kg_per_kwh;
    assert!((report.carbon.co2e_kg_yr - expected_co2e).abs() < 1e-2);
    assert!(
        (report.carbon.equivalents.cars_off_road - report.carbon.co2e_kg_yr / CAR_CO2E_KG_YR).abs()
            < 1e-4
    );
    assert!(
        (report.carbon.equivalents.trees_planted - report.carbon.co2e_kg_yr / TREE_CO2E_KG_YR)
            .abs()
            < 1e-3
    );
}

#[test]
fn empty_losses_produce_a_zero_report() {
    let report = plan(vec![]);
    assert!(report.plan.placements.is_empty());
    assert_eq!(report.plan.coverage_pct, 0.0);
    assert_eq!(report.savings.recoverable_kwh_yr, 0.0);
    assert_eq!(report.carbon.co2e_kg_yr, 0.0);
    assert!(report.investment.payback_years.is_infinite()); // nothing to pay back
    assert!(!report.assumptions.is_empty()); // assumptions always disclosed
}

#[test]
fn report_always_discloses_estimate_assumptions() {
    let report = plan(vec![site(
        "pozsonyi14",
        12,
        3000.0,
        0.8,
        LossCause::GroundCoupling,
    )]);
    assert!(report.assumptions.iter().any(|a| a.contains("estimate")));
}

// ───────────────────────── property tests (the kWh→€→CO₂e chain) ─────────────────────────

proptest! {
    /// Conservation + bounds across the whole estimate chain, for any mix of loss sites.
    #[test]
    fn prop_recovery_chain_is_consistent(
        kwhs in proptest::collection::vec(0.0f32..20_000.0, 0..12),
        confs in proptest::collection::vec(0.0f32..1.0, 0..12),
    ) {
        let losses: Vec<ThermalLossSite> = kwhs
            .iter()
            .zip(confs.iter().cycle())
            .enumerate()
            .map(|(i, (kwh, conf))| site("pozsonyi14", i as u32, *kwh, *conf, LossCause::PipeInsulation))
            .collect();
        let total: f32 = losses.iter().map(|s| s.estimated_loss_kwh_yr).sum();
        let report = plan(losses);
        let p = DEFAULT_RECOVERY_PARAMS;

        // coverage is a fraction.
        prop_assert!(report.plan.coverage_pct >= 0.0 && report.plan.coverage_pct <= 1.0);
        // observed loss never exceeds total loss.
        prop_assert!(report.plan.observed_loss_kwh_yr <= total + 1.0);
        // recoverable ≤ observed (you cannot harvest more than you observe).
        prop_assert!(report.savings.recoverable_kwh_yr <= report.plan.observed_loss_kwh_yr + 1.0);
        // savings = recoverable × price (exact).
        prop_assert!((report.savings.energy_cost_savings_yr
            - report.savings.recoverable_kwh_yr * p.price_per_kwh).abs() < 1.0);
        // carbon = recoverable × grid factor (exact).
        prop_assert!((report.carbon.co2e_kg_yr
            - report.savings.recoverable_kwh_yr * p.grid_emission_factor_kg_per_kwh).abs() < 1.0);
        // payback is positive (or infinite when nothing is saved).
        prop_assert!(report.investment.payback_years > 0.0);
    }
}

// ───────────────────────── read-service contract ─────────────────────────

#[tokio::test]
async fn plan_service_reports_through_the_read_api() {
    let svc = StaticRecoveryPlanService::new(
        RecoveryPlannerService::default(),
        vec![site("pozsonyi14", 12, 6000.0, 0.9, LossCause::JunctionLeak)],
        graph(),
    );
    let report = svc.report().await.unwrap();
    assert!(report.savings.recoverable_kwh_yr > 0.0);
    assert!(report.investment.capex > 0.0);
}
