// R12 #1/#2 — dashboard behaviour against a MOCKED socket (FR-9.1/9.2/9.6).

import { describe, expect, it, vi } from "vitest";
import { wireDashboard } from "../controller";
import { createMockSocket } from "../socket";
import { DashboardStore, edgeKey } from "../store";

describe("dashboard controller (mocked socket)", () => {
  it("trade:executed updates the edge once and animates the route once", () => {
    const socket = createMockSocket();
    const store = new DashboardStore();
    const updateEdgeWeight = vi.spyOn(store, "updateEdgeWeight");
    const animateEnergyFlow = vi.spyOn(store, "animateEnergyFlow");
    wireDashboard(socket, { store, requestClaudeExplanation: vi.fn() });

    socket.emit("trade:executed", {
      seller: "pozsonyi14.thermal.budapest.dark",
      buyer: "pozsonyi22.thermal.budapest.dark",
      kwh: 40,
      pipe_route: [7, 12, 18],
    });

    expect(updateEdgeWeight).toHaveBeenCalledTimes(1);
    expect(animateEnergyFlow).toHaveBeenCalledTimes(1);
    expect(animateEnergyFlow).toHaveBeenCalledWith([7, 12, 18]);
    expect(
      store
        .getState()
        .edges.get(
          edgeKey(
            "pozsonyi14.thermal.budapest.dark",
            "pozsonyi22.thermal.budapest.dark",
          ),
        ),
    ).toBe(40);
  });

  it("anomaly:detected highlights the node and requests a Claude explanation", () => {
    const socket = createMockSocket();
    const store = new DashboardStore();
    const requestClaudeExplanation = vi.fn();
    wireDashboard(socket, { store, requestClaudeExplanation });

    socket.emit("anomaly:detected", {
      node_id: "pozsonyi30.thermal.budapest.dark",
    });

    expect(store.getState().highlightedNodes.has("pozsonyi30.thermal.budapest.dark")).toBe(
      true,
    );
    expect(requestClaudeExplanation).toHaveBeenCalledTimes(1);
    expect(requestClaudeExplanation).toHaveBeenCalledWith(
      "pozsonyi30.thermal.budapest.dark",
    );
  });

  it("anomaly:explained attaches the explanation to the feed", () => {
    const socket = createMockSocket();
    const store = new DashboardStore();
    wireDashboard(socket, { store, requestClaudeExplanation: vi.fn() });

    socket.emit("anomaly:detected", { node_id: "n.dark" });
    socket.emit("anomaly:explained", {
      node_id: "n.dark",
      explanation: { text: "pipe stress rising near junction 12" },
    });

    const feed = store.getState().anomalies;
    expect(feed).toHaveLength(1);
    expect(feed[0].explanation).toContain("pipe stress");
  });

  it("is read-only: a non-conforming frame is dropped, no projection changes", () => {
    const socket = createMockSocket();
    const store = new DashboardStore();
    const updateEdgeWeight = vi.spyOn(store, "updateEdgeWeight");
    wireDashboard(socket, { store, requestClaudeExplanation: vi.fn() });

    socket.emit("trade:executed", { seller: "only-seller" }); // missing fields ⇒ dropped

    expect(updateEdgeWeight).not.toHaveBeenCalled();
    expect(store.getState().trades).toHaveLength(0);
  });
});
