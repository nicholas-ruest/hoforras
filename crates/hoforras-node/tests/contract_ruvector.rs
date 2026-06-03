//! Contract test for `RuVectorAdapter` vs the `VectorIndex`/`GnnEngine` ports (R9 contract / FR-6.3).
//!
//! Production runs `ruvector` HNSW (sub-ms search confirmed in Phase C). Here the reference adapter
//! is exercised end-to-end with no mocks, pinning the FR-6.3 invariant: search returns ≤ topK
//! candidates, all satisfying the surplus+district filter (never witnesses), ordered by similarity.

use hoforras_domain::{DistrictGraph, Embedding, JunctionId, NodeId, Query};
use hoforras_node::memory::RuVectorAdapter;
use hoforras_ports::memory::{GnnEngine, VectorIndex};
use serde_json::json;

fn node(s: &str) -> NodeId {
    NodeId::new(format!("{s}.thermal.budapest.dark")).unwrap()
}

/// A unit-norm 128-dim embedding with `weights` placed at the leading indices (rest 0). Caller is
/// responsible for passing already-normalized weights.
fn embedding(weights: &[f32]) -> Embedding {
    let mut v = vec![0.0f32; 128];
    v[..weights.len()].copy_from_slice(weights);
    Embedding::new(v).unwrap()
}

async fn seed(idx: &RuVectorAdapter) {
    // similarity to query [1,0,..]: a=1.0, b=0.6
    idx.upsert(
        node("a"),
        embedding(&[1.0]),
        json!({"kind":"state","thermal_surplus_available":true,"district":"XIII","timestamp_ns":1}),
    )
    .await
    .unwrap();
    idx.upsert(
        node("b"),
        embedding(&[0.6, 0.8]),
        json!({"kind":"state","thermal_surplus_available":true,"district":"XIII","timestamp_ns":2}),
    )
    .await
    .unwrap();
    // excluded: no surplus
    idx.upsert(node("d"), embedding(&[1.0]),
        json!({"kind":"state","thermal_surplus_available":false,"district":"XIII","timestamp_ns":3}))
        .await.unwrap();
    // excluded: wrong district
    idx.upsert(
        node("e"),
        embedding(&[1.0]),
        json!({"kind":"state","thermal_surplus_available":true,"district":"XIV","timestamp_ns":4}),
    )
    .await
    .unwrap();
    // excluded: witness record
    idx.upsert(
        node("w"),
        embedding(&[1.0]),
        json!({"kind":"witness","district":"XIII"}),
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn search_is_filtered_ordered_and_topk_bounded() {
    let idx = RuVectorAdapter::new();
    seed(&idx).await;

    let query = Query {
        vector: embedding(&[1.0]),
        top_k: 2,
        surplus_available: true,
        district: "XIII".into(),
    };
    let results = idx.search(query).await.unwrap();

    // ≤ topK …
    assert!(results.len() <= 2);
    // … only the surplus-available, in-district, non-witness points survive …
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].node_id, node("a"));
    assert_eq!(results[1].node_id, node("b"));
    // … ordered by descending similarity.
    assert!(results[0].similarity >= results[1].similarity);
    assert!((results[0].similarity - 1.0).abs() < 1e-5);
}

#[tokio::test]
async fn topk_truncates_to_one() {
    let idx = RuVectorAdapter::new();
    seed(&idx).await;
    let query = Query {
        vector: embedding(&[1.0]),
        top_k: 1,
        surplus_available: true,
        district: "XIII".into(),
    };
    let results = idx.search(query).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].node_id, node("a")); // the single most-similar match
}

#[tokio::test]
async fn witness_records_are_never_returned() {
    let idx = RuVectorAdapter::new();
    idx.upsert(
        node("w"),
        embedding(&[1.0]),
        json!({"kind":"witness","district":"XIII"}),
    )
    .await
    .unwrap();
    let query = Query {
        vector: embedding(&[1.0]),
        top_k: 5,
        surplus_available: true,
        district: "XIII".into(),
    };
    assert!(idx.search(query).await.unwrap().is_empty());
}

#[tokio::test]
async fn gnn_inference_scores_nodes_over_weighted_edges() {
    let idx = RuVectorAdapter::new();
    let graph = DistrictGraph {
        nodes: vec![node("a"), node("b"), node("c")],
        edges: vec![
            (JunctionId(0), JunctionId(1), 1.0),
            (JunctionId(1), JunctionId(2), 9.0), // node b/c carry the most weight
        ],
    };
    let state = idx.infer(graph).await.unwrap();
    assert_eq!(state.node_scores.len(), 3);
    // The highest-degree-weight node is normalized to 1.0.
    let max = state
        .node_scores
        .iter()
        .map(|(_, s)| *s)
        .fold(0.0f32, f32::max);
    assert!((max - 1.0).abs() < 1e-5);
    assert!(state
        .node_scores
        .iter()
        .all(|(_, s)| (0.0..=1.0).contains(s)));
}
