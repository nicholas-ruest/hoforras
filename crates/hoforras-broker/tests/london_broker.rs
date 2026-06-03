//! R1 — the `BrokerAgent` MRAP interaction suite (FR-3.1–3.9), the keystone of AC-7.
//!
//! All nine collaborators are mocked. The headline assertions are `rule_violation_short_circuits`
//! (governance Violation ⇒ `execute` NEVER called) and the strict orderings `rules_checked_before_
//! execute`, `consensus_before_routing`, and the three-witness audit on execute.

use hoforras_broker::{BrokerAgent, TickReport};
use hoforras_domain::{
    AcceptedTrade, Adaptation, DemandForecast, Finality, JunctionId, Kwh, NodeId, Price,
    PrivilegedAction, ProposedTrade, Reflection, RuleId, RuleVerdict, Strategy, ThermalBalance,
    Timestamp, TradeWindow, WitnessRecord,
};
use hoforras_ports::broker::{
    MockEconomyLedger, MockMarketGateway, MockRulesEngine, MockSeedMesh, MockStrategyStore,
    MockTradeEvaluator,
};
use hoforras_ports::consensus::MockConsensusGateway;
use hoforras_ports::inference::MockForecastService;
use hoforras_ports::security::MockWitnessChain;
use mockall::Sequence;

fn node() -> NodeId {
    NodeId::new("pozsonyi14.thermal.budapest.dark").unwrap()
}

fn balance(surplus: f32, deficit: f32) -> ThermalBalance {
    ThermalBalance {
        node_id: node(),
        surplus_kwh: surplus,
        deficit_kwh: deficit,
        window: TradeWindow::new(6).unwrap(),
    }
}

fn forecast() -> DemandForecast {
    DemandForecast {
        hourly_kwh: vec![10.0; 24], // total 240 ⇒ reserve need 36 at 0.15
    }
}

fn strategy() -> Strategy {
    Strategy {
        offer_threshold_kwh: 5.0,
        bid_threshold_kwh: 5.0,
        reserve_pct: 0.15,
        base_credit_per_kwh: 2.0,
        window_hours: 6,
    }
}

fn accepted() -> AcceptedTrade {
    AcceptedTrade {
        proposed: ProposedTrade {
            seller: node(),
            buyer: NodeId::new("pozsonyi22.thermal.budapest.dark").unwrap(),
            kwh: Kwh(40.0),
            price: Price {
                credit_per_kwh: 2.3,
            },
            window: TradeWindow::new(6).unwrap(),
            pipe_route: vec![JunctionId(7), JunctionId(12)],
        },
        accepted_at: Timestamp(1),
        pipe_route: vec![JunctionId(7), JunctionId(12), JunctionId(18)],
    }
}

fn witness_ok() -> WitnessRecord {
    WitnessRecord::new([0u8; 64])
}

/// Mocks that monitor+reason to the decision implied by (surplus, deficit).
fn monitor_reason(
    surplus: f32,
    deficit: f32,
) -> (MockSeedMesh, MockForecastService, MockStrategyStore) {
    let mut seed = MockSeedMesh::new();
    seed.expect_read_surplus_deficit()
        .returning(move || Ok(balance(surplus, deficit)));
    let mut fc = MockForecastService::new();
    fc.expect_demand_24h().returning(|_| Ok(forecast()));
    let mut st = MockStrategyStore::new();
    st.expect_load().returning(strategy);
    (seed, fc, st)
}

// 1 — Monitor then Reason: read_surplus_deficit before demand_24h (FR-3.1).
#[tokio::test]
async fn tick_monitors_then_reasons() {
    let mut seq = Sequence::new();
    let mut seed = MockSeedMesh::new();
    seed.expect_read_surplus_deficit()
        .times(1)
        .in_sequence(&mut seq)
        .returning(|| Ok(balance(40.0, 0.0))); // ⇒ Hold
    let mut fc = MockForecastService::new();
    fc.expect_demand_24h()
        .times(1)
        .in_sequence(&mut seq)
        .returning(|_| Ok(forecast()));
    let mut st = MockStrategyStore::new();
    st.expect_load().returning(strategy);

    let agent = BrokerAgent::new(
        seed,
        fc,
        MockRulesEngine::new(),
        MockMarketGateway::new(),
        MockTradeEvaluator::new(),
        st,
        MockEconomyLedger::new(),
        MockWitnessChain::new(),
        MockConsensusGateway::new(),
    );
    assert_eq!(agent.tick().await.unwrap(), TickReport::Idle);
}

