//! `DaaEconomyAdapter` — the thermal-credit ledger over `daa-economy` (DDD-01 / ADR-0013).
//!
//! The unit of account is the **thermal credit**, denominated kWh-equivalent (ADR-0013, replacing
//! rUv). Each executed trade debits the buyer and credits the seller by `kwh × credit_per_kwh`, so
//! credits are **conserved** — the sum of all balances stays zero (FR-3.6). The real `daa-economy`
//! substrate is unavailable; this reference ledger keeps per-node balances in memory.

use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use hoforras_domain::ExecutedTrade;
use hoforras_ports::broker::EconomyLedger;
use hoforras_ports::PortResult;

/// Reference thermal-credit ledger. Balances are credit-denominated (kWh-equivalent).
#[derive(Default)]
pub struct DaaEconomyAdapter {
    balances: Mutex<HashMap<String, f32>>,
}

impl DaaEconomyAdapter {
    pub fn new() -> Self {
        Self::default()
    }

    /// The credit balance of a node (0 if it has never traded).
    pub fn balance_of(&self, node: &str) -> f32 {
        *self
            .balances
            .lock()
            .expect("ledger poisoned")
            .get(node)
            .unwrap_or(&0.0)
    }

    /// Total credits across all nodes — must stay 0 (conservation, FR-3.6).
    pub fn total(&self) -> f32 {
        self.balances
            .lock()
            .expect("ledger poisoned")
            .values()
            .sum()
    }
}

#[async_trait]
impl EconomyLedger for DaaEconomyAdapter {
    async fn debit_credit(&self, trade: &ExecutedTrade) -> PortResult<()> {
        let p = &trade.accepted.proposed;
        let amount = p.kwh.0 * p.price.credit_per_kwh; // thermal credits (kWh-equivalent)
        let mut balances = self.balances.lock().expect("ledger poisoned");
        *balances.entry(p.buyer.as_str().to_string()).or_insert(0.0) -= amount; // buyer pays
        *balances.entry(p.seller.as_str().to_string()).or_insert(0.0) += amount; // seller earns
        Ok(())
    }
}
