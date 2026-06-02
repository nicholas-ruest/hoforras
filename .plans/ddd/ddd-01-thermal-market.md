# DDD-01 — Thermal Market & Brokerage (Core Domain)

**Subdomain type:** Core · **Crate:** `hoforras-broker` · **SPARC units:** `BrokerAgent`,
`decide()`, `FederatedTrainer` (Part 2 §4) · **Key ADRs:** 0005, 0006, 0013, 0015

This is the heart of the invention: the autonomous market participant. It receives the deepest
tactical model.

## 1. Aggregates

### 1.1 `TradingNode` (aggregate root)
The autonomous participant for one building. **Single-writer / single consistency boundary**
(ADR-0006) — exactly one MRAP tick mutates it at a time.

- **Identity:** `NodeId` (a `.dark` domain).
- **State:** current `ThermalBalance`, active `Strategy`, open positions (posted `Offer`/`Bid`),
  reference to the `EconomyLedger` balance in thermal credits.
- **Behaviour (the MRAP loop):**
  - *Monitor* — ingest `ThermalBalance` (from Sensing) + `DemandForecast` (from Forecasting).
  - *Reason* — `decide()` → `Offer | Bid | Hold` (pure, see §4).
  - *Act* — post offer/bid; on counterparty acceptance, drive consensus then execute+route.
  - *Reflect* — evaluate trade outcome.
  - *Adapt* — update `Strategy`.
- **Invariants:** see §5.

### 1.2 `TradeProposal` (aggregate)
A candidate trade from a decision, awaiting governance and consensus.

- **Contains:** `ProposedTrade` (kWh, price, window, pipe route, counterparty).
- **Lifecycle:** `Proposed → (Rejected | GovernancePassed → Signed → Executed → Routed)`.
- **Rule:** transitions to `Executed` only after `GovernancePassed` **and** `ConsensusReached`.

## 2. Entities

| Entity | Identity | Notes |
|--------|----------|-------|
| `Offer` | offer id | sell intent: kWh, `Price`, `TradeWindow` |
| `Bid` | bid id | buy intent: kWh, max `Price`, `TradeWindow` |
| `Strategy` | per-node | offer/bid thresholds, reserve %, pricing params; mutated by Adapt |

## 3. Value Objects (immutable)

`ThermalBalance{ surplus_kwh, deficit_kwh, window }` · `Kwh(f32)` ·
`Price{ credit_per_kwh }` · `TradeWindow{ hours ≤ 6 }` · `Decision = Offer|Bid|Hold` ·
`Reflection` · `Adaptation` · `Gradient` (the cross-building learning payload, ADR-0005).

## 4. Domain Services

- **`PricingService`** (pure) — `price_offer`/`price_bid` from `Strategy` + `DemandForecast`.
- **`decide(balance, forecast, strategy) -> Decision`** (pure core) — the reasoning kernel; no I/O,
  example-tested by state (the one place classicist testing is used).
- **Application service `BrokerAgent`** — orchestrates the MRAP tick across ports (this is not a
  domain service but the use-case orchestrator; see ports §7).
- **`FederatedTrainer`** — computes a local `Gradient`, submits to BFT aggregation; **only**
  `Gradient` crosses the boundary (ADR-0005).

## 5. Invariants (→ property tests, Part 4 R14)

1. **Governance-before-execute (FR-3.7):** on every executed trade, `RulesEngine.check` was called
   first; a `Violation` ⇒ **zero** executions. *(ADR-0006 makes this trivial; ADR-0015 makes it
   fail-closed.)*
2. **Hard limits:** `max_daily_thermal_transfer_kwh ≤ 500`, `min_safety_reserve_percent ≥ 0.15`,
   `pipe_pressure_ceiling_bar ≤ 6.0`, `trade_window_hours ≤ 6`. Non-overridable.
3. **Consensus-before-routing:** `ConsensusGateway.finalize` precedes `execute` and `route`.
4. **Audit completeness:** an executed trade emits witness records for {TradeSigned, Exec, Routing}.
5. **Gradient-only egress:** no `ThermalFrame`/`RawReading` ever reaches `GradientAggregator`
   (compile-time, ADR-0005).
6. **Credit conservation:** `EconomyLedger` debit/credit balances per executed trade (FR-3.6).

## 6. Domain Events

`SurplusDetected`, `DeficitDetected` (in), `OfferPosted`, `BidPosted`, `TradeProposed`,
`GovernanceRejected` (witnessed), `GovernancePassed`, `ConsensusReached` (in), `TradeExecuted`,
`EnergyRouted`, `StrategyAdapted`, `GradientAggregated`.

## 7. Repositories & Ports (boundaries to other contexts)

| Port (consumed) | Upstream context | Pattern |
|-----------------|------------------|---------|
| `SeedMesh` | Sensing | Published Language (`ThermalBalance`) |
| `ForecastService` | Forecasting | Customer/Supplier |
| `RulesEngine`, `MarketGateway`, `EconomyLedger`, `TradeEvaluator`, `StrategyStore` | (own, via daa) | adapters over daa |
| `ConsensusGateway` | Trade Consensus | Anti-Corruption Layer |
| `WitnessChain` | Node Isolation | Open Host Service |
| `GradientAggregator` | peer Markets | Partnership (Gradient-only) |
| `ThermalMemory` | Thermal Memory | Customer/Supplier (downstream write) |

## 8. Why this is Core (investment note)

The pricing strategy, the decide() policy, governance enforcement, and the fail-closed safety
posture are the differentiators — they are what make an *autonomous, safe* heat market. The daa
crates supply mechanism (rules engine, economy ledger, BFT aggregation); the **policy and
decisioning modeled here are proprietary** and get the richest tests and the strictest invariants.
