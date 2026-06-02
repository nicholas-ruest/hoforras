# SPARC Specification — Hőforrás Budapest

**Project:** Hőforrás — Peer-to-Peer Thermal Energy Intelligence System
**District:** Budapest XIII (pilot), federating to V and VII
**Methodology:** SPARC (Specification · Pseudocode · Architecture · Refinement · Completion) — Reuven Cohen
**Test discipline:** London School TDD (outside-in, mock-driven, interaction-based)
**Document phase:** **S — Specification** (Phase 1 of 5)
**Status:** Draft v1.0
**Source of truth:** `.plans/research.md`

---

## 0. How to read this document within SPARC

SPARC is a five-phase pipeline. Each phase consumes the artifact of the previous one:

| Phase | Output artifact | This document |
|-------|-----------------|---------------|
| **S — Specification** | Requirements, contracts, acceptance criteria, glossary | **← you are here** |
| P — Pseudocode | Algorithm-level designs per component, derived from §6 contracts | downstream |
| A — Architecture | Crate/module topology, deployment, interface bindings | downstream |
| R — Refinement | TDD red-green-refactor cycles against §6 contracts & §9 criteria | downstream |
| C — Completion | Integration, milestone validation, witness-auditable demo | downstream |

The Specification phase fixes **what** and **why**, never **how**. Every requirement here is written to be (a) testable and (b) expressible as a London-School *collaborator contract* — an interaction between a unit and its mocked neighbours. §6 and §7 are the explicit bridge into the Pseudocode and Refinement phases.

---

## 1. Problem Statement & Vision

Budapest's District XIII has buildings sitting on geothermal resources and thermal-waste streams that are, at any given hour, in **surplus** or **deficit**. Today there is no mechanism for a building in surplus to sell heat to a neighbour in deficit autonomously, verifiably, and safely.

**Hőforrás** is a peer-to-peer thermal energy market running on **edge hardware** (Cognitum Appliances), built almost entirely from Ruv's existing repositories, predominantly in Rust with a TypeScript/React operator surface. Each building is an autonomous market participant that senses its thermal state, forecasts demand, brokers trades with neighbours, and routes physical heat — with every privileged action cryptographically witnessed and quantum-resistantly agreed.

### Vision statement
> A self-organising, self-healing district thermal mesh in which buildings autonomously trade heat, with sub-second trade finality, no central directory, no raw-data sharing across buildings, and a complete forensic audit trail — all on edge hardware with no GPU and no cloud dependency.

---

## 2. Stakeholders & Actors

| Actor | Role | Primary concerns |
|-------|------|------------------|
| District Operator | Human supervisor at dashboard | Safety, observability, override, anomaly explanations |
| Building Node (Appliance) | Autonomous software agent per building | Maximise local thermal value within safety rules |
| SEED Sensor | Physical sensing device (ESP32 + Cognitum) | Produce signed, validated thermal readings |
| Thermal Market | Emergent system of all building nodes | District-wide load balancing, cascade prevention |
| Auditor / Regulator | External reviewer (post-hoc) | Tamper-evident proof of every trade & reading |
| Claude (via ruflo MCP) | AI reasoning layer | Anomaly explanation, natural-language operator Q&A |
| Federation Peer | Appliance mesh in another district | Cross-district market signals without raw-data sharing |

---

## 3. Scope

### 3.1 In scope (Pilot — Hőforrás Alpha, District XIII)
- Sensor ingestion of thermal, vibration, pipe-resonance, and pressure signals with Ed25519 witnessing.
- On-device LSTM / N-BEATS thermal anomaly detection, pipe-burst precursor prediction (6–48 h horizon), and 24 h demand forecasting.
- Autonomous thermal-energy broker per building implementing the MRAP loop (Monitor → Reason → Act → Reflect → Adapt).
- Thermal-credit token economy (kWh-equivalent) with enforced governance rules.
- Per-building coherence-domain isolation, capability-gated cross-building data access, and hash-chained witness audit.
- Quantum-resistant, DAG-based signed trade agreements with sub-second finality and directory-less peer discovery.
- Persistent vector/GNN thermal-state memory with sub-millisecond similarity search.
- District-scale self-healing swarm coordination.
- AI orchestration via ruflo with a ThermalSense-Bridge MCP server exposing thermal tools to Claude.
- TypeScript/React operator dashboard with live 3D district map, trade timeline, pipe-health heatmap, and anomaly feed.

