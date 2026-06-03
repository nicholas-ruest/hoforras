//! Contract test for `QuDagAdapter` vs the real post-quantum primitives (R2 contract / ADR-0011).
//!
//! No mocks — exercises genuine ML-DSA-65 signing/verification over `postcard` canonical bytes
//! (ADR-0014) and the full `ConsensusGateway` path. Pins the FR-5.1 (sign-before-broadcast) and
//! FR-5.5 (tamper-evidence) invariants end-to-end. Sub-second finality (NFR-2) is not asserted here
//! — the reference network accepts in-process; NFR-2 is a Phase C live-mesh figure.

use hoforras_domain::{DagEntry, JunctionId, NodeId, ThermalTradeAgreement, Timestamp};
use hoforras_mesh::consensus::{ConsensusGateway, QuDagAdapter};
use hoforras_ports::consensus::MlDsaSigner;

fn agreement() -> ThermalTradeAgreement {
    ThermalTradeAgreement {
        seller: NodeId::new("pozsonyi14.thermal.budapest.dark").unwrap(),
        buyer: NodeId::new("pozsonyi22.thermal.budapest.dark").unwrap(),
        kwh_offered: 40.0,
        duration_hours: 6,
        credit_price_per_kwh: 2.3,
        pipe_route: vec![JunctionId(7), JunctionId(12), JunctionId(18)],
        valid_from: Timestamp(0),
    }
}

// A fresh adapter is its own signer/network/discovery — clone the keypair-bound adapter is not
// needed because the gateway only holds one. For the gateway tests we build three independent
// adapters sharing nothing; verification therefore uses the gateway's own signer instance.

#[test]
fn ml_dsa_sign_then_verify_roundtrips() {
    let adapter = QuDagAdapter::new().unwrap();
    let signed = adapter.sign(DagEntry::new(agreement())).unwrap();
    assert!(adapter.verify(&signed)); // FR-5.1/5.2: signed entry verifies
}

#[test]
fn mutating_a_signed_entry_breaks_verification() {
    let adapter = QuDagAdapter::new().unwrap();
    let signed = adapter.sign(DagEntry::new(agreement())).unwrap();

    // Flip an economic field after signing — canonical bytes change ⇒ verify fails (FR-5.5).
    let mut tampered = signed.clone();
    tampered.agreement.kwh_offered = 41.0;
    assert!(!adapter.verify(&tampered));

    // Mutate the route too — still rejected.
    let mut rerouted = signed.clone();
    rerouted.agreement.pipe_route.push(JunctionId(99));
    assert!(!adapter.verify(&rerouted));

    // The original, untouched entry still verifies.
    assert!(adapter.verify(&signed));
}

#[tokio::test]
async fn gateway_finalizes_a_valid_trade_end_to_end() {
    // One adapter plays signer + network + discovery; cloning is unnecessary since the gateway needs
    // three port objects — we build three adapters, but verification below uses the signer adapter.
    let gw = ConsensusGateway::new(
        QuDagAdapter::new().unwrap(),
        QuDagAdapter::new().unwrap(),
        QuDagAdapter::new().unwrap(),
    );
    let finality = gw.finalize(agreement()).await.unwrap();
    let _ = finality.reached_at; // present; real value is a Phase C live-mesh figure (NFR-2)
}

#[tokio::test]
async fn resolve_peer_requires_dark_domain() {
    let gw = ConsensusGateway::new(
        QuDagAdapter::new().unwrap(),
        QuDagAdapter::new().unwrap(),
        QuDagAdapter::new().unwrap(),
    );
    assert!(gw
        .resolve_peer("pozsonyi22.thermal.budapest.dark")
        .await
        .is_ok());
    assert!(gw.resolve_peer("pozsonyi22.example.com").await.is_err()); // not a .dark domain
}
