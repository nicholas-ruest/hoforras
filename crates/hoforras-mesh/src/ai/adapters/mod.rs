//! Reference adapters for the AI Orchestration ports (DDD-08).
//!
//! [`RufloMcpAdapter`] (MCP registry + Claude reasoning over ruflo) and [`WebSocketSink`] (operator
//! channel over tokio-tungstenite) stand in for the unavailable substrates behind the
//! `hoforras-ports` AI traits — swappable per ADR-0001.

pub mod ruflo_mcp;
pub mod websocket_sink;

pub use ruflo_mcp::RufloMcpAdapter;
pub use websocket_sink::WebSocketSink;
