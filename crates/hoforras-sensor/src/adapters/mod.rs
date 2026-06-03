//! Reference adapters for the Sensing & Ingestion ports (DDD-02 / ADR-0001 — swappable).

pub mod rvcsi;
pub mod sim_sensor;
pub mod witness_ed25519;

pub use rvcsi::RvcsiIngestAdapter;
pub use sim_sensor::SimSensorAdapter;
pub use witness_ed25519::Ed25519WitnessAdapter;
