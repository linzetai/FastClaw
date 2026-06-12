## 0. Bug Fixes（前置）— COMPLETED

- [x] 0.1 在 `useMessageStreamChat.ts` 中添加 `sub_agent_notification` case handler，将数据推入 stream-store
- [x] 0.2 审计 `subagent_prompt` 参数：确认有实际使用方（`build_subagent_prompt_block`），移除 `append_subagent_prompt_to_system` 死代码
- [x] 0.3 移除 `SubAgentManager.default_policy` 的 `#[allow(dead_code)]`，删除该未使用字段
- [x] 0.4 修复 `SubAgentTool` 中 `parent_tx` 为 None 时事件丢失：实现 session_event_senders + task-local 路由
- [x] 0.5 修复 `RuntimeTurnExecutor.subagent_manager` 初始化为 None（builder.rs 创建顺序问题）
- [x] 0.6 修复 `SUBAGENT_SESSION_ID` task-local 无法穿越 `tokio::spawn()` 边界

## 1. Stream-based Agent Loop

### Phase 1a: 定义类型（AgentStep + AgentContext）

- [ ] 1a.1 新建 `crates/xiaolin-agent/src/runtime/agent_step.rs`，定义 `AgentStep` 枚举
  - 变体：TurnStart, Delta, ToolExecuting, ToolResult, ToolRoundBoundary, SteeringInjected, ContextUsage, ContextWarning, ModeChange, GoalUpdated, PlanFileUpdate, TurnEnd, Error
  - Delta 字段: `delta: serde_json::Value`（与 AgentEvent::ContentDelta 一致，非 String）
  - ToolExecuting/ToolResult 字段: `tool_name`（非 `name`，与 AgentEvent 对齐）
  - ContextUsage 字段: `used_tokens, limit_tokens, compressed, tokens_saved`（与 AgentEvent::ContextUsageUpdate 一致）
  - TurnEndReason: Completed, MaxTurns, Cancelled, ContextLimit, BudgetExceeded, ConsecutiveErrors, DiminishingReturns, PlanApprovalPending, Error（与 TerminalReason 对齐）
  - 实现 `AgentStep::into_agent_events(turn_id) -> Vec<AgentEvent>`（部分 step 不产生 event，如 ToolRoundBoundary）
  - 实现 `AgentStep::is_lossy() -> bool`
- [ ] 1a.2 新建 `crates/xiaolin-agent/src/runtime/agent_context.rs`，定义 `AgentContext` struct
  - Required: config, request, tool_registry
  - Streaming: tx (Option, 侧路径 clone 给 orchestrator/tools)
  - Optional: llm_override, subagent_prompt, mode_state, orchestrator, interaction_handle, runtime_registry, session_store, todo_store, goal_store, cost_store, cancel_token
  - Non-optional with default: approval_strategy (default AutoApprove)
  - 提供 `AgentContext::from_params(exec: ExecutionParams, stream: StreamParams)` 构造方法
  - 提供 `AgentContext::minimal(config, request, tool_registry)` 精简构造
- [ ] 1a.3 在 `crates/xiaolin-agent/src/runtime/mod.rs` 中声明新模块并导出
- [ ] 1a.4 `cargo clippy -- -D warnings` 零警告

### Phase 1b: 抽取阶段函数（重构，行为不变）

- [ ] 1b.1 新建 `crates/xiaolin-agent/src/runtime/turn_setup.rs`
  - 抽取 lines 865-1153 为 `pub(crate) fn setup_turn(ctx: &AgentContext, runtime: &AgentRuntime) -> TurnSetup`
  - `TurnSetup` struct 包含：messages, tool_defs, state, deps, services, dispatcher, streaming_executor_config
- [ ] 1b.2 新建 `crates/xiaolin-agent/src/runtime/iteration_check.rs`
  - 抽取 lines 1155-1370 为 `pub(crate) fn pre_iteration_check(state, deps, tx) -> PreCheckResult`
  - `PreCheckResult` 枚举：Continue, FatalError, BlockingLimit
