# ADR-0013 — Thermal-credit token economy (kWh-equivalent, replaces rUv)

- **Status:** Accepted
- **Date:** 2026-06-02
- **Traces:** `sparc.md` Part 1 FR-3.6; Part 2 §4; research Layer 3
- **Constrains DDD:** Thermal Market context (EconomyLedger, Price value objects)
- **Related:** ADR-0006, ADR-0015

## Context

The broker is built on `daa-economy`, whose native unit of account is the rUv token. Hőforrás trades
**heat**, not generic value. The unit of account must be physically meaningful, auditable against
delivered energy, and intelligible to operators and (eventually) regulators.

## Decision

Replace the rUv token with a **thermal credit** denominated as **kWh-equivalent**. `daa-economy`
accounts every trade in thermal credits; `credit_price_per_kwh` prices offers/bids; the
`EconomyLedger` debits/credits nodes on executed trades (FR-3.6). Settlement to fiat currency is
explicitly **out of pilot scope** (spec §3.2).

## Consequences

**Positive**
- The unit of account maps 1:1 to the physical commodity (heat), so trades are auditable against
  metered delivery and the witness trail.
- Operators reason in kWh, not an abstract token.

**Negative / costs**
- Requires a clean mapping/adapter layer over `daa-economy`'s rUv semantics (anti-corruption).
- No fiat settlement means the pilot demonstrates a *market*, not a billing system — stated plainly.

## Alternatives considered
- **Keep rUv tokens** — rejected: not physically meaningful for heat; poor operator/regulator
  intelligibility.
- **Direct fiat pricing** — rejected: drags in billing, compliance, and FX out of pilot scope.
