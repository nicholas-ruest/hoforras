# ADR-0003 — Introduce pure `hoforras-domain` and `hoforras-ports` crates

- **Status:** Accepted
- **Date:** 2026-06-02
- **Traces:** `sparc.md` Part 3 §2, §4.1
- **Constrains DDD:** the shared kernel of value types; the port catalogue
- **Related:** ADR-0002, ADR-0005

## Context

The research's Cargo workspace named four crates: `hoforras-sensor`, `hoforras-broker`,
`hoforras-node`, `hoforras-mesh`. None of these is a natural home for *shared, vendor-free* value
types (`ThermalFrame`, `ThermalTradeAgreement`, `Capability`, `Gradient`) or for the port traits
that all four implement/consume. Placing shared types in one of the four would create cyclic or
arbitrary dependencies.

## Decision

Add two **dependency-free** crates beneath the four:

- **`hoforras-domain`** — pure value types and the ubiquitous language (SPARC spec §10). No I/O, no
  Ruv dependencies.
- **`hoforras-ports`** — the ~30 port traits, expressed over `hoforras-domain` types only.

Dependency rule (acyclic, CI-enforced):
`domain ← ports ← {sensor, broker, node, mesh}`. A domain/ports crate importing any
`daa_*|rvm_*|qudag|ruvector|ruv_*` type is a build-failing layering violation.

## Consequences

**Positive**
- A single, canonical model shared by all contexts → DDD model integrity across the workspace.
- Ports are mockable (`mockall`) without pulling any vendor crate into the test binary.
- The type wall of ADR-0005 (`Gradient`-only) lives in `domain` where it cannot be bypassed.

**Negative / costs**
- Two extra crates to maintain.
- Discipline required: tempting to "just import the daa type" — prevented by the CI lint.

## Alternatives considered
- **Shared types inside `hoforras-broker`** — rejected: makes broker an upstream dependency of
  sensor/node/mesh, inverting the intended flow.
- **Re-export Ruv crate types as the domain model** — rejected: defeats ADR-0002 isolation and
  pollutes the ubiquitous language.
