//! Context Collapse — out-of-band storage for API round summaries.
//!
//! Instead of mutating the original message array, `CollapseStore` keeps
//! a side table of `CollapseSpan`s. Each span records which API rounds
//! have been collapsed and what the LLM-generated summary text is.
//!
//! At query time, [`project`] merges these summaries into the message
//! list, replacing collapsed rounds with their summary while leaving
//! uncollapsed messages intact. The original messages are never modified.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// A collapsed range of API rounds together with its summary.
///
/// `start_round..=end_round` identifies the rounds (0-indexed, matching
/// [`ApiRound::index`] from `snip.rs`) that this summary replaces.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CollapseSpan {
    /// First API round index (inclusive) covered by this span.
    pub start_round: usize,
    /// Last API round index (inclusive) covered by this span.
    pub end_round: usize,
    /// LLM-generated summary that replaces the original messages.
    pub summary: String,
    /// Estimated token count of the summary text.
    pub summary_tokens: usize,
    /// Total token count of the original messages before collapse.
    pub original_tokens: usize,
    /// Unix-millis timestamp when this collapse was created.
    pub created_at: u64,
}

impl CollapseSpan {
    /// How many tokens this collapse saves (positive = net win).
    pub fn tokens_saved(&self) -> usize {
        self.original_tokens.saturating_sub(self.summary_tokens)
    }

    /// Number of rounds covered.
    pub fn round_count(&self) -> usize {
        self.end_round - self.start_round + 1
    }
}

/// Persistent, non-destructive storage for collapsed API round summaries.
///
/// Keyed by the `start_round` of each span for O(log n) lookups. Spans
/// must not overlap — `add` rejects overlapping entries.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct CollapseStore {
    spans: BTreeMap<usize, CollapseSpan>,
}

impl CollapseStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a collapse span. Returns `Err` if it overlaps an existing span.
    pub fn add(&mut self, span: CollapseSpan) -> Result<(), CollapseOverlapError> {
        if span.start_round > span.end_round {
            return Err(CollapseOverlapError {
                message: format!(
                    "invalid range: start_round ({}) > end_round ({})",
                    span.start_round, span.end_round
                ),
            });
        }
        for existing in self.spans.values() {
            if ranges_overlap(
                span.start_round,
                span.end_round,
                existing.start_round,
                existing.end_round,
            ) {
                return Err(CollapseOverlapError {
                    message: format!(
                        "new span [{}..={}] overlaps existing [{}..={}]",
                        span.start_round, span.end_round,
                        existing.start_round, existing.end_round,
                    ),
                });
            }
        }
        self.spans.insert(span.start_round, span);
        Ok(())
    }

    /// Look up the collapse span that covers `round_index`, if any.
    pub fn get_for_round(&self, round_index: usize) -> Option<&CollapseSpan> {
        self.spans
            .values()
            .find(|span| round_index >= span.start_round && round_index <= span.end_round)
    }

    /// Return all collapse spans in ascending round order.
    pub fn all(&self) -> Vec<&CollapseSpan> {
        self.spans.values().collect()
    }

    /// Remove the span that starts at `start_round`.
    pub fn remove(&mut self, start_round: usize) -> Option<CollapseSpan> {
        self.spans.remove(&start_round)
    }

    /// Number of stored spans.
    pub fn len(&self) -> usize {
        self.spans.len()
    }

    pub fn is_empty(&self) -> bool {
        self.spans.is_empty()
    }

    /// Total tokens saved across all spans.
    pub fn total_tokens_saved(&self) -> usize {
        self.spans.values().map(|s| s.tokens_saved()).sum()
    }

    /// Check if a given round is collapsed.
    pub fn is_round_collapsed(&self, round_index: usize) -> bool {
        self.get_for_round(round_index).is_some()
    }

    /// Clear all spans.
    pub fn clear(&mut self) {
        self.spans.clear();
    }
}

