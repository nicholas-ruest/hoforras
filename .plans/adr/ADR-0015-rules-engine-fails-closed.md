# ADR-0015 — Governance rules are hard limits; rules-engine outage fails closed

- **Status:** Accepted
- **Date:** 2026-06-02
- **Traces:** `sparc.md` Part 1 FR-3.7; Part 2 §4; Part 5 §6 (safety envelope)
- **Constrains DDD:** Thermal Market context (governance invariants)
- **Related:** ADR-0006

## Context

The thermal market is autonomous — trades execute with no human in the loop (AC-7). Physical safety
depends on hard constraints: `max_daily_thermal_transfer_kwh = 500`, `min_safety_reserve_percent =
0.15`, `pipe_pressure_ceiling_bar = 6.0`, `trade_window_hours = 6` (FR-3.7). The question is how the
broker behaves when the rules engine is unavailable or errors, and whether limits are advisory or
absolute.

## Decision

Governance rules are **hard, non-overridable limits**, evaluated by `RulesEngine.check` **before**
any `market.execute` (enforced by the single-tick ordering of ADR-0006). If the rules engine is
**unavailable or returns an error, the broker fails closed**: the trade is **not** executed and the
rejection is witnessed. There is no "proceed on rules error" path.

## Consequences

**Positive**
- Physical safety is preserved even under partial system failure — the autonomous market cannot
  exceed pipe-pressure or reserve limits because uncertainty resolves to "do not trade."
- The fail-closed branch is itself auditable (a witnessed rejection).

**Negative / costs**
- A rules-engine outage halts trading for the affected node (availability traded for safety) — the
  correct trade-off for an autonomous physical-energy system.

## Alternatives considered
- **Fail-open / advisory limits** — rejected outright: an autonomous broker exceeding pressure or
  reserve limits is a physical-safety hazard.
- **Operator override of limits** — rejected for the pilot: the system is designed to be safe
  *without* a human; an observe-only mode (Part 5 §6) exists for caution instead.
