# DDD-07 — District Mesh Coordination (Supporting)

**Subdomain type:** Supporting · **Crate:** `hoforras-mesh` · **SPARC unit:** `MeshCoordinator`
(Part 2 §8) · **Key ADRs:** 0007 (DAG over RPC)

The district-scale fabric that propagates state and enables collective, self-healing behaviour
across all Appliances. Domain-shaped (thermal load balancing, cascade prevention) over the generic
Synaptic-Mesh substrate.

## 1. Aggregates

### `MeshTopology` (aggregate root, district-scoped)
The live view of which Appliances are participating.

- **State:** `NodeRegistry` (live set), committed market/anomaly/consensus state propagated as DAG
  entries.
- **Behaviour:** self-heal on node drop (reconfigure over the live set) and resync on return (replay
  missed entries) — without losing committed state (NFR-9/10).

## 2. Value Objects

`DagEntry` (signed; the unit of propagation) · `NodeEvent = Drop(n) | Up(n)` ·
`CollectiveSignal = Overload(zone) | CascadeRisk(path) | HeatWave`.

## 3. Domain Services

- **`MeshCoordinator`** — `propagate(state)` (publish a signed DAG entry, never RPC),
  `on_node_event` (mark down/up + reconfigure/resync), `collective_behaviour(signal)` →
  rebalance / shed-and-reroute / emergency-routing-plan.

## 4. Invariants (→ Part 4 R10 / R14)

1. **DAG-not-RPC (FR-7.1):** cross-node propagation occurs via `MeshTransport.publish`; there is no
   RPC port (ADR-0007).
2. **Self-heal without state loss (FR-7.2/NFR-9):** after `Drop(n)` then `Up(n)`, survivors' committed
   state is unchanged and `n` converges to the same state (eventual consistency).
3. **Collective dispatch (FR-7.3):** each `CollectiveSignal` maps to its mitigation behaviour.

## 5. Domain Events

`NodeDropped`, `NodeRejoined`, `StateResynced`, `LoadRebalanced`, `CascadePrevented`,
`EmergencyRoutingEngaged`.

## 6. Ports

| Port | Adapter | Substrate |
|------|---------|-----------|
| `MeshTransport`, `NodeRegistry` | `SynapticMeshAdapter` | Synaptic-Mesh (DAG fabric) |

## 7. Relationships

- **↔ Thermal Market / Trade Consensus:** Published Language — signed DAG entries (trades, anomaly
  signals, consensus decisions) are the shared medium.
- **↔ peer Appliances:** Partnership — a peer-to-peer mesh, with `.dark` discovery (no directory).
- **→ Operator Experience:** the mesh's `trade:executed` / `anomaly:detected` events feed the
  dashboard.
