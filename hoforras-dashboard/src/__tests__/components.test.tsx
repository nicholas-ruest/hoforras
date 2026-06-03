// R12 #3/#4/#5 — DOM component tests (FR-9.3/9.4/9.5).

import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";
import { AnomalyFeedView } from "../components/AnomalyFeedView";
import { BuildingCardView } from "../components/BuildingCardView";
import { PipeHealthHeatmap } from "../components/PipeHealthHeatmap";
import { TradeTimelineView } from "../components/TradeTimelineView";

describe("BuildingCardView (FR-9.3)", () => {
  it("renders surplus/deficit and a 24h sparkline per building", () => {
    render(
      <BuildingCardView
        buildings={[
          {
            node_id: "pozsonyi14.thermal.budapest.dark",
            surplus_kwh: 64,
            deficit_kwh: 0,
            demand24h: Array.from({ length: 24 }, (_, i) => i + 1),
          },
        ]}
      />,
    );
    expect(screen.getByTestId("surplus")).toHaveTextContent("64.0 kWh");
    expect(screen.getByTestId("deficit")).toHaveTextContent("0.0 kWh");
    expect(screen.getByLabelText("24h demand sparkline")).toBeInTheDocument();
  });
});

describe("TradeTimelineView (FR-9.4)", () => {
  it("reflects the QuDAG consensus status of each trade", () => {
    render(
      <TradeTimelineView
        trades={[
          {
            seller: "a.dark",
            buyer: "b.dark",
            kwh: 40,
            consensusStatus: "final",
          },
        ]}
      />,
    );
    expect(screen.getByTestId("consensus-status")).toHaveTextContent("final");
    expect(screen.getByTestId("kwh")).toHaveTextContent("40.0 kWh");
  });
});

describe("PipeHealthHeatmap (FR-9.5)", () => {
  it("drill-down on a junction opens the witness log", async () => {
    const user = userEvent.setup();
    render(
      <PipeHealthHeatmap
        pipes={[{ junction: 12, score: 0.4 }]}
        witnessLogFor={(j) => [{ digest: `0x${j}abc`, action: "Routing" }]}
      />,
    );
    // No witness log until drill-down.
    expect(screen.queryByTestId("witness-log")).not.toBeInTheDocument();

    await user.click(screen.getByTestId("pipe-12"));

    const log = screen.getByTestId("witness-log");
    expect(log).toBeInTheDocument();
    expect(screen.getByTestId("witness-entry")).toHaveTextContent("Routing");
  });
});

describe("AnomalyFeedView (FR-9.6)", () => {
  it("shows the Claude explanation once it arrives", () => {
    render(
      <AnomalyFeedView
        anomalies={[
          { node_id: "n.dark", explanation: "burst precursor near junction 18" },
        ]}
      />,
    );
    expect(screen.getByTestId("explanation")).toHaveTextContent("burst precursor");
  });
});
