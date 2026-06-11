# Design: Fix Permission Propagation

## Decision 1: RuntimeTurnExecutor config 更新策略

**选择：`effective_behavior()` 中查询 `last_good_agents`**

```rust
// session_bridge.rs
fn effective_behavior(&self, session_id: &str) -> BehaviorConfig {
    // 1. Per-session override 优先
    if let Some(ref overrides) = self.behavior_overrides {
        if let Some(entry) = overrides.get(session_id) {
            return entry.value().clone();
        }
    }
    // 2. 查 live config（hot-reload 后立即生效）
    if let Some(ref live_agents) = self.live_agents {
        if let Some(agent) = live_agents.load().first() {
            return agent.behavior.clone();
        }
    }
    // 3. 启动时快照 fallback
    self.config.behavior.clone()
}
```

**需要的变更：**
- `RuntimeTurnExecutor` 新增字段 `live_agents: Option<Arc<ArcSwap<Vec<AgentConfig>>>>`
- builder 中将 `last_good_agents` 的引用传给 executor

**理由：**
- 最小侵入性，不改变现有 per-session override 逻辑
- `ArcSwap` 读操作是 lock-free，不影响性能
- 即时生效，无需额外的同步机制

## Decision 2: SubAgent behavior 继承

**选择：spawn 时传入 parent effective behavior**

```rust
// subagent_manager.rs — run_subagent()
let request = ChatRequest {
    messages,
    stream: true,
    work_dir: parent_work_dir.clone(),  // 继承父级 work_dir
    ..Default::default()
};

// config.behavior 也需要覆盖
let mut sub_config = agent_config.clone();
sub_config.behavior.file_access = parent_file_access;
sub_config.behavior.additional_allowed_paths = parent_additional_paths;
```

**SubAgentManager::spawn() 新增参数：**
```rust
pub struct SubAgentInheritedContext {
    pub work_dir: Option<String>,
    pub file_access: FileAccessMode,
    pub additional_allowed_paths: Vec<String>,
}
```

**理由：**
- SubAgent 是在父 session 的安全上下文中运行的
- 如果父 session 允许 Full access，subagent 也应该允许
- work_dir 继承避免了 gateway cwd fallback 的陷阱

## Decision 3: request.work_dir fallback 策略

**选择：在 turn 执行前从 session store 补充 work_dir**

```rust
// session_bridge.rs — execute_turn()
let mut request = request;
if request.work_dir.is_none() {
    if let Some(ref store) = self.session_store {
        if let Ok(session) = store.get_session(&sid).await {
            request.work_dir = session.work_dir;
        }
    }
}
```

**理由：**
- Session 创建时已经通过 detect_workspace_root 计算了合理的 work_dir
- 每个 turn 的 request 不应该重复传递
- 减少前端和后端的耦合

## 变更文件清单

| 文件 | 变更类型 | 说明 |
|------|---------|------|
| `crates/xiaolin-agent/src/session_bridge.rs` | 修改 | `effective_behavior()` + work_dir fallback |
| `crates/xiaolin-agent/src/subagent_manager.rs` | 修改 | spawn/run_subagent 继承 context |
| `crates/xiaolin-gateway/src/state/builder.rs` | 修改 | 传 `last_good_agents` 引用给 executor |

## 不变量

- Per-session override 仍然优先于全局配置
- `ApprovalStrategy` 不变（它是 approval 层面的，不影响 file_access）
- Plan mode 阻塞逻辑不变
- Gateway 重启后 per-session override 仍然丢失（这是单独的持久化问题）