### 3.2 Out of scope (Pilot)
- Physical valve/pump actuation firmware certification (modeled, not field-certified).
- Billing/settlement to fiat currency or regulatory financial reporting.
- Multi-district federation beyond signalling stubs (full federation is a later milestone).
- Mobile-native clients.

### 3.3 Assumptions
- Each building has one Cognitum Appliance and ≥1 SEED node.
- Pipe network topology (junctions, routes) is known and supplied as configuration.
- Appliances have intermittent but generally-available LAN/mesh connectivity.
- Upstream Ruv crates/packages are consumable at the pinned versions in §5.

### 3.4 Constraints
- **Edge-only inference:** all neural inference runs as WASM on-device; no GPU, no cloud.
- **No raw-data egress:** raw sensor data must never cross a building's partition boundary.
- **Rust-first core**, TypeScript-accessible surface.
- File size ≤ 500 lines per module; input validated at every system boundary (per project rules).

---

## 4. Glossary (Ubiquitous Language)

| Term | Definition |
|------|------------|
| **Appliance** | Edge hardware unit per building running the full Hőforrás node stack. |
| **SEED node** | Cognitum sensor (ESP32 firmware) emitting signed thermal frames. |
| **ThermalFrame** | Validated, quality-scored, witnessed sensor reading (adapted from rvcsi `CsiFrame`). |
| **Thermal credit** | kWh-equivalent token of account in the market economy. |
| **MRAP** | Monitor → Reason → Act → Reflect → Adapt autonomous control loop (daa SDK). |
| **Coherence domain** | RVM-isolated partition; one per building node. |
| **Witness record** | 64-byte hash-chained audit entry of a privileged action. |
| **Capability** | Unforgeable token granting scoped, expiring rights (rvm-cap). |
| **CommEdge** | Capability-gated communication channel between partitions. |
| **Trade agreement** | Signed DAG entry recording a thermal trade (QuDAG). |
| **.dark domain** | Directory-less peer namespace (e.g. `pozsonyi14.thermal.budapest.dark`). |
| **ThermalSense-Bridge** | MCP server exposing thermal tools to Claude (adapted from @ruvnet/rvagent). |
| **District mesh** | Self-healing DAG-based swarm fabric across all Appliances (Synaptic-Mesh). |

---

## 5. Component Inventory & Dependency Pins

All nine layers map to concrete Ruv repos/packages. Specification fixes these as the canonical dependency set; Architecture phase binds them into crates.

| Layer | Component | Source | Pin |
|-------|-----------|--------|-----|
| 1 | Sensor ingestion runtime | `@ruv/rvcsi`, `ruvnet/rvcsi` | latest |
| 1 | SEED firmware + Ed25519 witness chain | `ruvnet/RuView` | latest |
| 1 | Sensing→agent MCP bridge | `@ruvnet/rvagent` | latest |
| 2 | Neural inference (LSTM / N-BEATS) | `ruvnet/ruv-FANN` (Neuro-Divergent) | — |
| 2 | Ephemeral swarm agents | `ruv-swarm` | `1.0.5` cargo / `1.0.19` npm |
| 3 | Thermal broker (MRAP, economy, rules, AI) | `ruvnet/daa` — `daa-rules`, `daa-economy`, `daa-ai`, `daa-orchestrator` | `0.2.1` |
| 3 | Federated district training | `daa-prime-core`, `daa-prime-dht`, `daa-prime-trainer`, `daa-prime-coordinator` | `0.2.1` |
| 4 | Node isolation + audit | `ruvnet/rvm` — `rvm-kernel`, `rvm-coherence`, `rvm-witness`, `rvm-security`, `rvm-cap`, `rvm-proof` | workspace path |
| 5 | Quantum-resistant consensus | `ruvnet/QuDAG` (ML-DSA, ML-KEM-1024, QR-Avalanche) | latest |
| 6 | Vector + GNN memory | `ruvnet/RuVector`, `ruvector` | `128`-dim, hnsw |
| 7 | District swarm fabric | `ruvnet/Synaptic-Mesh` | — |
| 8 | AI orchestration + MCP | `ruvnet/ruflo`, `ruflo` npm, `@ruvnet/rvagent` | latest |
| 9 | Frontend | `react ^18`, `@react-three/fiber ^8`, `@react-three/drei ^9`, `three ^0.163` | as listed |

