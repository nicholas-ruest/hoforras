// Recovery Planner read models (DDD-10 / ADR-0016) — the advisory plane the dashboard projects.
//
// These TS types mirror the published `RecoveryReport` shape from `hoforras-planning`. There is no
// live planning backend in local dev, so `demoRecovery()` supplies a calibrated District XIII
// example; production swaps in the `RecoveryPlanService` read API. Everything here is **read-only,
// estimate-grade** decision support.

export type DeviceKind = "SeedSensor" | "Appliance" | "RecoveryUnit";

export interface LossSite {
  node_id: string;
  junction: number;
  loss_kwh_yr: number;
  confidence: number;
  cause: string;
}

export interface Placement {
  kind: DeviceKind;
  junction: number;
  covers: number[];
  rationale: string;
}

export interface RecoveryReport {
  plan: {
    placements: Placement[];
    observed_loss_kwh_yr: number;
    coverage_pct: number;
  };
  investment: { capex: number; opex_per_year: number; payback_years: number };
  savings: { recoverable_kwh_yr: number; price_per_kwh: number; energy_cost_savings_yr: number };
  carbon: {
    co2e_kg_yr: number;
    grid_emission_factor_kg_per_kwh: number;
    equivalents: { cars_off_road: number; trees_planted: number; homes_powered: number };
  };
  assumptions: string[];
}

export interface RecoveryData {
  report: RecoveryReport;
  lossSites: LossSite[];
}

const N14 = "pozsonyi14.thermal.budapest.dark";
const N22 = "pozsonyi22.thermal.budapest.dark";
const N30 = "pozsonyi30.thermal.budapest.dark";

/** A calibrated District XIII recovery report (the same shape the Rust planner emits). */
export function demoRecovery(): RecoveryData {
  const lossSites: LossSite[] = [
    { node_id: N14, junction: 7, loss_kwh_yr: 45000, confidence: 0.88, cause: "JunctionLeak" },
    { node_id: N14, junction: 12, loss_kwh_yr: 18000, confidence: 0.82, cause: "PipeInsulation" },
    { node_id: N22, junction: 5, loss_kwh_yr: 32000, confidence: 0.42, cause: "UnmeteredSegment" },
    { node_id: N22, junction: 18, loss_kwh_yr: 51000, confidence: 0.91, cause: "JunctionLeak" },
    { node_id: N30, junction: 3, loss_kwh_yr: 12000, confidence: 0.35, cause: "UnmeteredSegment" },
    { node_id: N30, junction: 9, loss_kwh_yr: 1500, confidence: 0.75, cause: "GroundCoupling" },
  ];

  const report: RecoveryReport = {
    plan: {
      placements: [
        { kind: "RecoveryUnit", junction: 7, covers: [7], rationale: "high-loss junction (45,000 kWh/yr) — recovery unit harvests it" },
        { kind: "RecoveryUnit", junction: 12, covers: [12], rationale: "high-loss junction (18,000 kWh/yr) — recovery unit harvests it" },
        { kind: "RecoveryUnit", junction: 18, covers: [18], rationale: "high-loss junction (51,000 kWh/yr) — recovery unit harvests it" },
        { kind: "SeedSensor", junction: 5, covers: [5], rationale: "low-confidence loss (32,000 kWh/yr, conf 0.42) — sensor confirms before harvest" },
        { kind: "SeedSensor", junction: 3, covers: [3], rationale: "low-confidence loss (12,000 kWh/yr, conf 0.35) — sensor confirms before harvest" },
        { kind: "Appliance", junction: 7, covers: [7, 12], rationale: "coherence-domain coordinator for pozsonyi14" },
        { kind: "Appliance", junction: 5, covers: [5, 18], rationale: "coherence-domain coordinator for pozsonyi22" },
        { kind: "Appliance", junction: 3, covers: [3, 9], rationale: "coherence-domain coordinator for pozsonyi30" },
      ],
      observed_loss_kwh_yr: 158000,
      coverage_pct: 0.99,
    },
    investment: { capex: 36060, opex_per_year: 2163.6, payback_years: 4.06 },
    savings: { recoverable_kwh_yr: 74100, price_per_kwh: 0.12, energy_cost_savings_yr: 8892 },
    carbon: {
      co2e_kg_yr: 17043,
      grid_emission_factor_kg_per_kwh: 0.23,
      equivalents: { cars_off_road: 3.7, trees_planted: 811, homes_powered: 6.2 },
    },
    assumptions: [
      "recoverable fraction = 65% of observed loss",
      "energy price = €0.120/kWh (kWh-equivalent)",
      "grid carbon intensity = 0.230 kg CO₂e/kWh (regional estimate)",
      "all figures are estimates — calibrate per district before external quotation",
    ],
  };

  return { report, lossSites };
}
