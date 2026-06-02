# SPARC Architecture — Hőforrás Budapest

**Project:** Hőforrás — Peer-to-Peer Thermal Energy Intelligence System
**Methodology:** SPARC — Reuven Cohen
**Test discipline:** London School TDD (outside-in, mock-driven)
**Document phase:** **A — Architecture** (Phase 3 of 5)
**Consumes:** `.plans/sparc-specification.md` (FRs, NFRs, ports), `.plans/sparc-pseudocode.md` (units, algorithms)
**Produces (for Phase R/C):** crate/module topology, port→adapter bindings, deployment & data-flow, interface signatures, threading/runtime model.
**Status:** Draft v1.0

---

## 0. Architectural goals (traceable to NFRs)

| Goal | Driven by | Architectural response |
|------|-----------|------------------------|
| Edge-only, no GPU/cloud | NFR-5 | WASM inference in-process; zero remote-inference dependency in the dependency graph |
| Testability / mockability | NFR-14, §7 TDD | **Hexagonal (ports & adapters)**: domain crates depend on traits, adapters wrap Ruv crates |
| No raw-data egress | NFR-7 | Partition boundary = process/RVM boundary; only aggregates + gradients cross, by type |
| Real-time isolation | NFR-3 | RVM coherence runs in-process in `hoforras-node`; no network hop for isolation |
| Sub-second finality | NFR-2 | QuDAG consensus client co-located on the Appliance; async, non-blocking broker |
| Self-healing under churn | NFR-9/10 | Synaptic-Mesh DAG transport, not RPC; eventual consistency via replay |
| ≤ 500-line modules | NFR-12 | One unit per module; adapters thin; domain logic pure |

**Chosen style: Hexagonal / Ports-and-Adapters, per crate.** Domain logic (the units from Pseudocode) sits in the centre and depends only on port *traits*. Ruv crates are reached exclusively through *adapters* that implement those traits. This is what makes the London-School mocking in Phase R possible without touching third-party internals (spec §7.1.4).

---

## 1. System context (C4 level 1)

```
                          ┌─────────────────────────────────────────┐
   District Operator ───▶ │  hoforras-dashboard (TS/React/r3f)       │
        (browser)         └───────────────┬─────────────────────────┘
                                          │ WebSocket + MCP (ruflo)
                                          ▼
        Claude ◀──── MCP (ThermalSense-Bridge) ────┐
                                                    │
   ┌────────────────────────────────────────────── Appliance (per building) ─────────┐
   │  hoforras-node (RVM coherence domain = isolation/security boundary)              │
   │   ├── hoforras-sensor   (ingestion)        ├── hoforras-broker (MRAP + economy)  │
   │   ├── hoforras-mesh      (swarm/consensus)  └── memory + inference adapters       │
   └──────────┬───────────────────────────────────────────────┬──────────────────────┘
              │ .thermal.budapest.dark (QuDAG/Kademlia)         │ SEED nodes (ESP32)
              ▼                                                 ▼
      Peer Appliances (District XIII mesh)              Physical thermal sensors
              │
              ▼
      Federation peers (District V / VII)  — signals only, mTLS+Ed25519 (deferred)
```

The **Appliance** is the deployment unit; one per building. The **RVM coherence domain** is the security/isolation boundary — everything raw stays inside it (NFR-7).

---

## 2. Cargo workspace topology (C4 level 2 — containers)

From research §195–210, the workspace has four crates. Architecture adds an internal layering so domain stays free of Ruv-crate types.

