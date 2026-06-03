// App entry — mounts the COGS-style District XIII operator console over a map of Budapest.
//
// Local dev has no live Appliance feed, so the shell seeds the read models and uses a self-contained
// demo feed + a calibrated recovery report. In production, swap `makeSocket` for the live ruflo
// WebSocket and `recovery` for the `RecoveryPlanService` read API — nothing else changes.

import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "maplibre-gl/dist/maplibre-gl.css";
import "./index.css";
import { DistrictDashboard } from "./App";
import type { WitnessLogEntry } from "./components/WitnessLogView";
import { createDemoFeed } from "./demoFeed";
import { demoRecovery } from "./recovery";
import { DashboardStore } from "./store";

const N14 = "pozsonyi14.thermal.budapest.dark";
const N22 = "pozsonyi22.thermal.budapest.dark";
const N30 = "pozsonyi30.thermal.budapest.dark";

const store = new DashboardStore();

const sparkline = (base: number) =>
  Array.from({ length: 24 }, (_, h) => base + 6 * Math.sin((h * Math.PI) / 12) + (h % 3));
store.setBuildings([
  { node_id: N14, surplus_kwh: 64, deficit_kwh: 0, demand24h: sparkline(20) },
  { node_id: N22, surplus_kwh: 0, deficit_kwh: 22, demand24h: sparkline(28) },
  { node_id: N30, surplus_kwh: 4, deficit_kwh: 12, demand24h: sparkline(24) },
]);
store.setPipes([
  { junction: 7, score: 0.96 },
  { junction: 12, score: 0.41 },
  { junction: 18, score: 0.88 },
  { junction: 5, score: 0.73 },
]);

const requestClaudeExplanation = (node_id: string) =>
  console.info(`requesting Claude explanation for ${node_id}`);
const witnessLogFor = (junction: number): WitnessLogEntry[] => [
  { digest: `0x${junction.toString(16).padStart(4, "0")}f3a1…`, action: "TradeSigned" },
  { digest: `0x${junction.toString(16).padStart(4, "0")}b7c2…`, action: "Exec" },
  { digest: `0x${junction.toString(16).padStart(4, "0")}9d5e…`, action: "Routing" },
];

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <DistrictDashboard
      store={store}
      makeSocket={createDemoFeed}
      requestClaudeExplanation={requestClaudeExplanation}
      witnessLogFor={witnessLogFor}
      recovery={demoRecovery()}
    />
  </StrictMode>,
);