Cargo workspace members (from research): `crates/hoforras-sensor`, `crates/hoforras-broker`, `crates/hoforras-node`, `crates/hoforras-mesh`.

---

## 6. Functional Requirements & Collaborator Contracts

Each requirement is stated as **FR-n** with London-School framing: the **unit under test (UUT)**, its **collaborators** (to be mocked), and the **interaction** to be verified. This is the contract the Pseudocode and Refinement phases must satisfy.

### Layer 1 — Sensor Ingestion (`hoforras-sensor`)

**FR-1.1 — Thermal frame construction.** The ingestion layer SHALL transform raw multi-source sensor input into a `ThermalFrame { node_id, timestamp_ns, temperature_celsius, pipe_vibration_hz, fluid_pressure_bar, ground_thermal_gradient, quality_score, validation, witness }`.
- *UUT:* `IngestionPipeline`. *Mocks:* `Validator`, `QualityScorer`, `WitnessSigner`.
- *Interaction:* on `ingest(raw)`, UUT calls `Validator::validate` → `QualityScorer::score` → `WitnessSigner::sign` in order, emitting a frame only when validation passes.

**FR-1.2 — Validation gating.** Frames failing validation SHALL NOT be emitted downstream; they SHALL be recorded as rejected with a reason.
- *Interaction:* given `Validator` mock returns `Invalid`, UUT MUST NOT call `EventEmitter::emit` and MUST call `RejectionLog::record`.

**FR-1.3 — Witnessed readings.** Every emitted frame SHALL carry a valid Ed25519 signature over its canonical bytes.
- *Interaction:* `WitnessSigner::sign(frame_bytes)` is invoked exactly once per emitted frame; signature is attached.

**FR-1.4 — Typed, confidence-scored event emission.** Emitted events SHALL carry a `QualityScore` and `ValidationStatus`.

**FR-1.5 — MCP sensing tools.** The ThermalSense-Bridge SHALL expose live readings to agents via MCP tool calls (see FR-8.x).

### Layer 2 — Local Neural Inference (`ruv-FANN` / `ruv-swarm`)

**FR-2.1 — Ephemeral forecaster spawn.** The system SHALL spawn a per-building ephemeral inference agent (`type: analyst`, `neuralModel: lstm`, capabilities `anomaly_detection`, `demand_forecasting`, `pipe_health_scoring`), solve, and dissolve it.
- *UUT:* `ForecastService`. *Mocks:* `SwarmFactory`, `InferenceAgent`.
- *Interaction:* `SwarmFactory::spawn(profile)` → `InferenceAgent::infer(window)` → `InferenceAgent::dissolve()`; dissolve MUST occur even on inference error.

**FR-2.2 — Anomaly detection.** SHALL flag thermal anomalies from a reading window, returning a typed anomaly with confidence.

**FR-2.3 — Pipe-burst precursor prediction.** SHALL produce a 6–48 h warning window when precursor patterns are detected.

**FR-2.4 — 24 h demand forecast.** SHALL produce a 24 h per-building thermal-demand forecast consumable by the broker's trade scheduler.

**FR-2.5 — Latency budget.** A single building prediction SHALL complete in < 100 ms on-device (NFR-tied, see §8).

**FR-2.6 — Edge-only.** Inference SHALL run as WASM on-device; no network call to a remote inference service is permitted (assert *absence* of `RemoteInferenceClient` interaction).

### Layer 3 — Autonomous Thermal Energy Broker (`hoforras-broker`, `daa`)

