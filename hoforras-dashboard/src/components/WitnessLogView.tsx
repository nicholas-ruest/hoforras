// WitnessLogView (FR-9.5) — read-only view of the rvm-witness audit trail for a junction.

import type { JunctionId } from "../events";

export interface WitnessLogEntry {
  digest: string;
  action: string;
}

const actionColor: Record<string, string> = {
  TradeSigned: "hsl(142 70% 58%)",
  Exec: "hsl(185 80% 60%)",
  Routing: "hsl(24 90% 60%)",
};

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
    <aside
      aria-label="witness log"
      data-testid="witness-log"
      className="mt-2 overflow-hidden rounded-lg border border-[hsl(220_15%_18%/0.7)] bg-[hsl(220_20%_8%/0.5)]"
    >
      <header className="flex items-center gap-2 border-b border-[hsl(220_15%_18%/0.5)] px-2.5 py-1.5">
        <span className="text-[11px] font-semibold text-slate-300">🔗 Witness log · junction {junction}</span>
        <button onClick={onClose} className="ml-auto rounded px-1.5 text-xs text-slate-500 hover:text-slate-200">
          ✕
        </button>
      </header>
      <ul className="flex flex-col gap-1 p-2 font-mono text-[10px]">
        {entries.map((e, i) => (
          <li key={i} data-testid="witness-entry" className="flex items-center gap-2">
            <span
              className="rounded px-1.5 py-0.5 font-semibold"
              style={{ color: actionColor[e.action] ?? "#c8d6f0", background: `${actionColor[e.action] ?? "#888"}1f` }}
            >
              {e.action}
            </span>
            <code className="truncate text-slate-500">{e.digest}</code>
            <span className="ml-auto" style={{ color: "hsl(142 70% 58%)" }} title="verified">
              ✓
            </span>
          </li>
        ))}
      </ul>
    </aside>
  );
}
