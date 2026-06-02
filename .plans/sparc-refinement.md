# SPARC Refinement — Hőforrás Budapest

**Project:** Hőforrás — Peer-to-Peer Thermal Energy Intelligence System
**Methodology:** SPARC — Reuven Cohen
**Test discipline:** London School TDD (outside-in, mock-driven, interaction-based)
**Document phase:** **R — Refinement** (Phase 4 of 5)
**Consumes:** `.plans/sparc-specification.md`, `.plans/sparc-pseudocode.md`, `.plans/sparc-architecture.md`
**Produces (for Phase C):** the red-green-refactor test/build plan, per-unit test schedules, contract-test matrix, property tests, and the iteration log structure that proves every FR.
**Status:** Draft v1.0

---

## 0. What Refinement is (and is not)

Refinement is where the design from S/P/A is **driven into existence test-first**. In the London school this is strictly **outside-in**: we start at the operator-visible acceptance test (AC-7), let it fail because a collaborator is missing, and *discover* each unit and each port by the need the failing test expresses. Mocks stand in for not-yet-built neighbours; they are replaced by real adapters only at the contract-test boundary.

This document is the **execution plan and test specification** for that process — not the production code. Each section below is a red-green-refactor schedule with the exact interaction assertions (lifted from the `// london:` markers in Pseudocode) that make a test "green for the right reason."

### 0.1 The three-beat cycle, applied here
- **RED** — write the test; assert the *interaction* (message, order, cardinality, or absence). It fails to compile or fails the expectation.
- **GREEN** — write the minimum domain code to satisfy the expectation. No adapter yet — collaborators are `mockall` doubles.
- **REFACTOR** — extract pure cores (e.g. `decide()`), enforce ≤ 500-line modules, remove duplication. Tests stay green.

### 0.2 Tooling (from Architecture §12)
- `mockall` — auto-mocks for every port trait in `hoforras-ports`.
- `tokio::test` — async unit tests.
- `proptest` — the safety/security invariants (FR-3.7, FR-4.4, FR-3.8, FR-4.3).
- In-memory fakes (hand-written) — acceptance wiring only.
- `wasm-bindgen-test` — inference adapter contract tests.
- `vitest` + mocked socket — dashboard component tests.

---

## 1. Outside-in execution order

We do **not** build bottom-up. We build in the order the failing acceptance test demands. Each step's RED is created by the previous step's GREEN exposing a missing collaborator.

```
R0  Acceptance harness AC-7 (all collaborators mocked/faked)   ──┐ drives ↓
R1  BrokerAgent (MRAP spine)                                     │
R2  ConsensusGateway        (BrokerAgent needs finality)         │
R3  RulesEngine usage + DaaRules contract                        │
R4  AuditTrail / WitnessChain (every privileged action)          │
R5  CapabilityBroker (no-raw-egress gate)                        │
R6  IngestionPipeline (SeedMesh needs frames)                    │
R7  ForecastService (Reason needs demand_24h)                    │
R8  CoherenceSupervisor (anomaly → isolate)                      │
R9  ThermalMemory                                                │
R10 MeshCoordinator                                              │
R11 ThermalBridgeServer + AnomalyExplainer                       │
R12 Dashboard components                                         │
R13 Adapter contract tests (each port ↔ real Ruv crate)        ──┘ then ↑ swap mocks → adapters
R14 Property/invariant suite (cross-cutting)
R15 Acceptance AC-1..AC-6, AC-8 go green end-to-end
```

The headline (R0→R1→R2→R4→R5) is sequenced first because AC-7 — "autonomous trade with complete audit, no raw egress" — is the riskiest, highest-value path (spec §9). Everything else hangs off it.

---

## 2. R0 — Acceptance harness (the outermost RED)

**Test:** `tests/acceptance/ac7_autonomous_trade.rs`

```
GIVEN  a TestDistrict with building A (surplus) and B (deficit)
       wired from in-memory fakes: FakeSeedMesh, FakeMarket, FakeConsensus,
       SpyWitnessChain, FakeCapabilityGate, FixedClock
WHEN   district.building("A").broker.tick().await
THEN   (assertions below)
```

