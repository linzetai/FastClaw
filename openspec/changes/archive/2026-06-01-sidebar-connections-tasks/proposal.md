## Why

侧边栏目前有 6 个导航按钮，但除「对话」外其余全部显示 Coming Soon 占位页。后端已具备 MCP 服务器状态查询、Channel 插件列表、Cron Jobs 完整 CRUD 等 API，但前端没有任何 UI 接入。用户无法查看已连接的 MCP 服务器和通道状态，也无法管理定时任务。

本次变更聚焦最高优先级的两个页面：**连接（Connections）** 和 **任务（Tasks）**，同时移除已废弃的「专家」Tab，精简侧边栏。

## What Changes

- **移除「专家」Tab**：从 NavItem 类型和 NavRail 中移除 `experts` 选项
- **实现「连接」页面**：展示 MCP 服务器列表（状态/工具数/错误信息）+ Channel 插件列表（名称/模式/状态），支持连接/断开 MCP 服务器
- **实现「任务」页面**：展示 Cron Job 列表（名称/调度/状态），支持创建/编辑/删除/启用禁用，展示运行历史
- **新增后端 API**：`GET /api/v1/mcp/servers` 暴露 MCP 状态，`GET /api/v1/cron/jobs/:id/runs` 暴露运行历史
- **WebSocket 实时推送**：MCP 状态变更和 Cron Job 运行状态变更通过现有 WS 推送给前端

## Capabilities

### New Capabilities
- `connections-page`: 侧边栏「连接」页面，展示和管理 MCP 服务器、Channel 插件的连接状态
- `tasks-page`: 侧边栏「任务」页面，Cron Jobs 的 CRUD 管理和运行历史查看

### Modified Capabilities
（无现有 spec 需要修改）

## Impact

- **前端**：`NavRail.tsx`、`AppLayout.tsx`、`ui-store.ts`（NavItem 类型）、新增 `ConnectionsPage.tsx` 和 `TasksPage.tsx` 组件
- **后端路由**：`routes/mod.rs` 新增 MCP 列表和 cron runs 路由
- **后端 routes**：新增 `routes/mcp.rs` 或在现有 `chat.rs` 中添加 MCP 状态端点
- **WebSocket**：在 `ws.rs` 中新增 MCP/Cron 状态变更事件类型
- **依赖**：无新增外部依赖
