//! Replay driver: deterministic replay of recorded [`ConversationTrace`] data.

use std::collections::HashMap;

use async_trait::async_trait;

use fastclaw_core::types::ConversationTrace;

use crate::{EvalAgentDriver, EvalCase, EvalRunArtifacts};

/// Difference between two turns at the same index.
#[derive(Debug, Clone)]
pub struct TurnDiff {
    pub turn_index: u32,
    pub original_tool_calls: Vec<String>,
    pub replayed_tool_calls: Vec<String>,
    pub original_response: Option<String>,
    pub replayed_response: Option<String>,
    pub tools_match: bool,
    pub response_match: bool,
}

/// Result of replaying a trace against a new execution.
#[derive(Debug, Clone)]
pub struct ReplayResult {
    pub trace_id: String,
    pub original_turns: u32,
    pub replayed_turns: u32,
    pub turn_diffs: Vec<TurnDiff>,
    pub fully_matches: bool,
}

impl ReplayResult {
    /// Compare an original trace against a new run's artifacts.
    pub fn compare(trace: &ConversationTrace, artifacts: &EvalRunArtifacts) -> Self {
        let mut turn_diffs = Vec::new();

        for turn in &trace.turns {
            let original_tools: Vec<String> =
                turn.tool_calls.iter().map(|tc| tc.tool_name.clone()).collect();
            let original_response = turn.assistant_message.text_content();
            let orig_resp_str = original_response.as_deref().map(|s| s.to_string());

            let diff = TurnDiff {
                turn_index: turn.turn_index,
                original_tool_calls: original_tools.clone(),
                replayed_tool_calls: artifacts.tool_calls_made.clone(),
                original_response: orig_resp_str.clone(),
                replayed_response: artifacts.final_response.clone(),
                tools_match: original_tools == artifacts.tool_calls_made,
                response_match: orig_resp_str == artifacts.final_response,
            };
            turn_diffs.push(diff);
        }

        let fully_matches = turn_diffs.iter().all(|d| d.tools_match && d.response_match);

        ReplayResult {
            trace_id: trace.trace_id.clone(),
            original_turns: trace.turns.len() as u32,
            replayed_turns: artifacts.total_turns,
            turn_diffs,
            fully_matches,
        }
    }
}

/// Replays traces deterministically by returning stored artifacts from traces.
/// Maps case ID → trace, extracting tool calls and final response from the trace data.
pub struct ReplayDriver {
    traces: HashMap<String, ConversationTrace>,
}

impl ReplayDriver {
    pub fn new() -> Self {
        Self {
            traces: HashMap::new(),
        }
    }

    /// Register a trace for a specific case ID.
    pub fn add_trace(&mut self, case_id: impl Into<String>, trace: ConversationTrace) {
        self.traces.insert(case_id.into(), trace);
    }

    /// Get the stored trace for a case.
    pub fn get_trace(&self, case_id: &str) -> Option<&ConversationTrace> {
        self.traces.get(case_id)
    }
}

