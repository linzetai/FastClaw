use fastclaw_core::types::{ChatMessage, Role};

/// A contiguous group of messages that form one "API round" — typically a user
/// turn followed by an assistant reply, possibly with interleaved tool calls.
/// System messages at the start of the conversation are folded into the first
/// round rather than forming their own group.
#[derive(Debug, Clone)]
pub struct ApiRound {
    /// Index of this round (0-based, in chronological order).
    pub index: usize,
    /// The messages belonging to this round. Borrows from the source slice.
    pub messages: Vec<ChatMessage>,
    /// Estimated token count for all messages in this round.
    pub estimated_tokens: usize,
}

/// Partition a flat message list into logical API rounds.
///
/// A new round boundary is placed **before** each `Role::Assistant` message
/// whose preceding context does not already belong to the same assistant turn
/// (i.e. each top-level assistant reply starts a new round). Tool messages
/// that follow an assistant message with `tool_calls` stay in the same round.
///
/// `Role::System` messages at the very beginning are attached to the first
/// conversational round instead of creating a standalone group.
pub fn group_by_api_round(messages: &[ChatMessage]) -> Vec<ApiRound> {
    if messages.is_empty() {
        return Vec::new();
    }

    let mut rounds: Vec<Vec<ChatMessage>> = Vec::new();
    let mut current: Vec<ChatMessage> = Vec::new();
    let mut seen_non_system = false;

    for msg in messages {
        match msg.role {
            Role::System => {
                // System messages always attach to the current (or first) group.
                current.push(msg.clone());
            }
            Role::User => {
                if seen_non_system && !current.is_empty() {
                    // A new user turn after we already have content means we
                    // should check whether the previous round is "complete"
                    // (has an assistant reply). If it does, start a new round.
                    let has_assistant = current.iter().any(|m| m.role == Role::Assistant);
                    if has_assistant {
                        rounds.push(std::mem::take(&mut current));
                    }
                }
                seen_non_system = true;
                current.push(msg.clone());
            }
            Role::Assistant => {
                seen_non_system = true;
                current.push(msg.clone());
            }
            Role::Tool => {
                // Tool results stay with the current round (the assistant
                // message that triggered them).
                current.push(msg.clone());
            }
        }
    }

    if !current.is_empty() {
        rounds.push(current);
    }

    rounds
        .into_iter()
        .enumerate()
        .map(|(i, msgs)| {
            let estimated_tokens = super::estimate_messages_tokens(&msgs);
            ApiRound {
                index: i,
                messages: msgs,
                estimated_tokens,
            }
        })
        .collect()
}

/// Configuration for the snip compactor.
pub struct SnipCompactorConfig {
    /// Maximum token budget. If total tokens exceed this, oldest rounds are removed.
    pub max_tokens: usize,
    /// Minimum number of recent rounds to keep, even if over budget.
    pub min_rounds_to_keep: usize,
}

/// Result of a snip compaction pass.
#[derive(Debug)]
pub struct SnipResult {
    /// The surviving messages after compaction (flattened from kept rounds).
    pub messages: Vec<ChatMessage>,
    /// Number of tokens freed by removing rounds.
    pub tokens_freed: usize,
    /// Number of rounds removed.
    pub rounds_removed: usize,
    /// Whether any compaction actually happened.
    pub compacted: bool,
}

/// Snip compactor: removes entire API rounds from oldest-first when the
/// conversation exceeds the token budget.
pub struct SnipCompactor {
    config: SnipCompactorConfig,
}

impl SnipCompactor {
    pub fn new(config: SnipCompactorConfig) -> Self {
        Self { config }
    }

    /// Run snip compaction. Returns a no-op result if tokens are within budget.
    pub fn compact(&self, messages: &[ChatMessage]) -> SnipResult {
        let total_tokens = super::estimate_messages_tokens(messages);

        if total_tokens <= self.config.max_tokens {
            return SnipResult {
                messages: messages.to_vec(),
                tokens_freed: 0,
                rounds_removed: 0,
                compacted: false,
            };
        }

        let rounds = group_by_api_round(messages);
        if rounds.is_empty() {
            return SnipResult {
                messages: Vec::new(),
                tokens_freed: 0,
                rounds_removed: 0,
                compacted: false,
            };
        }

        let n = rounds.len();
        let protected_start = n.saturating_sub(self.config.min_rounds_to_keep);

        let mut tokens_freed = 0usize;
        let mut current_tokens = total_tokens;
        let mut remove_set = vec![false; n];

        // Walk from oldest round forward, skipping protected recent rounds.
        for (i, round) in rounds.iter().enumerate() {
            if current_tokens <= self.config.max_tokens {
                break;
            }
            if i >= protected_start {
                break;
            }
            if round_contains_error(round) {
                continue;
            }
            remove_set[i] = true;
            tokens_freed += round.estimated_tokens;
            current_tokens = current_tokens.saturating_sub(round.estimated_tokens);
        }

        let rounds_removed = remove_set.iter().filter(|&&r| r).count();
        if rounds_removed == 0 {
            return SnipResult {
                messages: messages.to_vec(),
                tokens_freed: 0,
                rounds_removed: 0,
                compacted: false,
            };
        }

        let kept_messages: Vec<ChatMessage> = rounds
            .into_iter()
            .enumerate()
            .filter(|(i, _)| !remove_set[*i])
            .flat_map(|(_, r)| r.messages)
            .collect();

        SnipResult {
            messages: kept_messages,
            tokens_freed,
            rounds_removed,
            compacted: true,
        }
    }
}

