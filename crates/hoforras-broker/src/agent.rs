//! `BrokerAgent` — the MRAP application service (DDD-01 / FR-3.1–3.9), the keystone of AC-7.
//!
//! One `tick()` is one Monitor→Reason→Act→Reflect→Adapt cycle with a **strict ordering** that is the
//! spine of the whole system:
//!
//! ```text
//! monitor → reason → rules.check → [act] → consensus.finalize → execute → route → witness* → reflect → adapt
//! ```
//!
//! Invariants enforced here:
//! * **Single-writer (ADR-0006):** at most one tick in flight per node — a `tokio::Mutex` serializes
//!   ticks, so the FR-3.7 ordering can never be interleaved across ticks.
//! * **Governance-before-execute, fail-closed (FR-3.7 / ADR-0015):** `rules.check` runs before any
//!   `execute`; a `Violation` (including a fail-closed rules engine that denies) emits a witnessed
//!   rejection and performs **zero** executions.
//! * **Consensus-before-routing (FR-5.1):** `consensus.finalize` precedes `execute` and `route`.
//! * **Audit completeness (FR-4.3 / AC-7):** an executed trade emits witnesses for
//!   `{TradeSigned, Exec, Routing}`.
//! * **Thermal credits (ADR-0013):** executed trades are accounted in the `EconomyLedger`.

use hoforras_domain::ReadingWindow;
use hoforras_domain::{
    Decision, ExecutedTrade, PrivilegedAction, RuleVerdict, ThermalTradeAgreement, Timestamp,
    TradeOutcome,
};
use hoforras_ports::broker::{
    EconomyLedger, MarketGateway, RulesEngine, SeedMesh, StrategyStore, TradeEvaluator,
};
use hoforras_ports::consensus::ConsensusGateway;
use hoforras_ports::inference::ForecastService;
use hoforras_ports::security::WitnessChain;
use hoforras_ports::PortResult;

use crate::decide::{decide, to_adaptation, to_proposed};

const HOUR_NS: u64 = 3_600_000_000_000;

/// The outcome of one MRAP tick.
#[derive(Clone, Debug, PartialEq)]
pub enum TickReport {
    /// `decide()` chose `Hold` — nothing posted.
    Idle,
    /// Posted an offer/bid but no counterparty accepted this tick.
    Posted,
    /// Governance rejected the trade (or the rules engine failed closed). Witnessed; no execution.
    Rejected(hoforras_domain::RuleId),
    /// A trade was finalized, executed, routed, accounted, and reflected on.
    Executed {
        reflection: hoforras_domain::Reflection,
    },
}

/// The autonomous market participant for one building (DDD-01 aggregate root, single-writer).
#[allow(clippy::too_many_arguments)]
pub struct BrokerAgent<SM, FC, RE, MG, TE, ST, EL, WC, CG> {
    seed_mesh: SM,
    forecast: FC,
    rules: RE,
    market: MG,
    evaluator: TE,
    strategy: ST,
    ledger: EL,
    witness: WC,
    consensus: CG,
    /// ADR-0006: serializes ticks so at most one is in flight per node.
    tick_lock: tokio::sync::Mutex<()>,
}

