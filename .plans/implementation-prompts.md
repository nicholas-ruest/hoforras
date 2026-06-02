# Implementation Prompts — Hőforrás Budapest

A dependency-ordered series of build prompts derived from `.plans/adr/` (ADR-0001…0015) and
`.plans/ddd/` (DDD-00…09), grounded in `.plans/sparc.md`.

## How to use

- Run the prompts **in order** (Prompt 0 → Prompt 10). Each prompt's *Depends on* line lists the
  prompts that must be **complete and green** before it starts. Prompt 0 depends on nothing.
- Each prompt is a **copy-paste block**. It spawns a coordinated swarm team (SendMessage-first, per
  `CLAUDE.md`), drives all five SPARC phases for that component, then tests → validates → benchmarks
  → optimizes, runs on `/loop` until the exit criteria are met, and finishes with a review that
  confirms **every cited ADR is fully implemented**.
- ADR-0001 (SPARC + London-School TDD) governs **every** prompt; it is not re-listed each time.
- "ADR #N" in a prompt means `ADR-00NN`; "DDD #N" means the tactical doc `ddd-0N-*` (DDD #0 is the
  strategic design).

## Dependency graph (build order)

```
P0  Foundation: hoforras-domain + hoforras-ports + CI guardrails        (no deps)
      │
      ├──────────────┬───────────────┬───────────────┬─────────────────┐
      ▼              ▼               ▼               ▼                 ▼
P1 Node Isolation  P4 Thermal     P5 Trade        (P2 needs P1         (others need P0)
   & Security        Memory          Consensus       for witness)
   (OHS/kernel)      │               │
      │              │               ▼
      ▼              │             P6 District Mesh  (needs P5 DAG entries)
P2 Sensing &         │
   Ingestion ────────┤
      │              │
      ▼              │
P3 Forecasting       │
   & Anomaly         │
      └──────┬───────┴───────┬───────────────┐
             ▼                ▼               ▼
          P7 Thermal Market & Brokerage (CORE)   (needs P1,P2,P3,P4,P5)
             │
             ▼
          P8 AI Orchestration (ThermalSense-Bridge)   (needs P7,P3,P2,P5)
             │
             ▼
          P9 Operator Experience (Dashboard)          (needs P6,P8 event shapes)
             │
             ▼
          P10 Appliance Integration & Completion       (needs ALL)
```

**Rationale for the order:** the pure model/contracts (P0) come first because nothing compiles
without them. Node Isolation (P1) is next because its capability tokens and witness chain are an
Open Host Service / Shared Kernel that every other context consumes (DDD-00 §4). Sensing (P2) needs
witnessing; Forecasting (P3) needs sensor windows. Memory (P4) and Consensus (P5) are independent
generic subdomains buildable in parallel once P0 lands; Mesh (P6) needs Consensus's DAG entries.
The **core** Thermal Market (P7) is the integration hub and is built only after every collaborator
it consumes exists. AI Orchestration (P8) exposes those services; the Dashboard (P9) conforms to the
events P6/P8 publish; Completion (P10) wires the single tokio runtime, benchmarks the NFRs, and
demonstrates AC-7 live.

---

## Prompt 0 — Workspace Foundation (domain, ports, CI guardrails)

- **Depends on:** nothing — **build this first.**
- **Incorporates:** ADR #2 (hexagonal), ADR #3 (pure `hoforras-domain`/`hoforras-ports` crates),
  ADR #5 (define the `Gradient` type wall), ADR #10 (Appliance binary + single tokio runtime
  skeleton), ADR #14 (postcard/serde canonical bytes) · DDD #0 (strategic — ubiquitous language &
  value types) and the Value-Object sections of DDD #1–#9.
- **SPARC ref:** Part 1 §10 (data contracts), Part 3 §2–§5.

