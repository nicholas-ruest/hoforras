//! # Forecasting & Anomaly (DDD-03)
//!
//! Node-local predictions the market reasons over — anomalies, pipe-burst precursors, and 24-hour
//! demand — produced by **on-device** inference (ADR-0012) inside the node's RVM boundary
//! (ADR-0009).
//!
//! * [`ForecastServiceImpl`] — the domain unit. Each of `anomalies`, `burst_precursor`,
//!   `demand_24h` runs through a `with_agent` guard that spawns an ephemeral specialist, infers
//!   once, and **always** dissolves (FR-2.1 — spawn count == dissolve count across all outcomes).
//! * [`RuvSwarmAdapter`] / [`NeuroDivergentAdapter`] — reference ruv-swarm / ruv-FANN adapters that
//!   host the LSTM/N-BEATS reference network in-process. There is intentionally **no**
//!   `RemoteInferenceClient` port: edge-only is enforced by the absence of that seam (ADR-0012).

pub mod adapters;
pub mod service;

pub use adapters::{ForecastTask, NeuroDivergentAdapter, RuvSwarmAdapter};
pub use service::{ForecastServiceImpl, SPEC_ANOMALY, SPEC_BURST, SPEC_DEMAND};
