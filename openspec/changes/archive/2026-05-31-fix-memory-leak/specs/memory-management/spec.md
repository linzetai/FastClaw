# Memory Management Spec

## Requirements

### REQ-1: Session GC

- Gateway 必须定期清理已停止的 session actors
- 清理间隔 ≤ 60 秒
- 清理时必须同步清理关联的 DashMap 条目（chat_locks, chat_model_overrides, stream_event_tx）
- GC 不得阻塞正常请求处理

### REQ-2: Streaming 输出限制

- 单次 turn 的 assistant 输出不得超过 MAX_ASSISTANT_CONTENT_BYTES（默认 2MB）
- 单次 turn 的执行时间不得超过 MAX_TURN_DURATION_SECS（默认 600s）
- 超出限制时必须：
  - Cancel 当前 turn
  - Emit Error event 通知客户端
  - 保存已产生的部分输出

### REQ-3: DashMap 生命周期管理

- `chat_locks` 中的条目必须在对应 session 死亡后被移除
- `stream_event_tx` 中已 closed 的 sender 必须被清理
- `chat_model_overrides` 中的条目必须在对应 session 死亡后被移除

### REQ-4: 内存监控

- 进程 RSS 超过 1GB 时必须输出 warn 级日志
- 进程 RSS 超过 4GB 时必须输出 error 级日志
- 监控不得引入重量级外部依赖
- 监控数据应通过 /health endpoint 可查

## Non-Requirements

- 不做自动内存限制/OOM kill
- 不做内存泄漏自动定位（依赖 heaptrack/Instruments 等外部工具）
- 不做跨进程内存共享
