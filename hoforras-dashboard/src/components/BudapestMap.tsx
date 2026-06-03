// BudapestMap — the operator's geographic view (MapLibre GL, three open & free base views).
//
//   • Schematic — dark CARTO street map (clean, abstract).
//   • Satellite — Esri World Imagery (Maxar/Earthstar) — precise aerial, no API key.
//   • 3D        — OpenFreeMap vector tiles extruded into a tiltable 3D city model (open data,
//                 no key, no limits) — the open analog to Google Earth's 3D.
//
// Two data layers ride on top of every view: the Cognitum SEED devices across District XIII and
// Budapest's thermal hot springs (+ proposed recovery units in Planner mode). Read-only.

import type { StyleSpecification } from "maplibre-gl";
import { useMemo, useState } from "react";
import Map, { Marker, NavigationControl, Popup } from "react-map-gl/maplibre";
import { type HotSpring, type SeedDevice } from "../budapest";
import type { RecoveryData } from "../recovery";

type BaseView = "schematic" | "satellite" | "3d";

const SCHEMATIC: StyleSpecification = {
  version: 8,
  sources: {
    carto: {
      type: "raster",
      tiles: ["a", "b", "c", "d"].map((s) => `https://${s}.basemaps.cartocdn.com/dark_all/{z}/{x}/{y}.png`),
      tileSize: 256,
      attribution: '&copy; OSM &copy; <a href="https://carto.com/attributions">CARTO</a>',
    },
  },
  layers: [{ id: "carto", type: "raster", source: "carto" }],
};

const SATELLITE: StyleSpecification = {
  version: 8,
  sources: {
    esri: {
      type: "raster",
      tiles: ["https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{z}/{y}/{x}"],
      tileSize: 256,
      attribution: "Imagery &copy; Esri, Maxar, Earthstar Geographics",
    },
    labels: {
      type: "raster",
      tiles: ["https://server.arcgisonline.com/ArcGIS/rest/services/Reference/World_Boundaries_and_Places/MapServer/tile/{z}/{y}/{x}"],
      tileSize: 256,
    },
  },
  layers: [
    { id: "esri", type: "raster", source: "esri" },
    { id: "labels", type: "raster", source: "labels", paint: { "raster-opacity": 0.9 } },
  ],
};

// OpenFreeMap "liberty" — full OSM vector style incl. 3D building extrusions. Open, free, no key.
const STYLE_3D = "https://tiles.openfreemap.org/styles/liberty";

function styleFor(v: BaseView): StyleSpecification | string {
  return v === "schematic" ? SCHEMATIC : v === "satellite" ? SATELLITE : STYLE_3D;
}

type Selected =
  | { kind: "seed"; lng: number; lat: number; d: SeedDevice; warn: boolean }
  | { kind: "spring"; lng: number; lat: number; s: HotSpring; harvested: boolean };

