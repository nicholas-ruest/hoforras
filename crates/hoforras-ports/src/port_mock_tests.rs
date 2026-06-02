//! Verifies that `mockall::automock` generates a working mock for representative ports — sync,
//! async, and the tricky boxed-trait-object return (`SwarmFactory → Box<dyn InferenceAgent>`).
//! This is the London-School seam (ADR-0001): if these compile and the interaction expectations
//! pass, every port is mockable for downstream unit tests.

use hoforras_domain::{AgentProfile, ProposedTrade, ReadingWindow, RuleVerdict};

use crate::broker::{MarketGateway, MockMarketGateway, MockRulesEngine, RulesEngine};
use crate::inference::{MockInferenceAgent, MockSwarmFactory, SwarmFactory};

fn proposed() -> ProposedTrade {
    use hoforras_domain::{JunctionId, Kwh, NodeId, Price, TradeWindow};
    ProposedTrade {
        seller: NodeId::new("a.thermal.budapest.dark").unwrap(),
        buyer: NodeId::new("b.thermal.budapest.dark").unwrap(),
        kwh: Kwh(40.0),
        price: Price {
            credit_per_kwh: 2.3,
        },
        window: TradeWindow::new(6).unwrap(),
        pipe_route: vec![JunctionId(7)],
    }
}

#[test]
fn sync_port_mock_records_interaction() {
    let mut rules = MockRulesEngine::new();
    rules
        .expect_check()
        .times(1)
        .returning(|_| RuleVerdict::Allow);
    assert_eq!(rules.check(&proposed()), RuleVerdict::Allow);
}

#[tokio::test]
async fn async_port_mock_records_interaction() {
    let mut market = MockMarketGateway::new();
    market.expect_post_bid().times(1).returning(|_| Ok(()));
    use hoforras_domain::{Bid, Kwh, NodeId, Price, TradeWindow};
    let bid = Bid {
        buyer: NodeId::new("b.thermal.budapest.dark").unwrap(),
        kwh: Kwh(10.0),
        max_price: Price {
            credit_per_kwh: 3.0,
        },
        window: TradeWindow::new(6).unwrap(),
    };
    market.post_bid(bid).await.unwrap();
}

#[tokio::test]
async fn factory_mock_yields_boxed_agent_mock() {
    // The hardest case: a mocked async method returning a boxed trait object.
    let mut factory = MockSwarmFactory::new();
    factory.expect_spawn().times(1).returning(|_| {
        let mut agent = MockInferenceAgent::new();
        agent.expect_dissolve().times(1).return_const(());
        Ok(Box::new(agent) as Box<dyn crate::inference::InferenceAgent>)
    });

    let agent = factory.spawn(AgentProfile::default()).await.unwrap();
    agent.dissolve();
    let _ = ReadingWindow {
        node_id: hoforras_domain::NodeId::new("a.thermal.budapest.dark").unwrap(),
        from_ts: hoforras_domain::Timestamp(0),
        to_ts: hoforras_domain::Timestamp(1),
        frames: vec![],
    };
}
