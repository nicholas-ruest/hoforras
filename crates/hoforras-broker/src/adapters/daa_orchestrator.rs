//! `DaaOrchestratorAdapter` — the market mechanism over `daa-orchestrator` (DDD-01).
//!
//! Bundles the three market-mechanism ports the broker drives in Act/Reflect/Adapt: `MarketGateway`
//! (post/execute/route), `TradeEvaluator` (Reflect), and `StrategyStore` (Adapt). The real
//! `daa-orchestrator` substrate is unavailable; this reference adapter records posted/executed
//! trades and keeps a strategy in memory. By default no counterparty is matched (`await_acceptance`
//! returns `None`) — tests inject acceptance via the mock; a configured acceptance can be set for
//! integration.

use std::sync::Mutex;

use async_trait::async_trait;
use hoforras_domain::{
    AcceptedTrade, Adaptation, Bid, Decision, JunctionId, Offer, Reflection, Strategy, TradeOutcome,
};
use hoforras_ports::broker::{MarketGateway, StrategyStore, TradeEvaluator};
use hoforras_ports::PortResult;

/// Reference market orchestrator.
pub struct DaaOrchestratorAdapter {
    strategy: Mutex<Strategy>,
    posted: Mutex<usize>,
    executed: Mutex<usize>,
    acceptance: Mutex<Option<AcceptedTrade>>,
}

impl DaaOrchestratorAdapter {
    pub fn new(initial: Strategy) -> Self {
        Self {
            strategy: Mutex::new(initial),
            posted: Mutex::new(0),
            executed: Mutex::new(0),
            acceptance: Mutex::new(None),
        }
    }

    /// Pre-arm a counterparty acceptance the next `await_acceptance` will return (integration use).
    pub fn set_acceptance(&self, accepted: Option<AcceptedTrade>) {
        *self.acceptance.lock().expect("orchestrator poisoned") = accepted;
    }

    pub fn posted_count(&self) -> usize {
        *self.posted.lock().expect("orchestrator poisoned")
    }

    pub fn executed_count(&self) -> usize {
        *self.executed.lock().expect("orchestrator poisoned")
    }
}

#[async_trait]
impl MarketGateway for DaaOrchestratorAdapter {
    async fn post_offer(&self, _offer: Offer) -> PortResult<()> {
        *self.posted.lock().expect("orchestrator poisoned") += 1;
        Ok(())
    }

    async fn post_bid(&self, _bid: Bid) -> PortResult<()> {
        *self.posted.lock().expect("orchestrator poisoned") += 1;
        Ok(())
    }

    async fn await_acceptance(&self, _decision: &Decision) -> PortResult<Option<AcceptedTrade>> {
        Ok(self
            .acceptance
            .lock()
            .expect("orchestrator poisoned")
            .clone())
    }

    async fn execute(&self, _trade: &AcceptedTrade) -> PortResult<()> {
        *self.executed.lock().expect("orchestrator poisoned") += 1;
        Ok(())
    }

    async fn route(&self, _trade: &AcceptedTrade, _path: &[JunctionId]) -> PortResult<()> {
        Ok(())
    }
}

impl TradeEvaluator for DaaOrchestratorAdapter {
    fn evaluate(&self, outcome: &TradeOutcome) -> Reflection {
        let value_ratio = if outcome.expected_kwh > 0.0 {
            outcome.delivered_kwh / outcome.expected_kwh
        } else {
            0.0
        };
        Reflection {
            value_ratio,
            note: "reference evaluation".into(),
        }
    }
}

impl StrategyStore for DaaOrchestratorAdapter {
    fn load(&self) -> Strategy {
        self.strategy.lock().expect("orchestrator poisoned").clone()
    }

    fn update(&self, adaptation: Adaptation) {
        let mut s = self.strategy.lock().expect("orchestrator poisoned");
        s.offer_threshold_kwh = (s.offer_threshold_kwh + adaptation.offer_threshold_delta).max(0.0);
        s.bid_threshold_kwh = (s.bid_threshold_kwh + adaptation.bid_threshold_delta).max(0.0);
    }
}
