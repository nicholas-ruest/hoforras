//! `ThermalMemory` (DDD-06 / FR-6.1–6.5) — node-local vector/GNN memory of a building's thermal
//! behaviour.
//!
//! Generic subdomain: the storage, similarity search, and GNN are off-the-shelf (RuVector). The one
//! domain decision is the **embedding scheme** ([`embed`]): a deterministic 128-dim map of
//! normalized sensor channels plus rolling stats, so similarity is stable and witness replay is
//! reproducible (FR-6.2). Raw `ThermalFrame` history stays node-local (ADR-0009); only the
//! embedding (an aggregate) is ever handed to the index — the raw frame never leaves this unit.

use hoforras_domain::{
    Candidate, DistrictGraph, DomainError, Embedding, GnnState, NodeId, Query, ThermalFrame,
    WitnessRecord, EMBEDDING_DIM,
};
use hoforras_ports::memory::{GnnEngine, VectorIndex};
use hoforras_ports::PortResult;
use serde_json::json;

/// Number of surplus candidates returned per deficit query (`sparc.md` Part 2 §7).
const TOP_K: usize = 5;

/// Node-local thermal memory. Generic over its ports so it is unit-tested against mocks
/// (London-School, ADR-0001).
pub struct ThermalMemory<I, G> {
    index: I,
    gnn: G,
}

impl<I: VectorIndex, G: GnnEngine> ThermalMemory<I, G> {
    pub fn new(index: I, gnn: G) -> Self {
        Self { index, gnn }
    }

    /// Record a building's thermal state as a 128-dim embedding (FR-6.1/6.2). The **raw** frame is
    /// consumed here and only its embedding is stored — raw data never crosses the port (ADR-0009).
    pub async fn record_state(&self, node: NodeId, frame: ThermalFrame) -> PortResult<()> {
        let embedding = embed(&frame)?;
        let meta = json!({
            "kind": "state",
            "node": node.as_str(),
            "timestamp_ns": frame.timestamp_ns,
        });
        self.index.upsert(node, embedding, meta).await
    }

    /// Find buildings whose surplus matches a deficit embedding, within a district (FR-6.3). Builds
    /// the filtered top-K query; the index enforces ≤ topK / filter / similarity-order (NFR-4).
    pub async fn find_surplus_matches(
        &self,
        deficit_embedding: Embedding,
        district: impl Into<String>,
    ) -> PortResult<Vec<Candidate>> {
        let query = Query {
            vector: deficit_embedding,
            top_k: TOP_K,
            surplus_available: true,
            district: district.into(),
        };
        self.index.search(query).await
    }

    /// Run GNN inference over the district graph — buildings = nodes, pipes = weighted edges
    /// (FR-6.4). Delegated wholesale to the engine (generic-by-design, DDD-06 §7).
    pub async fn district_inference(&self, graph: DistrictGraph) -> PortResult<GnnState> {
        self.gnn.infer(graph).await
    }

    /// Store a witness record as an embedding for forensic replay (FR-6.5). Tagged `kind: witness`
    /// so it is never returned by surplus matching.
    pub async fn store_witness_for_replay(
        &self,
        node: NodeId,
        record: WitnessRecord,
    ) -> PortResult<()> {
        let embedding = embed_witness(&record)?;
        let meta = json!({ "kind": "witness", "node": node.as_str() });
        self.index.upsert(node, embedding, meta).await
    }
}

// ───────────────────────── embedding scheme (FR-6.2, Q-2) ─────────────────────────