/// Heuristic: a round "contains an error" if any tool result message includes
/// a JSON object with an `"error"` or `"is_error"` key set to a truthy value,
/// or if any message text contains a `[ERROR]` / `Error:` marker.
fn round_contains_error(round: &ApiRound) -> bool {
    for msg in &round.messages {
        if msg.role != Role::Tool {
            continue;
        }
        if let Some(content) = &msg.content {
            // Check for structured error indicators.
            if let Some(obj) = content.as_object() {
                if obj.get("error").is_some() || obj.get("is_error").is_some() {
                    return true;
                }
            }
            // Check for text markers.
            let text = match content.as_str() {
                Some(s) => s.to_string(),
                None => content.to_string(),
            };
            if text.contains("[ERROR]") || text.contains("\"is_error\":true") {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use fastclaw_core::types::ChatMessage;
    use serde_json::json;

    fn sys(text: &str) -> ChatMessage {
        ChatMessage {
            role: Role::System,
            content: Some(json!(text)),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }

    fn user(text: &str) -> ChatMessage {
        ChatMessage {
            role: Role::User,
            content: Some(json!(text)),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }

    fn assistant(text: &str) -> ChatMessage {
        ChatMessage {
            role: Role::Assistant,
            content: Some(json!(text)),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }

    fn tool(text: &str) -> ChatMessage {
        ChatMessage {
            role: Role::Tool,
            content: Some(json!(text)),
            name: None,
            tool_calls: None,
            tool_call_id: Some("call_1".into()),
        }
    }

    #[test]
    fn empty_messages_returns_empty_groups() {
        let rounds = group_by_api_round(&[]);
        assert!(rounds.is_empty());
    }

    #[test]
    fn single_round_user_assistant() {
        let msgs = vec![user("hello"), assistant("hi there")];
        let rounds = group_by_api_round(&msgs);
        assert_eq!(rounds.len(), 1);
        assert_eq!(rounds[0].messages.len(), 2);
        assert_eq!(rounds[0].index, 0);
    }

    #[test]
    fn system_message_folds_into_first_round() {
        let msgs = vec![
            sys("You are helpful"),
            user("hello"),
            assistant("hi"),
        ];
        let rounds = group_by_api_round(&msgs);
        assert_eq!(rounds.len(), 1, "system + 1 turn = 1 round");
        assert_eq!(rounds[0].messages.len(), 3);
        assert_eq!(rounds[0].messages[0].role, Role::System);
    }

    #[test]
    fn multiple_system_messages_fold_into_first_round() {
        let msgs = vec![
            sys("You are helpful"),
            sys("Additional instructions"),
            user("hello"),
            assistant("hi"),
        ];
        let rounds = group_by_api_round(&msgs);
        assert_eq!(rounds.len(), 1);
        assert_eq!(rounds[0].messages.len(), 4);
    }

    #[test]
    fn ten_rounds_produces_ten_groups() {
        let mut msgs = vec![sys("system prompt")];
        for i in 0..10 {
            msgs.push(user(&format!("question {i}")));
            msgs.push(assistant(&format!("answer {i}")));
        }
        let rounds = group_by_api_round(&msgs);
        assert_eq!(rounds.len(), 10, "10 user-assistant pairs = 10 rounds");
        // System message should be in the first round.
        assert_eq!(rounds[0].messages[0].role, Role::System);
        assert_eq!(rounds[0].messages.len(), 3); // sys + user + assistant
        for r in &rounds[1..] {
            assert_eq!(r.messages.len(), 2); // user + assistant
        }
    }

    #[test]
    fn tool_messages_stay_with_their_round() {
        let msgs = vec![
            user("search for X"),
            assistant("calling tool"),
            tool("result of tool"),
            assistant("based on the result..."),
            user("thanks"),
            assistant("you're welcome"),
        ];
        let rounds = group_by_api_round(&msgs);
        assert_eq!(rounds.len(), 2);
        // First round: user + assistant + tool + assistant
        assert_eq!(rounds[0].messages.len(), 4);
        // Second round: user + assistant
        assert_eq!(rounds[1].messages.len(), 2);
    }

    #[test]
    fn estimated_tokens_are_nonzero() {
        let msgs = vec![user("hello world"), assistant("hi there buddy")];
        let rounds = group_by_api_round(&msgs);
        assert_eq!(rounds.len(), 1);
        assert!(rounds[0].estimated_tokens > 0);
    }

    #[test]
    fn incomplete_round_is_still_returned() {
        let msgs = vec![user("hello")];
        let rounds = group_by_api_round(&msgs);
        assert_eq!(rounds.len(), 1);
        assert_eq!(rounds[0].messages.len(), 1);
    }
}
