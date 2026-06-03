//! `RuVectorAdapter` — reference vector index + GNN engine over RuVector (DDD-06 ACL / ADR-0009).
//!
//! Production uses `ruvector` (HNSW/DiskANN for sub-ms similarity search, native GNN). That crate is
//! unavailable here, so this is a **reference in-process** implementation behind the `VectorIndex`
//! and `GnnEngine` ports — swappable per ADR-0001 with no change to `ThermalMemory`. It runs
//! node-local: embeddings/aggregates only, never raw frames (ADR-0009).
//!
//! Similarity is dot-product over the L2-normalized embeddings the service produces (== cosine).
//! Search applies the filter (surplus-available, district, never witness records), orders by
//! similarity, and truncates to `top_k` — the FR-6.3 invariant the contract test pins down.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use async_trait::async_trait;
use hoforras_domain::{Candidate, DistrictGraph, Embedding, GnnState, Json, NodeId, Query};
use hoforras_ports::memory::{GnnEngine, VectorIndex};
use hoforras_ports::PortResult;

/// One stored point: an embedding plus the metadata search filters on.
struct Point {
    key: String,
    node: NodeId,
    vector: Vec<f32>,
    surplus: bool,
    district: String,
    is_witness: bool,
}

/// Reference RuVector index/GNN. In-memory; `Mutex` guards the point set.
#[derive(Default)]
pub struct RuVectorAdapter {
    points: Mutex<Vec<Point>>,
    seq: AtomicU64,
}

impl RuVectorAdapter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of stored points (test/observability helper).
    pub fn len(&self) -> usize {
        self.points.lock().expect("ruvector state poisoned").len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Read a meta key as bool, defaulting to false.
fn meta_bool(meta: &Json, key: &str) -> bool {
    meta.get(key).and_then(Json::as_bool).unwrap_or(false)
}

/// Read a meta key as a string, defaulting to empty.
fn meta_str(meta: &Json, key: &str) -> String {
    meta.get(key)
        .and_then(Json::as_str)
        .unwrap_or("")
        .to_string()
}

#[async_trait]
impl VectorIndex for RuVectorAdapter {
    async fn upsert(&self, id: NodeId, vector: Embedding, meta: Json) -> PortResult<()> {
        let is_witness = meta_str(&meta, "kind") == "witness";
        // State points are keyed by (node, timestamp) so a building's history coexists; witness
        // points are append-only (unique per call) for forensic replay.
        let key = if is_witness {
            format!(
                "witness|{}|{}",
                id.as_str(),
                self.seq.fetch_add(1, Ordering::SeqCst)
            )
        } else {
            let ts = meta.get("timestamp_ns").and_then(Json::as_u64).unwrap_or(0);
            format!("state|{}|{}", id.as_str(), ts)
        };

        let point = Point {
            key,
            node: id,
            vector: vector.as_slice().to_vec(),
            surplus: meta_bool(&meta, "thermal_surplus_available"),
            district: meta_str(&meta, "district"),
            is_witness,
        };

        let mut points = self.points.lock().expect("ruvector state poisoned");
        // Upsert semantics: replace any existing point with the same key.
        if let Some(existing) = points.iter_mut().find(|p| p.key == point.key) {
            *existing = point;
        } else {
            points.push(point);
        }
        Ok(())
    }

    async fn search(&self, query: Query) -> PortResult<Vec<Candidate>> {
        let q = query.vector.as_slice();
        let points = self.points.lock().expect("ruvector state poisoned");

        let mut scored: Vec<Candidate> = points
            .iter()
            // Witness records never participate in surplus matching (FR-6.5).
            .filter(|p| !p.is_witness)
            // Surplus filter (FR-6.3): when asked, only surplus-available buildings.
            .filter(|p| !query.surplus_available || p.surplus)
            // District filter (FR-6.3).
            .filter(|p| p.district == query.district)
            .map(|p| Candidate {
                node_id: p.node.clone(),
                similarity: dot(q, &p.vector),
            })
            .collect();

        // Order by descending similarity, then keep ≤ top_k (FR-6.3 invariant).
        scored.sort_by(|a, b| {
            b.similarity
                .partial_cmp(&a.similarity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        scored.truncate(query.top_k);
        Ok(scored)
    }
}

#[async_trait]
impl GnnEngine for RuVectorAdapter {
    /// Reference GNN: one round of weighted-degree aggregation over the district graph (buildings =
    /// nodes, pipes = weighted edges), normalized to [0, 1]. The production GNN runs natively in
    /// RuVector; this stands in behind the same port (FR-6.4).
    async fn infer(&self, graph: DistrictGraph) -> PortResult<GnnState> {
        let n = graph.nodes.len();
        if n == 0 {
            return Ok(GnnState {
                node_scores: vec![],
            });
        }
        let mut weight = vec![0.0f32; n];
        for (a, b, w) in &graph.edges {
            weight[(a.0 as usize) % n] += *w;
            weight[(b.0 as usize) % n] += *w;
        }
        let max = weight.iter().copied().fold(0.0f32, f32::max);
        let node_scores = graph
            .nodes
            .iter()
            .enumerate()
            .map(|(i, node)| {
                let score = if max > 0.0 { weight[i] / max } else { 0.0 };
                (node.clone(), score)
            })
            .collect();
        Ok(GnnState { node_scores })
    }
}

/// Dot product of two equal-length vectors (== cosine for the L2-normalized embeddings we store).
/// Mismatched lengths score 0 so a malformed query can never panic.
fn dot(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use hoforras_domain::JunctionId;
    use serde_json::json;

    fn node(s: &str) -> NodeId {
        NodeId::new(format!("{s}.thermal.budapest.dark")).unwrap()
    }

    fn vec128(fill: f32) -> Embedding {
        Embedding::new(vec![fill; 128]).unwrap()
    }

    #[tokio::test]
    async fn search_excludes_witness_and_wrong_district() {
        let idx = RuVectorAdapter::new();
        idx.upsert(
            node("a"),
            vec128(0.1),
            json!({"kind":"state","thermal_surplus_available":true,"district":"XIII"}),
        )
        .await
        .unwrap();
        idx.upsert(
            node("b"),
            vec128(0.1),
            json!({"kind":"witness","district":"XIII"}),
        )
        .await
        .unwrap();
        idx.upsert(
            node("c"),
            vec128(0.1),
            json!({"kind":"state","thermal_surplus_available":true,"district":"XIV"}),
        )
        .await
        .unwrap();

        let q = Query {
            vector: vec128(0.1),
            top_k: 5,
            surplus_available: true,
            district: "XIII".into(),
        };
        let results = idx.search(q).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].node_id, node("a"));
    }

    #[tokio::test]
    async fn gnn_scores_every_node() {
        let idx = RuVectorAdapter::new();
        let graph = DistrictGraph {
            nodes: vec![node("a"), node("b"), node("c")],
            edges: vec![
                (JunctionId(0), JunctionId(1), 2.0),
                (JunctionId(1), JunctionId(2), 4.0),
            ],
        };
        let state = idx.infer(graph).await.unwrap();
        assert_eq!(state.node_scores.len(), 3);
        assert!(state
            .node_scores
            .iter()
            .all(|(_, s)| (0.0..=1.0).contains(s)));
    }
}
