use std::sync::Arc;

use futures::Stream;

use super::agent_context::AgentContext;
use super::agent_step::AgentStep;
use super::turn_loop;
use super::turn_setup;
use super::AgentRuntime;

impl AgentRuntime {
    /// Execute an agent turn as a composable async stream.
    ///
    /// This is the primary execution API. The stream yields `AgentStep` events
    /// as the agent processes LLM responses and tool calls.
    ///
    /// Architecture: two channels carry events out of the spawned execution task:
    /// - `step_tx` → `step_rx`: main-loop events yielded directly as `AgentStep`
    /// - `event_tx` (caller's channel): side-path events forwarded by the tool dispatcher
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
        async_stream::stream! {
            let (step_tx, mut step_rx) = tokio::sync::mpsc::channel::<AgentStep>(512);

            let handle = tokio::spawn(async move {
                let mut ctx = ctx;
                ctx.step_tx = Some(step_tx);
                // ctx.event_tx is already set by the caller (for side-path events)

                let (mut ms, svc) = turn_setup::setup_turn(runtime, &ctx).await?;
                turn_loop::run_turn_loop(&mut ms, &svc).await
            });

            while let Some(step) = step_rx.recv().await {
                yield step;
            }

            match handle.await {
                Ok(Ok(_summary)) => {
                    // TurnEnd was already emitted via step_tx and yielded above.
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
