// COGS-style shared UI primitives: Card, Kpi, Appbar.

import type { ReactNode } from "react";

export function Card({
  title,
  sub,
  right,
  children,
  className = "",
}: {
  title: string;
  sub?: string;
  right?: ReactNode;
  children: ReactNode;
  className?: string;
}) {
  return (
    <section className={`card flex min-h-0 flex-col ${className}`}>
      <header className="card-h">
        <span className="card-title">{title}</span>
        {sub && <span className="card-sub">{sub}</span>}
        {right && <span className="ml-auto">{right}</span>}
      </header>
      <div className="min-h-0 flex-1 overflow-auto p-3">{children}</div>
    </section>
  );
}

export function Kpi({
  icon,
  label,
  value,
  sub,
  tone = "",
}: {
  icon: ReactNode;
  label: string;
  value: ReactNode;
  sub?: string;
  tone?: "" | "accent" | "thermal";
}) {
  return (
    <div className="kpi">
      <div className={`kpi-ic ${tone}`}>{icon}</div>
      <div className="min-w-0 flex-1">
        <div className="kpi-label">{label}</div>
        <div className="kpi-val">{value}</div>
        {sub && <div className="kpi-sub">{sub}</div>}
      </div>
    </div>
  );
}

export type DashboardMode = "live" | "planner";

export function Appbar({
  mode,
  onMode,
  hasPlanner,
  seedsOnline,
  seedsTotal,
}: {
  mode: DashboardMode;
  onMode: (m: DashboardMode) => void;
  hasPlanner: boolean;
  seedsOnline: number;
  seedsTotal: number;
}) {
  return (
    <header className="sticky top-0 z-[1000] border-b border-[hsl(220_15%_18%/0.5)] bg-[hsl(220_25%_6%/0.9)] px-4 py-2.5 backdrop-blur">
      <div className="mx-auto flex max-w-[1500px] flex-wrap items-center gap-3">
        <div className="flex items-center gap-2.5">
          <div
            className="flex h-8 w-8 items-center justify-center rounded-lg text-lg"
            style={{ background: "hsl(24 90% 55% / 0.14)" }}
          >
            🔥
          </div>
          <div className="text-[15px] leading-none">
            <span className="text-slate-400">Hőforrás</span>
            <span className="mx-1.5 font-semibold" style={{ color: "hsl(185 80% 55%)" }}>
              /
            </span>
            <b className="text-slate-100">District XIII</b>
            <span className="ml-2 font-mono text-[11px] text-slate-500">Budapest · v0.1</span>
          </div>
        </div>

        <nav className="ml-3 flex gap-1">
          <button
            data-testid="mode-live"
            onClick={() => onMode("live")}
            className={`btn ${mode === "live" ? "on" : ""}`}
            style={{ fontSize: "0.78rem", padding: "0.35rem 0.8rem" }}
          >
            ● Live Grid
          </button>
          {hasPlanner && (
            <button
              data-testid="mode-planner"
              onClick={() => onMode("planner")}
              className={`btn ${mode === "planner" ? "on" : ""}`}
              style={{ fontSize: "0.78rem", padding: "0.35rem 0.8rem" }}
            >
              ◈ Recovery Planner
            </button>
          )}
        </nav>

        <div className="ml-auto flex items-center gap-2">
          <span className="badge online">
            <span className="dot up" /> mesh online
          </span>
          <span className="badge primary">
            {seedsOnline}/{seedsTotal} SEEDs
          </span>
        </div>
      </div>
    </header>
  );
}
