## 1. SpawnController 核心实现

- [x] 1.1 新建 `crates/fastclaw-agent/src/spawn_controller.rs` 模块，定义 `SpawnConfig`、`SpawnControllerError`、`SlotEvent` 类型
- [x] 1.2 实现 `GlobalSlotPool`：`AtomicUsize` CAS acquire/release + `tokio::sync::Notify` 唤醒
- [x] 1.3 实现 `SessionSlotPool`：per-session `AtomicUsize` + `Arc<tokio::sync::RwLock<()>>` + `broadcast::Sender<SlotEvent>` + `last_activity` 时间戳
- [x] 1.4 实现 `SpawnReservation` RAII guard：持有 global/session slot 和 `OwnedRwLockReadGuard`/`OwnedRwLockWriteGuard`，drop 时释放所有资源并广播 `SlotEvent::Released`
- [x] 1.5 实现 `SpawnController`：`DashMap<String, Arc<SessionSlotPool>>` 管理 session pools，`reserve()` 方法按 global → session → rw_gate 顺序获取
- [x] 1.6 实现 `ConcurrencySnapshot` / `SessionSnapshot` / `RwState` / `ActiveAgentInfo` 可观测结构体和 `snapshot()` 方法
- [x] 1.7 实现 `gc_idle_sessions(max_idle: Duration)` 清理无活跃 agent 的 session pool
- [x] 1.8 在 `crates/fastclaw-agent/src/lib.rs` 中声明 `pub mod spawn_controller` 并导出公共类型

## 2. SpawnController 单元测试

- [x] 2.1 测试全局 slot 上限：`global_pool_rejects_when_full` — 超过 `max_global` 时 reserve 超时失败，释放后成功
- [x] 2.2 测试 per-session 隔离：`sessions_have_independent_pools` — Session A 占满不影响 Session B
- [x] 2.3 测试读者并行：`concurrent_safe_agents_run_in_parallel` — 5 个 `safe=true` 同时持有
- [x] 2.4 测试写者独占：`non_concurrent_safe_blocks_others` — 写者持有时读者超时
- [x] 2.5 测试写者等读者 drain：`writer_waits_for_readers_to_drain` — 读者全部 drop 后写者获得锁
- [x] 2.6 测试 RAII drop 释放：`reservation_drop_releases_slot` — drop 后新 reserve 成功
- [x] 2.7 测试 task 取消释放：`reservation_released_on_task_cancel` — tokio task abort 后 slot 释放
- [x] 2.8 测试 broadcast 事件：`slot_events_are_broadcast` — Acquired/Released 事件可收到
- [x] 2.9 测试 snapshot：`snapshot_reflects_current_state` — 数值与实际一致
- [x] 2.10 测试 GC：`gc_removes_idle_session_pools` — 无活跃 agent 的 pool 被回收

## 3. 接入 SubAgentManager

- [x] 3.1 修改 `SubAgentManager` struct：删除 `concurrency: Arc<Semaphore>`，新增 `controller: Arc<SpawnController>`
- [x] 3.2 修改 `SubAgentManager::new()` 签名：接收 `Arc<SpawnController>` 参数
- [x] 3.3 修改 `spawn()` 签名：新增 `concurrency_safe: bool` 参数
- [x] 3.4 修改 `spawn()` 内部：`tokio::spawn` 闭包内将 `Semaphore::acquire()` 替换为 `controller.reserve(session_id, run_id, concurrency_safe, timeout)`
- [x] 3.5 在 `spawn()` 的完成/失败路径中广播 `SlotEvent::Completed` / `SlotEvent::Failed`
- [x] 3.6 新增 `spawn_and_wait()` 方法：用 `broadcast::Receiver` 监听完成事件替代 `spawn_sync()` 的 100ms 轮询
- [x] 3.7 保留 `spawn_sync()` 作为 `spawn_and_wait()` 的别名（向后兼容），标记 `#[deprecated]`
- [x] 3.8 暴露 `pub fn controller(&self) -> &Arc<SpawnController>` 给外部（routes、WS handler）

## 4. 接入 SubAgentTool

- [x] 4.1 修改 `SubAgentTool::execute()`：从 `SubAgentDef` 读取 `concurrency_safe`（无 def 时默认 `false`），传给 `manager.spawn()`
- [x] 4.2 修改 `spawn()` 和 `spawn_and_wait()` 调用处的参数列表

