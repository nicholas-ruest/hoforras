# ADR-0007 — DAG transport over RPC for cross-node propagation

- **Status:** Accepted
- **Date:** 2026-06-02
- **Traces:** `sparc.md` Part 3 §6, ADR-5; Part 1 FR-7.1, NFR-9, NFR-10
- **Constrains DDD:** District Mesh Coordination, Trade Consensus
- **Related:** ADR-0011

## Context

Building Appliances have intermittent connectivity (spec assumption). Market state, anomaly
signals, and consensus decisions must propagate across the district and survive node drop/return
without losing committed state (NFR-9/10). Synchronous RPC assumes both peers are online and offers
no natural replay on reconnection.

## Decision

All cross-node propagation occurs as **signed DAG entries** over the Synaptic-Mesh / QuDAG fabric
(`MeshTransport.publish` / `subscribe`), **never** via direct RPC calls. There is intentionally no
RPC port in `hoforras-ports`.

## Consequences

**Positive**
- The mesh self-heals: a returning node replays missed DAG entries to converge (eventual
  consistency) — satisfies NFR-9/10.
- Propagation is event-driven, matching the system-level "no polling" ethos.
- Entries are signed and tamper-evident, dovetailing with consensus (ADR-0011).

**Negative / costs**
- Eventual (not immediate) consistency between nodes; the dashboard reflects state as entries
  arrive. Acceptable for a thermal market operating on multi-hour trade windows.
- Requires DAG entry versioning/replay logic in the mesh adapter.

## Alternatives considered
- **gRPC/HTTP RPC mesh** — rejected: no offline tolerance, no replay, brittle under churn.
- **Central message broker** — rejected: introduces a directory/single point of failure, conflicts
  with directory-less `.dark` discovery (FR-5.4).