fn ranges_overlap(a_start: usize, a_end: usize, b_start: usize, b_end: usize) -> bool {
    a_start <= b_end && b_start <= a_end
}

/// Returned when a new span overlaps an existing one.
#[derive(Debug, Clone)]
pub struct CollapseOverlapError {
    pub message: String,
}

impl std::fmt::Display for CollapseOverlapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "collapse overlap: {}", self.message)
    }
}

impl std::error::Error for CollapseOverlapError {}

// ─── Project collapsed summaries into a message list ─────────────────

use fastclaw_core::types::{ChatMessage, Role};
use serde_json::json;

use crate::snip::group_by_api_round;

/// Non-destructively project collapsed summaries into the message list.
///
/// For each API round that is collapsed, replace its messages with a
/// single system message containing the summary. Uncollapsed rounds
/// pass through unchanged. The original `messages` slice is not modified.
pub fn project(messages: &[ChatMessage], store: &CollapseStore) -> Vec<ChatMessage> {
    if store.is_empty() {
        return messages.to_vec();
    }
    let rounds = group_by_api_round(messages);
    let mut result: Vec<ChatMessage> = Vec::new();
    let mut emitted_spans = std::collections::HashSet::new();

    for round in &rounds {
        if let Some(span) = store.get_for_round(round.index) {
            if emitted_spans.insert(span.start_round) {
                result.push(ChatMessage {
                    role: Role::System,
                    content: Some(json!(format!(
                        "[Summary of rounds {}–{}]: {}",
                        span.start_round, span.end_round, span.summary
                    ))),
                    name: None,
                    tool_calls: None,
                    tool_call_id: None,
                });
            }
        } else {
            result.extend(round.messages.iter().cloned());
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use fastclaw_core::types::Role;
    use serde_json::json;

    fn now_millis() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }

    fn make_span(start: usize, end: usize, summary: &str) -> CollapseSpan {
        CollapseSpan {
            start_round: start,
            end_round: end,
            summary: summary.to_string(),
            summary_tokens: summary.len() / 4,
            original_tokens: 1000,
            created_at: now_millis(),
        }
    }

    fn msg(role: Role, text: &str) -> ChatMessage {
        ChatMessage {
            role,
            content: Some(json!(text)),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // Basic CRUD
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn add_and_retrieve() {
        let mut store = CollapseStore::new();
        assert!(store.is_empty());

        store.add(make_span(0, 2, "First three rounds discussed setup.")).unwrap();
        assert_eq!(store.len(), 1);

        let span = store.get_for_round(1).unwrap();
        assert_eq!(span.start_round, 0);
        assert_eq!(span.end_round, 2);
        assert!(span.summary.contains("setup"));
    }

    #[test]
    fn remove_span() {
        let mut store = CollapseStore::new();
        store.add(make_span(0, 2, "rounds 0-2")).unwrap();
        store.add(make_span(5, 7, "rounds 5-7")).unwrap();
        assert_eq!(store.len(), 2);

        let removed = store.remove(0).unwrap();
        assert_eq!(removed.start_round, 0);
        assert_eq!(store.len(), 1);
        assert!(store.get_for_round(1).is_none());
        assert!(store.get_for_round(5).is_some());
    }

    #[test]
    fn remove_nonexistent_returns_none() {
        let mut store = CollapseStore::new();
        assert!(store.remove(42).is_none());
    }

    #[test]
    fn clear_empties_store() {
        let mut store = CollapseStore::new();
        store.add(make_span(0, 1, "a")).unwrap();
        store.add(make_span(3, 4, "b")).unwrap();
        store.clear();
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);
    }

    // ═══════════════════════════════════════════════════════════════
    // Multiple rounds collapsed simultaneously
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn multiple_non_overlapping_spans() {
        let mut store = CollapseStore::new();
        store.add(make_span(0, 2, "early context")).unwrap();
        store.add(make_span(5, 8, "middle context")).unwrap();
        store.add(make_span(12, 15, "later context")).unwrap();
        assert_eq!(store.len(), 3);

        assert!(store.is_round_collapsed(0));
        assert!(store.is_round_collapsed(2));
        assert!(!store.is_round_collapsed(3));
        assert!(store.is_round_collapsed(6));
        assert!(!store.is_round_collapsed(10));
        assert!(store.is_round_collapsed(14));
    }

    #[test]
    fn overlapping_spans_rejected() {
        let mut store = CollapseStore::new();
        store.add(make_span(2, 5, "original")).unwrap();

        let result = store.add(make_span(4, 7, "overlap"));
        assert!(result.is_err());
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn exact_overlap_rejected() {
        let mut store = CollapseStore::new();
        store.add(make_span(3, 6, "first")).unwrap();
        assert!(store.add(make_span(3, 6, "duplicate")).is_err());
    }

    #[test]
    fn adjacent_spans_allowed() {
        let mut store = CollapseStore::new();
        store.add(make_span(0, 2, "first")).unwrap();
        store.add(make_span(3, 5, "second")).unwrap();
        assert_eq!(store.len(), 2);
    }

    #[test]
    fn invalid_range_rejected() {
        let mut store = CollapseStore::new();
        let result = store.add(make_span(5, 3, "bad range"));
        assert!(result.is_err());
    }

    // ═══════════════════════════════════════════════════════════════
    // Serialization / deserialization
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn serde_round_trip() {
        let mut store = CollapseStore::new();
        store.add(make_span(0, 3, "setup & config")).unwrap();
        store.add(make_span(6, 9, "debugging session")).unwrap();

        let json = serde_json::to_string(&store).unwrap();
        let restored: CollapseStore = serde_json::from_str(&json).unwrap();

        assert_eq!(store, restored);
        assert_eq!(restored.len(), 2);
        assert!(restored.is_round_collapsed(1));
        assert!(restored.is_round_collapsed(7));
    }

    #[test]
    fn span_serde_round_trip() {
        let span = make_span(10, 15, "complex analysis");
        let json = serde_json::to_string(&span).unwrap();
        let restored: CollapseSpan = serde_json::from_str(&json).unwrap();
        assert_eq!(span, restored);
    }

    // ═══════════════════════════════════════════════════════════════
    // Token accounting
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn total_tokens_saved() {
        let mut store = CollapseStore::new();
        // original_tokens=1000, summary_tokens ≈ len/4
        store.add(make_span(0, 2, "short")).unwrap(); // saves ~999
        store.add(make_span(5, 7, "another short")).unwrap(); // saves ~997
        assert!(store.total_tokens_saved() > 1900);
    }

    #[test]
    fn tokens_saved_per_span() {
        let span = CollapseSpan {
            start_round: 0,
            end_round: 3,
            summary: "x".repeat(400),
            summary_tokens: 100,
            original_tokens: 5000,
            created_at: now_millis(),
        };
        assert_eq!(span.tokens_saved(), 4900);
        assert_eq!(span.round_count(), 4);
    }

    // ═══════════════════════════════════════════════════════════════
    // Projection — non-destructive message replacement
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn project_empty_store_returns_original() {
        let messages = vec![
            msg(Role::System, "sys"),
            msg(Role::User, "q1"),
            msg(Role::Assistant, "a1"),
        ];
        let store = CollapseStore::new();
        let projected = project(&messages, &store);
        assert_eq!(projected.len(), messages.len());
    }

    #[test]
    fn project_replaces_collapsed_rounds() {
        let messages = vec![
            msg(Role::System, "system prompt"),
            msg(Role::User, "question 0"),
            msg(Role::Assistant, "answer 0"),
            msg(Role::User, "question 1"),
            msg(Role::Assistant, "answer 1"),
            msg(Role::User, "question 2"),
            msg(Role::Assistant, "answer 2"),
        ];

        let mut store = CollapseStore::new();
        // Collapse round 0 (sys + user-q0 + assistant-a0)
        store.add(make_span(0, 0, "Initial setup discussion")).unwrap();

        let projected = project(&messages, &store);

        // Round 0 replaced by 1 summary message, rounds 1-2 remain intact (4 msgs)
        assert_eq!(projected.len(), 5); // 1 summary + 2 user + 2 assistant
        let summary_text = projected[0].text_content().unwrap();
        assert!(summary_text.contains("Initial setup discussion"));
        assert_eq!(projected[1].role, Role::User);
    }

    #[test]
    fn project_preserves_original_messages() {
        let messages = vec![
            msg(Role::System, "system"),
            msg(Role::User, "q0"),
            msg(Role::Assistant, "a0"),
            msg(Role::User, "q1"),
            msg(Role::Assistant, "a1"),
        ];
        let original_clone = messages.clone();

        let mut store = CollapseStore::new();
        store.add(make_span(0, 0, "summary")).unwrap();
        let _projected = project(&messages, &store);

        // Original messages must be unchanged
        assert_eq!(messages.len(), original_clone.len());
        for (a, b) in messages.iter().zip(original_clone.iter()) {
            assert_eq!(a.role, b.role);
            assert_eq!(a.content, b.content);
        }
    }

    #[test]
    fn project_multiple_collapsed_spans() {
        let mut messages = vec![msg(Role::System, "sys")];
        for i in 0..10 {
            messages.push(msg(Role::User, &format!("q{i}")));
            messages.push(msg(Role::Assistant, &format!("a{i}")));
        }

        let mut store = CollapseStore::new();
        store.add(make_span(0, 2, "rounds 0-2 summary")).unwrap();
        store.add(make_span(5, 7, "rounds 5-7 summary")).unwrap();

        let projected = project(&messages, &store);

        // Rounds 0-2 (3 rounds) → 1 summary
        // Rounds 3-4 (2 rounds) → 4 messages (2 user + 2 assistant)
        // Rounds 5-7 (3 rounds) → 1 summary
        // Rounds 8-9 (2 rounds) → 4 messages (2 user + 2 assistant)
        // Total: 1 + 4 + 1 + 4 = 10
        assert_eq!(projected.len(), 10);

        let summary_count = projected
            .iter()
            .filter(|m| {
                m.role == Role::System
                    && m.text_content()
                        .as_deref()
                        .map_or(false, |t| t.contains("[Summary"))
            })
            .count();
        assert_eq!(summary_count, 2);
    }

    #[test]
    fn project_reduces_token_count() {
        let mut messages = vec![msg(Role::System, "sys")];
        for i in 0..20 {
            messages.push(msg(Role::User, &format!("q{i} {}", "x".repeat(500))));
            messages.push(msg(Role::Assistant, &format!("a{i} {}", "x".repeat(500))));
        }

        let tokens_before = crate::compressor::estimate_messages_tokens(&messages);

        let mut store = CollapseStore::new();
        store.add(make_span(0, 5, "Early discussion summary")).unwrap();
        store.add(make_span(8, 12, "Middle discussion summary")).unwrap();

        let projected = project(&messages, &store);
        let tokens_after = crate::compressor::estimate_messages_tokens(&projected);

        assert!(
            tokens_after < tokens_before,
            "projected tokens ({tokens_after}) should be less than original ({tokens_before})"
        );
    }

    // ═══════════════════════════════════════════════════════════════
    // all() ordering
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn all_returns_spans_in_order() {
        let mut store = CollapseStore::new();
        store.add(make_span(10, 12, "later")).unwrap();
        store.add(make_span(0, 2, "earlier")).unwrap();
        store.add(make_span(5, 7, "middle")).unwrap();

        let all = store.all();
        assert_eq!(all.len(), 3);
        assert_eq!(all[0].start_round, 0);
        assert_eq!(all[1].start_round, 5);
        assert_eq!(all[2].start_round, 10);
    }
}
