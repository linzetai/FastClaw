## Technical Approach

本批次共 10 个 Bug，按模块分组设计修复方案。

---

### Bug 11: Error 事件后未 break 循环

**文件**: `crates/xiaolin-gateway/src/ws/chat.rs:766-780`

**分析**: `AgentEvent::Error` 处理完后没有 `break`，如果后续还有 `TurnEnd` 事件，`after_chat` 会被再次调用，导致 assistant 消息重复持久化。

**方案**: 在 Error 事件处理的 `after_chat` 调用后添加 `break`（或设置一个 `error_handled` 标记位跳过后续 `TurnEnd` 分支）。选择 `break` 更简洁，因为 Error 之后的事件已无需处理。

**备选**: 用 `error_handled: bool` 标记位取代 `break`，保留后续事件的日志采集能力。但增加复杂度，当前不需要。

---

### Bug 12: response_language 被硬编码 None

**文件**: `crates/xiaolin-gateway/src/chat_pipeline.rs:216`

**分析**: `enriched_request` 构造时将 `response_language` 写死为 `None`，而入参 `request.response_language` 已经携带了用户设置。

**方案**: 改为 `response_language: request.response_language.clone()`。

---

### Bug 13: relay task 用 try_send 静默丢弃事件

**文件**: `crates/xiaolin-session-actor/src/actor.rs:302-304`

**分析**: `try_send()` 在 channel 满时返回 `Err`，但 `let _ =` 忽略了错误。这违背了 `EventFanout` 中 `BackpressurePolicy::Block` 的设计意图。

**方案A (推荐)**: 将 `try_send` 改为 `send().await`，使 relay task 在背压时阻塞等待。需把 `for tx in &senders` 循环改成逐个 await。

**方案B**: 增大 channel buffer（当前默认 16），同时对 `try_send` 失败记录 warn 日志。

**选择**: 方案 A。relay task 已经在独立 spawn 中，阻塞不影响主循环。

---

### Bug 14: GoalStore set_session_id 不重置 idle_rounds/budget_warning

**文件**: `crates/xiaolin-agent/src/builtin_tools/goal.rs:160-167`

**分析**: `reset_accounting()` 只重置了 token/time 计数，但 `idle_rounds` 和 `budget_warning_sent` 未重置。切换 session 后这些值会错误携带到新 session。

**方案**: 在 `reset_accounting()` 中增加 `idle_rounds.store(0)` 和 `budget_warning_sent.store(false)`。

---

### Bug 15: turn 10 分钟超时不通知前端

**文件**: `crates/xiaolin-gateway/src/ws/chat.rs:667-676`

**分析**: 超时后只 `tracing::error` + `turn_cancel.cancel()`，前端不知道原因就看到流结束。

**方案**: 在 `break` 前通过 `ws_sender` 发送一个错误消息（如 `AgentEvent::Error` 或自定义 WS 消息），告知前端 "turn 超时被取消"。

---

### Bug 16: ensure_system_messages 只比数量不比内容

**文件**: `crates/xiaolin-context/src/reactive.rs:98-121`

**分析**: 只检查 `existing_sys.len() >= original_system.len()`，如果压缩器生成了等量但内容不同的 system 消息，原始 system prompt 会丢失。

**方案**: 用 content 做相等性检查。对每个 `original_system` 消息，检查 `compacted` 中是否存在相同 `content` 的 system 消息。如果缺失，重新插入。

**权衡**: content 比较可能有性能开销，但 system 消息通常 ≤3 条，影响可忽略。

---

### Bug 17: handleRecallLastMessage 不处理 multimodal content

**文件**: `crates/xiaolin-app/src/components/message-stream/StreamFooter.tsx:131-135`

**分析**: 直接返回 `item.data.content`，当 content 为 `Array`（multimodal 消息）时，显示为 `[object Object]`。

**方案**: 如果 `content` 是数组，遍历找第一个 `type === "text"` 的条目，提取其 `.text` 属性；如果没有文本条目则返回空字符串。

---

### Bug 18: GoalStore 全局单例多 session 竞态

**文件**: `crates/xiaolin-gateway/src/ws/session.rs:326-351`

**分析**: `GoalStore` 是全局共享的，`set_session_id` 修改内部 session_id。多 session 并发时互相覆盖。

**方案A (推荐)**: 将 `GoalStore` 改为 per-session 实例（在 `SessionActor` 内持有独立的 `GoalStore`）。

**方案B**: 改 GoalStore 内部为 `DashMap<SessionId, GoalState>`，按 session 隔离状态。

**选择**: 方案 A 更简洁，GoalStore 本身就是轻量结构体。

---

### Bug 19: MCP prompt 中 `&name[4..]` 可能 panic

**文件**: `crates/xiaolin-gateway/src/chat_pipeline.rs:657-659`

**分析**: `&name[4..]` 假设所有 MCP 工具名以 "mcp_" 开头且长度 ≥ 4。如果不满足，会 slice out of bounds panic。

**方案**: 使用 `name.strip_prefix("mcp_").unwrap_or(name)` 替代裸切片。

---

### Bug 20: turn 取消时持久化不完整 assistant 消息

**文件**: `crates/xiaolin-gateway/src/ws/chat.rs:693-708`

**分析**: `turn_cancel.is_cancelled()` 为 true 时仍调用 `after_chat` 保存 `assistant_content`。该内容可能是流式传输中途的不完整响应。

**方案**: 在 `after_chat` 调用前检查 `turn_cancel.is_cancelled()`：
- 如果已取消且 `assistant_content` 非空，在末尾追加 `\n\n[此回复因超时被截断]` 标记
- 或者完全不持久化，但这可能导致上下文丢失

**选择**: 追加截断标记。让 LLM 在后续 turn 中知道上一轮回复不完整。

---

## Risks

| 风险 | 影响 | 缓解 |
|------|------|------|
| Bug 18 (GoalStore per-session) 需要改动 SessionActor 构造 | 中 | 保持 GoalStore 接口不变，只改注入点 |
| Bug 13 (try_send → send) 可能在慢消费者时导致 relay task 阻塞 | 低 | 添加 send 超时兜底 |
| Bug 16 (content 比较) 大 system prompt 时有性能开销 | 极低 | system 消息通常 ≤ 3 条 |

## Dependencies

- Bug 18 与 Bug 14 有关联——如果 GoalStore 改为 per-session，Bug 14 的 `set_session_id` 重置问题自然消解，但仍需确保 `reset_accounting` 完整。
- 其余 Bug 互不依赖，可独立修复。