// 2 — Hold posts nothing: no post_offer/post_bid/execute (FR-3.2).
#[tokio::test]
async fn hold_when_balanced_posts_nothing() {
    let (seed, fc, st) = monitor_reason(40.0, 0.0); // ⇒ Hold
    let mut market = MockMarketGateway::new();
    market.expect_post_offer().times(0);
    market.expect_post_bid().times(0);
    market.expect_execute().times(0);

    let agent = BrokerAgent::new(
        seed,
        fc,
        MockRulesEngine::new(),
        market,
        MockTradeEvaluator::new(),
        st,
        MockEconomyLedger::new(),
        MockWitnessChain::new(),
        MockConsensusGateway::new(),
    );
    assert_eq!(agent.tick().await.unwrap(), TickReport::Idle);
}

// 3 — Surplus posts an offer with kwh=net, duration≤6 (FR-3.2/3.3).
#[tokio::test]
async fn surplus_posts_offer() {
    let (seed, fc, st) = monitor_reason(100.0, 0.0); // net = 100 - 36 = 64 ⇒ Offer
    let mut rules = MockRulesEngine::new();
    rules.expect_check().returning(|_| RuleVerdict::Allow);
    let mut market = MockMarketGateway::new();
    market
        .expect_post_offer()
        .times(1)
        .withf(|o| (o.kwh.0 - 64.0).abs() < 1e-3 && o.window.hours() <= 6)
        .returning(|_| Ok(()));
    market.expect_await_acceptance().returning(|_| Ok(None)); // no counterparty ⇒ Posted

    let agent = BrokerAgent::new(
        seed,
        fc,
        rules,
        market,
        MockTradeEvaluator::new(),
        st,
        MockEconomyLedger::new(),
        MockWitnessChain::new(),
        MockConsensusGateway::new(),
    );
    assert_eq!(agent.tick().await.unwrap(), TickReport::Posted);
}

// 4 — Deficit posts a bid (FR-3.2).
#[tokio::test]
async fn deficit_posts_bid() {
    let (seed, fc, st) = monitor_reason(0.0, 80.0); // net = -80 - 36 = -116 ⇒ Bid
    let mut rules = MockRulesEngine::new();
    rules.expect_check().returning(|_| RuleVerdict::Allow);
    let mut market = MockMarketGateway::new();
    market.expect_post_bid().times(1).returning(|_| Ok(()));
    market.expect_await_acceptance().returning(|_| Ok(None));

    let agent = BrokerAgent::new(
        seed,
        fc,
        rules,
        market,
        MockTradeEvaluator::new(),
        st,
        MockEconomyLedger::new(),
        MockWitnessChain::new(),
        MockConsensusGateway::new(),
    );
    assert_eq!(agent.tick().await.unwrap(), TickReport::Posted);
}

// 5 — HEADLINE: a rule Violation short-circuits — execute NEVER called, one RuleRejection witness (FR-3.7).
#[tokio::test]
async fn rule_violation_short_circuits() {
    let (seed, fc, st) = monitor_reason(100.0, 0.0); // would be an Offer
    let mut rules = MockRulesEngine::new();
    rules
        .expect_check()
        .times(1)
        .returning(|_| RuleVerdict::Violation(RuleId::MaxDailyThermalTransferKwh));
    let mut market = MockMarketGateway::new();
    market.expect_post_offer().times(0); // never posted
    market.expect_execute().times(0); // NEVER executed
    let mut witness = MockWitnessChain::new();
    witness
        .expect_emit()
        .times(1)
        .withf(|a| matches!(a, PrivilegedAction::RuleRejection { .. }))
        .returning(|_| Ok(witness_ok()));

    let agent = BrokerAgent::new(
        seed,
        fc,
        rules,
        market,
        MockTradeEvaluator::new(),
        st,
        MockEconomyLedger::new(),
        witness,
        MockConsensusGateway::new(),
    );
    assert_eq!(
        agent.tick().await.unwrap(),
        TickReport::Rejected(RuleId::MaxDailyThermalTransferKwh)
    );
}

/// Configure the remaining collaborators for a full execute path (rules already Allow, acceptance
/// Some). Witness/ledger/evaluator/strategy-update are set with their happy-path cardinalities.
fn arm_execute_tail(
    witness: &mut MockWitnessChain,
    ledger: &mut MockEconomyLedger,
    evaluator: &mut MockTradeEvaluator,
) {
    witness
        .expect_emit()
        .times(3)
        .returning(|_| Ok(witness_ok())); // TradeSigned, Exec, Routing
    ledger.expect_debit_credit().times(1).returning(|_| Ok(()));
    evaluator
        .expect_evaluate()
        .times(1)
        .returning(|_| Reflection {
            value_ratio: 1.0,
            note: "ok".into(),
        });
}

