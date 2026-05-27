//! Bridge between `fastclaw-session-actor`'s `TurnExecutor` trait and
//! `AgentRuntime`'s `execute_unified`.
//!
//! With Phase A, the relay tasks have been removed. `InteractionHandle` is
//! passed directly to `ToolOrchestrator` (via `StreamParams`) and to builtin
//! tools (via task-local), so approvals and answers resolve without polling.

use std::sync::Arc;

use dashmap::DashMap;
use fastclaw_core::agent_config::AgentConfig;
use fastclaw_core::tool::ToolRegistry;
use fastclaw_core::types::ChatRequest;
use fastclaw_protocol::{AgentEvent, AgentId, SessionId};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use fastclaw_session_actor::{InteractionHandle, TurnError, TurnExecutor, TurnParams, TurnResult};

use crate::llm::LlmProvider;
use crate::AgentRuntime;

/// Adapter implementing `TurnExecutor` by delegating to `AgentRuntime`.
///
/// `InteractionHandle` is threaded into two places:
/// 1. `StreamParams.interaction_handle` — used by `ToolOrchestrator` for
///    approval resolution.
/// 2. Task-local `TASK_INTERACTION_HANDLE` — used by builtin tools
///    (`ask_question`, `confirm`) for answer resolution.
pub struct RuntimeTurnExecutor {
    pub runtime: Arc<AgentRuntime>,
    pub config: AgentConfig,
    pub tool_registry: Arc<ToolRegistry>,
    pub llm_override: Option<Arc<dyn LlmProvider>>,
    pub session_store: Option<Arc<fastclaw_session::SessionStore>>,
    pub mode_registry: Option<crate::builtin_tools::SessionModeRegistry>,
    pub todo_store: Option<crate::builtin_tools::TodoStore>,
    pub plan_file_store: Option<crate::builtin_tools::PlanFileStore>,
    pub stream_event_tx:
        Option<Arc<DashMap<String, mpsc::Sender<AgentEvent>>>>,
    pub subagent_manager: Option<Arc<crate::SubAgentManager>>,
    pub tool_orchestrator: Option<Arc<crate::runtime::orchestrator::ToolOrchestrator>>,
}

impl RuntimeTurnExecutor {
    /// Run explicit context compaction (manual `/compact` or `SessionOp::Compact`).
    async fn execute_compact(
        &self,
        params: &TurnParams,
        tx: &mpsc::Sender<AgentEvent>,
    ) -> Result<TurnResult, TurnError> {
        let session_id = params.session_id.to_string();
        let Some(ref store) = self.session_store else {
            return Err(TurnError::Runtime {
                message: "session store not available for compaction".into(),
                code: fastclaw_protocol::event::ErrorCode::Other,
            });
        };

        let mut messages = store
            .load_chat_messages(&session_id)
            .await
            .map_err(|e| TurnError::Runtime {
                message: format!("failed to load messages: {e}"),
                code: fastclaw_protocol::event::ErrorCode::Other,
            })?;

        let pre_count = messages.len();
        let pre_tokens = fastclaw_context::compressor::estimate_messages_tokens(&messages);

        let context_window = self
            .config
            .model
            .context_window
            .unwrap_or(128_000);

        fastclaw_context::ContextEngine::fit_to_context_window(
            &mut messages,
            context_window,
            self.config.model.max_tokens,
        );

        let post_tokens = fastclaw_context::compressor::estimate_messages_tokens(&messages);
        let post_count = messages.len();
        let removed = pre_count.saturating_sub(post_count);

        if removed > 0 {
            if let Err(e) = store.replace_messages(&session_id, &messages).await {
                tracing::warn!(error = %e, "failed to persist compacted messages");
            }
        }

        let _ = tx
            .send(AgentEvent::CompactBoundary {
                turn_id: params.turn_id.clone(),
                trigger: fastclaw_protocol::CompactTrigger::Manual,
                pre_compact_tokens: pre_tokens,
                post_compact_tokens: post_tokens,
                messages_removed: removed,
            })
            .await;

        tracing::info!(
            session_id = %session_id,
            pre_tokens,
            post_tokens,
            messages_removed = removed,
            "manual compaction complete"
        );

        Ok(TurnResult {
            tool_calls_made: 0,
            iterations: 0,
            usage: None,
        })
    }