```
hoforras/                          (workspace root)
├── Cargo.toml                     [workspace] members = sensor, broker, node, mesh, + ports, adapters
├── crates/
│   ├── hoforras-ports/            ← NEW: pure trait definitions (no Ruv deps)  [the hexagon edges]
│   │     src/{sensor,inference,broker,security,consensus,memory,mesh,ai}.rs
│   │     // PORT traits from pseudocode §1; depends only on hoforras-domain types
│   ├── hoforras-domain/           ← NEW: pure value types (ThermalFrame, TradeAgreement, Capability…)
│   │     // no I/O, no Ruv deps — the ubiquitous language (spec §10)
│   │
│   ├── hoforras-sensor/           Layer 1 — IngestionPipeline (domain) + rvcsi/RuView adapters
│   ├── hoforras-broker/           Layer 3 — BrokerAgent (MRAP), FederatedTrainer + daa adapters
│   ├── hoforras-node/             Layer 4 — CoherenceSupervisor, CapabilityBroker, AuditTrail + rvm adapters
│   │                              Layer 2 — ForecastService + ruv-FANN/ruv-swarm adapters (inference is node-local)
│   │                              Layer 6 — ThermalMemory + RuVector adapter
│   └── hoforras-mesh/             Layer 5 — ConsensusGateway + QuDAG adapter
│                                  Layer 7 — MeshCoordinator + Synaptic-Mesh adapter
│                                  Layer 8 — ThermalBridgeServer (MCP), AnomalyExplainer + ruflo/rvagent adapters
└── vendor/rvm/                    path-dep crates: rvm-kernel, rvm-coherence, rvm-witness, rvm-security, rvm-cap, rvm-proof
```

**Dependency rule (enforced, acyclic):**
`domain ← ports ← {sensor, broker, node, mesh}`. Adapters depend on Ruv crates; domain/ports never do. A domain crate that `use`d a `daa_*` or `rvm_*` type is a layering violation (lint/CI gate).

```
                 ┌────────────┐
                 │  domain    │  (pure types)
                 └─────▲──────┘
                       │
                 ┌─────┴──────┐
                 │   ports    │  (traits over domain types)
                 └─────▲──────┘
        ┌───────┬──────┼───────┬────────┐
   ┌────┴───┐ ┌─┴────┐ ┌┴─────┐ ┌┴──────┐
   │ sensor │ │broker│ │ node │ │ mesh  │   ← domain logic + adapters (impl ports using Ruv crates)
   └────────┘ └──────┘ └──────┘ └───────┘
```

---

## 3. Port → Adapter binding (the hexagon edges)

Each pseudocode PORT gets exactly one production adapter (a thin wrapper over a Ruv crate) and, in Phase R, mock implementations. This table is the contract-test (spec §7.2 "Contract" level) work-list.

| Port (pseudocode §1) | Production adapter | Ruv crate / package | Crate home |
|----------------------|--------------------|---------------------|------------|
| `SensorSource`, `Validator`, `QualityScorer` | `RvcsiIngestAdapter` | `@ruv/rvcsi` / `rvcsi-core` (`ValidationStatus`, `QualityScore`) | hoforras-sensor |
| `WitnessSigner` (Ed25519) | `Ed25519WitnessAdapter` | `ed25519-dalek 2` + RuView witness chain | hoforras-sensor |
| `EventEmitter`, `RejectionLog` | `TypedEventBus` | in-house (tokio mpsc) | hoforras-sensor |
| `SwarmFactory`, `InferenceAgent` | `RuvSwarmAdapter` | `ruv-swarm` (1.0.5 cargo) | hoforras-node |
| `ForecastService` models | `NeuroDivergentAdapter` | `ruv-FANN` (LSTM/N-BEATS, WASM) | hoforras-node |
| `SeedMesh` | `SeedMeshAdapter` | reads hoforras-sensor event bus | hoforras-broker |
| `MarketGateway`, `EconomyLedger` | `DaaEconomyAdapter` | `daa-economy 0.2.1` | hoforras-broker |
| `RulesEngine` | `DaaRulesAdapter` | `daa-rules 0.2.1` | hoforras-broker |
| `TradeEvaluator`, `StrategyStore` | `DaaOrchestratorAdapter` | `daa-orchestrator`, `daa-ai 0.2.1` | hoforras-broker |
| `GradientAggregator` (BFT) | `PrimeCoordinatorAdapter` | `daa-prime-coordinator/-trainer/-core/-dht 0.2.1` | hoforras-broker |
| `MincutEngine`, `PartitionController` | `RvmCoherenceAdapter` | `rvm-coherence`, `rvm-kernel` | hoforras-node |
| `WitnessChain` | `RvmWitnessAdapter` | `rvm-witness`, `rvm-proof` | hoforras-node |
| `CapabilityGate` | `RvmCapAdapter` | `rvm-security`, `rvm-cap` | hoforras-node |
| `MlDsaSigner`, `DagNetwork`, `PeerDiscovery` | `QuDagAdapter` | `QuDAG` (ML-DSA, ML-KEM-1024, Kademlia/.dark) | hoforras-mesh |
| `VectorIndex`, `GnnEngine` | `RuVectorAdapter` | `RuVector` / `ruvector` (HNSW, GNN) | hoforras-node |
| `MeshTransport`, `NodeRegistry` | `SynapticMeshAdapter` | `Synaptic-Mesh` (DAG fabric) | hoforras-mesh |
| `McpToolRegistry`, `ClaudeReasoner` | `RufloMcpAdapter` | `ruflo`, `@ruvnet/rvagent` | hoforras-mesh |
| `OperatorChannel` | `WebSocketSink` | tokio-tungstenite → dashboard | hoforras-mesh |

