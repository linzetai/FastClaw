## Why

`SubAgentManager` 使用单个 `tokio::sync::Semaphore` 控制所有 sub-agent 并发，存在三个结构性缺陷：`SubAgentDef.concurrency_safe` 字段已定义但从未生效（只读和写入 agent 无区分）、`spawn_sync` 用 100ms 轮询浪费 CPU 并引入延迟、全局 Semaphore 导致不同 session 互相干扰。LLM 也缺少 `wait_agent` 工具来批量等待多个子 agent 结果。

参考 Codex（AtomicUsize CAS + 写优先 RwLock）和 Claude Code（notification 事件回流），设计分层并发控制替代现有 Semaphore。

## What Changes

- 新增 `SpawnController` 分层并发控制器，替代 `SubAgentManager` 中的 `Arc<Semaphore>`
- 新增 `GlobalSlotPool`（全局上限，防单机资源耗尽）和 `SessionSlotPool`（per-session 隔离 + RwLock 读写区分）
- 新增 `SpawnReservation` RAII guard，drop 自动释放 slot 和 RwLock guard，杜绝 Codex 已知的 slot 泄漏问题
- 新增 `SlotEvent` broadcast channel，替代 `spawn_sync` 的 100ms 轮询
- 新增 `WaitAgentTool`（`wait_agent`），支持 wait-all 和 wait-any 语义
- 新增 `GET /api/v1/subagents/concurrency` 和 `sub_agents.concurrency` WS op 暴露实时并发状态
- `SubAgentTool` 从 `SubAgentDef.concurrency_safe` 读取标记传递给 `spawn()`
- `SubAgentPolicy.max_parallel` 保留兼容，语义映射到 `SpawnConfig.max_per_session`
- 新增 `config.toml [concurrency]` section 支持 `max_global`、`max_per_session`、`enforce_rw_isolation`

## Capabilities

### New Capabilities
- `spawn-controller`: 分层并发控制器（GlobalSlotPool + SessionSlotPool + SpawnReservation RAII + RwLock 读写隔离）
- `wait-agent-tool`: LLM 可用的批量等待工具，支持 wait-all / wait-any 语义
- `concurrency-observability`: HTTP/WS API 暴露 `ConcurrencySnapshot`（全局/per-session 状态、RwState、排队数）

### Modified Capabilities

## Impact

- **核心改动**: `crates/fastclaw-agent/src/subagent_manager.rs` — 删除 `Semaphore`，接入 `SpawnController`
- **工具层**: `crates/fastclaw-agent/src/subagent.rs` — `SubAgentTool` 传递 `concurrency_safe`；新增 `WaitAgentTool`
- **网关层**: `crates/fastclaw-gateway/src/state/builder.rs` — 创建 `SpawnController`；`routes/subagent.rs` 和 `ws/mod.rs` 新增 API
- **协议层**: `crates/fastclaw-protocol/src/op.rs` — 新增 `SubAgentsConcurrency` op
- **配置层**: `crates/fastclaw-core/src/agent_config.rs` — 新增 `SpawnConfig`；`config.toml` 新增 `[concurrency]`
- **提示词**: `crates/fastclaw-agent/src/runtime/prompt_builder.rs` — 更新并发说明和 `wait_agent` 使用指南
