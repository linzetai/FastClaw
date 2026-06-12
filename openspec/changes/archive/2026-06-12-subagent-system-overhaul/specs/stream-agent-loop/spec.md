## Implementation Architecture

### Event Architecture: Dual-Channel Design

```
┌──────────────────────────────────────────────────────────┐
│  execute_as_stream() 主循环                               │
│  yields: AgentStep (主循环事件)                            │
│    TurnStart, Delta, ToolExecuting, ToolResult,           │
│    ToolRoundBoundary, SteeringInjected, ContextWarning,   │
│    ModeChange, GoalUpdated, TurnEnd, Error                │
└─────────────────────┬────────────────────────────────────┘
                      │ 兼容层 collect + convert
                      ▼
┌──────────────────────────────────────────────────────────┐
│  tx: mpsc::Sender<AgentEvent>                             │
│  - 主循环 step 转为 AgentEvent 后 send                    │
│  - 侧路径直接 send（不经过 stream）：                      │
│    · ToolProgress (from orchestrator shell runtime)        │
│    · ApprovalRequired/Resolved (from orchestrator)         │
│    · GuardianAssessment/Warning (from orchestrator)        │
│    · AskQuestion, BriefMessage (from builtin tools)        │
│    · SubAgent* events (from subagent_manager)              │
│    · CompactBoundary (from session_bridge)                 │
│    · MemoryStored/Recalled (from memory tools)             │
└──────────────────────────────────────────────────────────┘
```

**原则**: AgentStep 只覆盖主循环内产生的事件。侧路径（tools/orchestrator/subagent）继续直接 send 到 `tx`。兼容层中 `tx` 被 clone 传给 orchestrator/tools，主循环 yield 的 step 也转为 AgentEvent 后 send 到同一 tx。

### File Structure

```
crates/xiaolin-agent/src/runtime/
├── mod.rs                    # 入口 + execute_unified / execute_stream (兼容层)
├── agent_step.rs             # [NEW] AgentStep 枚举 + TurnEndReason + From<AgentStep> for AgentEvent
├── agent_context.rs          # [NEW] AgentContext struct + from_params 构造
├── turn_setup.rs             # [NEW] Pre-loop setup (从 mod.rs L865-1153 抽取)
├── iteration_check.rs        # [NEW] Pre-iteration checks (从 L1155-1370 抽取)
├── llm_call.rs               # [NEW] LLM stream 准备 + 消费 (从 L1385-1865 抽取)
├── tool_round.rs             # [NEW] 工具轮次 dispatch + 结果处理 (从 L2327-2794 抽取)
├── post_tool.rs              # [NEW] 后处理 (从 L2796-3019 抽取)
├── query_state.rs            # [EXISTING] QueryLoopState + TerminalReason (保留)
├── streaming_tool_executor.rs # [EXISTING] StreamingToolExecutor (保留)
├── dispatcher.rs             # [EXISTING] ToolDispatcher (保留)
└── stream_engine.rs          # [EXISTING] send_stream_event (保留，兼容层使用)
```

### AgentStep Enum Definition

**Crate 位置**: `xiaolin-agent`（运行时内部类型，不暴露到协议层）

