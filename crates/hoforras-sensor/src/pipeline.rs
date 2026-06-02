//! `IngestionPipeline` — the ingestion boundary unit (FR-1.1..1.4).
//!
//! Generic over the five sensor ports (constructor injection, ADR-0002) so it is unit-tested
//! against `mockall` mocks (London-School, ADR-0001). [`IngestionPipeline::ingest`] enforces a
//! **strict order**: boundary check → validate → score → sign → emit. A reading that fails the
//! boundary check is rejected *before any collaborator is touched* (FR-1.1); a reading the
//! validator rejects is recorded and *never emitted* (FR-1.2).

use hoforras_domain::{
    canonical_bytes, DomainError, NodeId, QualityScore, RawReading, ThermalFrame, ValidationStatus,
};
use hoforras_ports::sensor::{EventEmitter, QualityScorer, RejectionLog, Validator, WitnessSigner};
use serde::Serialize;

/// The content actually signed by the witness (FR-1.3 / ADR-0014): everything in a
/// [`ThermalFrame`] **except** the witness itself. Private so the signed shape is owned by this
/// unit and cannot drift.
#[derive(Serialize)]
struct FrameContent<'a> {
    node_id: &'a NodeId,
    timestamp_ns: u64,
    temperature_celsius: f32,
    pipe_vibration_hz: f32,
    fluid_pressure_bar: f32,
    ground_thermal_gradient: f32,
    quality_score: QualityScore,
    validation: &'a ValidationStatus,
}

/// Validate → score → sign → emit, in that order. Generic over its ports (ADR-0002).
pub struct IngestionPipeline<V, Q, S, E, R> {
    validator: V,
    scorer: Q,
    signer: S,
    emitter: E,
    rejections: R,
}

impl<V, Q, S, E, R> IngestionPipeline<V, Q, S, E, R>
where
    V: Validator,
    Q: QualityScorer,
    S: WitnessSigner,
    E: EventEmitter,
    R: RejectionLog,
{
    pub fn new(validator: V, scorer: Q, signer: S, emitter: E, rejections: R) -> Self {
        Self {
            validator,
            scorer,
            signer,
            emitter,
            rejections,
        }
    }

    /// Ingest a single raw reading. Strict order (FR-1.1):
    ///
    /// 1. **boundary check** — reject NaN/inf or negative-pressure readings *before* touching any
    ///    collaborator;
    /// 2. **validate** — on `Invalid`, record the rejection and return `Err` without emitting
    ///    (FR-1.2);
    /// 3. **score** the reading's quality;
    /// 4. **sign** the canonical bytes of the frame content (ADR-0014);
    /// 5. **emit** the witnessed [`ThermalFrame`].
    pub async fn ingest(&self, raw: RawReading) -> Result<(), DomainError> {
        // (a) boundary check FIRST — no collaborator is consulted for a malformed reading.
        Self::boundary_check(&raw)?;

        // (b) validate; an invalid reading is recorded and never emitted (FR-1.2).
        match self.validator.validate(&raw) {
            ValidationStatus::Valid => {}
            ValidationStatus::Invalid(reason) => {
                self.rejections.record(raw, reason.clone());
                return Err(DomainError::Invalid(reason));
            }
        }

        // (c) score.
        let quality = self.scorer.score(&raw);

        // (d) sign the canonical bytes of everything except the witness (ADR-0014).
        let content = FrameContent {
            node_id: &raw.node_id,
            timestamp_ns: raw.timestamp_ns,
            temperature_celsius: raw.temperature_celsius,
            pipe_vibration_hz: raw.pipe_vibration_hz,
            fluid_pressure_bar: raw.fluid_pressure_bar,
            ground_thermal_gradient: raw.ground_thermal_gradient,
            quality_score: quality,
            validation: &ValidationStatus::Valid,
        };
        let signature = self.signer.sign(&canonical_bytes(&content)?)?;

        // (e) construct the witnessed frame and emit it.
        let frame = ThermalFrame {
            node_id: raw.node_id,
            timestamp_ns: raw.timestamp_ns,
            temperature_celsius: raw.temperature_celsius,
            pipe_vibration_hz: raw.pipe_vibration_hz,
            fluid_pressure_bar: raw.fluid_pressure_bar,
            ground_thermal_gradient: raw.ground_thermal_gradient,
            quality_score: quality,
            validation: ValidationStatus::Valid,
            witness: signature,
        };
        self.emitter.emit(frame).await?;
        Ok(())
    }

    /// Reject structurally malformed readings (NFR-12). Runs before any collaborator so a garbage
    /// frame can never reach the validator, scorer, signer or emitter (FR-1.1).
    fn boundary_check(raw: &RawReading) -> Result<(), DomainError> {
        let fields = [
            ("temperature_celsius", raw.temperature_celsius),
            ("pipe_vibration_hz", raw.pipe_vibration_hz),
            ("fluid_pressure_bar", raw.fluid_pressure_bar),
            ("ground_thermal_gradient", raw.ground_thermal_gradient),
        ];
        for (name, value) in fields {
            if !value.is_finite() {
                return Err(DomainError::Invalid(format!("non-finite {name}: {value}")));
            }
        }
        if raw.fluid_pressure_bar < 0.0 {
            return Err(DomainError::Invalid(format!(
                "negative fluid_pressure_bar: {}",
                raw.fluid_pressure_bar
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hoforras_domain::Ed25519Signature;
    use hoforras_ports::sensor::{
        MockEventEmitter, MockQualityScorer, MockRejectionLog, MockValidator, MockWitnessSigner,
    };

    fn reading() -> RawReading {
        RawReading {
            node_id: NodeId::new("pozsonyi14.thermal.budapest.dark").unwrap(),
            timestamp_ns: 1_000,
            temperature_celsius: 21.0,
            pipe_vibration_hz: 4.0,
            fluid_pressure_bar: 2.0,
            ground_thermal_gradient: 0.3,
        }
    }

    #[tokio::test]
    async fn valid_reading_is_emitted() {
        let mut validator = MockValidator::new();
        validator
            .expect_validate()
            .returning(|_| ValidationStatus::Valid);
        let mut scorer = MockQualityScorer::new();
        scorer.expect_score().returning(|_| QualityScore(0.9));
        let mut signer = MockWitnessSigner::new();
        signer
            .expect_sign()
            .returning(|_| Ok(Ed25519Signature([7u8; 64])));
        let mut emitter = MockEventEmitter::new();
        emitter.expect_emit().times(1).returning(|_| Ok(()));
        let rejections = MockRejectionLog::new();

        let pipeline = IngestionPipeline::new(validator, scorer, signer, emitter, rejections);
        assert!(pipeline.ingest(reading()).await.is_ok());
    }

    #[tokio::test]
    async fn malformed_reading_rejected_at_boundary() {
        let mut raw = reading();
        raw.temperature_celsius = f32::NAN;
        // No expectations set ⇒ any collaborator call would panic.
        let pipeline = IngestionPipeline::new(
            MockValidator::new(),
            MockQualityScorer::new(),
            MockWitnessSigner::new(),
            MockEventEmitter::new(),
            MockRejectionLog::new(),
        );
        assert!(pipeline.ingest(raw).await.is_err());
    }
}
