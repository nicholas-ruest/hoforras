//! Thermal Market ports (DDD-01, core). Consumed by `BrokerAgent` (built in Prompt 7).

use async_trait::async_trait;
use hoforras_domain::{
    AcceptedTrade, Adaptation, Bid, ExecutedTrade, Gradient, JunctionId, Offer, ProposedTrade,
    Reflection, RuleVerdict, Strategy, ThermalBalance, TradeOutcome,
};

use crate::PortResult;

/// Reads aggregated surplus/deficit from the local SEED mesh — Monitor (FR-3.1). Only the
/// aggregate `ThermalBalance` crosses; raw frames never do (NFR-7).
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
#[async_trait]
pub trait SeedMesh: Send + Sync {
    async fn read_surplus_deficit(&self) -> PortResult<ThermalBalance>;
}

/// Posts offers/bids, executes accepted trades, routes heat — Act (FR-3.3).
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
#[async_trait]
pub trait MarketGateway: Send + Sync {
    async fn post_offer(&self, offer: Offer) -> PortResult<()>;
    async fn post_bid(&self, bid: Bid) -> PortResult<()>;
    async fn execute(&self, trade: &AcceptedTrade) -> PortResult<()>;
    async fn route(&self, trade: &AcceptedTrade, path: &[JunctionId]) -> PortResult<()>;
}

/// Governance gate — hard limits, checked BEFORE execute (FR-3.7 / ADR-0006 / ADR-0015). Sync.
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait RulesEngine: Send + Sync {
    fn check(&self, trade: &ProposedTrade) -> RuleVerdict;
}

/// Evaluates a completed trade — Reflect (FR-3.4). Sync.
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait TradeEvaluator: Send + Sync {
    fn evaluate(&self, outcome: &TradeOutcome) -> Reflection;
}

/// Loads/updates the node's pricing strategy — Adapt (FR-3.5). Sync.
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait StrategyStore: Send + Sync {
    fn load(&self) -> Strategy;
    fn update(&self, adaptation: Adaptation);
}

/// Accounts executed trades in kWh-equivalent thermal credits (FR-3.6 / ADR-0013).
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
#[async_trait]
pub trait EconomyLedger: Send + Sync {
    async fn debit_credit(&self, trade: &ExecutedTrade) -> PortResult<()>;
}

/// Byzantine-fault-tolerant gradient aggregation for federated district training (FR-3.8).
///
/// **ADR-0005:** this accepts ONLY `Gradient` — no raw frame type can be passed, making
/// no-raw-egress a compile-time guarantee.
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
#[async_trait]
pub trait GradientAggregator: Send + Sync {
    async fn aggregate(&self, gradients: Vec<Gradient>) -> PortResult<Gradient>;
}
