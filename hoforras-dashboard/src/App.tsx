// DistrictDashboard — COGS-style operator console for District XIII, Budapest.
//
//   • Live Grid       — the autonomous market over a map of Budapest: SEED devices + hot springs,
//                       live trades, anomalies, witness log (read-only over the feed).
//   • Recovery Planner — Thermal Recovery Planning (ADR-0016): the map shows harvest targets, and
//                        KPI cards quantify investment, payback, savings, and carbon. Advisory.
// Both are read-only / conformist (DDD-09 §6).

import { useEffect, useMemo, useState } from "react";
import { AnomalyFeedView } from "./components/AnomalyFeedView";
import { BudapestMap } from "./components/BudapestMap";
import { BuildingCardView } from "./components/BuildingCardView";
import { PipeHealthHeatmap, type WitnessLogProvider } from "./components/PipeHealthHeatmap";
import {
  AdvisoryBadge,
  CarbonView,
  InvestmentView,
  SavingsView,
  SitingPlanView,
} from "./components/recovery/RecoveryPanels";
import { TradeTimelineView } from "./components/TradeTimelineView";
import { Appbar, Card, Kpi, type DashboardMode } from "./components/ui";
import { wireDashboard, type RequestClaudeExplanation } from "./controller";
import { HOT_SPRINGS, SEED_DEVICES } from "./budapest";
import type { RecoveryData } from "./recovery";
import { DashboardStore } from "./store";
import { WebSocketOperatorFeed, type OperatorSocket } from "./socket";
import { useDashboard } from "./useStore";

export interface DashboardProps {
  store: DashboardStore;
  makeSocket?: () => OperatorSocket;
  requestClaudeExplanation: RequestClaudeExplanation;
  witnessLogFor: WitnessLogProvider;
  recovery?: RecoveryData;
}

const fmtEur = (n: number) =>
  new Intl.NumberFormat("en-IE", { style: "currency", currency: "EUR", maximumFractionDigits: 0 }).format(n);

