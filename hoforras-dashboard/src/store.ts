// The dashboard read models (DDD-09 §1) — projections, NOT write aggregates.
//
// Every field here is derived from inbound published events. There are no methods that write back to
// the backend — the dashboard is read-only over the system's events (DDD-09 §6). Components subscribe
// and re-render; the store never mutates domain state.

import type { JunctionId, NodeId } from "./events";

export interface EnergyFlow {
  pipe_route: JunctionId[];
  /** A monotonically increasing id so a component can animate each flow exactly once. */
  id: number;
}

export interface TradeTimelineEntry {
  seller: NodeId;
  buyer: NodeId;
  kwh: number;
  /** QuDAG consensus status as surfaced to the operator (FR-9.4). */
  consensusStatus: "final";
}

export interface AnomalyFeedEntry {
  node_id: NodeId;
  /** Filled in when the Claude explanation arrives (FR-9.6). */
  explanation?: string;
}

export interface BuildingCard {
  node_id: NodeId;
  surplus_kwh: number;
  deficit_kwh: number;
  /** 24 hourly demand values for the sparkline (FR-9.3). */
  demand24h: number[];
}

export interface PipeHealth {
  junction: JunctionId;
  score: number;
}

export interface DashboardState {
  /** Edge weights keyed by `"seller→buyer"` (FR-9.1). */
  edges: Map<string, number>;
  energyFlows: EnergyFlow[];
  highlightedNodes: Set<NodeId>;
  anomalies: AnomalyFeedEntry[];
  trades: TradeTimelineEntry[];
  buildings: BuildingCard[];
  pipes: PipeHealth[];
}

export function edgeKey(seller: NodeId, buyer: NodeId): string {
  return `${seller}→${buyer}`;
}

type Listener = () => void;

/** A tiny observable projection store (no external state lib). */
export class DashboardStore {
  private state: DashboardState = {
    edges: new Map(),
    energyFlows: [],
    highlightedNodes: new Set(),
    anomalies: [],
    trades: [],
    buildings: [],
    pipes: [],
  };
  private listeners = new Set<Listener>();
  private flowSeq = 0;

  getState(): DashboardState {
    return this.state;
  }

  subscribe(listener: Listener): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  private emit(): void {
    for (const l of this.listeners) l();
  }

  // ── projection actions (driven by the controller from inbound events) ──

  /** FR-9.1: set the seller→buyer edge weight from an executed trade. */
  updateEdgeWeight(seller: NodeId, buyer: NodeId, kwh: number): void {
    this.state.edges.set(edgeKey(seller, buyer), kwh);
    this.state.trades = [
      ...this.state.trades,
      { seller, buyer, kwh, consensusStatus: "final" },
    ];
    this.emit();
  }

  /** FR-9.2: register an energy-flow animation along the pipe route (once per trade). */
  animateEnergyFlow(pipe_route: JunctionId[]): EnergyFlow {
    const flow = { pipe_route, id: this.flowSeq++ };
    this.state.energyFlows = [...this.state.energyFlows, flow];
    this.emit();
    return flow;
  }

  /** FR-9.6: highlight a node as a warning on an anomaly. */
  highlightNode(node_id: NodeId): void {
    this.state.highlightedNodes = new Set(this.state.highlightedNodes).add(node_id);
    this.state.anomalies = [...this.state.anomalies, { node_id }];
    this.emit();
  }

  /** FR-9.6: attach a Claude explanation to the most recent matching anomaly. */
  addExplanation(node_id: NodeId, text: string): void {
    let attached = false;
    this.state.anomalies = this.state.anomalies.map((a) => {
      if (!attached && a.node_id === node_id && a.explanation === undefined) {
        attached = true;
        return { ...a, explanation: text };
      }
      return a;
    });
    if (!attached) {
      this.state.anomalies = [...this.state.anomalies, { node_id, explanation: text }];
    }
    this.emit();
  }

  /** Seed/refresh the read-only topology projections (building cards, pipe health). */
  setBuildings(buildings: BuildingCard[]): void {
    this.state.buildings = buildings;
    this.emit();
  }
  setPipes(pipes: PipeHealth[]): void {
    this.state.pipes = pipes;
    this.emit();
  }
}