```rust
use xiaolin_protocol::{TurnId, TurnSummary, AgentEvent};

/// 主循环内产生的事件。不覆盖侧路径事件（ToolProgress, ApprovalRequired 等）。
pub enum AgentStep {
    /// 轮次开始
    TurnStart { turn_id: TurnId, session_id: Option<String> },
    
    /// LLM 流式增量（与 AgentEvent::ContentDelta 一致，保留原始 JSON value）
    Delta { delta: serde_json::Value, reasoning: Option<String> },
    
    /// 工具开始执行
    ToolExecuting { turn_id: TurnId, call_id: String, tool_name: String, args: serde_json::Value },
    
    /// 工具执行完成
    ToolResult { turn_id: TurnId, call_id: String, tool_name: String, output: String, success: bool, metadata: Option<serde_json::Value> },
    
    /// 工具轮次边界（所有 tool results 后、下一次 LLM 调用前）
    ToolRoundBoundary { iteration: u32 },
    
    /// Steering 消息被注入到对话中
    SteeringInjected { count: usize, sources: Vec<String> },
    
    /// Context 使用量更新（对应 AgentEvent::ContextUsageUpdate）
    ContextUsage { used_tokens: u32, limit_tokens: u32, compressed: bool, tokens_saved: Option<u32> },
    
    /// Context 警告（即将超限）（对应 AgentEvent::ContextWarning）
    ContextWarning { message: String, percentage: f32 },
    
    /// 模式切换（对应 AgentEvent::ModeChange）
    ModeChange { from: String, to: String },
    
    /// Goal 状态更新（对应 AgentEvent::GoalUpdated）
    GoalUpdated { goal_id: String, status: String },
    
    /// Plan file 更新（对应 AgentEvent::PlanFileUpdate）
    PlanFileUpdate { path: String },
    
    /// 轮次结束
    TurnEnd { turn_id: TurnId, reason: TurnEndReason, summary: TurnSummary },
    
    /// 错误
    Error { message: String, error_code: Option<String>, recoverable: bool },
}

/// 轮次结束原因（与 query_state.rs TerminalReason 对齐）
pub enum TurnEndReason {
    /// 正常完成（LLM 不再调用工具）— 对应 TerminalReason::EndTurn
    Completed,
    /// 达到最大迭代次数 — 对应 TerminalReason::MaxIterations
    MaxTurns,
    /// 被用户/父级取消 — 对应 TerminalReason::Aborted
    Cancelled,
    /// 上下文窗口已满 — 对应 TerminalReason::BlockingLimit
    ContextLimit,
    /// 预算耗尽 — 对应 TerminalReason::BudgetExhausted
    BudgetExceeded,
    /// 连续错误超限 — 对应 TerminalReason::ConsecutiveErrors
    ConsecutiveErrors,
    /// 收益递减（工具重复调用）— 对应 TerminalReason::DiminishingReturns
    DiminishingReturns,
    /// Plan 审批等待（暂停循环）
    PlanApprovalPending,
    /// 不可恢复错误
    Error(String),
}

/// AgentStep → AgentEvent 转换（用于兼容层）
impl AgentStep {
    /// 转换为 AgentEvent。部分 step 可能产生多个 event（如 Delta 同时有 content 和 reasoning）。
    /// 返回 None 表示该 step 无需转发为 AgentEvent（如 ToolRoundBoundary）。
    pub fn into_agent_events(self, turn_id: &TurnId) -> Vec<AgentEvent> {
        match self {
            Self::TurnStart { .. } => vec![AgentEvent::TurnStart { .. }],
            Self::Delta { delta, reasoning } => {
                let mut events = vec![AgentEvent::ContentDelta { delta }];
                if let Some(r) = reasoning {
                    events.push(AgentEvent::ReasoningDelta { delta: r });
                }
                events
            },
            Self::ToolExecuting { .. } => vec![AgentEvent::ToolExecuting { .. }],
            Self::ToolResult { .. } => vec![AgentEvent::ToolResult { .. }],
            Self::ContextUsage { .. } => vec![AgentEvent::ContextUsageUpdate { .. }],
            Self::ContextWarning { .. } => vec![AgentEvent::ContextWarning { .. }],
            Self::ModeChange { .. } => vec![AgentEvent::ModeChange { .. }],
            Self::GoalUpdated { .. } => vec![AgentEvent::GoalUpdated { .. }],
            Self::PlanFileUpdate { .. } => vec![AgentEvent::PlanFileUpdate { .. }],
            Self::TurnEnd { .. } => vec![AgentEvent::TurnEnd { .. }],
            Self::Error { .. } => vec![AgentEvent::Error { .. }],
            // 内部标记，不产生协议事件
            Self::ToolRoundBoundary { .. } | Self::SteeringInjected { .. } => vec![],
        }
    }
    
    /// 是否允许在 channel full 时丢弃
    pub fn is_lossy(&self) -> bool {
        matches!(self, Self::ContextUsage { .. } | Self::ContextWarning { .. })
    }
}
```

### AgentContext Struct Definition

