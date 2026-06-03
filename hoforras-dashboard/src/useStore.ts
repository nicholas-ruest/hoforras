// React binding for the projection store — re-render on any projection change.

import { useSyncExternalStore } from "react";
import type { DashboardState, DashboardStore } from "./store";

export function useDashboard(store: DashboardStore): DashboardState {
  return useSyncExternalStore(
    (cb) => store.subscribe(cb),
    () => store.getState(),
  );
}
