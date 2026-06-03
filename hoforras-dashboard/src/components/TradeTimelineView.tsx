// TradeTimelineView (FR-9.4) — active agreements with their QuDAG consensus status.

import { AnimatePresence, motion } from "framer-motion";
import type { TradeTimelineEntry } from "../store";

const short = (id: string) => id.split(".")[0];

export function TradeTimelineView({ trades }: { trades: TradeTimelineEntry[] }) {
  const recent = [...trades].reverse().slice(0, 12);
  return (
    <section aria-label="trade timeline">
      {recent.length === 0 && (
        <p className="py-6 text-center text-[11px] uppercase tracking-wider text-slate-600">awaiting first trade…</p>
      )}
      <ol className="flex flex-col gap-1.5">
        <AnimatePresence initial={false}>
          {recent.map((t, i) => (
            <motion.li
              key={`${trades.length - i}-${t.seller}-${t.buyer}`}
              data-testid="trade-row"
              initial={{ opacity: 0, y: -6 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.25 }}
              className="flex items-center gap-2 rounded-md border border-[hsl(220_15%_18%/0.5)] bg-[hsl(220_20%_8%/0.4)] px-2 py-1.5 font-mono text-[11px]"
            >
              <span className="text-slate-300">
                {short(t.seller)} <span style={{ color: "hsl(24 90% 60%)" }}>→</span> {short(t.buyer)}
              </span>
              <span data-testid="kwh" className="ml-auto font-semibold" style={{ color: "hsl(24 90% 62%)" }}>
                {t.kwh.toFixed(1)} kWh
              </span>
              <span data-testid="consensus-status" className="badge online">
                {t.consensusStatus}
              </span>
            </motion.li>
          ))}
        </AnimatePresence>
      </ol>
    </section>
  );
}