```
Implement the Hőforrás workspace foundation — the pure `hoforras-domain` and `hoforras-ports`
crates plus the Cargo workspace and CI guardrails — incorporating ADR-0002, ADR-0003, ADR-0005,
ADR-0010, ADR-0014 and DDD-00 (ubiquitous language) with the value-object definitions from DDD-01
through DDD-09. Build the Cargo workspace (members: hoforras-domain, hoforras-ports,
hoforras-sensor, hoforras-broker, hoforras-node, hoforras-mesh). In hoforras-domain define every
shared value type from sparc.md Part 1 §10 (ThermalFrame, ThermalBalance, ThermalTradeAgreement,
ProposedTrade, Capability+Scope, WitnessRecord, PrivilegedAction, Gradient, NodeId/.dark,
JunctionId, GovernanceRules consts) with serde derives; canonical_bytes = postcard. In
hoforras-ports define all ~30 port traits from sparc.md Part 2 §1 over domain types only. CRITICAL:
hoforras-domain and hoforras-ports must NOT depend on any daa_*/rvm_*/qudag/ruvector/ruv_* crate —
add a CI layering lint that fails the build on violation (ADR-0002/0003). Add mockall mock
generation for every port. Stand up the hoforras-node Appliance binary skeleton on a single tokio
runtime (ADR-0010). Spawn a coordinated swarm team (SendMessage-first per CLAUDE.md): system-architect,
coder, tester, reviewer — all in one message, run_in_background, each told who to message next. Drive
all five SPARC phases for this slice, then fully implement, write London-style + compile-time tests,
validate (cargo build --workspace && cargo test green; layering lint green; Gradient has no
From<ThermalFrame>), benchmark trivial serde roundtrips, and optimize. Continue until complete via
/loop. Then review and ensure ADR-0002, 0003, 0005, 0010, 0014 are each FULLY implemented (cite the
file/line that satisfies each) before declaring done.
```

**Exit criteria:** workspace compiles; all ports have mocks; layering lint blocks Ruv types in
domain/ports; `Gradient` cannot be constructed from a `ThermalFrame` (ADR-0005); canonical_bytes is
deterministic (ADR-0014).

---

## Prompt 1 — Node Isolation & Security (Open Host Service / Shared Kernel)

- **Depends on:** P0.
- **Incorporates:** ADR #4 (RVM coherence in-process, off async path), ADR #8 (CapabilityBroker
  denies `Raw` before the gate), ADR #14 (witness canonical bytes) · DDD #4.
- **SPARC ref:** Part 2 §5, Part 4 R4/R5/R8.

```
Implement the Node Isolation & Security context (hoforras-node) incorporating ADR-0004, ADR-0008,
ADR-0014 and DDD-04. Build the CoherenceSupervisor (anomaly signal → mincut recompute → isolate,
witnessed; in-process and OFF the tokio async path per ADR-0004), the CapabilityBroker (deny
Scope::Raw BEFORE consulting the gate and reject expired capabilities against an injected Clock per
ADR-0008), and the AuditTrail over a 64-byte hash-chained WitnessChain. Provide the RvmCoherenceAdapter,
RvmWitnessAdapter, and RvmCapAdapter over rvm-coherence/-kernel/-witness/-proof/-security/-cap.
Spawn a coordinated swarm team (SendMessage-first): security-architect, coder, tester (security focus),
security-auditor, reviewer — all in one message, run_in_background, wired who-messages-whom. Drive all
five SPARC phases, fully implement, write London interaction tests (raw scope ⇒ gate NEVER invoked;
anomaly ⇒ exactly one recompute then one isolate; benign ⇒ neither; record then verify()==true; tamper
⇒ false) plus the FR-4.4/4.3 property tests, validate, benchmark the isolation path (report measured
partition-switch/mincut/witness-emit latency vs NFR-3 targets — honestly, since these are vendor
benchmarks, ADR-0004), and optimize. Continue until complete via /loop. Then review and ensure ADR-0004,
0008, 0014 are each FULLY implemented before declaring done.
```

**Exit criteria:** the no-raw-egress gate invariant (FR-4.4) and audit completeness/tamper-evidence
(FR-4.3) pass as property tests; RVM ops run in-process; latency reported against NFR-3.

---

## Prompt 2 — Sensing & Ingestion

- **Depends on:** P0, P1 (witness chain / Ed25519).
- **Incorporates:** ADR #14 (canonical bytes for Ed25519), ADR #9 (raw stays node-local) · DDD #2.
- **SPARC ref:** Part 2 §2, Part 4 R6.