**Why this matters for TDD:** in Phase R, the domain unit (e.g. `BrokerAgent`) is constructed with mock `RulesEngine`/`MarketGateway`/… The adapters above are exercised separately by *contract tests* against the real Ruv crate. The two never mix — interaction tests stay fast and deterministic (NFR-14).

---

## 4. Crate-by-crate module layout (C4 level 3 — components)

### 4.1 `hoforras-domain` (pure, no deps)
```
domain/src/
  thermal_frame.rs      ThermalFrame, QualityScore, ValidationStatus (re-exported newtypes)
  trade.rs              ThermalTradeAgreement, ProposedTrade, ExecutedTrade, Finality
  capability.rs         Capability{rights,scope,expiry}, Scope::{Raw, AggregatedThermalAvailability}
  witness.rs            WitnessRecord (64-byte), PrivilegedAction
  governance.rs         GovernanceRules consts (500.0 / 0.15 / 6.0 / 6)
  ids.rs                NodeId(.dark), JunctionId, Timestamp
  gradient.rs           Gradient  ← aggregator accepts ONLY this (NFR-7 type wall)
```

### 4.2 `hoforras-sensor` (Layer 1)
```
sensor/src/
  pipeline.rs           UNIT IngestionPipeline   (≤500 ln)
  bus.rs                TypedEventBus (EventEmitter + RejectionLog)
  adapters/
    rvcsi.rs            RvcsiIngestAdapter : SensorSource + Validator + QualityScorer
    witness_ed25519.rs  Ed25519WitnessAdapter : WitnessSigner
```

### 4.3 `hoforras-broker` (Layer 3) — the MRAP hub
```
broker/src/
  agent.rs              UNIT BrokerAgent (orchestrates MRAP; depends on ports only)
  decide.rs             pure decide()/price_* (no I/O — example-tested)
  federated.rs          UNIT FederatedTrainer
  adapters/
    daa_rules.rs        DaaRulesAdapter : RulesEngine
    daa_economy.rs      DaaEconomyAdapter : MarketGateway + EconomyLedger
    daa_orch.rs         DaaOrchestratorAdapter : TradeEvaluator + StrategyStore
    prime.rs            PrimeCoordinatorAdapter : GradientAggregator
    seed_mesh.rs        SeedMeshAdapter : SeedMesh  (subscribes to sensor bus)
```

