# 设计决策

## D1: SessionManager GC 策略

**决策**：在 gateway 的 `start_background_tasks()` 中 spawn 一个 GC loop，间隔 60s：
1. 调用 `session_manager.gc()` 清理 dead sessions
2. 收集当前 active session IDs
3. 用 active set 清理 `chat_locks`、`chat_model_overrides` 中的孤立条目

**理由**：60s 足够低频不影响性能，又不会让 dead sessions 堆积太久。

## D2: Streaming 输出安全阀

**决策**：
- `MAX_ASSISTANT_CONTENT_BYTES = 2 * 1024 * 1024`（2MB，约 50 万汉字）
- `MAX_TURN_DURATION_SECS = 600`（10 分钟）
- 任一条件触发时，cancel 当前 turn 并 emit Error event

**理由**：正常对话不会超 2MB。10min 是防止 LLM 无限 streaming 的兜底。

## D3: DashMap 清理方式

**决策**：不使用独立 TTL 机制，而是复用 SessionManager GC 周期。GC 时：
- 遍历 `chat_locks`，移除 key 不在 active sessions 中的条目
- `stream_event_tx` 移除 sender 已 closed 的条目
- `chat_model_overrides` 移除不在 active sessions 中的条目

**理由**：避免引入新的定时器和复杂度。一个 GC 任务统一清理。

## D4: 内存监控

**决策**：
- 在 GC loop 中顺带读取进程 RSS（`sysinfo` crate 或直接读 `/proc/self/status`）
- RSS > 1GB 时 warn 日志
- RSS > 4GB 时 error 日志
- 不做自动 restart（留给用户/运维决策）

**macOS 实现**：使用 `mach_task_basic_info` 获取 `resident_size`。

**理由**：轻量级监控，不依赖外部 crate，能在问题升级前给出警告。
