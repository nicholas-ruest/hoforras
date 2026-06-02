# Architecture Decision Records — Hőforrás Budapest

Decision log for the Hőforrás peer-to-peer thermal energy intelligence system, derived from
`.plans/sparc.md` (SPARC: Specification · Pseudocode · Architecture · Refinement · Completion).

ADRs capture **one architecturally-significant decision each**, in [MADR](https://adr.github.io/madr/)-style:
*Status · Context · Decision · Consequences · Alternatives considered*. They are immutable once
**Accepted**; a reversal is a new ADR that *supersedes* the old one.

## Index

| ADR | Title | Status | SPARC origin |
|-----|-------|--------|--------------|
| [ADR-0001](ADR-0001-sparc-with-london-tdd.md) | Adopt SPARC methodology underpinned by London-School TDD | Accepted | process |
| [ADR-0002](ADR-0002-hexagonal-ports-and-adapters.md) | Hexagonal (ports-and-adapters) architecture | Accepted | A·ADR-1 |
| [ADR-0003](ADR-0003-pure-domain-and-ports-crates.md) | Introduce pure `hoforras-domain` and `hoforras-ports` crates | Accepted | A·ADR-1 |
| [ADR-0004](ADR-0004-rvm-coherence-in-process.md) | Run RVM coherence in-process, off the async path | Accepted | A·ADR-2 |
| [ADR-0005](ADR-0005-gradient-only-type-wall.md) | `GradientAggregator` accepts only `domain::Gradient` (compile-time no-raw-egress) | Accepted | A·ADR-3 |
| [ADR-0006](ADR-0006-single-mrap-tick-per-node.md) | One MRAP tick in flight per node | Accepted | A·ADR-4 |
| [ADR-0007](ADR-0007-dag-transport-over-rpc.md) | DAG transport over RPC for cross-node propagation | Accepted | A·ADR-5 |
| [ADR-0008](ADR-0008-capability-broker-denies-raw-first.md) | `CapabilityBroker` denies `Raw` scope before consulting the gate | Accepted | A·ADR-6 |
| [ADR-0009](ADR-0009-inference-memory-inside-node.md) | Inference & memory live inside `hoforras-node` (the RVM boundary) | Accepted | A·ADR-7 |
| [ADR-0010](ADR-0010-single-tokio-runtime.md) | One tokio runtime per Appliance | Accepted | A·ADR-8 |
| [ADR-0011](ADR-0011-post-quantum-consensus.md) | Post-quantum crypto (ML-DSA / ML-KEM-1024) for trade consensus | Accepted | S·FR-5 |
| [ADR-0012](ADR-0012-edge-only-wasm-inference.md) | Edge-only WASM inference — no GPU, no cloud | Accepted | S·NFR-5 |
| [ADR-0013](ADR-0013-thermal-credit-economy.md) | Thermal-credit token economy (kWh-equivalent, replaces rUv) | Accepted | S·FR-3.6 |
| [ADR-0014](ADR-0014-postcard-canonical-bytes.md) | `postcard` canonical serialization for all signed bytes | Accepted | P·§6 |
| [ADR-0015](ADR-0015-rules-engine-fails-closed.md) | Governance rules are hard limits; rules-engine outage fails closed | Accepted | S·FR-3.7 |

## Conventions
- File name: `ADR-NNNN-kebab-title.md`.
- Statuses: `Proposed` → `Accepted` → (`Superseded by ADR-MMMM` | `Deprecated`).
- Every ADR traces back to a SPARC requirement (FR/NFR) or phase decision and forward to the
  DDD bounded context(s) it constrains (`.plans/ddd/`).
