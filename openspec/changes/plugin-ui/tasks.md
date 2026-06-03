## 1. WS API（plugins.*）

- [ ] 1.1 在 `xiaolin-protocol` 中定义 `PluginSummary`、`PluginTool` 等类型及 `plugins.list/enable/disable/restart/tools` 请求参数
- [ ] 1.2 在 `xiaolin-gateway/src/ws/` 新增 `plugins.rs`，实现 `handle_plugins_list`：合并 user config + project mcp.json + `mcp_status`
- [ ] 1.3 实现 `plugins.enable` / `plugins.disable`：按 scope 写回 config 或 project mcp.json，调用 `reload_mcp_servers()`
- [ ] 1.4 实现 `plugins.restart`：单 server 重连（disconnect + connect 或 targeted reload）
- [ ] 1.5 实现 `plugins.tools`：返回指定 id 的 tools 列表（复用 `mcp_handles` + tools()）
- [ ] 1.6 在 `reload_mcp_servers` / status 更新路径广播 `plugins.status_changed` 事件
- [ ] 1.7 在 `ws/mod.rs` dispatcher 注册 `plugins.*` handlers

## 2. 前端 Store

- [ ] 2.1 创建 `stores/plugin-store.ts`：`usePluginStore`（plugins、loading、panelOpen、error、toolsById）
- [ ] 2.2 在 `transport.ts` 添加 `listPlugins`、`enablePlugin`、`disablePlugin`、`restartPlugin`、`getPluginTools` 封装
- [ ] 2.3 在 store 中订阅 `plugins.status_changed`，实现 `fetchPlugins`、`enablePlugin`、`disablePlugin`、`restartPlugin` 及 `connectedCount` selector

## 3. Plugin Panel UI

- [ ] 3.1 创建 `components/plugins/PluginPanel.tsx`：overlay 容器、标题、关闭按钮
- [ ] 3.2 创建 `PluginList.tsx` + `PluginRow.tsx`：列表、scope 标签、状态徽章、toggle
- [ ] 3.3 实现行展开 `PluginDetail.tsx`：tools 列表、connection info、lastError
- [ ] 3.4 实现快捷操作：Restart、View logs（错误信息展示）
- [ ] 3.5 实现 `PluginEmptyState.tsx` 与「Manage in Settings」入口
- [ ] 3.6 在 `AppShell` 挂载 `PluginPanel`，绑定 `panelOpen`

## 4. Sidebar 集成

- [ ] 4.1 修改 `AppSidebar`：Plugins 按钮调用 `openPanel()`，移除 ComingSoon 路由
- [ ] 4.2 添加 connected 数量徽章，订阅 `usePluginStore`；挂载时静默 `fetchPlugins()`

## 5. 验证

- [ ] 5.1 E2E（Tauri MCP）：打开 Plugins 面板 → 列表显示 → toggle enable/disable → restart → 徽章数量更新
- [ ] 5.2 运行 `cargo clippy -- -D warnings` 与前端 typecheck，确认无 dead_code / 未使用导出