impl<SM, FC, RE, MG, TE, ST, EL, WC, CG> BrokerAgent<SM, FC, RE, MG, TE, ST, EL, WC, CG>
where
    SM: SeedMesh,
    FC: ForecastService,
    RE: RulesEngine,
    MG: MarketGateway,
    TE: TradeEvaluator,
    ST: StrategyStore,
    EL: EconomyLedger,
    WC: WitnessChain,
    CG: ConsensusGateway,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        seed_mesh: SM,
        forecast: FC,
        rules: RE,
        market: MG,
        evaluator: TE,
        strategy: ST,
        ledger: EL,
        witness: WC,
        consensus: CG,
    ) -> Self {
        Self {
            seed_mesh,
            forecast,
            rules,
            market,
            evaluator,
            strategy,
            ledger,
            witness,
            consensus,
            tick_lock: tokio::sync::Mutex::new(()),
        }
    }

    /// Run exactly one MRAP cycle. Holds the single-writer lock for its whole duration (ADR-0006).
    pub async fn tick(&self) -> PortResult<TickReport> {
        let _guard = self.tick_lock.lock().await; // ADR-0006: one tick in flight per node

        // ---- MONITOR (FR-3.1) ----
        let balance = self.seed_mesh.read_surplus_deficit().await?;
        // The node-local forecaster holds the actual sensor history (P3); this descriptor names the
        // node + window it should forecast 24h demand over.
        let window = ReadingWindow {
            node_id: balance.node_id.clone(),
            from_ts: Timestamp(0),
            to_ts: Timestamp(balance.window.hours() as u64 * HOUR_NS),
            frames: vec![],
        };
        let forecast = self.forecast.demand_24h(window).await?;

        // ---- REASON (FR-3.2) ----
        let strategy = self.strategy.load();
        let decision = decide(&balance, &forecast, &strategy);
        let proposed = match to_proposed(&decision) {
            Some(p) => p,
            None => return Ok(TickReport::Idle), // Hold
        };

        // ---- GOVERNANCE GATE — HARD, BEFORE ANY EXECUTE (FR-3.7 / ADR-0015) ----
        if let RuleVerdict::Violation(rule) = self.rules.check(&proposed) {
            // Fail-closed: a violation (or an unavailable engine that denies) ⇒ witnessed rejection,
            // ZERO executions.
            self.witness
                .emit(PrivilegedAction::RuleRejection { rule })?;
            return Ok(TickReport::Rejected(rule));
        }

        // ---- ACT (FR-3.3) — only reached after governance Allow ----
        match &decision {
            Decision::Offer(o) => self.market.post_offer(o.clone()).await?,
            Decision::Bid(b) => self.market.post_bid(b.clone()).await?,
            Decision::Hold => unreachable!("Hold returned Idle above"),
        }

        let accepted = match self.market.await_acceptance(&decision).await? {
            Some(a) => a,
            None => return Ok(TickReport::Posted),
        };

        // ---- CONSENSUS BEFORE EXECUTE/ROUTE (FR-5.1) ----
        let agreement = to_agreement(&accepted);
        let _finality = self.consensus.finalize(agreement).await?;
        self.witness.emit(PrivilegedAction::TradeSigned)?; // witness 1/3

        // ---- EXECUTE + ROUTE, each witnessed (FR-4.3 / AC-7) ----
        self.market.execute(&accepted).await?;
        self.witness.emit(PrivilegedAction::Exec)?; // witness 2/3
        self.market.route(&accepted, &accepted.pipe_route).await?;
        self.witness.emit(PrivilegedAction::Routing)?; // witness 3/3

        // ---- ACCOUNT in thermal credits (FR-3.6 / ADR-0013) ----
        let executed = ExecutedTrade {
            accepted: accepted.clone(),
            executed_at: accepted.accepted_at,
        };
        self.ledger.debit_credit(&executed).await?;

        // ---- REFLECT (FR-3.4) ----
        let delivered = accepted.proposed.kwh.0;
        let outcome = TradeOutcome {
            executed,
            delivered_kwh: delivered,
            expected_kwh: delivered,
        };
        let reflection = self.evaluator.evaluate(&outcome);

        // ---- ADAPT (FR-3.5) ----
        self.strategy.update(to_adaptation(&reflection));

        Ok(TickReport::Executed { reflection })
    }
}

/// Map an accepted trade to the consensus context's `ThermalTradeAgreement` (DDD-01 → DDD-05 ACL).
fn to_agreement(accepted: &hoforras_domain::AcceptedTrade) -> ThermalTradeAgreement {
    let p = &accepted.proposed;
    ThermalTradeAgreement {
        seller: p.seller.clone(),
        buyer: p.buyer.clone(),
        kwh_offered: p.kwh.0,
        duration_hours: p.window.hours(),
        credit_price_per_kwh: p.price.credit_per_kwh,
        pipe_route: accepted.pipe_route.clone(),
        valid_from: accepted.accepted_at,
    }
}
