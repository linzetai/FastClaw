## Goal Store Isolation

### Bug 14: set_session_id 不重置 idle_rounds/budget_warning

**现状**: `GoalStore::reset_accounting()` 重置了 token/time 计数，但 `idle_rounds` 和 `budget_warning_sent` 未重置。切换 session 后这些值错误保留。

**要求**:
- [ ] `reset_accounting()` 必须同时重置 `idle_rounds`（归零）和 `budget_warning_sent`（设为 false）
- [ ] 切换 session 后，新 session 的 goal 不应受旧 session 的 idle/budget 状态影响

### Bug 18: GoalStore 全局单例多 session 竞态

**现状**: `GoalStore` 是全局共享的单例，`set_session_id` 修改内部 `session_id`。多 session 并发时，不同 session 的 `set_session_id` 互相覆盖，导致 goal 操作指向错误的 session。

**要求**:
- [ ] GoalStore 必须实现 session 级隔离（推荐改为 per-session 实例）
- [ ] 同时存在多个 active session 时，各自的 goal 状态互不干扰
- [ ] 保持 GoalStore API 不变，仅修改注入方式
