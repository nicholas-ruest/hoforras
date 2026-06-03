// ADR-0007 — event-shape conformance. The dashboard conforms to the published UiEvent shapes and
// defensively drops anything that does not match (it never asks upstream to change, never throws).

import { describe, expect, it } from "vitest";
import { parseUiEvent } from "../events";

describe("event-shape conformance (ADR-0007)", () => {
  it("parses a well-formed trade:executed event", () => {
    const e = parseUiEvent("trade:executed", {
      seller: "a.dark",
      buyer: "b.dark",
      kwh: 40,
      pipe_route: [7, 12],
    });
    expect(e).toEqual({
      type: "trade:executed",
      seller: "a.dark",
      buyer: "b.dark",
      kwh: 40,
      pipe_route: [7, 12],
    });
  });

  it("parses anomaly:detected and anomaly:explained", () => {
    expect(parseUiEvent("anomaly:detected", { node_id: "n.dark" })).toEqual({
      type: "anomaly:detected",
      node_id: "n.dark",
    });
    expect(
      parseUiEvent("anomaly:explained", {
        node_id: "n.dark",
        explanation: { text: "hi" },
      }),
    ).toEqual({
      type: "anomaly:explained",
      node_id: "n.dark",
      explanation: { text: "hi" },
    });
  });

  it("drops non-conforming and unknown events (returns null)", () => {
    expect(parseUiEvent("trade:executed", { seller: "a" })).toBeNull(); // missing fields
    expect(parseUiEvent("trade:executed", { seller: "a", buyer: "b", kwh: "40", pipe_route: [] })).toBeNull(); // wrong type
    expect(parseUiEvent("anomaly:detected", {})).toBeNull();
    expect(parseUiEvent("totally:unknown", { node_id: "n.dark" })).toBeNull();
    expect(parseUiEvent("trade:executed", null)).toBeNull();
  });
});
