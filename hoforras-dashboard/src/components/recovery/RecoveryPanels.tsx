// Recovery Planner panels (DDD-10 / ADR-0016) — read-only, estimate-grade decision support.

import type { DeviceKind, RecoveryReport } from "../../recovery";

const fmtEur = (n: number) =>
  new Intl.NumberFormat("en-IE", { style: "currency", currency: "EUR", maximumFractionDigits: 0 }).format(n);
const fmtKwh = (n: number) => `${Math.round(n).toLocaleString()} kWh/yr`;

const deviceMeta: Record<DeviceKind, { glyph: string; color: string; label: string }> = {
  RecoveryUnit: { glyph: "♨", color: "hsl(24 90% 60%)", label: "Recovery unit" },
  SeedSensor: { glyph: "◈", color: "hsl(185 80% 60%)", label: "SEED sensor" },
  Appliance: { glyph: "▣", color: "hsl(142 70% 58%)", label: "Appliance" },
};

export function AdvisoryBadge() {
  return (
    <span data-testid="advisory-badge" className="badge advisory">
      advisory · estimates only
    </span>
  );
}

export function SitingPlanView({ report }: { report: RecoveryReport }) {
  const counts = report.plan.placements.reduce(
    (acc, p) => ((acc[p.kind] = (acc[p.kind] ?? 0) + 1), acc),
    {} as Record<DeviceKind, number>,
  );
  return (
    <section aria-label="siting plan" className="flex flex-col gap-2.5">
      <div className="flex items-center justify-between">
        <div>
          <span className="font-display text-2xl font-semibold" style={{ color: "hsl(142 70% 58%)" }}>
            {Math.round(report.plan.coverage_pct * 100)}%
          </span>
          <span className="ml-1 kpi-label">loss covered</span>
        </div>
        <span data-testid="observed-loss" className="font-mono text-[11px] text-slate-500">
          {fmtKwh(report.plan.observed_loss_kwh_yr)} observed
        </span>
      </div>
      <div className="grid grid-cols-3 gap-2">
        {(Object.keys(deviceMeta) as DeviceKind[]).map((k) => (
          <div key={k} className="rounded-lg border border-[hsl(220_15%_18%/0.6)] bg-[hsl(220_20%_8%/0.4)] px-2 py-2 text-center">
            <div className="text-lg" style={{ color: deviceMeta[k].color }}>{deviceMeta[k].glyph}</div>
            <div className="font-display text-base font-semibold text-slate-100">{counts[k] ?? 0}</div>
            <div className="kpi-label">{deviceMeta[k].label}</div>
          </div>
        ))}
      </div>
      <ul className="flex flex-col gap-1 text-[10px]">
        {report.plan.placements.map((p, i) => (
          <li key={i} className="flex items-center gap-1.5 font-mono">
            <span style={{ color: deviceMeta[p.kind].color }}>{deviceMeta[p.kind].glyph}</span>
            <span className="text-slate-500">J{p.junction}</span>
            <span className="truncate text-slate-600">{p.rationale}</span>
          </li>
        ))}
      </ul>
    </section>
  );
}

export function InvestmentView({ report }: { report: RecoveryReport }) {
  const i = report.investment;
  return (
    <section aria-label="investment" className="flex items-center justify-between gap-3">
      <div data-testid="capex">
        <div className="kpi-val" style={{ color: "hsl(185 80% 60%)" }}>{fmtEur(i.capex)}</div>
        <div className="kpi-label">total CapEx</div>
      </div>
      <div className="text-right font-mono text-[11px] text-slate-400">
        <div data-testid="opex">{fmtEur(i.opex_per_year)}/yr OpEx</div>
        <div data-testid="payback" className="mt-0.5 font-semibold" style={{ color: "hsl(142 70% 58%)" }}>
          {i.payback_years.toFixed(1)} yr payback
        </div>
      </div>
    </section>
  );
}

export function SavingsView({ report }: { report: RecoveryReport }) {
  const s = report.savings;
  return (
    <section aria-label="savings" className="flex items-center justify-between gap-3">
      <div data-testid="recoverable">
        <div className="kpi-val" style={{ color: "hsl(24 90% 60%)" }}>
          {Math.round(s.recoverable_kwh_yr).toLocaleString()}
        </div>
        <div className="kpi-label">kWh/yr recovered</div>
      </div>
      <div data-testid="savings" className="text-right">
        <div className="kpi-val" style={{ color: "hsl(142 70% 58%)" }}>{fmtEur(s.energy_cost_savings_yr)}</div>
        <div className="kpi-label">saved / yr</div>
      </div>
    </section>
  );
}

export function CarbonView({ report }: { report: RecoveryReport }) {
  const c = report.carbon;
  const tonnes = c.co2e_kg_yr / 1000;
  const eq = c.equivalents;
  return (
    <section aria-label="carbon" className="flex flex-col gap-2.5">
      <div data-testid="carbon">
        <div className="kpi-val" style={{ color: "hsl(142 70% 58%)" }}>{tonnes.toFixed(1)}</div>
        <div className="kpi-label">t CO₂e/yr avoided</div>
      </div>
      <div className="grid grid-cols-3 gap-2 text-center">
        <Equiv icon="🚗" value={eq.cars_off_road.toFixed(1)} label="cars off road" />
        <Equiv icon="🌳" value={Math.round(eq.trees_planted).toLocaleString()} label="trees/yr" />
        <Equiv icon="🏠" value={eq.homes_powered.toFixed(1)} label="homes heated" />
      </div>
    </section>
  );
}

function Equiv({ icon, value, label }: { icon: string; value: string; label: string }) {
  return (
    <div className="rounded-lg border border-[hsl(220_15%_18%/0.6)] bg-[hsl(220_20%_8%/0.4)] px-1 py-2">
      <div className="text-base">{icon}</div>
      <div className="font-display text-sm font-semibold text-slate-100">{value}</div>
      <div className="kpi-label">{label}</div>
    </div>
  );
}
