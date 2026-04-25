use std::sync::Arc;

use fastclaw_core::types::{ChatMessage, Role};

use crate::llm::{CompletionParams, LlmProvider};

/// Fraction of context window at which LLM compression triggers.
pub const COMPRESSION_THRESHOLD: f32 = 0.70;

/// Fraction of recent history to preserve (the rest gets compressed).
const PRESERVE_FRACTION: f32 = 0.30;

/// Minimum fraction of history that must be compressible to justify an LLM call.
const MIN_COMPRESSIBLE_FRACTION: f32 = 0.05;

const COMPRESSION_SYSTEM_PROMPT: &str = r#"You are a conversation compression engine. Distill the provided conversation history into a structured state snapshot. This snapshot will become the agent's ONLY memory of the past. Preserve ALL critical details.

Generate a <state_snapshot> containing:

<state_snapshot>
<overall_goal>
    <!-- One sentence: the user's high-level objective -->
</overall_goal>

<key_knowledge>
    <!-- Crucial facts, constraints, conventions. Bullet points. -->
</key_knowledge>

<file_system_state>
    <!-- Files created/read/modified/deleted. Status and key findings. -->
</file_system_state>

<recent_actions>
    <!-- Last significant agent actions and outcomes. -->
</recent_actions>

<current_plan>
    <!-- Step-by-step plan. Mark [DONE] / [IN PROGRESS] / [TODO]. -->
</current_plan>
</state_snapshot>

Be extremely dense with information. Omit conversational filler."#;

pub struct CompressionResult {
    pub compressed: bool,
    pub original_tokens: usize,
    pub new_tokens: usize,
    pub messages: Vec<ChatMessage>,
}

/// Find the split point: preserve the last `preserve_fraction` of non-system messages.
/// Split must land on a user message boundary.
fn find_split_point(non_system: &[&ChatMessage], preserve_fraction: f32) -> usize {
    if non_system.is_empty() {
        return 0;
    }

    let char_counts: Vec<usize> = non_system.iter().map(|m| {
        m.content.as_ref().map_or(0, |c| {
            serde_json::to_string(c).map(|s| s.len()).unwrap_or(0)
        }) + m.tool_calls.as_ref().map_or(0, |tc| {
            tc.iter().map(|t| t.function.name.len() + t.function.arguments.len()).sum()
        })
    }).collect();

    let total_chars: usize = char_counts.iter().sum();
    let target_chars = (total_chars as f32 * (1.0 - preserve_fraction)) as usize;

    let mut cumulative = 0usize;
    let mut last_user_boundary = 0usize;

    for (i, msg) in non_system.iter().enumerate() {
        if matches!(msg.role, Role::User) && !has_tool_response(msg) {
            if cumulative >= target_chars {
                return i;
            }
            last_user_boundary = i;
        }
        cumulative += char_counts[i];
    }

    last_user_boundary
}

fn has_tool_response(msg: &ChatMessage) -> bool {
    msg.tool_call_id.is_some()
}

