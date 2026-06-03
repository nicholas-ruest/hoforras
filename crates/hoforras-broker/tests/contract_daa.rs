//! Contract tests for the reference `daa-*` adapters (R3 governance + ADR-0013 credits + ADR-0005 BFT).
//!
//! These pin the real adapter behaviour with no mocks: the governance hard limits (research §78–82),
//! thermal-credit conservation, and Byzantine-robust gradient aggregation.

use hoforras_broker::adapters::{DaaEconomyAdapter, DaaRulesAdapter, PrimeCoordinatorAdapter};
use hoforras_domain::{
    AcceptedTrade, ExecutedTrade, Gradient, JunctionId, Kwh, NodeId, Price, ProposedTrade, RuleId,
    RuleVerdict, ThermalTradeAgreement, Timestamp, TradeWindow,
};
use hoforras_ports::broker::{EconomyLedger, GradientAggregator, RulesEngine};

fn node(s: &str) -> NodeId {
    NodeId::new(format!("{s}.thermal.budapest.dark")).unwrap()
}

fn proposed(kwh: f32) -> ProposedTrade {
    ProposedTrade {
        seller: node("a"),
        buyer: node("b"),
        kwh: Kwh(kwh),
        price: Price {
            credit_per_kwh: 2.0,
        },
        window: TradeWindow::new(6).unwrap(),
        pipe_route: vec![JunctionId(1)],
    }
}

// ───────────────────────── R3 — governance hard limits (FR-3.7) ─────────────────────────

#[test]
fn rejects_over_500kwh_daily() {
    let rules = DaaRulesAdapter::new();
    assert_eq!(
        rules.check(&proposed(600.0)),
        RuleVerdict::Violation(RuleId::MaxDailyThermalTransferKwh)
    );
}

#[test]
fn rejects_below_15pct_reserve() {
    let rules = DaaRulesAdapter::with_node_state(0.10, 6.0); // reserve below floor
    assert_eq!(
        rules.check(&proposed(40.0)),
        RuleVerdict::Violation(RuleId::MinSafetyReservePercent)
    );
}

#[test]
fn rejects_over_6bar() {
    let rules = DaaRulesAdapter::with_node_state(0.15, 7.0); // pressure over ceiling
    assert_eq!(
        rules.check(&proposed(40.0)),
        RuleVerdict::Violation(RuleId::PipePressureCeilingBar)
    );
}

#[test]
fn rejects_over_6h_window() {
    // The 6h ceiling is enforced at the type boundary: a >6h window cannot even be constructed,
    // so a `ProposedTrade` can never carry one (defense in depth in the rules engine too).
    assert!(TradeWindow::new(7).is_err());
}

#[test]
fn accepts_within_all_limits() {
    let rules = DaaRulesAdapter::new();
    assert_eq!(rules.check(&proposed(40.0)), RuleVerdict::Allow);
}

#[test]
fn unavailable_engine_fails_closed() {
    // ADR-0015: an unavailable engine denies everything.
    let rules = DaaRulesAdapter::unavailable();
    assert!(matches!(
        rules.check(&proposed(40.0)),
        RuleVerdict::Violation(_)
    ));
}

// ───────────────────────── ADR-0013 — thermal-credit conservation (FR-3.6) ─────────────────────────

fn executed(seller: &str, buyer: &str, kwh: f32, price: f32) -> ExecutedTrade {
    let proposed = ProposedTrade {
        seller: node(seller),
        buyer: node(buyer),
        kwh: Kwh(kwh),
        price: Price {
            credit_per_kwh: price,
        },
        window: TradeWindow::new(6).unwrap(),
        pipe_route: vec![JunctionId(1)],
    };
    ExecutedTrade {
        accepted: AcceptedTrade {
            proposed,
            accepted_at: Timestamp(0),
            pipe_route: vec![JunctionId(1)],
        },
        executed_at: Timestamp(0),
    }
}

#[tokio::test]
async fn credits_are_conserved_across_trades() {
    let ledger = DaaEconomyAdapter::new();
    ledger
        .debit_credit(&executed("a", "b", 40.0, 2.3))
        .await
        .unwrap();
    ledger
        .debit_credit(&executed("a", "c", 10.0, 1.5))
        .await
        .unwrap();
    ledger
        .debit_credit(&executed("c", "b", 5.0, 3.0))
        .await
        .unwrap();

    // Seller earns, buyer pays the same amount — the system total is always zero.
    assert!(
        ledger.total().abs() < 1e-3,
        "credits must be conserved, got {}",
        ledger.total()
    );
    // 'a' sold 40*2.3 + 10*1.5 = 107 credits.
    assert!((ledger.balance_of("a.thermal.budapest.dark") - 107.0).abs() < 1e-3);
}

#[allow(dead_code)]
fn _agreement_type_is_reachable(_: ThermalTradeAgreement) {}

// ───────────────────────── ADR-0005 — Byzantine-robust aggregation (FR-3.8) ─────────────────────────

#[tokio::test]
async fn bft_aggregation_trims_outliers() {
    let coordinator = PrimeCoordinatorAdapter::new();
    // Four honest gradients near 1.0, one Byzantine spike at 1000.0 — the trimmed mean drops the
    // extreme high (and low) so the result stays near the honest cluster.
    let gradients = vec![
        Gradient::from_local(vec![1.0]),
        Gradient::from_local(vec![1.1]),
        Gradient::from_local(vec![0.9]),
        Gradient::from_local(vec![1.0]),
        Gradient::from_local(vec![1000.0]), // Byzantine
    ];
    let aggregated = coordinator.aggregate(gradients).await.unwrap();
    assert!(
        aggregated.as_slice()[0] < 2.0,
        "trimmed mean should reject the Byzantine outlier, got {}",
        aggregated.as_slice()[0]
    );
}

#[tokio::test]
async fn aggregation_rejects_mismatched_dimensions() {
    let coordinator = PrimeCoordinatorAdapter::new();
    let gradients = vec![
        Gradient::from_local(vec![1.0, 2.0]),
        Gradient::from_local(vec![1.0]), // wrong dim
    ];
    assert!(coordinator.aggregate(gradients).await.is_err());
}
