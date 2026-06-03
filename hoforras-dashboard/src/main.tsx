// App entry — mounts the read-only District XIII dashboard over the live ruflo feed.

import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { DistrictDashboard } from "./App";
import type { GraphTopology } from "./components/DistrictGraphView";
import type { WitnessLogEntry } from "./components/WitnessLogView";
import { DashboardStore } from "./store";

const store = new DashboardStore();

// Topology would be loaded from the district's published metadata; a small static seed for the shell.
const topology: GraphTopology = {
  nodes: [
    "pozsonyi14.thermal.budapest.dark",
    "pozsonyi22.thermal.budapest.dark",
    "pozsonyi30.thermal.budapest.dark",
  ],
  pipes: [
    [0, 1],
    [1, 2],
  ],
  junctions: [7, 12],
};

// In production these call the ThermalSense-Bridge MCP tools (P8 OHS); the shell uses no-op stubs so
// the read-only dashboard never writes back to the system.
const requestClaudeExplanation = (node_id: string) => {
  console.info(`requesting Claude explanation for ${node_id}`);
};
const witnessLogFor = (junction: number): WitnessLogEntry[] => [
  { digest: `0x${junction.toString(16).padStart(4, "0")}…`, action: "Routing" },
];

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <DistrictDashboard
      store={store}
      topology={topology}
      requestClaudeExplanation={requestClaudeExplanation}
      witnessLogFor={witnessLogFor}
    />
  </StrictMode>,
);
