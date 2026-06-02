# ADR-0004 — Run RVM coherence in-process, off the async path

- **Status:** Accepted
- **Date:** 2026-06-02
- **Traces:** `sparc.md` Part 3 §6, ADR-2; Part 1 NFR-3
- **Constrains DDD:** Node Isolation & Security context
- **Related:** ADR-0009, ADR-0010

## Context

NFR-3 requires real-time, effectively-imperceptible node isolation: partition switch ~6 ns,
16-node mincut ~331 ns, witness emit ~17 ns (vendor benchmarks; spec risk R-3). Any network hop or
async scheduling jitter around the coherence engine would dominate these budgets by orders of
magnitude and make automatic re-isolation (FR-4.2) too slow to be "without perceptible latency."

## Decision

Run `rvm-coherence` / `rvm-kernel` **synchronously, in-process** inside `hoforras-node`. The
`CoherenceSupervisor` calls the mincut/partition operations inline; they are **not** placed on the
tokio async path or behind any RPC.

## Consequences

**Positive**
- Isolation latency is bounded by RVM itself, not by the runtime — preserves NFR-3.
- The coherence domain *is* the process/security boundary, so raw data physically cannot escape it
  (reinforces ADR-0008, NFR-7).

**Negative / costs**
- Coherence operations block the calling thread briefly; acceptable given nanosecond scale.
- Couples node lifecycle to RVM; mitigated by the `MincutEngine`/`PartitionController` ports
  (still mockable in unit tests per ADR-0002).

## Alternatives considered
- **Isolation-as-a-service (separate process/host)** — rejected: a network round-trip alone blows
  the NFR-3 budget by ~6 orders of magnitude.
- **Coherence on the async runtime** — rejected: scheduler latency/jitter is unbounded relative to
  nanosecond targets.
