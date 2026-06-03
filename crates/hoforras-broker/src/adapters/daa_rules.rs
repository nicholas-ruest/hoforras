//! `DaaRulesAdapter` — the governance gate over `daa-rules` (DDD-01 / ADR-0015).
//!
//! Governance limits are **hard and non-overridable** (research §78–82): ≤ 500 kWh/day, ≥ 15%
//! safety reserve, ≤ 6 bar pipe pressure, ≤ 6 h trade window. The real `daa-rules` crate is
//! unavailable here, so this reference adapter encodes those limits directly behind the
//! `RulesEngine` port (swappable per ADR-0001).
//!
//! **Fail-closed (ADR-0015):** an adapter built with [`DaaRulesAdapter::unavailable`] denies every
//! trade. The broker treats any `Violation` as "do not execute, witness the rejection", so an
//! unavailable rules engine halts trading rather than risking an unsafe trade.

use hoforras_domain::{ProposedTrade, RuleId, RuleVerdict, GOVERNANCE};
use hoforras_ports::broker::RulesEngine;

/// Reference governance engine. Carries the node's current operating envelope (reserve, pressure)
/// since those are node state, not part of a `ProposedTrade`.
pub struct DaaRulesAdapter {
    reserve_pct: f32,
    pipe_pressure_bar: f32,
    available: bool,
}

impl Default for DaaRulesAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl DaaRulesAdapter {
    /// A healthy engine at the safe envelope (reserve at the floor, pressure at the ceiling).
    pub fn new() -> Self {
        Self {
            reserve_pct: GOVERNANCE.min_safety_reserve_percent,
            pipe_pressure_bar: GOVERNANCE.pipe_pressure_ceiling_bar,
            available: true,
        }
    }

    /// A healthy engine with a specific node envelope (used to exercise reserve/pressure limits).
    pub fn with_node_state(reserve_pct: f32, pipe_pressure_bar: f32) -> Self {
        Self {
            reserve_pct,
            pipe_pressure_bar,
            available: true,
        }
    }

    /// An **unavailable** rules engine — fails closed: every check is a `Violation` (ADR-0015).
    pub fn unavailable() -> Self {
        Self {
            reserve_pct: GOVERNANCE.min_safety_reserve_percent,
            pipe_pressure_bar: GOVERNANCE.pipe_pressure_ceiling_bar,
            available: false,
        }
    }
}

impl RulesEngine for DaaRulesAdapter {
    fn check(&self, trade: &ProposedTrade) -> RuleVerdict {
        if !self.available {
            // Fail closed: when the engine cannot evaluate, deny (ADR-0015). The specific rule is
            // immaterial — the point is the trade is refused and the broker witnesses it.
            return RuleVerdict::Violation(RuleId::MaxDailyThermalTransferKwh);
        }
        if trade.kwh.0 > GOVERNANCE.max_daily_thermal_transfer_kwh {
            return RuleVerdict::Violation(RuleId::MaxDailyThermalTransferKwh);
        }
        if self.reserve_pct < GOVERNANCE.min_safety_reserve_percent {
            return RuleVerdict::Violation(RuleId::MinSafetyReservePercent);
        }
        if self.pipe_pressure_bar > GOVERNANCE.pipe_pressure_ceiling_bar {
            return RuleVerdict::Violation(RuleId::PipePressureCeilingBar);
        }
        // Defense in depth: `TradeWindow` already caps hours ≤ ceiling at construction, but re-check.
        if trade.window.hours() > GOVERNANCE.trade_window_hours {
            return RuleVerdict::Violation(RuleId::TradeWindowHours);
        }
        RuleVerdict::Allow
    }
}