### 4.4 `hoforras-node` (Layers 4, 2, 6) — the isolation boundary + node-local services
```
node/src/
  coherence.rs          UNIT CoherenceSupervisor
  capability.rs         UNIT CapabilityBroker   (the no-raw-egress gate — §5.2)
  audit.rs              UNIT AuditTrail
  forecast.rs           UNIT ForecastServiceImpl (+ with_agent resource guard)
  memory.rs             UNIT ThermalMemory
  adapters/
    rvm_coherence.rs    RvmCoherenceAdapter : MincutEngine + PartitionController
    rvm_witness.rs      RvmWitnessAdapter   : WitnessChain
    rvm_cap.rs          RvmCapAdapter       : CapabilityGate
    ruv_swarm.rs        RuvSwarmAdapter     : SwarmFactory + InferenceAgent
    neuro.rs            NeuroDivergentAdapter (WASM model host)
    ruvector.rs         RuVectorAdapter     : VectorIndex + GnnEngine
```

### 4.5 `hoforras-mesh` (Layers 5, 7, 8)
```
mesh/src/
  consensus.rs          UNIT ConsensusGateway
  coordinator.rs        UNIT MeshCoordinator
  bridge.rs             UNIT ThermalBridgeServer (5 MCP tools)
  explainer.rs          UNIT AnomalyExplainer
  adapters/
    qudag.rs            QuDagAdapter : MlDsaSigner + DagNetwork + PeerDiscovery
    synaptic.rs         SynapticMeshAdapter : MeshTransport + NodeRegistry
    ruflo_mcp.rs        RufloMcpAdapter : McpToolRegistry + ClaudeReasoner
    ws_sink.rs          WebSocketSink : OperatorChannel
```

### 4.6 `hoforras-dashboard` (Layer 9, TS)
```
dashboard/src/
  connection/socket.ts          ruflo WebSocket client (OperatorChannel-feed)
  scene/DistrictGraph.tsx       r3f force-directed graph (nodes/edges/particles)
  panels/BuildingCards.tsx      surplus/deficit + 24h sparklines
  panels/TradeTimeline.tsx      QuDAG consensus status
  panels/PipeHealthHeatmap.tsx  drill-down → WitnessLogView
  panels/AnomalyFeed.tsx        Claude explanations
  domain/events.ts              typed trade:executed / anomaly:detected
```

---

## 5. Key interface signatures (port traits, Rust-leaning)

These make the Pseudocode ports concrete for Phase R. Domain types only; `async` where I/O-bound.

```rust
// hoforras-ports/src/broker.rs
pub trait RulesEngine        { fn check(&self, t: &ProposedTrade) -> RuleVerdict; }
pub trait MarketGateway {
    async fn post_offer(&self, o: Offer) -> Result<()>;
    async fn post_bid(&self, b: Bid) -> Result<()>;
    async fn execute(&self, t: &AcceptedTrade) -> Result<()>;
    async fn route(&self, t: &AcceptedTrade, path: &[JunctionId]) -> Result<()>;
}
pub trait StrategyStore      { fn load(&self) -> Strategy; fn update(&self, a: Adaptation); }

// hoforras-ports/src/security.rs
pub trait CapabilityGate     { fn authorize(&self, cap: &Capability, req: &AccessRequest) -> Decision; }
pub trait WitnessChain {
    fn emit(&self, a: PrivilegedAction) -> Result<WitnessRecord>;
    fn verify(&self) -> Result<bool>;
}

// hoforras-ports/src/consensus.rs
pub trait MlDsaSigner        { fn sign(&self, e: DagEntry) -> Result<DagEntry>; fn verify(&self, e: &DagEntry) -> bool; }
pub trait DagNetwork         { async fn broadcast_and_await_consensus(&self, e: DagEntry) -> Result<Finality>; }

// hoforras-ports/src/inference.rs
pub trait GradientAggregator { async fn aggregate(&self, g: Vec<Gradient>) -> Result<Gradient>; } // ← Gradient ONLY
```

Construction (constructor injection → the mock seam):
```rust
let broker = BrokerAgent::new(seed_mesh, forecast, rules, market, evaluator, strategy, ledger, witness, consensus);
//                            ^ all trait objects / generics — mocked in unit tests, real adapters in main()
```

