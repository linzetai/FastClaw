## Event Loop Fix

### Bug 11: Error 事件后未 break 循环

**现状**: `ws/chat.rs` 事件循环中，`AgentEvent::Error` 处理后调用了 `after_chat` 持久化 assistant 消息，但没有 `break` 退出循环。后续如果收到 `TurnEnd` 事件，`after_chat` 会被再次调用，导致 assistant 消息重复。

**要求**:
- [ ] Error 事件处理完 `after_chat` 后必须 `break` 退出事件循环
- [ ] 或者用 `error_handled` 标记位，在 `TurnEnd` 分支中跳过 `after_chat`
- [ ] 修复后不应产生重复的 assistant 消息

### Bug 15: turn 10 分钟超时不通知前端

**现状**: 超时后只有 `tracing::error` 日志和 `turn_cancel.cancel()`，前端看到的是流突然结束，无任何错误提示。

**要求**:
- [ ] 超时时通过 WS 发送错误通知消息给前端
- [ ] 消息需包含超时原因说明
- [ ] 前端应能正常展示该错误

### Bug 20: turn 取消时持久化不完整消息

**现状**: `turn_cancel.is_cancelled()` 为 true 时，不完整的 `assistant_content` 仍被 `after_chat` 持久化，可能误导后续 LLM 推理。

**要求**:
- [ ] 如果 turn 已取消且 assistant_content 非空，追加截断标记（如 `[此回复因超时被截断]`）
- [ ] 标记应明确告知 LLM 上轮回复不完整
