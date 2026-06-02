# SPARC Pseudocode — Hőforrás Budapest

**Project:** Hőforrás — Peer-to-Peer Thermal Energy Intelligence System
**Methodology:** SPARC — Reuven Cohen
**Test discipline:** London School TDD (outside-in, mock-driven)
**Document phase:** **P — Pseudocode** (Phase 2 of 5)
**Consumes:** `.plans/sparc-specification.md` (FRs, ports §7.3, contracts §6)
**Produces (for Phase A/R):** algorithm-level designs per unit-under-test, written against ports so each can be driven red-green-refactor.
**Status:** Draft v1.0

---

## 0. Conventions

This document is **language-neutral pseudocode**, not Rust/TypeScript. It is the bridge between *what* (Specification) and *structure* (Architecture).

### 0.1 Notation
- `PORT Name { method(args) -> Return }` — a trait-seam from spec §7.3 (the mock boundary).
- `UNIT Name(deps...)` — a unit-under-test; deps are ports injected at construction (constructor injection → mockable).
- `// london:` — a comment marking an interaction the Refinement-phase test MUST assert (message, order, cardinality, or *absence*).
- `INVARIANT` — a property-test obligation.
- `RESULT<T>` — fallible return (`Ok(T) | Err(e)`); `?` propagates error.
- Errors never panic across a port boundary; they return `Err`.

### 0.2 Universal preconditions (every UNIT)
1. Validate inputs at the boundary (project rule) before any collaborator call.
2. On any error, emit a typed `Err`; never leave a spawned/borrowed resource undissolved (see ForecastService).
3. Keep each unit ≤ 500 lines when realized (project rule); pseudocode below is one unit per block.

### 0.3 Test-first ordering (London, outside-in)
Refinement drives these in this order so collaborators are *discovered* by need:
**Layer 9/8 acceptance → 3 Broker (MRAP) → 1 Ingestion → 2 Inference → 5 Consensus → 4 Isolation → 6 Memory → 7 Mesh → 9 Frontend.**
The Broker is driven first among units because AC-7 (autonomous trade + audit) is the headline scenario and the Broker is its hub.

---

## 1. Shared Ports (consolidated from spec §7.3)

```
PORT SensorSource     { poll() -> RESULT<RawReading> }
PORT Validator        { validate(raw: RawReading) -> ValidationStatus }
PORT QualityScorer    { score(raw: RawReading) -> QualityScore }
PORT WitnessSigner    { sign(bytes: Bytes) -> RESULT<Ed25519Signature> }
PORT EventEmitter     { emit(frame: ThermalFrame) -> RESULT<()> }
PORT RejectionLog     { record(raw: RawReading, reason: Reason) -> () }

PORT SwarmFactory     { spawn(profile: AgentProfile) -> RESULT<InferenceAgent> }
PORT InferenceAgent   { infer(window: ReadingWindow) -> RESULT<Inference>; dissolve() -> () }
PORT ForecastService  { anomalies(w) -> RESULT<Vec<Anomaly>>;
                        burst_precursor(w) -> RESULT<Option<BurstWarning>>;
                        demand_24h(w) -> RESULT<DemandForecast> }

PORT SeedMesh         { read_surplus_deficit() -> RESULT<ThermalBalance> }
PORT MarketGateway    { post_offer(o); post_bid(b); execute(t); route(t, path) -> RESULT<_> }
PORT RulesEngine      { check(t: ProposedTrade) -> RuleVerdict }
PORT TradeEvaluator   { evaluate(outcome: TradeOutcome) -> Reflection }
PORT StrategyStore    { load() -> Strategy; update(a: Adaptation) -> () }
PORT EconomyLedger    { debit_credit(t: ExecutedTrade) -> RESULT<()> }
PORT GradientAggregator{ aggregate(grads: Vec<Gradient>) -> RESULT<Gradient> }   // BFT

PORT MincutEngine     { recompute(graph: NodeGraph) -> Partitioning }
PORT PartitionController { isolate(node: NodeId) -> (); rejoin(node) -> () }
PORT WitnessChain     { emit(action: PrivilegedAction) -> RESULT<WitnessRecord>;
                        verify() -> RESULT<bool> }
PORT CapabilityGate   { authorize(cap: Capability, req: AccessRequest) -> Decision }

PORT MlDsaSigner      { sign(entry: DagEntry) -> RESULT<DagEntry>; verify(e) -> bool }
PORT DagNetwork       { broadcast_and_await_consensus(e: DagEntry) -> RESULT<Finality> }
PORT PeerDiscovery    { resolve(dark_domain: String) -> RESULT<NodeAddr> }       // Kademlia/.dark

PORT VectorIndex      { upsert(id, vec, meta); search(q: Query) -> RESULT<Vec<Candidate>> }
PORT GnnEngine        { infer(graph: DistrictGraph) -> RESULT<GnnState> }

PORT MeshTransport    { publish(entry: DagEntry); subscribe(handler) }
PORT NodeRegistry     { mark_down(n); mark_up(n); live() -> Set<NodeId> }

PORT McpToolRegistry  { register(name, handler); invoke(name, args) -> RESULT<Json> }
PORT ClaudeReasoner   { explain(ctx: AnomalyContext) -> RESULT<Explanation> }
PORT OperatorChannel  { push(event: UiEvent) -> () }                              // WebSocket sink
```