export function DistrictDashboard({
  store,
  makeSocket,
  requestClaudeExplanation,
  witnessLogFor,
  recovery,
}: DashboardProps) {
  const state = useDashboard(store);
  const [mode, setMode] = useState<DashboardMode>("live");
  const socket = useMemo(() => (makeSocket ? makeSocket() : new WebSocketOperatorFeed()), [makeSocket]);

  useEffect(() => {
    const dispose = wireDashboard(socket, { store, requestClaudeExplanation });
    return dispose;
  }, [socket, store, requestClaudeExplanation]);

  const seedsOnline = SEED_DEVICES.filter((d) => d.online).length;
  const seedCount = SEED_DEVICES.filter((d) => d.kind === "seed").length;
  const applianceCount = SEED_DEVICES.filter((d) => d.kind === "appliance").length;
  const planner = mode === "planner" && recovery ? recovery : null;
  const report = recovery?.report;
  const totalKwh = state.trades.reduce((s, t) => s + t.kwh, 0);

  return (
    <div className="flex h-screen flex-col">
      <Appbar
        mode={mode}
        onMode={setMode}
        hasPlanner={!!recovery}
        seedsOnline={seedsOnline}
        seedsTotal={SEED_DEVICES.length}
      />

      {/* KPI row */}
      <div className="grid shrink-0 grid-cols-2 gap-2.5 px-3 pt-3 sm:grid-cols-3 lg:grid-cols-5">
        {mode === "live" || !report ? (
          <>
            <Kpi icon="◈" label="SEED sensors" value={`${seedCount}`} sub={`${seedsOnline}/${SEED_DEVICES.length} devices online`} />
            <Kpi icon="▣" label="v0 Appliances" value={applianceCount} sub="coherence domains" />
            <Kpi icon="♨" label="Hot springs" value={HOT_SPRINGS.length} sub="geothermal sources" tone="thermal" />
            <Kpi icon="⚡" label="kWh traded" value={totalKwh.toFixed(0)} sub={`${state.trades.length} trades`} tone="thermal" />
            <Kpi icon="⚠" label="Anomalies" value={state.anomalies.length} sub="detected" tone="accent" />
          </>
        ) : (
          <>
            <Kpi icon="♻" label="Recoverable" value={`${Math.round(report.savings.recoverable_kwh_yr / 1000)} MWh`} sub="per year" tone="thermal" />
            <Kpi icon="€" label="Annual savings" value={fmtEur(report.savings.energy_cost_savings_yr)} sub="per year" tone="accent" />
            <Kpi icon="⏱" label="Payback" value={`${report.investment.payback_years.toFixed(1)} yr`} sub={fmtEur(report.investment.capex) + " CapEx"} />
            <Kpi icon="🌍" label="CO₂e avoided" value={`${(report.carbon.co2e_kg_yr / 1000).toFixed(1)} t`} sub="per year" tone="accent" />
            <Kpi icon="🌳" label="Equivalent" value={Math.round(report.carbon.equivalents.trees_planted).toLocaleString()} sub="trees / yr" tone="accent" />
          </>
        )}
      </div>

      {/* Map + side rail */}
      <div className="grid min-h-0 flex-1 grid-cols-1 gap-2.5 p-3 lg:grid-cols-[1fr_360px]">
        {/* Map centerpiece */}
        <div className="relative min-h-[320px] overflow-hidden rounded-xl border border-[hsl(220_15%_18%/0.5)]">
          <BudapestMap
            seeds={SEED_DEVICES}
            springs={HOT_SPRINGS}
            highlighted={state.highlightedNodes}
            planner={planner}
          />
          {/* Legend */}
          <div className="pointer-events-none absolute bottom-3 left-3 z-[500] rounded-lg border border-[hsl(220_15%_18%/0.6)] bg-[hsl(220_25%_6%/0.85)] px-3 py-2 text-[11px] backdrop-blur">
            <div className="mb-1 font-semibold text-slate-300">
              {mode === "planner" ? "Recovery plan · Budapest" : "District XIII · Budapest"}
            </div>
            <Legend color="hsl(185 80% 55%)" label="SEED sensor" />
            <Legend color="hsl(265 80% 62%)" label="v0 Appliance" />
            <Legend color="hsl(24 90% 58%)" label="Thermal hot spring" />
            {mode === "planner" && <Legend color="hsl(142 70% 55%)" label="Proposed recovery unit" />}
          </div>
        </div>

        {/* Side rail */}
        <div className="flex min-h-0 flex-col gap-2.5 overflow-auto pr-0.5">
          {mode === "live" ? (
            <>
              <Card title="Buildings" sub="surplus / deficit">
                <BuildingCardView buildings={state.buildings} />
              </Card>
              <Card title="Trade Timeline" sub="QuDAG finality">
                <TradeTimelineView trades={state.trades} />
              </Card>
              <Card title="Pipe Health" sub="drill → witness log">
                <PipeHealthHeatmap pipes={state.pipes} witnessLogFor={witnessLogFor} />
              </Card>
              <Card title="Anomaly Feed" sub="Claude explanations">
                <AnomalyFeedView anomalies={state.anomalies} />
              </Card>
            </>
          ) : (
            report && (
              <>
                <Card title="Loss & Siting Plan" sub="device placement" right={<AdvisoryBadge />}>
                  <SitingPlanView report={report} />
                </Card>
                <Card title="Investment" sub="CapEx · OpEx · payback">
                  <InvestmentView report={report} />
                </Card>
                <Card title="Energy Savings" sub="recovered heat">
                  <SavingsView report={report} />
                </Card>
                <Card title="Carbon Reduction" sub="GHG avoided">
                  <CarbonView report={report} />
                </Card>
                <ul className="px-1 text-[10px] leading-relaxed text-slate-600">
                  {report.assumptions.map((a, i) => (
                    <li key={i}>· {a}</li>
                  ))}
                </ul>
              </>
            )
          )}
        </div>
      </div>
    </div>
  );
}

function Legend({ color, label }: { color: string; label: string }) {
  return (
    <div className="flex items-center gap-1.5">
      <span className="h-2.5 w-2.5 rounded-full" style={{ background: color, boxShadow: `0 0 8px ${color}` }} />
      <span className="text-slate-400">{label}</span>
    </div>
  );
}
