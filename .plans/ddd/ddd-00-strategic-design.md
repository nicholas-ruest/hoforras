# DDD-00 — Strategic Design

**Project:** Hőforrás Budapest — Peer-to-Peer Thermal Energy Intelligence System
**Derived from:** `.plans/sparc.md`; constrained by `.plans/adr/`
**Scope:** the whole problem space — subdomain classification, bounded contexts, context map,
and the shared ubiquitous language.

---

## 1. The domain in one paragraph

Buildings in Budapest's District XIII hold geothermal and waste-heat resources that fluctuate
between surplus and deficit hour to hour. Hőforrás lets each building **autonomously sense, forecast,
and trade heat** with its neighbours — safely (hard governance limits), privately (no raw data
leaves a building), verifiably (every privileged action witnessed), and resiliently (a self-healing
district mesh) — all on edge hardware. The competitive heart of the invention is the **autonomous
thermal market**; almost everything else is reused from Ruv repositories.

## 2. Subdomain classification

Per Eric Evans / Vaughn Vernon, we separate where to invest custom modelling (core) from what to
build plainly (supporting) from what to buy/reuse (generic).

| Subdomain | Type | Why | Primary Ruv reuse |
|-----------|------|-----|-------------------|
| **Thermal Market & Brokerage** | **Core** | The autonomous MRAP trader + pricing + governance *is* the invention (research §67 "the heart"). This is where bugs are costly and differentiation lives. | daa (rules/economy/ai/prime) — but the *policy & decisioning* are ours |
| Sensing & Ingestion | Supporting | Necessary, domain-specific (ThermalFrame, witnessing), but not the differentiator | rvcsi / RuView |
| Forecasting & Anomaly | Supporting | Domain-specific model usage; thin layer over ruv-FANN | ruv-FANN / ruv-swarm |
| Node Isolation & Security | Supporting | Domain *policy* (no-raw-egress, capability scopes) over a generic substrate | rvm (generic substrate) |
| District Mesh Coordination | Supporting | Domain-shaped (thermal load balancing, cascade prevention) over a generic fabric | Synaptic-Mesh |
| Trade Consensus | **Generic** | Post-quantum DAG consensus is a solved, off-the-shelf capability | QuDAG |
| Thermal Memory | **Generic** | Vector/GNN store is off-the-shelf; we only choose the embedding | RuVector |
| AI Orchestration | **Generic** | MCP/agent plumbing is infrastructure | ruflo / rvagent |
| Operator Experience | Supporting | Domain-specific visualization, but not the moat | React / r3f |

**Investment implication:** model the Thermal Market richly (it gets the deepest tactical design,
DDD-01); wrap the generic subdomains behind anti-corruption layers (ports/adapters, ADR-0002) and
keep their models thin.

## 3. Bounded contexts

One context per subdomain. Each context owns its model and ubiquitous language; terms do **not**
silently cross boundaries (the same word may mean different things, or be translated by an ACL).

```
┌──────────────────────────────────────────────────────────────────────────┐
│                          APPLIANCE (per building)                          │
│  ┌────────────────┐   balance    ┌───────────────────────────┐            │
│  │ Sensing &      │─────────────▶│  THERMAL MARKET (CORE)     │            │
│  │ Ingestion      │  ThermalFrame│  MRAP · pricing · rules    │            │
│  └──────┬─────────┘              └───┬───────────┬───────────┘            │
│         │ window                     │ agreement │ state                   │
│         ▼                            ▼           ▼                         │
│  ┌────────────────┐          ┌──────────────┐ ┌─────────────────┐         │
│  │ Forecasting &  │          │ Trade        │ │ Thermal Memory  │         │
│  │ Anomaly        │          │ Consensus    │ │ (vectors/GNN)   │         │
│  └────────────────┘          └──────┬───────┘ └─────────────────┘         │
│                                     │ signed DAG entries                   │
│  ┌───────────────────────────────────────────────────────────────────┐   │
│  │ NODE ISOLATION & SECURITY  (coherence domain = this whole box)      │   │
│  │   capability gate · witness chain · auto re-isolation               │   │
│  └───────────────────────────────────────────────────────────────────┘   │
└───────────────┬─────────────────────────────────────────┬────────────────┘
                │ DAG fabric                                │ MCP / WebSocket
                ▼                                           ▼
        ┌────────────────┐                          ┌────────────────┐
        │ DISTRICT MESH  │◀───── other Appliances    │ AI ORCH. +     │
        │ COORDINATION   │                           │ OPERATOR EXP.  │
        └────────────────┘                           └────────────────┘
```

## 4. Context map (relationships & patterns)

DDD integration patterns, realized as hexagonal ports/adapters (ADR-0002).

