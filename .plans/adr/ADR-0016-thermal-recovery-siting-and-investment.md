# ADR-0016 — Thermal-Recovery Siting & Investment Analysis (an advisory planning context)

- **Status:** Proposed
- **Date:** 2026-06-03
- **Traces:** new FR-10.1–10.5 (loss localization · device siting · investment · savings · carbon);
  aligns with `sparc.md` Part 1 NFR-7 (no raw egress); extends Part 2 §10 (Operator Experience)
- **Constrains DDD:** **new** Thermal Recovery Planning context (DDD-10); Operator Experience (DDD-09)
- **Related:** ADR-0002 (hexagonal), ADR-0009 (raw stays node-local), ADR-0007 (read-only conformist
  dashboard), ADR-0005/0008 (no-raw-egress), ADR-0013 (kWh-equivalent economics)

## Context

The operational system (P0–P10) runs the *autonomous* thermal market: it trades surplus heat in
real time. But before a district commits capital, operators need to answer a different, **advisory**
question:

> *Where is thermal power actually being lost, where should we place Cognitum devices (SEED sensors,
> Appliances, recovery units) to observe and harvest it, what will that infrastructure cost, how much
> energy will it save, and how much CO₂e will it avoid?*

This is **planning / decision-support**, not autonomous control. It has a different risk profile
(human-in-the-loop, no actuation), a different data cadence (annualized estimates, not per-tick), and
a different audience (capital planners, sustainability officers, regulators). Folding it into the
safety-critical broker would mix estimate-grade figures into the AC-7 path and break the "exactly
five operational MCP tools" invariant (FR-8.1 / P8). It also must not become an excuse to move **raw**
sensor data across a building boundary (NFR-7 / ADR-0009).

We need a clean home for it — in the platform's functionality *and* in the dashboard.

## Decision

Introduce a new **Thermal Recovery Planning** bounded context (DDD-10) — a **Supporting, advisory,
read-only** subdomain that maps heat loss, proposes device placements, and quantifies investment,
savings, and carbon reduction. It **never** triggers actuation or trades; it produces a `SitingPlan`
artifact that humans act on.

### 1. Placement in the hexagon (platform functionality)

- **Domain (`hoforras-domain::recovery`)** — pure value objects (below), no I/O.
- **Ports (`hoforras-ports`)** — a `LossAnalyzer`, a `RecoveryPlanner`, and a `RecoveryPlanService`
  (the read API the dashboard consumes), all over domain types only (ADR-0002/0003).
- **New crate `hoforras-planning`** — the advisory services + **deterministic estimator adapters**.
  It depends on `hoforras-domain`/`hoforras-ports` only and is **off** the broker/consensus path.
- **Node-local loss aggregation stays in `hoforras-node`** (ADR-0009): each Appliance emits a
  `ThermalLossEstimate` **aggregate** (per pipe-segment/zone loss, confidence) — raw `ThermalFrame`s
  never leave the building (NFR-7). The planner consumes these aggregates **plus** the public
  `DistrictGraph` (pipe topology + junction coordinates); it never sees a raw reading.

### 2. Domain model (value objects, FR-10.x)

```
ThermalLossSite   { location: JunctionId/Coord, segment, estimated_loss_kwh_yr, confidence, cause }
DeviceKind        = SeedSensor | Appliance | RecoveryUnit
DevicePlacement   { kind, at: JunctionId/Coord, covers: [ThermalLossSite], rationale }
SitingPlan        { placements: [DevicePlacement], observed_loss_kwh_yr, coverage_pct }
InvestmentEstimate{ capex, opex_per_year, payback_years }
SavingsProjection { recoverable_kwh_yr, price_per_kwh, energy_cost_savings_yr }
CarbonReduction   { co2e_kg_yr, grid_emission_factor_kg_per_kwh, equivalents }
RecoveryReport    { plan, investment, savings, carbon, assumptions, generated_at }
```

### 3. Deterministic, documented estimators (no hidden ML)

Estimation is **transparent and swappable** (ADR-0001) — operators (and regulators) can see every
coefficient. Models live behind the ports; defaults are **configurable constants** with cited
provenance:

- **Loss localization** — derive `ThermalLossSite`s from measured temperature/gradient drop along a
  pipe segment vs. its expected profile, weighted by sensor coverage; gaps in coverage become
  low-confidence candidate sites (flagged for a sensor placement).
- **Siting** — a greedy max-coverage optimizer places the **fewest devices** that observe/harvest the
  **most** annual loss (SEED sensors where observability is missing; RecoveryUnits at high-loss
  junctions; Appliances per coherence domain).
