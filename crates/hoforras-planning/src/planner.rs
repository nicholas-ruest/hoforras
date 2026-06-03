//! `RecoveryPlannerService` (DDD-10 / ADR-0016 §3) — the deterministic, transparent estimator.
//!
//! From aggregated loss sites + the district graph it produces a [`RecoveryReport`]: where to place
//! devices, what it costs, what it saves, and how much CO₂e it avoids. Every coefficient is visible
//! in [`RecoveryParams`]; there is no hidden ML. The figures are **estimates** (ADR-0016 §4).

use std::collections::BTreeMap;

use hoforras_domain::{
    CarbonEquivalents, CarbonReduction, DeviceKind, DevicePlacement, DistrictGraph,
    InvestmentEstimate, JunctionId, RecoveryParams, RecoveryReport, SavingsProjection, SitingPlan,
    ThermalLossSite, CAR_CO2E_KG_YR, DEFAULT_RECOVERY_PARAMS, HOME_HEATING_KWH_YR, TREE_CO2E_KG_YR,
};
use hoforras_ports::recovery::RecoveryPlanner;
use hoforras_ports::PortResult;

/// Deterministic recovery planner. Generic only over its (configurable) coefficients.
pub struct RecoveryPlannerService {
    params: RecoveryParams,
}

impl Default for RecoveryPlannerService {
    fn default() -> Self {
        Self::new(DEFAULT_RECOVERY_PARAMS)
    }
}

impl RecoveryPlannerService {
    pub fn new(params: RecoveryParams) -> Self {
        Self { params }
    }

    pub fn params(&self) -> &RecoveryParams {
        &self.params
    }
}

/// Which dedicated device (if any) a site warrants.
enum Assignment {
    Sensor,
    Recovery,
    MonitorOnly,
}

impl RecoveryPlannerService {
    fn assign(&self, site: &ThermalLossSite) -> Assignment {
        if site.confidence < self.params.min_confidence_for_harvest {
            Assignment::Sensor // confirm the loss before harvesting
        } else if site.estimated_loss_kwh_yr >= self.params.recovery_unit_min_loss_kwh_yr {
            Assignment::Recovery // worth a dedicated harvester
        } else {
            Assignment::MonitorOnly // covered by the building's Appliance only
        }
    }
}

impl RecoveryPlanner for RecoveryPlannerService {
    fn plan(
        &self,
        losses: Vec<ThermalLossSite>,
        _graph: &DistrictGraph,
    ) -> PortResult<RecoveryReport> {
        let p = &self.params;
        let total_loss: f32 = losses.iter().map(|s| s.estimated_loss_kwh_yr).sum();

        let mut placements: Vec<DevicePlacement> = Vec::new();
        let mut observed_loss = 0.0f32; // addressed by a dedicated device
        let mut harvestable_loss = 0.0f32; // recovery-unit sites only
        let mut n_sensor = 0usize;
        let mut n_recovery = 0usize;

        // Appliance coverage: one per building (coherence domain), covering all its site junctions.
        let mut by_node: BTreeMap<String, (Vec<JunctionId>, String)> = BTreeMap::new();

        for site in &losses {
            by_node
                .entry(site.node_id.as_str().to_string())
                .or_insert_with(|| (Vec::new(), site.node_id.as_str().to_string()))
                .0
                .push(site.junction);

            match self.assign(site) {
                Assignment::Sensor => {
                    n_sensor += 1;
                    observed_loss += site.estimated_loss_kwh_yr;
                    placements.push(DevicePlacement {
                        kind: DeviceKind::SeedSensor,
                        junction: site.junction,
                        covers: vec![site.junction],
                        rationale: format!(
                            "low-confidence loss ({:.0} kWh/yr, conf {:.2}) — sensor confirms before harvest",
                            site.estimated_loss_kwh_yr, site.confidence
                        ),
                    });
                }
                Assignment::Recovery => {
                    n_recovery += 1;
                    observed_loss += site.estimated_loss_kwh_yr;
                    harvestable_loss += site.estimated_loss_kwh_yr;
                    placements.push(DevicePlacement {
                        kind: DeviceKind::RecoveryUnit,
                        junction: site.junction,
                        covers: vec![site.junction],
                        rationale: format!(
                            "high-loss junction ({:.0} kWh/yr) — recovery unit harvests it",
                            site.estimated_loss_kwh_yr
                        ),
                    });
                }
                Assignment::MonitorOnly => {}
            }
        }

        // One Appliance per building (deterministic order via BTreeMap).
        let n_appliance = by_node.len();
        for (junctions, node) in by_node.into_values() {
            placements.push(DevicePlacement {
                kind: DeviceKind::Appliance,
                junction: *junctions.first().unwrap_or(&JunctionId(0)),
                covers: junctions,
                rationale: format!("coherence-domain coordinator for {node}"),
            });
        }

        let coverage_pct = if total_loss > 0.0 {
            (observed_loss / total_loss).clamp(0.0, 1.0)
        } else {
            0.0
        };

        // ── savings (FR-10.4) ──
        let recoverable_kwh_yr = harvestable_loss * p.recoverable_fraction;
        let energy_cost_savings_yr = recoverable_kwh_yr * p.price_per_kwh;
        let savings = SavingsProjection {
            recoverable_kwh_yr,
            price_per_kwh: p.price_per_kwh,
            energy_cost_savings_yr,
        };

        // ── investment (FR-10.3) ──
        let n_total = n_sensor + n_recovery + n_appliance;
        let capex = n_sensor as f32 * p.seed_sensor_cost
            + n_recovery as f32 * p.recovery_unit_cost
            + n_appliance as f32 * p.appliance_cost
            + n_total as f32 * p.install_cost_per_device;
        let opex_per_year = capex * p.opex_fraction_of_capex;
        let payback_years = if energy_cost_savings_yr > 0.0 {
            capex / energy_cost_savings_yr
        } else {
            f32::INFINITY
        };
        let investment = InvestmentEstimate {
            capex,
            opex_per_year,
            payback_years,
        };

        // ── carbon (FR-10.5) ──
        let co2e_kg_yr = recoverable_kwh_yr * p.grid_emission_factor_kg_per_kwh;
        let carbon = CarbonReduction {
            co2e_kg_yr,
            grid_emission_factor_kg_per_kwh: p.grid_emission_factor_kg_per_kwh,
            equivalents: CarbonEquivalents {
                cars_off_road: co2e_kg_yr / CAR_CO2E_KG_YR,
                trees_planted: co2e_kg_yr / TREE_CO2E_KG_YR,
                homes_powered: recoverable_kwh_yr / HOME_HEATING_KWH_YR,
            },
        };

        Ok(RecoveryReport {
            plan: SitingPlan {
                placements,
                observed_loss_kwh_yr: observed_loss,
                coverage_pct,
            },
            investment,
            savings,
            carbon,
            assumptions: vec![
                format!(
                    "recoverable fraction = {:.0}% of observed loss",
                    p.recoverable_fraction * 100.0
                ),
                format!(
                    "energy price = €{:.3}/kWh (kWh-equivalent)",
                    p.price_per_kwh
                ),
                format!(
                    "grid carbon intensity = {:.3} kg CO₂e/kWh (regional estimate)",
                    p.grid_emission_factor_kg_per_kwh
                ),
                "all figures are estimates — calibrate per district before external quotation"
                    .into(),
            ],
        })
    }
}
