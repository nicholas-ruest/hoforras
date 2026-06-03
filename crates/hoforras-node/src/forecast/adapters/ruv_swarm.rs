//! `RuvSwarmAdapter` — reference ephemeral-agent factory over ruv-swarm (ADR-0012 / FR-2.1).
//!
//! ruv-swarm spawns a tiny specialist network per prediction and dissolves it afterwards; the agent
//! never outlives the call. The real ruv-swarm crate is unavailable here, so this reference factory
//! hosts the [`NeuroDivergentAdapter`] model **in-process** (node-local, ADR-0009) and tracks
//! spawn/dissolve counts so resource safety (spawn count == dissolve count) is observable on the
//! real adapter too — not only under mocks.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use hoforras_domain::{AgentProfile, Inference, ReadingWindow};
use hoforras_ports::inference::{InferenceAgent, SwarmFactory};
use hoforras_ports::PortResult;

use super::neuro_divergent::{ForecastTask, NeuroDivergentAdapter};

/// Reference ruv-swarm factory. Cheap to clone the shared counters; create one per Appliance.
#[derive(Default)]
pub struct RuvSwarmAdapter {
    spawned: Arc<AtomicUsize>,
    dissolved: Arc<AtomicUsize>,
}

impl RuvSwarmAdapter {
    pub fn new() -> Self {
        Self::default()
    }

    /// How many ephemeral agents have been spawned.
    pub fn spawned(&self) -> usize {
        self.spawned.load(Ordering::SeqCst)
    }

    /// How many ephemeral agents have been dissolved. Should always equal [`spawned`] once calls
    /// have returned (FR-2.1).
    pub fn dissolved(&self) -> usize {
        self.dissolved.load(Ordering::SeqCst)
    }
}

#[async_trait]
impl SwarmFactory for RuvSwarmAdapter {
    async fn spawn(&self, profile: AgentProfile) -> PortResult<Box<dyn InferenceAgent>> {
        self.spawned.fetch_add(1, Ordering::SeqCst);
        Ok(Box::new(RuvSwarmAgent {
            task: ForecastTask::from_spec(&profile.specialization),
            model: NeuroDivergentAdapter::new(),
            dissolved: self.dissolved.clone(),
        }))
    }
}

/// An ephemeral specialist agent. Holds its model by value (on-device) and bumps the shared
/// dissolve counter when torn down.
struct RuvSwarmAgent {
    task: ForecastTask,
    model: NeuroDivergentAdapter,
    dissolved: Arc<AtomicUsize>,
}

#[async_trait]
impl InferenceAgent for RuvSwarmAgent {
    async fn infer(&self, window: ReadingWindow) -> PortResult<Inference> {
        // On-device, synchronous compute — no network, no GPU (ADR-0012).
        Ok(self.model.infer(self.task, &window))
    }

    fn dissolve(&self) {
        self.dissolved.fetch_add(1, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::forecast::service::SPEC_DEMAND;
    use hoforras_domain::{NodeId, ReadingWindow, Timestamp};

    fn window() -> ReadingWindow {
        ReadingWindow {
            node_id: NodeId::new("pozsonyi14.thermal.budapest.dark").unwrap(),
            from_ts: Timestamp(0),
            to_ts: Timestamp(1),
            frames: vec![],
        }
    }

    #[tokio::test]
    async fn spawn_then_dissolve_balances_counters() {
        let factory = RuvSwarmAdapter::new();
        let agent = factory
            .spawn(AgentProfile {
                specialization: SPEC_DEMAND.into(),
                neural_model: "n-beats".into(),
                analytical: 0.95,
                systematic: 0.9,
            })
            .await
            .unwrap();
        assert_eq!(factory.spawned(), 1);
        assert_eq!(factory.dissolved(), 0);

        let inf = agent.infer(window()).await.unwrap();
        assert_eq!(inf.scores.len(), 24); // demand head

        agent.dissolve();
        assert_eq!(factory.dissolved(), 1);
    }
}