**FR-3.1 — MRAP loop.** Each node SHALL run Monitor → Reason → Act → Reflect → Adapt.
- *UUT:* `BrokerAgent`. *Mocks:* `SeedMesh` (monitor source), `ForecastService`, `MarketGateway` (act), `TradeEvaluator` (reflect), `StrategyStore` (adapt).
- *Interaction per tick:* `SeedMesh::read_surplus_deficit()` → `BrokerAgent` decides → `MarketGateway::post_offer | post_bid | execute | route` → `TradeEvaluator::evaluate(result)` → `StrategyStore::update(adaptation)`.

**FR-3.2 — Offer/bid decision.** Reason SHALL decide offer vs bid, price, and duration from surplus/deficit + forecast.

**FR-3.3 — Trade execution & routing.** Act SHALL post offers/bids, execute accepted trades, and route thermal energy along a pipe route.

**FR-3.4 — Reflection.** Reflect SHALL evaluate whether a completed trade delivered expected value, producing a signal that feeds Adapt.

**FR-3.5 — Adaptation.** Adapt SHALL update pricing strategy and forecast-model inputs based on reflection.

**FR-3.6 — Thermal credit economy.** `daa-economy` SHALL account trades in kWh-equivalent thermal credits (rUv token replaced by thermal credit).

**FR-3.7 — Governance rules (HARD constraints).** `daa-rules` SHALL enforce, rejecting any trade that violates:
- `max_daily_thermal_transfer_kwh = 500.0`
- `min_safety_reserve_percent = 0.15`
- `pipe_pressure_ceiling_bar = 6.0`
- `trade_window_hours = 6`
- *Interaction:* `RulesEngine::check(proposed_trade)` is called *before* `MarketGateway::execute`; a rule violation MUST short-circuit execution and emit a rejection (assert execute is *not* called).

**FR-3.8 — Federated district model.** `daa-prime-coordinator` SHALL perform Byzantine-fault-tolerant gradient aggregation so N buildings collectively train a district predictor **without sharing raw building data** (assert only gradients/aggregates cross, never raw frames).

**FR-3.9 — AI-assisted anomaly reasoning.** `daa-ai` SHALL integrate Claude for anomaly reasoning (see Layer 8).

### Layer 4 — Node Isolation & Security (`hoforras-node`, `rvm`)

**FR-4.1 — Coherence domain per building.** Each building node SHALL run as an RVM coherence domain.

**FR-4.2 — Automatic re-isolation.** On anomalous patterns (injected bad readings, hardware fault, compromised Appliance), `rvm-coherence`'s mincut SHALL split the offending node into an isolated partition with no manual intervention or restart.
- *UUT:* `CoherenceSupervisor`. *Mocks:* `AnomalySource`, `MincutEngine`, `PartitionController`.
- *Interaction:* anomaly signal → `MincutEngine::recompute(graph)` → `PartitionController::isolate(node)`; system continues serving other nodes.

**FR-4.3 — Witness audit trail.** `rvm-witness` SHALL record every privileged action (trade signed, reading accepted, routing decision) as a 64-byte hash-chained witness record; the chain SHALL be verifiable.

**FR-4.4 — Capability-gated access.** No building SHALL read another building's raw sensor data. Only **aggregated thermal availability** crosses partition boundaries, via capability-gated CommEdges using unforgeable `rvm-cap` tokens with expiry.
- *Interaction:* cross-partition read requires a valid `Capability { Rights::READ, Scope::AggregatedThermalAvailability, Expiry::Hours(6) }`; a request for raw scope or expired token MUST be denied.

**FR-4.5 — Real-time isolation latency.** Partition switching SHALL operate within the documented RVM envelope (see NFR §8): partition switch ~6 ns, 16-node mincut ~331 ns, witness emit ~17 ns.

### Layer 5 — Quantum-Resistant Trade Consensus (`QuDAG`)

