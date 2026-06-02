# ADR-0009 — Inference & memory live inside `hoforras-node` (the RVM boundary)

- **Status:** Accepted
- **Date:** 2026-06-02
- **Traces:** `sparc.md` Part 3 §4.4, ADR-7; Part 1 NFR-7, NFR-5
- **Constrains DDD:** Forecasting context, Thermal Memory context
- **Related:** ADR-0004, ADR-0012

## Context

Forecasting (ruv-FANN / ruv-swarm) and thermal memory (RuVector) both operate on **raw**,
per-building `ThermalFrame` data. NFR-7 forbids raw data leaving the building's RVM coherence
domain. If inference or memory ran as separate crates/processes, raw frames would cross a
crate/process boundary to reach them.

## Decision

House the Forecasting and Thermal Memory units **inside `hoforras-node`**, within the same RVM
coherence domain as ingestion — rather than as their own top-level crates. Their ports
(`SwarmFactory`, `InferenceAgent`, `VectorIndex`, `GnnEngine`) keep them mockable, but their runtime
home is node-local.

## Consequences

**Positive**
- Raw frames are consumed where they are produced — they never appear on a mesh or capability edge.
- Inference stays on-device (supports ADR-0012 edge-only / NFR-5).
- Memory's witness replay (FR-6.5) sits beside the witness chain it mirrors.

**Negative / costs**
- `hoforras-node` is the largest crate; mitigated by per-unit modules ≤500 lines (NFR-12).
- Cross-building model improvement must go through the gradient path (ADR-0005), not by sharing
  memory — intended.

## Alternatives considered
- **Separate `hoforras-inference` / `hoforras-memory` crates** — rejected: raw data would cross a
  boundary, violating NFR-7, and an inference service invites a network hop that breaks edge-only.
