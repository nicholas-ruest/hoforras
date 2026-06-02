# DDD-08 — AI Orchestration (Generic)

**Subdomain type:** Generic (ruflo / rvagent plumbing) · **Crate:** `hoforras-mesh` ·
**SPARC units:** `ThermalBridgeServer`, `AnomalyExplainer` (Part 2 §9)

A **generic** subdomain: agent/MCP orchestration is infrastructure. It acts as an **Open Host
Service** — the ThermalSense-Bridge publishes a stable tool API that Claude (and the dashboard)
consume.

## 1. Aggregates

### `ThermalBridge` (aggregate root)
The MCP server exposing thermal capabilities as callable tools.

- **Registered tools (the Published Language / OHS API):**
  `hoforras.thermal.district_status`, `hoforras.trade.active_agreements`,
  `hoforras.pipe.health_scores`, `hoforras.anomaly.recent`, `hoforras.forecast.demand_24h`.
- **Rule:** each tool delegates to exactly one backing service; an unknown tool returns a typed
  error (no delegation).

## 2. Value Objects

`ToolName` · `ToolArgs` (validated) · `Explanation` (natural-language anomaly narrative) ·
`AnomalyContext`.

## 3. Domain Services

- **`ThermalBridgeServer`** — `boot()` registers the five tools; `invoke(name, args)` routes to the
  backing service or errors.
- **`AnomalyExplainer`** — `explain_and_surface(ctx)`: ask Claude, then push to the operator.

## 4. Invariants (→ Part 4 R11)

1. **Exactly-five tools (FR-8.1):** boot registers precisely the five named tools.
2. **One tool → one service (FR-8.1):** each tool delegates to its single backing service; unknown
   ⇒ `Err(UnknownTool)` with no delegation.
3. **Explain-then-surface (FR-8.2):** `ClaudeReasoner.explain` precedes `OperatorChannel.push`.

## 5. Domain Events

`ToolInvoked`, `AnomalyExplained`, `FederationSignalExchanged` (deferred — see relationships).

## 6. Ports

| Port | Adapter | Substrate |
|------|---------|-----------|
| `McpToolRegistry`, `ClaudeReasoner` | `RufloMcpAdapter` | ruflo / @ruvnet/rvagent |
| `OperatorChannel` | `WebSocketSink` | tokio-tungstenite |

## 7. Relationships

- **Open Host Service → Claude & Operator Experience:** the MCP tool API is the published contract.
- **Federation (FR-8.3, deferred):** cross-district expansion authenticates via mTLS + Ed25519 and
  exchanges **only market signals** — never raw data (same Gradient-only spirit as ADR-0005). The
  pilot ships a stub; full acceptance is a post-pilot SPARC pass (spec risk R-5).
- **Generic-by-design:** no proprietary AI; Claude reasons over the tools the core domain exposes.