export function BudapestMap({
  seeds,
  springs,
  highlighted,
  planner,
}: {
  seeds: SeedDevice[];
  springs: HotSpring[];
  highlighted?: Set<string>;
  planner?: RecoveryData | null;
}) {
  const [view, setView] = useState<BaseView>("schematic");
  const [vs, setVs] = useState({ longitude: 19.045, latitude: 47.505, zoom: 11.5, pitch: 0, bearing: 0 });
  const [sel, setSel] = useState<Selected | null>(null);

  const harvestTargets = useMemo(() => {
    if (!planner) return new Set<string>();
    const ranked = [...springs].sort((a, b) => b.temp_c * b.flow_lpm - a.temp_c * a.flow_lpm);
    return new Set(ranked.slice(0, 3).map((s) => s.name));
  }, [planner, springs]);

  const changeView = (v: BaseView) => {
    setView(v);
    setVs((s) => ({ ...s, pitch: v === "3d" ? 58 : 0, bearing: v === "3d" ? -18 : 0 }));
  };

  return (
    <div className="relative h-full w-full">
      <Map
        {...vs}
        onMove={(e) => setVs(e.viewState)}
        mapStyle={styleFor(view)}
        style={{ width: "100%", height: "100%" }}
        maxPitch={75}
      >
        <NavigationControl position="bottom-right" visualizePitch showCompass />

        {/* Hot springs — geothermal sources (size by flow). */}
        {springs.map((s) => {
          const size = 12 + Math.min(s.flow_lpm, 6000) / 280;
          const harvested = harvestTargets.has(s.name);
          return (
            <Marker
              key={s.name}
              longitude={s.pos[1]}
              latitude={s.pos[0]}
              anchor="center"
              onClick={(e) => {
                e.originalEvent.stopPropagation();
                setSel({ kind: "spring", lng: s.pos[1], lat: s.pos[0], s, harvested });
              }}
            >
              <div className="map-pin spring" style={{ width: size, height: size, cursor: "pointer" }} />
            </Marker>
          );
        })}

        {/* Proposed recovery units at harvest targets (Planner mode). */}
        {planner &&
          springs
            .filter((s) => harvestTargets.has(s.name))
            .map((s) => (
              <Marker key={`rec-${s.name}`} longitude={s.pos[1]} latitude={s.pos[0] + 0.0016} anchor="center">
                <div className="map-pin recovery" style={{ width: 16, height: 16 }} />
              </Marker>
            ))}

        {/* Cognitum devices, clustered near the thermal sources. */}
        {seeds.map((d) => {
          const warn = highlighted?.has(d.id) ?? false;
          const isApp = d.kind === "appliance";
          const sz = isApp ? 16 : 13;
          return (
            <Marker
              key={d.id}
              longitude={d.pos[1]}
              latitude={d.pos[0]}
              anchor="center"
              onClick={(e) => {
                e.originalEvent.stopPropagation();
                setSel({ kind: "seed", lng: d.pos[1], lat: d.pos[0], d, warn });
              }}
            >
              <div
                className={`map-pin ${isApp ? "appliance" : "seed"}`}
                style={{ width: sz, height: sz, opacity: d.online ? 1 : 0.4, cursor: "pointer" }}
              />
            </Marker>
          );
        })}

        {sel && (
          <Popup longitude={sel.lng} latitude={sel.lat} anchor="bottom" onClose={() => setSel(null)} closeOnClick={false} offset={12}>
            {sel.kind === "spring" ? (
              <div>
                <strong style={{ color: "hsl(24 90% 65%)" }}>♨ {sel.s.name} spring</strong>
                <br />
                source {sel.s.temp_c} °C · ~{sel.s.flow_lpm.toLocaleString()} L/min
                {sel.harvested && (
                  <>
                    <br />
                    <span style={{ color: "hsl(142 70% 60%)" }}>▲ recovery target</span>
                  </>
                )}
              </div>
            ) : (
              <div>
                <strong style={{ color: sel.d.kind === "appliance" ? "hsl(265 80% 70%)" : sel.warn ? "hsl(0 70% 68%)" : "hsl(185 80% 60%)" }}>
                  {sel.d.kind === "appliance" ? "▣" : "◈"} {sel.d.label}
                </strong>
                <br />
                {sel.d.kind === "appliance" ? "v0 Appliance" : "SEED sensor"} · serves {sel.d.near}
                <br />
                {sel.d.id}
                <br />
                {sel.d.online ? "● online" : "○ offline"}
                {sel.warn ? " · ⚠ anomaly" : ""}
              </div>
            )}
          </Popup>
        )}
      </Map>

      {/* Base-view toggle */}
      <div className="absolute right-3 top-3 z-[600] flex gap-1 rounded-lg border border-[hsl(220_15%_18%/0.6)] bg-[hsl(220_25%_6%/0.85)] p-1 backdrop-blur">
        {(["schematic", "satellite", "3d"] as const).map((b) => (
          <button
            key={b}
            data-testid={`base-${b}`}
            onClick={() => changeView(b)}
            className={`btn ${view === b ? "on" : ""}`}
            style={{ fontSize: "0.72rem", padding: "0.3rem 0.65rem" }}
          >
            {b === "schematic" ? "◳ Schematic" : b === "satellite" ? "🛰 Satellite" : "⛶ 3D"}
          </button>
        ))}
      </div>
    </div>
  );
}