**FR-5.1 — Signed trade agreement as DAG entry.** A `ThermalTradeAgreement { seller, buyer, kwh_offered, duration_hours, credit_price_per_kwh, pipe_route, valid_from }` SHALL be encoded as a DAG entry signed with an ML-DSA key.
- *UUT:* `ConsensusGateway`. *Mocks:* `MlDsaSigner`, `DagNetwork`.
- *Interaction:* `DagEntry::new(trade)` → `MlDsaSigner::sign` → `DagNetwork::broadcast_and_await_consensus(entry)`.

**FR-5.2 — Post-quantum cryptography.** Signatures SHALL use ML-DSA; transport encryption SHALL use ML-KEM-1024.

**FR-5.3 — Sub-second finality.** QR-Avalanche consensus SHALL provide sub-second finality (NFR §8).

**FR-5.4 — Directory-less discovery.** Nodes SHALL discover peers via Kademlia DHT over `.dark` domains in the `.thermal.budapest.dark` namespace, with no directory server.

**FR-5.5 — Tamper-evidence.** A mutated agreement SHALL fail signature/consensus verification.

### Layer 6 — Vector & GNN Memory (`RuVector`)

**FR-6.1 — Time-series thermal history.** SHALL persist per-building thermal-state history.

**FR-6.2 — Pattern embeddings.** SHALL store 128-dim vector embeddings of each building's thermal consumption patterns.

**FR-6.3 — Similarity search.** SHALL find buildings whose surplus profiles match a given deficit embedding, with `topK` and filter (`thermal_surplus_available`, `district`), using HNSW/DiskANN.
- *UUT:* `ThermalMemory`. *Mock:* `VectorIndex`.
- *Interaction:* `search({vector, topK, filter})` returns ordered candidates; sub-millisecond per NFR §8.

**FR-6.4 — GNN graph inference.** SHALL run graph-structured inference natively over the district graph (buildings = nodes, pipes = weighted edges).

**FR-6.5 — Witness replay.** SHALL store the RVM witness trail to support forensic replay.

### Layer 7 — District Swarm Coordination (`Synaptic-Mesh`)

**FR-7.1 — DAG-based propagation.** Market state, anomaly signals, and consensus decisions SHALL propagate as signed DAG entries, not RPC calls.

**FR-7.2 — Self-healing.** The mesh SHALL self-heal when nodes drop and resume when they return.
- *UUT:* `MeshCoordinator`. *Mocks:* `NodeRegistry`, `DagTransport`.
- *Interaction:* node-drop event → coordinator reconfigures without losing committed state; node-return → state resync.

**FR-7.3 — Collective behaviours.** SHALL support district-wide thermal load balancing, cascade-failure prevention, and heat-wave emergency routing.

**FR-7.4 — Micro-mind node model.** Each Appliance SHALL act as a neural-mesh micro-mind node with per-node WASM inference.

### Layer 8 — AI Orchestration & Reasoning (`ruflo`)

**FR-8.1 — ThermalSense-Bridge MCP server.** SHALL register MCP tools callable by Claude: `hoforras.thermal.district_status`, `hoforras.trade.active_agreements`, `hoforras.pipe.health_scores`, `hoforras.anomaly.recent`, `hoforras.forecast.demand_24h`.
- *UUT:* `ThermalBridgeServer`. *Mocks:* per-tool backing service.
- *Interaction:* each registered tool, when invoked, delegates to exactly its backing service and returns typed results; unknown tool → typed error.

**FR-8.2 — Anomaly explanation.** On anomaly, the system SHALL query Claude (via ruflo / `daa-ai`) for a natural-language explanation surfaced to the operator.

**FR-8.3 — Federation.** When expanding to Districts V/VII, Appliance meshes SHALL discover each other, authenticate via mTLS + Ed25519, and federate market signals **without sharing raw building data**.

### Layer 9 — Operator Frontend (React + r3f)

**FR-9.1 — Live 3D district map.** SHALL render a force-directed 3D graph: buildings as nodes, pipe connections as weighted edges, active trades as animated particle streams.

**FR-9.2 — WebSocket feed.** SHALL connect to the Appliance via ruflo WebSocket and react to `trade:executed` (update edge weight, animate flow along `pipe_route`) and `anomaly:detected` (highlight node, request Claude explanation).

