## 1. SQLite Schema + Migration

- [ ] 1.1 在 `xiaolin-core` 的 migration 目录新增 migration 文件，创建 `token_usage_daily`、`tool_call_daily`、`session_cost_summary` 三张表
- [ ] 1.2 验证 migration 在 `cargo test` 中能正确执行

## 2. 后端持久化层

- [ ] 2.1 在 `xiaolin-agent/src/runtime/` 新增 `cost_persistence.rs` 模块，封装 SQLite 写入逻辑（upsert token_usage_daily、upsert tool_call_daily、update session_cost_summary）
- [ ] 2.2 在 `CostTracker::record()` 调用后，通过 channel/spawn 异步触发 `cost_persistence::record_token_usage()`
- [ ] 2.3 在 `RuntimeObserver::record_tool_call()` 调用后，异步触发 `cost_persistence::record_tool_call()`
- [ ] 2.4 Session 开始/结束时更新 `session_cost_summary`（started_at, ended_at, model_breakdown）
- [ ] 2.5 实现 batch flush 机制：攒 1-2 秒后批量写入，减少 SQLite 事务开销

## 3. REST API 端点

- [ ] 3.1 在 `xiaolin-gateway` 新增 `/api/v1/cost/daily` 路由，查询 `token_usage_daily` 按日期范围返回
- [ ] 3.2 新增 `/api/v1/cost/tools` 路由，查询 `tool_call_daily` 并计算 avg_duration_ms 和 success_rate
- [ ] 3.3 新增 `/api/v1/cost/sessions` 路由，查询 `session_cost_summary` 按 started_at 倒序
- [ ] 3.4 新增 `/api/v1/cost/summary` 路由，返回总成本 + 今日成本 + budget 状态

## 4. WebSocket CostUpdated 事件

- [ ] 4.1 在 `xiaolin-protocol` 定义 `CostUpdated` 事件结构（session_id, turn_index, delta_cost_usd, total_cost_usd, model, input_tokens, output_tokens）
- [ ] 4.2 在 agent runtime turn 结束后发送 `CostUpdated` 事件到前端 WebSocket channel
- [ ] 4.3 前端 `stream-store.ts` 中监听 `CostUpdated` 事件并更新 cost-store

## 5. 前端 cost-store

- [ ] 5.1 新建 `src/lib/stores/cost-store.ts`，管理当前 session 实时成本 + 历史查询数据
- [ ] 5.2 实现 `fetchDailyCost(from, to)` → 调用 `/api/v1/cost/daily` 并缓存结果
- [ ] 5.3 实现 `fetchToolStats(from, to)` → 调用 `/api/v1/cost/tools`
- [ ] 5.4 实现 `fetchSummary()` → 调用 `/api/v1/cost/summary`
- [ ] 5.5 处理 WebSocket `CostUpdated` 事件，实时更新当前 session 的成本数据

## 6. 前端 CostDashboard 页面

- [ ] 6.1 新建 `src/components/cost/CostDashboard.tsx` 页面组件
- [ ] 6.2 实现日期范围选择器（近 7 天 / 近 30 天 / 自定义）
- [ ] 6.3 实现 Token 消耗时间轴折线图（X 轴为天，Y 轴为 token 数，按模型分色）
- [ ] 6.4 实现模型成本占比图（饼图或水平柱状图）
- [ ] 6.5 实现工具健康度表格（tool_name, total_calls, success_rate, avg_duration）
- [ ] 6.6 实现 Budget 进度条（如果配置了 budget_limit_usd）
- [ ] 6.7 实现今日成本实时显示（由 CostUpdated 事件驱动）

## 7. 路由 + 侧边栏入口

- [ ] 7.1 在 AppSidebar 中新增「成本」导航入口（图标 + 标签）
- [ ] 7.2 在路由配置中注册 CostDashboard 页面
- [ ] 7.3 在 AppHeader 中可选地显示当前 session 成本摘要（小数字 badge）

## 8. Goal Budget 行为确认

- [ ] 8.1 确认 goal-budget-steering 逻辑：有 budget → 到达时 pause goal + 推送 GoalPaused 事件
- [ ] 8.2 确认无 budget 时 goal continuation loop 不受限制
- [ ] 8.3 Goal pause 时前端 GoalStatusCard 显示"已暂停（预算耗尽）"状态
