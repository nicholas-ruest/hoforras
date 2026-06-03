// Geographic data for the Budapest deployment (real coordinates).
//
// Budapest is the "City of Spas" — its geothermal field feeds thermal springs along the Buda fault
// line (Római → Lukács/Császár → Király → Rác/Rudas/Gellért) plus deep drilled wells on the Pest
// side (Széchenyi, Dagály, Paskál, Dandár) and Margaret Island. This module identifies those
// underground thermal sources and places the Cognitum devices **near** them — SEED sensors clustered
// around each spring group with a v0 Appliance coordinating each coherence domain, so the deployment
// actually monitors and harvests the geothermal resource.

export type LatLng = [number, number];

export type DeviceKindGeo = "seed" | "appliance";

export interface SeedDevice {
  id: string; // .dark node id
  label: string;
  pos: LatLng;
  online: boolean;
  kind: DeviceKindGeo;
  near: string; // the spring/area it serves
}

export interface HotSpring {
  name: string;
  pos: LatLng;
  /** Source temperature (°C) of the geothermal water. */
  temp_c: number;
  /** Approximate yield (litres/minute) — a proxy for harvestable thermal output. */
  flow_lpm: number;
}

export const BUDAPEST_CENTER: LatLng = [47.51, 19.05];

/** Budapest's thermal hot springs / spas (the underground geothermal sources). */
export const HOT_SPRINGS: HotSpring[] = [
  // ── Buda thermal fault line (natural springs) ──
  { name: "Gellért", pos: [47.4836, 19.0517], temp_c: 44, flow_lpm: 1200 },
  { name: "Rudas", pos: [47.4889, 19.0494], temp_c: 42, flow_lpm: 900 },
  { name: "Rác", pos: [47.4925, 19.0489], temp_c: 40, flow_lpm: 350 },
  { name: "Király", pos: [47.5119, 19.0381], temp_c: 40, flow_lpm: 500 },
  { name: "Lukács", pos: [47.5147, 19.0369], temp_c: 49, flow_lpm: 1700 },
  { name: "Veli Bej (Császár)", pos: [47.5172, 19.0372], temp_c: 64, flow_lpm: 800 },
  { name: "Molnár János Cave", pos: [47.517, 19.036], temp_c: 27, flow_lpm: 1000 },
  // ── Margaret Island wells ──
  { name: "Margaret Island wells", pos: [47.526, 19.049], temp_c: 70, flow_lpm: 2500 },
  { name: "Palatinus", pos: [47.5283, 19.0506], temp_c: 40, flow_lpm: 2000 },
  // ── Pest deep thermal wells ──
  { name: "Széchenyi", pos: [47.5188, 19.0819], temp_c: 76, flow_lpm: 6000 },
  { name: "Dagály", pos: [47.5378, 19.0719], temp_c: 38, flow_lpm: 2200 }, // District XIII
  { name: "Paskál", pos: [47.5288, 19.1156], temp_c: 35, flow_lpm: 1200 },
  { name: "Dandár", pos: [47.4756, 19.0686], temp_c: 38, flow_lpm: 400 },
  // ── North Buda / Óbuda ──
  { name: "Római", pos: [47.5606, 19.0497], temp_c: 26, flow_lpm: 3500 },
  { name: "Csillaghegyi", pos: [47.5912, 19.0382], temp_c: 24, flow_lpm: 1500 },
  { name: "Pünkösdfürdő", pos: [47.628, 19.066], temp_c: 22, flow_lpm: 800 },
];

/** Cognitum devices, clustered around the thermal sources. SEED sensors observe; v0 Appliances
 *  coordinate each coherence domain. The District XIII pilot core (Pozsonyi / Dagály) is kept. */
export const SEED_DEVICES: SeedDevice[] = [
  // ── District XIII pilot core — Dagály thermal + Margaret Island (the pilot) ──
  { id: "dagaly-hub.thermal.budapest.dark", label: "Dagály Hub", pos: [47.5375, 19.071], online: true, kind: "appliance", near: "Dagály" },
  { id: "pozsonyi14.thermal.budapest.dark", label: "Pozsonyi 14", pos: [47.5273, 19.0512], online: true, kind: "seed", near: "Margaret Island wells" },
  { id: "pozsonyi22.thermal.budapest.dark", label: "Pozsonyi 22", pos: [47.5299, 19.0548], online: true, kind: "seed", near: "Palatinus" },
  { id: "pozsonyi30.thermal.budapest.dark", label: "Pozsonyi 30", pos: [47.5316, 19.0581], online: true, kind: "seed", near: "Dagály" },
  { id: "nepfurdo21.thermal.budapest.dark", label: "Népfürdő 21", pos: [47.536, 19.07], online: true, kind: "seed", near: "Dagály" },
  { id: "margitsziget.thermal.budapest.dark", label: "Margitsziget", pos: [47.5275, 19.05], online: true, kind: "seed", near: "Margaret Island wells" },

  // ── City Park / Pest wells — Széchenyi + Paskál ──
  { id: "varosliget-hub.thermal.budapest.dark", label: "Városliget Hub", pos: [47.5179, 19.0805], online: true, kind: "appliance", near: "Széchenyi" },
  { id: "hosoktere2.thermal.budapest.dark", label: "Hősök tere 2", pos: [47.5152, 19.078], online: true, kind: "seed", near: "Széchenyi" },
  { id: "paskal8.thermal.budapest.dark", label: "Paskál 8", pos: [47.5295, 19.114], online: true, kind: "seed", near: "Paskál" },

  // ── Buda thermal line — Lukács / Király / Veli Bej / Molnár János ──
  { id: "frankel-hub.thermal.budapest.dark", label: "Frankel Leó Hub", pos: [47.515, 19.0375], online: true, kind: "appliance", near: "Lukács" },
  { id: "fo84.thermal.budapest.dark", label: "Fő utca 84", pos: [47.5125, 19.0385], online: true, kind: "seed", near: "Király" },
  { id: "margitkrt2.thermal.budapest.dark", label: "Margit krt 2", pos: [47.5135, 19.037], online: false, kind: "seed", near: "Veli Bej (Császár)" },

  // ── Gellért Hill — Gellért / Rudas / Rác ──
  { id: "gellert-hub.thermal.budapest.dark", label: "Gellért Hub", pos: [47.484, 19.0525], online: true, kind: "appliance", near: "Gellért" },
  { id: "szentgellert4.thermal.budapest.dark", label: "Szent Gellért 4", pos: [47.4855, 19.051], online: true, kind: "seed", near: "Rudas" },
  { id: "dobrentei6.thermal.budapest.dark", label: "Döbrentei 6", pos: [47.4905, 19.049], online: true, kind: "seed", near: "Rác" },

  // ── South Pest — Dandár ──
  { id: "soroksari2.thermal.budapest.dark", label: "Soroksári 2", pos: [47.477, 19.068], online: true, kind: "seed", near: "Dandár" },

  // ── North Buda / Óbuda — Római / Csillaghegyi / Pünkösdfürdő ──
  { id: "romai-hub.thermal.budapest.dark", label: "Római Hub", pos: [47.56, 19.05], online: true, kind: "appliance", near: "Római" },
  { id: "csillaghegy5.thermal.budapest.dark", label: "Csillaghegy 5", pos: [47.5905, 19.039], online: true, kind: "seed", near: "Csillaghegyi" },
  { id: "punkosdfurdo1.thermal.budapest.dark", label: "Pünkösdfürdő 1", pos: [47.627, 19.0655], online: true, kind: "seed", near: "Pünkösdfürdő" },
];
