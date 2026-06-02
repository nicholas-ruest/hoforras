//! # Hőforrás Appliance binary
//!
//! One Appliance per building. **Exactly one tokio runtime per Appliance** (ADR-0010): the single
//! `#[tokio::main]` entry point owns it. I/O-bound subsystems (consensus, mesh, ingestion) will run
//! on this runtime; latency-critical RVM coherence runs synchronously *off* it (ADR-0004).
//!
//! This is the foundation-slice skeleton (Prompt 0): it stands up the runtime and tracing. The
//! Node Isolation, Forecasting, and Memory contexts (DDD-04/03/06) and the full wiring of all
//! subsystems are implemented in later prompts (P1, P3, P4) and integrated in P10 (Completion).

use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    tracing::info!(
        adr = "ADR-0010",
        "Hőforrás Appliance starting on a single tokio runtime"
    );

    // Subsystems are wired here in Prompt 10 (Completion): sensor ingestion, broker MRAP loop,
    // node-isolation coherence supervisor, and the mesh/consensus fabric — all sharing this runtime.
    tracing::info!(
        domain_ok = hoforras_domain::EMBEDDING_DIM == 128,
        ports_present = true,
        "Foundation skeleton up; no subsystems wired yet (Prompt 0)."
    );

    Ok(())
}