---

## 2. Layer 1 — Ingestion: `UNIT IngestionPipeline` (FR-1.1–1.4)

```
UNIT IngestionPipeline(validator, scorer, signer, emitter, rejections)

FUNCTION ingest(raw: RawReading) -> RESULT<()>:
    boundary_check(raw)                              // reject malformed at boundary

    status = validator.validate(raw)                 // london: called once, first
    IF status == Invalid(reason):
        rejections.record(raw, reason)               // london: record called
        // london: emitter.emit MUST NOT be called   (FR-1.2 absence assertion)
        RETURN Err(Rejected(reason))

    quality = scorer.score(raw)                      // london: called after validate
    frame   = build_frame(raw, quality, status)      // ThermalFrame minus witness

    sig = signer.sign(canonical_bytes(frame))?       // london: sign once per emitted frame (FR-1.3)
    frame.witness = sig

    emitter.emit(frame)?                             // london: emit once, only on valid path
    RETURN Ok(())

// Order contract (FR-1.1): validate -> score -> sign -> emit
INVARIANT: a frame reaches emitter ⇒ it carries a valid Ed25519 signature   (FR-1.3)
INVARIANT: validation==Invalid ⇒ zero emit calls                            (FR-1.2)
```

`build_frame` maps raw multi-source input (temperature, pipe_vibration_hz, fluid_pressure_bar, ground_thermal_gradient) into the canonical `ThermalFrame` (spec §10). `canonical_bytes` is deterministic (postcard/serde) so signatures are reproducible.

---

## 3. Layer 2 — Inference: `UNIT ForecastService` (FR-2.1–2.6)