```rust
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

pub struct AgentContext {
    // === Required ===
    pub config: AgentConfig,
    pub request: ChatRequest,
    pub tool_registry: Arc<ToolRegistry>,
    
    // === Streaming (required for execute_unified, not for pure stream consumers) ===
    /// 事件发送通道（侧路径 clone 此 tx 直接 emit ToolProgress 等事件）
    pub tx: Option<mpsc::Sender<AgentEvent>>,
    
    // === Optional - LLM ===
    pub llm_override: Option<Arc<dyn LlmProvider>>,
    
    // === Optional - SubAgent ===
    pub subagent_prompt: Option<String>,
    
    // === Optional - Execution control ===
    pub mode_state: Option<ExecutionModeState>,
    pub orchestrator: Option<Arc<ToolOrchestrator>>,
    pub interaction_handle: Option<InteractionHandle>,
    /// 默认 AutoApprove（与 execute_stream 行为一致）
    pub approval_strategy: ApprovalStrategy,
    pub runtime_registry: Option<Arc<RuntimeRegistry>>,
    
    // === Optional - Persistence ===
    pub session_store: Option<Arc<SessionStore>>,
    pub todo_store: Option<TodoStore>,
    pub goal_store: Option<Arc<GoalStore>>,
    pub cost_store: Option<Arc<CostStore>>,
    
    // === Optional - Lifecycle ===
    pub cancel_token: Option<CancellationToken>,
}

impl AgentContext {
    /// 从旧版 ExecutionParams + StreamParams 构建（兼容层使用）
    pub fn from_params(exec: ExecutionParams<'_>, stream: StreamParams) -> Self {
        Self {
            config: exec.config.clone(),
            request: exec.request.clone(),
            tool_registry: exec.tool_registry.clone(),
            tx: Some(stream.tx),
            llm_override: exec.llm_override.clone(),
            subagent_prompt: exec.subagent_prompt.clone(),
            mode_state: exec.mode_state.clone(),
            orchestrator: Some(stream.orchestrator),
            interaction_handle: stream.interaction_handle,
            approval_strategy: stream.approval_strategy,
            runtime_registry: stream.runtime_registry,
            session_store: exec.session_store.clone(),
            todo_store: exec.todo_store.clone(),
            goal_store: exec.goal_store.clone(),
            cost_store: exec.cost_store.clone(),
            cancel_token: None,
        }
    }
    
    /// 精简构建（用于 SubAgent / 测试）
    pub fn minimal(config: AgentConfig, request: ChatRequest, tool_registry: Arc<ToolRegistry>) -> Self {
        Self {
            config, request, tool_registry,
            tx: None,
            llm_override: None, subagent_prompt: None,
            mode_state: None, orchestrator: None, interaction_handle: None,
            approval_strategy: ApprovalStrategy::AutoApprove,
            runtime_registry: None,
            session_store: None, todo_store: None, goal_store: None, cost_store: None,
            cancel_token: None,
        }
    }
}
```

### Stream Composition Pattern

