## Context

ConnectionsPage 目前用列表卡片展示 MCP 服务器和 Channel 的摘要。后端已有丰富的数据源（`McpClient.tools()`, `McpServerConfig`, `ChannelPlugin.tools()`, `ChannelConfig`）但前端未暴露。

现有交互模式：点击卡片区域内的操作按钮（删除/断开），以及 Modal 弹窗（AddMcpModal, WechatQrModal）。

## Goals / Non-Goals

**Goals:**
- 点击卡片打开详情 Modal，展示完整信息
- MCP 详情：配置（command/args/transport）、完整 tool 列表（name + description）、错误信息、操作按钮
- Channel 详情：元数据（aliases、mode）、能力详情、脱敏配置、channel tools、操作按钮
- 后端新增 `mcp.detail` 和 `channels.detail` WS 方法
- 保持与现有 Modal 组件一致的视觉风格

**Non-Goals:**
- 不修改 tool 的 input_schema 展示（太复杂，后续做）
- 不添加 MCP 服务器在线编辑配置的功能
- 不添加 Channel 配置编辑功能
- 不重构现有卡片组件

## Decisions

### 1. 后端 API 设计：独立 detail 方法 vs 增强 list 返回

**选择**: 新增 `mcp.detail` 和 `channels.detail` 独立方法。

**理由**: list 返回轻量摘要保证列表加载快，detail 按需加载完整信息（tool 列表可能很大）。避免 list 返回臃肿。

### 2. MCP detail 数据来源

- **配置**: 从 `config_live` 中读取 `McpServerConfig`（command, args, transport, env）
- **Tool 列表**: 从 `mcp_handles` 获取 `SharedMcpClient`，调用 `.tools()` 返回 `Vec<McpTool>`
- **状态**: 从 `mcp_status` 读取 `McpServerStatus`

### 3. Channel detail 数据来源

- **Plugin 数据**: 从 `channel_registry` 取 plugin，调用 `meta()`, `capabilities()`, `connection_mode()`, `tools()`
- **配置数据**: 从 `config.channels` 读取，敏感字段（app_secret, encrypt_key, token）在后端做掩码处理（前4字符 + "****"）
- **工具列表**: 调用 `ChannelPlugin.tools()` 获取 `Vec<Arc<dyn Tool>>`，返回 name + description

### 4. 前端交互：卡片点击事件

**选择**: 卡片整体可点击打开详情，但操作按钮（删除/断开等）阻止冒泡。

**理由**: 最直观的交互方式，点击卡片 = 查看详情，按钮保持独立功能。添加 `cursor: pointer` 和 hover 效果提示可点击。

### 5. Modal 组件复用

两个详情 Modal（`McpDetailModal`, `ChannelDetailModal`）作为独立组件放在 `ConnectionsPage.tsx` 内部，复用现有 Modal 的 overlay + panel 样式。

## Risks / Trade-offs

- **MCP Tool 列表可能很长（30+）**: → Modal 内部设置 `max-height` 和 `overflow-y: auto` 滚动
- **MCP client lock 时获取 tools 可能阻塞**: → `mcp_handles` 是 `Mutex`，快速 clone tool 列表后释放锁
- **敏感信息泄露风险**: → 后端统一掩码，前端不接触原始密钥
