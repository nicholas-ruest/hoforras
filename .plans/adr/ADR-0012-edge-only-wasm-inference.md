# ADR-0012 — Edge-only WASM inference — no GPU, no cloud

- **Status:** Accepted
- **Date:** 2026-06-02
- **Traces:** `sparc.md` Part 1 NFR-5, FR-2.5, FR-2.6; Part 2 §3; Part 3 §0
- **Constrains DDD:** Forecasting context
- **Related:** ADR-0009

## Context

Each building's thermal forecasting (LSTM / N-BEATS anomaly detection, burst-precursor prediction,
24 h demand) must run on the Cognitum Appliance with a <100 ms budget (NFR-1/FR-2.5). A cloud
dependency would add latency, a connectivity requirement, a data-egress path (conflicting with
NFR-7), and operating cost. ruv-FANN's Neuro-Divergent layer runs as WASM.

## Decision

Run **all neural inference as WASM on-device**. No GPU, no remote inference service. Forecasters are
**ephemeral** ruv-swarm agents: spawn a tiny specialist network, infer, dissolve (FR-2.1). There is
intentionally **no `RemoteInferenceClient` port** — edge-only is enforced by the absence of that
seam.

## Consequences

**Positive**
- No network dependency for the prediction hot path; supports intermittent connectivity (NFR-10).
- No raw data leaves the device to a model service (reinforces NFR-7, ADR-0009).
- Cost: commodity edge hardware, no GPU fleet.

**Negative / costs**
- Model size/complexity is bounded by the Appliance and the <100 ms budget.
- The <100 ms target is measured on real hardware in Phase C, not asserted against mocks.

## Alternatives considered
- **Cloud/GPU inference** — rejected: latency, connectivity, egress, and cost all violate stated
  constraints.
- **Persistent per-node model server** — rejected in favour of ephemeral spawn/dissolve, which
  matches ruv-swarm's model and bounds resource use (property-tested: spawn count == dissolve count).
