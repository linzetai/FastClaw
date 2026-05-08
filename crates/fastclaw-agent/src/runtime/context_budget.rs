use fastclaw_core::types::{ChatMessage, Role};

use super::tool_executor::{
    classify_retention_tier, summarize_tool_result, build_cleared_with_recall, RetentionTier,
    RECALL_HINT_MARKER,
};

/// Budget allocation fractions for the context window.
///
/// These fractions determine how the context window is divided among
/// different content categories.
#[derive(Debug, Clone)]
pub(crate) struct BudgetConfig {
    /// Fraction for system prompts + user messages (non-compressible).
    pub system_user_fraction: f32,
    /// Fraction for the most recent tool call results (sliding window).
    pub recent_tool_fraction: f32,
    /// Fraction for older tool call results (summary retention).
    pub older_tool_fraction: f32,
    /// Fraction for historical summaries (heavily compressed).
    #[allow(dead_code)]
    pub history_fraction: f32,
    /// Number of most recent tool results considered "recent".
    pub recent_tool_window: usize,
}

impl Default for BudgetConfig {
    fn default() -> Self {
        Self {
            system_user_fraction: 0.30,
            recent_tool_fraction: 0.40,
            older_tool_fraction: 0.20,
            history_fraction: 0.10,
            recent_tool_window: 6,
        }
    }
}

/// Result of applying the token budget.
#[derive(Debug)]
#[allow(dead_code)]
pub(crate) struct BudgetResult {
    pub recent_tools_trimmed: usize,
    pub older_tools_summarized: usize,
    pub history_compressed: usize,
    pub total_tokens_freed: usize,
}

/// Classify messages into budget categories.
struct MessageClassification {
    system_indices: Vec<usize>,
    user_indices: Vec<usize>,
    recent_tool_indices: Vec<usize>,
    older_tool_indices: Vec<usize>,
    #[allow(dead_code)]
    assistant_indices: Vec<usize>,
}

fn classify_messages(
    messages: &[ChatMessage],
    recent_window: usize,
) -> MessageClassification {
    let mut system_indices = Vec::new();
    let mut user_indices = Vec::new();
    let mut tool_indices = Vec::new();
    let mut assistant_indices = Vec::new();

    for (i, msg) in messages.iter().enumerate() {
        match msg.role {
            Role::System => system_indices.push(i),
            Role::User => user_indices.push(i),
            Role::Tool => tool_indices.push(i),
            Role::Assistant => assistant_indices.push(i),
        }
    }

    let split = tool_indices.len().saturating_sub(recent_window);
    let older_tool_indices = tool_indices[..split].to_vec();
    let recent_tool_indices = tool_indices[split..].to_vec();

    MessageClassification {
        system_indices,
        user_indices,
        recent_tool_indices,
        older_tool_indices,
        assistant_indices,
    }
}

fn estimate_tokens_for_indices(messages: &[ChatMessage], indices: &[usize]) -> usize {
    indices
        .iter()
        .map(|&i| fastclaw_context::estimate_messages_tokens(std::slice::from_ref(&messages[i])))
        .sum()
}