impl Default for ReplayDriver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fastclaw_core::types::{
        ChatMessage, Role, TraceLlmRequest, TraceLlmResponse, TraceToolCall, TraceTurn,
    };

    fn msg(role: Role, text: &str) -> ChatMessage {
        ChatMessage {
            role,
            content: Some(serde_json::Value::String(text.to_string())),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }

    fn turn(idx: u32, user: &str, assistant: &str, tools: Vec<&str>) -> TraceTurn {
        TraceTurn {
            turn_index: idx,
            user_message: msg(Role::User, user),
            assistant_message: msg(Role::Assistant, assistant),
            tool_calls: tools
                .into_iter()
                .map(|name| TraceToolCall {
                    tool_name: name.to_string(),
                    call_id: "c1".into(),
                    arguments: serde_json::json!({}),
                    output: "ok".into(),
                    success: true,
                    latency_ms: 10,
                })
                .collect(),
            llm_request: TraceLlmRequest {
                model: "test".into(),
                message_count: 1,
                estimated_tokens: 100,
            },
            llm_response: TraceLlmResponse {
                model: "test".into(),
                usage: None,
                finish_reason: Some("stop".into()),
                latency_ms: 50,
            },
            context_tokens: 100,
            latency_ms: 60,
            compaction_applied: false,
        }
    }

    fn trace(id: &str, turns: Vec<TraceTurn>) -> ConversationTrace {
        ConversationTrace {
            trace_id: id.into(),
            session_id: "s1".into(),
            agent_id: "a1".into(),
            model: "test".into(),
            context_window: None,
            started_at: "2026-01-01T00:00:00Z".into(),
            finished_at: None,
            turns,
            metadata: serde_json::Map::new(),
        }
    }

    fn case(id: &str) -> EvalCase {
        EvalCase {
            id: id.into(),
            category: "test".into(),
            description: "test case".into(),
            user_messages: vec!["hi".into()],
            expected_behaviors: vec![],
            max_turns: 5,
            timeout_secs: 30,
            agent_id: None,
        }
    }

    #[test]
    fn add_and_get_trace() {
        let mut driver = ReplayDriver::new();
        let t = trace("tr-1", vec![turn(0, "hi", "hello", vec![])]);
        driver.add_trace("case-1", t);
        assert!(driver.get_trace("case-1").is_some());
    }

    #[test]
    fn get_missing_trace() {
        let driver = ReplayDriver::new();
        assert!(driver.get_trace("nope").is_none());
    }

    #[tokio::test]
    async fn run_case_replays_last_turn() {
        let mut driver = ReplayDriver::new();
        let t = trace(
            "tr-2",
            vec![
                turn(0, "q1", "a1", vec!["tool_x"]),
                turn(1, "q2", "a2", vec!["tool_y"]),
            ],
        );
        driver.add_trace("c2", t);
        let artifacts = driver.run_case(&case("c2")).await.unwrap();
        assert_eq!(artifacts.total_turns, 2);
        assert_eq!(artifacts.tool_calls_made, vec!["tool_x", "tool_y"]);
        assert_eq!(artifacts.final_response.as_deref(), Some("a2"));
    }

    #[tokio::test]
    async fn run_case_missing_trace() {
        let driver = ReplayDriver::new();
        let result = driver.run_case(&case("missing")).await;
        assert!(result.is_err());
    }

    #[test]
    fn compare_fully_matches() {
        let t = trace("tr-3", vec![turn(0, "q", "resp", vec!["tool_a"])]);
        let artifacts = EvalRunArtifacts {
            tool_calls_made: vec!["tool_a".into()],
            total_turns: 1,
            final_response: Some("resp".into()),
            latency_ms: 0,
            total_tokens: 0,
        };
        let result = ReplayResult::compare(&t, &artifacts);
        assert!(result.fully_matches);
    }

    #[test]
    fn compare_detects_tool_mismatch() {
        let t = trace("tr-4", vec![turn(0, "q", "resp", vec!["tool_a"])]);
        let artifacts = EvalRunArtifacts {
            tool_calls_made: vec!["tool_b".into()],
            total_turns: 1,
            final_response: Some("resp".into()),
            latency_ms: 0,
            total_tokens: 0,
        };
        let result = ReplayResult::compare(&t, &artifacts);
        assert!(!result.fully_matches);
        assert!(!result.turn_diffs[0].tools_match);
    }

    #[test]
    fn compare_detects_response_mismatch() {
        let t = trace("tr-5", vec![turn(0, "q", "resp-old", vec![])]);
        let artifacts = EvalRunArtifacts {
            tool_calls_made: vec![],
            total_turns: 1,
            final_response: Some("resp-new".into()),
            latency_ms: 0,
            total_tokens: 0,
        };
        let result = ReplayResult::compare(&t, &artifacts);
        assert!(!result.fully_matches);
        assert!(!result.turn_diffs[0].response_match);
    }

    #[test]
    fn compare_empty_turns() {
        let t = trace("tr-6", vec![]);
        let artifacts = EvalRunArtifacts {
            tool_calls_made: vec![],
            total_turns: 0,
            final_response: None,
            latency_ms: 0,
            total_tokens: 0,
        };
        let result = ReplayResult::compare(&t, &artifacts);
        assert!(result.fully_matches);
        assert!(result.turn_diffs.is_empty());
    }
}

#[async_trait]
impl EvalAgentDriver for ReplayDriver {
    async fn run_case(&self, case: &EvalCase) -> anyhow::Result<EvalRunArtifacts> {
        let trace = self
            .traces
            .get(&case.id)
            .ok_or_else(|| anyhow::anyhow!("ReplayDriver: no trace for case {:?}", case.id))?;

        let mut tool_calls = Vec::new();
        let mut final_response = None;
        let mut total_tokens = 0u64;
        let mut total_latency = 0u64;

        for turn in &trace.turns {
            for tc in &turn.tool_calls {
                tool_calls.push(tc.tool_name.clone());
            }
            if let Some(text) = turn.assistant_message.text_content() {
                final_response = Some(text.to_string());
            }
            if let Some(ref usage) = turn.llm_response.usage {
                total_tokens += usage.total_tokens as u64;
            }
            total_latency += turn.latency_ms;
        }

        Ok(EvalRunArtifacts {
            tool_calls_made: tool_calls,
            total_turns: trace.turns.len() as u32,
            final_response,
            latency_ms: total_latency,
            total_tokens,
        })
    }
}
