# ADR-0011 — Post-quantum crypto (ML-DSA / ML-KEM-1024) for trade consensus

- **Status:** Accepted
- **Date:** 2026-06-02
- **Traces:** `sparc.md` Part 1 FR-5.1–5.5, NFR-2, NFR-6; Part 3 §8
- **Constrains DDD:** Trade Consensus context
- **Related:** ADR-0007, ADR-0014

## Context

Cross-building thermal trade agreements are long-lived, financially-meaningful, and forensically
auditable commitments. They must be tamper-evident and verifiable for years. Classical signatures
(Ed25519/ECDSA) are vulnerable to future quantum adversaries ("harvest now, decrypt later"). QuDAG
provides post-quantum primitives and DAG-based consensus out of the box.

## Decision

Use **QuDAG** for the trade-agreement consensus: **ML-DSA** signatures on DAG entries, **ML-KEM-1024**
for transport encryption, **QR-Avalanche** for sub-second finality (NFR-2), and **Kademlia DHT over
`.dark` domains** for directory-less peer discovery (FR-5.4). Reached through the
`MlDsaSigner` / `DagNetwork` / `PeerDiscovery` ports (an anti-corruption layer around QuDAG).

Note: sensor-reading authenticity continues to use **Ed25519** witnessing (FR-1.3) — a separate,
shorter-lived concern. Post-quantum strength is applied where commitments are durable (trades).

## Consequences

**Positive**
- Trade agreements remain verifiable against future quantum threats (NFR-6).
- Directory-less discovery removes a central failure point (aligns with ADR-0007).

**Negative / costs**
- ML-DSA signatures/keys are larger than Ed25519; bandwidth/storage overhead per entry. Acceptable
  given low trade frequency.
- Finality budget (<1 s) is validated only on real hardware (Phase C), not in unit tests.

## Alternatives considered
- **Classical signatures everywhere** — rejected for durable trade commitments (quantum risk).
- **Post-quantum for sensor readings too** — rejected as over-engineering: readings are
  short-lived, high-volume; Ed25519 is sufficient and cheaper there.