// 6 — rules.check precedes market.execute (FR-3.7).
#[tokio::test]
async fn rules_checked_before_execute() {
    let mut seq = Sequence::new();
    let (seed, fc, mut st) = monitor_reason(100.0, 0.0);
    st.expect_update().times(1).returning(|_: Adaptation| ());
    let mut rules = MockRulesEngine::new();
    rules
        .expect_check()
        .times(1)
        .in_sequence(&mut seq)
        .returning(|_| RuleVerdict::Allow);
    let mut market = MockMarketGateway::new();
    market.expect_post_offer().returning(|_| Ok(()));
    market
        .expect_await_acceptance()
        .returning(|_| Ok(Some(accepted())));
    market
        .expect_execute()
        .times(1)
        .in_sequence(&mut seq) // execute AFTER check
        .returning(|_| Ok(()));
    market.expect_route().returning(|_, _| Ok(()));
    let mut consensus = MockConsensusGateway::new();
    consensus.expect_finalize().returning(|_| {
        Ok(Finality {
            reached_at: Timestamp(0),
        })
    });
    let (mut witness, mut ledger, mut evaluator) = (
        MockWitnessChain::new(),
        MockEconomyLedger::new(),
        MockTradeEvaluator::new(),
    );
    arm_execute_tail(&mut witness, &mut ledger, &mut evaluator);

    let agent = BrokerAgent::new(
        seed, fc, rules, market, evaluator, st, ledger, witness, consensus,
    );
    assert!(matches!(
        agent.tick().await.unwrap(),
        TickReport::Executed { .. }
    ));
}

// 7 — consensus.finalize precedes execute precedes route (FR-3.3/5.1).
#[tokio::test]
async fn consensus_before_routing() {
    let mut seq = Sequence::new();
    let (seed, fc, mut st) = monitor_reason(100.0, 0.0);
    st.expect_update().times(1).returning(|_: Adaptation| ());
    let mut rules = MockRulesEngine::new();
    rules.expect_check().returning(|_| RuleVerdict::Allow);
    let mut consensus = MockConsensusGateway::new();
    consensus
        .expect_finalize()
        .times(1)
        .in_sequence(&mut seq)
        .returning(|_| {
            Ok(Finality {
                reached_at: Timestamp(0),
            })
        });
    let mut market = MockMarketGateway::new();
    market.expect_post_offer().returning(|_| Ok(()));
    market
        .expect_await_acceptance()
        .returning(|_| Ok(Some(accepted())));
    market
        .expect_execute()
        .times(1)
        .in_sequence(&mut seq) // execute AFTER finalize
        .returning(|_| Ok(()));
    market
        .expect_route()
        .times(1)
        .in_sequence(&mut seq) // route AFTER execute
        .returning(|_, _| Ok(()));
    let (mut witness, mut ledger, mut evaluator) = (
        MockWitnessChain::new(),
        MockEconomyLedger::new(),
        MockTradeEvaluator::new(),
    );
    arm_execute_tail(&mut witness, &mut ledger, &mut evaluator);

    let agent = BrokerAgent::new(
        seed, fc, rules, market, evaluator, st, ledger, witness, consensus,
    );
    assert!(matches!(
        agent.tick().await.unwrap(),
        TickReport::Executed { .. }
    ));
}

// 8 — an executed trade emits three witnesses: TradeSigned, Exec, Routing (FR-4.3 / AC-7).
#[tokio::test]
async fn executed_trade_emits_three_witnesses() {
    let (seed, fc, mut st) = monitor_reason(100.0, 0.0);
    st.expect_update().returning(|_: Adaptation| ());
    let mut rules = MockRulesEngine::new();
    rules.expect_check().returning(|_| RuleVerdict::Allow);
    let mut market = MockMarketGateway::new();
    market.expect_post_offer().returning(|_| Ok(()));
    market
        .expect_await_acceptance()
        .returning(|_| Ok(Some(accepted())));
    market.expect_execute().returning(|_| Ok(()));
    market.expect_route().returning(|_, _| Ok(()));
    let mut consensus = MockConsensusGateway::new();
    consensus.expect_finalize().returning(|_| {
        Ok(Finality {
            reached_at: Timestamp(0),
        })
    });
    let mut evaluator = MockTradeEvaluator::new();
    evaluator.expect_evaluate().returning(|_| Reflection {
        value_ratio: 1.0,
        note: "ok".into(),
    });
    let mut ledger = MockEconomyLedger::new();
    ledger.expect_debit_credit().returning(|_| Ok(()));
    let mut witness = MockWitnessChain::new();
    witness
        .expect_emit()
        .withf(|a| matches!(a, PrivilegedAction::TradeSigned))
        .times(1)
        .returning(|_| Ok(witness_ok()));
    witness
        .expect_emit()
        .withf(|a| matches!(a, PrivilegedAction::Exec))
        .times(1)
        .returning(|_| Ok(witness_ok()));
    witness
        .expect_emit()
        .withf(|a| matches!(a, PrivilegedAction::Routing))
        .times(1)
        .returning(|_| Ok(witness_ok()));

    let agent = BrokerAgent::new(
        seed, fc, rules, market, evaluator, st, ledger, witness, consensus,
    );
    assert!(matches!(
        agent.tick().await.unwrap(),
        TickReport::Executed { .. }
    ));
}