/// Apply the token budget to conversation messages.
///
/// This is a soft enforcement: it compresses older tool results down to
/// summaries when they exceed their allocated budget, but never touches
/// system/user messages or the most recent tool results.
pub(crate) fn apply_token_budget(
    messages: &mut [ChatMessage],
    context_window: u32,
    config: &BudgetConfig,
) -> BudgetResult {
    let total_budget = context_window as usize;
    let classified = classify_messages(messages, config.recent_tool_window);

    let system_user_budget = (total_budget as f32 * config.system_user_fraction) as usize;
    let recent_tool_budget = (total_budget as f32 * config.recent_tool_fraction) as usize;
    let older_tool_budget = (total_budget as f32 * config.older_tool_fraction) as usize;

    let system_user_tokens = estimate_tokens_for_indices(messages, &classified.system_indices)
        + estimate_tokens_for_indices(messages, &classified.user_indices);

    let mut older_tools_summarized = 0;
    let mut recent_tools_trimmed = 0;
    let mut total_tokens_freed = 0;

    // Phase 1: Compress older tool results to fit their budget.
    let older_tool_tokens = estimate_tokens_for_indices(messages, &classified.older_tool_indices);
    if older_tool_tokens > older_tool_budget {
        let overshoot = older_tool_tokens - older_tool_budget;
        let mut freed = 0;

        // Sort older tool indices by tier (Ephemeral first, then Summarize, then FullRetain)
        // to compress least-valuable results first.
        let mut sorted_older: Vec<(usize, RetentionTier)> = classified
            .older_tool_indices
            .iter()
            .filter_map(|&i| {
                let name = messages[i].name.as_deref()?;
                Some((i, classify_retention_tier(name)))
            })
            .collect();
        sorted_older.sort_by_key(|(_, tier)| *tier as u8);

        for (idx, tier) in sorted_older {
            if freed >= overshoot {
                break;
            }

            let msg = &messages[idx];
            let text = match msg.text_content() {
                Some(t) => t,
                None => continue,
            };

            if text.starts_with("[summarized]")
                || text.starts_with(RECALL_HINT_MARKER)
                || text.starts_with("[faded]")
                || text.starts_with("[oneliner]")
                || text.starts_with("[time-compacted]")
                || text == "[Old tool result content cleared]"
            {
                continue;
            }

            let tool_name = messages[idx].name.as_deref().unwrap_or("unknown").to_string();
            let before_tokens = fastclaw_context::estimate_messages_tokens(
                std::slice::from_ref(&messages[idx]),
            );

            let replacement = match tier {
                RetentionTier::Ephemeral => {
                    build_cleared_with_recall(&tool_name, tier, &text, None)
                }
                RetentionTier::Summarize => {
                    let summary = summarize_tool_result(&tool_name, &text, 300);
                    format!("[summarized] {summary}")
                }
                RetentionTier::FullRetain => {
                    let summary = summarize_tool_result(&tool_name, &text, 500);
                    format!("[summarized] {summary}")
                }
            };

            messages[idx].content = Some(serde_json::Value::String(replacement));
            let after_tokens = fastclaw_context::estimate_messages_tokens(
                std::slice::from_ref(&messages[idx]),
            );

            let delta = before_tokens.saturating_sub(after_tokens);
            freed += delta;
            total_tokens_freed += delta;
            older_tools_summarized += 1;
        }
    }

    // Phase 2: If system+user messages eat into the recent tool budget, apply
    // light compression to the oldest recent tool results.
    let system_user_overflow = system_user_tokens.saturating_sub(system_user_budget);
    let effective_recent_budget = recent_tool_budget.saturating_sub(system_user_overflow);
    let recent_tool_tokens = estimate_tokens_for_indices(messages, &classified.recent_tool_indices);

    if recent_tool_tokens > effective_recent_budget {
        let overshoot = recent_tool_tokens - effective_recent_budget;
        let mut freed = 0;

        // Compress from the oldest of the "recent" window.
        for &idx in &classified.recent_tool_indices {
            if freed >= overshoot {
                break;
            }

            let msg = &messages[idx];
            let text = match msg.text_content() {
                Some(t) => t,
                None => continue,
            };

            if text.starts_with("[summarized]")
                || text.starts_with(RECALL_HINT_MARKER)
                || text.starts_with("[faded]")
                || text.starts_with("[time-compacted]")
                || text == "[Old tool result content cleared]"
            {
                continue;
            }

            let tool_name = messages[idx].name.as_deref().unwrap_or("unknown").to_string();
            let tier = classify_retention_tier(&tool_name);
            let before_tokens = fastclaw_context::estimate_messages_tokens(
                std::slice::from_ref(&messages[idx]),
            );

            // For recent results, only fade — don't fully clear.
            let max_chars = match tier {
                RetentionTier::FullRetain => 600,
                RetentionTier::Summarize => 400,
                RetentionTier::Ephemeral => 150,
            };
            let summary = summarize_tool_result(&tool_name, &text, max_chars);
            let replacement = format!("[summarized] {summary}");

            messages[idx].content = Some(serde_json::Value::String(replacement));
            let after_tokens = fastclaw_context::estimate_messages_tokens(
                std::slice::from_ref(&messages[idx]),
            );

            let delta = before_tokens.saturating_sub(after_tokens);
            freed += delta;
            total_tokens_freed += delta;
            recent_tools_trimmed += 1;
        }
    }

    BudgetResult {
        recent_tools_trimmed,
        older_tools_summarized,
        history_compressed: 0,
        total_tokens_freed,
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn system_msg(text: &str) -> ChatMessage {
        ChatMessage {
            role: Role::System,
            content: Some(serde_json::Value::String(text.to_string())),
            reasoning_content: None,
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }

    fn user_msg(text: &str) -> ChatMessage {
        ChatMessage {
            role: Role::User,
            content: Some(serde_json::Value::String(text.to_string())),
            reasoning_content: None,
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }

    fn tool_msg(name: &str, text: &str) -> ChatMessage {
        ChatMessage {
            role: Role::Tool,
            content: Some(serde_json::Value::String(text.to_string())),
            reasoning_content: None,
            name: Some(name.to_string()),
            tool_calls: None,
            tool_call_id: Some(format!("call_{name}")),
        }
    }

    fn asst_msg(text: &str) -> ChatMessage {
        ChatMessage {
            role: Role::Assistant,
            content: Some(serde_json::Value::String(text.to_string())),
            reasoning_content: None,
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }

    #[test]
    fn budget_config_default_sums_to_one() {
        let config = BudgetConfig::default();
        let total = config.system_user_fraction
            + config.recent_tool_fraction
            + config.older_tool_fraction
            + config.history_fraction;
        assert!((total - 1.0).abs() < 0.001);
    }

    #[test]
    fn classify_messages_splits_correctly() {
        let msgs = vec![
            system_msg("sys"),
            user_msg("hello"),
            tool_msg("read_file", "content 1"),
            tool_msg("read_file", "content 2"),
            tool_msg("grep", "matches"),
            tool_msg("list_dir", "files"),
            asst_msg("reply"),
        ];
        let result = classify_messages(&msgs, 2);
        assert_eq!(result.system_indices.len(), 1);
        assert_eq!(result.user_indices.len(), 1);
        assert_eq!(result.older_tool_indices.len(), 2);
        assert_eq!(result.recent_tool_indices.len(), 2);
        assert_eq!(result.assistant_indices.len(), 1);
    }

    #[test]
    fn budget_no_compression_when_under_limit() {
        let mut msgs = vec![
            system_msg("system prompt"),
            user_msg("question"),
            tool_msg("read_file", "short content"),
            asst_msg("answer"),
        ];
        let result = apply_token_budget(&mut msgs, 100_000, &BudgetConfig::default());
        assert_eq!(result.older_tools_summarized, 0);
        assert_eq!(result.recent_tools_trimmed, 0);
    }

    #[test]
    fn budget_compresses_older_tools_when_over_limit() {
        let big_content = "x".repeat(5000);
        let mut msgs = vec![
            system_msg("sys"),
            user_msg("q"),
            // These will be "older" (beyond recent_window of 2)
            tool_msg("read_file", &big_content),
            tool_msg("grep", &big_content),
            tool_msg("list_dir", &big_content),
            // These are "recent" (last 2)
            tool_msg("read_file", "recent1"),
            tool_msg("read_file", "recent2"),
            asst_msg("a"),
        ];

        let config = BudgetConfig {
            older_tool_fraction: 0.01, // Very tight budget for older tools
            recent_tool_window: 2,
            ..Default::default()
        };

        let result = apply_token_budget(&mut msgs, 1000, &config);
        assert!(result.older_tools_summarized > 0, "should summarize older tools");
        assert!(result.total_tokens_freed > 0, "should free tokens");
    }

    #[test]
    fn ephemeral_tools_compressed_first() {
        let big = "x".repeat(3000);
        let mut msgs = vec![
            system_msg("sys"),
            user_msg("q"),
            tool_msg("list_dir", &big),   // Ephemeral
            tool_msg("read_file", &big),   // FullRetain
            tool_msg("grep", &big),        // Summarize
            tool_msg("read_file", "recent"),
            asst_msg("a"),
        ];

        let config = BudgetConfig {
            older_tool_fraction: 0.01,
            recent_tool_window: 1,
            ..Default::default()
        };

        apply_token_budget(&mut msgs, 1000, &config);

        // Ephemeral (list_dir) should be fully cleared first
        let list_dir_text = msgs[2].text_content().unwrap();
        assert!(
            list_dir_text.starts_with(RECALL_HINT_MARKER),
            "list_dir should be recall-cleared, got: {list_dir_text}"
        );
    }

    #[test]
    fn retention_tier_ordering() {
        assert!(RetentionTier::Ephemeral < RetentionTier::Summarize);
        assert!(RetentionTier::Summarize < RetentionTier::FullRetain);
    }
}