```
Implement the Sensing & Ingestion context (hoforras-sensor) incorporating ADR-0014, ADR-0009 and
DDD-02. Build the IngestionPipeline that maps raw multi-source sensor input to a witnessed
ThermalFrame in strict order validate → score → sign → emit, rejecting malformed input at the
boundary and logging invalid readings with a reason. Provide the RvcsiIngestAdapter (anti-corruption
layer translating @ruv/rvcsi CsiFrame/subcarrier vocabulary into the thermal ubiquitous language),
the Ed25519WitnessAdapter (sign canonical_bytes via postcard, ADR-0014), and the TypedEventBus. Keep
RawReading/ThermalFrame node-local — never publish them across a boundary; only ThermalBalance
aggregate is exposed (ADR-0009). Spawn a coordinated swarm team (SendMessage-first): system-architect,
coder, tester, reviewer — one message, run_in_background, wired. Drive all five SPARC phases, fully
implement, write London tests (order Sequence; Invalid ⇒ emit NEVER called + rejection recorded;
emitted frame is signed; malformed rejected before any collaborator call) and the rvcsi/Ed25519
contract tests, validate, benchmark per-frame ingestion throughput, optimize. Continue until complete
via /loop. Then review and ensure ADR-0014 and ADR-0009 are FULLY implemented before declaring done.
```

**Exit criteria:** FR-1.1–1.4 interaction tests green; Ed25519 sign/verify roundtrip contract green;
no raw type crosses a boundary.

---

## Prompt 3 — Forecasting & Anomaly

- **Depends on:** P0, P2 (ReadingWindow).
- **Incorporates:** ADR #12 (edge-only WASM inference), ADR #9 (node-local) · DDD #3.
- **SPARC ref:** Part 2 §3, Part 4 R7.

```
Implement the Forecasting & Anomaly context (hoforras-node) incorporating ADR-0012, ADR-0009 and
DDD-03. Build the ForecastService with the with_agent resource guard that spawns an ephemeral
ruv-swarm specialist, infers, and ALWAYS dissolves (even on error). Implement anomalies()
(typed, confidence-scored Anomaly), burst_precursor() (horizon clamped to [6h,48h]), and demand_24h().
Provide the RuvSwarmAdapter and the NeuroDivergentAdapter hosting ruv-FANN LSTM/N-BEATS as WASM
ON-DEVICE — there must be NO RemoteInferenceClient port; edge-only is enforced by that absence
(ADR-0012). Spawn a coordinated swarm team (SendMessage-first): performance-engineer, coder, tester,
reviewer — one message, run_in_background, wired. Drive all five SPARC phases, fully implement, write
London tests (spawn→infer→dissolve Sequence; dissolve STILL called on infer error; no remote-inference
dependency) plus the WASM inference contract test, validate, benchmark single-building inference and
report against the <100ms NFR-1 budget (on target hardware where available), optimize. Continue until
complete via /loop. Then review and ensure ADR-0012 and ADR-0009 are FULLY implemented before declaring done.
```

**Exit criteria:** spawn==dissolve property holds across all outcomes; no remote-inference seam
exists; inference benchmarked against NFR-1.

---

## Prompt 4 — Thermal Memory

- **Depends on:** P0 (buildable in parallel with P2/P3/P5).
- **Incorporates:** ADR #9 (node-local) · DDD #6.
- **SPARC ref:** Part 2 §7, Part 4 R9.

```
Implement the Thermal Memory context (hoforras-node) incorporating ADR-0009 and DDD-06. Build the
ThermalMemory service: record_state (deterministic 128-dim Embedding of normalized sensor channels +
rolling stats), find_surplus_matches(deficit_embedding, district) with topK + filter, district GNN
inference over the DistrictGraph (buildings=nodes, pipes=weighted edges), and store_witness_for_replay.
Provide the RuVectorAdapter over ruvector (HNSW/DiskANN, GNN). Keep raw history node-local (ADR-0009);
only embeddings/aggregates inform cross-building matching. Spawn a coordinated swarm team
(SendMessage-first): performance-engineer, coder, tester, reviewer — one message, run_in_background,
wired. Drive all five SPARC phases, fully implement, write London tests (upsert is 128-dim; search
passes topK+filter; results ≤ topK satisfy the filter ordered by similarity; GNN delegated) plus the
RuVector contract test, validate, benchmark similarity search against the sub-millisecond NFR-4 target,
optimize. Continue until complete via /loop. Then review and ensure ADR-0009 is FULLY implemented before
declaring done.
```

**Exit criteria:** 128-dim deterministic embeddings; filtered/ordered search (FR-6.3); sub-ms search
benchmarked (NFR-4).

---

## Prompt 5 — Trade Consensus

