//! AI Orchestration ports (DDD-08). Open Host Service: the MCP tool API + Claude reasoning.

use async_trait::async_trait;
use hoforras_domain::{AnomalyContext, Explanation, Json, UiEvent};

use crate::PortResult;

/// MCP tool registry exposing thermal tools to Claude (FR-8.1). Registration is an adapter
/// concern; the port surface is discovery + invocation.
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
#[async_trait]
pub trait McpToolRegistry: Send + Sync {
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
