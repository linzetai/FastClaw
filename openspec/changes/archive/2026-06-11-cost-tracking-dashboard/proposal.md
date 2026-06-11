## Why

XiaoLin 当前有内存级的 CostTracker（per-model 累计统计）和 Prometheus metrics 输出，但缺乏**历史持久化**和**前端可视化**。开发者无法追溯昨天花了多少 token、哪个模型消耗最高、哪些工具调用失败率高。参考 Claude Code 的 statsCache（按天聚合 + dailyModelTokens），XiaoLin 需要将 token 消耗持久化到 SQLite 并提供前端 Dashboard 页面，支持分天/分模型/分工具的成本查看和趋势分析。

## What Changes

- **Token 消耗按天持久化**：每次 LLM 调用完成后，除了内存累加，还将 token 数据写入 SQLite `token_usage_daily` 表（按天+模型聚合）
- **工具调用按天持久化**：每次 tool call 完成后，将结果（成功/失败/耗时）写入 SQLite `tool_call_daily` 表
- **Session 级成本汇总**：维护 `session_cost_summary` 表，记录每个 session 的总成本/tokens/turns
- **后端 REST API**：新增 `/api/v1/cost/daily`、`/api/v1/cost/tools`、`/api/v1/cost/sessions` 端点
- **WebSocket 实时推送**：turn 结束后推送 `CostUpdated` 事件到前端
- **前端 CostDashboard 页面**：时间轴折线图（分天 token 趋势）、模型占比图、工具健康度表格、Budget 进度条
- **Goal 模式 Budget 行为**：有 budget 时到达上限 pause goal；无 budget 时不限制

## Capabilities

### New Capabilities
- `cost-persistence`: SQLite 持久化层——在 CostTracker.record() 和 ObservationStore.record_tool_call() 之后异步写入按天聚合表
- `cost-api`: REST 端点提供历史成本数据查询（分天、分模型、分工具、分 session）
- `cost-realtime-event`: WebSocket `CostUpdated` 事件，turn 结束后推送 delta 和累计值
- `cost-dashboard-ui`: 前端成本可视化页面（折线图、饼图、表格、Budget 进度）
- `tool-failure-daily`: 工具调用成功/失败/耗时的按天持久化统计

### Modified Capabilities
- `goal-budget-steering`: 明确 budget 行为——有 budget 到达时 pause goal，无 budget 不限制
- `goal-token-accounting`: token delta 写入时同步写入 SQLite

## Impact

- **后端 crates**: `xiaolin-agent/runtime/cost_tracker.rs`（增加 SQLite writer）、`xiaolin-agent/runtime/observer.rs`（tool call 持久化）、`xiaolin-gateway`（新增 REST 路由）、`xiaolin-protocol`（新增 CostUpdated 事件）、`xiaolin-core`（SQLite schema migration）
- **前端**: 新增 `CostDashboard.tsx` 页面组件 + `cost-store.ts` 状态管理 + 路由/侧边栏入口
- **数据库**: 新增 3 张 SQLite 表 + migration
- **依赖**: 前端需要图表库（推荐复用现有依赖或轻量 recharts/chart.js）