**Interaction assertions (the AC-7 contract, pseudocode §11):**
1. `rules.check` called **before** `market.execute`  → order spy records `[check, …, execute]`.
2. On a passing trade, `consensus.finalize` returns within budget → `FakeConsensus` stamps `< 1s` (logical clock).
3. `market.route` called with exactly `agreement.pipe_route`.
4. `SpyWitnessChain` contains records for `{TradeSigned, Exec, Routing}` — cardinality ≥ 3, correct order.
5. `AuditTrail.verify_integrity() == true`.
6. **No** `RawReading`/`ThermalFrame` value ever passed across the capability/mesh fakes (AC-8) — fakes assert the type wall.

**RED state:** fails to compile — `BrokerAgent` doesn't exist. This *is* the signal to start R1. The harness stays red until R1–R5 are green; that is expected and correct in London TDD.

---

## 3. R1 — `BrokerAgent` (MRAP spine) — the keystone unit

**Module:** `broker/src/agent.rs` + pure `broker/src/decide.rs`
**Mocks:** all nine ports via `mockall` (`MockSeedMesh`, `MockForecastService`, `MockRulesEngine`, `MockMarketGateway`, `MockTradeEvaluator`, `MockStrategyStore`, `MockEconomyLedger`, `MockWitnessChain`, `MockConsensusGateway`).

### Test schedule
| # | Test (RED→GREEN) | Asserts | FR |
|---|------------------|---------|----|
| 1 | `tick_monitors_then_reasons` | `seed_mesh.read_surplus_deficit` called once, then `forecast.demand_24h` | 3.1 |
| 2 | `hold_when_balanced_posts_nothing` | `decide()==Hold` ⇒ `post_offer`/`post_bid`/`execute` **never** called | 3.2 |
| 3 | `surplus_posts_offer` | net>threshold ⇒ `post_offer` once with kwh=net, duration≤6 | 3.2/3.3 |
| 4 | `deficit_posts_bid` | net<−threshold ⇒ `post_bid` once | 3.2 |
| 5 | **`rule_violation_short_circuits`** | `rules.check` returns Violation ⇒ `execute` **never** called; one `RuleRejection` witness | **3.7** |
| 6 | `rules_checked_before_execute` | sequence: `check` index < `execute` index (mockall `Sequence`) | **3.7** |
| 7 | `consensus_before_routing` | `consensus.finalize` index < `execute` < `route` | 3.3/5.1 |
| 8 | `executed_trade_emits_three_witnesses` | witness emitted for TradeSigned, Exec, Routing | 4.3 |
| 9 | `credits_accounted_on_execute` | `ledger.debit_credit` called once on success | 3.6 |
| 10 | `reflect_then_adapt` | `evaluator.evaluate` then `strategy.update` (Sequence) | 3.4/3.5 |
| 11 | `no_counterparty_posts_only` | accepted=None ⇒ `execute`/`route` not called, returns Posted | 3.3 |

### Pure-core tests (`decide.rs`, no mocks — example-based)
`decide(balance, forecast, strat)` → table tests over {surplus, deficit, balanced} × edge thresholds. This is the one place state-based testing is used, because the core is pure (Architecture §4.3).

**REFACTOR:** extract `decide`, `price_offer`, `price_bid` as pure fns; keep `agent.rs` orchestration ≤ 500 lines.

**GREEN exposes:** `ConsensusGateway` (test 7) and `WitnessChain`/`AuditTrail` (test 8) as missing → triggers R2, R4.

---

## 4. R2 — `ConsensusGateway` (FR-5)

**Module:** `mesh/src/consensus.rs`. **Mocks:** `MockMlDsaSigner`, `MockDagNetwork`, `MockPeerDiscovery`.

| # | Test | Asserts | FR |
|---|------|---------|----|
| 1 | `finalize_signs_then_broadcasts` | `signer.sign` index < `network.broadcast…` index | 5.1 |
| 2 | `sign_uses_ml_dsa_once` | `sign` called exactly once per finalize | 5.1/5.2 |
| 3 | `resolve_peer_uses_dark_domain` | `discovery.resolve("pozsonyi22.thermal.budapest.dark")` once; no directory port exists | 5.4 |
| 4 | `verify_rejects_mutated_entry` | mutate(entry) ⇒ `verify==false` | 5.5 |
| 5 | `boundary_rejects_overlong_duration` | duration>6h ⇒ Err before sign (defense in depth) | 5.1/3.7 |

