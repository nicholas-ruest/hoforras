// A self-contained demo feed for the dev shell (`npm run dev`).
//
// There is no live Appliance WebSocket in local dev, so this scripts a small, looping stream of the
// exact published `UiEvent` shapes — autonomous trades animating along pipe routes, an anomaly being
// detected and then explained by Claude. It drives the *same* conformist parser/controller path as
// the real feed; production simply swaps in `WebSocketOperatorFeed`.

import { parseUiEvent, type EventName, type UiEvent } from "./events";
import type { OperatorSocket } from "./socket";

const N14 = "pozsonyi14.thermal.budapest.dark";
const N22 = "pozsonyi22.thermal.budapest.dark";
const N30 = "pozsonyi30.thermal.budapest.dark";

type RawFrame = { name: string; payload: unknown };

// A looping script: surplus→deficit trades plus an anomaly detected then explained.
const SCRIPT: RawFrame[] = [
  { name: "trade:executed", payload: { seller: N14, buyer: N22, kwh: 40, pipe_route: [7, 12, 18] } },
  { name: "anomaly:detected", payload: { node_id: N30 } },
  {
    name: "anomaly:explained",
    payload: {
      node_id: N30,
      explanation: { text: "Pressure drop near junction 12 — possible pipe-burst precursor (0.78)." },
    },
  },
  { name: "trade:executed", payload: { seller: N22, buyer: N30, kwh: 18, pipe_route: [12, 5] } },
  { name: "trade:executed", payload: { seller: N14, buyer: N30, kwh: 25, pipe_route: [7, 5] } },
];

/** Build a demo `OperatorSocket` that emits the scripted stream on an interval. */
export function createDemoFeed(intervalMs = 2200): OperatorSocket {
  const handlers = new Map<EventName, ((e: UiEvent) => void)[]>();
  let i = 0;
  let timer: ReturnType<typeof setInterval> | null = null;

  const fire = (frame: RawFrame) => {
    const event = parseUiEvent(frame.name, frame.payload);
    if (!event) return;
    for (const h of handlers.get(frame.name as EventName) ?? []) h(event);
  };

  const start = () => {
    if (timer) return;
    // Kick off shortly after mount (handlers registered in the controller's effect), then loop.
    timer = setInterval(() => {
      fire(SCRIPT[i % SCRIPT.length]);
      i += 1;
    }, intervalMs);
  };

  return {
    on(name, handler) {
      const list = handlers.get(name) ?? [];
      list.push(handler);
      handlers.set(name, list);
      start();
    },
    close() {
      if (timer) clearInterval(timer);
      timer = null;
      handlers.clear();
    },
  };
}
