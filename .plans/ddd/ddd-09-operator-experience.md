# DDD-09 — Operator Experience (Supporting)

**Subdomain type:** Supporting · **Package:** `hoforras-dashboard` (TS/React/r3f) ·
**SPARC unit:** `DistrictDashboard` (Part 2 §10)

The operator's window into the district: a live 3D thermal map, trade timeline, pipe-health
heatmap, and anomaly feed. A **downstream / conformist** context — it conforms to the event and MCP
shapes published by the Mesh and AI Orchestration contexts.

## 1. Read models (projections, not write aggregates)

The dashboard owns **projections**, not domain aggregates — it is read-only over the system's
published events. Key read models:

| Read model | Fed by | Renders |
|------------|--------|---------|
| `DistrictGraphView` | `trade:executed` stream + topology | 3D force-directed nodes/edges/particles |
| `BuildingCardView` | balance + `demand_24h` | surplus/deficit + 24 h sparklines |
| `TradeTimelineView` | active agreements + consensus status | timeline with QuDAG finality state |
| `PipeHealthView` | pipe health scores | heatmap → drill-down to witness log |
| `AnomalyFeedView` | `anomaly:detected` + `AnomalyExplained` | alerts with Claude explanations |

## 2. Value Objects (UI domain)

`UiEvent = TradeExecuted | AnomalyDetected | AnomalyExplained` · `EdgeWeight` · `EnergyFlow(route)`
· `WitnessLogEntry` (read-only view of a `WitnessRecord`).

## 3. Application behaviour

- On `trade:executed` → `updateEdgeWeight(seller, buyer, kwh)` + `animateEnergyFlow(pipe_route)`.
- On `anomaly:detected` → `highlightNode(warning)` + `requestClaudeExplanation`.

## 4. Invariants (→ Part 4 R12)

1. A `trade:executed` message updates the edge **once** and animates the route **once** (FR-9.1/9.2).
2. An `anomaly:detected` highlights the node and requests an explanation (FR-9.6).
3. Drill-down on pipe health opens the **witness log** (FR-9.5) — the audit trail is operator-visible.

## 5. Ports / integration

| Port | Direction | Substrate | Pattern |
|------|-----------|-----------|---------|
| ruflo WebSocket (`OperatorChannel`-feed) | in | `ws://appliance.district-xiii:3001` | Conformist |
| ThermalSense-Bridge MCP | in | ruflo MCP | Conformist (consumes OHS) |

## 6. Relationships

- **← District Mesh / AI Orchestration:** Conformist downstream — the dashboard adapts to whatever
  event and tool shapes those contexts publish; it does not ask them to change.
- **Read-only ethos:** the operator *observes* an autonomous system (AC-7 runs with no human input).
  The dashboard exposes the witness trail so autonomy stays accountable; an observe-only broker mode
  (Part 5 §6) is the cautious operational posture, not a per-trade approval gate.
