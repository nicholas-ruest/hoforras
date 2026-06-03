// The dashboard controller (DDD-09 §3) — wires the inbound feed to the read-model projections.
//
// This is the conformist application behaviour, isolated from any WebGL/DOM so it is unit-testable
// against a mocked socket:
//   • on `trade:executed`   → updateEdgeWeight + animateEnergyFlow(pipe_route)   (FR-9.1/9.2)
//   • on `anomaly:detected` → highlightNode(warning) + requestClaudeExplanation (FR-9.6)
//   • on `anomaly:explained`→ attach the explanation to the anomaly feed        (FR-9.6)

import type { OperatorSocket } from "./socket";
import type { DashboardStore } from "./store";

/** Asks the AI Orchestration context (P8) for a Claude explanation of an anomaly. */
export type RequestClaudeExplanation = (node_id: string) => void;

export interface ControllerDeps {
  store: DashboardStore;
  requestClaudeExplanation: RequestClaudeExplanation;
}

/** Subscribe the read models to the operator feed. Returns a disposer that closes the feed. */
export function wireDashboard(
  socket: OperatorSocket,
  { store, requestClaudeExplanation }: ControllerDeps,
): () => void {
  socket.on("trade:executed", (event) => {
    if (event.type !== "trade:executed") return;
    store.updateEdgeWeight(event.seller, event.buyer, event.kwh); // FR-9.1 — once
    store.animateEnergyFlow(event.pipe_route); // FR-9.2 — once
  });

  socket.on("anomaly:detected", (event) => {
    if (event.type !== "anomaly:detected") return;
    store.highlightNode(event.node_id); // FR-9.6
    requestClaudeExplanation(event.node_id); // FR-9.6 — ask Claude (via P8 OHS)
  });

  socket.on("anomaly:explained", (event) => {
    if (event.type !== "anomaly:explained") return;
    store.addExplanation(event.node_id, event.explanation.text);
  });

  return () => socket.close();
}