**FR-9.3 — Per-building cards.** SHALL show surplus/deficit with 24 h forecast sparklines.

**FR-9.4 — Trade timeline.** SHALL show active trade agreements with QuDAG consensus status.

**FR-9.5 — Pipe-health heatmap.** SHALL render pipe-health with drill-down into the RVM witness log.

**FR-9.6 — Anomaly feed.** SHALL show an anomaly alert feed with Claude-generated natural-language explanations.

---

## 7. London School TDD Framework (binding for Phase R — Refinement)

This specification is **mock-driven by design**. The Refinement phase MUST follow the London (mockist, outside-in) school:

### 7.1 Principles
1. **Outside-in.** Start from the operator-visible behaviour (acceptance test, §9) and drive inward, discovering collaborators as you go.
2. **Interaction over state.** Verify *how* a unit talks to its collaborators (messages sent, order, cardinality), not internal state — RVM partition logic, consensus, and ML inference are otherwise expensive/non-deterministic to assert by state.
3. **Tell, don't ask.** Units command collaborators; contracts in §6 are expressed as message expectations.
4. **Mock at the boundary you own.** Mock the *roles* (`Validator`, `MarketGateway`, `MlDsaSigner`, `VectorIndex`, …), not third-party internals. Each Ruv crate is wrapped behind a Hőforrás **port** (trait) so it can be doubled.
5. **Need-driven design.** A collaborator exists only because a unit needs it; this keeps modules ≤ 500 lines and boundaries sharp.

### 7.2 Test taxonomy
| Level | Scope | Doubles used | Example |
|-------|-------|--------------|---------|
| Acceptance | Operator-visible scenario (§9) | Real wiring, stubbed hardware/network | "Autonomous trade with full witness trail" |
| Unit (London) | One UUT + mocked collaborators | Mocks/spies for all ports | FR-3.7 rules short-circuit |
| Contract | Port ↔ real Ruv adapter | Real crate, narrow | `MlDsaSigner` adapter signs/verifies |
| Property | Invariants | Generators | "No raw frame ever crosses a partition" (FR-4.4/FR-3.8) |

### 7.3 Ports to be defined (trait seams for mocking)
`SensorSource`, `Validator`, `QualityScorer`, `WitnessSigner`, `EventEmitter`, `RejectionLog`, `SwarmFactory`, `InferenceAgent`, `ForecastService`, `SeedMesh`, `MarketGateway`, `RulesEngine`, `TradeEvaluator`, `StrategyStore`, `EconomyLedger`, `GradientAggregator`, `MincutEngine`, `PartitionController`, `WitnessChain`, `CapabilityGate`, `MlDsaSigner`, `DagNetwork`, `PeerDiscovery`, `VectorIndex`, `GnnEngine`, `MeshTransport`, `NodeRegistry`, `McpToolRegistry`, `ClaudeReasoner`, `OperatorChannel`.

### 7.4 Definition of Done (per unit, Phase R)
- A failing London-style test existed first (red).
- Interaction expectations on all collaborators are asserted (no un-asserted critical message).
- Governance/safety/no-raw-egress invariants (FR-3.7, FR-4.4, FR-3.8) covered by property tests.
- Module ≤ 500 lines; inputs validated at the boundary.

---

## 8. Non-Functional Requirements

