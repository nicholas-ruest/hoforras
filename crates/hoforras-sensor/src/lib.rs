//! # hoforras-sensor — Sensing & Ingestion context (DDD-02).
//!
//! Implements the ingestion boundary (FR-1.1..1.4): raw multi-source readings are validated,
//! quality-scored, Ed25519-witnessed (ADR-0014 canonical bytes) and emitted as [`hoforras_domain::ThermalFrame`]s.
//!
//! Architectural invariants:
//! - **Node-local** (ADR-0009): `RawReading` / `ThermalFrame` never cross a partition boundary.
//!   Only the [`hoforras_domain::ThermalBalance`] aggregate produced by
//!   [`aggregate::ThermalAggregator`] is exposed across a boundary.
//! - **Anti-corruption layer** (DDD-02): the rvcsi adapter translates @ruv/rvcsi's vocabulary into
//!   the thermal ubiquitous language; the real crate is swappable per ADR-0001.
//! - **Hexagonal** (ADR-0002): [`pipeline::IngestionPipeline`] is generic over the sensor ports and
//!   unit-tested against `mockall` mocks (London-School, ADR-0001).

pub mod adapters;
pub mod aggregate;
pub mod bus;
pub mod pipeline;

/// Marker proving the crate is wired into the workspace and can reach the ports layer.
pub const CONTEXT: &str = "sensing-ingestion";

pub use adapters::{Ed25519WitnessAdapter, RvcsiIngestAdapter, SimSensorAdapter};
pub use aggregate::ThermalAggregator;
pub use bus::TypedEventBus;
pub use pipeline::IngestionPipeline;
