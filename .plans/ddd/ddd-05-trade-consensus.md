# DDD-05 — Trade Consensus (Generic)

**Subdomain type:** Generic (off-the-shelf QuDAG) · **Crate:** `hoforras-mesh` ·
**SPARC unit:** `ConsensusGateway` (Part 2 §6) · **Key ADRs:** 0011 (post-quantum), 0007, 0014

A **generic** subdomain: post-quantum DAG consensus is a solved capability supplied by QuDAG. Hőforrás
wraps it behind an **anti-corruption layer** and contributes only the trade-agreement model.

## 1. Aggregates

### `ThermalTradeAgreement` (aggregate root)
The signed, tamper-evident, consensus-final record of a cross-building trade.

- **Identity:** the DAG entry hash.
- **State (VO payload):** `{ seller: NodeId, buyer: NodeId, kwh_offered, duration_hours ≤ 6,
  credit_price_per_kwh, pipe_route: [JunctionId], valid_from }`.
- **Lifecycle:** `Unsigned → Signed(ML-DSA) → Broadcast → Final(QR-Avalanche)`.
- **Rule:** an agreement is actionable by the market only once `Final`.

## 2. Value Objects

`DagEntry` · `MlDsaSignature` · `Finality{ reached_at }` · `NodeId` (`.dark` domain) ·
`JunctionId` · `NodeAddr` (resolved peer).

## 3. Domain Services

- **`ConsensusGateway`** (the ACL) — `finalize(trade)` (sign **then** broadcast), `resolve_peer(dark)`
  (Kademlia/`.dark`), `verify(entry)` (tamper check). Validates `duration ≤ 6h` at the boundary as
  defense in depth against the market's own rules.

## 4. Invariants (→ Part 4 R2 / R14)

1. **Sign-before-broadcast (FR-5.1):** `MlDsaSigner.sign` precedes `DagNetwork.broadcast`.
2. **Post-quantum (FR-5.2/NFR-6):** signatures are ML-DSA; transport is ML-KEM-1024 (ADR-0011).
3. **Tamper-evidence (FR-5.5):** mutating an entry ⇒ `verify() == false`.
4. **Directory-less discovery (FR-5.4):** peers resolved via `.dark` domains; no directory port
   exists.
5. **Sub-second finality (NFR-2):** measured on hardware (Phase C), not against mocks.

## 5. Domain Events

`AgreementSigned`, `AgreementBroadcast`, `ConsensusReached`.

## 6. Ports

| Port | Adapter | Substrate |
|------|---------|-----------|
| `MlDsaSigner`, `DagNetwork`, `PeerDiscovery` | `QuDagAdapter` | QuDAG |

## 7. Relationships

- **← Thermal Market:** Anti-Corruption Layer — the market calls `ConsensusGateway.finalize` and
  receives `Finality`; QuDAG's vocabulary never leaks into the market's model.
- **→ District Mesh:** the signed `DagEntry` is the Published Language propagated across the mesh
  (ADR-0007).
- **Generic-by-design:** no custom consensus algorithm; the contribution is solely the
  `ThermalTradeAgreement` schema and the boundary validation.
