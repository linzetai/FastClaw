## Tasks

### Group 1: Event Loop 修复 (Bug 11, 15, 20)

- [x] **Bug 11** — `ws/chat.rs:766-780`：Error 事件处理的 `after_chat` 调用后添加 `break` 退出事件循环，防止后续 TurnEnd 重复调用 `after_chat`
- [x] **Bug 15** — `ws/chat.rs:667-676`：turn 超时 `break` 前，通过 `bg_tx` 发送错误消息通知前端，包含超时原因
- [x] **Bug 20** — `ws/chat.rs` 事件循环后：在 `turn_cancel.is_cancelled()` 且 `assistant_content` 非空时，追加截断标记 `\n\n[此回复因超时被截断]` 再调用 `after_chat`

### Group 2: Goal Store 隔离 (Bug 14, 18)

- [x] **Bug 18** — 将 `GoalStore` 内部状态改为 per-session 隔离：使用 `DashMap<String, Arc<SessionGoalState>>` 替代全局 atomic 字段，保持 API 兼容
- [x] **Bug 14** — `set_session_id` 中重置 `idle_rounds`（通过 per-session state 自然解决）
- [x] 验证 `GoalStore` 构造和 `set_session_id` 在多 session 并发下互不干扰（通过 DashMap 隔离保证）

### Group 3: Relay 背压 (Bug 13)

- [x] **Bug 13** — `actor.rs:302-304`：将 `tx.try_send(session_event.clone())` 改为带 5 秒超时的 `tx.send(session_event.clone()).await`
- [x] 超时后记录 `tracing::warn!` 日志，包含 session_id
- [x] 确认 relay task 在 `tokio::spawn` 内的 async context 支持 `.await`（已验证编译通过）

### Group 4: Context 完整性 (Bug 12, 16, 17, 19)

- [x] **Bug 12** — `chat_pipeline.rs:216`：`response_language: None` 改为 `response_language: request.response_language.clone()`
- [x] **Bug 16** — `reactive.rs:98-121`：`ensure_system_messages` 改为逐条按 content 比对原始 system 消息是否存在于 compacted 中，缺失则重新插入
- [x] **Bug 17** — `StreamFooter.tsx:131-135`：`handleRecallLastMessage` 中判断 `content` 是否为数组，若是则提取第一个 `type === "text"` 条目的 `.text`
- [x] **Bug 19** — `chat_pipeline.rs:657`：`&name[4..]` 改为 `name.strip_prefix("mcp_").unwrap_or(name)`

### 验证

- [x] `cargo check` 编译通过
- [x] `cargo clippy -- -D warnings` 零警告
- [x] `cargo test` 相关 crate 通过（gateway e2e 测试因环境原因跳过，修改前即存在相同失败）
- [x] 前端构建无错误
