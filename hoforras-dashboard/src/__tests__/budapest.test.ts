// Budapest geo-data sanity: comprehensive thermal springs + devices sited near them.

import { describe, expect, it } from "vitest";
import { HOT_SPRINGS, SEED_DEVICES, type LatLng } from "../budapest";

const inBudapest = ([lat, lng]: LatLng) => lat > 47.3 && lat < 47.7 && lng > 18.9 && lng < 19.3;

/** Rough great-circle distance in km. */
function km(a: LatLng, b: LatLng): number {
  const R = 6371;
  const dLat = ((b[0] - a[0]) * Math.PI) / 180;
  const dLng = ((b[1] - a[1]) * Math.PI) / 180;
  const lat1 = (a[0] * Math.PI) / 180;
  const lat2 = (b[0] * Math.PI) / 180;
  const h = Math.sin(dLat / 2) ** 2 + Math.cos(lat1) * Math.cos(lat2) * Math.sin(dLng / 2) ** 2;
  return 2 * R * Math.asin(Math.sqrt(h));
}

describe("Budapest thermal springs", () => {
  it("identifies a comprehensive set of thermal sources within Budapest", () => {
    expect(HOT_SPRINGS.length).toBeGreaterThanOrEqual(15);
    for (const s of HOT_SPRINGS) {
      expect(inBudapest(s.pos)).toBe(true);
      expect(s.temp_c).toBeGreaterThan(20);
      expect(s.flow_lpm).toBeGreaterThan(0);
    }
  });

  it("includes the major baths along the Buda line and the Pest wells", () => {
    const names = new Set(HOT_SPRINGS.map((s) => s.name));
    for (const required of ["Széchenyi", "Gellért", "Rudas", "Rác", "Lukács", "Dagály", "Római", "Paskál", "Dandár"]) {
      expect(names.has(required)).toBe(true);
    }
  });
});

describe("Cognitum device siting", () => {
  it("places every device in Budapest with a .dark id and a kind", () => {
    expect(SEED_DEVICES.length).toBeGreaterThanOrEqual(15);
    for (const d of SEED_DEVICES) {
      expect(inBudapest(d.pos)).toBe(true);
      expect(d.id).toMatch(/\.thermal\.budapest\.dark$/);
      expect(["seed", "appliance"]).toContain(d.kind);
    }
  });

  it("deploys at least one v0 Appliance to coordinate coherence domains", () => {
    expect(SEED_DEVICES.some((d) => d.kind === "appliance")).toBe(true);
  });

  it("sites a Cognitum device within ~1.7 km of EVERY thermal spring", () => {
    for (const s of HOT_SPRINGS) {
      const nearest = Math.min(...SEED_DEVICES.map((d) => km(s.pos, d.pos)));
      expect(nearest, `no device near ${s.name} (nearest ${nearest.toFixed(2)} km)`).toBeLessThan(1.7);
    }
  });
});