- [ ] 1b.3 新建 `crates/xiaolin-agent/src/runtime/llm_call.rs`
  - 抽取 lines 1385-1483 为 `pub(crate) fn prepare_llm_params(...) -> CompletionParams`
  - 抽取 lines 1484-1865 为 `pub(crate) async fn consume_llm_stream(...) -> StreamConsumeResult`
  - `StreamConsumeResult` 包含：accumulated_content, accumulated_reasoning, tool_call_accum, usage, errored, force_stop, finish_reason, withheld_ptl
- [ ] 1b.4 新建 `crates/xiaolin-agent/src/runtime/tool_round.rs`
  - 抽取 lines 2327-2483 (dispatch) + 2485-2794 (per-tool) 为 `pub(crate) async fn execute_tool_round(...) -> ToolRoundResult`
  - `ToolRoundResult` 包含：tool_messages, events_to_emit, post_actions (micro-compact, mode-change, goal-update)
- [ ] 1b.5 新建 `crates/xiaolin-agent/src/runtime/post_tool.rs`
  - 抽取 lines 2796-3019 为 `pub(crate) fn post_tool_process(...) -> PostToolAction`
  - `PostToolAction` 枚举：Continue, PlanApprovalPending, ForceStopLoop, GoalCancelled
- [ ] 1b.6 重写 `execute_stream_inner` 使其调用以上函数，验证行为不变
- [ ] 1b.7 `cargo test` 全部通过 + `cargo clippy -- -D warnings` 零警告

### Phase 1c: 实现 execute_as_stream（核心变更）

- [ ] 1c.1 添加 `async-stream = "0.3"` 依赖到 `crates/xiaolin-agent/Cargo.toml`
- [ ] 1c.2 实现 `AgentRuntime::execute_as_stream(ctx: AgentContext) -> impl Stream<Item=AgentStep>`
  - 使用 `async_stream::stream!` 宏
  - Stream 内部结构：setup -> loop { pre_check -> llm_stream -> transition -> tool_round -> boundary }
  - 内部持有 `CancellationToken`，stream drop 时触发 cancel
- [ ] 1c.3 实现 tool-round boundary 顺序：
  1. 检查 abort/cancel
  2. drain MessageQueue (priority <= Next)
  3. 注入 drained messages 到对话历史
  4. yield `AgentStep::SteeringInjected`（如有）
  5. yield `AgentStep::ToolRoundBoundary`
  6. 继续下一轮 LLM 调用
- [ ] 1c.4 实现 Stream 取消语义：drop stream -> cancel internal token -> abort LLM call -> release SpawnReservation
- [ ] 1c.5 单元测试：mock LLM provider，验证 stream yield 顺序正确

### Phase 1d: 兼容层桥接（外部 API 不变）

- [ ] 1d.1 重写 `execute_unified` 内部实现：构建 AgentContext → pin stream → poll → 转发 AgentEvent
- [ ] 1d.2 重写 `execute_stream` 同理
- [ ] 1d.3 验证所有调用方（gateway state, session_bridge, subagent_manager）零修改
- [ ] 1d.4 保留 `execute_stream_inner` 为 `#[deprecated]` 备用，待 1e 验证后删除

### Phase 1e: 全量回归验证 + 清理

- [ ] 1e.1 `cargo test --workspace` 全部通过
- [ ] 1e.2 `cargo clippy -- -D warnings` 零警告
- [ ] 1e.3 移除 `#[allow(clippy::too_many_arguments)]` 注解
- [ ] 1e.4 删除旧 `execute_stream_inner` 函数
- [ ] 1e.5 E2E 测试矩阵：
  - 普通对话（纯文本回复，TurnStart → Delta* → TurnEnd）
  - 单轮工具调用（ToolExecuting → ToolResult → Delta → TurnEnd）
  - 多轮工具调用（多次 ToolRoundBoundary）
  - SubAgent 触发 + 事件流传递
  - Context compact 触发（大量消息后自动 compact）
  - Stream 取消（发送后立即 stop，验证无泄露）
