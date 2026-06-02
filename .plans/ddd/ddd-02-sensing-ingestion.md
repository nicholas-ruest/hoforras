# DDD-02 — Sensing & Ingestion (Supporting)

**Subdomain type:** Supporting · **Crate:** `hoforras-sensor` · **SPARC unit:** `IngestionPipeline`
(Part 2 §2) · **Key ADRs:** 0014 (canonical bytes), 0009 (raw stays node-local)

Transforms raw multi-source sensor input into validated, quality-scored, cryptographically witnessed
thermal readings — the trustworthy foundation everything else builds on.

## 1. Aggregates

### `SensorReading → ThermalFrame` (aggregate root)
The unit of ingestion for one SEED reading.

- **Identity:** `(NodeId, timestamp_ns)`.
- **Lifecycle:** `Raw → Validated → Scored → Witnessed → Emitted` **or** `Raw → Rejected(reason)`.
- **Rule:** a `ThermalFrame` is only `Emitted` if validation passed **and** it carries a valid
  Ed25519 signature.

## 2. Value Objects

`ThermalFrame{ node_id, timestamp_ns, temperature_celsius, pipe_vibration_hz, fluid_pressure_bar,
ground_thermal_gradient, quality_score, validation, witness }` ·
`QualityScore` · `ValidationStatus = Valid | Invalid(reason)` · `Ed25519Signature` ·
`RawReading` (transient input, never crosses a boundary) · `ReadingWindow` (published to Forecasting).

## 3. Domain Services

- **`IngestionPipeline`** — orchestrates `validate → score → sign → emit` in strict order
  (FR-1.1). Rejects malformed input at the boundary (NFR-12).

## 4. Invariants (→ Part 4 R6 / R14)

1. **Validation gating (FR-1.2):** `Invalid` ⇒ never emitted; recorded with a reason.
2. **Every emitted frame is witnessed (FR-1.3):** a valid Ed25519 signature over `canonical_bytes`
   (`postcard`, ADR-0014) — reproducible so downstream verification holds.
3. **Order (FR-1.1):** `validate` precedes `score` precedes `sign` precedes `emit`.
4. **Raw stays node-local:** `RawReading`/`ThermalFrame` never leave the coherence domain
   (ADR-0009, NFR-7); only `ThermalBalance` (aggregate) is published to the Market.

## 5. Domain Events

`FrameValidated`, `FrameRejected(reason)`, `FrameWitnessed`, `FrameEmitted`.

## 6. Ports

| Port | Direction | Adapter | Pattern |
|------|-----------|---------|---------|
| `SensorSource`, `Validator`, `QualityScorer` | in | `RvcsiIngestAdapter` (@ruv/rvcsi) | ACL over rvcsi (CsiFrame→ThermalFrame) |
| `WitnessSigner` | out | `Ed25519WitnessAdapter` | uses RuView witness chain |
| `EventEmitter`, `RejectionLog` | out | `TypedEventBus` (tokio mpsc) | downstream publish |

## 7. Relationships

- **→ Forecasting:** Customer/Supplier — publishes `ReadingWindow`.
- **→ Thermal Market:** Published Language — exposes aggregated `ThermalBalance` only.
- **Translation note:** rvcsi's `CsiFrame`/WiFi-subcarrier vocabulary is translated by the adapter
  into the thermal ubiquitous language; vendor terms do not enter `hoforras-domain`.
