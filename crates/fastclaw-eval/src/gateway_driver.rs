//! Gateway eval driver: runs eval cases against a live FastClaw gateway via HTTP.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{EvalAgentDriver, EvalCase, EvalRunArtifacts};

/// Drives eval cases by sending HTTP requests to a running FastClaw gateway.
pub struct GatewayEvalDriver {
    base_url: String,
    api_key: Option<String>,
    client: reqwest::Client,
}

impl GatewayEvalDriver {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            api_key: None,
            client: reqwest::Client::new(),
        }
    }

    pub fn with_api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }
}

#[derive(Serialize)]
struct ChatPayload {
    messages: Vec<ChatMsg>,
    #[serde(skip_serializing_if = "Option::is_none")]
    agent_id: Option<String>,
    stream: bool,
}

#[derive(Serialize)]
struct ChatMsg {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResp {
    choices: Option<Vec<ChatChoice>>,
    usage: Option<UsageResp>,
    #[serde(default)]
    _meta: Option<MetaResp>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatChoiceMessage,
}

#[derive(Deserialize)]
struct ChatChoiceMessage {
    content: Option<String>,
    #[serde(default)]
    tool_calls: Option<Vec<ToolCallResp>>,
}

#[derive(Deserialize)]
struct ToolCallResp {
    function: ToolCallFunction,
}

#[derive(Deserialize)]
struct ToolCallFunction {
    name: String,
}

#[derive(Deserialize)]
struct UsageResp {
    #[serde(default)]
    total_tokens: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct MetaResp {
    #[serde(default)]
    tool_calls_made: u32,
    #[serde(default)]
    iterations: u32,
}

#[async_trait]
impl EvalAgentDriver for GatewayEvalDriver {
    async fn run_case(&self, case: &EvalCase) -> anyhow::Result<EvalRunArtifacts> {
        let url = format!("{}/api/v1/chat", self.base_url.trim_end_matches('/'));

        let messages: Vec<ChatMsg> = case
            .user_messages
            .iter()
            .map(|m| ChatMsg {
                role: "user".into(),
                content: m.clone(),
            })
            .collect();

        let payload = ChatPayload {
            messages,
            agent_id: case.agent_id.clone(),
            stream: false,
        };

        let start = std::time::Instant::now();

        let mut req = self.client.post(&url).json(&payload);
        if let Some(ref key) = self.api_key {
            req = req.header("Authorization", format!("Bearer {key}"));
        }

        let resp = req.send().await?.error_for_status()?;
        let elapsed = start.elapsed().as_millis() as u64;

        let body: ChatResp = resp.json().await?;

        let mut tool_calls = Vec::new();
        let mut final_response = None;

        if let Some(choices) = &body.choices {
            if let Some(choice) = choices.first() {
                final_response = choice.message.content.clone();
                if let Some(tcs) = &choice.message.tool_calls {
                    for tc in tcs {
                        tool_calls.push(tc.function.name.clone());
                    }
                }
            }
        }

        let total_tokens = body.usage.map(|u| u.total_tokens).unwrap_or(0);
        let total_turns = body
            ._meta
            .as_ref()
            .map(|m| m.iterations)
            .unwrap_or(1);

        Ok(EvalRunArtifacts {
            tool_calls_made: tool_calls,
            total_turns,
            final_response,
            latency_ms: elapsed,
            total_tokens,
        })
    }
}