    /// Auto-compact if the current session's messages exceed a token threshold.
    async fn maybe_auto_compact(
        &self,
        params: &TurnParams,
        tx: &mpsc::Sender<AgentEvent>,
    ) {
        let Some(ref store) = self.session_store else {
            return;
        };
        let context_window = self
            .config
            .model
            .context_window
            .unwrap_or(128_000);

        // Trigger auto-compact at 85% of context window.
        let threshold = (context_window as f64 * 0.85) as usize;
        let session_id = params.session_id.to_string();

        let mut messages = match store.load_chat_messages(&session_id).await {
            Ok(m) => m,
            Err(_) => return,
        };

        let estimated = fastclaw_context::compressor::estimate_messages_tokens(&messages);
        if estimated <= threshold {
            return;
        }

        tracing::info!(
            session_id = %session_id,
            estimated,
            threshold,
            context_window,
            "auto-compacting: token estimate exceeds threshold"
        );

        let pre_tokens = estimated;
        let pre_count = messages.len();

        fastclaw_context::ContextEngine::fit_to_context_window(
            &mut messages,
            context_window,
            self.config.model.max_tokens,
        );

        let post_tokens = fastclaw_context::compressor::estimate_messages_tokens(&messages);
        let post_count = messages.len();
        let removed = pre_count.saturating_sub(post_count);

        if removed > 0 {
            if let Err(e) = store.replace_messages(&session_id, &messages).await {
                tracing::warn!(error = %e, "failed to persist auto-compacted messages");
            }

            let _ = tx
                .send(AgentEvent::CompactBoundary {
                    turn_id: params.turn_id.clone(),
                    trigger: fastclaw_protocol::CompactTrigger::Auto,
                    pre_compact_tokens: pre_tokens,
                    post_compact_tokens: post_tokens,
                    messages_removed: removed,
                })
                .await;
        }
    }
}

