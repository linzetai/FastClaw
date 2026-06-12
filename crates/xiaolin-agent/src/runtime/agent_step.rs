use xiaolin_protocol::event::GoalData;
use xiaolin_protocol::{AgentEvent, ContextWarningLevel, ErrorCode, ExecutionMode, TurnId, TurnSummary};

use super::query_state::TerminalReason;

/// Events produced by the main agent loop (`execute_as_stream`).
///
/// Side-path events (ToolProgress, ApprovalRequired, SubAgent*, etc.) are emitted
/// directly to `tx` by orchestrator/tools and are NOT represented here.
#[derive(Debug, Clone)]
pub enum AgentStep {
    TurnStart {
        turn_id: TurnId,
        session_id: Option<String>,
    },

    /// LLM streaming content delta. `delta` matches `AgentEvent::ContentDelta.delta`.
    Delta {
        turn_id: TurnId,
        delta: serde_json::Value,
        #[allow(dead_code)]
        raw_bytes: Option<bytes::Bytes>,
    },

    /// Reasoning/thinking content (separate from visible content).
    ReasoningDelta {
        turn_id: TurnId,
        content: String,
    },

    ToolExecuting {
        turn_id: TurnId,
        call_id: String,
        tool_name: String,
        args: Option<String>,
    },

    ToolResult {
        turn_id: TurnId,
        call_id: String,
        tool_name: String,
        output: String,
        display_output: Option<String>,
        success: bool,
        metadata: Option<serde_json::Value>,
    },

    /// Marks the boundary between tool rounds (after all results, before next LLM call).
    ToolRoundBoundary {
        iteration: u32,
    },

    /// Steering messages were injected at a tool-round boundary.
    SteeringInjected {
        count: usize,
        sources: Vec<String>,
    },

    ContextUsage {
        turn_id: TurnId,
        used_tokens: u32,
        limit_tokens: u32,
        compressed: bool,
        tokens_saved: u32,
    },

    ContextWarning {
        turn_id: TurnId,
        level: ContextWarningLevel,
        used_tokens: u32,
        limit_tokens: u32,
        message: String,
    },

    ModeChange {
        turn_id: TurnId,
        from: ExecutionMode,
        to: ExecutionMode,
    },

    PlanFileUpdate {
        turn_id: TurnId,
        session_id: String,
        path: String,
        exists: bool,
    },

    GoalUpdated {
        turn_id: TurnId,
        goal: GoalData,
    },

    GoalCleared {
        turn_id: TurnId,
        goal_id: String,
    },

    TurnEnd {
        turn_id: TurnId,
        reason: TurnEndReason,
        summary: TurnSummary,
        session_id: Option<String>,
    },

    Error {
        turn_id: TurnId,
        message: String,
        error_code: Option<ErrorCode>,
        recoverable: bool,
    },
}

/// Why the agent loop terminated (aligned with `TerminalReason`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TurnEndReason {
    Completed,
    MaxTurns,
    Cancelled,
    ContextLimit,
    BudgetExceeded,
    ConsecutiveErrors,
    DiminishingReturns,
    PlanApprovalPending,
    Error(String),
}

impl From<TerminalReason> for TurnEndReason {
    fn from(r: TerminalReason) -> Self {
        match r {
            TerminalReason::EndTurn => Self::Completed,
            TerminalReason::MaxIterations => Self::MaxTurns,
            TerminalReason::Aborted => Self::Cancelled,
            TerminalReason::BlockingLimit => Self::ContextLimit,
            TerminalReason::BudgetExhausted => Self::BudgetExceeded,
            TerminalReason::ConsecutiveErrors => Self::ConsecutiveErrors,
            TerminalReason::DiminishingReturns => Self::DiminishingReturns,
        }
    }
}

