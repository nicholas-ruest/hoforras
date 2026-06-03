//! Reference adapters for the Thermal Market ports (DDD-01 §7).
//!
//! The `daa-*` substrate (rules, economy, orchestrator, prime-coordinator) and the SEED mesh are
//! unavailable here, so these are reference implementations behind the `hoforras-ports` broker
//! traits — swappable per ADR-0001. Governance limits (ADR-0015) and thermal-credit accounting
//! (ADR-0013) are real domain logic; the BFT aggregation enforces gradient-only egress (ADR-0005).

pub mod daa_economy;
pub mod daa_orchestrator;
pub mod daa_rules;
pub mod prime_coordinator;
pub mod seed_mesh;

pub use daa_economy::DaaEconomyAdapter;
pub use daa_orchestrator::DaaOrchestratorAdapter;
pub use daa_rules::DaaRulesAdapter;
pub use prime_coordinator::PrimeCoordinatorAdapter;
pub use seed_mesh::SeedMeshAdapter;
