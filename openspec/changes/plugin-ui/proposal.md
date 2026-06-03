## Why

MCP 插件（MCP servers）目前只能通过 Settings 页 buried 在配置里，或直接编辑 `mcp.json` / `config.json` 管理。Codex 布局原型在 Sidebar 顶部提供了 **Plugins** 快捷入口，用户期望一键查看已安装插件、启用/禁用、检查连接状态，而不必进入深层设置。现有 gateway 已具备 MCP 客户端生命周期（connect / reconnect / disconnect）和 `mcp.status` / `mcp.detail` 等 WS 能力，但缺少面向「插件管理」的聚合 API 与专用 UI。

## What Changes

- **Sidebar「Plugins」按钮** → 打开插件管理面板（modal/overlay），替代当前 `ComingSoon` 占位
- **插件列表**：展示名称、连接状态徽章（绿=connected、红=error、灰=disabled）、作用域（user / project）、启用/禁用开关
- **插件详情**：展开显示可用 tools 列表、连接信息（command/transport）、最近错误
- **快捷操作**：重启插件、查看连接日志
- **Sidebar 徽章**：显示当前已连接插件数量

## Capabilities

### New Capabilities

- `plugin-panel`: 插件管理面板组件——列表、详情展开、状态徽章、快捷操作、空状态
- `plugin-store`: 前端 Zustand store——聚合 MCP 插件列表与连接状态，WS 同步，enable/disable/restart actions

### Modified Capabilities

- `app-sidebar`: 在顶部操作区保留 Plugins 按钮，点击打开 `plugin-panel`；增加 connected 数量徽章

## Impact

- **Gateway WS**：新增 `plugins.*` 命名空间（list / enable / disable / restart / tools），内部复用现有 `mcp_status`、`mcp_handles`、`reload_mcp_servers` 与配置持久化逻辑；可选保留 `mcp.*` 向后兼容
- **前端**：新增 `components/plugins/`、`stores/plugin-store.ts`；`transport.ts` 增加 plugins API 封装
- **配置**：enable/disable 需写回 user 级 `mcpServers` 或 project 级 `.xiaolin/mcp.json`（按 scope 区分）
- **Out of scope (v1)**：插件市场、安装新插件向导（仍通过 Settings / 配置文件添加）；LLM provider plugins（`llm-plugins` API）不在本 change 范围
