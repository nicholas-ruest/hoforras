//! # hoforras-ports
//!
//! The hexagon edges (ADR-0002): the ~30 port traits that domain units depend on and adapters
//! implement. Defined **only** over `hoforras-domain` types — no Ruv-crate dependency (ADR-0003),
//! enforced by `scripts/layering-lint.sh`.
//!
//! Every trait is annotated with [`mockall::automock`] under `cfg(test)` or the `mock` feature, so
//! domain units can be unit-tested against mocks (London-School TDD, ADR-0001). Async, I/O-bound
//! ports use `async-trait`; pure / in-process ports (crypto, RVM coherence per ADR-0004, governance)
//! stay synchronous.

pub mod ai;
pub mod broker;
pub mod consensus;
pub mod inference;
pub mod memory;
pub mod mesh;
pub mod security;
pub mod sensor;
pub mod time;

#[cfg(test)]
mod port_mock_tests;

/// Result type returned by fallible ports.
pub type PortResult<T> = Result<T, hoforras_domain::DomainError>;

// Convenience re-exports of every port trait.
pub use ai::{ClaudeReasoner, McpToolRegistry, OperatorChannel};
pub use broker::{
    EconomyLedger, GradientAggregator, MarketGateway, RulesEngine, SeedMesh, StrategyStore,
    TradeEvaluator,
};
pub use consensus::{DagNetwork, MlDsaSigner, PeerDiscovery};
pub use inference::{ForecastService, InferenceAgent, SwarmFactory};
pub use memory::{GnnEngine, VectorIndex};
pub use mesh::{MeshTransport, NodeRegistry};
pub use security::{CapabilityGate, MincutEngine, PartitionController, WitnessChain};
pub use sensor::{
    EventEmitter, QualityScorer, RejectionLog, SensorSource, Validator, WitnessSigner,
};
pub use time::Clock;
