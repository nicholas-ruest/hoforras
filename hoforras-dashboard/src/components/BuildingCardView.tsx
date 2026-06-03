// BuildingCardView (FR-9.3) — surplus/deficit per building + a 24h demand sparkline.

import type { BuildingCard } from "../store";
import { Sparkline } from "./Sparkline";

export function BuildingCardView({ buildings }: { buildings: BuildingCard[] }) {
  return (
    <section aria-label="building cards">
      {buildings.map((b) => (
        <article key={b.node_id} data-testid="building-card">
          <h3>{b.node_id}</h3>
          <p data-testid="surplus">surplus {b.surplus_kwh.toFixed(1)} kWh</p>
          <p data-testid="deficit">deficit {b.deficit_kwh.toFixed(1)} kWh</p>
          <div data-testid="sparkline">
            <Sparkline values={b.demand24h} />
          </div>
        </article>
      ))}
    </section>
  );
}