| ID | Category | Requirement | Source / target |
|----|----------|-------------|-----------------|
| NFR-1 | Latency — inference | Single-building prediction < 100 ms on-device | FR-2.5 |
| NFR-2 | Latency — consensus | Trade finality < 1 s (QR-Avalanche) | FR-5.3 |
| NFR-3 | Latency — isolation | Partition switch ~6 ns; 16-node mincut ~331 ns; witness emit ~17 ns | FR-4.5 |
| NFR-4 | Latency — memory | Similarity query sub-millisecond | FR-6.3 |
| NFR-5 | Resource | No GPU; no cloud inference; WASM on-device | Constraint §3.4 |
| NFR-6 | Security — crypto | Post-quantum: ML-DSA sign, ML-KEM-1024 encrypt | FR-5.2 |
| NFR-7 | Security — isolation | Raw sensor data never crosses partition boundary | FR-4.4, FR-3.8 |
| NFR-8 | Security — audit | Every privileged action witnessed, chain verifiable | FR-4.3 |
| NFR-9 | Resilience | Mesh self-heals on node drop/return without state loss | FR-7.2 |
| NFR-10 | Availability | District operates under intermittent connectivity (DAG, not RPC) | FR-7.1 |
| NFR-11 | Scalability | Pilot 10–20 building nodes; design for district federation | §3, FR-8.3 |
| NFR-12 | Maintainability | Modules ≤ 500 lines; trait-seam ports; Rust-first, TS surface | Project rules |
| NFR-13 | Observability | Live dashboard reflects trades/anomalies/health in real time | Layer 9 |
| NFR-14 | Determinism (test) | Hardware, network, crypto, inference behind mockable ports | §7 |

---

## 9. Acceptance Criteria — First Milestone (Hőforrás Alpha, District XIII)

These are the outermost (acceptance-level) London tests. Each is **Given/When/Then** and traces to FRs.

**AC-1 — Sensor deployment & ingestion.**
*Given* 10 Cognitum SEED nodes running the rvcsi thermal-firmware fork, *when* they emit readings, *then* each accepted reading becomes a witnessed `ThermalFrame` and each rejected reading is logged with a reason. *(FR-1.1–1.4)*

**AC-2 — District coordinator up.**
*Given* one Cognitum Appliance running `daa-orchestrator` + `rvm-kernel`, *when* started, *then* it forms a coherence domain and begins coordinating nodes. *(FR-3.1, FR-4.1)*

**AC-3 — 30-day thermal baseline.**
*Given* ruv-FANN LSTM models fed 30 days of frames, *when* baselining completes, *then* the system produces per-building 24 h demand forecasts. *(FR-2.4)*

**AC-4 — Broker between 3 buildings with governance.**
*Given* a daa broker across 3 buildings with `daa-rules` set to the §FR-3.7 limits, *when* a proposed trade would breach a limit, *then* it is rejected before execution; otherwise it proceeds. *(FR-3.1–3.3, FR-3.7)*

**AC-5 — First signed thermal trade via QuDAG.**
*Given* a valid trade agreement, *when* submitted, *then* it is ML-DSA-signed, broadcast, reaches QR-Avalanche consensus in < 1 s, and is tamper-evident. *(FR-5.1–5.5, NFR-2)*

**AC-6 — Live dashboard.**
*Given* ruflo MCP + ruvector GNN state, *when* the operator opens the dashboard, *then* the 3D district map, trade timeline, pipe-health heatmap, and anomaly feed render and update over WebSocket. *(FR-8.1, FR-6.x, FR-9.x)*

**AC-7 — Fully autonomous trade with complete audit (headline milestone).**
*Given* the full stack, *when* a building enters surplus and a neighbour enters deficit, *then* a peer-to-peer thermal trade executes **with no human intervention**, heat is routed along a pipe route, and a **complete rvm-witness audit trail** exists for every privileged action in the trade. *(FR-3.x, FR-4.3, FR-5.x — integrative)*

**AC-8 — No-raw-egress invariant (property).**
Across all of AC-1…AC-7, no raw sensor frame crosses any partition boundary; only aggregated thermal availability and gradients do. *(FR-4.4, FR-3.8, NFR-7)*

---

## 10. Data Contracts (canonical types)

These typed contracts are fixed in Specification; Pseudocode/Architecture bind their representation.

