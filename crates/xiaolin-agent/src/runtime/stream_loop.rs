use futures::Stream;

use super::agent_context::AgentContext;
use super::agent_step::AgentStep;
use super::AgentRuntime;

impl AgentRuntime {
    /// Execute an agent turn as a composable async stream.
    ///
    /// This is the new core execution primitive. The stream yields `AgentStep`
    /// events as the agent processes LLM responses and tool calls.
    ///
    /// The existing `execute_unified` / `execute_stream` APIs are preserved as
    /// compatibility wrappers that consume this stream and forward events to
    /// an `mpsc::Sender<AgentEvent>`.
    ///
    /// # Cancellation
    ///
    /// Dropping the stream cancels the internal `CancellationToken`, aborting
    /// any in-flight LLM call and releasing SpawnReservation slots via RAII.
    pub fn execute_as_stream(
        &self,
        ctx: AgentContext,
    ) -> impl Stream<Item = AgentStep> + '_ {
        async_stream::stream! {
            let turn_id = xiaolin_protocol::TurnId::generate();

            yield AgentStep::TurnStart {
                turn_id: turn_id.clone(),
                session_id: ctx.request.session_id.as_ref().map(|s| s.to_string()),
            };

            // TODO(Phase 1c): Full implementation — setup, agentic loop,
            // tool rounds, boundary processing. Currently stubs with Error.
            yield AgentStep::Error {
                turn_id,
                message: "execute_as_stream: not yet implemented".into(),
                error_code: None,
                recoverable: false,
            };
        }
    }
}