```rust
impl AgentRuntime {
    pub fn execute_as_stream(&self, ctx: AgentContext) -> impl Stream<Item = AgentStep> {
        let runtime = self.clone();
        async_stream::stream! {
            let cancel = ctx.cancel_token.clone()
                .unwrap_or_else(CancellationToken::new);
            
            // Phase: Setup
            // tx 被 clone 给 orchestrator/tools 用于侧路径 emit
            let side_tx = ctx.tx.clone();
            let mut setup = match turn_setup::setup_turn(&ctx, &runtime, side_tx) {
                Ok(s) => s,
                Err(e) => {
                    yield AgentStep::Error {
                        message: e.to_string(),
                        error_code: None,
                        recoverable: false,
                    };
                    return;
                }
            };
            yield AgentStep::TurnStart { turn_id: setup.turn_id.clone(), session_id: None };
            
            // Phase: Agentic loop
            loop {
                // Pre-iteration checks
                match iteration_check::pre_check(&mut setup.state, &setup.deps) {
                    PreCheckResult::FatalError(e) => {
                        yield AgentStep::Error { message: e, error_code: None, recoverable: false };
                        return;
                    }
                    PreCheckResult::Terminate(reason) => {
                        yield AgentStep::TurnEnd {
                            turn_id: setup.turn_id.clone(),
                            reason: TurnEndReason::from(reason),
                            summary: setup.make_summary(),
                        };
                        return;
                    }
                    PreCheckResult::Continue => {}
                }
                
                // LLM call
                let params = llm_call::prepare(&mut setup);
                let stream_result = llm_call::consume(params, &cancel, |delta| {
                    // 实时 yield delta（通过内部 channel 转发给 async_stream）
                    // 注意：async_stream 不支持闭包内 yield，需要收集后统一 yield
                }).await;
                
                // 错误恢复（prompt_too_long, max_output_tokens escalation）
                if let Some(recovery) = stream_result.recovery_action {
                    // 处理恢复逻辑...
                    continue;
                }
                
                // Yield accumulated deltas（batch mode: LLM 完成后统一 yield）
                for delta in stream_result.deltas.drain(..) {
                    yield AgentStep::Delta { delta, reasoning: None };
                }
                if let Some(reasoning) = stream_result.reasoning.take() {
                    yield AgentStep::Delta { delta: serde_json::Value::Null, reasoning: Some(reasoning) };
                }
                
                // Context usage
                yield AgentStep::ContextUsage {
                    used_tokens: setup.state.last_estimated_tokens,
                    limit_tokens: setup.deps.context_window,
                    compressed: false,
                    tokens_saved: None,
                };
                
                // Transition decision
                let has_tools = !stream_result.tool_calls.is_empty();
                if !has_tools {
                    // Terminal path
                    // Run stop hooks (todo_store, goal_store)
                    let hook_result = setup.run_stop_hooks().await;
                    if hook_result.should_continue {
                        continue; // stop hook injected continuation message
                    }
                    yield AgentStep::TurnEnd {
                        turn_id: setup.turn_id.clone(),
                        reason: TurnEndReason::Completed,
                        summary: setup.make_summary(),
                    };
                    return;
                }
                
                // === Tool Round ===
                for call in &stream_result.tool_calls {
                    yield AgentStep::ToolExecuting {
                        turn_id: setup.turn_id.clone(),
                        call_id: call.id.clone(),
                        tool_name: call.name.clone(),
                        args: call.arguments.clone(),
                    };
                }
                
                // 工具执行（侧路径事件如 ToolProgress 由 orchestrator 直接 send 到 tx）
                let outcomes = tool_round::execute(&mut setup, stream_result.tool_calls).await;
                
                for outcome in &outcomes {
                    yield AgentStep::ToolResult {
                        turn_id: setup.turn_id.clone(),
                        call_id: outcome.call_id.clone(),
                        tool_name: outcome.tool_name.clone(),
                        output: outcome.output.clone(),
                        success: outcome.success,
                        metadata: outcome.metadata.clone(),
                    };
                }
                
                // Post-tool processing
                let post_action = post_tool::process(&mut setup, &outcomes);
                match post_action {
                    PostToolAction::PlanApproval => {
                        yield AgentStep::TurnEnd {
                            turn_id: setup.turn_id.clone(),
                            reason: TurnEndReason::PlanApprovalPending,
                            summary: setup.make_summary(),
                        };
                        return;
                    }
                    PostToolAction::ModeChange { from, to } => {
                        yield AgentStep::ModeChange { from, to };
                    }
                    PostToolAction::GoalUpdated { goal_id, status } => {
                        yield AgentStep::GoalUpdated { goal_id, status };
                    }
                    PostToolAction::ForceStopAfterNext => {
                        setup.state.force_stop_after_next = true;
                    }
                    PostToolAction::Continue => {}
                }
                
                // === Tool Round Boundary ===
                // 1. Check abort/cancel
                if cancel.is_cancelled() {
                    yield AgentStep::TurnEnd {
                        turn_id: setup.turn_id.clone(),
                        reason: TurnEndReason::Cancelled,
                        summary: setup.make_summary(),
                    };
                    return;
                }
                
                // 2. Drain MessageQueue (future: Phase 4)
                // let injected = setup.drain_steer_queue();
                // if injected > 0 {
                //     yield AgentStep::SteeringInjected { count: injected, sources };
                // }
                
                // 3. Yield boundary
                yield AgentStep::ToolRoundBoundary { iteration: setup.state.iteration };
            }
        }
    }
}
```

### Compatibility Layer

```rust
pub async fn execute_unified(
    &self,
    config: &AgentConfig,
    request: &ChatRequest,
    tool_registry: &Arc<ToolRegistry>,
    tx: mpsc::Sender<AgentEvent>,
    approval_strategy: ApprovalStrategy,
    llm_override: Option<Arc<dyn LlmProvider>>,
    orchestrator: Arc<ToolOrchestrator>,
    interaction_handle: Option<InteractionHandle>,
    subagent_prompt: Option<String>,
    mode_state: Option<ExecutionModeState>,
    session_store: Option<Arc<SessionStore>>,
    todo_store: Option<TodoStore>,
    goal_store: Option<Arc<GoalStore>>,
) -> Result<TurnSummary> {
    let ctx = AgentContext {
        config: config.clone(),
        request: request.clone(),
        tool_registry: tool_registry.clone(),
        tx: Some(tx.clone()),  // tx 传入 ctx，setup 时 clone 给 orchestrator
        llm_override,
        subagent_prompt,
        mode_state,
        orchestrator: Some(orchestrator),
        interaction_handle,
        approval_strategy,
        runtime_registry: self.cached_runtime_registry.clone(),
        session_store,
        todo_store,
        goal_store,
        cost_store: None,
        cancel_token: None,
    };
    
    use futures::StreamExt;
    let mut stream = std::pin::pin!(self.execute_as_stream(ctx));
    let mut summary = None;
    
    while let Some(step) = stream.next().await {
        if let AgentStep::TurnEnd { summary: ref s, .. } = step {
            summary = Some(s.clone());
        }
        // 主循环事件转为 AgentEvent 并发送到 tx
        for event in step.into_agent_events(&turn_id) {
            let _ = send_stream_event(&tx, event, false).await;
        }
    }
    
    summary.ok_or_else(|| anyhow::anyhow!("stream ended without TurnEnd"))
}
```

