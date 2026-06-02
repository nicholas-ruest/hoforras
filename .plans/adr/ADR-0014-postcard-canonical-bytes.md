# ADR-0014 — `postcard` canonical serialization for all signed bytes

- **Status:** Accepted
- **Date:** 2026-06-02
- **Traces:** `sparc.md` Part 3 §6; Part 2 §2 (`canonical_bytes`); research Cargo pins (`postcard = 1`, `serde`)
- **Constrains DDD:** Sensing & Ingestion, Trade Consensus, Node Isolation & Security
- **Related:** ADR-0011

## Context

Three subsystems sign or hash byte representations of domain objects: Ed25519 witnessing of
`ThermalFrame` (FR-1.3), ML-DSA signing of `DagEntry`/`ThermalTradeAgreement` (FR-5.1), and the
64-byte hash-chained witness records (FR-4.3). Signatures and hash chains are only verifiable if the
byte encoding is **deterministic and reproducible** across nodes and over time. A non-canonical
encoding (field reordering, float formatting drift) would break verification.

## Decision

Use **`postcard`** (with `serde` derive on all `hoforras-domain` types) as the single canonical wire
encoding for any bytes that are signed or hashed. `canonical_bytes(x)` is defined as
`postcard::to_allocvec(x)` and is the only serializer used for witnessing, consensus signing, and
witness-chain emission.

## Consequences

**Positive**
- Reproducible signatures/hashes across heterogeneous Appliances and software versions → reliable
  tamper-evidence (FR-5.5) and audit verification (FR-4.3).
- Compact, `no_std`-friendly encoding suited to edge devices.

**Negative / costs**
- Domain types must remain `serde`-stable; field reordering or type changes are signature-breaking
  and require versioning. Treated as a schema-evolution discipline.

## Alternatives considered
- **JSON** — rejected: non-canonical by default (key order, whitespace, float formatting), verbose.
- **Protobuf/bincode** — bincode lacks `postcard`'s `no_std`/compactness focus; protobuf adds schema
  tooling overhead. `postcard` is already a research pin and fits edge constraints.