impl AgentStep {
    /// Convert this step into zero or more `AgentEvent`s for the compatibility layer.
    ///
    /// Internal markers (`ToolRoundBoundary`, `SteeringInjected`) produce no events.
    /// Most steps produce exactly one event; `Delta` with reasoning may produce two.
    pub fn into_agent_events(self) -> Vec<AgentEvent> {
        match self {
            Self::TurnStart { turn_id, session_id } => {
                vec![AgentEvent::TurnStart { turn_id, session_id }]
            }

            Self::Delta { turn_id, delta, raw_bytes } => {
                vec![AgentEvent::ContentDelta { turn_id, delta, raw_bytes }]
            }

            Self::ReasoningDelta { turn_id, content } => {
                vec![AgentEvent::ReasoningDelta { turn_id, content }]
            }

            Self::ToolExecuting { turn_id, call_id, tool_name, args } => {
                vec![AgentEvent::ToolExecuting { turn_id, tool_name, call_id, args }]
            }

            Self::ToolResult { turn_id, call_id, tool_name, output, display_output, success, metadata } => {
                vec![AgentEvent::ToolResult {
                    turn_id,
                    tool_name,
                    call_id,
                    output,
                    display_output,
                    success,
                    metadata,
                }]
            }

            Self::ContextUsage { turn_id, used_tokens, limit_tokens, compressed, tokens_saved } => {
                vec![AgentEvent::ContextUsageUpdate {
                    turn_id,
                    used_tokens,
                    limit_tokens,
                    compressed,
                    tokens_saved,
                }]
            }

            Self::ContextWarning { turn_id, level, used_tokens, limit_tokens, message } => {
                vec![AgentEvent::ContextWarning {
                    turn_id,
                    level,
                    used_tokens,
                    limit_tokens,
                    message,
                }]
            }

            Self::ModeChange { turn_id, from, to } => {
                vec![AgentEvent::ModeChange { turn_id, from, to }]
            }

            Self::PlanFileUpdate { turn_id, session_id, path, exists } => {
                vec![AgentEvent::PlanFileUpdate { turn_id, session_id, path, exists }]
            }

            Self::GoalUpdated { turn_id, goal } => {
                vec![AgentEvent::GoalUpdated { turn_id, goal }]
            }

            Self::GoalCleared { turn_id, goal_id } => {
                vec![AgentEvent::GoalCleared { turn_id, goal_id }]
            }

            Self::TurnEnd { turn_id, summary, session_id, .. } => {
                vec![AgentEvent::TurnEnd {
                    turn_id,
                    summary,
                    session_id,
                    final_tool_calls: None,
                }]
            }

            Self::Error { turn_id, message, error_code, .. } => {
                vec![AgentEvent::Error { turn_id, message, error_code }]
            }

            Self::ToolRoundBoundary { .. } | Self::SteeringInjected { .. } => vec![],
        }
    }

    /// Whether this step can be dropped when the channel is full.
    pub fn is_lossy(&self) -> bool {
        matches!(self, Self::ContextUsage { .. } | Self::ContextWarning { .. })
    }

    /// Try to convert an `AgentEvent` into an `AgentStep`.
    ///
    /// Returns `Ok(step)` for main-loop events that map to `AgentStep` variants.
    /// Returns `Err(event)` for side-path events (ToolProgress, SubAgent*, Approval*, etc.)
    /// which should be forwarded to `ctx.tx` directly.
    #[allow(clippy::result_large_err)]
    pub fn from_agent_event(event: AgentEvent) -> Result<Self, AgentEvent> {
        match event {
            AgentEvent::TurnStart { turn_id, session_id } => Ok(Self::TurnStart { turn_id, session_id }),

            AgentEvent::ContentDelta { turn_id, delta, raw_bytes } => Ok(Self::Delta { turn_id, delta, raw_bytes }),

            AgentEvent::ReasoningDelta { turn_id, content } => Ok(Self::ReasoningDelta { turn_id, content }),

            AgentEvent::ToolExecuting { turn_id, tool_name, call_id, args } => {
                Ok(Self::ToolExecuting { turn_id, call_id, tool_name, args })
            }

            AgentEvent::ToolResult { turn_id, tool_name, call_id, output, display_output, success, metadata } => {
                Ok(Self::ToolResult { turn_id, call_id, tool_name, output, display_output, success, metadata })
            }

            AgentEvent::ContextUsageUpdate { turn_id, used_tokens, limit_tokens, compressed, tokens_saved } => {
                Ok(Self::ContextUsage { turn_id, used_tokens, limit_tokens, compressed, tokens_saved })
            }

            AgentEvent::ContextWarning { turn_id, level, used_tokens, limit_tokens, message } => {
                Ok(Self::ContextWarning { turn_id, level, used_tokens, limit_tokens, message })
            }

            AgentEvent::ModeChange { turn_id, from, to } => Ok(Self::ModeChange { turn_id, from, to }),

            AgentEvent::PlanFileUpdate { turn_id, session_id, path, exists } => {
                Ok(Self::PlanFileUpdate { turn_id, session_id, path, exists })
            }

            AgentEvent::GoalUpdated { turn_id, goal } => Ok(Self::GoalUpdated { turn_id, goal }),

            AgentEvent::GoalCleared { turn_id, goal_id } => Ok(Self::GoalCleared { turn_id, goal_id }),

            AgentEvent::TurnEnd { turn_id, summary, session_id, .. } => {
                Ok(Self::TurnEnd {
                    turn_id,
                    reason: TurnEndReason::Completed,
                    summary,
                    session_id,
                })
            }

            AgentEvent::Error { turn_id, message, error_code } => {
                Ok(Self::Error { turn_id, message, error_code, recoverable: false })
            }

            AgentEvent::TurnAborted { turn_id, reason, .. } => {
                Ok(Self::Error {
                    turn_id,
                    message: format!("turn aborted: {reason:?}"),
                    error_code: None,
                    recoverable: false,
                })
            }

            AgentEvent::StreamError { turn_id, message, error_code, .. } => {
                Ok(Self::Error { turn_id, message, error_code, recoverable: true })
            }

            // Side-path events — not part of the main agent loop stream.
            // Forward to ctx.tx for direct consumption.
            other => Err(other),
        }
    }
}