### Key Design Constraints

1. **Dual-Channel**: 主循环 yield AgentStep，侧路径直接 send AgentEvent 到 tx。两者共享同一 tx channel。
2. **AgentStep 不入协议层**: AgentStep 定义在 `xiaolin-agent` crate，不暴露给前端/gateway。前端只看到 AgentEvent。
3. **needsFollowUp**: 仅看 tool_use 块存在性决定（不信任 stop_reason）
4. **Stream 取消**: drop stream → CancellationToken → abort LLM call → SpawnReservation RAII drop
5. **Delta 类型**: `serde_json::Value`（与 `AgentEvent::ContentDelta { delta }` 完全一致）
6. **字段名对齐**: 使用 `tool_name`（与 AgentEvent 一致），不是 `name`
7. **TurnEndReason**: 1:1 映射已有 `TerminalReason`，补充 `PlanApprovalPending` 和 `Error`
8. **错误恢复**: 保留 prompt_too_long recovery、stream resume、max_output_tokens escalation
9. **Stop hooks**: TurnEnd 前运行 todo_store/goal_store hooks，可能注入继续消息（loop continue）
10. **Side-path 事件无需 AgentStep**: ToolProgress, ApprovalRequired, GuardianAssessment, AskQuestion, SubAgent* 等由 tools/orchestrator 直接 emit

### Migration from claude-code

| claude-code pattern | XiaoLin equivalent |
|--------------------|--------------------|
| `async function* query()` | `fn execute_as_stream() -> impl Stream<Item=AgentStep>` |
| `yield AssistantMessage` | `yield AgentStep::Delta { delta }` |
| `yield UserMessage(tool_result)` | `yield AgentStep::ToolResult { tool_name, output, .. }` |
| `needsFollowUp = true` | `!stream_result.tool_calls.is_empty()` |
| `state = next; continue` | 隐式 loop continue |
| `return { reason }` | `yield TurnEnd { reason }; return;` |
| `StreamingToolExecutor.getCompletedResults()` | 保留现有 `StreamingToolExecutor` |
| Side emitters (progress, approval) | 侧路径直接 send 到 tx（不经 stream） |

### SteeringInjected 语义（与 message-queue-steering spec 对齐）

`AgentStep::SteeringInjected { count, sources }` 在 Phase 4 Message Queue 实现后激活：
- `count`: 本次 drain 注入的消息数
- `sources`: 消息来源列表（如 `["user_steering", "coordinator_instruction"]`）

Phase 1 中 SteeringInjected 暂不 yield（MessageQueue 未实现），但枚举定义在 Phase 1a 中预留。

---

## ADDED Requirements

### Requirement: execute_as_stream returns composable Stream

`AgentRuntime` SHALL 提供 `execute_as_stream(ctx: AgentContext) -> impl Stream<Item=AgentStep>` 方法，使 agent 执行产出可迭代的事件流。

#### Scenario: Basic LLM turn without tools
- **WHEN** 调用 `execute_as_stream` 且 LLM 返回纯文本（无 tool_calls）
- **THEN** stream 依次 yield `AgentStep::TurnStart` → 多个 `AgentStep::Delta(text)` → `AgentStep::TurnEnd(reason: "completed")`

#### Scenario: LLM turn with tool calls
- **WHEN** LLM 返回包含 tool_calls 的 response
- **THEN** stream yield `Delta` → `ToolExecuting { name, args }` → `ToolResult { name, result }` → 继续下一轮 LLM 调用