- [ ] 1e.6 提交最终 commit

## 2. Sidechain Transcript

- [ ] 2.1 创建 `crates/xiaolin-agent/src/sidechain.rs` 模块：`SidechainWriter` struct（path, BufWriter）
- [ ] 2.2 实现 `SidechainWriter::new(session_dir, run_id)` — 创建目录 + 写入 metadata header
- [ ] 2.3 实现 `SidechainWriter::append(message)` — 序列化为 JSON line 并 flush
- [ ] 2.4 实现 `SidechainReader::load(session_dir, run_id)` — 读取 JSONL 还原消息列表
- [ ] 2.5 在 `SubAgentManager::run_subagent()` 中创建 SidechainWriter，child event 持久化前 forward
- [ ] 2.6 实现 result extraction：子 agent 完成时取最后 assistant 消息（截断 4096 chars）
- [ ] 2.7 新增 `resume_subagent` 工具：读取 sidechain → 构建 initial messages → 继续执行
- [ ] 2.8 在 session 删除逻辑中添加 sidechains 目录清理

## 3. Fork Agent

- [ ] 3.1 在 `SubAgentTool::execute()` 中解析 `inherit_context` 参数
- [ ] 3.2 实现 `filter_parent_messages(session_store, max_messages, max_tokens)` 函数
- [ ] 3.3 过滤逻辑：移除 system messages、incomplete tool_calls，限制条数和 token 数
- [ ] 3.4 将 filtered messages 作为 child agent 的 initial context prefix
- [ ] 3.5 在 SubAgentDef 中添加 `max_context_messages` 可选字段（默认 20）

## 4. Message Queue + SendMessage

- [ ] 4.1 创建 `crates/xiaolin-agent/src/message_queue.rs`：定义 `Priority` enum 和 `MessageQueue` struct
- [ ] 4.2 实现 `MessageQueue::push(priority, source, message)` 和 `drain(max_priority) -> Vec<QueuedMessage>`
- [ ] 4.3 在 `AgentContext` 中添加 `message_queue: Option<Arc<MessageQueue>>` 字段
- [ ] 4.4 在 `execute_as_stream` 的 ToolRoundBoundary 处添加 drain + inject 逻辑
- [ ] 4.5 创建 `SendMessageTool` struct，实现 Tool trait（查找目标 run 的 queue → push）
- [ ] 4.6 在 SubAgentManager 中维护 `run_queues: DashMap<String, Arc<MessageQueue>>`
- [ ] 4.7 定义 `AgentStep::SteeringInjected` 变体 + 对应的 `AgentEvent::SteeringMessage`
- [ ] 4.8 在 gateway WebSocket handler 中支持前端 `steering_message` 命令 → push 到 queue

## 5. Permission Bubble

- [ ] 5.1 在 `xiaolin-core` 中定义 `PermissionMode` enum（AutoApprove, Bubble, Deny）
- [ ] 5.2 在 `SubAgentDef` 中添加 `permission_mode` 字段（默认 AutoApprove）
- [ ] 5.3 定义 `ApprovalStrategy::ParentApproval(oneshot::Sender<ApprovalResult>)` 变体
- [ ] 5.4 在 `SubAgentManager::run_subagent()` 中根据 permission_mode 构建对应的 ApprovalStrategy
- [ ] 5.5 定义 `AgentEvent::ApprovalBubble { run_id, tool_name, args_preview, respond_tx }` 变体
- [ ] 5.6 实现 30s timeout logic：tokio::select! approval_rx vs sleep(30s)
- [ ] 5.7 在 gateway WebSocket handler 中转发 approval_bubble → 前端
- [ ] 5.8 在 gateway 中实现 `approval_respond` 命令 → 通过 saved respond_tx 回复
- [ ] 5.9 管理 pending approvals map：`DashMap<request_id, oneshot::Sender<ApprovalResult>>`

