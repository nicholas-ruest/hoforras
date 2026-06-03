// Render/update benchmark on a 20-node district graph (DDD-09 / Part 2 §10).
//
// Measures the projection-update cost of a trade across a 20-node / ~20-edge district — the hot path
// a `trade:executed` drives (updateEdgeWeight + animateEnergyFlow). WebGL render time is out of scope
// for the headless bench; this is the state-update budget that gates re-render frequency.

import { bench, describe } from "vitest";
import { DashboardStore } from "../store";

const NODES = Array.from({ length: 20 }, (_, i) => `b${i}.thermal.budapest.dark`);

describe("district projection update (20 nodes)", () => {
  bench("apply one trade:executed (edge + flow)", () => {
    const store = new DashboardStore();
    // Seed a 20-node ring of edges.
    for (let i = 0; i < NODES.length; i++) {
      store.updateEdgeWeight(NODES[i], NODES[(i + 1) % NODES.length], i);
    }
    // The measured operation: one more trade update + its energy-flow animation.
    store.updateEdgeWeight(NODES[3], NODES[7], 40);
    store.animateEnergyFlow([7, 12, 18]);
  });
});