- **Depends on:** P0 (buildable in parallel with P2/P3/P4).
- **Incorporates:** ADR #11 (post-quantum ML-DSA/ML-KEM-1024), ADR #7 (DAG entries), ADR #14 · DDD #5.
- **SPARC ref:** Part 2 §6, Part 4 R2.

```
Implement the Trade Consensus context (hoforras-mesh) incorporating ADR-0011, ADR-0007, ADR-0014 and
DDD-05. Build the ConsensusGateway (anti-corruption layer over QuDAG): finalize(trade) signs with
ML-DSA THEN broadcasts to QR-Avalanche consensus; resolve_peer(dark) uses Kademlia/.dark discovery
with NO directory port; verify(entry) is tamper-evident. Encode ThermalTradeAgreement as a signed
DagEntry via postcard canonical bytes (ADR-0014). Validate duration ≤ 6h at the boundary (defense in
depth). Provide the QuDagAdapter (ML-DSA, ML-KEM-1024, .dark). Spawn a coordinated swarm team
(SendMessage-first): security-architect, coder, tester, security-auditor, reviewer — one message,
run_in_background, wired. Drive all five SPARC phases, fully implement, write London tests
(sign-before-broadcast Sequence; sign exactly once; resolve uses .dark domain; mutated entry ⇒
verify==false) plus the QuDAG contract test, validate, benchmark finality and report against the <1s
NFR-2 target on a live multi-node mesh where available, optimize. Continue until complete via /loop.
Then review and ensure ADR-0011, 0007, 0014 are FULLY implemented before declaring done.
```

**Exit criteria:** sign-before-broadcast + tamper-evidence (FR-5.1/5.5) green; post-quantum primitives
in use; finality benchmarked (NFR-2).

---

## Prompt 6 — District Mesh Coordination

- **Depends on:** P0, P5 (DAG entries).
- **Incorporates:** ADR #7 (DAG transport over RPC) · DDD #7.
- **SPARC ref:** Part 2 §8, Part 4 R10.

```
Implement the District Mesh Coordination context (hoforras-mesh) incorporating ADR-0007 and DDD-07.
Build the MeshCoordinator: propagate(state) publishes a SIGNED DAG entry via MeshTransport — there
must be NO RPC port (ADR-0007); on_node_event marks nodes down/up and reconfigures/resyncs;
collective_behaviour dispatches Overload→rebalance, CascadeRisk→shed-and-reroute, HeatWave→emergency
routing. Provide the SynapticMeshAdapter (DAG fabric, pub/sub). Spawn a coordinated swarm team
(SendMessage-first): system-architect, coder, tester, reviewer — one message, run_in_background, wired.
Drive all five SPARC phases, fully implement, write London tests (propagate publishes a DAG entry, no
RPC seam; Drop ⇒ mark_down + reconfigure; Up ⇒ mark_up + resync; collective dispatch) plus a churn
property test (after Drop then Up, committed state preserved and the node converges — NFR-9), validate,
benchmark self-heal/resync time, optimize. Continue until complete via /loop. Then review and ensure
ADR-0007 is FULLY implemented before declaring done.
```

**Exit criteria:** no RPC seam exists; self-heal-without-state-loss property holds (NFR-9); collective
behaviours dispatch correctly (FR-7.3).

---

## Prompt 7 — Thermal Market & Brokerage (CORE)

- **Depends on:** P1, P2, P3, P4, P5 (the collaborators it consumes).
- **Incorporates:** ADR #5 (Gradient-only egress), ADR #6 (one MRAP tick per node), ADR #13 (thermal
  credits), ADR #15 (governance hard limits, fail closed) · DDD #1.
- **SPARC ref:** Part 2 §4, Part 4 R1/R3, the AC-7 spine.

