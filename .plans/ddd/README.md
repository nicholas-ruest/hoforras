# Domain-Driven Design — Hőforrás Budapest

Domain model for the Hőforrás peer-to-peer thermal energy intelligence system, derived from
`.plans/sparc.md` and constrained by the decisions in `.plans/adr/`.

These documents express the **strategic** design (how the problem decomposes into subdomains and
bounded contexts) and the **tactical** design per context (aggregates, entities, value objects,
domain events, domain services, repositories, and invariants).

## Documents

| Doc | Context | Subdomain type |
|-----|---------|----------------|
| [ddd-00-strategic-design](ddd-00-strategic-design.md) | *all* — subdomains, context map, ubiquitous language | — |
| [ddd-01-thermal-market](ddd-01-thermal-market.md) | Thermal Market & Brokerage | **Core** |
| [ddd-02-sensing-ingestion](ddd-02-sensing-ingestion.md) | Sensing & Ingestion | Supporting |
| [ddd-03-forecasting](ddd-03-forecasting.md) | Forecasting & Anomaly | Supporting |
| [ddd-04-node-isolation-security](ddd-04-node-isolation-security.md) | Node Isolation & Security | Supporting (generic substrate) |
| [ddd-05-trade-consensus](ddd-05-trade-consensus.md) | Trade Consensus | Generic |
| [ddd-06-thermal-memory](ddd-06-thermal-memory.md) | Thermal Memory | Generic |
| [ddd-07-district-mesh](ddd-07-district-mesh.md) | District Mesh Coordination | Supporting |
| [ddd-08-ai-orchestration](ddd-08-ai-orchestration.md) | AI Orchestration | Generic |
| [ddd-09-operator-experience](ddd-09-operator-experience.md) | Operator Experience | Supporting |

## How DDD maps to the rest of the plan

- **Bounded context ≈ Cargo crate / SPARC layer.** Each context's tactical model is realized by the
  units in `sparc.md` Part 2 and housed per `sparc.md` Part 3 §4.
- **Ports = context boundaries.** The hexagonal ports (ADR-0002/0003) are the technical form of the
  context map relationships below; an anti-corruption layer is an adapter.
- **Invariants = property tests.** Aggregate invariants here are the property suite in
  `sparc.md` Part 4 §16 (R14).
