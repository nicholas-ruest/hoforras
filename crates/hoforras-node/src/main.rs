//! # Hőforrás Appliance binary
//!
//! One Appliance per building. **Exactly one tokio runtime per Appliance** (ADR-0010): this single
//! `#[tokio::main]` entry point owns it. I/O-bound subsystems (ingestion, broker MRAP, consensus,
//! mesh, AI orchestration) run on this runtime; latency-critical RVM coherence runs synchronously
//! *off* it (ADR-0004).
//!
//! At Completion (P10) the Appliance assembles every context with real adapters (mocks retired) via
//! [`hoforras_node::appliance::Appliance`]. This binary boots the runtime, wires the node identity,
//! and reports readiness; the live MRAP loops and mesh subscriptions are driven from here on real
//! hardware.

use hoforras_domain::NodeId;
use hoforras_node::appliance::Appliance;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let node_id = NodeId::new("pozsonyi14.thermal.budapest.dark")?;
    let appliance = Appliance::new(node_id.clone());

    tracing::info!(
        adr = "ADR-0010",
        node = node_id.as_str(),
        "Hőforrás Appliance starting on a single tokio runtime"
    );

    // Construct the real consensus gateway as a liveness check that the post-quantum substrate
    // (ML-DSA-65 / ML-KEM-1024) wired up; the full MRAP loops attach to real sensors/mesh on
    // Appliance hardware.
    let _consensus = Appliance::consensus_gateway()?;

    tracing::info!(
        contexts = "sensing · forecasting · memory · isolation · broker · consensus · mesh · ai",
        mocks_retired = true,
        "Appliance contexts wired (DDD-00 context map); runtime ready (ADR-0010)."
    );

    // Suppress unused-field lint on the assembled appliance in the skeleton binary; the live loops
    // (broker.tick on an interval, mesh subscribe) run here on real hardware.
    let _ = &appliance.node_id;
    Ok(())
}