```
Implement the CORE Thermal Market & Brokerage context (hoforras-broker) incorporating ADR-0005,
ADR-0006, ADR-0013, ADR-0015 and DDD-01. Build the TradingNode aggregate as a SINGLE-WRITER with one
MRAP tick in flight at a time (ADR-0006). Implement BrokerAgent.tick() = Monitor→Reason→Act→Reflect→
Adapt with STRICT ordering: rules.check BEFORE market.execute, consensus.finalize before execute/route,
and a witness record for {TradeSigned, Exec, Routing}. Implement the pure decide() core and PricingService
(example-tested). Enforce governance as HARD, non-overridable limits (≤500 kWh/day, ≥15% reserve, ≤6
bar, ≤6h) and FAIL CLOSED if the rules engine errors/is unavailable, witnessing the rejection
(ADR-0015). Account trades in kWh-equivalent thermal credits (ADR-0013). Implement FederatedTrainer
whose GradientAggregator accepts ONLY domain::Gradient — no raw frame can reach it (ADR-0005). Provide
DaaRulesAdapter, DaaEconomyAdapter, DaaOrchestratorAdapter, PrimeCoordinatorAdapter, SeedMeshAdapter.
Spawn a coordinated swarm team (SendMessage-first): system-architect, coder, tester, performance-engineer,
security-auditor, reviewer — one message, run_in_background, wired. Drive all five SPARC phases, fully
implement, write the full London interaction suite from sparc.md Part 4 R1 (incl. rule_violation_short_circuits:
execute NEVER called on Violation; rules_checked_before_execute; consensus_before_routing; three witnesses
on execute) plus the FR-3.7/3.8 property tests and the daa-rules governance contract test (R3), validate,
benchmark the tick loop, optimize. Continue until complete via /loop. Then review and ensure ADR-0005,
0006, 0013, 0015 are each FULLY implemented before declaring done.
```

**Exit criteria:** governance-before-execute + fail-closed (FR-3.7) and gradient-only-egress (FR-3.8)
property tests green; single-tick consistency; credits conserved; the full R1 interaction suite green.

---

## Prompt 8 — AI Orchestration (ThermalSense-Bridge + AnomalyExplainer)

- **Depends on:** P7, P3, P2, P5 (the services exposed as tools).
- **Incorporates:** ADR #2 (Open Host Service boundary), ADR #7 (federation signals; deferred) · DDD #8.
- **SPARC ref:** Part 2 §9, Part 4 R11.

```
Implement the AI Orchestration context (hoforras-mesh) incorporating ADR-0002, ADR-0007 and DDD-08.
Build the ThermalBridgeServer MCP server (Open Host Service) registering EXACTLY five tools —
hoforras.thermal.district_status, hoforras.trade.active_agreements, hoforras.pipe.health_scores,
hoforras.anomaly.recent, hoforras.forecast.demand_24h — each delegating to exactly one backing service,
with unknown tools returning a typed error and no delegation. Build the AnomalyExplainer: explain via
ClaudeReasoner THEN push to the OperatorChannel. Provide the RufloMcpAdapter (ruflo/@ruvnet/rvagent)
and WebSocketSink. Stub cross-district federation (mTLS+Ed25519, market-signals-only, no raw data —
deferred per spec R-5). Spawn a coordinated swarm team (SendMessage-first): system-architect, coder,
tester, reviewer — one message, run_in_background, wired. Drive all five SPARC phases, fully implement,
write London tests (boot registers exactly five tools; each delegates to its single service; unknown ⇒
Err with no delegation; explain-then-push Sequence) plus the ruflo MCP contract test, validate, benchmark
tool-invocation latency, optimize. Continue until complete via /loop. Then review and ensure ADR-0002 and
ADR-0007 are FULLY implemented before declaring done.
```

**Exit criteria:** exactly five MCP tools, one-tool-one-service delegation (FR-8.1); explain-then-surface
(FR-8.2); federation stubbed and disclosed.

---

## Prompt 9 — Operator Experience (Dashboard)

- **Depends on:** P6, P8 (the event/MCP shapes it conforms to).
- **Incorporates:** ADR #7 (event shapes), conformist downstream · DDD #9.
- **SPARC ref:** Part 2 §10, Part 4 R12.

```
Implement the Operator Experience context (hoforras-dashboard, TS/React/react-three-fiber)
incorporating ADR-0007 and DDD-09 as a READ-ONLY, conformist-downstream projection layer. Connect to
the ruflo WebSocket (ws://appliance.district-xiii:3001). Render: a 3D force-directed DistrictGraphView
(buildings=nodes, pipes=weighted edges, active trades=animated particle streams); BuildingCardView
(surplus/deficit + 24h sparklines); TradeTimelineView (QuDAG consensus status); PipeHealthHeatmap with
drill-down to the WitnessLogView; AnomalyFeedView with Claude explanations. On trade:executed →
updateEdgeWeight + animateEnergyFlow(pipe_route); on anomaly:detected → highlightNode(warning) +
requestClaudeExplanation. Spawn a coordinated swarm team (SendMessage-first): mobile-dev/frontend,
coder, tester, reviewer — one message, run_in_background, wired. Drive all five SPARC phases, fully
implement, write vitest component tests with a MOCKED socket (trade:executed updates edge once +
animates once; anomaly highlights + requests explanation; pipe drill-down opens witness log), validate,
benchmark render/update on a 20-node graph, optimize. Continue until complete via /loop. Then review and
ensure ADR-0007 (event-shape conformance) is FULLY implemented before declaring done.
```