/// Deterministic 128-dim embedding of a thermal frame: normalized channels + rolling stats,
/// positionally modulated and L2-normalized so dot-product == cosine similarity.
pub fn embed(frame: &ThermalFrame) -> Result<Embedding, DomainError> {
    let ch = normalized_channels(frame);
    let mean = ch.iter().sum::<f32>() / ch.len() as f32;
    let var = ch.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / ch.len() as f32;
    let min = ch.iter().copied().fold(f32::INFINITY, f32::min);
    let max = ch.iter().copied().fold(f32::NEG_INFINITY, f32::max);

    // 11 base features: 4 normalized channels + 7 rolling/derived stats.
    let basis = [
        ch[0],
        ch[1],
        ch[2],
        ch[3],
        mean,
        var,
        min,
        max,
        max - min,
        ch[0] * ch[2],
        ch[1] * ch[3],
    ];

    let mut values = Vec::with_capacity(EMBEDDING_DIM);
    for i in 0..EMBEDDING_DIM {
        let f = basis[i % basis.len()];
        // Deterministic positional basis (a fixed Fourier-style feature map), no randomness.
        let phase = (i as f32) * std::f32::consts::FRAC_PI_8;
        values.push(f * phase.cos() + (f * f) * phase.sin());
    }
    l2_normalize(&mut values);
    Embedding::new(values)
}

/// Deterministic 128-dim embedding of a 64-byte witness record (FR-6.5). Each byte's nibbles become
/// two normalized features (64 × 2 = 128), then L2-normalized.
pub fn embed_witness(record: &WitnessRecord) -> Result<Embedding, DomainError> {
    let mut values = Vec::with_capacity(EMBEDDING_DIM);
    for &b in record.as_bytes().iter() {
        values.push((b >> 4) as f32 / 15.0);
        values.push((b & 0x0f) as f32 / 15.0);
    }
    l2_normalize(&mut values);
    Embedding::new(values)
}

/// The four sensor channels normalized into [0, 1] over physically-sane envelopes.
fn normalized_channels(frame: &ThermalFrame) -> [f32; 4] {
    [
        unit(frame.temperature_celsius, -40.0, 120.0),
        unit(frame.pipe_vibration_hz, 0.0, 80.0),
        unit(frame.fluid_pressure_bar, 0.0, 16.0),
        unit(frame.ground_thermal_gradient, -1.0, 1.0),
    ]
}

fn unit(x: f32, lo: f32, hi: f32) -> f32 {
    if !x.is_finite() || hi <= lo {
        return 0.0;
    }
    ((x - lo) / (hi - lo)).clamp(0.0, 1.0)
}

/// Scale a vector to unit L2 norm in place (no-op for the zero vector).
fn l2_normalize(v: &mut [f32]) {
    let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for x in v.iter_mut() {
            *x /= norm;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hoforras_domain::{Ed25519Signature, QualityScore, ValidationStatus};

    fn frame(temp: f32) -> ThermalFrame {
        ThermalFrame {
            node_id: NodeId::new("pozsonyi14.thermal.budapest.dark").unwrap(),
            timestamp_ns: 42,
            temperature_celsius: temp,
            pipe_vibration_hz: 4.0,
            fluid_pressure_bar: 3.0,
            ground_thermal_gradient: 0.1,
            quality_score: QualityScore(1.0),
            validation: ValidationStatus::Valid,
            witness: Ed25519Signature([0u8; 64]),
        }
    }

    #[test]
    fn embedding_is_128_dim() {
        assert_eq!(embed(&frame(20.0)).unwrap().as_slice().len(), EMBEDDING_DIM);
    }

    #[test]
    fn embedding_is_deterministic() {
        // Same frame ⇒ byte-identical embedding (FR-6.2 — stable similarity & replay).
        assert_eq!(embed(&frame(20.0)).unwrap(), embed(&frame(20.0)).unwrap());
        // Different input ⇒ different embedding (the map is not constant).
        assert_ne!(embed(&frame(20.0)).unwrap(), embed(&frame(90.0)).unwrap());
    }

    #[test]
    fn embedding_is_l2_normalized() {
        let norm: f32 = embed(&frame(55.0))
            .unwrap()
            .as_slice()
            .iter()
            .map(|x| x * x)
            .sum::<f32>()
            .sqrt();
        assert!(
            (norm - 1.0).abs() < 1e-5,
            "embedding should be unit-norm, got {norm}"
        );
    }

    #[test]
    fn witness_embedding_is_128_dim_and_deterministic() {
        let rec = WitnessRecord::new([7u8; 64]);
        assert_eq!(embed_witness(&rec).unwrap().as_slice().len(), EMBEDDING_DIM);
        assert_eq!(embed_witness(&rec).unwrap(), embed_witness(&rec).unwrap());
    }
}
