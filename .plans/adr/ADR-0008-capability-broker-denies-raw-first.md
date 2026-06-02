# ADR-0008 — `CapabilityBroker` denies `Raw` scope before consulting the gate

- **Status:** Accepted
- **Date:** 2026-06-02
- **Traces:** `sparc.md` Part 3 ADR-6, §8; Part 1 FR-4.4, NFR-7; Part 2 §5.2
- **Constrains DDD:** Node Isolation & Security context (CapabilityBroker service)
- **Related:** ADR-0005 (the compile-time sibling)

## Context

Cross-partition reads are mediated by unforgeable `rvm-cap` capability tokens (scope + expiry).
The headline privacy invariant (NFR-7) is that **raw** sensor data never crosses a partition
boundary — only aggregated thermal availability does. If the broker delegated *all* decisions to the
capability gate, a forged-but-structurally-valid capability with `Scope::Raw` could, in principle,
be authorized.

## Decision

The `CapabilityBroker` applies a **hard scope rule before** consulting the gate:
`req.scope == Raw ⇒ Err(RawScopeForbidden)`, and the `CapabilityGate.authorize` collaborator is
**not even called**. Expiry is likewise checked against an injected clock before delegation.

## Consequences

**Positive**
- Defense in depth: even a valid capability cannot exfiltrate raw data; the gate is a second line,
  not the only line.
- Directly testable as an interaction (London): "raw scope ⇒ gate never invoked" (Refinement R5 #1),
  and generalized by a property test (R14).

**Negative / costs**
- Slight duplication of policy (broker + gate). Intentional — the redundancy *is* the control.

## Alternatives considered
- **Gate-only authorization** — rejected: single point of failure for the system's most important
  invariant; a gate bug or forged token would be catastrophic.
- **Compile-time only (as ADR-0005)** — not sufficient here: cross-partition *reads* are dynamic and
  capability-scoped at runtime, so a runtime gate is required; this ADR hardens it.
