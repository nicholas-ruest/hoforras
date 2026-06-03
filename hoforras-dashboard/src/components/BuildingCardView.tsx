// BuildingCardView (FR-9.3) — surplus/deficit per building + a 24h demand sparkline.

import type { BuildingCard } from "../store";
import { Sparkline } from "./Sparkline";

const short = (id: string) => id.split(".")[0];

export function BuildingCardView({ buildings }: { buildings: BuildingCard[] }) {
  return (
    <section aria-label="building cards" className="flex flex-col gap-2">
      {buildings.map((b) => {
        const net = b.surplus_kwh - b.deficit_kwh;
        const surplusy = net >= 0;
        return (
          <article
            key={b.node_id}
            data-testid="building-card"
            className="rounded-lg border border-[hsl(220_15%_18%/0.6)] bg-[hsl(220_20%_8%/0.4)] p-2.5"
          >
            <div className="flex items-center justify-between">
              <span className="text-[13px] font-semibold text-slate-200">{short(b.node_id)}</span>
              <span className={`badge ${surplusy ? "thermal" : "primary"}`}>
                {surplusy ? "surplus" : "deficit"}
              </span>
            </div>
            <div className="mt-2 flex items-end justify-between">
              <div className="flex gap-4 font-mono">
                <div>
                  <div data-testid="surplus" className="text-sm font-semibold" style={{ color: "hsl(24 90% 60%)" }}>
                    {b.surplus_kwh.toFixed(1)} kWh
                  </div>
                  <div className="kpi-label">surplus</div>
                </div>
                <div>
                  <div data-testid="deficit" className="text-sm font-semibold" style={{ color: "hsl(185 80% 60%)" }}>
                    {b.deficit_kwh.toFixed(1)} kWh
                  </div>
                  <div className="kpi-label">deficit</div>
                </div>
              </div>
              <div data-testid="sparkline">
                <Sparkline values={b.demand24h} color={surplusy ? "hsl(24 90% 55%)" : "hsl(185 80% 50%)"} />
              </div>
            </div>
          </article>
        );
      })}
    </section>
  );
}
