//! London-School interaction tests for Trade Consensus (R2 / FR-5.1–5.5).
//!
//! Interactions against mocked ports (`MockMlDsaSigner`, `MockDagNetwork`, `MockPeerDiscovery`).
//! The headline assertion is **sign-before-broadcast** (`finalize_signs_then_broadcasts`, FR-5.1):
//! the signer's `sign` must be ordered strictly before the network's broadcast.

use hoforras_domain::{
    DagEntry, Finality, JunctionId, MlDsaSignature, NodeAddr, NodeId, ThermalTradeAgreement,
    Timestamp,
};
use hoforras_mesh::consensus::ConsensusGateway;
use hoforras_ports::consensus::{MockDagNetwork, MockMlDsaSigner, MockPeerDiscovery};
use mockall::Sequence;

fn agreement(duration_hours: u32) -> ThermalTradeAgreement {
    ThermalTradeAgreement {
        seller: NodeId::new("pozsonyi14.thermal.budapest.dark").unwrap(),
        buyer: NodeId::new("pozsonyi22.thermal.budapest.dark").unwrap(),
        kwh_offered: 40.0,
        duration_hours,
        credit_price_per_kwh: 2.3,
        pipe_route: vec![JunctionId(7), JunctionId(12), JunctionId(18)],
        valid_from: Timestamp(0),
    }
}

fn signed(entry: DagEntry) -> DagEntry {
    DagEntry {
        signature: Some(MlDsaSignature(vec![1, 2, 3])),
        ..entry
    }
}

// 1 — sign happens strictly before broadcast (FR-5.1). HEADLINE.
#[tokio::test]
async fn finalize_signs_then_broadcasts() {
    let mut seq = Sequence::new();
    let mut signer = MockMlDsaSigner::new();
    signer
        .expect_sign()
        .times(1)
        .in_sequence(&mut seq)
        .returning(|e| Ok(signed(e)));
    let mut network = MockDagNetwork::new();
    network
        .expect_broadcast_and_await_consensus()
        .times(1)
        .in_sequence(&mut seq) // strictly after sign
        // defensive: the entry handed to the network must already be signed
        .withf(|e| e.signature.is_some())
        .returning(|_| {
            Ok(Finality {
                reached_at: Timestamp(0),
            })
        });

    let gw = ConsensusGateway::new(signer, network, MockPeerDiscovery::new());
    gw.finalize(agreement(6)).await.unwrap();
}

// 2 — sign is invoked exactly once per finalize (FR-5.1/5.2).
#[tokio::test]
async fn sign_uses_ml_dsa_once() {
    let mut signer = MockMlDsaSigner::new();
    signer.expect_sign().times(1).returning(|e| Ok(signed(e)));
    let mut network = MockDagNetwork::new();
    network
        .expect_broadcast_and_await_consensus()
        .returning(|_| {
            Ok(Finality {
                reached_at: Timestamp(0),
            })
        });

    let gw = ConsensusGateway::new(signer, network, MockPeerDiscovery::new());
    gw.finalize(agreement(6)).await.unwrap();
}

// 3 — peers resolve over .dark domains via Kademlia; no directory port exists (FR-5.4).
#[tokio::test]
async fn resolve_peer_uses_dark_domain() {
    let mut discovery = MockPeerDiscovery::new();
    discovery
        .expect_resolve()
        .times(1)
        .withf(|d| d == "pozsonyi22.thermal.budapest.dark")
        .returning(|d| Ok(NodeAddr(format!("kad://{d}"))));

    let gw = ConsensusGateway::new(MockMlDsaSigner::new(), MockDagNetwork::new(), discovery);
    let addr = gw
        .resolve_peer("pozsonyi22.thermal.budapest.dark")
        .await
        .unwrap();
    assert_eq!(addr.0, "kad://pozsonyi22.thermal.budapest.dark");
}

// 4 — verify delegates to the signer; a mutated entry returns false (FR-5.5).
#[tokio::test]
async fn verify_rejects_mutated_entry() {
    let good = signed(DagEntry::new(agreement(6)));
    let mut tampered = good.clone();
    tampered.agreement.kwh_offered = 9_999.0; // mutate the payload after signing

    let good_match = good.clone();
    let mut signer = MockMlDsaSigner::new();
    signer.expect_verify().returning(move |e| *e == good_match); // only the untampered entry verifies

    let gw = ConsensusGateway::new(signer, MockDagNetwork::new(), MockPeerDiscovery::new());
    assert!(gw.verify(&good));
    assert!(!gw.verify(&tampered));
}

// 5 — an overlong trade is rejected at the boundary, before any signing (FR-5.1/3.7).
#[tokio::test]
async fn boundary_rejects_overlong_duration() {
    let mut signer = MockMlDsaSigner::new();
    signer.expect_sign().times(0); // never reached
    let mut network = MockDagNetwork::new();
    network.expect_broadcast_and_await_consensus().times(0); // never reached

    let gw = ConsensusGateway::new(signer, network, MockPeerDiscovery::new());
    let res = gw.finalize(agreement(7)).await; // 7h > 6h ceiling
    assert!(res.is_err());
}