| Upstream → Downstream | Pattern | Realized as |
|-----------------------|---------|-------------|
| Sensing → Forecasting | Customer/Supplier | `ReadingWindow` published; forecasting consumes |
| Sensing → Thermal Market | Published Language | `ThermalBalance` via `SeedMesh` port (aggregate, not raw) |
| Forecasting → Thermal Market | Customer/Supplier | `DemandForecast`/`Anomaly` via `ForecastService` port |
| Thermal Market → Trade Consensus | **Anti-Corruption Layer** | `ConsensusGateway` wraps QuDAG (`MlDsaSigner`/`DagNetwork`) |
| Thermal Market → Thermal Memory | Customer/Supplier (downstream) | `ThermalMemory` port; market writes state, queries matches |
| Node Isolation & Security → *all* | **Open Host Service / Shared Kernel** | capability tokens + witness chain are offered to every context; `hoforras-domain` holds the shared `Capability`/`WitnessRecord` kernel |
| Thermal Market ↔ District Mesh | Published Language | signed DAG entries are the lingua franca (ADR-0007) |
| District Mesh → other Appliances | Partnership (peer mesh) | Synaptic-Mesh fabric, `.dark` discovery |
| AI Orchestration → *all* | Open Host Service | ThermalSense-Bridge MCP exposes a published tool API to Claude |
| Mesh/AI → Operator Experience | Conformist (downstream) | dashboard conforms to the event/MCP shapes it receives |
| Thermal Market → (peer) Thermal Market | Partnership via gradients | federated training, **Gradient-only** ACL (ADR-0005) |

**Key boundary decisions:**
- The **coherence domain is the privacy boundary** — Node Isolation wraps the whole Appliance; only
  *aggregated availability* (capability-gated) and *signed DAG entries* cross it (ADR-0004/0008).
- Federated learning crosses building boundaries with **gradients only**, enforced at the type level
  (ADR-0005).

## 5. Ubiquitous language (canonical glossary)

These terms are shared via the `hoforras-domain` crate (ADR-0003). Context-local nuances are noted.

| Term | Meaning | Owning context |
|------|---------|----------------|
| **Appliance** | Edge unit per building running the full node stack | (deployment) |
| **SEED node** | Cognitum ESP32 sensor emitting signed readings | Sensing |
| **ThermalFrame** | Validated, quality-scored, witnessed reading | Sensing |
| **ThermalBalance** | Aggregated surplus/deficit (kWh) — the *only* thermal quantity that crosses to the market | Sensing → Market |
| **MRAP** | Monitor→Reason→Act→Reflect→Adapt control loop | Thermal Market |
| **Offer / Bid** | Intent to sell / buy heat at a price for a window | Thermal Market |
| **Thermal credit** | kWh-equivalent unit of account (ADR-0013) | Thermal Market |
| **ProposedTrade** | A candidate trade awaiting governance + consensus | Thermal Market |
| **ThermalTradeAgreement** | A signed, consensus-final trade commitment | Trade Consensus |
| **Governance rule** | A hard, non-overridable safety limit (ADR-0015) | Thermal Market |
| **Coherence domain** | RVM-isolated partition = one building = privacy boundary | Node Isolation |
| **Capability** | Unforgeable, scoped, expiring access token | Node Isolation |
| **CommEdge** | Capability-gated cross-partition channel | Node Isolation |
| **Witness record** | 64-byte hash-chained audit entry of a privileged action | Node Isolation |
| **Gradient** | Model update; the *only* cross-building learning payload | Thermal Market / Forecasting |
| **DAG entry** | Signed, tamper-evident unit of mesh propagation | Mesh / Consensus |
| **.dark domain** | Directory-less peer identity (e.g. `pozsonyi14.thermal.budapest.dark`) | Consensus / Mesh |
| **ThermalSense-Bridge** | MCP server exposing thermal tools to Claude | AI Orchestration |

## 6. Domain events (cross-context, big picture)

The event flow that realizes AC-7 (autonomous trade), spanning contexts:

```
SurplusDetected / DeficitDetected      (Sensing → Market)
  → OfferPosted / BidPosted            (Market)
  → TradeProposed                      (Market)
  → [GovernanceRejected | GovernancePassed]   (Market; rejection witnessed)
  → AgreementSigned                    (Consensus)
  → ConsensusReached                   (Consensus → Market)
  → TradeExecuted → EnergyRouted       (Market)
  → ActionWitnessed ×3                 (Node Isolation: signed, exec, routing)
  → StrategyAdapted                    (Market)
  → TradeExecuted published as DAG     (Mesh → Operator Experience)
```

Anomaly flow (cross-context):
```
AnomalyDetected (Forecasting) → NodeIsolated (Node Isolation, if warranted)
                              → AnomalyExplained (AI Orchestration → Operator)
```

## 7. Aggregate boundaries & consistency

- **The TradingNode is the primary consistency boundary** and single-writer (ADR-0006): one MRAP
  tick mutates it at a time.
- **Cross-aggregate / cross-context consistency is eventual**, carried by DAG entries (ADR-0007).
- **The CoherenceDomain aggregate** governs the privacy/isolation boundary and is mutated only by the
  `CoherenceSupervisor` on anomaly signals.

See each tactical document (DDD-01…09) for the aggregates, invariants, and ports per context.
