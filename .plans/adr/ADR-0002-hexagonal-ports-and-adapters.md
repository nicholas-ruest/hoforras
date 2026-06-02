# ADR-0002 — Hexagonal (ports-and-adapters) architecture

- **Status:** Accepted
- **Date:** 2026-06-02
- **Traces:** `sparc.md` Part 3 §0, §3, ADR-1; Part 1 §7 (TDD)
- **Constrains DDD:** all contexts; defines the anti-corruption boundary to Ruv crates
- **Related:** ADR-0001, ADR-0003, ADR-0005

## Context

Domain logic must be testable in isolation (ADR-0001) and insulated from upstream Ruv-crate API
changes (spec risk R-1). If domain code depended directly on `daa_*`, `rvm_*`, `qudag`, `ruvector`,
etc., it would (a) be unmockable without those crates present, (b) leak third-party types into the
ubiquitous language, and (c) break whenever an upstream API shifts.

## Decision

Adopt **Hexagonal / Ports-and-Adapters** per crate. Domain units (the units from SPARC Pseudocode)
depend **only on port traits**. Each Ruv crate is reached **exclusively** through an *adapter* that
implements a port. Adapters are the only place third-party types appear.

```
domain (pure types) ◀ ports (traits) ◀ {sensor, broker, node, mesh} (units + adapters)
                                          adapters → Ruv crates
```

## Consequences

**Positive**
- Units are constructed with mocks in unit tests, real adapters in `main()` (constructor injection).
- Upstream drift is contained to a single adapter and caught by its contract test (ADR-0001).
- The ubiquitous language stays free of vendor nouns (supports DDD model integrity).

**Negative / costs**
- Indirection: one trait + one adapter per integration point (~30 ports). Accepted as the price of
  testability and isolation.
- Requires a CI lint enforcing the dependency direction (no Ruv types in `domain`/`ports`).

## Alternatives considered
- **Direct dependence on Ruv crates in domain** — rejected: untestable without the crates, leaks
  types, brittle to drift.
- **Single facade layer over all Ruv crates** — rejected: one god-module violates ≤500-line rule and
  couples unrelated concerns; per-port adapters keep seams sharp.
