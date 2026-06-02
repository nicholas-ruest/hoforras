# ADR-0001 — Adopt SPARC methodology underpinned by London-School TDD

- **Status:** Accepted
- **Date:** 2026-06-02
- **Deciders:** Hőforrás architecture group
- **Traces:** `sparc.md` Part 1 §0, §7; Part 4 (Refinement)
- **Constrains DDD:** all bounded contexts (process-level)

## Context

Hőforrás integrates nine distinct technical layers (sensing, inference, brokerage, isolation,
consensus, memory, mesh, AI orchestration, frontend), most reused from upstream Ruv repositories
whose APIs may drift. We need a development process that (a) fixes *what/why* before *how*,
(b) makes correctness verifiable at every step, and (c) tolerates third-party churn without
rewriting domain logic.

Two failure modes loom: building the wrong thing (no requirements discipline), and building on
collaborators that are slow/non-deterministic to test (real consensus, real ML inference, real
RVM partitions).

## Decision

Adopt **SPARC** (Specification → Pseudocode → Architecture → Refinement → Completion) as the
phase model, and **London-School (mockist, outside-in) TDD** as the test discipline that
underpins it.

- Every functional requirement is expressed as a *collaborator contract* — a unit-under-test and
  the interaction with its mocked neighbours.
- Refinement proceeds **outside-in** from the headline acceptance test (AC-7) and discovers
  collaborators by need.
- Interaction is asserted over state (message, order, cardinality, absence), because the
  expensive/non-deterministic neighbours are mocked.

## Consequences

**Positive**
- Requirements, algorithms, structure, and tests form one traceable chain (`sparc.md` Part 5 §8).
- Mockability becomes a *first-class architectural constraint*, forcing clean seams (see ADR-0002).
- Third-party drift is caught at adapter contract tests, not in domain code.

**Negative / costs**
- Up-front documentation effort (the five SPARC artifacts).
- London-style tests couple to interaction shape; refactors that change collaboration require test
  updates. Mitigated by keeping pure cores (e.g. `decide()`) state-tested.

## Alternatives considered
- **Classicist (Detroit) TDD** — state-based, fewer mocks. Rejected: the core collaborators
  (consensus, inference, RVM) are too costly/non-deterministic to drive by real state in unit tests.
- **Ad-hoc design + integration tests only** — rejected: no early correctness signal; third-party
  drift would surface late and expensively.
