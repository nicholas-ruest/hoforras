//! `AnomalyExplainer` (DDD-08 / FR-8.2) — explain an anomaly with Claude, then surface it.
//!
//! Strict order: ask `ClaudeReasoner` for a natural-language explanation **first**, then push the
//! `AnomalyExplained` event to the operator. The dashboard never shows an unexplained anomaly card.

use hoforras_domain::{AnomalyContext, UiEvent};
use hoforras_ports::ai::{ClaudeReasoner, OperatorChannel};
use hoforras_ports::PortResult;

/// Generic over its two ports (London-School, ADR-0001).
pub struct AnomalyExplainer<C, O> {
    claude: C,
    operator: O,
}

impl<C, O> AnomalyExplainer<C, O>
where
    C: ClaudeReasoner,
    O: OperatorChannel,
{
    pub fn new(claude: C, operator: O) -> Self {
        Self { claude, operator }
    }

    /// Explain then surface (FR-8.2): `claude.explain(ctx)` precedes `operator.push(...)`.
    pub async fn explain_and_surface(&self, context: AnomalyContext) -> PortResult<()> {
        let node_id = context.node_id.clone();
        let explanation = self.claude.explain(context).await?; // explain FIRST
        self.operator
            .push(UiEvent::AnomalyExplained {
                node_id,
                explanation,
            })
            .await // THEN surface
    }
}
