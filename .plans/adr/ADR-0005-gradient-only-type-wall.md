# ADR-0005 — `GradientAggregator` accepts only `domain::Gradient` (compile-time no-raw-egress)

- **Status:** Accepted
- **Date:** 2026-06-02
- **Traces:** `sparc.md` Part 3 ADR-3, §7; Part 1 FR-3.8, NFR-7; Part 2 §4.1
- **Constrains DDD:** Thermal Market (federated training), Node Isolation & Security
- **Related:** ADR-0008 (the runtime sibling of this compile-time control)

## Context

NFR-7 / FR-3.8: raw sensor data (`ThermalFrame`, `RawReading`) must **never** leave a building's
partition. Federated district-model training (`daa-prime-coordinator`) deliberately moves data
*across* buildings — gradients only. A runtime check ("is this a gradient?") can regress silently
if a future change passes the wrong type.

## Decision

Make no-raw-egress a **compile-time guarantee** in the federated-training path: the
`GradientAggregator` port's signature accepts **only** `domain::Gradient`:

```rust
pub trait GradientAggregator { async fn aggregate(&self, g: Vec<Gradient>) -> Result<Gradient>; }
```

`Gradient` lives in `hoforras-domain` and is constructed only from local computation; there is no
`From<ThermalFrame> for Gradient` and no path that places a frame into the aggregator argument.

## Consequences

**Positive**
- A raw frame crossing the federation boundary becomes a *type error*, not a test failure — the
  strongest possible enforcement of the system's headline privacy invariant.
- Complements the runtime gate (ADR-0008) for defense in depth; together they cover both the
  capability path and the training path.

**Negative / costs**
- The embedding/gradient derivation must be deliberate (no convenience conversions). This is the
  intended friction.

## Alternatives considered
- **Runtime scope check on a generic payload** — rejected (ADR-3 rationale in SPARC): weaker, can
  regress, and shifts a safety-critical invariant from the compiler to test coverage.
