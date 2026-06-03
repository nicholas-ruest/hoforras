//! AI Orchestration ports (DDD-08). Open Host Service: the MCP tool API + Claude reasoning.

use async_trait::async_trait;
use hoforras_domain::{AnomalyContext, Explanation, Json, UiEvent};

use crate::PortResult;

/// MCP tool registry exposing thermal tools to Claude (FR-8.1). `register` is the OHS publication
/// seam the bridge's `boot()` drives; `has`/`list_tools` are discovery; `invoke` is the adapter's
/// MCP-facing call (the bridge routes to backing services itself).
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
#[async_trait]
pub trait McpToolRegistry: Send + Sync {
    async fn register(&self, name: String) -> PortResult<()>;
    fn has(&self, name: &str) -> bool;
    fn list_tools(&self) -> Vec<String>;
    async fn invoke(&self, name: String, args: Json) -> PortResult<Json>;
}

/// Asks Claude for a natural-language anomaly explanation (FR-8.2).
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
#[async_trait]
pub trait ClaudeReasoner: Send + Sync {
    async fn explain(&self, context: AnomalyContext) -> PortResult<Explanation>;
}

/// Pushes UI events to the operator dashboard over the WebSocket (FR-9.2).
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
#[async_trait]
pub trait OperatorChannel: Send + Sync {
    async fn push(&self, event: UiEvent) -> PortResult<()>;
}

// ───────────────────── DDD-08 backing services (one per MCP tool, FR-8.1) ─────────────────────
//
// Each `ThermalBridgeServer` tool delegates to exactly one of these. They are query-only views over
// the core contexts (status/trade/pipe/anomaly/forecast), returning canonical `Json` for the OHS.

/// `hoforras.thermal.district_status` backing service.
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
#[async_trait]
pub trait DistrictStatusService: Send + Sync {
    async fn snapshot(&self) -> PortResult<Json>;
}

/// `hoforras.trade.active_agreements` backing service.
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
#[async_trait]
pub trait TradeQueryService: Send + Sync {
    async fn active(&self) -> PortResult<Json>;
}

/// `hoforras.pipe.health_scores` backing service.
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
#[async_trait]
pub trait PipeHealthService: Send + Sync {
    async fn scores(&self) -> PortResult<Json>;
}

/// `hoforras.anomaly.recent` backing service (`args.since`).
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
#[async_trait]
pub trait AnomalyQueryService: Send + Sync {
    async fn recent(&self, args: Json) -> PortResult<Json>;
}

/// `hoforras.forecast.demand_24h` backing service (`args.node`).
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
#[async_trait]
pub trait ForecastQueryService: Send + Sync {
    async fn demand(&self, args: Json) -> PortResult<Json>;
}