- **Investment** — `capex = Σ device_unit_cost + install`; `opex_per_year` per device; `payback_years
  = capex / max(energy_cost_savings_yr, ε)`.
- **Savings** — `recoverable_kwh_yr = observed_loss_kwh_yr × recoverable_fraction`;
  `energy_cost_savings_yr = recoverable_kwh_yr × price_per_kwh` (kWh-equivalent, ADR-0013).
- **Carbon** — `co2e_kg_yr = recoverable_kwh_yr × grid_emission_factor`; the **grid emission factor**
  defaults to a configurable regional value (Hungary district-heat/grid intensity) and is **labeled
  as an estimate**, with relatable `equivalents` (cars off the road, trees, homes powered).

### 4. Honest, estimate-grade reporting (project ethos)

Every figure carries explicit **assumptions** and a **confidence**; outputs are decision-support, not
guarantees. Units are fixed: **kWh/year**, **EUR**, **kg CO₂e/year**. The planner is **human-in-the-
loop** — the deliberate opposite of the autonomous market (it issues no commands, signs no trades).

### 5. Advisory plane kept off the operational OHS

To preserve FR-8.1's *exactly-five operational MCP tools* (P8/AC-7), planning is exposed on a
**separate advisory path**: the dashboard reads `RecoveryReport` via the `RecoveryPlanService` port
directly; if surfaced over MCP it is a distinct `hoforras.recovery.*` tool group, **never** added to
the five live operational tools.

### 6. UI structure (Operator Experience, DDD-09 extension)

Add a **"Recovery Planner" mode** to the dashboard — a HUD toggle **Live ⇄ Planner**. The Planner is
**read-only / conformist** (ADR-0007): it projects the planning service's `RecoveryReport`; it issues
no commands, and an **"advisory · estimates only"** badge is always visible.

- **3D map — heat-loss overlay:** loss hotspots render as pulsing volumetric markers along pipe
  segments/junctions, **sized by kWh/yr lost** and colored by severity.
- **Proposed device placements:** distinct glyphs for `SeedSensor` / `Appliance` / `RecoveryUnit`
  with translucent **coverage halos**; clicking a placement shows the loss it captures and its cost.
- **New panels (read-only):**
  - **Siting Plan** — ranked placements + coverage %.
  - **Investment** — CapEx / OpEx / payback period.
  - **Savings** — recoverable kWh/yr and € saved.
  - **Carbon** — tonnes CO₂e/yr avoided + relatable equivalents.

The existing Live mode (3D map, trades, anomalies, witness log) is unchanged; Planner mode is an
additive overlay/view that reuses the same conformist event/read pipeline.

## Consequences

**Positive**
- Operators can target capital where loss is **highest** — a quantified ROI **and** carbon case from
  the same sensing data the platform already produces.
- Stays inside the privacy envelope: loss is aggregated **node-local**; only estimates cross a
  boundary (NFR-7 / ADR-0009).
- Transparent, auditable models (no black box) suit regulators and sustainability reporting.
- Clean separation keeps the safety-critical autonomous path (AC-7) and its five-tool OHS untouched.

**Negative / costs**
- Estimate accuracy depends on **sensor coverage** and **coefficient calibration**; figures must be
  labeled as estimates with stated assumptions — not guarantees.
- Cost and **carbon factors are regional** and require calibration per district before they are
  quoted externally.
- Adds a bounded context, a crate, and a dashboard mode (more surface to maintain).

## Alternatives considered

- **Fold planning into the broker/market** — rejected: mixes estimate-grade advisory figures into the
  safety-critical autonomous path and breaks the five-tool operational invariant (FR-8.1).
- **ML/black-box loss & siting model** — rejected for the pilot: not explainable; operators and
  regulators need transparent, auditable assumptions. Deterministic models slot behind the ports and
  can be upgraded later (ADR-0001).
- **Compute loss centrally from raw frames** — rejected: would move raw data across a boundary,
  violating NFR-7 / ADR-0009. Loss is aggregated where the raw data lives.
- **Real-time actuation to harvest the loss** — out of scope: physical valve/pump actuation
  certification is a post-pilot concern (spec §3.2). The planner is decision-support only.

## Implementation note (follows this ADR)

Built S→P→A→R→C like every other context, London-TDD: domain value objects + property tests
(coverage %, payback, conservation of kWh→€→CO₂e), a deterministic estimator with example-tested
coefficients, a `RecoveryPlanService` contract test, and a Planner dashboard mode with vitest
component tests (overlay renders placements; panels show CapEx/savings/carbon; advisory badge always
present; mode is read-only). Proposed FR-10.1–10.5 and an AC-9 ("a calibrated recovery report for
District XIII") trace this context to closure.
