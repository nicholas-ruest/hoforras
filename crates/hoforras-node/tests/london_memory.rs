//! London-School interaction tests for Thermal Memory (R9 / FR-6.1–6.5).
//!
//! Interactions against mocked ports (`MockVectorIndex`, `MockGnnEngine`): the unit's job is to
//! shape the embedding, build the filtered top-K query, and delegate. The real filtering/ordering
//! lives in the adapter and is pinned by `contract_ruvector.rs`.

use hoforras_domain::{
    Candidate, DistrictGraph, Ed25519Signature, Embedding, GnnState, JunctionId, NodeId,
    QualityScore, ThermalFrame, ValidationStatus, WitnessRecord,
};
use hoforras_node::memory::ThermalMemory;
use hoforras_ports::memory::{MockGnnEngine, MockVectorIndex};

fn node(s: &str) -> NodeId {
    NodeId::new(format!("{s}.thermal.budapest.dark")).unwrap()
}

fn frame() -> ThermalFrame {
    ThermalFrame {
        node_id: node("pozsonyi14"),
        timestamp_ns: 1_700_000_000_000,
        temperature_celsius: 21.0,
        pipe_vibration_hz: 4.0,
        fluid_pressure_bar: 3.0,
        ground_thermal_gradient: 0.1,
        quality_score: QualityScore(1.0),
        validation: ValidationStatus::Valid,
        witness: Ed25519Signature([0u8; 64]),
    }
}

fn embedding() -> Embedding {
    Embedding::new(vec![0.1; 128]).unwrap()
}

// 1 — record_state upserts a 128-dim vector (FR-6.1/6.2).
#[tokio::test]
async fn record_state_upserts_128dim() {
    let mut index = MockVectorIndex::new();
    index
        .expect_upsert()
        .times(1)
        .withf(|_id, vector, _meta| vector.as_slice().len() == 128) // 128-dim
        .returning(|_, _, _| Ok(()));
    let mem = ThermalMemory::new(index, MockGnnEngine::new());

    mem.record_state(node("pozsonyi14"), frame()).await.unwrap();
}

// 2 — find_surplus_matches builds a topK=5 query with the surplus+district filter (FR-6.3).
#[tokio::test]
async fn search_passes_topk_and_filter() {
    let mut index = MockVectorIndex::new();
    index
        .expect_search()
        .times(1)
        .withf(|q| q.top_k == 5 && q.surplus_available && q.district == "XIII")
        .returning(|_| Ok(vec![]));
    let mem = ThermalMemory::new(index, MockGnnEngine::new());

    mem.find_surplus_matches(embedding(), "XIII").await.unwrap();
}

// 3 — results are passed through ≤ topK and in the order the index returns (FR-6.3).
#[tokio::test]
async fn search_results_respect_filter() {
    let ordered = vec![
        Candidate {
            node_id: node("a"),
            similarity: 0.91,
        },
        Candidate {
            node_id: node("b"),
            similarity: 0.72,
        },
        Candidate {
            node_id: node("c"),
            similarity: 0.55,
        },
    ];
    let expected = ordered.clone();
    let mut index = MockVectorIndex::new();
    index
        .expect_search()
        .returning(move |_| Ok(ordered.clone()));
    let mem = ThermalMemory::new(index, MockGnnEngine::new());

    let got = mem.find_surplus_matches(embedding(), "XIII").await.unwrap();
    assert!(got.len() <= 5);
    assert_eq!(got, expected); // unchanged, still similarity-ordered
    assert!(got.windows(2).all(|w| w[0].similarity >= w[1].similarity));
}

// 4 — district_inference delegates to the GNN over the graph (FR-6.4).
#[tokio::test]
async fn gnn_infer_over_district_graph() {
    let graph = DistrictGraph {
        nodes: vec![node("a"), node("b")],
        edges: vec![(JunctionId(0), JunctionId(1), 3.0)],
    };
    let mut gnn = MockGnnEngine::new();
    gnn.expect_infer()
        .times(1)
        .withf(|g| g.nodes.len() == 2 && g.edges.len() == 1)
        .returning(|g| {
            Ok(GnnState {
                node_scores: g.nodes.into_iter().map(|n| (n, 1.0)).collect(),
            })
        });
    let mem = ThermalMemory::new(MockVectorIndex::new(), gnn);

    let state = mem.district_inference(graph).await.unwrap();
    assert_eq!(state.node_scores.len(), 2);
}

// 5 — witness records are stored for replay, tagged kind=witness (FR-6.5).
#[tokio::test]
async fn witness_stored_for_replay() {
    let mut index = MockVectorIndex::new();
    index
        .expect_upsert()
        .times(1)
        .withf(|_id, vector, meta| {
            vector.as_slice().len() == 128
                && meta.get("kind").and_then(|v| v.as_str()) == Some("witness")
        })
        .returning(|_, _, _| Ok(()));
    let mem = ThermalMemory::new(index, MockGnnEngine::new());

    mem.store_witness_for_replay(node("pozsonyi14"), WitnessRecord::new([3u8; 64]))
        .await
        .unwrap();
}
