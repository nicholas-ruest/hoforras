// TradeTimelineView (FR-9.4) — active agreements with their QuDAG consensus status.

import type { TradeTimelineEntry } from "../store";

export function TradeTimelineView({ trades }: { trades: TradeTimelineEntry[] }) {
  return (
    <section aria-label="trade timeline">
      <ol>
        {trades.map((t, i) => (
          <li key={i} data-testid="trade-row">
            <span>
              {t.seller} → {t.buyer}
            </span>
            <span data-testid="kwh">{t.kwh.toFixed(1)} kWh</span>
            <span data-testid="consensus-status">{t.consensusStatus}</span>
          </li>
        ))}
      </ol>
    </section>
  );
}