```
UNIT ForecastServiceImpl(swarm_factory)

// Core resource-safety helper: spawn → use → ALWAYS dissolve  (FR-2.1)
FUNCTION with_agent<T>(profile, body: fn(agent) -> RESULT<T>) -> RESULT<T>:
    agent = swarm_factory.spawn(profile)?            // london: spawn(profile) once
    result = TRY body(agent)                         // capture Ok or Err, do not early-return
    agent.dissolve()                                 // london: dissolve ALWAYS, even on Err (FR-2.1)
    RETURN result
    // london (FR-2.6): no RemoteInferenceClient port exists/used — assert by omission

FUNCTION anomalies(window) -> RESULT<Vec<Anomaly>>:
    profile = AgentProfile{ type: analyst, model: lstm,
                            caps: [anomaly_detection], cognitive:{analytical:0.95, systematic:0.9} }
    RETURN with_agent(profile, λ agent ->
        inf = agent.infer(window)?                   // london: infer(window) once
        Ok(map_to_anomalies(inf)))                   // typed, confidence-scored (FR-2.2)

FUNCTION burst_precursor(window) -> RESULT<Option<BurstWarning>>:
    RETURN with_agent(profile(burst), λ agent ->
        inf = agent.infer(window)?
        IF inf.precursor_score > THRESHOLD:
            Ok(Some(BurstWarning{ horizon: clamp(inf.eta, 6h, 48h), score }))   // FR-2.3
        ELSE Ok(None))

FUNCTION demand_24h(window) -> RESULT<DemandForecast>:
    RETURN with_agent(profile(demand), λ agent ->
        Ok(to_forecast(agent.infer(window)?)))       // 24h per-building (FR-2.4)

INVARIANT: every spawn is paired with exactly one dissolve (success OR failure)   (FR-2.1)
NFR-1: infer path budgeted < 100ms; measured in Phase C, not asserted in unit test
```

---

## 4. Layer 3 — Broker: `UNIT BrokerAgent` (MRAP) (FR-3.1–3.9) — headline unit

```
UNIT BrokerAgent(seed_mesh, forecast, rules, market, evaluator, strategy, ledger, witness, consensus)

FUNCTION tick() -> RESULT<TickReport>:        // one MRAP cycle
    // ---- MONITOR ----
    balance  = seed_mesh.read_surplus_deficit()?     // london: monitor source read once (FR-3.1)
    forecast = forecast.demand_24h(balance.window)?

    // ---- REASON ---- (FR-3.2)
    strat    = strategy.load()
    decision = decide(balance, forecast, strat)      // pure: Offer | Bid | Hold
    IF decision == Hold: RETURN Ok(TickReport::Idle)

    proposed = to_proposed_trade(decision)

    // ---- GOVERNANCE GATE (HARD) ---- (FR-3.7)
    verdict = rules.check(proposed)                  // london: check BEFORE any execute
    IF verdict == Violation(rule):
        witness.emit(RuleRejection(proposed, rule))? // auditable rejection
        // london: market.execute MUST NOT be called (FR-3.7 absence assertion)
        RETURN Ok(TickReport::Rejected(rule))

    // ---- ACT ---- (FR-3.3)
    MATCH decision:
        Offer(o) -> market.post_offer(o)?            // london: post_offer
        Bid(b)   -> market.post_bid(b)?              // london: post_bid

    accepted = await_counterparty(decision)          // returns Option<AcceptedTrade>
    IF accepted is None: RETURN Ok(TickReport::Posted)

    // consensus-finalize the agreement before moving heat (ties to Layer 5)
    final = consensus.finalize(accepted.agreement)?  // london: consensus before execute/route
    witness.emit(TradeSigned(final))?                // FR-4.3 audit

    market.execute(accepted)?                        // london: execute AFTER rules+consensus
    witness.emit(ReadingAccepted_or_Exec(accepted))?
    market.route(accepted, accepted.pipe_route)?     // london: route along pipe_route
    witness.emit(RoutingDecision(accepted.pipe_route))?
    ledger.debit_credit(accepted)?                   // kWh-equiv thermal credits (FR-3.6)

    // ---- REFLECT ---- (FR-3.4)
    outcome    = observe_outcome(accepted)
    reflection = evaluator.evaluate(outcome)         // london: evaluate(outcome)

    // ---- ADAPT ---- (FR-3.5)
    strategy.update(to_adaptation(reflection))       // london: strategy.update
    RETURN Ok(TickReport::Executed(final, reflection))

// Pure decision core — no I/O, trivially unit-testable by example
FUNCTION decide(balance, forecast, strat) -> Decision:
    net = balance.surplus_kwh - projected_need(forecast, strat.reserve_pct)
    IF net > strat.offer_threshold:  RETURN Offer{ kwh: net, price: price_offer(strat, forecast),
                                                   duration: min(strat.window_h, 6) }
    IF net < -strat.bid_threshold:   RETURN Bid{ kwh: -net, max_price: price_bid(strat, forecast),
                                                 duration: min(strat.window_h, 6) }
    RETURN Hold

// Strict ordering contract (the spine of AC-7):
//   monitor → reason → rules.check → [act] → consensus → execute → route → witness* → reflect → adapt
INVARIANT (FR-3.7): rules.check precedes market.execute on EVERY executed trade
INVARIANT (FR-3.7): any Violation ⇒ zero execute calls
INVARIANT (AC-7):   every executed trade produces a witness record for sign+exec+route
```

