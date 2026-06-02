How to Build Hőforrás Budapest's Peer-to-Peer Thermal Energy Intelligence System

### Architecture Overview

[Cognitum **SEED** Sensors] → thermal, vibration, pipe resonance, pressure
         ↓
[@ruv/rvcsi ingestion layer] → validate, normalize, typed event emission
         ↓
[ruv-**FANN** + Neuro-Divergent] → **LSTM**/N-**BEATS** thermal anomaly forecasting
         ↓
[daa Orchestrator + Prime **DHT**] → autonomous peer-to-peer energy broker
         ↓
[rvm Coherence Engine] → building node isolation, capability gating
         ↓
[QuDAG] → quantum-resistant consensus for cross-building trade agreements
         ↓
[RuVector **GNN** Memory DB] → thermal state history, similarity search
         ↓
[Synaptic-Mesh] → district-scale swarm coordination, self-healing mesh
         ↓
[ruflo + @ruvnet/rvagent] → AI reasoning, **MCP** orchestration
         ↓
[TypeScript + React frontend] → live district thermal map + trade dashboard

Layer 1 — Sensor Ingestion GitHub:

[https://github.com/ruvnet/RuView](https://github.com/ruvnet/RuView) [https://github.com/ruvnet/rvcsi](https://github.com/ruvnet/rvcsi)

npm:

[https://[www.npmjs.com/package/@ruv/rvcsi](https://www.npmjs.com/package/@ruv/rvcsi](https://www.npmjs.com/package/@ruv/rvcsi](https://www.npmjs.com/package/@ruv/rvcsi)) [https://[www.npmjs.com/package/@ruvnet/rvagent](https://www.npmjs.com/package/@ruvnet/rvagent](https://www.npmjs.com/package/@ruvnet/rvagent](https://www.npmjs.com/package/@ruvnet/rvagent))

The rvcsi repo is the direct foundation for the **SEED** sensor ingestion layer. Its architecture already handles multi-source ingestion, validation pipelines, **DSP** stages, and typed confidence-scored event emission with a clean Rust-first / TypeScript-accessible surface. You adapt the schema — CsiFrame becomes ThermalFrame, WiFi subcarrier data becomes thermal gradient, pipe vibration, and fluid pressure signals — but the validation pipeline, event state machines, and RuVector RF-memory export pattern carry over directly. RuView's Cognitum **SEED** integration, **ESP32** firmware, and Ed25519 witness chain (every sensor reading cryptographically signed and provably real) are reused without modification. The @ruvnet/rvagent **MCP** server pattern provides the bridge between the Rust sensing stack and AI agent orchestration — in Hőforrás this becomes the ThermalSense-Bridge, giving ruflo swarms direct tool-call access to live thermal readings across the district. rust// Adapt rvcsi's CsiFrame → ThermalFrame // crates/hoforras-sensor/src/thermal_frame.rs use rvcsi_core::{ValidationStatus, QualityScore};

pub struct ThermalFrame {
    pub node_id: NodeId,
    pub timestamp_ns: u64,
    pub temperature_celsius: f32,
    pub pipe_vibration_hz: f32,
    pub fluid_pressure_bar: f32,
    pub ground_thermal_gradient: f32,
    pub quality_score: QualityScore,
    pub validation: ValidationStatus,
    pub witness: Ed25519Signature,
}

Layer 2 — Local Neural Inference GitHub:

[https://github.com/ruvnet/ruv-**FANN**](https://github.com/ruvnet/ruv-**FANN**)

npm:

[https://[www.npmjs.com/package/ruv-swarm](https://www.npmjs.com/package/ruv-swarm](https://www.npmjs.com/package/ruv-swarm](https://www.npmjs.com/package/ruv-swarm))

Cargo: ruv-swarm = *1.0.5* (source at [https://github.com/ruvnet/ruv-**FANN**/tree/main/ruv-swarm](https://github.com/ruvnet/ruv-**FANN**/tree/main/ruv-swarm)) ruv-**FANN**'s Neuro-Divergent forecasting layer provides the on-device inference engine running on each Cognitum Appliance. Specifically you use the **LSTM** and N-**BEATS** models for thermal anomaly detection, pipe burst precursor prediction with 6–48 hour warning windows, and 24-hour thermal demand forecasting per building as inputs to the energy broker's trade scheduling. This runs entirely as **WASM** on-device. No **GPU**, no cloud. ruv-swarm's ephemeral agent model is exactly right: spin up a tiny specialist inference network per building node, solve the prediction in under 100ms, dissolve it. The ruv-swarm **MCP** server gives Claude Code tools to query, retrain, and inspect these models live from the dashboard. typescript// Spawn ephemeral thermal prediction agent via ruv-swarm import { RuvSwarm } from 'ruv-swarm';

const swarm = await RuvSwarm.initialize({ topology: 'mesh' });

const thermalForecaster = await swarm.spawn({
    type: 'analyst',
    specialization: 'thermal_timeseries',
    cognitiveProfile: { analytical: 0.95, systematic: 0.9 },
    neuralModel: 'lstm',
    capabilities: ['anomaly_detection', 'demand_forecasting', 'pipe_health_scoring']
});

Layer 3 — Autonomous Thermal Energy Broker GitHub:

[https://github.com/ruvnet/daa](https://github.com/ruvnet/daa)

Cargo crates:

daa-rules = *0.2.1* daa-economy = *0.2.1* daa-ai = *0.2.1* daa-prime-core = *0.2.1* daa-prime-dht = *0.2.1* daa-prime-trainer = *0.2.1* daa-prime-coordinator = *0.2.1*

This is the heart of the invention. The daa **SDK**'s **MRAP** loop (Monitor → Reason → Act → Reflect → Adapt) maps directly onto what each building node needs to do as an autonomous thermal market participant. Monitor reads thermal surplus/deficit from the local **SEED** mesh. Reason decides whether to offer heat to the network or bid for it, at what price, for how long. Act posts offers and bids, executes accepted trades, routes thermal energy. Reflect evaluates whether trades delivered expected value. Adapt updates pricing strategy and forecast models. daa-economy manages the thermal credit token — replace rUv tokens with kWh-equivalent thermal credits. daa-rules enforces governance: maximum daily transfer limits, minimum safety reserve thresholds, network pipe capacity constraints. daa-ai integrates Claude for intelligent anomaly reasoning. The daa-prime-coordinator handles Byzantine fault-tolerant gradient aggregation across building nodes, mapping cleanly onto coordinating federated thermal demand models so 20 buildings can collectively train a district-level predictor without sharing raw building data. rust// Cargo.toml for hoforras-broker crate [dependencies] daa-rules             = *0.2.1* daa-economy           = *0.2.1* daa-ai                = *0.2.1* daa-prime-core        = *0.2.1* daa-prime-coordinator = *0.2.1* daa-prime-trainer     = *0.2.1* tokio                 = { version = *1*, features = [*full*] }

// Thermal trade governance rules
agent.rules_engine()
    .add_rule(*max_daily_thermal_transfer_kwh*, **500**.0)
    .add_rule(*min_safety_reserve_percent*, 0.15)
    .add_rule(*pipe_pressure_ceiling_bar*, 6.0)
    .add_rule(*trade_window_hours*, 6)?;

Layer 4 — Node Isolation and Security GitHub:

[https://github.com/ruvnet/rvm](https://github.com/ruvnet/rvm)

Cargo crates (workspace):

rvm-kernel rvm-coherence rvm-witness rvm-security rvm-cap rvm-proof

Each building node runs as a coherence domain inside **RVM**. The rvm-coherence engine's graph-theoretic mincut algorithm handles dynamic re-isolation automatically: if a building's thermal data shows anomalous patterns — injected bad readings, hardware fault, a compromised Appliance — **RVM** splits that node into an isolated partition without manual intervention or system restart.
The rvm-witness hash-chained audit trail records every privileged action in 64-byte witness records. Every thermal trade agreement signed, every sensor reading accepted, every energy routing decision made is forensically auditable. The rvm-security capability gate with rvm-cap's unforgeable capability tokens prevents any building from reading another building's raw sensor data — they only receive aggregated thermal availability signals through capability-gated CommEdges.
**RVM**'s benchmarks are well-suited to this real-time use case: partition switch at ~6ns, mincut on a 16-node graph at ~331ns, witness emit at ~17ns. The district thermal mesh can reconfigure itself without perceptible latency.
rust// Only **AGGREGATE** thermal availability crosses partition boundaries
// Raw sensor data never leaves the building's partition
let thermal_availability_cap = Capability::new(
    Rights::**READ**,
    Scope::AggregatedThermalAvailability,
    Expiry::Hours(6),
);

Layer 5 — Quantum-Resistant Trade Consensus GitHub:

[https://github.com/ruvnet/QuDAG](https://github.com/ruvnet/QuDAG)

QuDAG's post-quantum **DAG**-based consensus (ML-**DSA** signatures, ML-**KEM**-**1024** encryption, Kademlia **DHT** peer discovery via .dark domains) handles the cross-building thermal trade agreement protocol. When Building A agrees to route 40kWh of geothermal waste heat to Building B for the next 6 hours at a given kWh credit price, that agreement is a signed **DAG** entry — verifiable, tamper-evident, and quantum-resistant. QR-Avalanche consensus provides sub-second finality. .dark domain discovery means building nodes find each other without a directory server. Every Appliance joins the district's .thermal.budapest.dark namespace and discovers peers automatically.
rustlet trade = ThermalTradeAgreement {
    seller: NodeId::from(*pozsonyi14.thermal.budapest.dark*),
    buyer:  NodeId::from(*pozsonyi22.thermal.budapest.dark*),
    kwh_offered: 40.0,
    duration_hours: 6,
    credit_price_per_kwh: 2.3,
    pipe_route: vec![junction_7, junction_12, junction_18],
    valid_from: timestamp_now(),
};

let entry = DagEntry::new(trade).sign_with(ml_dsa_key)?; network.broadcast_and_await_consensus(entry).await?;

Layer 6 — Vector Memory and **GNN** State GitHub:

[https://github.com/ruvnet/RuVector](https://github.com/ruvnet/RuVector)

npm:

[https://[www.npmjs.com/package/ruvector](https://www.npmjs.com/package/ruvector](https://www.npmjs.com/package/ruvector](https://www.npmjs.com/package/ruvector))

RuVector is the persistent memory layer for each Appliance. It stores time-series thermal state history per building, vector embeddings of each building's thermal consumption patterns (enabling similarity matching — find buildings with thermal surplus profiles similar to what Building 22 needs), and the **RVM** witness trail for forensic replay. RuVector's **GNN** (Graph Neural Network) layer is especially well-suited here because the district thermal network is literally a graph — buildings as nodes, pipe connections as weighted edges — and RuVector runs graph-structured inference over it natively. The DiskANN / **HNSW** vector index means thermal pattern similarity queries run in sub-millisecond time even across thousands of historical readings. typescriptimport { RuVector } from 'ruvector';

const memory = new RuVector({ indexType: 'hnsw', dimensions: **128** });

// Find buildings whose thermal surplus patterns match building 22's deficit
const candidates = await memory.search({
    vector: building22_deficit_embedding,
    topK: 5,
    filter: { thermal_surplus_available: true, district: '**XIII**' }
});

Layer 7 — District-Scale Swarm Coordination GitHub:

[https://github.com/ruvnet/Synaptic-Mesh](https://github.com/ruvnet/Synaptic-Mesh)

Synaptic-Mesh provides the **DAG**-based swarm fabric across all Appliances in the district. Each Appliance is a micro-mind node in the neural mesh. Thermal market state, anomaly signals, and consensus decisions propagate not through **RPC** calls but as signed **DAG** entries. The mesh self-heals when nodes drop and resumes when they return. The QuDAG networking substrate provides the quantum-resistant **P2P** layer, ruv-**FANN**'s **WASM** neural runtime provides per-node inference, and daa's swarm coordination protocols handle collective intelligence behaviors — district-wide thermal load balancing, cascade failure prevention, and heat wave emergency routing. bash# Initialize the district thermal mesh cargo run -- node start --port **8080** --mesh-id hoforras-district-xiii

# Create district thermal swarm

cargo run -- swarm create \
    --agents 20 \
    --behavior thermal_market_optimization \
    --topology mesh

# Initialize thermal market state

cargo run -- market init --db-path district_xiii_thermal.db

Layer 8 — AI Orchestration and Reasoning GitHub:

[https://github.com/ruvnet/ruflo](https://github.com/ruvnet/ruflo)

npm:

[https://[www.npmjs.com/package/ruflo](https://www.npmjs.com/package/ruflo](https://www.npmjs.com/package/ruflo](https://www.npmjs.com/package/ruflo)) [https://[www.npmjs.com/package/@ruvnet/rvagent](https://www.npmjs.com/package/@ruvnet/rvagent](https://www.npmjs.com/package/@ruvnet/rvagent](https://www.npmjs.com/package/@ruvnet/rvagent))

ruflo is the multi-agent harness that gives the entire Hőforrás system an intelligent reasoning layer. The ThermalSense-Bridge **MCP** server (adapted from @ruvnet/rvagent) registers thermal sensing tools that Claude can call directly from the dashboard or **CLI**: hoforras.thermal.district_status, hoforras.trade.active_agreements, hoforras.pipe.health_scores, hoforras.anomaly.recent, and hoforras.forecast.demand_24h.
ruflo's agent federation layer handles cross-district coordination — when Hőforrás expands from District **XIII** to District V or **VII**, their Appliance meshes discover each other, authenticate via mTLS + Ed25519, and federate thermal market signals without sharing raw building data.
bash# Register ThermalSense-Bridge as **MCP** server in Claude Code
claude mcp add hoforras -- npx ruflo@latest mcp start \
    --plugin hoforras-thermal-bridge \
    --sensing-url [http://localhost:**8080**](http://localhost:**8080**)

# Initialize ruflo for district operations

npx ruflo@latest init wizard \
    --profile thermal_district_operator \
    --mesh-id hoforras-district-xiii

Layer 9 — TypeScript Frontend Built on React + react-three-fiber + WebSocket feeds from ruflo **MCP**. The operator dashboard renders a live 3D force-directed graph of the district — buildings as nodes, thermal pipe connections as weighted edges, animated energy flow as particle streams along active trade routes. Per-building thermal surplus/deficit cards with 24h forecast sparklines. Active trade agreement timeline with QuDAG consensus status. Pipe health heat map with **RVM** witness log drill-down. Anomaly alert feed with natural language explanations from Claude via ruflo. typescriptimport { Canvas } from '@react-three/fiber'; import { ruflo } from 'ruflo';

const thermalMesh = await ruflo.connect('ws://appliance.district-xiii:**3001**');

thermalMesh.on('trade:executed', (agreement) => {
    updateEdgeWeight(agreement.seller, agreement.buyer, agreement.kwh_offered);
    animateEnergyFlow(agreement.pipe_route);
});

thermalMesh.on('anomaly:detected', (event) => {
    highlightNode(event.node_id, 'warning');
    queryClaudeForExplanation(event);
});

### Complete Reference Table

ComponentGitHubnpmSEED sensor firmware + ingestion[https://github.com/ruvnet/RuViewhttps://[www.npmjs.com/package/@ruv/rvcsiSensor](https://github.com/ruvnet/RuViewhttps://www.npmjs.com/package/@ruv/rvcsiSensor](https://www.npmjs.com/package/@ruv/rvcsiSensor](https://github.com/ruvnet/RuViewhttps://www.npmjs.com/package/@ruv/rvcsiSensor)) ingestion runtime[https://github.com/ruvnet/rvcsihttps://[www.npmjs.com/package/@ruv/rvcsiSensing-to-agent](https://github.com/ruvnet/rvcsihttps://www.npmjs.com/package/@ruv/rvcsiSensing-to-agent](https://www.npmjs.com/package/@ruv/rvcsiSensing-to-agent](https://github.com/ruvnet/rvcsihttps://www.npmjs.com/package/@ruv/rvcsiSensing-to-agent)) **MCP** bridge[https://github.com/ruvnet/RuViewhttps://[www.npmjs.com/package/@ruvnet/rvagentThermal](https://github.com/ruvnet/RuViewhttps://www.npmjs.com/package/@ruvnet/rvagentThermal](https://www.npmjs.com/package/@ruvnet/rvagentThermal](https://github.com/ruvnet/RuViewhttps://www.npmjs.com/package/@ruvnet/rvagentThermal)) neural inference (**LSTM**/N-**BEATS**)[https://github.com/ruvnet/ruv-FANNhttps://[www.npmjs.com/package/ruv-swarmEphemeral](https://github.com/ruvnet/ruv-FANNhttps://www.npmjs.com/package/ruv-swarmEphemeral](https://www.npmjs.com/package/ruv-swarmEphemeral](https://github.com/ruvnet/ruv-FANNhttps://www.npmjs.com/package/ruv-swarmEphemeral)) swarm agents[https://github.com/ruvnet/ruv-FANNhttps://[www.npmjs.com/package/ruv-swarmAutonomous](https://github.com/ruvnet/ruv-FANNhttps://www.npmjs.com/package/ruv-swarmAutonomous](https://www.npmjs.com/package/ruv-swarmAutonomous](https://github.com/ruvnet/ruv-FANNhttps://www.npmjs.com/package/ruv-swarmAutonomous)) thermal broker (**MRAP**)[https://github.com/ruvnet/daaCargo:](https://github.com/ruvnet/daaCargo:) daa-orchestrator, daa-economy, daa-rules, daa-aiFederated district model training[https://github.com/ruvnet/daaCargo:](https://github.com/ruvnet/daaCargo:) daa-prime-coordinator, daa-prime-trainer, daa-prime-dhtBuilding node isolation + audit[https://github.com/ruvnet/rvmCargo:](https://github.com/ruvnet/rvmCargo:) rvm-kernel, rvm-coherence, rvm-witness, rvm-securityQuantum-resistant trade consensus[https://github.com/ruvnet/QuDAGCargo:](https://github.com/ruvnet/QuDAGCargo:) qudagThermal state vector memory (**GNN**)[https://github.com/ruvnet/RuVectorhttps://[www.npmjs.com/package/ruvectorDistrict](https://github.com/ruvnet/RuVectorhttps://www.npmjs.com/package/ruvectorDistrict](https://www.npmjs.com/package/ruvectorDistrict](https://github.com/ruvnet/RuVectorhttps://www.npmjs.com/package/ruvectorDistrict)) swarm coordination[https://github.com/ruvnet/Synaptic-Mesh—AI](https://github.com/ruvnet/Synaptic-Mesh—AI) orchestration + Claude reasoning[https://github.com/ruvnet/ruflohttps://[www.npmjs.com/package/ruflo](https://github.com/ruvnet/ruflohttps://www.npmjs.com/package/ruflo](https://www.npmjs.com/package/ruflo](https://github.com/ruvnet/ruflohttps://www.npmjs.com/package/ruflo))

Cargo.toml for the Core Appliance Binary
toml[workspace]
members = [
    *crates/hoforras-sensor*,
    *crates/hoforras-broker*,
    *crates/hoforras-node*,
    *crates/hoforras-mesh*,
]

# crates/hoforras-broker/Cargo.toml

[dependencies] daa-rules             = *0.2.1* daa-economy           = *0.2.1* daa-ai                = *0.2.1* daa-prime-core        = *0.2.1* daa-prime-dht         = *0.2.1* daa-prime-trainer     = *0.2.1* daa-prime-coordinator = *0.2.1* tokio                 = { version = *1*, features = [*full*] } serde                 = { version = *1*, features = [*derive*] } postcard              = *1* ed25519-dalek         = *2*

# crates/hoforras-node/Cargo.toml — RVM isolation layer

[dependencies] rvm-kernel    = { path = *../vendor/rvm/crates/rvm-kernel* } rvm-coherence = { path = *../vendor/rvm/crates/rvm-coherence* } rvm-witness   = { path = *../vendor/rvm/crates/rvm-witness* } rvm-security  = { path = *../vendor/rvm/crates/rvm-security* } rvm-cap       = { path = *../vendor/rvm/crates/rvm-cap* }

package.json for the Dashboard and Orchestration Layer
json{
    *name*: *hoforras-dashboard*,
    *dependencies*: {
    *ruflo*: *latest*,
    *@ruv/rvcsi*: *latest*,
    *@ruvnet/rvagent*: *latest*,
    *ruv-swarm*: *1.0.19*,
    *ruvector*: *latest*,
    *react*: *^18*,
    *@react-three/fiber*: *^8*,
    *@react-three/drei*: *^9*,
    *three*: *^0.**163***
    }
}

First Milestone — Hőforrás Alpha, District **XIII** Pilot

Deploy 10 Cognitum **SEED** nodes on buildings using rvcsi thermal firmware fork — source at [https://github.com/ruvnet/rvcsi](https://github.com/ruvnet/rvcsi) Stand up one Cognitum Appliance as district coordinator running daa-orchestrator + rvm-kernel — source at [https://github.com/ruvnet/daa](https://github.com/ruvnet/daa) and [https://github.com/ruvnet/rvm](https://github.com/ruvnet/rvm) Establish a 30-day thermal baseline using ruv-**FANN** **LSTM** models — source at [https://github.com/ruvnet/ruv-**FANN**](https://github.com/ruvnet/ruv-**FANN**) Bring up daa broker between 3 buildings with manual governance rules in daa-rules Run QuDAG consensus for the first signed thermal trade agreement — source at [https://github.com/ruvnet/QuDAG](https://github.com/ruvnet/QuDAG) Surface the live dashboard with ruflo **MCP** + ruvector **GNN** state — [https://[www.npmjs.com/package/ruflo](https://www.npmjs.com/package/ruflo](https://www.npmjs.com/package/ruflo](https://www.npmjs.com/package/ruflo)) and [https://[www.npmjs.com/package/ruvector](https://www.npmjs.com/package/ruvector](https://www.npmjs.com/package/ruvector](https://www.npmjs.com/package/ruvector)) Execute one fully autonomous peer-to-peer thermal trade with no human intervention and a complete rvm-witness audit trail — source at [https://github.com/ruvnet/rvm](https://github.com/ruvnet/rvm)

That is the working invention — a real peer-to-peer thermal energy market running on edge hardware in Budapest, built almost entirely from Ruv's existing repos, in Rust.