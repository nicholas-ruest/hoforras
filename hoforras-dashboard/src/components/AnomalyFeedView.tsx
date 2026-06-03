// AnomalyFeedView (FR-9.6) — detected anomalies with their Claude explanations as they arrive.

import type { AnomalyFeedEntry } from "../store";

export function AnomalyFeedView({ anomalies }: { anomalies: AnomalyFeedEntry[] }) {
  return (
    <section aria-label="anomaly feed">
      <ul>
        {anomalies.map((a, i) => (
          <li key={i} data-testid="anomaly-row">
            <strong>{a.node_id}</strong>
            <p data-testid="explanation">
              {a.explanation ?? "awaiting Claude explanation…"}
            </p>
          </li>
        ))}
      </ul>
    </section>
  );
}
