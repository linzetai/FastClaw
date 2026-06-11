# Fix Permission Propagation

## Problem

用户在前端设置"完全权限"（Full/YOLO mode）后，后端工具执行仍然报 "outside all allowed locations" 错误，导致大量 token 浪费。

### 数据支撑

从 `trajectory_steps` 表统计（9,944 次工具调用）：
- 总失败 452 次（4.5%）
- 其中 278 次（61.5%）是 **路径作用域拒绝**
- 99 次（21.9%）是 **文件不存在**（LLM 猜错路径）

278 次路径拒绝全部错误信息为：
> "Cannot access path '...': it is outside all allowed locations"

此错误只在 `FileAccessMode::Workspace` 分支产生，`Full` 模式下不可能出现。

## Root Cause

三个独立但相关的 Bug：

### Bug 1: Settings 修改不传播到已有 session（P0）

`Settings → Security → 执行模式` 修改通过 `api.updateAgent()` → `reload_agents()` 路径更新 agent config JSON 并 hot-reload。但 `RuntimeTurnExecutor.config` 是 gateway 启动时的 **clone 快照**，`reload_agents()` 只更新了 `last_good_agents` 和 `AgentRouter`，不更新 executor 的 config。

结果：`effective_behavior(session_id)` 对没有 per-session override 的 session 回退到旧快照值。

### Bug 2: SubAgent 不继承父级 behavior（P1）

`SubAgentManager::run_subagent()` 中：
- `request.work_dir = None`（不继承父 session 的 work_dir）
- `config.behavior` 用的是 subagent 自己的 AgentConfig（默认 `FileAccessMode::Workspace`）

即使父 session 设了 Full mode，subagent 的文件操作仍然受 Workspace 路径限制。

### Bug 3: work_dir 缺失导致 workspace_root() 错误（P2）

`workspace_root()` 优先读 `EFFECTIVE_WORK_DIR` task-local，没有则 fallback 到 `std::env::current_dir()`。当 gateway 进程 cwd ≠ 项目目录时，所有绝对路径都被判定为"在工作区外"。

## Solution

### Fix 1: reload_agents 同步更新 executor config

`reload_agents()` 完成后，将新的 behavior config 广播给 `RuntimeTurnExecutor`。可通过以下方式之一：
- 将 `RuntimeTurnExecutor.config` 改为 `Arc<ArcSwap<AgentConfig>>`
- 或在 `effective_behavior()` 中先查 `last_good_agents` 再 fallback

推荐后者，侵入性小。

### Fix 2: SubAgent 继承父级 behavior 和 work_dir

`run_subagent()` 中：
- 将父 session 的 `work_dir` 传给 subagent 的 `request.work_dir`
- 将父 session effective behavior 的 `file_access` 传给 subagent config

### Fix 3: Session 创建时确保 work_dir 有效

Session 创建的 fallback 链已存在（env cwd → detect_workspace_root），但 chat request 中的 `work_dir` 应该优先使用 session 存储的 work_dir 作为 fallback：
- 如果 request 没有传 `workDir`，从 session 数据库读取该 session 之前设置的 `work_dir`

## Impact

- 消除 278 次路径作用域错误（-61.5% 失败率）
- 节省约 6,000-10,000 tokens/session（避免 LLM 重试循环）
- 权限设置即时生效，无需重启 gateway

## Scope

- `crates/xiaolin-agent/src/session_bridge.rs` — effective_behavior 读取逻辑
- `crates/xiaolin-agent/src/subagent_manager.rs` — subagent behavior/work_dir 继承
- `crates/xiaolin-gateway/src/state/mod.rs` — reload_agents 后更新 executor
- `crates/xiaolin-agent/src/runtime/mod.rs` — request.work_dir fallback

## Non-goals

- 动态 tool profile（已决定不做）
- 路径作用域本身的设计变更（只修复传播问题）