```text
ThermalFrame {
  node_id: NodeId
  timestamp_ns: u64
  temperature_celsius: f32
  pipe_vibration_hz: f32
  fluid_pressure_bar: f32
  ground_thermal_gradient: f32
  quality_score: QualityScore        // from rvcsi_core
  validation: ValidationStatus       // from rvcsi_core
  witness: Ed25519Signature
}

ThermalTradeAgreement {
  seller: NodeId                      // e.g. pozsonyi14.thermal.budapest.dark
  buyer:  NodeId
  kwh_offered: f32
  duration_hours: u32                 // ≤ trade_window_hours (6)
  credit_price_per_kwh: f32
  pipe_route: Vec<JunctionId>
  valid_from: Timestamp
}

Capability {
  rights: Rights                      // READ
  scope:  Scope                       // AggregatedThermalAvailability (never Raw across boundary)
  expiry: Expiry                      // Hours(6)
}

WitnessRecord { /* 64 bytes, hash-chained */ }

GovernanceRules {
  max_daily_thermal_transfer_kwh: 500.0
  min_safety_reserve_percent: 0.15
  pipe_pressure_ceiling_bar: 6.0
  trade_window_hours: 6
}
```

---

## 11. Risks & Open Questions (to resolve before/within Phase A)

| ID | Risk / Question | Impact | Disposition |
|----|-----------------|--------|-------------|
| R-1 | Upstream Ruv crate versions/APIs may drift from pins in §5 | Build breakage | Wrap each behind a port (§7.3); contract-test adapters |
| R-2 | Physical heat routing is modeled, not actuated, in pilot | Realism gap | Explicitly out of scope §3.2; route = data only |
| R-3 | NFR-3 RVM nanosecond figures are vendor benchmarks | Over-promise | Treat as targets; re-measure on Appliance hardware in Phase C |
| R-4 | 30-day baseline requires real data lead time | Schedule | Provide simulated-data harness behind `SensorSource` port |
| R-5 | Federation auth (mTLS + Ed25519) only stubbed in pilot | Security scope | FR-8.3 acceptance deferred to later milestone |
| Q-1 | Exact pricing strategy for Reason step (FR-3.2) | Behaviour | Specify in Pseudocode phase; testable via `StrategyStore` mock |
| Q-2 | GNN embedding scheme / 128-dim feature definition | Memory quality | Define in Pseudocode; FR-6.2 fixes dimensionality only |
| Q-3 | Anomaly taxonomy granularity (FR-2.2) | Test scope | Enumerate in Pseudocode; confidence-scored contract fixed here |

---

## 12. Traceability Matrix (FR → AC → NFR)

| FR group | Layer | Acceptance | Key NFRs |
|----------|-------|-----------|----------|
| FR-1.x | 1 Ingestion | AC-1 | NFR-8, NFR-14 |
| FR-2.x | 2 Inference | AC-3 | NFR-1, NFR-5 |
| FR-3.x | 3 Broker | AC-2, AC-4, AC-7 | NFR-7, NFR-12 |
| FR-4.x | 4 Isolation/Security | AC-7, AC-8 | NFR-3, NFR-7, NFR-8 |
| FR-5.x | 5 Consensus | AC-5, AC-7 | NFR-2, NFR-6 |
| FR-6.x | 6 Memory/GNN | AC-6 | NFR-4 |
| FR-7.x | 7 Swarm | AC-7 (integrative) | NFR-9, NFR-10, NFR-11 |
| FR-8.x | 8 AI/MCP | AC-6 | NFR-11, NFR-13 |
| FR-9.x | 9 Frontend | AC-6 | NFR-13 |

---

## 13. Exit Criteria for the Specification Phase

The Specification phase is complete when:
1. Every component in `research.md` (all nine layers) is represented by ≥ 1 testable FR. ✅ (§5–6)
2. Every FR is expressible as a London-School collaborator contract. ✅ (§6, §7.3)
3. NFRs are quantified and traced. ✅ (§8, §12)
4. Acceptance criteria cover the First Milestone end-to-end, including the headline autonomous-trade-with-audit scenario. ✅ (§9)
5. Safety/security invariants (governance limits, no-raw-egress, witnessed actions) are explicit and property-testable. ✅ (FR-3.7, FR-4.4, FR-3.8)
6. Risks/open questions are logged with a disposition into later phases. ✅ (§11)

**Next phase:** P — Pseudocode. Produce algorithm-level designs per UUT in §6, written against the §7.3 ports so the Refinement phase can drive them red-green-refactor.
