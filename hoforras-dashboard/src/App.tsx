// DistrictDashboard (DDD-09 §10 / Part 2 §10) — composes the read-only views over the operator feed.
//
// Read-only / conformist: it connects to the ruflo WebSocket, wires inbound events to the projection
// store, and renders. It never writes back — the operator observes an autonomous system (AC-7).

import { useEffect, useMemo } from "react";
import { AnomalyFeedView } from "./components/AnomalyFeedView";
import { BuildingCardView } from "./components/BuildingCardView";
import {
  DistrictGraphView,
  type GraphTopology,
} from "./components/DistrictGraphView";
import {
  PipeHealthHeatmap,
  type WitnessLogProvider,
} from "./components/PipeHealthHeatmap";
import { TradeTimelineView } from "./components/TradeTimelineView";
import { wireDashboard, type RequestClaudeExplanation } from "./controller";
import { DashboardStore } from "./store";
import { WebSocketOperatorFeed, type OperatorSocket } from "./socket";
import { useDashboard } from "./useStore";

export interface DashboardProps {
  store: DashboardStore;
  topology: GraphTopology;
  /** Injected so tests can supply a mock socket; defaults to the live ruflo feed. */
  makeSocket?: () => OperatorSocket;
  requestClaudeExplanation: RequestClaudeExplanation;
  witnessLogFor: WitnessLogProvider;
}

export function DistrictDashboard({
  store,
  topology,
  makeSocket,
  requestClaudeExplanation,
  witnessLogFor,
}: DashboardProps) {
  const state = useDashboard(store);
  const socket = useMemo(
    () => (makeSocket ? makeSocket() : new WebSocketOperatorFeed()),
    [makeSocket],
  );

  useEffect(() => {
    const dispose = wireDashboard(socket, { store, requestClaudeExplanation });
    return dispose;
  }, [socket, store, requestClaudeExplanation]);

  return (
    <main>
      <DistrictGraphView topology={topology} state={state} />
      <BuildingCardView buildings={state.buildings} />
      <TradeTimelineView trades={state.trades} />
      <PipeHealthHeatmap pipes={state.pipes} witnessLogFor={witnessLogFor} />
      <AnomalyFeedView anomalies={state.anomalies} />
    </main>
  );
}
