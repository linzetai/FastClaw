## Context

XiaoLin agent runtime（`xiaolin-agent` crate）经代码审查发现 10 个 Bug，分布在工具执行、流式恢复、状态管理、事件传播、上下文管理等多个子系统。这些 Bug 均在 `crates/xiaolin-agent/src/runtime/` 和 `crates/xiaolin-agent/src/session_bridge.rs` 中。

**当前架构关键路径**：`session_bridge::execute_as_stream` → `turn_setup` → `turn_loop` → `(iteration_check → llm_call → tool_round → post_tool → end_turn)` 循环。

## Goals / Non-Goals

**Goals:**
- 修复全部 10 个已识别的 Bug，确保每个修复都有对应的测试覆盖
- 所有修改通过 `cargo check` + `cargo clippy -- -D warnings` + `cargo test` 零错误零警告
- 不改变现有 public API 签名

**Non-Goals:**
- 不做大规模架构重构（如重写 streaming 执行器）
- 不新增功能
- 不修改非 `xiaolin-agent` crate 的代码（除非 Bug 修复要求跨 crate 改动）

## Decisions

### D1: Undo 快照时序修复（Bug 1 + Bug 10）

**问题**：streaming 路径下，工具先执行再进入 `for stream_results` 循环捕获快照，此时文件已被修改。`create_file` 的 `file_exists` 也在执行后判断，导致新文件被错误标记为 `FileOp::Modified`。

**方案**：将 undo 快照和 file_exists 预计算提前到工具调度之前。在 `tool_round.rs` 的 streaming 路径中，遍历 `assembled_calls` 提取文件路径，构建 `HashMap<PathBuf, (Option<String>, bool)>` 预快照映射，在分发工具前完成文件内容读取和存在性判断。后续循环直接查表。

**替代方案**：在 `StreamingToolExecutor.add_tool` 中加 hook —— 但这会侵入工具执行器的抽象层，不如在调用方统一处理。

### D2: GoalUpdated 事件丢失修复（Bug 2）

**问题**：`session_bridge.rs:772` 调用 `injector.abort()` 后关闭了 channel，后续 `inner_tx.send(GoalUpdated)` 必然失败。

**方案**：使用外层 `tx`（执行方法参数传入的 `mpsc::Sender`）发送 GoalUpdated 事件。`inner_tx` 与 injector 生命周期绑定，abort 后不可用；但外层 `tx` 在整个 `execute_as_stream` 作用域内存活。

**替代方案**：延迟 `injector.abort()` 到事件发送之后 —— 可行但增加 injector 悬挂时间，不如切换 sender。

### D3: Reactive loop goal_store 传递（Bug 3）

**问题**：`session_bridge.rs:438` 传 `None` 给 `goal_store` 参数。

**方案**：改为 `self.goal_store.clone()`。由于 `goal_store` 是 `Option<Arc<GoalStore>>`，clone 开销极低。

### D4: 流式恢复上下文堆积（Bug 4）

**问题**：每次 stream resume 将 partial content 推入 `ms.messages`，多次重试后累积多条不完整 Assistant 消息。

**方案**：在重试前检查 `ms.messages` 末尾是否为 partial Assistant 消息（无 tool_calls），如果是则 pop 掉再推入新的 partial。用一个 `last_partial_push_idx: Option<usize>` 追踪上一次推入的位置，resume 时先移除。

**替代方案**：在每次 resume 前清空所有非用户消息 —— 过于激进，会丢失有效的 assistant 历史。

### D5: `had_tool_calls_this_round` 标志残留（Bug 5）

**问题**：该标志在工具执行后设置为 true，但在下一次循环迭代开始时不被重置。当 LLM 在后续迭代中纯文本回复（无 tool calls），`end_turn` 仍看到 `true`。

**方案**：在 `turn_loop.rs` 的 `loop` 体最顶部（Phase 0 cancellation check 之前），重置 `ms.had_tool_calls_this_round = false`。这确保每次迭代开始时为 false，只有当前迭代有工具调用才设 true。

### D6: Model Switch Reminder 位置（Bug 6）

**问题**：`messages.insert(last_user_idx, ...)` 插在最后一条 user message **之前**，model 提醒被挤到了 user 消息上方。

**方案**：改为 `messages.insert(last_user_idx + 1, ...)`，将提醒插在 user message 之后，使其位于 LLM 即将生成回复的上下文末端。

### D7: Streaming 工具结果错位（Bug 7）

**问题**：`tool_round.rs:200-209` 用迭代器顺序匹配 streaming 结果到 `assembled_calls` 位置，假设 `drain_remaining()` 返回的顺序与提交顺序一致。但并发执行时结果可能按完成顺序返回。

**方案**：改为按 `call_id` 匹配。将 `completed` 转换为 `HashMap<String, (String, ToolResult)>`（key=call_id），然后在遍历 `assembled_calls` 时用 `tc.id` 查表。

**替代方案**：在 `StreamingToolExecutor` 内部保证输出顺序 —— 但这需要修改执行器实现，且会降低并发性能。

### D8: `reactive_target_tokens` 未预留空间（Bug 8）

**问题**：`turn_setup.rs:195` 设置 `reactive_target_tokens = context_window`，等于全窗口大小。

**方案**：计算 `reactive_target_tokens = context_window * 80%`（留 20% 给输出 tokens 和 tool definitions）。使用比例而非精确计算，因为 tool definitions 的 token 数难以在 setup 阶段精确估算。

### D9: PermissionSelector 切换预设不生效（Bug 9）

**问题**：`approval_strategy` 在 turn 开始时固定，mid-turn 通过 `PermissionSelector` 切换预设后不影响当前 turn。

**方案**：利用已实现的 `ApprovedAllForSession` + `global_approved` 机制。当 `PermissionSelector` 切换到 "full-auto" 预设时，直接在 `ApprovalCache` 上设置 `global_approved = true`。由于 `ApprovalCache` 在 `TurnMutableState` 中按引用传递，修改立即生效。需要添加 `ApprovalCache::set_global_approved()` public 方法，并在 `session_behavior_overrides` 变更时触发。

**替代方案**：让 `approval_strategy` 变为 `Arc<RwLock<ApprovalStrategy>>` 动态读取 —— 改动范围大，且已有 global_approved 机制可以覆盖。

### D10: 无额外架构决策

Bug 6（位置修复）和 Bug 8（比例计算）为局部单行修改，无需额外设计决策。

## Risks / Trade-offs

- **[Risk] Bug 7 call_id 匹配依赖 StreamingToolExecutor 返回的 call_id 正确性** → Mitigation: 添加 debug_assert 验证 completed 中的 call_id 在 assembled_calls 中存在；缺失时 fallback 到位置匹配并 warn 日志。
- **[Risk] Bug 5 重置时机可能影响 stop hooks 原有行为** → Mitigation: 仔细审查 `evaluate_stop_hooks` 对 `had_tool_calls_this_round` 的依赖语义，确保"当前迭代"语义正确。
- **[Risk] Bug 4 pop 操作可能误删有效消息** → Mitigation: 只 pop 满足条件的消息（role=Assistant, tool_calls 为空, 且由 resume 推入标记）。
- **[Risk] Bug 9 依赖现有 global_approved 机制的正确性** → Mitigation: Bug 9 修复建立在前序已验证的 ApprovedAllForSession 实现之上。
- **[Trade-off] Bug 1/10 预快照增加一次文件系统读取** → 对于文件编辑工具这是必要的正确性开销，且通常是小文件。
