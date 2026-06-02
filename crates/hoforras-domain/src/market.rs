//! Thermal Market value objects (DDD-01, the core domain).

use serde::{Deserialize, Serialize};

use crate::ids::{JunctionId, NodeId, Timestamp};

/// Kilowatt-hours of thermal energy.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Kwh(pub f32);

/// A price in thermal credits per kWh (ADR-0013).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Price {
    pub credit_per_kwh: f32,
}

/// A trade duration. Governance caps this at `GOVERNANCE.trade_window_hours` (FR-3.7); the
/// constructor enforces the hard upper bound (ADR-0015).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct TradeWindow {
    hours: u32,
}

impl TradeWindow {
    pub fn new(hours: u32) -> Result<Self, crate::DomainError> {
        if hours == 0 || hours > crate::GOVERNANCE.trade_window_hours {
            return Err(crate::DomainError::Governance(format!(
                "trade window {hours}h outside (0, {}h]",
                crate::GOVERNANCE.trade_window_hours
            )));
        }
        Ok(Self { hours })
    }

    pub fn hours(&self) -> u32 {
        self.hours
    }
}

/// Aggregated surplus/deficit for a building — the **only** thermal quantity published to the
/// market (raw frames stay node-local; NFR-7 / ADR-0009 / DDD-02→DDD-01 Published Language).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ThermalBalance {
    pub node_id: NodeId,
    pub surplus_kwh: f32,
    pub deficit_kwh: f32,
    pub window: TradeWindow,
}

/// Intent to sell heat.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Offer {
    pub seller: NodeId,
    pub kwh: Kwh,
    pub price: Price,
    pub window: TradeWindow,
}

/// Intent to buy heat.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Bid {
    pub buyer: NodeId,
    pub kwh: Kwh,
    pub max_price: Price,
    pub window: TradeWindow,
}

/// The reasoning output of the MRAP loop (FR-3.2).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Decision {
    Offer(Offer),
    Bid(Bid),
    Hold,
}

/// A candidate trade awaiting governance + consensus (DDD-01 `TradeProposal`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProposedTrade {
    pub seller: NodeId,
    pub buyer: NodeId,
    pub kwh: Kwh,
    pub price: Price,
    pub window: TradeWindow,
    pub pipe_route: Vec<JunctionId>,
}

/// A trade a counterparty has accepted, ready for consensus then execution/routing.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AcceptedTrade {
    pub proposed: ProposedTrade,
    pub accepted_at: Timestamp,
    pub pipe_route: Vec<JunctionId>,
}

/// A trade that has been executed and routed.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExecutedTrade {
    pub accepted: AcceptedTrade,
    pub executed_at: Timestamp,
}

/// Observed outcome of a completed trade (input to Reflect).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TradeOutcome {
    pub executed: ExecutedTrade,
    pub delivered_kwh: f32,
    pub expected_kwh: f32,
}

/// Reflection produced by evaluating a `TradeOutcome` (FR-3.4).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Reflection {
    pub value_ratio: f32,
    pub note: String,
}

/// A strategy adaptation produced from a `Reflection` (FR-3.5).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Adaptation {
    pub offer_threshold_delta: f32,
    pub bid_threshold_delta: f32,
}

/// The mutable pricing/decision strategy of a `TradingNode` (DDD-01 entity).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Strategy {
    pub offer_threshold_kwh: f32,
    pub bid_threshold_kwh: f32,
    pub reserve_pct: f32,
    pub base_credit_per_kwh: f32,
    pub window_hours: u32,
}
