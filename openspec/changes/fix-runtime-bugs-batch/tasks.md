## 1. 工具执行时序修复（Bug 1 + Bug 10）

- [ ] 1.1 在 `tool_round.rs` streaming 路径中，在工具分发（`dispatch_batch` / `executor.add_tool`）之前，遍历 `assembled_calls` 提取文件路径，构建 `HashMap<PathBuf, (Option<String>, bool)>` 预快照映射（content, file_exists）
- [ ] 1.2 修改 `for stream_results` 循环中的 undo 和 file_tracker 逻辑，改为从预快照映射查表，而非实时读取文件
- [ ] 1.3 添加单元测试：验证 `create_file` 时 `FileOp` 为 `Created`（而非 `Modified`）
- [ ] 1.4 添加单元测试：验证 undo 快照捕获的是修改前的文件内容

## 2. 事件传播修复（Bug 2）

- [ ] 2.1 在 `session_bridge.rs` 的 `injector.abort()` 后的错误处理路径中，将 `inner_tx.send(GoalUpdated)` 改为使用外层 `tx`（执行方法参数传入的 sender）
- [ ] 2.2 添加测试：模拟 turn 取消场景，验证 GoalUpdated 事件能被正确接收

## 3. Reactive Loop 目标追踪修复（Bug 3）

- [ ] 3.1 在 `session_bridge.rs` reactive loop 中将 `execute_unified_with_cost_store` 的 `goal_store` 参数从 `None` 改为 `self.goal_store.clone()`
- [ ] 3.2 Review：确认 `goal_store` 的 `Arc` 引用在 reactive loop 的多次迭代中安全共享

## 4. 流式恢复上下文清理（Bug 4）

- [ ] 4.1 在 `llm_call.rs` 的 stream resume 逻辑中，push partial Assistant message 之前检查 `ms.messages` 末尾是否存在上一次 resume 推入的 partial message，如果是则 pop 掉
- [ ] 4.2 添加识别条件：role=Assistant、tool_calls 为空/None、由 resume 推入（可用 metadata 标记或位置追踪）
- [ ] 4.3 添加单元测试：模拟 3 次 stream resume，验证 `ms.messages` 中只保留最新一条 partial Assistant message（而非累积 3 条）

## 5. 迭代状态重置（Bug 5）

- [ ] 5.1 在 `turn_loop.rs` 的 `loop` 体最顶部（Phase 0 之前）添加 `ms.had_tool_calls_this_round = false`
- [ ] 5.2 Review `end_turn.rs` 中 `evaluate_stop_hooks` 对 `had_tool_calls_this_round` 的使用，确认重置不破坏语义
- [ ] 5.3 添加测试：验证纯文本回复迭代后 `had_tool_calls_this_round` 为 false

## 6. Model Switch Reminder 位置修复（Bug 6）

- [ ] 6.1 在 `mod.rs` 中将 `messages.insert(last_user_idx, ...)` 改为 `messages.insert(last_user_idx + 1, ...)`
- [ ] 6.2 添加测试：验证 reminder 消息在最后一条 User 消息之后

## 7. Streaming 工具结果匹配修复（Bug 7）

- [ ] 7.1 在 `tool_round.rs` 中将 `completed` 转换为 `HashMap<String, (String, ToolResult)>`（key=call_id）
- [ ] 7.2 修改 streaming results 匹配逻辑，改为按 `tc.id` 从 HashMap 查表
- [ ] 7.3 添加 fallback：当 call_id 查不到时 warn 日志并 fallback 到位置匹配
- [ ] 7.4 添加 `debug_assert` 验证 completed 中的 call_id 在 assembled_calls 中存在

## 8. Reactive Target Tokens 计算修复（Bug 8）

- [ ] 8.1 在 `turn_setup.rs` 中将 `reactive_target_tokens` 从 `context_window as usize` 改为 `(context_window as f64 * 0.8) as usize`
- [ ] 8.2 Review `PipelineConfig` 中 `reactive_target_tokens` 的下游使用，确认 80% 比例合理

## 9. PermissionSelector 中途切换生效（Bug 9）

- [ ] 9.1 为 `ApprovalCache` 添加 `pub fn set_global_approved(&mut self, value: bool)` 方法
- [ ] 9.2 在 `session_behavior_overrides` 变更触发时（当新预设为 "full-auto"），查找当前活跃 turn 的 `ApprovalCache` 并设置 `global_approved = true`
- [ ] 9.3 添加测试：模拟 mid-turn preset 切换到 full-auto，验证后续工具调用自动批准

## 10. 编译与验证

- [ ] 10.1 运行 `cargo check` 确认编译通过
- [ ] 10.2 运行 `cargo test -p xiaolin-agent` 确认所有测试通过
- [ ] 10.3 运行 `cargo clippy -- -D warnings` 确认零警告
