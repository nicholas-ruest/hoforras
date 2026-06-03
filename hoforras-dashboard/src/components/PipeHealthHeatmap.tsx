// PipeHealthHeatmap (FR-9.5) — per-junction health, with drill-down to the witness log.

import { useState } from "react";
import type { JunctionId } from "../events";
import type { PipeHealth } from "../store";
import { WitnessLogView, type WitnessLogEntry } from "./WitnessLogView";

/** Resolves the witness-log entries for a junction (injected; reads the audit projection). */
export type WitnessLogProvider = (junction: JunctionId) => WitnessLogEntry[];

function healthColor(score: number): string {
  // green (healthy) → red (failing)
  const hue = Math.round(Math.max(0, Math.min(1, score)) * 120);
  return `hsl(${hue}, 70%, 45%)`;
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
      <div data-testid="heatmap">
        {pipes.map((p) => (
          <button
            key={p.junction}
            data-testid={`pipe-${p.junction}`}
            style={{ backgroundColor: healthColor(p.score) }}
            onClick={() => setDrilled(p.junction)}
          >
            J{p.junction}: {(p.score * 100).toFixed(0)}%
          </button>
        ))}
      </div>
      {drilled !== null && (
        <WitnessLogView
          junction={drilled}
          entries={witnessLogFor(drilled)}
          onClose={() => setDrilled(null)}
        />
      )}
    </section>
  );
}
