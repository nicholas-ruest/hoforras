// PipeHealthHeatmap (FR-9.5) — per-junction health, with drill-down to the witness log.

import { useState } from "react";
import type { JunctionId } from "../events";
import type { PipeHealth } from "../store";
import { WitnessLogView, type WitnessLogEntry } from "./WitnessLogView";

export type WitnessLogProvider = (junction: JunctionId) => WitnessLogEntry[];

function healthColor(score: number): string {
  const hue = Math.round(Math.max(0, Math.min(1, score)) * 130); // red → green
  return `hsl(${hue} 70% 52%)`;
}

export function PipeHealthHeatmap({
  pipes,
  witnessLogFor,
}: {
  pipes: PipeHealth[];
  witnessLogFor: WitnessLogProvider;
}) {
  const [drilled, setDrilled] = useState<JunctionId | null>(null);
  return (
    <section aria-label="pipe health heatmap">
      <div data-testid="heatmap" className="grid grid-cols-4 gap-1.5">
        {pipes.map((p) => {
          const c = healthColor(p.score);
          const selected = drilled === p.junction;
          return (
            <button
              key={p.junction}
              data-testid={`pipe-${p.junction}`}
              onClick={() => setDrilled(selected ? null : p.junction)}
              className="flex flex-col items-center rounded-md border px-1 py-1.5 transition"
              style={{ borderColor: selected ? c : `${c}44`, background: `${c}14` }}
            >
              <span className="text-[10px] font-semibold text-slate-300">J{p.junction}</span>
              <span className="font-mono text-sm font-semibold" style={{ color: c }}>
                {(p.score * 100).toFixed(0)}%
              </span>
            </button>
          );
        })}
      </div>
      {drilled !== null && (
        <WitnessLogView junction={drilled} entries={witnessLogFor(drilled)} onClose={() => setDrilled(null)} />
      )}
    </section>
  );
}
