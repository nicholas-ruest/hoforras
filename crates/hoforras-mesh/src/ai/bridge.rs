//! `ThermalBridgeServer` (DDD-08 / FR-8.1) — the ThermalSense-Bridge MCP server, an Open Host
//! Service (ADR-0002).
//!
//! `boot()` registers **exactly five** tools — the published OHS API. `invoke()` routes each tool to
//! **exactly one** backing service; an unknown tool returns a typed error with **no** delegation. The
//! tool names are the stable contract Claude and the dashboard consume.

use hoforras_domain::{DomainError, Json};
use hoforras_ports::ai::{
    AnomalyQueryService, DistrictStatusService, ForecastQueryService, McpToolRegistry,
    PipeHealthService, TradeQueryService,
};
use hoforras_ports::PortResult;

/// The five — and only five — MCP tools the bridge publishes (FR-8.1).
pub const TOOL_NAMES: [&str; 5] = [
    "hoforras.thermal.district_status",
    "hoforras.trade.active_agreements",
    "hoforras.pipe.health_scores",
    "hoforras.anomaly.recent",
    "hoforras.forecast.demand_24h",
];

/// The Open Host Service. Generic over the registry and its five backing services (London-School,
/// ADR-0001).
pub struct ThermalBridgeServer<R, ST, TR, PI, AN, FO> {
    registry: R,
    status: ST,
    trade: TR,
    pipe: PI,
    anomaly: AN,
    forecast: FO,
}

impl<R, ST, TR, PI, AN, FO> ThermalBridgeServer<R, ST, TR, PI, AN, FO>
where
    R: McpToolRegistry,
    ST: DistrictStatusService,
    TR: TradeQueryService,
    PI: PipeHealthService,
    AN: AnomalyQueryService,
    FO: ForecastQueryService,
{
    pub fn new(registry: R, status: ST, trade: TR, pipe: PI, anomaly: AN, forecast: FO) -> Self {
        Self {
            registry,
            status,
            trade,
            pipe,
            anomaly,
            forecast,
        }
    }

    /// Register exactly the five published tools (FR-8.1). Idempotent registration of the OHS API.
    pub async fn boot(&self) -> PortResult<()> {
        for name in TOOL_NAMES {
            self.registry.register(name.to_string()).await?;
        }
        Ok(())
    }

    /// Invoke a tool by name. An unknown tool returns `Err` **without** touching any backing service
    /// (FR-8.1); a known tool delegates to exactly its one backing service.
    pub async fn invoke(&self, name: &str, args: Json) -> PortResult<Json> {
        if !self.registry.has(name) {
            return Err(DomainError::NotFound(format!("unknown tool: {name}")));
        }
        match name {
            "hoforras.thermal.district_status" => self.status.snapshot().await,
            "hoforras.trade.active_agreements" => self.trade.active().await,
            "hoforras.pipe.health_scores" => self.pipe.scores().await,
            "hoforras.anomaly.recent" => self.anomaly.recent(args).await,
            "hoforras.forecast.demand_24h" => self.forecast.demand(args).await,
            // `has()` returned true for a name the bridge does not route — registry/bridge drift.
            other => Err(DomainError::NotFound(format!("unrouted tool: {other}"))),
        }
    }
}
