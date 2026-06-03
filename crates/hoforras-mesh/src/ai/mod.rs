//! # AI Orchestration (DDD-08)
//!
//! The ThermalSense-Bridge — an **Open Host Service** (ADR-0002) publishing a stable MCP tool API
//! that Claude and the dashboard consume — plus the anomaly explainer and a deferred federation stub.
//!
//! * [`ThermalBridgeServer`] — boots exactly five tools, each delegating to one backing service
//!   (FR-8.1); unknown tools error with no delegation.
//! * [`AnomalyExplainer`] — explain via Claude, then surface to the operator (FR-8.2).
//! * [`FederationStub`] — cross-district federation: market-signals-only, no raw data, **deferred**
//!   per spec R-5 (ADR-0007 spirit).
//! * [`RufloMcpAdapter`] / [`WebSocketSink`] — reference ruflo MCP / WebSocket adapters.

pub mod adapters;
pub mod bridge;
pub mod explainer;
pub mod federation;

pub use adapters::{RufloMcpAdapter, WebSocketSink};
pub use bridge::{ThermalBridgeServer, TOOL_NAMES};
pub use explainer::AnomalyExplainer;
pub use federation::{FederationOutcome, FederationStub};
