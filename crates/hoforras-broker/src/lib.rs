//! # hoforras-broker — Thermal Market & Brokerage (DDD-01, the CORE domain)
//!
//! The autonomous market participant — the heart of the invention and the spine of AC-7.
//!
//! * [`agent::BrokerAgent`] — the MRAP application service (Monitor→Reason→Act→Reflect→Adapt) with
//!   strict ordering: governance **before** execute (FR-3.7 / ADR-0015, fail-closed), consensus
//!   **before** execute/route (FR-5.1), three witnesses per executed trade (FR-4.3), thermal-credit
//!   accounting (ADR-0013), and one tick in flight per node (ADR-0006).
//! * [`decide`] — the pure reasoning kernel + `PricingService` (example-tested).
//! * [`federated::FederatedTrainer`] — gradient-only district training (ADR-0005, compile-time wall).
//! * [`adapters`] — reference `daa-*` / SEED adapters behind the broker ports.

pub mod adapters;
pub mod agent;
pub mod decide;
pub mod federated;

pub use adapters::{
    DaaEconomyAdapter, DaaOrchestratorAdapter, DaaRulesAdapter, PrimeCoordinatorAdapter,
    SeedMeshAdapter,
};
pub use agent::{BrokerAgent, TickReport};
pub use decide::{decide, PricingService};
pub use federated::FederatedTrainer;
