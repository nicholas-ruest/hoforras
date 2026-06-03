// DistrictGraphView (FR-9.1/9.2) — the 3D force-directed district map: buildings = nodes, pipes =
// weighted edges, active trades = animated particle streams.
//
// Rendered with react-three-fiber. The projection logic (edge weights, energy flows, highlights)
// lives in the store and is unit-tested without WebGL; this component is the visual binding. A
// deterministic ring layout stands in for the production force-directed solver.

import { Canvas } from "@react-three/fiber";
import type { JunctionId, NodeId } from "../events";
import { edgeKey, type DashboardState } from "../store";

export interface GraphTopology {
  nodes: NodeId[];
  /** undirected pipes between buildings, by node index. */
  pipes: [number, number][];
  /** junction id per pipe, parallel to `pipes`, for the energy-flow routes. */
  junctions: JunctionId[];
}

/** Deterministic ring layout — a placeholder for the production force-directed layout. */
export function ringLayout(count: number, radius = 6): [number, number, number][] {
  return Array.from({ length: count }, (_, i) => {
    const a = (i / Math.max(count, 1)) * Math.PI * 2;
    return [Math.cos(a) * radius, Math.sin(a) * radius, 0];
  });
}

function Building({
  position,
  warning,
}: {
  position: [number, number, number];
  warning: boolean;
}) {
  return (
    <mesh position={position}>
      <sphereGeometry args={[0.4, 16, 16]} />
      <meshStandardMaterial color={warning ? "#e23b3b" : "#3ba0e2"} />
    </mesh>
  );
}

export function DistrictGraphView({
  topology,
  state,
}: {
  topology: GraphTopology;
  state: DashboardState;
}) {
  const positions = ringLayout(topology.nodes.length);

  return (
    <div aria-label="district graph" data-testid="district-graph">
      <Canvas camera={{ position: [0, 0, 16] }}>
        <ambientLight intensity={0.8} />
        <pointLight position={[10, 10, 10]} />
        {topology.nodes.map((node, i) => (
          <Building
            key={node}
            position={positions[i]}
            warning={state.highlightedNodes.has(node)}
          />
        ))}
        {topology.pipes.map(([a, b], i) => {
          const seller = topology.nodes[a];
          const buyer = topology.nodes[b];
          const weight = state.edges.get(edgeKey(seller, buyer)) ?? 0;
          return (
            <Pipe key={i} from={positions[a]} to={positions[b]} weight={weight} />
          );
        })}
        {state.energyFlows.map((flow) => (
          <ParticleStream key={flow.id} route={flow.pipe_route} />
        ))}
      </Canvas>
    </div>
  );
}

function Pipe({
  from,
  to,
  weight,
}: {
  from: [number, number, number];
  to: [number, number, number];
  weight: number;
}) {
  const mid: [number, number, number] = [
    (from[0] + to[0]) / 2,
    (from[1] + to[1]) / 2,
    (from[2] + to[2]) / 2,
  ];
  const thickness = 0.02 + Math.min(weight, 100) / 1000;
  return (
    <mesh position={mid}>
      <boxGeometry args={[thickness, thickness, 0.01]} />
      <meshBasicMaterial color="#888" />
    </mesh>
  );
}

function ParticleStream({ route }: { route: JunctionId[] }) {
  // One small emissive marker per routed junction (the animated stream is driven by useFrame in the
  // full build; kept declarative here so the topology is what the test reasons about).
  return (
    <group data-route={route.join(",")}>
      {route.map((j, i) => (
        <mesh key={j} position={[i * 0.3 - route.length * 0.15, 0, 0]}>
          <sphereGeometry args={[0.08, 8, 8]} />
          <meshBasicMaterial color="#ffd23b" />
        </mesh>
      ))}
    </group>
  );
}
