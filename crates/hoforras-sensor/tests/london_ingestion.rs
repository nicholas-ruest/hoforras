//! London-School interaction tests for the ingestion boundary (FR-1.1..1.3).
//!
//! These assert *interactions* — message, order, cardinality and ABSENCE — against mocked sensor
//! ports. The headline assertions are the strict order `validate → score → sign → emit` and the
//! invariant that a rejected reading is recorded and **never** emitted.

use hoforras_domain::{Ed25519Signature, NodeId, QualityScore, RawReading, ValidationStatus};
use hoforras_ports::sensor::{
    MockEventEmitter, MockQualityScorer, MockRejectionLog, MockValidator, MockWitnessSigner,
};
use hoforras_sensor::IngestionPipeline;
use mockall::Sequence;

fn reading() -> RawReading {
    RawReading {
        node_id: NodeId::new("pozsonyi14.thermal.budapest.dark").unwrap(),
        timestamp_ns: 42,
        temperature_celsius: 22.5,
        pipe_vibration_hz: 3.0,
        fluid_pressure_bar: 2.0,
        ground_thermal_gradient: 0.2,
    }
}

#[tokio::test]
async fn ingest_valid_runs_validate_score_sign_emit_in_order() {
    let mut seq = Sequence::new();

    let mut validator = MockValidator::new();
    validator
        .expect_validate()
        .times(1)
        .in_sequence(&mut seq)
        .returning(|_| ValidationStatus::Valid);

    let mut scorer = MockQualityScorer::new();
    scorer
        .expect_score()
        .times(1)
        .in_sequence(&mut seq)
        .returning(|_| QualityScore(0.95));

    let mut signer = MockWitnessSigner::new();
    signer
        .expect_sign()
        .times(1)
        .in_sequence(&mut seq)
        .returning(|_| Ok(Ed25519Signature([3u8; 64])));

    let mut emitter = MockEventEmitter::new();
    emitter
        .expect_emit()
        .times(1)
        .in_sequence(&mut seq)
        .returning(|_| Ok(()));

    let rejections = MockRejectionLog::new(); // never recorded on the happy path

    let pipeline = IngestionPipeline::new(validator, scorer, signer, emitter, rejections);
    pipeline.ingest(reading()).await.unwrap();
}

#[tokio::test]
async fn ingest_invalid_records_rejection_and_never_emits() {
    let mut validator = MockValidator::new();
    validator
        .expect_validate()
        .times(1)
        .returning(|_| ValidationStatus::Invalid("bad reading".into()));

    let mut scorer = MockQualityScorer::new();
    scorer.expect_score().times(0); // never scored once invalid

    let mut signer = MockWitnessSigner::new();
    signer.expect_sign().times(0); // never signed

    let mut emitter = MockEventEmitter::new();
    emitter.expect_emit().times(0); // FR-1.2: invalid readings are never emitted

    let mut rejections = MockRejectionLog::new();
    rejections.expect_record().times(1).return_const(());

    let pipeline = IngestionPipeline::new(validator, scorer, signer, emitter, rejections);
    let res = pipeline.ingest(reading()).await;
    assert!(res.is_err());
}

#[tokio::test]
async fn ingest_emitted_frame_is_signed() {
    const SIG: [u8; 64] = [11u8; 64];

    let mut validator = MockValidator::new();
    validator
        .expect_validate()
        .returning(|_| ValidationStatus::Valid);
    let mut scorer = MockQualityScorer::new();
    scorer.expect_score().returning(|_| QualityScore(0.8));
    let mut signer = MockWitnessSigner::new();
    signer
        .expect_sign()
        .times(1)
        .returning(|_| Ok(Ed25519Signature(SIG)));

    let mut emitter = MockEventEmitter::new();
    emitter
        .expect_emit()
        .times(1)
        .withf(|frame| frame.witness == Ed25519Signature(SIG))
        .returning(|_| Ok(()));

    let rejections = MockRejectionLog::new();
    let pipeline = IngestionPipeline::new(validator, scorer, signer, emitter, rejections);
    pipeline.ingest(reading()).await.unwrap();
}

#[tokio::test]
async fn ingest_malformed_rejected_before_any_collaborator() {
    let mut raw = reading();
    raw.temperature_celsius = f32::NAN;

    // No expectations on ANY collaborator ⇒ any call panics the test.
    let validator = MockValidator::new();
    let scorer = MockQualityScorer::new();
    let signer = MockWitnessSigner::new();
    let emitter = MockEventEmitter::new();
    let rejections = MockRejectionLog::new();

    let pipeline = IngestionPipeline::new(validator, scorer, signer, emitter, rejections);
    let res = pipeline.ingest(raw).await;
    assert!(res.is_err());
}