### 4.1 `UNIT FederatedTrainer` (FR-3.8)

```
UNIT FederatedTrainer(local_grad_source, aggregator, witness)

FUNCTION round() -> RESULT<Gradient>:
    local = local_grad_source.compute()?             // derived from local frames only
    // london: only gradients are passed to aggregator — never raw ThermalFrame
    agg = aggregator.aggregate(collect_peer_gradients_plus(local))?   // BFT (daa-prime-coordinator)
    witness.emit(GradientAggregated(agg.digest))?
    RETURN Ok(agg)

INVARIANT (FR-3.8 / NFR-7): no RawReading or ThermalFrame value is ever an argument to aggregator
    → enforced by type: aggregator accepts ONLY Gradient (property/compile-time check)
```

---

## 5. Layer 4 — Isolation & Security

### 5.1 `UNIT CoherenceSupervisor` (FR-4.2, FR-4.5)

```
UNIT CoherenceSupervisor(anomaly_source, mincut, partitions, witness)

FUNCTION on_signal(sig: AnomalySignal) -> RESULT<()>:
    IF NOT sig.is_anomalous(): RETURN Ok(())         // ignore benign

    graph = current_node_graph()
    parting = mincut.recompute(graph)                // london: recompute(graph) on anomaly (FR-4.2)
    offending = parting.isolated_side(sig.node)
    partitions.isolate(offending)                    // london: isolate(node), no restart/manual step
    witness.emit(NodeIsolated(offending, sig.reason))?
    RETURN Ok(())                                    // other nodes keep serving (continuity)

INVARIANT (FR-4.2): anomalous signal ⇒ exactly one recompute then one isolate; benign ⇒ neither
NFR-3: recompute/isolate latency targets (~331ns/~6ns) verified in Phase C bench, not unit test
```

### 5.2 `UNIT CapabilityBroker` (FR-4.4) — the no-raw-egress gate

```
UNIT CapabilityBroker(gate, witness)

FUNCTION request_cross_partition(req: AccessRequest, cap: Capability) -> RESULT<Payload>:
    // Hard scope rule BEFORE delegating to gate:
    IF req.scope == Raw:                             // raw never crosses a boundary
        RETURN Err(Denied(RawScopeForbidden))        // london: gate may not even be consulted; deny
    IF cap.expired(now_injected):                    // expiry honored (clock injected for determinism)
        RETURN Err(Denied(Expired))

    decision = gate.authorize(cap, req)              // london: authorize(cap, req)
    IF decision == Allow AND req.scope == AggregatedThermalAvailability:
        witness.emit(CrossPartitionRead(req))?
        RETURN Ok(fetch_aggregate(req))
    RETURN Err(Denied(decision.reason))

INVARIANT (FR-4.4 / NFR-7): scope==Raw ⇒ always Err, regardless of capability
INVARIANT: only AggregatedThermalAvailability with valid, unexpired cap returns Ok
```

### 5.3 `UNIT AuditTrail` (FR-4.3)