/// Attempt LLM-based compression of conversation history.
///
/// Triggers when estimated tokens exceed `COMPRESSION_THRESHOLD * context_window`.
/// Calls the LLM with a compression prompt to generate a state snapshot,
/// then replaces the compressed portion with the snapshot.
pub async fn try_compress_chat(
    messages: &mut Vec<ChatMessage>,
    context_window: u32,
    provider: &Arc<dyn LlmProvider>,
    model: &str,
) -> CompressionResult {
    let estimated = fastclaw_context::estimate_messages_tokens(messages);
    let threshold = (context_window as f32 * COMPRESSION_THRESHOLD) as usize;

    if estimated <= threshold {
        return CompressionResult {
            compressed: false,
            original_tokens: estimated,
            new_tokens: estimated,
            messages: messages.clone(),
        };
    }

    tracing::info!(
        estimated,
        threshold,
        context_window,
        "context compression triggered"
    );

    let mut system_indices: Vec<usize> = Vec::new();
    let mut non_system_indices: Vec<usize> = Vec::new();
    for (i, m) in messages.iter().enumerate() {
        if matches!(m.role, Role::System) {
            system_indices.push(i);
        } else {
            non_system_indices.push(i);
        }
    }

    let non_system_msgs: Vec<&ChatMessage> = non_system_indices.iter().map(|&i| &messages[i]).collect();

    if non_system_msgs.is_empty() {
        return CompressionResult {
            compressed: false,
            original_tokens: estimated,
            new_tokens: estimated,
            messages: messages.clone(),
        };
    }

    let split = find_split_point(&non_system_msgs, PRESERVE_FRACTION);
    if split == 0 {
        return CompressionResult {
            compressed: false,
            original_tokens: estimated,
            new_tokens: estimated,
            messages: messages.clone(),
        };
    }

    let to_compress = &non_system_msgs[..split];
    let to_keep = &non_system_msgs[split..];

    let compress_chars: usize = to_compress.iter().map(|m| {
        m.content.as_ref().map_or(0, |c| serde_json::to_string(c).map(|s| s.len()).unwrap_or(0))
    }).sum();
    let total_chars: usize = non_system_msgs.iter().map(|m| {
        m.content.as_ref().map_or(0, |c| serde_json::to_string(c).map(|s| s.len()).unwrap_or(0))
    }).sum();

    if total_chars > 0 && (compress_chars as f32 / total_chars as f32) < MIN_COMPRESSIBLE_FRACTION {
        tracing::info!("compressible fraction too small, skipping LLM compression");
        return CompressionResult {
            compressed: false,
            original_tokens: estimated,
            new_tokens: estimated,
            messages: messages.clone(),
        };
    }

    // Build the LLM compression request
    let mut compress_messages: Vec<ChatMessage> = Vec::new();
    compress_messages.push(ChatMessage {
        role: Role::System,
        content: Some(serde_json::Value::String(COMPRESSION_SYSTEM_PROMPT.to_string())),
        name: None,
        tool_calls: None,
        tool_call_id: None,
    });
    for msg in to_compress {
        compress_messages.push((*msg).clone());
    }
    compress_messages.push(ChatMessage {
        role: Role::User,
        content: Some(serde_json::Value::String(
            "First, reason in your scratchpad. Then, generate the <state_snapshot>.".to_string(),
        )),
        name: None,
        tool_calls: None,
        tool_call_id: None,
    });

    let params = CompletionParams {
        model,
        messages: &compress_messages,
        temperature: 0.0,
        max_tokens: Some(4096),
        tools: None,
    };

    let summary = match provider.chat_completion(&params).await {
        Ok(resp) => {
            resp.choices.first().and_then(|c| c.message.text_content()).unwrap_or_default()
        }
        Err(e) => {
            tracing::warn!(error = %e, "LLM compression failed, falling back to rule-based");
            return CompressionResult {
                compressed: false,
                original_tokens: estimated,
                new_tokens: estimated,
                messages: messages.clone(),
            };
        }
    };

    if summary.trim().is_empty() {
        tracing::warn!("LLM compression returned empty summary");
        return CompressionResult {
            compressed: false,
            original_tokens: estimated,
            new_tokens: estimated,
            messages: messages.clone(),
        };
    }

    // Rebuild messages: system msgs + summary as user/assistant pair + kept history
    let mut new_messages: Vec<ChatMessage> = Vec::new();

    for &idx in &system_indices {
        let msg: ChatMessage = messages[idx].clone();
        new_messages.push(msg);
    }

    new_messages.push(ChatMessage {
        role: Role::User,
        content: Some(serde_json::Value::String(summary)),
        name: None,
        tool_calls: None,
        tool_call_id: None,
    });
    new_messages.push(ChatMessage {
        role: Role::Assistant,
        content: Some(serde_json::Value::String(
            "Got it. I have the full context from the previous conversation. Let me continue.".to_string(),
        )),
        name: None,
        tool_calls: None,
        tool_call_id: None,
    });

    for msg in to_keep {
        new_messages.push((*msg).clone());
    }

    let new_estimated = fastclaw_context::estimate_messages_tokens(&new_messages);

    if new_estimated >= estimated {
        tracing::warn!(
            new_estimated,
            original = estimated,
            "compression inflated tokens, discarding"
        );
        return CompressionResult {
            compressed: false,
            original_tokens: estimated,
            new_tokens: estimated,
            messages: messages.clone(),
        };
    }

    tracing::info!(
        original_tokens = estimated,
        new_tokens = new_estimated,
        evicted_messages = to_compress.len(),
        kept_messages = to_keep.len(),
        "LLM compression successful"
    );

    *messages = new_messages.clone();

    CompressionResult {
        compressed: true,
        original_tokens: estimated,
        new_tokens: new_estimated,
        messages: new_messages,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn user(text: &str) -> ChatMessage {
        ChatMessage {
            role: Role::User,
            content: Some(text.to_string().into()),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }

    fn asst(text: &str) -> ChatMessage {
        ChatMessage {
            role: Role::Assistant,
            content: Some(text.to_string().into()),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }

    #[test]
    fn split_point_preserves_recent_fraction() {
        let msgs = vec![
            user("old question 1"),
            asst("old answer 1"),
            user("old question 2"),
            asst("old answer 2"),
            user("recent question"),
            asst("recent answer"),
        ];
        let refs: Vec<&ChatMessage> = msgs.iter().collect();
        let split = find_split_point(&refs, 0.3);
        assert!(split > 0, "should split somewhere");
        assert!(split < msgs.len(), "should keep some messages");
    }

    #[test]
    fn split_point_empty_returns_zero() {
        let msgs: Vec<&ChatMessage> = vec![];
        assert_eq!(find_split_point(&msgs, 0.3), 0);
    }
}
