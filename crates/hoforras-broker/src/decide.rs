//! The pure decision core (DDD-01 §4 / FR-3.2) — no I/O, example-tested by state.
//!
//! `decide` is the proprietary reasoning kernel of the autonomous market: given the building's
//! thermal balance, a demand forecast, and the current strategy, it returns `Offer | Bid | Hold`.
//! It is the one place classicist (state-based) testing is used, because it is a pure function.
//! `PricingService` is likewise pure. Nothing here performs governance enforcement — that is the
//! broker's hard gate (ADR-0015); these functions only *propose*.

use hoforras_domain::{
    Adaptation, Bid, Decision, DemandForecast, Kwh, Offer, Price, ProposedTrade, Reflection,
    Strategy, ThermalBalance, TradeWindow, GOVERNANCE,
};

/// Pure pricing from strategy + forecast (DDD-01 §4). Higher forecast demand pressure ⇒ higher
/// price (sellers ask more, buyers will pay more under scarcity).
pub struct PricingService;

impl PricingService {
    pub fn price_offer(strategy: &Strategy, forecast: &DemandForecast) -> Price {
        let scarcity = demand_pressure(forecast);
        Price {
            credit_per_kwh: strategy.base_credit_per_kwh * (1.0 + 0.5 * scarcity),
        }
    }

    pub fn price_bid(strategy: &Strategy, forecast: &DemandForecast) -> Price {
        let scarcity = demand_pressure(forecast);
        Price {
            credit_per_kwh: strategy.base_credit_per_kwh * (1.0 + scarcity),
        }
    }
}

/// Demand pressure in [0, 1]: how peaky the next-24h forecast is (peak above mean / peak).
fn demand_pressure(forecast: &DemandForecast) -> f32 {
    if forecast.hourly_kwh.is_empty() {
        return 0.0;
    }
    let max = forecast.hourly_kwh.iter().copied().fold(0.0f32, f32::max);
    let mean = forecast.hourly_kwh.iter().sum::<f32>() / forecast.hourly_kwh.len() as f32;
    if max <= 0.0 {
        0.0
    } else {
        ((max - mean) / max).clamp(0.0, 1.0)
    }
}

/// The reserve the building must hold back before trading surplus (FR-3.2 / DDD-01 §4): a fraction
/// of forecast 24h demand.
fn projected_need(forecast: &DemandForecast, reserve_pct: f32) -> f32 {
    let total: f32 = forecast.hourly_kwh.iter().sum();
    total * reserve_pct
}

/// A trade window clamped to the governance ceiling (≤ 6h) and at least 1h. Infallible by
/// construction — the clamp keeps it inside `TradeWindow`'s valid range.
fn clamped_window(strategy: &Strategy) -> TradeWindow {
    let hours = strategy
        .window_hours
        .clamp(1, GOVERNANCE.trade_window_hours);
    TradeWindow::new(hours).expect("clamped window is within (0, ceiling]")
}

/// The pure reasoning kernel (FR-3.2). Net available = surplus − deficit − reserve. Beyond the
/// offer threshold ⇒ sell; below the negative bid threshold ⇒ buy; otherwise hold.
pub fn decide(
    balance: &ThermalBalance,
    forecast: &DemandForecast,
    strategy: &Strategy,
) -> Decision {
    let net =
        balance.surplus_kwh - balance.deficit_kwh - projected_need(forecast, strategy.reserve_pct);
    let window = clamped_window(strategy);

    if net > strategy.offer_threshold_kwh {
        Decision::Offer(Offer {
            seller: balance.node_id.clone(),
            kwh: Kwh(net),
            price: PricingService::price_offer(strategy, forecast),
            window,
        })
    } else if net < -strategy.bid_threshold_kwh {
        Decision::Bid(Bid {
            buyer: balance.node_id.clone(),
            kwh: Kwh(-net),
            max_price: PricingService::price_bid(strategy, forecast),
            window,
        })
    } else {
        Decision::Hold
    }
}

