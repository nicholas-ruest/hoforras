# ADR-0010 — One tokio runtime per Appliance

- **Status:** Accepted
- **Date:** 2026-06-02
- **Traces:** `sparc.md` Part 3 §6, ADR-8 (research `tokio = { version = "1", features = ["full"] }`)
- **Constrains DDD:** all contexts (runtime/process model)
- **Related:** ADR-0004, ADR-0006

## Context

The Appliance hosts ingestion, broker, consensus, mesh, inference, and memory. These need
async I/O (consensus, mesh) interleaved with synchronous, latency-critical work (RVM coherence,
ADR-0004). The research pins `tokio 1` (full). We must decide the runtime topology.

## Decision

Use **a single tokio runtime per Appliance**. Async I/O-bound work (consensus broadcast, mesh
pub/sub, WebSocket) runs on it; the broker MRAP loop is interval-driven on it (one tick at a time,
ADR-0006); RVM coherence runs synchronously *off* the async path (ADR-0004); ephemeral inference
agents are spawned per call and dissolved (FR-2.1).

## Consequences

**Positive**
- Simple supervision and shutdown; matches the research dependency pin.
- No cross-runtime coordination complexity at pilot scale (10–20 nodes).

**Negative / costs**
- A blocking call on the async path would stall the runtime; mitigated by keeping RVM work brief
  (nanoseconds) and using `spawn_blocking` for any genuinely blocking adapter.

## Alternatives considered
- **Multiple runtimes (per subsystem)** — rejected: needless complexity; no demonstrated need at
  pilot scale; complicates the single-tick invariant.
- **Thread-per-subsystem (no async)** — rejected: consensus/mesh are inherently async I/O.
