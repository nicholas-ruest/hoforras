// AnomalyFeedView (FR-9.6) — detected anomalies with their Claude explanations as they arrive.

import { AnimatePresence, motion } from "framer-motion";
import type { AnomalyFeedEntry } from "../store";

const short = (id: string) => id.split(".")[0];

export function AnomalyFeedView({ anomalies }: { anomalies: AnomalyFeedEntry[] }) {
  const recent = [...anomalies].reverse().slice(0, 8);
  return (
    <section aria-label="anomaly feed">
      {recent.length === 0 && (
        <p className="py-6 text-center text-[11px] uppercase tracking-wider text-slate-600">
          no anomalies · grid nominal
        </p>
      )}
      <ul className="flex flex-col gap-2">
        <AnimatePresence initial={false}>
          {recent.map((a, i) => {
            const pending = a.explanation === undefined;
            return (
              <motion.li
                key={`${anomalies.length - i}-${a.node_id}`}
                data-testid="anomaly-row"
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                className="rounded-lg border border-[hsl(0_65%_50%/0.25)] bg-[hsl(0_65%_50%/0.06)] p-2.5"
              >
                <div className="flex items-center gap-2">
                  <span className="text-[13px] font-semibold" style={{ color: "hsl(0 70% 70%)" }}>
                    {short(a.node_id)}
                  </span>
                  <span className={`badge ${pending ? "danger" : "advisory"} ml-auto`}>
                    {pending ? "analysing" : "explained"}
                  </span>
                </div>
                <p
                  data-testid="explanation"
                  className={`mt-1 text-[11px] leading-snug ${pending ? "italic text-slate-500" : "text-slate-300"}`}
                >
                  {a.explanation ?? "awaiting Claude explanation…"}
                </p>
              </motion.li>
            );
          })}
        </AnimatePresence>
      </ul>
    </section>
  );
}