NFR-2 (`<1s`) is **not** asserted here (mocked network) — deferred to R13 contract test + Phase C bench.

---

## 5. R3 — `RulesEngine` interaction & `DaaRulesAdapter` contract (FR-3.7)

Two layers, kept separate (Architecture §3):

**Unit (mocked, already covered by R1 #5/#6)** — verifies the *broker* consults rules correctly.

**Contract (`--features contract`)** — `DaaRulesAdapter` against real `daa-rules 0.2.1`:
| # | Test | Asserts |
|---|------|---------|
| 1 | `rejects_over_500kwh_daily` | proposed daily total > 500.0 ⇒ Violation(max_daily_thermal_transfer_kwh) |
| 2 | `rejects_below_15pct_reserve` | reserve < 0.15 ⇒ Violation |
| 3 | `rejects_over_6bar` | pressure > 6.0 ⇒ Violation |
| 4 | `rejects_over_6h_window` | window > 6 ⇒ Violation |
| 5 | `accepts_within_all_limits` | compliant trade ⇒ Allow |

This is the literal encoding of research §78–82 governance rules.

---

## 6. R4 — `AuditTrail` / `WitnessChain` (FR-4.3)

**Module:** `node/src/audit.rs`. **Mock:** `MockWitnessChain`; **Contract:** `RvmWitnessAdapter` vs `rvm-witness`.

| # | Level | Test | Asserts |
|---|-------|------|---------|
| 1 | unit | `record_delegates_to_chain` | `chain.emit(action)` once, returns record |
| 2 | unit | `verify_delegates` | `verify_integrity` calls `chain.verify` |
| 3 | contract | `record_then_verify_true` | emit N actions ⇒ `verify()==true` |
| 4 | contract | `tampered_chain_fails_verify` | flip a byte ⇒ `verify()==false` |
| 5 | contract | `record_is_64_bytes` | witness record length == 64 |

---

## 7. R5 — `CapabilityBroker` (FR-4.4 / NFR-7) — the headline invariant unit

**Module:** `node/src/capability.rs`. **Mocks:** `MockCapabilityGate`, `MockWitnessChain`, `FixedClock`.

| # | Test | Asserts | FR |
|---|------|---------|----|
| 1 | **`raw_scope_denied_without_consulting_gate`** | `req.scope==Raw` ⇒ Err(RawScopeForbidden); `gate.authorize` **never** called | **4.4** (ADR-6) |
| 2 | `expired_capability_denied` | `cap.expired(clock)` ⇒ Err(Expired); gate not consulted | 4.4 |
| 3 | `aggregate_allow_returns_payload` | Allow + AggregatedThermalAvailability ⇒ Ok + one `CrossPartitionRead` witness | 4.4 |
| 4 | `gate_deny_returns_err` | gate Deny ⇒ Err, no payload | 4.4 |

Property test (R14) generalizes #1 over arbitrary capabilities.

---

## 8. R6 — `IngestionPipeline` (FR-1)

**Module:** `sensor/src/pipeline.rs`. **Mocks:** `MockValidator`, `MockQualityScorer`, `MockWitnessSigner`, `MockEventEmitter`, `MockRejectionLog`.

| # | Test | Asserts | FR |
|---|------|---------|----|
| 1 | `valid_path_order` | Sequence: `validate` → `score` → `sign` → `emit` | 1.1 |
| 2 | **`invalid_not_emitted`** | `validate`→Invalid ⇒ `emit` **never** called; `rejections.record` once | **1.2** |
| 3 | `emitted_frame_is_signed` | `sign` called once; frame.witness set | 1.3 |
| 4 | `frame_carries_quality_and_status` | emitted frame has QualityScore + ValidationStatus | 1.4 |
| 5 | `malformed_rejected_at_boundary` | boundary_check fails ⇒ Err before any collaborator call | NFR-12 |

**Contract:** `RvcsiIngestAdapter` vs `@ruv/rvcsi` (CsiFrame→ThermalFrame mapping); `Ed25519WitnessAdapter` vs `ed25519-dalek` (sign/verify roundtrip).

---

## 9. R7 — `ForecastService` (FR-2)

**Module:** `node/src/forecast.rs`. **Mocks:** `MockSwarmFactory`, `MockInferenceAgent`.

| # | Test | Asserts | FR |
|---|------|---------|----|
| 1 | `spawns_infers_dissolves` | Sequence: `spawn` → `infer` → `dissolve` | 2.1 |
| 2 | **`dissolve_on_infer_error`** | `infer` returns Err ⇒ `dissolve` **still** called once | **2.1** |
| 3 | `anomalies_typed_scored` | returns typed Anomaly with confidence | 2.2 |
| 4 | `burst_warning_clamped_6_48h` | precursor>threshold ⇒ horizon ∈ [6h,48h] | 2.3 |
| 5 | `demand_24h_shape` | returns 24h forecast | 2.4 |
| 6 | **`no_remote_inference_dependency`** | compile-time: no `RemoteInferenceClient` in deps; runtime: only swarm ports touched | **2.6** |

**Contract:** `NeuroDivergentAdapter` runs an LSTM under `wasm-bindgen-test`; assert inference completes (latency measured but NFR-1 `<100ms` confirmed in Phase C on Appliance hardware).

---

## 10. R8 — `CoherenceSupervisor` (FR-4.2)

**Mocks:** `MockAnomalySource`(signal), `MockMincutEngine`, `MockPartitionController`, `MockWitnessChain`.

| # | Test | Asserts | FR |
|---|------|---------|----|
| 1 | `benign_signal_noop` | not anomalous ⇒ `recompute`/`isolate` **never** called | 4.2 |
| 2 | `anomaly_recomputes_then_isolates` | Sequence: `recompute(graph)` → `isolate(node)`; once each | 4.2 |
| 3 | `isolation_is_witnessed` | one `NodeIsolated` witness | 4.2/4.3 |
| 4 | `other_nodes_keep_serving` | supervisor returns Ok; no global halt | 4.2 |

**Contract:** `RvmCoherenceAdapter` vs `rvm-coherence` mincut. NFR-3 nanosecond latencies → Phase C bench, logged not asserted.

---

## 11. R9 — `ThermalMemory` (FR-6)

**Mocks:** `MockVectorIndex`, `MockGnnEngine`.

| # | Test | Asserts | FR |
|---|------|---------|----|
| 1 | `record_state_upserts_128dim` | `upsert` with 128-dim vector | 6.1/6.2 |
| 2 | `search_passes_topk_and_filter` | `search` query has topK=5, filter{surplus, district} | 6.3 |
| 3 | `search_results_respect_filter` | all candidates satisfy filter, ≤ topK, ordered | 6.3 |
| 4 | `gnn_infer_over_district_graph` | `gnn.infer(graph)` delegated | 6.4 |
| 5 | `witness_stored_for_replay` | witness upserted with kind=witness | 6.5 |

**Contract:** `RuVectorAdapter` vs `ruvector` HNSW — sub-ms search confirmed Phase C.

---

## 12. R10 — `MeshCoordinator` (FR-7)

**Mocks:** `MockMeshTransport`, `MockNodeRegistry`.

| # | Test | Asserts | FR |
|---|------|---------|----|
| 1 | `propagate_publishes_dag_entry` | `transport.publish` called; **no** RPC port exists | 7.1 |
| 2 | `node_drop_marks_down_and_reconfigures` | `mark_down(n)` then reconfigure over `live()` | 7.2 |
| 3 | `node_up_marks_up_and_resyncs` | `mark_up(n)` then resync | 7.2 |
| 4 | `collective_behaviour_dispatch` | Overload→rebalance, CascadeRisk→reroute, HeatWave→emergency | 7.3 |

**Contract:** `SynapticMeshAdapter` — drop/return → committed state preserved (NFR-9), property-style.

---

## 13. R11 — `ThermalBridgeServer` + `AnomalyExplainer` (FR-8)

**Bridge mocks:** `MockMcpToolRegistry` + 5 mock backing services.
| # | Test | Asserts | FR |
|---|------|---------|----|
| 1 | `boot_registers_five_tools` | exactly the 5 named tools registered | 8.1 |
| 2 | `each_tool_delegates_to_its_service` | `district_status`→status_svc, … (one each) | 8.1 |
| 3 | `unknown_tool_errors` | unknown name ⇒ Err(UnknownTool); no delegation | 8.1 |

**Explainer mocks:** `MockClaudeReasoner`, `MockOperatorChannel`.
| # | Test | Asserts | FR |
|---|------|---------|----|
| 4 | `explain_then_push` | `claude.explain(ctx)` once, then `operator.push(AnomalyExplained)` | 8.2 |

**Contract:** `RufloMcpAdapter` registers/invokes against real ruflo MCP; federation (FR-8.3) stub test only (spec R-5 deferred).

---

## 14. R12 — Dashboard components (FR-9)

**Tooling:** `vitest` + mocked WebSocket (the `OperatorChannel`-feed double).
| # | Test | Asserts | FR |
|---|------|---------|----|
| 1 | `trade_executed_updates_edge_and_animates` | message ⇒ `updateEdgeWeight` once + `animateEnergyFlow(pipe_route)` once | 9.1/9.2 |
| 2 | `anomaly_detected_highlights_and_requests_explanation` | ⇒ `highlightNode(warning)` + `requestClaudeExplanation` | 9.6 |
| 3 | `building_cards_render_sparklines` | cards show surplus/deficit + 24h sparkline | 9.3 |
| 4 | `trade_timeline_shows_consensus_status` | timeline reflects QuDAG status | 9.4 |
| 5 | `pipe_heatmap_drilldown_opens_witness_log` | drill-down → WitnessLogView | 9.5 |

---

## 15. R13 — Adapter contract-test matrix (swap mocks → real Ruv crates)

Run under `--features contract`. Each row replaces a mock from R1–R12 with its real adapter and proves the port contract holds against the actual crate (Architecture §3 table).

| Adapter | Ruv crate | Pin | Contract focus |
|---------|-----------|-----|----------------|
| RvcsiIngestAdapter | @ruv/rvcsi | latest | CsiFrame→ThermalFrame, ValidationStatus/QualityScore |
| Ed25519WitnessAdapter | ed25519-dalek | 2 | sign/verify roundtrip; tamper fails |
| RuvSwarmAdapter | ruv-swarm | 1.0.5 | spawn/infer/dissolve lifecycle |
| NeuroDivergentAdapter | ruv-FANN | — | LSTM/N-BEATS WASM inference |
| DaaRulesAdapter | daa-rules | 0.2.1 | 4 governance limits (R3) |
| DaaEconomyAdapter | daa-economy | 0.2.1 | offer/bid/execute/route + credit ledger |
| DaaOrchestratorAdapter | daa-orchestrator/-ai | 0.2.1 | evaluate/strategy update |
| PrimeCoordinatorAdapter | daa-prime-coordinator | 0.2.1 | BFT aggregate; **Gradient-only signature** |
| RvmCoherenceAdapter | rvm-coherence | path | mincut recompute/isolate |
| RvmWitnessAdapter | rvm-witness | path | 64-byte hash chain (R4) |
| RvmCapAdapter | rvm-cap/-security | path | capability authorize, scope/expiry |
| QuDagAdapter | QuDAG | latest | ML-DSA sign, ML-KEM-1024, .dark resolve, finality |
| RuVectorAdapter | ruvector | latest | HNSW upsert/search, GNN infer |
| SynapticMeshAdapter | Synaptic-Mesh | — | publish/subscribe, churn resilience |
| RufloMcpAdapter | ruflo / rvagent | latest | tool register/invoke, Claude reason |

**Rule:** a contract test failure here means the Ruv-crate API drifted from the port (spec risk R-1). The port/domain code does not change; only the adapter is repaired. This is the entire payoff of ADR-1.

---

## 16. R14 — Property / invariant suite (cross-cutting, `proptest`)

The four safety/security invariants from Pseudocode, generalized over generated inputs. These are the CI gate (Architecture §12).

| Invariant | Property test | Source FR |
|-----------|---------------|-----------|
| **No raw egress (gate)** | ∀ capability, ∀ request: `scope==Raw` ⇒ `CapabilityBroker` returns Err and `gate.authorize` is never invoked | FR-4.4 / NFR-7 |
| **No raw egress (type wall)** | compile-time + fuzz: `GradientAggregator::aggregate` accepts only `Gradient`; no `ThermalFrame`/`RawReading` constructible into its argument | FR-3.8 |
| **Governance gate ordering** | ∀ proposed trade: if executed, `rules.check` was called first; if `check`==Violation, `execute` call count == 0 | FR-3.7 |
| **Audit completeness** | ∀ executed trade: witness chain contains {TradeSigned, Exec, Routing} and `verify()==true`; tamper ⇒ false | FR-4.3 |
| **Forecaster resource safety** | ∀ infer outcome (Ok/Err/panic-as-Err): `spawn` count == `dissolve` count | FR-2.1 |
| **Consensus tamper-evidence** | ∀ entry: mutate ⇒ `verify==false` | FR-5.5 |

---

## 17. R15 — Acceptance suite goes green (AC-1..AC-8)

With units green (R1–R12), invariants green (R14), and adapters contract-verified (R13), the acceptance harness from R0 is rewired with real adapters behind faked **hardware and network only** (SEED sensors → `SimSensorAdapter` for the 30-day baseline, spec R-4).

| AC | Becomes green when | Trace |
|----|--------------------|-------|
| AC-1 ingestion | R6 + Rvcsi/Ed25519 contracts | FR-1.x |
| AC-2 coordinator up | R8 + node boot | FR-3.1/4.1 |
| AC-3 30-day baseline | R7 + SimSensor + Neuro contract | FR-2.4 |
| AC-4 broker + governance | R1 + R3 contract | FR-3.x/3.7 |
| AC-5 first signed trade | R2 + QuDag contract | FR-5.x |
| AC-6 dashboard | R11 + R12 + ruflo contract | FR-8/9 |
| **AC-7 autonomous trade + audit** | R1+R2+R4+R5 integrated | headline |
| **AC-8 no raw egress** | R14 properties hold across AC-1..7 | NFR-7 |

---

## 18. Iteration log structure (Refinement bookkeeping)

Each unit keeps a short log so Completion can audit the TDD discipline:

```
unit: BrokerAgent
  R1.5 rule_violation_short_circuits
     RED   2026-06-xx  expectation: execute never called on Violation — failed (executed)
     GREEN 2026-06-xx  added rules.check gate before ACT block
     REFACTOR          extracted decide() to decide.rs (agent.rs 612→380 ln, ≤500 ✓)
  coverage: 11/11 interaction tests green; decide() 14 example cases green
  invariants: FR-3.7 (R14) green
```

CI emits a Refinement report: per-FR test count, green/red, property status, module line counts, layering-lint result.

---

## 19. Definition of Done (per unit) — gate to Phase C

A unit exits Refinement only when **all** hold (spec §7.4):
1. A failing London test preceded the code (RED logged).
2. Every `// london:` interaction from Pseudocode is asserted (message/order/cardinality/absence) — no critical message un-asserted.
3. Relevant R14 property tests are green.
4. Its production adapter passes its R13 contract test.
5. Module ≤ 500 lines; inputs validated at the boundary; layering lint passes (no Ruv types in domain/ports).
6. `cargo build --workspace && cargo test` green (CLAUDE.md build gate).

---

## 20. Exit Criteria for the Refinement Phase

1. Every unit from Pseudocode has a red-green-refactor schedule with explicit interaction assertions. ✅ (R1–R12)
2. Every port has a contract test against its real Ruv crate. ✅ (R13)
3. All four+ safety/security invariants have property tests wired to CI. ✅ (R14)
4. AC-1..AC-8 have a defined path to green, with AC-7/AC-8 (the headline + no-raw-egress) explicitly integrated. ✅ (R15)
5. Per-unit Definition of Done enforces the London discipline and project rules. ✅ (R19)
6. An iteration log + CI report structure makes the TDD process auditable. ✅ (R18)

**Next phase:** C — Completion. Integrate on Appliance hardware, run the Phase-C benchmarks deferred here (NFR-1 <100ms, NFR-2 <1s, NFR-3 RVM nanoseconds, NFR-4 sub-ms), execute the District XIII First-Milestone bring-up, and demonstrate AC-7 — one fully autonomous peer-to-peer thermal trade with a complete rvm-witness audit trail.
