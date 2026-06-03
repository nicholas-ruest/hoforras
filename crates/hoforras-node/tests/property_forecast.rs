//! Property test for the forecaster's resource-safety invariant (FR-2.1 / DDD-03 §4).
//!
//! For ANY interleaving of inference outcomes (Ok / Err), the `ForecastServiceImpl::with_agent`
//! guard must spawn and dissolve exactly once per call — `spawn` count == `dissolve` count. We drive
//! the real service with a counting factory (not a mock — we read the counters after the run, the
//! same pattern P1 uses for its no-raw-egress property).

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use hoforras_domain::{AgentProfile, DomainError, Inference, NodeId, ReadingWindow, Timestamp};
use hoforras_node::forecast::ForecastServiceImpl;
use hoforras_ports::inference::{ForecastService, InferenceAgent, SwarmFactory};
use hoforras_ports::PortResult;
use proptest::prelude::*;

fn window() -> ReadingWindow {
    ReadingWindow {
        node_id: NodeId::new("pozsonyi14.thermal.budapest.dark").unwrap(),
        from_ts: Timestamp(0),
        to_ts: Timestamp(1),
        frames: vec![],
    }
}

/// An agent that infers (or deliberately fails) and bumps a shared dissolve counter on teardown.
struct CountingAgent {
    dissolved: Arc<AtomicUsize>,
    fail: bool,
}

#[async_trait]
impl InferenceAgent for CountingAgent {
    async fn infer(&self, _window: ReadingWindow) -> PortResult<Inference> {
        if self.fail {
            Err(DomainError::Adapter("forced inference failure".into()))
        } else {
            Ok(Inference {
                scores: vec![0.1; 24],
            })
        }
    }

    fn dissolve(&self) {
        self.dissolved.fetch_add(1, Ordering::SeqCst);
    }
}

/// A factory that counts spawns and hands out `CountingAgent`s with a fixed failure mode.
struct CountingFactory {
    spawned: Arc<AtomicUsize>,
    dissolved: Arc<AtomicUsize>,
    fail: bool,
}

#[async_trait]
impl SwarmFactory for CountingFactory {
    async fn spawn(&self, _profile: AgentProfile) -> PortResult<Box<dyn InferenceAgent>> {
        self.spawned.fetch_add(1, Ordering::SeqCst);
        Ok(Box::new(CountingAgent {
            dissolved: self.dissolved.clone(),
            fail: self.fail,
        }))
    }
}

proptest! {
    /// FR-2.1: across any mix of Ok/Err inference outcomes, every spawn is paired with exactly one
    /// dissolve.
    #[test]
    fn prop_spawn_count_equals_dissolve_count(outcomes in proptest::collection::vec(any::<bool>(), 1..32)) {
        let rt = tokio::runtime::Builder::new_current_thread().build().unwrap();
        let spawned = Arc::new(AtomicUsize::new(0));
        let dissolved = Arc::new(AtomicUsize::new(0));

        rt.block_on(async {
            for &fail in &outcomes {
                let factory = CountingFactory {
                    spawned: spawned.clone(),
                    dissolved: dissolved.clone(),
                    fail,
                };
                let svc = ForecastServiceImpl::new(factory);
                let _ = svc.anomalies(window()).await; // ignore Ok/Err — the invariant must hold regardless
            }
        });

        prop_assert_eq!(spawned.load(Ordering::SeqCst), outcomes.len());
        prop_assert_eq!(dissolved.load(Ordering::SeqCst), outcomes.len());
        prop_assert_eq!(spawned.load(Ordering::SeqCst), dissolved.load(Ordering::SeqCst));
    }
}
