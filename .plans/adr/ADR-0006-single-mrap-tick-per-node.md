# ADR-0006 — One MRAP tick in flight per node

- **Status:** Accepted
- **Date:** 2026-06-02
- **Traces:** `sparc.md` Part 3 §6, ADR-4; Part 1 FR-3.7; Part 2 §4
- **Constrains DDD:** Thermal Market context (TradingNode aggregate)
- **Related:** ADR-0015

## Context

The broker's MRAP loop (Monitor → Reason → Act → Reflect → Adapt) mutates shared node state:
strategy, open positions, the economy ledger. FR-3.7 requires a strict ordering invariant —
`rules.check` **before** `market.execute`, and zero executions on a rule violation. Concurrent
overlapping ticks would race on rules/economy state and could interleave a check from tick A with
an execute from tick B, defeating the invariant.

## Decision

Permit **at most one MRAP tick in flight per node**. `BrokerAgent::tick()` is driven on an interval
timer; a new tick does not start until the previous one completes (or times out). The TradingNode
aggregate is therefore a single-writer.

## Consequences

**Positive**
- The FR-3.7 ordering invariant is trivially guaranteed within a tick — no cross-tick interleaving.
- Aggregate consistency (DDD) holds without distributed locking; the node is the consistency
  boundary.

**Negative / costs**
- Throughput per node is bounded by tick duration. Acceptable: a building participates in few
  trades per window; consensus (async) does not block the next tick's *scheduling*, only its start.

## Alternatives considered
- **Concurrent ticks with locks around rules/economy** — rejected: reintroduces interleaving risk
  and lock complexity for no real throughput need at pilot scale (10–20 nodes).
- **Event-sourced command queue per node** — deferred: viable at larger scale; unnecessary now.
