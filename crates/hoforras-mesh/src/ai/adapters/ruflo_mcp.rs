//! `RufloMcpAdapter` — MCP tool registry + Claude reasoning over ruflo / @ruvnet/rvagent (DDD-08).
//!
//! Reference implementation behind the `McpToolRegistry` and `ClaudeReasoner` ports — the real
//! ruflo MCP runtime is unavailable here, so registration/discovery is an in-memory tool set and
//! reasoning is a deterministic narrative. Swappable per ADR-0001; the bridge (the OHS) is unchanged
//! when the real ruflo adapter is dropped in.

use std::collections::BTreeSet;
use std::sync::Mutex;

use async_trait::async_trait;
use hoforras_domain::{AnomalyContext, Explanation, Json};
use hoforras_ports::ai::{ClaudeReasoner, McpToolRegistry};
use hoforras_ports::PortResult;
use serde_json::json;

/// Reference ruflo MCP adapter. The registered tool set is the OHS catalogue.
#[derive(Default)]
pub struct RufloMcpAdapter {
    tools: Mutex<BTreeSet<String>>,
}

impl RufloMcpAdapter {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl McpToolRegistry for RufloMcpAdapter {
    async fn register(&self, name: String) -> PortResult<()> {
        self.tools
            .lock()
            .expect("mcp registry poisoned")
            .insert(name);
        Ok(())
    }

    fn has(&self, name: &str) -> bool {
        self.tools
            .lock()
            .expect("mcp registry poisoned")
            .contains(name)
    }

    fn list_tools(&self) -> Vec<String> {
        self.tools
            .lock()
            .expect("mcp registry poisoned")
            .iter()
            .cloned()
            .collect()
    }

    async fn invoke(&self, name: String, _args: Json) -> PortResult<Json> {
        // The bridge routes tool calls to backing services itself; this MCP-facing invoke just
        // acknowledges registration for an external Claude client probing the catalogue.
        if !self.has(&name) {
            return Err(hoforras_domain::DomainError::NotFound(format!(
                "unknown tool: {name}"
            )));
        }
        Ok(json!({ "tool": name, "status": "registered" }))
    }
}

#[async_trait]
impl ClaudeReasoner for RufloMcpAdapter {
    async fn explain(&self, context: AnomalyContext) -> PortResult<Explanation> {
        // Deterministic reference narrative (the real adapter calls Claude via ruflo).
        Ok(Explanation {
            text: format!(
                "Anomaly {:?} detected at {} (confidence {:.2}). Recommend inspecting the local \
                 thermal loop and recent trades.",
                context.anomaly.kind,
                context.node_id.as_str(),
                context.anomaly.confidence,
            ),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn register_then_has_and_list() {
        let adapter = RufloMcpAdapter::new();
        adapter
            .register("hoforras.thermal.district_status".into())
            .await
            .unwrap();
        assert!(adapter.has("hoforras.thermal.district_status"));
        assert!(!adapter.has("nope"));
        assert_eq!(adapter.list_tools().len(), 1);
    }
}
