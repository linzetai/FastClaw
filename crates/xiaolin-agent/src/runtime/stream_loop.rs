use std::sync::Arc;

use futures::Stream;
use xiaolin_protocol::AgentEvent;

use super::agent_context::AgentContext;
use super::agent_step::AgentStep;
use super::stream_engine::send_stream_event;
use super::turn_loop;
use super::turn_setup;
use super::{AgentRuntime, ExecutionParams, StreamParams};

impl AgentRuntime {
    /// Execute an agent turn as a composable async stream.
    ///
    /// This is the primary execution API. The stream yields `AgentStep` events
    /// as the agent processes LLM responses and tool calls.
    ///
    /// Internally, the execution logic is composed from extracted sub-functions:
    /// `turn_setup::setup_turn` → `turn_loop::run_turn_loop` (which orchestrates
    /// `iteration_check` → `llm_call` → `end_turn` / `tool_round` → `post_tool`).
    ///
    /// # Cancellation
    ///
    /// If `ctx.cancel_token` is set, the loop checks for cancellation at each
    /// iteration boundary. When cancelled, the stream emits an Error step and ends.
    /// Dropping the stream does NOT automatically cancel the spawned task — callers
    /// should cancel the token explicitly.
    pub fn execute_as_stream(
        runtime: Arc<Self>,
        ctx: AgentContext,
    ) -> impl Stream<Item = AgentStep> + Send + 'static {
        let side_path_tx = ctx.tx.clone();

        async_stream::stream! {
            let (internal_tx, mut internal_rx) = tokio::sync::mpsc::channel::<AgentEvent>(512);

            let cancel_token = ctx.cancel_token.clone();

            let handle = tokio::spawn(async move {
                let exec = ExecutionParams {
                    config: &ctx.config,
                    request: &ctx.request,
                    tool_registry: &ctx.tool_registry,
                    llm_override: ctx.llm_override.clone(),
                    subagent_prompt: ctx.subagent_prompt.clone(),
                    mode_state: ctx.mode_state.clone(),
                    session_store: ctx.session_store.clone(),
                    todo_store: ctx.todo_store.clone(),
                    goal_store: ctx.goal_store.clone(),
                    cost_store: ctx.cost_store.clone(),
                };
                let stream = StreamParams {
                    tx: internal_tx,
                    orchestrator: ctx.orchestrator.clone(),
                    interaction_handle: ctx.interaction_handle.clone(),
                    approval_strategy: ctx.approval_strategy.clone(),
                    runtime_registry: ctx.runtime_registry.clone(),
                };

                let (mut ms, svc) = turn_setup::setup_turn(
                    runtime,
                    &exec,
                    &stream,
                    cancel_token,
                ).await?;

                turn_loop::run_turn_loop(&mut ms, &svc).await
            });

            while let Some(event) = internal_rx.recv().await {
                match AgentStep::from_agent_event(event) {
                    Ok(step) => yield step,
                    Err(side_path_event) => {
                        if let Some(ref tx) = side_path_tx {
                            let _ = send_stream_event(tx, side_path_event, true).await;
                        }
                    }
                }
            }

            match handle.await {
                Ok(Ok(_summary)) => {
                    // TurnEnd was already emitted via the channel and yielded above.
                }
                Ok(Err(e)) => {
                    yield AgentStep::Error {
                        turn_id: xiaolin_protocol::TurnId::generate(),
                        message: e.to_string(),
                        error_code: None,
                        recoverable: false,
                    };
                }
                Err(join_err) => {
                    yield AgentStep::Error {
                        turn_id: xiaolin_protocol::TurnId::generate(),
                        message: format!("execution task panicked: {join_err}"),
                        error_code: None,
                        recoverable: false,
                    };
                }
            }
        }
    }
}
