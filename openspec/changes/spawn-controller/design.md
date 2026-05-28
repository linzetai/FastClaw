## Context

`SubAgentManager` 当前使用 `tokio::sync::Semaphore(max_parallel)` 作为唯一的并发控制原语。Semaphore 在 `new()` 时从 `SubAgentPolicy.max_parallel`（默认 5）初始化，在 `spawn()` 的 `tokio::spawn` 闭包内 acquire，任务结束时 RAII 释放。

`SubAgentDef` 已定义 `concurrency_safe: bool` 字段（标记只读 agent），但 spawn 路径未读取此字段。`spawn_sync` 用 100ms 轮询 `DashMap` 检查完成状态。所有 session 共享同一 Semaphore 实例。

竞品调研：Codex 用 `AtomicUsize` CAS + RAII `SpawnReservation`（存在手动释放导致的泄漏问题）；Claude Code 无硬并发限制，用 notification 事件回流结果。

## Goals / Non-Goals

**Goals:**
- 让 `concurrency_safe` 字段生效：只读 agent 可并行，写入 agent 独占
- 用事件驱动替代轮询：sub-agent 完成后零延迟通知
- Per-session 隔离：不同 session 的 sub-agent 不互相阻塞
- RAII slot 管理：杜绝 Codex 式的 slot 泄漏
- 提供 `wait_agent` 工具：LLM 可批量等待多个子 agent
- 实时可观测性：API 暴露并发状态

**Non-Goals:**
- 不做跨机器分布式 slot 控制
- 不做 per-agent-type 精细限额（如"最多 2 个 code agent"）
- 不做优先级队列 / 抢占式调度
- 不修改 `ToolOrchestrator` / `StreamingToolExecutor` 的工具级并行控制
- 不修改 `SubAgentDef` 结构体定义（仅消费已有字段）

## Decisions

### D1: 分层控制而非单一 Semaphore

**选择**: 三层结构 — `GlobalSlotPool` → `SessionSlotPool` → `SpawnReservation`

**替代方案**:
- A) 保留单 Semaphore + 加 RwLock — 简单但无 per-session 隔离
- B) Codex 式 AtomicUsize CAS — 灵活但需手动释放，已知泄漏问题
- C) Claude Code 式无限制 — 不适合资源受限的桌面端

**理由**: 分层结构同时解决全局保护（防 OOM）、session 隔离（公平性）、读写区分（安全性）三个问题。RAII guard 避免 Codex 的泄漏。tokio Semaphore 的 wait-and-acquire 语义天然支持排队。

### D2: RwLock 读写区分而非 Semaphore 分组

**选择**: `tokio::sync::RwLock<()>` — `concurrency_safe=true` 取 read guard，`false` 取 write guard

**替代方案**:
- A) 两个独立 Semaphore（read_sem + write_sem=1）— 无法保证写者运行时读者不进入
- B) 自定义调度器 — over-engineering

**理由**: RwLock 语义完美匹配需求（多读单写）。tokio 的 RwLock 是 write-preferring（有 pending writer 时新 reader 让步），避免 writer starvation。使用 `OwnedRwLockReadGuard`/`OwnedRwLockWriteGuard` 允许 guard 跨 `tokio::spawn` 边界移动。

### D3: broadcast channel 事件通知而非轮询

**选择**: `tokio::sync::broadcast::Sender<SlotEvent>`，每个 `SessionSlotPool` 一个

**替代方案**:
- A) `tokio::sync::watch` — 只能存最新值，不适合多 agent 事件
- B) `tokio::sync::mpsc` — 单消费者，不支持多个 waiter 同时监听
- C) 保留 DashMap 轮询 — 性能差

**理由**: broadcast 支持多 subscriber（`spawn_sync` + `WaitAgentTool` + 未来 UI）。capacity 设 128，`RecvError::Lagged` 时跳过旧事件（agent 完成事件数量有限，不会真正 lag）。

### D4: SpawnReservation 封装所有 guard

**选择**: 单个 `SpawnReservation` struct 持有 global slot guard + session slot guard + RwLock guard，drop 时全部释放

**替代方案**: 分开管理三个 guard — 容易漏释放

**理由**: 单一所有权 = 单一释放点。任务 panic、cancel、正常结束都走 drop。Codex 的 `SpawnReservation` 有 `commit` 概念（reserve → commit → release 三步），但 FastClaw 不需要 commit 阶段（无 agent nickname 注册等复杂操作），简化为 reserve → auto-release。

### D5: SpawnController 生命周期与 AppState 绑定

**选择**: `SpawnController` 作为 `Arc` 字段存在 `SubAgentManager` 中，由 `AppStateBuilder` 创建

**理由**: 与现有 Semaphore 的创建/持有方式一致，最小化启动流程改动。`SessionSlotPool` 按需创建（首次 spawn 某 session 时），通过 `gc_idle_sessions()` 回收。

## Risks / Trade-offs

**[Risk] RwLock writer starvation** → tokio RwLock 默认 write-preferring。新 reader 在有 pending writer 时会让步，保证 writer 最终获得锁。

**[Risk] broadcast channel 容量溢出** → capacity=128 远大于 `max_per_session`（默认 5）的事件数。即使 lag，`RecvError::Lagged(n)` 只丢中间事件，Completed 事件只要在 capacity 内就不丢。备选：fallback 到 `get_run()` 查状态。

**[Risk] Session pool 内存增长** → `gc_idle_sessions(max_idle: Duration)` 定期清理。由 gateway 的现有 GC 任务（`subagent_manager.gc()`）触发。

**[Risk] OwnedRwLockGuard 跨 spawn 边界的 Send 要求** → `tokio::sync::OwnedRwLockReadGuard<()>` 和 `OwnedRwLockWriteGuard<()>` 都是 `Send`，可以安全移入 `tokio::spawn`。

**[Trade-off] 全局 max_global vs per-session max** → 全局上限防止恶意/误操作耗尽资源，但可能导致一个活跃 session 阻塞其他 session 的全局 slot。缓解：`max_global` 设为 `max_per_session` 的 4 倍（20 vs 5），正常使用不会触及。

**[Trade-off] WaitAgentTool 的 wait-any 语义** → 类似 Codex 的已知问题：wait-any 返回第一个完成的 agent 后，其他 agent 继续运行（不自动 cancel）。LLM 需要自行决定是否 cancel 剩余 agent。这是有意设计——保持与 Codex 行为一致。