---

## 6. Runtime & concurrency model

| Concern | Decision |
|---------|----------|
| Async runtime | `tokio` (full) — single per-Appliance runtime; research pins `tokio 1` |
| Broker loop | `BrokerAgent::tick()` driven on an interval timer; one in-flight tick per node (no overlap) — keeps MRAP ordering invariant (FR-3.7) trivial |
| Inference | `ruv-swarm` ephemeral agents are spawned per call, `dissolve`d in a `Drop`/`finally` guard (`with_agent`) — never outlive the call |
| RVM coherence | in-process, synchronous, nanosecond-scale (NFR-3) — NOT on the async path; called inline from supervisor |
| Mesh transport | event-driven: Synaptic-Mesh DAG subscription → handler dispatch; no polling (CLAUDE.md SendMessage-first ethos mirrored at system level) |
| Consensus | async, non-blocking; broker awaits finality but yields the runtime |
| Backpressure | sensor `TypedEventBus` is bounded mpsc; on full, drop+`RejectionLog` (never block ingestion) |

**Serialization:** `postcard` for on-wire DAG/witness bytes (research pin), `serde` derive on all domain types. `canonical_bytes()` (pseudocode §2) is `postcard` so Ed25519/ML-DSA signatures are reproducible.

---

## 7. Data-flow architecture (the AC-7 path, end to end)

```
SEED(ESP32) ──raw──▶ RvcsiIngestAdapter ──▶ IngestionPipeline ──ThermalFrame──▶ TypedEventBus
                                                  │ (sign Ed25519, witness)
                                                  ▼
                                          SeedMeshAdapter.read_surplus_deficit
                                                  ▼
   NeuroDivergentAdapter ◀──with_agent── ForecastService ──demand_24h──▶ BrokerAgent.tick()
                                                  │ MONITOR→REASON
                                                  ▼
                                  DaaRulesAdapter.check  ──Violation?──▶ witness + STOP
                                                  │ pass
                                                  ▼ ACT
                                  DaaEconomyAdapter.post_offer/bid
                                                  ▼
                                  QuDagAdapter.broadcast_and_await_consensus ──< 1s──▶ Finality
                                                  ▼
                                  DaaEconomyAdapter.execute → route(pipe_route)
                                                  │  (each step:)
                                                  ▼
                                  RvmWitnessAdapter.emit {TradeSigned, Exec, Routing}
                                                  ▼ REFLECT→ADAPT
                                  DaaOrchestratorAdapter.evaluate → StrategyStore.update
                                                  │
   RuVectorAdapter.upsert ◀──state──┘            ▼
                                  SynapticMeshAdapter.publish(trade DAG entry)
                                                  ▼
   RufloMcpAdapter / WebSocketSink ──trade:executed──▶ Dashboard (animate pipe_route)
```

**Crossing boundaries:** the only things that leave the RVM partition are (a) the *aggregated* thermal availability via `CapabilityBroker` and (b) signed DAG entries (trades/gradients) via mesh. Raw `ThermalFrame` never appears on a mesh or capability edge — enforced by the `domain::Gradient`/`AggregatedThermalAvailability` type wall (§4.1, NFR-7).

---

## 8. Security architecture (Layer 4/5 cross-cut)

| Control | Mechanism | Where |
|---------|-----------|-------|
| Reading authenticity | Ed25519 witness per frame | sensor adapter |
| Privileged-action audit | rvm-witness 64-byte hash chain | node/audit.rs |
| Cross-building data wall | rvm-cap capability tokens, scope-gated, 6h expiry | node/capability.rs |
| Auto re-isolation | rvm-coherence mincut on anomaly | node/coherence.rs |
| Trade integrity | ML-DSA signatures | mesh/consensus.rs |
| Transport confidentiality | ML-KEM-1024 | qudag adapter |
| Directory-less identity | .dark domains, Kademlia | qudag adapter |
| Federation auth (deferred) | mTLS + Ed25519 | ruflo federation (stub) |

