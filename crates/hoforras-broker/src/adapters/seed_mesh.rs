//! `SeedMeshAdapter` — the Monitor source over the local SEED mesh (DDD-01 / FR-3.1).
//!
//! Reads the building's aggregated surplus/deficit. **Only** the `ThermalBalance` aggregate crosses
//! this boundary — raw frames stay node-local (NFR-7 / ADR-0009). The real SEED-mesh substrate is
//! unavailable; this reference adapter returns a configured balance.

use async_trait::async_trait;
use hoforras_domain::ThermalBalance;
use hoforras_ports::broker::SeedMesh;
use hoforras_ports::PortResult;

/// Reference SEED-mesh monitor. Holds the current balance to report.
pub struct SeedMeshAdapter {
    balance: ThermalBalance,
}

impl SeedMeshAdapter {
    pub fn new(balance: ThermalBalance) -> Self {
        Self { balance }
    }
}

#[async_trait]
impl SeedMesh for SeedMeshAdapter {
    async fn read_surplus_deficit(&self) -> PortResult<ThermalBalance> {
        Ok(self.balance.clone())
    }
}
