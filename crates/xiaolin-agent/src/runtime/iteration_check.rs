use xiaolin_core::types::{ChatMessage, Role};
use xiaolin_protocol::{AgentEvent, ContextWarningLevel, TurnSummary};

use super::query_deps::QueryDeps;
use super::query_state;
use super::session_memory;
use super::stream_engine::send_stream_event;
use super::turn_state::{TurnMutableState, TurnServices};
use super::make_turn_summary;

/// Outcome of the per-iteration pre-check phase.
pub(crate) enum PreCheckOutcome {
    /// All checks passed; proceed to LLM call.
    Continue {
        estimated_tokens: usize,
    },
    /// Turn should terminate gracefully (e.g. blocking limit reached).
    EarlyFinish(TurnSummary),
    /// Turn should terminate with a fatal error.
    FatalError(anyhow::Error),
}

/// Performs all per-iteration pre-checks before the LLM call.
///
/// Covers:
/// - Consecutive error limit check (→ FatalError)
/// - begin_iteration bookkeeping
/// - Steer message draining
/// - Message boundary tracking
/// - Plan content restoration
/// - Context compaction (pre_query_compact)
/// - Session memory persistence
/// - Context usage/warning events
/// - Blocking limit check (→ EarlyFinish)
pub(crate) async fn iteration_pre_check(
    ms: &mut TurnMutableState,
    svc: &TurnServices,
) -> PreCheckOutcome {
    // ── 1. Consecutive error limit ─────────────────────────────────────
    if let Some(query_state::LoopTransition::Terminal(_)) = ms.query_loop.check_pre_iteration() {
        tracing::warn!(
            agent_id = %svc.config.agent_id,
            consecutive_errors = ms.query_loop.consecutive_errors,
            "stopping outer stream loop — consecutive error limit reached"
        );
        let failure_detail = ms.query_loop.format_failure_summary();
        let user_msg = if failure_detail.is_empty() {
            format!(
                "执行过程中遇到连续 {} 次工具错误，已自动停止。请检查工具配置或尝试换一种方式描述任务。",
                ms.query_loop.consecutive_errors
            )
        } else {
            format!(
                "执行过程中遇到连续工具错误，已自动停止。\n出错的工具调用：\n{}\n\n请检查相关配置或尝试换一种方式。",
                failure_detail
            )
        };
        let _ = send_stream_event(
            &svc.tx,
            AgentEvent::Error {
                turn_id: svc.turn_id.clone(),
                message: user_msg,
                error_code: None,
            },
            false,
        )
        .await;
        svc.runtime
            .finalize_injected_skills(&ms.injected_skill_ids, false)
            .await;
        return PreCheckOutcome::FatalError(anyhow::anyhow!(
            "agent '{}' stopped: {} consecutive tool errors",
            svc.config.agent_id,
            ms.query_loop.consecutive_errors
        ));
    }

    // ── 2. Begin iteration bookkeeping ─────────────────────────────────
    ms.query_loop.begin_iteration();

    // ── 3. Drain mid-turn steer messages ───────────────────────────────
    if let Ok(inbox) = crate::builtin_tools::STEER_INBOX.try_with(|s| s.clone()) {
        let mut rx = inbox.lock().await;
        while let Ok(msg) = rx.try_recv() {
            tracing::info!(
                role = %msg.role,
                content_len = msg.content.len(),
                "injecting steer message into agentic loop"
            );
            ms.messages.push(ChatMessage {
                role: Role::User,
                content: Some(serde_json::Value::String(msg.content)),
                ..Default::default()
            });
        }
    }

    // ── 4. Record iteration message boundaries ─────────────────────────
    ms.query_loop
        .iteration_msg_boundaries
        .push((ms.messages.len(), std::time::Instant::now()));

    // ── 5. Populate plan content for restoration ───────────────────────
    if let Some(ref session_id) = svc.session_id {
        let plan_store = crate::builtin_tools::PlanFileStore::new(None);
        ms.query_loop
            .restoration_state
            .populate_plan_from_store(session_id, &plan_store);
    }

    // ── 6. Unified context compaction ──────────────────────────────────
    let compact_t0 = std::time::Instant::now();
    let compact_result = svc
        .deps
        .pre_query_compact(
            &mut ms.messages,
            svc.context_window,
            ms.max_tokens,
            &svc.model,
            ms.query_loop.last_estimated_tokens,
            &ms.query_loop.iteration_msg_boundaries,
            svc.todo_store.as_ref(),
            svc.config.behavior.enable_smart_compression,
            Some(&ms.query_loop.restoration_state),
            ms.query_loop.session_memory.as_ref(),
        )
        .await;
    tracing::info!(
        elapsed_ms = compact_t0.elapsed().as_millis() as u64,
        iteration = ms.query_loop.iteration,
        "perf: pre_query_compact"
    );
    ms.query_loop.last_estimated_tokens = compact_result.estimated_tokens;
    let estimated_tokens = compact_result.estimated_tokens;

    // ── 7. Persist session memory if extracted ─────────────────────────
    if let Some(ref mem) = compact_result.extracted_memory {
        ms.query_loop.session_memory = Some(mem.clone());
        if let (Some(store), Some(sid)) = (&svc.session_store, svc.session_id.as_deref()) {
            session_memory::persist_session_memory(store.as_ref(), sid, mem).await;
        }
    }

    // ── 8. Emit live context usage update ──────────────────────────────
    let _ = send_stream_event(
        &svc.tx,
        AgentEvent::ContextUsageUpdate {
            turn_id: svc.turn_id.clone(),
            used_tokens: estimated_tokens as u32,
            limit_tokens: svc.context_window,
            compressed: compact_result.compressed_by_llm,
            tokens_saved: compact_result.tokens_saved_by_llm as u32,
        },
        false,
    )
    .await;

    // ── 9. Context warnings (85% soft, 90% hard) ──────────────────────
    let usage_ratio = estimated_tokens as f32 / svc.context_window.max(1) as f32;

    if usage_ratio > 0.85 && !ms.query_loop.compact_warning_sent {
        ms.query_loop.compact_warning_sent = true;
        let _ = send_stream_event(
            &svc.tx,
            AgentEvent::ContextWarning {
                turn_id: svc.turn_id.clone(),
                level: ContextWarningLevel::Soft,
                used_tokens: estimated_tokens as u32,
                limit_tokens: svc.context_window,
                message: format!(
                    "Context is {:.0}% full ({}/{} tokens). \
                     Run /compact to free space, or the system will auto-compact if enabled.",
                    usage_ratio * 100.0,
                    estimated_tokens,
                    svc.context_window,
                ),
            },
            false,
        )
        .await;
    }

    if usage_ratio > 0.90 {
        let _ = send_stream_event(
            &svc.tx,
            AgentEvent::ContextWarning {
                turn_id: svc.turn_id.clone(),
                level: ContextWarningLevel::Hard,
                used_tokens: estimated_tokens as u32,
                limit_tokens: svc.context_window,
                message: format!(
                    "Context usage is at {:.0}% ({}/{} tokens). Consider starting a new session.",
                    usage_ratio * 100.0,
                    estimated_tokens,
                    svc.context_window,
                ),
            },
            false,
        )
        .await;
    }

    // ── 10. Record compaction event ────────────────────────────────────
    if compact_result.compressed_by_llm || compact_result.pipeline_applied {
        let method = if compact_result.compressed_by_llm {
            "llm"
        } else {
            "pipeline"
        };
        svc.runtime_observer
            .record_compact(
                ms.query_loop.last_estimated_tokens,
                compact_result.estimated_tokens,
                method,
            )
            .await;
    }

    // ── 11. Blocking limit check (95% context) ─────────────────────────
    let just_compacted = compact_result.compressed_by_llm || compact_result.pipeline_applied;
    if let Some(query_state::LoopTransition::Terminal(
        query_state::TerminalReason::BlockingLimit,
    )) = ms.query_loop.check_blocking_limit(
        estimated_tokens,
        svc.context_window,
        svc.auto_compact_enabled,
        just_compacted,
    ) {
        tracing::warn!(
            agent_id = %svc.config.agent_id,
            estimated_tokens,
            context_window = svc.context_window,
            "blocking limit reached (>= 95% context window) — stopping"
        );
        let _ = send_stream_event(
            &svc.tx,
            AgentEvent::Error {
                turn_id: svc.turn_id.clone(),
                message: format!(
                    "Context window is nearly full ({}/{} tokens, {:.0}%). \
                     Please run /compact to free space, or start a new session.",
                    estimated_tokens,
                    svc.context_window,
                    usage_ratio * 100.0,
                ),
                error_code: Some(xiaolin_protocol::ErrorCode::ContextWindowExceeded),
            },
            false,
        )
        .await;
        svc.runtime
            .finalize_injected_skills(&ms.injected_skill_ids, false)
            .await;
        return PreCheckOutcome::EarlyFinish(make_turn_summary(
            &svc.turn_id,
            &ms.query_loop,
            svc.stream_start,
            svc.context_window,
        ));
    }

    PreCheckOutcome::Continue { estimated_tokens }
}