## 6. Coordinator Mode

- [ ] 6.1 在 SubAgentDef 中添加 `mode` 字段（Normal / Coordinator）
- [ ] 6.2 实现 coordinator tool registry filter：仅允许 spawn_subagent, send_message, task_stop, subagent_list, subagent_get
- [ ] 6.3 创建 `TaskStopTool` struct（coordinator 主动结束编排）
- [ ] 6.4 在 coordinator 模式下 force worker spawn 为 background=true
- [ ] 6.5 worker 完成时将 CompletionSummary 格式化并 push 到 coordinator 的 MessageQueue
- [ ] 6.6 创建 `coordinator_system_prompt.txt` 默认编排指引
- [ ] 6.7 创建 builtin coordinator SubAgentDef（id="coordinator", mode=Coordinator）
- [ ] 6.8 集成测试：coordinator spawn 多个 worker → 收到 notifications → 综合输出

## 7. Markdown Agent Definitions

- [ ] 7.1 实现 `parse_agent_markdown(path) -> Result<SubAgentDef>` 函数（frontmatter YAML + body）
- [ ] 7.2 实现 `load_agents_from_dir(dir) -> Vec<SubAgentDef>` 函数
- [ ] 7.3 在 `SubAgentManager::new()` 中按优先级加载：builtin → `~/.xiaolin/agents/` → `{project}/.xiaolin/agents/`
- [ ] 7.4 实现 merge 逻辑：同 id 后者覆盖前者
- [ ] 7.5 添加 frontmatter schema 验证（required fields check + type validation）
- [ ] 7.6 处理无效文件：跳过 + warning 日志
- [ ] 7.7 实现 hot-reload：file watcher 监听 agents 目录变更 → 重新加载
- [ ] 7.8 更新 `ListAgentsTool` 输出包含 source 信息（builtin/user/project）

## 8. Frontend Interaction

- [ ] 8.1 在 stream-store 中添加 `notifications` 数组到 SubAgentRunUI
- [ ] 8.2 实现 `sub_agent_notification` handler 更新 store
- [ ] 8.3 在 SubAgentMonitor 中显示 notification feed
- [ ] 8.4 在 SubAgentCard 中添加 cancel 按钮（running 状态时显示）
- [ ] 8.5 在 SubAgentCard 展开态添加 steering 输入框（running 状态时显示）
- [ ] 8.6 实现 steering input → WebSocket `steering_message` 发送
- [ ] 8.7 创建 `ApprovalBubbleCard` 组件（tool_name, args_preview, Approve/Deny 按钮）
- [ ] 8.8 处理 `approval_bubble` WebSocket 事件 → 渲染 ApprovalBubbleCard
- [ ] 8.9 实现 Approve/Deny 按钮 → 发送 `approval_respond` + 更新卡片状态
- [ ] 8.10 处理 `approval_resolved` 事件（timeout/外部 resolve）→ 更新卡片状态
- [ ] 8.11 创建 CoordinatorPanel 组件（worker 列表 + 状态 + activity）
- [ ] 8.12 在 WorkspacePanel 中根据 coordinator run 存在与否显示/隐藏 Coordinator tab

## 9. 验证与清理

- [ ] 9.1 `cargo check` 全 workspace 通过
- [ ] 9.2 `cargo clippy -- -D warnings` 零警告
- [ ] 9.3 确认无 `#[allow(dead_code)]` 新增
- [ ] 9.4 `pnpm exec tsc --noEmit` 前端类型检查通过
- [ ] 9.5 现有 subagent 相关测试适配并通过
- [ ] 9.6 新增单元测试覆盖: MessageQueue, SidechainWriter/Reader, parse_agent_markdown, coordinator tool filter