**Threat-model note:** the `CapabilityBroker` denies `Scope::Raw` *before* consulting the gate (defense in depth — pseudocode §5.2), so even a forged-but-valid capability cannot exfiltrate raw data. This is the architecture's single most important invariant and gets a dedicated property test in Phase R.

---

## 9. Deployment topology — District XIII pilot

```
Per building (×10):  Appliance host
  ├─ hoforras-node binary  (RVM domain; sensor+inference+memory+broker+mesh linked in)
  ├─ SEED nodes (ESP32)    ── LAN ──▶ Appliance :8080 ingestion
  └─ exposes :3001 WebSocket (dashboard), MCP stdio (Claude/ruflo)

District coordinator (×1, can be one of the 10):
  ├─ daa-orchestrator + rvm-kernel  (research First-Milestone step 2)
  └─ mesh-id: hoforras-district-xiii

Mesh substrate:  .thermal.budapest.dark  (QuDAG/Kademlia, ML-KEM-1024)
Dashboard:       static TS/React app → ws://appliance.district-xiii:3001
Claude:          claude mcp add hoforras -- npx ruflo mcp start --plugin hoforras-thermal-bridge --sensing-url http://localhost:8080
```

Bring-up commands (from research §143–175) map to: `node start --port 8080 --mesh-id hoforras-district-xiii`, `swarm create --agents 20 --behavior thermal_market_optimization --topology mesh`, `market init --db-path district_xiii_thermal.db`.

---

## 10. Cross-cutting concerns

| Concern | Approach |
|---------|----------|
| Config | `pipe_route` graph, junctions, governance overrides → typed config loaded at boot; validated at boundary (NFR-12) |
| Errors | `thiserror` domain errors per crate; ports return `Result`; no panics across port boundaries |
| Observability | structured tracing spans per MRAP phase → surfaced to dashboard via `OperatorChannel`; witness chain is the source of truth for audit |
| Time | injected `Clock` port (pseudocode §5.2 `now_injected`) — capability expiry & timestamps deterministic in tests |
| WASM hosting | `NeuroDivergentAdapter` hosts ruv-FANN models via wasm runtime in-process; no external service (NFR-5) |
| Simulated data | `SensorSource` has a `SimSensorAdapter` for the 30-day baseline lead-time (spec R-4) |

---

## 11. Architecture Decision Records (ADRs)

| # | Decision | Rationale | Alternative rejected |
|---|----------|-----------|----------------------|
| ADR-1 | Hexagonal ports-and-adapters; new `hoforras-ports` + `hoforras-domain` crates | Enables London mocking without third-party internals; enforces no-Ruv-types-in-domain | Direct dependence on Ruv crates in domain (untestable, leaks types) |
| ADR-2 | RVM coherence in-process, off the async path | NFR-3 nanosecond isolation; a network hop would dominate | Isolation as a service (latency blows NFR-3) |
| ADR-3 | `GradientAggregator` accepts only `domain::Gradient` | Compile-time guarantee of no-raw-egress (NFR-7/FR-3.8) | Runtime check (weaker; can regress) |
| ADR-4 | One MRAP tick in flight per node | Makes FR-3.7 ordering invariant trivial; avoids trade races | Concurrent ticks (race on rules/economy state) |
| ADR-5 | DAG transport over RPC for all cross-node propagation | NFR-9/10 self-healing + intermittent connectivity | RPC mesh (no offline tolerance, no replay) |
| ADR-6 | `CapabilityBroker` denies `Raw` before gate consult | Defense in depth for the headline invariant | Gate-only (single point of failure) |
| ADR-7 | Inference & memory live in `hoforras-node` (not separate crates) | They are node-local and inside the RVM boundary; keeps raw data in-partition | Separate crates (raw data would cross a crate/process edge) |
| ADR-8 | `tokio` single runtime per Appliance | Matches research pin; simple supervision | Multi-runtime (needless complexity at pilot scale) |

