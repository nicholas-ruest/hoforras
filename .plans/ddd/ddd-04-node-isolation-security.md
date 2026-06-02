# DDD-04 — Node Isolation & Security (Supporting; generic substrate)

**Subdomain type:** Supporting policy over a generic substrate (RVM) · **Crate:** `hoforras-node` ·
**SPARC units:** `CoherenceSupervisor`, `CapabilityBroker`, `AuditTrail` (Part 2 §5) ·
**Key ADRs:** 0004 (in-process), 0008 (deny-raw-first), 0014

This context is the **privacy and safety boundary** of the entire Appliance and acts as an
**Open Host Service / Shared Kernel** to every other context (it offers capabilities and witnessing
to all).

## 1. Aggregates

### 1.1 `CoherenceDomain` (aggregate root)
One per building; the RVM-isolated partition. The coherence domain **is** the Appliance's trust
boundary (ADR-0004).

- **State:** node graph + current `Partitioning`.
- **Behaviour:** on an anomalous signal, recompute mincut and isolate the offending node — with no
  manual step or restart (FR-4.2); other nodes keep serving.

### 1.2 `WitnessChain` (aggregate root)
The hash-chained audit log of privileged actions.

- **Rule:** append-only; each `WitnessRecord` is 64 bytes and links to the prior; `verify()`
  recomputes the chain (FR-4.3).

## 2. Entities & Value Objects

| | |
|---|---|
| `Capability` (VO) | `{ rights: Rights, scope: Scope, expiry: Expiry }` — unforgeable, scoped, expiring |
| `Scope` (VO) | `Raw` (never crosses) \| `AggregatedThermalAvailability` (may cross) |
| `WitnessRecord` (VO) | 64-byte hash-chained entry |
| `PrivilegedAction` (VO) | `TradeSigned \| Exec \| Routing \| ReadingAccepted \| NodeIsolated \| CrossPartitionRead` |
| `Partitioning` (VO) | result of a mincut over the node graph |
| `AccessRequest` (VO) | a cross-partition read request (scope + target) |

## 3. Domain Services

- **`CoherenceSupervisor`** — anomaly signal → `recompute` → `isolate`, witnessed.
- **`CapabilityBroker`** — mediates cross-partition reads; **denies `Raw` before consulting the
  gate** and checks expiry against an injected clock (ADR-0008).
- **`AuditTrail`** — `record(action)` and `verify_integrity()` over the witness chain.

## 4. Invariants (→ Part 4 R4/R5/R8 / R14)

1. **No raw egress (FR-4.4/NFR-7):** `scope == Raw` ⇒ always `Err`, and the capability gate is
   **not invoked** (ADR-0008). Only `AggregatedThermalAvailability` with a valid, unexpired
   capability returns data.
2. **Auto re-isolation (FR-4.2):** anomalous signal ⇒ exactly one `recompute` then one `isolate`;
   benign ⇒ neither; isolation is witnessed.
3. **Audit completeness/tamper-evidence (FR-4.3):** record then `verify()==true`; any tamper ⇒
   `false`; every record is 64 bytes.
4. **Continuity:** isolating one node does not halt the others.

## 5. Domain Events

`NodeIsolated(node, reason)`, `CapabilityIssued`, `CapabilityDenied(reason)`,
`ActionWitnessed(action)`, `CrossPartitionRead(request)`.

## 6. Ports

| Port | Adapter | Substrate |
|------|---------|-----------|
| `MincutEngine`, `PartitionController` | `RvmCoherenceAdapter` | rvm-coherence, rvm-kernel |
| `WitnessChain` | `RvmWitnessAdapter` | rvm-witness, rvm-proof |
| `CapabilityGate` | `RvmCapAdapter` | rvm-security, rvm-cap |
| `Clock` | injected | deterministic in tests |

## 7. Relationships

- **Open Host Service / Shared Kernel → all contexts:** `Capability`, `WitnessRecord`,
  `PrivilegedAction` live in `hoforras-domain` and are offered to every context. The Thermal Market
  witnesses every trade step here; Forecasting anomalies trigger isolation here.
- **Substrate is generic:** RVM provides the mechanism; the **policy** (which scopes may cross,
  deny-raw-first, what counts as a privileged action) is the domain contribution and is what gets
  modeled and tested.
