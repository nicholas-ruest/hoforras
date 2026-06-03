//! Adapter for the Trade Consensus ports (DDD-05) — the anti-corruption layer over QuDAG.
//!
//! [`QuDagAdapter`] is a **real** post-quantum implementation: ML-DSA-65 signing (`fips204`) and
//! ML-KEM-1024 transport (`ml-kem`). QR-Avalanche finality and the Kademlia `.dark` DHT are
//! QuDAG-side and confirmed on a live mesh in Phase C; the rest is exercised here behind the
//! `MlDsaSigner` / `DagNetwork` / `PeerDiscovery` ports (swappable per ADR-0001).

pub mod qudag;

pub use qudag::QuDagAdapter;
