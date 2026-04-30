use std::sync::Arc;

use fastclaw_core::types::ChatMessage;

use crate::llm::LlmProvider;
use super::context_compressor;
use super::tool_executor::{
    dedup_repeated_tool_calls, microcompact_tool_results, time_based_microcompact,
    DEFAULT_CACHE_WINDOW_DURATION,
};

/// Result of the unified pre-query compression pipeline.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct UnifiedCompactResult {
    pub estimated_tokens: usize,
    pub compressed_by_llm: bool,
    pub tokens_saved_by_llm: usize,
    pub pipeline_applied: bool,
}

/// Run all pre-query compression steps in a single call.
///
/// Replaces the ~80 lines of scattered compression code in `execute_stream_inner`:
///   1. Microcompact old tool results
///   2. Deduplicate repeated tool calls
///   3. ContentFilterHook (truncate oversized results, remove empty messages)
///   4. SystemReminderHook (nudge every N turns)
///   5. ContextPipeline::pre_query_compact (snip + importance)
///   6. LLM-based compression (with circuit breaker)
///   7. Hard fit to context window
#[allow(clippy::too_many_arguments)]
pub(crate) async fn unified_pre_query_compact(
    messages: &mut Vec<ChatMessage>,
    pipeline: &mut fastclaw_context::ContextPipeline,
    context_window: u32,
    max_tokens: Option<u32>,
    provider: &Arc<dyn LlmProvider>,
    model: &str,
    last_estimated_tokens: usize,
    iteration_boundaries: &[(usize, std::time::Instant)],
) -> UnifiedCompactResult {
    // Step 0: Time-based microcompact — collapse tool results outside the prompt
    // cache window (default 5 min). These won't get cache hits so keeping them
    // verbatim wastes context budget.
    let time_compacted = time_based_microcompact(
        messages,
        iteration_boundaries,
        DEFAULT_CACHE_WINDOW_DURATION,
    );
    if time_compacted > 0 {
        tracing::debug!(time_compacted, "time-based microcompact collapsed stale tool results");
    }

    // Step 1: Microcompact old tool results (keep last 3 fully, next 3 faded)
    microcompact_tool_results(messages, 3);

    // Step 2: Deduplicate repeated tool calls on the same target
    dedup_repeated_tool_calls(messages);

    // Step 3: Content filter — truncate oversized tool results, remove empty,
    // deduplicate consecutive identical system messages.
    {
        let filter = fastclaw_context::ContentFilterHook::new(2000);
        let _ = fastclaw_context::ContextHook::on_assemble(&filter, messages).await;
    }

    // Step 4: System reminder — nudge every 20 user turns
    {
        let reminder = fastclaw_context::SystemReminderHook::default();
        let _ = fastclaw_context::ContextHook::on_assemble(&reminder, messages).await;
    }

    // Step 5: Pipeline pre_query_compact (snip + importance-based eviction)
    let (compacted, pipeline_meta) = pipeline.pre_query_compact(messages);
    let pipeline_applied = pipeline_meta.snip_applied || pipeline_meta.micro_applied;
    if pipeline_applied {
        tracing::info!(
            snip_freed = pipeline_meta.snip_tokens_freed,
            snip_rounds = pipeline_meta.snip_rounds_removed,
            micro_evicted = pipeline_meta.micro_evicted,
            total_freed = pipeline_meta.total_tokens_freed,
            "pre-query pipeline compacted context"
        );
        *messages = compacted;
    }

    // Step 6: LLM-based compression (check circuit breaker first)
    let compress_result = if pipeline.should_attempt_autocompact() {
        let local_estimate = fastclaw_context::estimate_messages_tokens(messages);
        tracing::debug!(
            local_estimate,
            api_prompt_tokens = last_estimated_tokens,
            "pre-compact: entering LLM compression"
        );
        let result = context_compressor::try_compress_chat(
            messages,
            context_window,
            provider,
            model,
            last_estimated_tokens,
        )
        .await;

        if result.compressed {
            pipeline.record_autocompact_success();
            tracing::info!(
                original = result.original_tokens,
                new = result.new_tokens,
                saved = result.original_tokens.saturating_sub(result.new_tokens),
                "post-compact: LLM compression reduced context"
            );
        } else if result.original_tokens > 0 {
            pipeline.record_autocompact_failure();
        }
        result
    } else {
        tracing::debug!("LLM autocompact skipped (circuit breaker tripped)");
        context_compressor::CompressionResult {
            compressed: false,
            original_tokens: 0,
            new_tokens: 0,
            messages: messages.clone(),
            history_file: None,
        }
    };

    // Step 7: Hard fit messages within context window budget
    let estimated_tokens = fastclaw_context::ContextEngine::fit_to_context_window(
        messages,
        context_window,
        max_tokens,
    );

    let tokens_saved_by_llm = if compress_result.compressed {
        compress_result
            .original_tokens
            .saturating_sub(compress_result.new_tokens)
    } else {
        0
    };

    UnifiedCompactResult {
        estimated_tokens,
        compressed_by_llm: compress_result.compressed,
        tokens_saved_by_llm,
        pipeline_applied,
    }
}
