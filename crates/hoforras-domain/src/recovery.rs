//! Thermal Recovery Planning value objects (DDD-10 / ADR-0016) — **advisory**.
//!
//! These types describe *where heat is lost*, *where to place Cognitum devices to harvest it*, and
//! the *investment / savings / carbon* case for doing so. They are estimate-grade decision-support —
//! never control. Critically, none of these types carries a raw sensor field (temperature,
//! vibration, pressure): only aggregated loss + economics cross a boundary, preserving NFR-7 /
//! ADR-0009.

use serde::{Deserialize, Serialize};

use crate::ids::{JunctionId, NodeId};

// ───────────────────────── coefficients (configurable, documented) ─────────────────────────

/// Annual CO₂e for one passenger car (~4.6 t/yr) — for relatable carbon equivalents.
pub const CAR_CO2E_KG_YR: f32 = 4600.0;
/// Annual CO₂e a mature tree sequesters (~21 kg/yr).
pub const TREE_CO2E_KG_YR: f32 = 21.0;
/// Annual heating energy for a typical home (~12 MWh/yr).
pub const HOME_HEATING_KWH_YR: f32 = 12_000.0;

/// Tunable estimator coefficients (ADR-0016 §3). Defaults are **estimates** to be calibrated per
/// district before any figure is quoted externally.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct RecoveryParams {
    /// Fraction of *observed* loss that is actually harvestable (0..1).
    pub recoverable_fraction: f32,
    /// Energy price (EUR/kWh, kWh-equivalent per ADR-0013).
    pub price_per_kwh: f32,
    /// Regional grid/district-heat carbon intensity (kg CO₂e/kWh) — Hungary default, an estimate.
    pub grid_emission_factor_kg_per_kwh: f32,
    /// A loss site at/above this annual loss warrants a dedicated `RecoveryUnit` (kWh/yr).
    pub recovery_unit_min_loss_kwh_yr: f32,
    /// Below this confidence a site needs a `SeedSensor` to confirm the loss (0..1).
    pub min_confidence_for_harvest: f32,
    /// Unit costs (EUR).
    pub seed_sensor_cost: f32,
    pub appliance_cost: f32,
    pub recovery_unit_cost: f32,
    pub install_cost_per_device: f32,
    /// Annual opex as a fraction of capex.
    pub opex_fraction_of_capex: f32,
}

/// Documented default coefficients (ADR-0016 §3). All are estimates; calibrate per district.
pub const DEFAULT_RECOVERY_PARAMS: RecoveryParams = RecoveryParams {
    recoverable_fraction: 0.65,
    price_per_kwh: 0.12,
    grid_emission_factor_kg_per_kwh: 0.23,
    recovery_unit_min_loss_kwh_yr: 2000.0,
    min_confidence_for_harvest: 0.5,
    seed_sensor_cost: 180.0,
    appliance_cost: 2200.0,
    recovery_unit_cost: 8500.0,
    install_cost_per_device: 450.0,
    opex_fraction_of_capex: 0.06,
};

// ───────────────────────── value objects ─────────────────────────

/// Why thermal energy is being lost at a site (FR-10.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LossCause {
    /// Degraded pipe insulation along a segment.
    PipeInsulation,
    /// Leakage at a junction.
    JunctionLeak,
    /// An unmetered / unobserved segment (coverage gap).
    UnmeteredSegment,
    /// Heat bleeding into the surrounding ground.
    GroundCoupling,
}

/// A localized thermal-loss site — the **aggregate** a node emits (no raw fields; ADR-0009).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ThermalLossSite {
    pub node_id: NodeId,
    pub junction: JunctionId,
    pub estimated_loss_kwh_yr: f32,
    /// 0..1 — how sure we are this loss is real (driven by sensor coverage/quality).
    pub confidence: f32,
    pub cause: LossCause,
}

/// A kind of Cognitum device proposed for placement (FR-10.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceKind {
    /// SEED thermal sensor — observes a coverage gap / low-confidence site.
    SeedSensor,
    /// Cognitum Appliance — coordinates a building's coherence domain.
    Appliance,
    /// Heat-recovery unit — harvests a high-loss junction.
    RecoveryUnit,
}

/// A proposed device placement covering one or more loss sites (FR-10.2).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DevicePlacement {
    pub kind: DeviceKind,
    pub junction: JunctionId,
    /// Junctions whose loss this device observes/harvests.
    pub covers: Vec<JunctionId>,
    pub rationale: String,
}

/// The siting plan: which devices to place where (FR-10.2).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SitingPlan {
    pub placements: Vec<DevicePlacement>,
    /// Total annual loss addressed by a dedicated device (sensor/recovery unit).
    pub observed_loss_kwh_yr: f32,
    /// Fraction of district loss covered by a dedicated device (0..1).
    pub coverage_pct: f32,
}

/// Infrastructural investment estimate (FR-10.3, EUR).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct InvestmentEstimate {
    pub capex: f32,
    pub opex_per_year: f32,
    pub payback_years: f32,
}

/// Energy-savings projection (FR-10.4).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SavingsProjection {
    pub recoverable_kwh_yr: f32,
    pub price_per_kwh: f32,
    pub energy_cost_savings_yr: f32,
}

/// Relatable carbon-reduction equivalents (FR-10.5).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct CarbonEquivalents {
    pub cars_off_road: f32,
    pub trees_planted: f32,
    pub homes_powered: f32,
}

/// Carbon-reduction estimate (FR-10.5).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct CarbonReduction {
    pub co2e_kg_yr: f32,
    pub grid_emission_factor_kg_per_kwh: f32,
    pub equivalents: CarbonEquivalents,
}

/// The full advisory report — the artifact operators act on (ADR-0016).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RecoveryReport {
    pub plan: SitingPlan,
    pub investment: InvestmentEstimate,
    pub savings: SavingsProjection,
    pub carbon: CarbonReduction,
    /// Human-readable assumptions behind the figures (estimate-grade reporting, ADR-0016 §4).
    pub assumptions: Vec<String>,
}
