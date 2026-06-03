// The conformist-downstream event contract (DDD-09 / ADR-0007).
//
// These TypeScript types mirror the published `UiEvent` shapes emitted by the Mesh / AI Orchestration
// contexts (hoforras-domain::UiEvent), and the socket event *names* from sparc.md Part 2 §10. The
// dashboard does NOT ask upstream to change — it conforms to whatever is published and defensively
// ignores anything that does not match the contract (read-only, fault-tolerant).

/** A `.dark` building/Appliance identity (e.g. `pozsonyi14.thermal.budapest.dark`). */
export type NodeId = string;
/** A pipe-network junction id. */
export type JunctionId = number;

/** `trade:executed` — mirrors `UiEvent::TradeExecuted`. */
export interface TradeExecutedEvent {
  type: "trade:executed";
  seller: NodeId;
  buyer: NodeId;
  kwh: number;
  pipe_route: JunctionId[];
}

/** `anomaly:detected` — mirrors `UiEvent::AnomalyDetected`. */
export interface AnomalyDetectedEvent {
  type: "anomaly:detected";
  node_id: NodeId;
}

/** `anomaly:explained` — mirrors `UiEvent::AnomalyExplained`. */
export interface AnomalyExplainedEvent {
  type: "anomaly:explained";
  node_id: NodeId;
  explanation: { text: string };
}

export type UiEvent =
  | TradeExecutedEvent
  | AnomalyDetectedEvent
  | AnomalyExplainedEvent;

/** The socket event names the dashboard subscribes to (the published language). */
export const EVENT_NAMES = [
  "trade:executed",
  "anomaly:detected",
  "anomaly:explained",
] as const;
export type EventName = (typeof EVENT_NAMES)[number];

function isString(v: unknown): v is string {
  return typeof v === "string";
}
function isNumber(v: unknown): v is number {
  return typeof v === "number" && Number.isFinite(v);
}

/**
 * Parse a raw socket message into a typed `UiEvent` — or `null` if it does not conform to the
 * published shape (ADR-0007 event-shape conformance). The dashboard never throws on a malformed or
 * unknown message; it simply drops it.
 */
export function parseUiEvent(name: string, payload: unknown): UiEvent | null {
  if (typeof payload !== "object" || payload === null) return null;
  const p = payload as Record<string, unknown>;

  switch (name) {
    case "trade:executed": {
      if (
        isString(p.seller) &&
        isString(p.buyer) &&
        isNumber(p.kwh) &&
        Array.isArray(p.pipe_route) &&
        p.pipe_route.every(isNumber)
      ) {
        return {
          type: "trade:executed",
          seller: p.seller,
          buyer: p.buyer,
          kwh: p.kwh,
          pipe_route: p.pipe_route as JunctionId[],
        };
      }
      return null;
    }
    case "anomaly:detected": {
      if (isString(p.node_id)) {
        return { type: "anomaly:detected", node_id: p.node_id };
      }
      return null;
    }
    case "anomaly:explained": {
      const exp = p.explanation as Record<string, unknown> | undefined;
      if (isString(p.node_id) && exp && isString(exp.text)) {
        return {
          type: "anomaly:explained",
          node_id: p.node_id,
          explanation: { text: exp.text },
        };
      }
      return null;
    }
    default:
      return null; // unknown event name — ignored
  }
}
