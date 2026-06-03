// The operator feed (DDD-09 §5) — the inbound, read-only WebSocket the dashboard conforms to.
//
// `OperatorSocket` is the seam the controller depends on (so tests inject a mock). The real feed
// connects to the ruflo WebSocket at `ws://appliance.district-xiii:3001`. The dashboard only ever
// *receives* — it never sends, matching the read-only / observe-only posture (DDD-09 §6).

import { parseUiEvent, type EventName, type UiEvent } from "./events";

export const DISTRICT_FEED_URL = "ws://appliance.district-xiii:3001";

export type UiEventHandler = (event: UiEvent) => void;

/** The inbound feed the dashboard subscribes to. Receive-only. */
export interface OperatorSocket {
  on(name: EventName, handler: UiEventHandler): void;
  close(): void;
}

/**
 * A real WebSocket-backed feed. Incoming frames are `{ name, payload }`; each is parsed against the
 * published contract and dispatched to the subscribed handler (non-conforming frames are dropped —
 * ADR-0007).
 */
export class WebSocketOperatorFeed implements OperatorSocket {
  private readonly ws: WebSocket;
  private readonly handlers = new Map<EventName, UiEventHandler[]>();

  constructor(url: string = DISTRICT_FEED_URL) {
    this.ws = new WebSocket(url);
    this.ws.onmessage = (msg) => this.dispatch(msg.data);
  }

  private dispatch(data: string): void {
    let frame: { name?: string; payload?: unknown };
    try {
      frame = JSON.parse(data);
    } catch {
      return; // malformed JSON — dropped
    }
    if (typeof frame.name !== "string") return;
    const event = parseUiEvent(frame.name, frame.payload);
    if (!event) return; // non-conforming — dropped
    for (const h of this.handlers.get(frame.name as EventName) ?? []) h(event);
  }

  on(name: EventName, handler: UiEventHandler): void {
    const list = this.handlers.get(name) ?? [];
    list.push(handler);
    this.handlers.set(name, list);
  }

  close(): void {
    this.ws.close();
  }
}

/** An in-memory `OperatorSocket` for tests — `emit` pushes a raw frame through the same parser. */
export function createMockSocket(): OperatorSocket & {
  emit(name: string, payload: unknown): void;
} {
  const handlers = new Map<EventName, UiEventHandler[]>();
  return {
    on(name, handler) {
      const list = handlers.get(name) ?? [];
      list.push(handler);
      handlers.set(name, list);
    },
    close() {
      handlers.clear();
    },
    emit(name, payload) {
      const event = parseUiEvent(name, payload);
      if (!event) return; // mirror the real feed: drop non-conforming frames
      for (const h of handlers.get(name as EventName) ?? []) h(event);
    },
  };
}