/// Lower a decision to the `ProposedTrade` the governance gate checks (FR-3.7). The counterparty is
/// a placeholder until acceptance (the rules check is over kWh/price/window, not the counterparty);
/// the pipe route is filled in by the accepted trade. `Hold` proposes nothing.
pub fn to_proposed(decision: &Decision) -> Option<ProposedTrade> {
    match decision {
        Decision::Offer(o) => Some(ProposedTrade {
            seller: o.seller.clone(),
            buyer: o.seller.clone(), // placeholder; bound at acceptance
            kwh: o.kwh,
            price: o.price,
            window: o.window,
            pipe_route: vec![],
        }),
        Decision::Bid(b) => Some(ProposedTrade {
            seller: b.buyer.clone(), // placeholder; bound at acceptance
            buyer: b.buyer.clone(),
            kwh: b.kwh,
            price: b.max_price,
            window: b.window,
            pipe_route: vec![],
        }),
        Decision::Hold => None,
    }
}

/// Map a reflection to a strategy adaptation (FR-3.5): a trade that delivered more value than
/// expected nudges thresholds to trade more readily, and vice-versa.
pub fn to_adaptation(reflection: &Reflection) -> Adaptation {
    let delta = (reflection.value_ratio - 1.0) * 0.1;
    Adaptation {
        offer_threshold_delta: -delta,
        bid_threshold_delta: -delta,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hoforras_domain::NodeId;

    fn balance(surplus: f32, deficit: f32) -> ThermalBalance {
        ThermalBalance {
            node_id: NodeId::new("pozsonyi14.thermal.budapest.dark").unwrap(),
            surplus_kwh: surplus,
            deficit_kwh: deficit,
            window: TradeWindow::new(6).unwrap(),
        }
    }

    fn forecast() -> DemandForecast {
        DemandForecast {
            hourly_kwh: vec![10.0; 24], // flat ⇒ no scarcity, total 240
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

    // reserve need = 240 * 0.15 = 36.

    #[test]
    fn large_surplus_offers() {
        // net = 100 - 0 - 36 = 64 > 5 ⇒ Offer 64
        match decide(&balance(100.0, 0.0), &forecast(), &strategy()) {
            Decision::Offer(o) => assert!((o.kwh.0 - 64.0).abs() < 1e-3),
            other => panic!("expected Offer, got {other:?}"),
        }
    }

    #[test]
    fn large_deficit_bids() {
        // net = 0 - 80 - 36 = -116 < -5 ⇒ Bid 116
        match decide(&balance(0.0, 80.0), &forecast(), &strategy()) {
            Decision::Bid(b) => assert!((b.kwh.0 - 116.0).abs() < 1e-3),
            other => panic!("expected Bid, got {other:?}"),
        }
    }

    #[test]
    fn balanced_holds() {
        // net = 40 - 0 - 36 = 4, within [-5, 5] ⇒ Hold
        assert_eq!(
            decide(&balance(40.0, 0.0), &forecast(), &strategy()),
            Decision::Hold
        );
    }

    #[test]
    fn window_is_clamped_to_ceiling() {
        let mut s = strategy();
        s.window_hours = 99;
        match decide(&balance(100.0, 0.0), &forecast(), &s) {
            Decision::Offer(o) => assert_eq!(o.window.hours(), GOVERNANCE.trade_window_hours),
            other => panic!("expected Offer, got {other:?}"),
        }
    }

    #[test]
    fn hold_proposes_nothing() {
        assert!(to_proposed(&Decision::Hold).is_none());
    }

    #[test]
    fn scarcity_raises_price() {
        let peaky = DemandForecast {
            hourly_kwh: (0..24).map(|h| if h == 12 { 100.0 } else { 1.0 }).collect(),
        };
        let flat = forecast();
        let s = strategy();
        assert!(
            PricingService::price_offer(&s, &peaky).credit_per_kwh
                > PricingService::price_offer(&s, &flat).credit_per_kwh
        );
    }
}