#### Scenario: Multi-turn loop with tool rounds
- **WHEN** LLM 连续产出 tool_calls（多轮工具调用）
- **THEN** stream 在每个 tool-round boundary 完成所有 tool results 后再开始下一轮 LLM 调用，保持正确的消息顺序

### Requirement: AgentStep enum covers all execution events

`AgentStep` 枚举 SHALL 覆盖 agent 执行全生命周期事件。

#### Scenario: Event type coverage
- **GIVEN** AgentStep 枚举
- **THEN** 至少包含: `TurnStart`, `Delta(ContentDelta)`, `ToolExecuting { id, name, args }`, `ToolResult { id, name, result, success }`, `TurnEnd { reason, summary }`, `Error(anyhow::Error)`

### Requirement: AgentContext consolidates parameters

所有 `execute_unified` 的 13+ 参数 SHALL 合并为单一 `AgentContext` struct。

#### Scenario: Context construction
- **GIVEN** 调用方需要执行 agent
- **WHEN** 构建 `AgentContext`
- **THEN** 必须提供 `config`, `request`, `tool_registry`；其他字段为 Optional

### Requirement: execute_unified backward compatibility

现有 `execute_unified` API SHALL 保留为兼容层，内部调用 `execute_as_stream` 并 collect。

#### Scenario: Existing callers unchanged
- **GIVEN** gateway/session_bridge 等调用 `execute_unified`
- **WHEN** 重构完成后
- **THEN** 所有现有调用方无需修改，行为不变

#### Scenario: Event forwarding to mpsc channel
- **GIVEN** `execute_unified` 接收 `tx: mpsc::Sender<AgentEvent>`
- **WHEN** 内部 stream 产出 `AgentStep`
- **THEN** 兼容层将每个 step 转换为对应的 `AgentEvent` 并 send 到 tx

### Requirement: Stream cancellation via drop

Stream 被 drop 时 SHALL 优雅终止当前执行。

#### Scenario: Parent drops child stream
- **WHEN** 父级 agent 不再需要子 agent 结果（如超时），drop stream
- **THEN** 内部 LLM 调用被取消（abort），已分配的资源被释放，SpawnReservation 通过 RAII drop 释放 slot

### Requirement: Tool-round boundary is explicit in stream

Stream 在每个 tool round 结束后 SHALL yield 一个 boundary marker，内部按固定顺序执行注入。

#### Scenario: Boundary detection
- **WHEN** 所有 tool results 收集完毕、下一次 LLM 调用之前
- **THEN** stream yield `AgentStep::ToolRoundBoundary`
- **AND** 按以下顺序处理：
  1. 检查 abort/cancel 信号
  2. drain MessageQueue（priority <= Next）
  3. 将 drained messages 作为 user messages 追加到对话历史
  4. yield `AgentStep::SteeringInjected` (如有注入)
  5. 继续下一轮 LLM 调用
- **NOTE** 此顺序参考 claude-code query.ts L1530-1773：abort check → drain queue → inject attachments → update state → continue

### Requirement: Internal state tracks transition reason

Stream 内部 SHALL 维护 transition state，记录为何继续循环。

#### Scenario: Normal tool-round continuation
- **WHEN** LLM 返回 tool_calls，工具执行完毕
- **THEN** 内部 state 记录 `transition_reason: "next_turn"`

#### Scenario: TurnEnd exposes reason
- **WHEN** stream 即将终止
- **THEN** yield `AgentStep::TurnEnd { reason }` 其中 reason 可为: "completed", "max_turns", "cancelled", "error"

### Requirement: needsFollowUp determined by tool_use presence

是否继续循环 SHALL 仅由 LLM response 中是否存在 `tool_use` 块决定。

#### Scenario: No tool_use blocks
- **WHEN** LLM streaming 完成后，response 中无 `tool_use` content block
- **THEN** `needs_follow_up = false`，进入终止路径

#### Scenario: Has tool_use blocks
- **WHEN** response 中存在至少一个 `tool_use` content block
- **THEN** `needs_follow_up = true`，执行工具后继续循环
- **NOTE** 不信任 LLM 的 stop_reason 字段（参考 claude-code: "stop_reason === 'tool_use' is unreliable"）

## MODIFIED Requirements

### Requirement: SpawnController reservation integrates with Stream lifecycle

SpawnReservation SHALL 与 stream 生命周期绑定。

#### Scenario: Reservation released on stream completion
- **WHEN** 子 agent stream 完成（自然结束或被 drop）
- **THEN** SpawnReservation 自动释放 global + session slots