```
UNIT AuditTrail(chain)

FUNCTION record(action: PrivilegedAction) -> RESULT<WitnessRecord>:
    rec = chain.emit(action)?                        // 64-byte hash-chained record
    RETURN Ok(rec)

FUNCTION verify_integrity() -> RESULT<bool>:
    RETURN chain.verify()                            // recompute hash links

INVARIANT (FR-4.3): record(a) then verify_integrity() == true; tamper ⇒ false
```

---

## 6. Layer 5 — Consensus: `UNIT ConsensusGateway` (FR-5.1–5.5)

```
UNIT ConsensusGateway(signer, network, discovery)

FUNCTION finalize(trade: ThermalTradeAgreement) -> RESULT<Finality>:
    boundary_check(trade)                            // duration ≤ 6h etc. (defense-in-depth vs rules)
    entry  = DagEntry::new(trade)
    signed = signer.sign(entry)?                     // london: ML-DSA sign once (FR-5.1/5.2)
    final  = network.broadcast_and_await_consensus(signed)?   // london: broadcast once (FR-5.3)
    RETURN Ok(final)                                 // QR-Avalanche sub-second (NFR-2)

FUNCTION resolve_peer(dark: String) -> RESULT<NodeAddr>:
    // e.g. "pozsonyi22.thermal.budapest.dark"
    RETURN discovery.resolve(dark)                   // london: Kademlia/.dark, no directory (FR-5.4)

FUNCTION verify(entry: DagEntry) -> bool:
    RETURN signer.verify(entry)                      // tamper-evident (FR-5.5)

INVARIANT (FR-5.5): mutate(entry) ⇒ verify == false
INVARIANT (FR-5.1): finalize signs before broadcasting (order)
NFR-2: finality < 1s — integration/Phase C assertion
```

---

## 7. Layer 6 — Memory: `UNIT ThermalMemory` (FR-6.1–6.5)

```
UNIT ThermalMemory(index, gnn)

FUNCTION record_state(node, frame: ThermalFrame) -> RESULT<()>:
    index.upsert(key(node, frame.timestamp_ns), embed(frame), meta(node, frame))   // FR-6.1/6.2
    RETURN Ok(())

FUNCTION find_surplus_matches(deficit_embedding, district) -> RESULT<Vec<Candidate>>:
    q = Query{ vector: deficit_embedding, topK: 5,
               filter: { thermal_surplus_available: true, district } }
    RETURN index.search(q)                           // london: search(q); HNSW sub-ms (FR-6.3/NFR-4)

FUNCTION district_inference(graph: DistrictGraph) -> RESULT<GnnState>:
    RETURN gnn.infer(graph)                          // buildings=nodes, pipes=weighted edges (FR-6.4)

FUNCTION store_witness_for_replay(rec: WitnessRecord) -> RESULT<()>:
    index.upsert(witness_key(rec), embed_witness(rec), {kind: witness})   // FR-6.5
    RETURN Ok(())

// embed(frame): deterministic 128-dim feature map (dimensionality fixed by spec FR-6.2;
//               feature scheme is an open question Q-2 resolved here as: normalized
//               [temp, vibration, pressure, gradient, rolling-stats…] padded/projected to 128)
INVARIANT (FR-6.3): search returns ≤ topK candidates, all satisfying filter, ordered by similarity
```

---

## 8. Layer 7 — Swarm: `UNIT MeshCoordinator` (FR-7.1–7.4)

```
UNIT MeshCoordinator(transport, registry)

FUNCTION propagate(state: MarketOrAnomalyOrConsensus) -> RESULT<()>:
    entry = sign_dag_entry(state)
    transport.publish(entry)                         // london: publish (DAG, not RPC) (FR-7.1)
    RETURN Ok(())

FUNCTION on_node_event(ev: NodeEvent):
    MATCH ev:
        Drop(n) -> registry.mark_down(n)             // london: mark_down
                   reconfigure(registry.live())      // self-heal without losing committed state (FR-7.2)
        Up(n)   -> registry.mark_up(n)               // london: mark_up
                   resync(n)                          // resume; replay missed DAG entries

FUNCTION collective_behaviour(signal) -> Action:     // FR-7.3
    MATCH signal:
        Overload(zone)   -> rebalance_load(zone)
        CascadeRisk(path)-> shed_and_reroute(path)
        HeatWave         -> emergency_routing_plan()

INVARIANT (FR-7.2): after Drop(n)+Up(n), committed market state at survivors is unchanged
                    and n converges to the same state (eventual consistency via DAG)
INVARIANT (FR-7.1): cross-node propagation occurs via transport.publish, never a direct RPC port
```

