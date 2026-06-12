## Relay Backpressure

### Bug 13: relay task 用 try_send 静默丢弃事件

**现状**: `session-actor` 的 relay task 用 `tx.try_send()` 转发事件给订阅者。当 channel buffer 满时，事件被静默丢弃（`let _ =`），违背了 `EventFanout` 的 `BackpressurePolicy::Block` 设计。

**要求**:
- [ ] 将 `try_send` 改为 `send().await`，在背压时阻塞等待
- [ ] 为 `send` 添加合理的超时兜底（建议 5-10 秒），超时后记录 warn 日志并丢弃
- [ ] 事件丢弃时必须有日志记录，不可完全静默