## 5. 接入 Gateway 启动流程

- [x] 5.1 在 `crates/fastclaw-gateway/src/state/builder.rs` 中创建 `SpawnController`（从 config 或 `SubAgentPolicy` 构建 `SpawnConfig`）
- [x] 5.2 将 `Arc<SpawnController>` 传给 `SubAgentManager::new()`
- [x] 5.3 新增 `SpawnConfig::from_policy_and_config()` 方法处理 config.toml 和 SubAgentPolicy 的兼容逻辑

## 6. Manager 集成测试

- [x] 6.1 测试 spawn 限流：`spawn_respects_controller_limits` — 快速 spawn N+1 个，第 N+1 个等待
- [x] 6.2 测试事件驱动完成：`spawn_and_wait_uses_event_not_polling` — 完成后延迟 < 10ms
- [x] 6.3 测试 cancel 释放 slot：`cancel_releases_slot_for_next_spawn` — 占满 → cancel → 新 spawn 成功
- [x] 6.4 测试读写语义 E2E：`explore_parallel_code_exclusive` — explore 并行，code 独占
- [x] 6.5 测试 concurrency_safe 传递：`concurrency_safe_flag_from_def` — SubAgentDef 标记影响 reserve 参数

## 7. WaitAgentTool 实现

- [x] 7.1 新建 `WaitAgentTool` struct，实现 `Tool` trait（name="wait_agent", kind=System, supports_parallel=false）
- [x] 7.2 实现参数 schema：`run_ids: Vec<String>`, `mode: "all"|"any"`, `timeout_seconds: Option<u64>`
- [x] 7.3 实现 execute：先检查已完成的 run，再订阅 broadcast events，tokio::select timeout/event loop
- [x] 7.4 实现结果格式化：JSON `{ results: { run_id: { status, result?, error? } }, timed_out: bool }`
- [x] 7.5 在 `builder.rs` 注册 `WaitAgentTool` 到 tool registry

## 8. WaitAgentTool 测试

- [x] 8.1 测试 wait-all：`wait_all_returns_when_all_complete` — 返回时间 ≈ 最慢 agent
- [x] 8.2 测试 wait-any：`wait_any_returns_on_first_completion` — 返回时间 ≈ 最快 agent
- [x] 8.3 测试超时：`wait_timeout_returns_partial` — 返回已完成 + timed_out 标记
- [x] 8.4 测试已完成立即返回：`wait_already_completed_returns_immediately`
- [x] 8.5 测试 unknown run_id：`wait_unknown_run_id_returns_error`

## 9. API 可观测性

- [x] 9.1 新增 `GET /api/v1/subagents/concurrency` HTTP endpoint 返回 `ConcurrencySnapshot`
- [x] 9.2 在 `routes/mod.rs` 注册路由
- [x] 9.3 新增 `SubAgentsConcurrency` ClientOp variant 到 `op.rs`
- [x] 9.4 在 `ws/mod.rs` 添加 `sub_agents.concurrency` dispatch 逻辑
- [x] 9.5 测试 HTTP endpoint 返回正确 JSON 结构

## 10. 配置与提示词

- [x] 10.1 在 `agent_config.rs` 新增 `ConcurrencyConfig` 结构体（`max_global`, `max_per_session`, `enforce_rw_isolation`, `slot_acquire_timeout_seconds`）
- [x] 10.2 在 config 加载逻辑中解析 `[concurrency]` section
- [x] 10.3 更新 `prompt_builder.rs` 并发说明：解释读写区分和 `wait_agent` 工具用法
- [x] 10.4 确保现有所有 `cargo test`、`cargo clippy -- -D warnings`、`npx tsc --noEmit` 通过

## 11. 验证

- [x] 11.1 全量 `cargo test` 通过（包括新增的 2.x / 6.x / 8.x 测试）
- [x] 11.2 `cargo clippy -- -D warnings` 零警告
- [x] 11.3 `npx tsc --noEmit` 通过（前端无影响确认）
- [x] 11.4 端到端验证：用tauri-mcp,通过 chat 触发 explore + code spawn，确认读写隔离行为