---

## 9. Layer 8 — AI Orchestration

### 9.1 `UNIT ThermalBridgeServer` (FR-8.1) — MCP tools

```
UNIT ThermalBridgeServer(registry, status_svc, trade_svc, pipe_svc, anomaly_svc, forecast_svc)

FUNCTION boot():
    registry.register("hoforras.thermal.district_status", λ args -> status_svc.snapshot())
    registry.register("hoforras.trade.active_agreements",  λ args -> trade_svc.active())
    registry.register("hoforras.pipe.health_scores",       λ args -> pipe_svc.scores())
    registry.register("hoforras.anomaly.recent",           λ args -> anomaly_svc.recent(args.since))
    registry.register("hoforras.forecast.demand_24h",      λ args -> forecast_svc.demand(args.node))

FUNCTION invoke(name, args) -> RESULT<Json>:
    IF NOT registry.has(name): RETURN Err(UnknownTool(name))   // london: typed error, no delegation
    RETURN registry.invoke(name, args)              // london: delegates to EXACTLY its backing svc

INVARIANT (FR-8.1): each registered tool delegates to exactly one backing service; unknown→Err
```

### 9.2 `UNIT AnomalyExplainer` (FR-8.2) & federation note (FR-8.3)

```
UNIT AnomalyExplainer(claude, operator)

FUNCTION explain_and_surface(ctx: AnomalyContext) -> RESULT<()>:
    exp = claude.explain(ctx)?                       // london: explain(ctx) once (via ruflo/daa-ai)
    operator.push(UiEvent::AnomalyExplained(ctx.node, exp))   // london: pushed to operator
    RETURN Ok(())

// FR-8.3 federation: cross-district uses mTLS + Ed25519 auth and exchanges ONLY market signals
// (same no-raw-egress INVARIANT as FR-3.8). Pilot: stubbed behind a Federation port; full
// acceptance deferred per spec §11 R-5.
```

---

## 10. Layer 9 — Frontend (React + react-three-fiber)

```
COMPONENT DistrictDashboard(thermalMesh: OperatorChannel-feed)

ON mount:
    socket = connect("ws://appliance.district-xiii:3001")     // ruflo WebSocket (FR-9.2)
    socket.on("trade:executed", onTradeExecuted)
    socket.on("anomaly:detected", onAnomalyDetected)

FUNCTION onTradeExecuted(agreement):                 // FR-9.1/9.2
    updateEdgeWeight(agreement.seller, agreement.buyer, agreement.kwh_offered)
    animateEnergyFlow(agreement.pipe_route)          // particle stream along route

FUNCTION onAnomalyDetected(event):                   // FR-9.6
    highlightNode(event.node_id, "warning")
    requestClaudeExplanation(event)                  // → AnomalyExplainer result rendered in feed

RENDER:
    <Canvas>  ForceDirectedGraph(nodes=buildings, edges=pipes-weighted, particles=active-trades) </Canvas>
    BuildingCards(surplus/deficit + 24h forecast sparklines)        // FR-9.3
    TradeTimeline(active agreements + QuDAG consensus status)        // FR-9.4
    PipeHealthHeatmap(onDrillDown -> RvmWitnessLogView)              // FR-9.5
    AnomalyFeed(Claude natural-language explanations)               // FR-9.6

// Frontend units are tested with a mocked socket (OperatorChannel double): assert that a
// "trade:executed" message triggers updateEdgeWeight + animateEnergyFlow exactly once.  (London)
```

