// WitnessLogView (FR-9.5) — a read-only view of the rvm-witness audit trail for a junction. Opened
// by drilling into the pipe-health heatmap; keeps the autonomous system accountable (DDD-09 §6).

import type { JunctionId } from "../events";

export interface WitnessLogEntry {
  /** Hex digest of a 64-byte WitnessRecord (read-only projection). */
  digest: string;
  action: string;
}

export function WitnessLogView({
  junction,
  entries,
  onClose,
}: {
  junction: JunctionId;
  entries: WitnessLogEntry[];
  onClose: () => void;
}) {
  return (
    <aside aria-label="witness log" data-testid="witness-log">
      <header>
        <h3>Witness log — junction {junction}</h3>
        <button onClick={onClose}>close</button>
      </header>
      <ul>
        {entries.map((e, i) => (
          <li key={i} data-testid="witness-entry">
            <code>{e.digest}</code> — {e.action}
          </li>
        ))}
      </ul>
    </aside>
  );
}
