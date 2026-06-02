# DDD-03 — Forecasting & Anomaly (Supporting)

**Subdomain type:** Supporting · **Crate:** `hoforras-node` (node-local, ADR-0009) ·
**SPARC unit:** `ForecastServiceImpl` (Part 2 §3) · **Key ADRs:** 0012 (edge-only WASM), 0009

Produces the predictions the market reasons over: thermal anomalies, pipe-burst precursors, and
24-hour demand — all from on-device WASM inference.

## 1. Aggregates

### `ForecastSession` (aggregate root, ephemeral)
A single spawn→infer→dissolve lifecycle of a specialist inference agent.

- **Lifecycle:** `Spawned → Inferred → Dissolved` (dissolve is guaranteed even on error, FR-2.1).
- **Rule:** the agent is **ephemeral** — it does not outlive the prediction (ADR-0012); spawn count
  must equal dissolve count.

## 2. Value Objects

`ReadingWindow` (input) · `DemandForecast` (24 h, per building) ·
`Anomaly{ kind: ThermalSpike|PressureDrop|VibrationPattern|BurstPrecursor, confidence }` ·
`BurstWarning{ horizon ∈ [6h, 48h], score }` · `AgentProfile{ type: analyst, model: lstm,
caps, cognitive }`.

## 3. Domain Services

- **`ForecastService`** — `anomalies(w)`, `burst_precursor(w)`, `demand_24h(w)`; each wraps a
  `with_agent` resource guard that spawns, infers, and **always** dissolves.

## 4. Invariants (→ Part 4 R7 / R14)

1. **Resource safety (FR-2.1):** `spawn` count == `dissolve` count across all outcomes
   (Ok / Err / panic-as-Err).
2. **Edge-only (FR-2.6):** no remote-inference dependency exists; only swarm ports are touched
   (ADR-0012). Enforced by the *absence* of a `RemoteInferenceClient` port.
3. **Burst horizon bound (FR-2.3):** any `BurstWarning.horizon ∈ [6h, 48h]`.
4. **Latency budget (NFR-1):** a single prediction < 100 ms — measured on hardware (Phase C),
   not asserted against mocks.

## 5. Domain Events

`DemandForecastProduced`, `AnomalyDetected`, `BurstPrecursorPredicted`.

## 6. Ports

| Port | Adapter | Pattern |
|------|---------|---------|
| `SwarmFactory`, `InferenceAgent` | `RuvSwarmAdapter` (ruv-swarm) | ACL over ruv-swarm |
| (model host) | `NeuroDivergentAdapter` (ruv-FANN, WASM) | ACL over ruv-FANN |

## 7. Relationships

- **← Sensing:** Customer/Supplier — consumes `ReadingWindow`.
- **→ Thermal Market:** Customer/Supplier — supplies `DemandForecast` (feeds Reason) and `Anomaly`.
- **→ Node Isolation:** an `AnomalyDetected` may trigger `CoherenceSupervisor` re-isolation.
- **→ AI Orchestration:** anomalies are surfaced to Claude for natural-language explanation.

## 8. Thin-by-design note

This context is intentionally a **thin layer over ruv-FANN/ruv-swarm**: the modelling value is in
*how* forecasts feed the core market, not in the inference engine itself. The anomaly taxonomy and
the demand/forecast value objects are the only domain artifacts modeled here.