---

## 11. Acceptance-Level Pseudocode (outside-in driver for AC-7)

The headline scenario, expressed as the outermost test the whole stack is driven from:

```
ACCEPTANCE AC-7 "autonomous trade with complete audit":
    GIVEN building A in surplus, building B in deficit, full stack wired
          (hardware + network behind in-memory fakes; crypto/consensus real-or-contract-faked)
    WHEN  A.broker.tick() runs to completion with no operator input
    THEN  rules.check was called before market.execute                       (FR-3.7)
    AND   consensus.finalize produced finality < 1s                          (NFR-2)
    AND   market.route was called with the agreement's pipe_route            (FR-3.3)
    AND   witness chain contains records for {TradeSigned, Exec, Routing}    (FR-4.3)
    AND   AuditTrail.verify_integrity() == true                              (FR-4.3)
    AND   no RawReading/ThermalFrame crossed any partition boundary          (AC-8/FR-4.4/3.8)
```

This drives, in order: `BrokerAgent` → `ConsensusGateway` → `MarketGateway` → `AuditTrail`/`WitnessChain`, with `CapabilityBroker` asserting the no-raw-egress invariant throughout.

---

## 12. Algorithmic Complexity Notes (for Architecture/Refinement)

| Unit | Hot path | Target complexity | Note |
|------|----------|-------------------|------|
| IngestionPipeline | per-frame | O(1) | streaming; no buffering of history |
| ForecastService | per-window infer | bounded by model | WASM, <100ms (NFR-1) |
| BrokerAgent.decide | per-tick | O(1) on balance+forecast summary | pure, no I/O |
| CoherenceSupervisor | on anomaly | mincut O(V·E) | only on signal, not steady-state (FR-4.5) |
| ThermalMemory.search | per-query | ~O(log N) HNSW | sub-ms (NFR-4) |
| MeshCoordinator.resync | on node-up | O(missed entries) | DAG replay |
| ConsensusGateway.finalize | per-trade | network-bound | QR-Avalanche sub-second |

---

## 13. Open Questions Resolved in This Phase

| From spec §11 | Resolution here |
|----------------|------------------|
| Q-1 pricing strategy (FR-3.2) | `decide()` core (§4): offer/bid thresholds + `price_offer/price_bid` from `Strategy`; strategy mutated by Adapt. Fully testable via `StrategyStore` mock. |
| Q-2 GNN embedding / 128-dim (FR-6.2) | `embed(frame)` (§7): normalized sensor channels + rolling stats projected to 128 dims; deterministic. |
| Q-3 anomaly taxonomy (FR-2.2) | `map_to_anomalies` returns typed, confidence-scored `Anomaly` variants {ThermalSpike, PressureDrop, VibrationPattern, BurstPrecursor}. |

Deferred (unchanged): R-1 (crate drift → ports), R-2 (routing modeled), R-3 (RVM ns benchmarks → Phase C), R-5 (federation stubbed).

---

## 14. Exit Criteria for the Pseudocode Phase

1. Every FR unit in spec §6 has algorithm-level pseudocode written against ports. ✅
2. Every `// london:` interaction is explicit and asserts message/order/cardinality/absence. ✅
3. Safety/security invariants (FR-3.7 gate ordering, FR-4.4 no-raw scope, FR-3.8 gradient-only, FR-4.3 audit) are expressed as INVARIANTs ready for property tests. ✅
4. The headline AC-7 scenario has outside-in acceptance pseudocode threading the units. ✅
5. Spec open questions Q-1…Q-3 resolved at algorithm level. ✅

**Next phase:** A — Architecture. Bind these units to the Cargo workspace crates (`hoforras-sensor`, `hoforras-broker`, `hoforras-node`, `hoforras-mesh`) + the TS dashboard, map each PORT to a concrete Ruv-crate adapter, and define deployment topology for the District XIII pilot.
