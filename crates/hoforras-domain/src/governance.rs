//! Governance rules — hard, non-overridable safety limits (FR-3.7 / ADR-0015 / DDD-01).

use serde::{Deserialize, Serialize};

/// The four hard limits from `sparc.md` FR-3.7. These are constants, not config: they are
/// physical-safety bounds an autonomous broker may never exceed (ADR-0015).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct GovernanceRules {
    pub max_daily_thermal_transfer_kwh: f32,
    pub min_safety_reserve_percent: f32,
    pub pipe_pressure_ceiling_bar: f32,
    pub trade_window_hours: u32,
}

/// The canonical district governance limits (research §78–82).
pub const GOVERNANCE: GovernanceRules = GovernanceRules {
    max_daily_thermal_transfer_kwh: 500.0,
    min_safety_reserve_percent: 0.15,
    pipe_pressure_ceiling_bar: 6.0,
    trade_window_hours: 6,
};

/// Identifies which governance rule a verdict refers to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleId {
    MaxDailyThermalTransferKwh,
    MinSafetyReservePercent,
    PipePressureCeilingBar,
    TradeWindowHours,
}

/// The result of checking a proposed trade against governance (FR-3.7).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum RuleVerdict {
    Allow,
    Violation(RuleId),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn governance_constants_match_spec() {
        assert_eq!(GOVERNANCE.max_daily_thermal_transfer_kwh, 500.0);
        assert_eq!(GOVERNANCE.min_safety_reserve_percent, 0.15);
        assert_eq!(GOVERNANCE.pipe_pressure_ceiling_bar, 6.0);
        assert_eq!(GOVERNANCE.trade_window_hours, 6);
    }
}