// 9 — credits accounted exactly once on execute (FR-3.6 / ADR-0013).
#[tokio::test]
async fn credits_accounted_on_execute() {
    let (seed, fc, mut st) = monitor_reason(100.0, 0.0);
    st.expect_update().returning(|_: Adaptation| ());
    let mut rules = MockRulesEngine::new();
    rules.expect_check().returning(|_| RuleVerdict::Allow);
    let mut market = MockMarketGateway::new();
    market.expect_post_offer().returning(|_| Ok(()));
    market
        .expect_await_acceptance()
        .returning(|_| Ok(Some(accepted())));
    market.expect_execute().returning(|_| Ok(()));
    market.expect_route().returning(|_, _| Ok(()));
    let mut consensus = MockConsensusGateway::new();
    consensus.expect_finalize().returning(|_| {
        Ok(Finality {
            reached_at: Timestamp(0),
        })
    });
    let mut evaluator = MockTradeEvaluator::new();
    evaluator.expect_evaluate().returning(|_| Reflection {
        value_ratio: 1.0,
        note: "ok".into(),
    });
    let mut witness = MockWitnessChain::new();
    witness
        .expect_emit()
        .times(3)
        .returning(|_| Ok(witness_ok()));
    let mut ledger = MockEconomyLedger::new();
    ledger.expect_debit_credit().times(1).returning(|_| Ok(())); // exactly once

    let agent = BrokerAgent::new(
        seed, fc, rules, market, evaluator, st, ledger, witness, consensus,
    );
    assert!(matches!(
        agent.tick().await.unwrap(),
        TickReport::Executed { .. }
    ));
}

// 10 — Reflect then Adapt: evaluate precedes strategy.update (FR-3.4/3.5).
#[tokio::test]
async fn reflect_then_adapt() {
    let mut seq = Sequence::new();
    let mut seed = MockSeedMesh::new();
    seed.expect_read_surplus_deficit()
        .returning(|| Ok(balance(100.0, 0.0)));
    let mut fc = MockForecastService::new();
    fc.expect_demand_24h().returning(|_| Ok(forecast()));
    let mut st = MockStrategyStore::new();
    st.expect_load().returning(strategy);
    let mut rules = MockRulesEngine::new();
    rules.expect_check().returning(|_| RuleVerdict::Allow);
    let mut market = MockMarketGateway::new();
    market.expect_post_offer().returning(|_| Ok(()));
    market
        .expect_await_acceptance()
        .returning(|_| Ok(Some(accepted())));
    market.expect_execute().returning(|_| Ok(()));
    market.expect_route().returning(|_, _| Ok(()));
    let mut consensus = MockConsensusGateway::new();
    consensus.expect_finalize().returning(|_| {
        Ok(Finality {
            reached_at: Timestamp(0),
        })
    });
    let mut witness = MockWitnessChain::new();
    witness
        .expect_emit()
        .times(3)
        .returning(|_| Ok(witness_ok()));
    let mut ledger = MockEconomyLedger::new();
    ledger.expect_debit_credit().returning(|_| Ok(()));
    let mut evaluator = MockTradeEvaluator::new();
    evaluator
        .expect_evaluate()
        .times(1)
        .in_sequence(&mut seq)
        .returning(|_| Reflection {
            value_ratio: 1.0,
            note: "ok".into(),
        });
    st.expect_update()
        .times(1)
        .in_sequence(&mut seq) // update AFTER evaluate
        .returning(|_: Adaptation| ());

    let agent = BrokerAgent::new(
        seed, fc, rules, market, evaluator, st, ledger, witness, consensus,
    );
    assert!(matches!(
        agent.tick().await.unwrap(),
        TickReport::Executed { .. }
    ));
}

// 11 — no counterparty ⇒ posts only, no execute/route, returns Posted (FR-3.3).
#[tokio::test]
async fn no_counterparty_posts_only() {
    let (seed, fc, st) = monitor_reason(100.0, 0.0);
    let mut rules = MockRulesEngine::new();
    rules.expect_check().returning(|_| RuleVerdict::Allow);
    let mut market = MockMarketGateway::new();
    market.expect_post_offer().returning(|_| Ok(()));
    market.expect_await_acceptance().returning(|_| Ok(None)); // none accepted
    market.expect_execute().times(0);
    market.expect_route().times(0);

    let agent = BrokerAgent::new(
        seed,
        fc,
        rules,
        market,
        MockTradeEvaluator::new(),
        st,
        MockEconomyLedger::new(),
        MockWitnessChain::new(),
        MockConsensusGateway::new(),
    );
    assert_eq!(agent.tick().await.unwrap(), TickReport::Posted);
}