**Exit criteria:** FR-9.1–9.6 component tests green against a mocked socket; witness-log drill-down works;
dashboard is read-only over published events.

---

## Prompt 10 — Appliance Integration & Completion

- **Depends on:** ALL (P0–P9).
- **Incorporates:** ADR #10 (single tokio runtime wiring), ADR #4/5/6/7/8 (invariants verified live),
  and the full set via the context map · DDD #0 (integration) and DDD #1–#9.
- **SPARC ref:** Part 5 (Completion) in full — integration ladder, hardware benchmarks, AC-1…AC-8.

```
Implement Appliance Integration & Completion incorporating ADR-0010 (single tokio runtime per Appliance)
and verifying ADR-0004, ADR-0005, ADR-0006, ADR-0007, ADR-0008 hold LIVE, using DDD-00 (context map) to
wire all contexts. Retire all mocks: wire every domain unit to its real adapter behind one tokio runtime;
fake only physical hardware/network where unavoidable (SimSensorAdapter for the 30-day backfill). Build the
integration ladder from sparc.md Part 5 §1 (sensor_to_broker, broker_to_consensus, consensus_to_witness,
node_isolation_live, mesh_self_heal, dashboard_live_feed). Run the hardware benchmark suite (Part 5 §2):
NFR-1 <100ms inference, NFR-2 <1s finality, NFR-3 RVM nanoseconds (reported honestly vs vendor figures),
NFR-4 sub-ms search, NFR-5 no-GPU/no-cloud runtime audit, NFR-9 self-heal-without-state-loss. Execute the
District XIII First-Milestone bring-up (Part 5 §3) and pass AC-1…AC-6. Then DEMONSTRATE AC-7 — one fully
autonomous P2P thermal trade (pozsonyi14 surplus → pozsonyi22 deficit, 40 kWh, 6h, 2.3 credits/kWh, routed
through [junction_7, junction_12, junction_18]) with NO human input and a complete verifiable rvm-witness
audit trail — and prove AC-8 (no raw frame crossed any partition boundary). Spawn a coordinated swarm team
(SendMessage-first): hierarchical-coordinator, system-architect, coder, tester, performance-engineer,
security-auditor, production-validator, reviewer — one message, run_in_background, wired. Drive integration
and the Completion phase, fully implement, validate against AC-1…AC-8, benchmark all NFRs, optimize. Continue
until complete via /loop. Then review and ensure EVERY ADR (0001–0015) is fully implemented across the
assembled system — produce the FR→ADR→test→demo traceability closure from sparc.md Part 5 §5/§8 — before
declaring the project done.
```

**Exit criteria:** integration ladder green with mocks retired; all NFR benchmarks measured and gated;
AC-1…AC-6 passed; **AC-7 demonstrated and recorded**; AC-8 (no-raw-egress) proven live; full
FR→ADR→test→demo traceability produced; every ADR 0001–0015 confirmed implemented.

---

## ADR → Prompt coverage matrix

| ADR | Built/owned in | Verified-live in |
|-----|----------------|------------------|
| 0001 SPARC+London TDD | P0 (and every prompt) | P10 |
| 0002 Hexagonal | P0 | P8, P10 |
| 0003 domain/ports crates | P0 | P10 |
| 0004 RVM in-process | P1 | P10 |
| 0005 Gradient type wall | P0 (type) · P7 (use) | P10 |
| 0006 single MRAP tick | P7 | P10 |
| 0007 DAG over RPC | P5, P6 | P9, P10 |
| 0008 deny-raw-first | P1 | P10 |
| 0009 inference/memory node-local | P3, P4 | P10 |
| 0010 single tokio runtime | P0 (skeleton) | P10 (wired) |
| 0011 post-quantum consensus | P5 | P10 |
| 0012 edge-only WASM | P3 | P10 |
| 0013 thermal credits | P7 | P10 |
| 0014 postcard canonical bytes | P0, P2, P5 | P10 |
| 0015 rules fail-closed | P7 | P10 |

Every ADR is implemented in at least one prompt and re-verified live in P10 (Completion).