#[async_trait::async_trait]
impl TurnExecutor for RuntimeTurnExecutor {
    async fn execute(
        &self,
        params: TurnParams,
        interaction: InteractionHandle,
        tx: mpsc::Sender<AgentEvent>,
        cancel: CancellationToken,
    ) -> Result<TurnResult, TurnError> {
        let is_compact = params
            .extra
            .get("_compact")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if is_compact {
            return self
                .execute_compact(&params, &tx)
                .await;
        }

        // Auto-compact: check message token count before starting the turn.
        // If exceeding threshold, run inline compaction.
        self.maybe_auto_compact(&params, &tx).await;

        let request: ChatRequest = params
            .extra
            .get("_enriched_request")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_else(|| ChatRequest {
                messages: serde_json::from_value(params.messages.clone()).unwrap_or_default(),
                session_id: Some(SessionId::new(params.session_id.to_string())),
                agent_id: Some(AgentId::new(params.agent_id.clone())),
                model: params.model.clone(),
                max_tokens: None,
                temperature: None,
                stream: true,
                tools: None,
                work_dir: params.work_dir.clone(),
                slash_intent: None,
            });

        let config: AgentConfig = params
            .extra
            .get("_agent_config")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_else(|| self.config.clone());

        let orchestrator = self.tool_orchestrator.clone().unwrap_or_else(|| {
            Arc::new(crate::runtime::orchestrator::ToolOrchestrator::new())
        });

        // Wrap the outbound tx to inject session_id into interaction events.
        let session_id_str = params.session_id.to_string();
        let (inner_tx, mut inner_rx) = mpsc::channel::<AgentEvent>(128);

        let stream_context_key = params
            .extra
            .get("_stream_context_key")
            .and_then(|v| v.as_str())
            .map(String::from)
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        if let Some(ref map) = self.stream_event_tx {
            map.insert(stream_context_key.clone(), inner_tx.clone());
        }

        let mode_state = self
            .mode_registry
            .as_ref()
            .map(|r| r.get_or_create(&params.session_id.to_string()));

        let plan_ctx = self.plan_file_store.as_ref().map(|store| {
            crate::builtin_tools::PlanContext {
                session_id: params.session_id.to_string(),
                store: store.clone(),
            }
        });

        let subagent_prompt = self.subagent_manager.as_ref().and_then(|mgr| {
            let policy = &config.behavior.subagent;
            let available = mgr.agent_descriptions();
            let ctx = crate::SubAgentPromptContext {
                policy,
                available_agents: &available,
                current_depth: 0,
            };
            crate::build_subagent_prompt_block(&ctx)
        });

        let injector = {
            let outer_tx = tx.clone();
            let sid = session_id_str.clone();
            tokio::spawn(async move {
                while let Some(mut event) = inner_rx.recv().await {
                    match &mut event {
                        AgentEvent::ApprovalRequired { session_id, .. }
                        | AgentEvent::AskQuestion { session_id, .. } => {
                            if session_id.is_none() {
                                *session_id = Some(sid.clone());
                            }
                        }
                        _ => {}
                    }
                    if outer_tx.send(event).await.is_err() {
                        break;
                    }
                }
            })
        };

        // Spawn a task to drain steer messages. The current agentic loop
        // does not support mid-turn injection yet, but we consume the channel
        // so the actor doesn't block on sends.
        let steer_drain = {
            let mut steer_rx = params.steer_rx;
            tokio::spawn(async move {
                while let Some(msg) = steer_rx.recv().await {
                    tracing::debug!(
                        role = %msg.role,
                        content_len = msg.content.len(),
                        "received steer message (consumed but not yet injected into agentic loop)"
                    );
                }
            })
        };

        let result = {
            let runtime = self.runtime.clone();
            let tool_registry = self.tool_registry.clone();
            let llm = self.llm_override.clone();
            let session_store = self.session_store.clone();
            let todo_store = self.todo_store.clone();
            let stream_ctx_key_inner = stream_context_key.clone();
            let ih_for_tools = interaction.clone();

            let runtime_fut: std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<fastclaw_protocol::TurnSummary>> + Send>> = Box::pin(runtime.execute_unified(
                &config,
                &request,
                &tool_registry,
                inner_tx,
                fastclaw_core::tool_runtime::ApprovalStrategy::Interactive,
                llm,
                orchestrator.clone(),
                Some(interaction),
                subagent_prompt,
                mode_state.clone(),
                session_store,
                todo_store,
            ));

            let wrapped_fut = async move {
                let runtime_with_ih = crate::builtin_tools::with_interaction_handle(
                    ih_for_tools,
                    runtime_fut,
                );
                if let Some(ms) = mode_state {
                    crate::builtin_tools::with_stream_context(
                        stream_ctx_key_inner,
                        crate::builtin_tools::with_session_mode(ms, plan_ctx, runtime_with_ih),
                    )
                    .await
                } else {
                    crate::builtin_tools::with_stream_context(
                        stream_ctx_key_inner,
                        runtime_with_ih,
                    )
                    .await
                }
            };

            tokio::select! {
                r = wrapped_fut => r,
                () = cancel.cancelled() => Err(anyhow::anyhow!("cancelled by session actor")),
            }
        };

        injector.abort();
        steer_drain.abort();

        if let Some(ref map) = self.stream_event_tx {
            map.remove(&stream_context_key);
        }

        match result {
            Ok(summary) => Ok(TurnResult {
                tool_calls_made: summary.tool_calls_made,
                iterations: summary.iterations,
                usage: summary.usage,
            }),
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("cancelled by session actor") {
                    Err(TurnError::Cancelled)
                } else {
                    let code = fastclaw_protocol::event::ErrorCode::classify(&msg);
                    tracing::error!(error = %msg, code = ?code, "turn execution failed");
                    Err(TurnError::Runtime { message: msg, code })
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ToolOrchestrator;

    #[test]
    fn default_orchestrator_construction() {
        let orch = ToolOrchestrator::new();
        let _default: ToolOrchestrator = Default::default();
        drop(orch);
    }
}