---

## 12. Build, test, and CI architecture (sets up Phase R)

```
cargo build --workspace            # all crates
cargo test  -p hoforras-broker     # London unit tests: BrokerAgent + mocks (fast, no Ruv crates)
cargo test  -p ... --features contract   # contract tests: adapters ↔ real Ruv crates
cargo test  --test acceptance      # AC-1..AC-8 outside-in (in-memory fakes for hw/net)
npm --prefix dashboard test        # r3f component tests with mocked socket
```

- **Mocking:** `mockall` on port traits (auto-generated mocks) for unit level; hand-written in-memory fakes for acceptance wiring.
- **CI gates:** (1) layering lint — domain/ports must not depend on any `daa_*|rvm_*|qudag|ruvector|ruv_*` crate; (2) property tests for FR-3.7, FR-4.4, FR-3.8, FR-4.3; (3) module ≤ 500 lines; (4) `build && test` green before any commit (CLAUDE.md).

---

## 13. Traceability — units → crates → ports → adapters

| Pseudocode unit | Crate · module | Key ports (mocked in R) | Adapter (contract-tested in R) |
|------------------|----------------|--------------------------|--------------------------------|
| IngestionPipeline | sensor/pipeline.rs | Validator, QualityScorer, WitnessSigner, EventEmitter, RejectionLog | Rvcsi, Ed25519, TypedEventBus |
| ForecastServiceImpl | node/forecast.rs | SwarmFactory, InferenceAgent | RuvSwarm, NeuroDivergent |
| BrokerAgent | broker/agent.rs | SeedMesh, ForecastService, RulesEngine, MarketGateway, TradeEvaluator, StrategyStore, EconomyLedger, WitnessChain, ConsensusGateway | DaaRules, DaaEconomy, DaaOrch |
| FederatedTrainer | broker/federated.rs | GradientAggregator, WitnessChain | PrimeCoordinator |
| CoherenceSupervisor | node/coherence.rs | MincutEngine, PartitionController, WitnessChain | RvmCoherence |
| CapabilityBroker | node/capability.rs | CapabilityGate, WitnessChain, Clock | RvmCap |
| AuditTrail | node/audit.rs | WitnessChain | RvmWitness |
| ConsensusGateway | mesh/consensus.rs | MlDsaSigner, DagNetwork, PeerDiscovery | QuDag |
| ThermalMemory | node/memory.rs | VectorIndex, GnnEngine | RuVector |
| MeshCoordinator | mesh/coordinator.rs | MeshTransport, NodeRegistry | SynapticMesh |
| ThermalBridgeServer | mesh/bridge.rs | McpToolRegistry (+5 backing svcs) | RufloMcp |
| AnomalyExplainer | mesh/explainer.rs | ClaudeReasoner, OperatorChannel | RufloMcp, WebSocketSink |
| DistrictDashboard | dashboard/* | OperatorChannel-feed (socket) | ruflo WebSocket |

---

## 14. Exit Criteria for the Architecture Phase

1. Every pseudocode unit is assigned to a crate/module ≤ 500 lines. ✅ (§4, §13)
2. Every port has a named production adapter and a crate home. ✅ (§3)
3. Dependency direction is acyclic and domain is Ruv-free (CI-enforceable). ✅ (§2, §12)
4. Concurrency/runtime model preserves the FR-3.7 ordering and FR-2.1 dissolve invariants. ✅ (§6)
5. The no-raw-egress invariant is realized as a *type wall* + capability gate + ADR. ✅ (§3, §7, §8, ADR-3/6)
6. Deployment topology matches the District XIII First Milestone. ✅ (§9)
7. Decisions are recorded with rationale and rejected alternatives. ✅ (§11)

**Next phase:** R — Refinement. Drive each unit red-green-refactor against its mocked ports (London), then contract-test each adapter against the real Ruv crate, working outside-in from the AC-7 acceptance test.
