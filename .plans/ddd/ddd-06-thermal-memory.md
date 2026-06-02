# DDD-06 — Thermal Memory (Generic)

**Subdomain type:** Generic (off-the-shelf RuVector) · **Crate:** `hoforras-node` (node-local,
ADR-0009) · **SPARC unit:** `ThermalMemory` (Part 2 §7) · **Key ADRs:** 0009

A **generic** subdomain: vector/GNN storage and similarity search are off-the-shelf via RuVector.
The only domain decision is the **embedding scheme**; the rest is reused.

## 1. Aggregates

### `ThermalStateHistory` (aggregate root, per building)
The persistent memory of a building's thermal behaviour.

- **Holds:** time-series `ThermalFrame` state, 128-dim consumption-pattern embeddings, and the RVM
  witness trail (for forensic replay, FR-6.5).
- **Rule:** raw history stays node-local (ADR-0009); only embeddings/aggregates inform cross-building
  matching.

## 2. Value Objects

`Embedding(dim=128)` (deterministic map of normalized sensor channels + rolling stats) ·
`Query{ vector, topK, filter }` · `Candidate{ node, similarity }` ·
`DistrictGraph` (buildings = nodes, pipes = weighted edges) · `GnnState`.

## 3. Domain Services

- **`ThermalMemory`** — `record_state`, `find_surplus_matches(deficit_embedding, district)`,
  `district_inference(graph)` (GNN), `store_witness_for_replay`.

## 4. Invariants (→ Part 4 R9 / NFR-4)

1. **Embedding dimensionality (FR-6.2):** embeddings are 128-dim and deterministic (so similarity is
   stable and signatures/replay reproducible).
2. **Filtered search (FR-6.3):** results ≤ `topK`, all satisfy the filter, ordered by similarity.
3. **Sub-millisecond query (NFR-4):** HNSW/DiskANN; measured on hardware (Phase C).

## 5. Domain Events

`ThermalStateRecorded`, `SurplusMatchFound`, `GnnStateUpdated`.

## 6. Ports

| Port | Adapter | Substrate |
|------|---------|-----------|
| `VectorIndex`, `GnnEngine` | `RuVectorAdapter` | RuVector / ruvector (HNSW, GNN) |

## 7. Relationships

- **← Thermal Market:** Customer/Supplier (downstream) — the market writes state and queries
  surplus/deficit matches (e.g. "find buildings whose surplus matches building 22's deficit").
- **← Node Isolation:** stores the witness trail for forensic replay.
- **Generic-by-design:** the district *is* literally a graph, so GNN inference is run natively by
  RuVector; Hőforrás contributes the graph shaping and the embedding, nothing more.
