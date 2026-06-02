//! Inference & Forecasting ports (DDD-03). Edge-only; no remote-inference port exists (ADR-0012).

use async_trait::async_trait;
use hoforras_domain::{
    AgentProfile, Anomaly, BurstWarning, DemandForecast, Inference, ReadingWindow,
};

use crate::PortResult;

/// Spawns an ephemeral specialist inference agent (ruv-swarm). The returned agent is `dissolve`d
/// after use — the spawn→infer→dissolve lifecycle (FR-2.1) is orchestrated in Prompt 3.
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
#[async_trait]
pub trait SwarmFactory: Send + Sync {
    async fn spawn(&self, profile: AgentProfile) -> PortResult<Box<dyn InferenceAgent>>;
}

/// An ephemeral inference agent. `dissolve` must be called after `infer`, even on error (FR-2.1).
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
#[async_trait]
pub trait InferenceAgent: Send + Sync {
    async fn infer(&self, window: ReadingWindow) -> PortResult<Inference>;
    fn dissolve(&self);
}

/// High-level forecasting service consumed by the broker's Reason step (FR-2.2–2.4).
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
#[async_trait]
pub trait ForecastService: Send + Sync {
    async fn anomalies(&self, window: ReadingWindow) -> PortResult<Vec<Anomaly>>;
    async fn burst_precursor(&self, window: ReadingWindow) -> PortResult<Option<BurstWarning>>;
    async fn demand_24h(&self, window: ReadingWindow) -> PortResult<DemandForecast>;
}
