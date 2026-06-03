//! Reference on-device adapters for the Forecasting & Anomaly ports (DDD-03).
//!
//! The real ruv-swarm / ruv-FANN substrate is unavailable here, so these are **reference**
//! implementations behind the `hoforras-ports` inference traits — swappable per ADR-0001. Both run
//! in-process on the Appliance (node-local, ADR-0009; edge-only, ADR-0012): there is no remote
//! inference client anywhere in this module.

pub mod neuro_divergent;
pub mod ruv_swarm;

pub use neuro_divergent::{ForecastTask, NeuroDivergentAdapter};
pub use ruv_swarm::RuvSwarmAdapter;
