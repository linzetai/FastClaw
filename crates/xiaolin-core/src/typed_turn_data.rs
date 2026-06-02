use std::any::Any;
use std::sync::Arc;

use crate::agent_config::AgentConfig;
use crate::types::ChatRequest;

/// Typed data passed through `SessionOp::UserTurn` to avoid JSON round-trips.
///
/// This struct is stored as `Arc<dyn Any + Send + Sync>` in the session actor's
/// `TurnParams::typed_data` field. The gateway wraps it when submitting a turn,
/// and the session bridge downcasts it to extract the typed request and config.
pub struct TypedTurnData {
    pub enriched_request: ChatRequest,
    pub agent_config: AgentConfig,
    /// Per-request LLM provider override (type-erased `Arc<dyn LlmProvider>`).
    /// Used when the user pins a model that requires a different provider endpoint.
    pub llm_override: Option<Arc<dyn Any + Send + Sync>>,
}

impl TypedTurnData {
    pub fn wrap(request: ChatRequest, config: AgentConfig) -> Arc<dyn Any + Send + Sync> {
        Arc::new(Self {
            enriched_request: request,
            agent_config: config,
            llm_override: None,
        })
    }

    pub fn wrap_with_llm_override(
        request: ChatRequest,
        config: AgentConfig,
        llm_override: Option<Arc<dyn Any + Send + Sync>>,
    ) -> Arc<dyn Any + Send + Sync> {
        Arc::new(Self {
            enriched_request: request,
            agent_config: config,
            llm_override,
        })
    }

    pub fn extract(data: &Arc<dyn Any + Send + Sync>) -> Option<&Self> {
        data.downcast_ref::<Self>()
    }
}
