## Context

XiaoLin 将 MCP servers 视为 Agent 可调用的「插件」。后端已有完整基础设施：

```
┌─────────────────────────────────────────────────────────────┐
│  Config sources                                              │
│  ┌──────────────────┐    ┌──────────────────────────────┐   │
│  │ user config      │    │ project .xiaolin/mcp.json    │   │
│  │ mcpServers[]     │    │ or .cursor/mcp.json          │   │
│  └────────┬─────────┘    └──────────────┬───────────────┘   │
│           └──────────────┬──────────────┘                    │
│                          ▼                                   │
│              reload_mcp_servers()                            │
│                          ▼                                   │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ mcp_handles: HashMap<id, McpClient>                   │   │
│  │ mcp_status:  ArcSwap<HashMap<id, McpServerStatus>>    │   │
│  └──────────────────────────────────────────────────────┘   │
│                          │                                   │
│           WS (existing)  │  mcp.status / mcp.detail /       │
│                          │  mcp.reload / mcp.add / remove    │
└─────────────────────────────────────────────────────────────┘
```

前端 Settings 中有 `McpManager`（部分仍为 mock），Sidebar（layout-overhaul）的 Plugins 按钮目前路由到 `ComingSoon`。

原型 `docs/prototype-codex-layout.html` 中 Plugins 位于 Sidebar 顶部操作区，与 Search / Automations 并列。

## Goals / Non-Goals

**Goals:**

- 从 Sidebar 一键打开插件管理面板，列表展示所有已配置 MCP 插件及实时状态
- 支持 enable/disable、restart、查看 tools 与连接详情
- 区分 user 级与 project 级插件，显示 scope 标签
- Sidebar 徽章显示 connected 数量
- 复用 gateway 已有 MCP 状态，不重复维护连接逻辑

**Non-Goals:**

- v1 不做插件市场（browse / install from registry）
- 不改造 LLM provider plugins（`/api/v1/llm-plugins`）
- 不在本 change 实现「添加新插件」表单（保留 Settings / 配置文件路径；可面板内链到 Settings）
- 不替换现有 `mcp.*` WS API（新 API 为面向 UI 的聚合层，内部可委托 `mcp.*`）

## Decisions

### D1: Plugin panel 作为 modal/overlay

**选择**：点击 Sidebar「Plugins」打开全屏或居中大尺寸 overlay（与 SettingsPanel 类似），而非占用 WorkspacePanel 或替换主内容区。

**理由**：插件管理是短时操作，不应挤占 Chat/Review 工作区。overlay 可在任意会话上下文中打开，关闭后回到原视图。与原型侧栏入口 + 独立面板的交互一致。

**实现要点**：`PluginPanel` 挂载在 `AppShell` 层级，由 `ui-store` 或 `plugin-store` 的 `panelOpen` 控制；ESC / 点击遮罩关闭。

### D2: 复用 gateway MCP 客户端状态

**选择**：`plugins.list` 聚合 `config_live` 中的 server 定义 + `mcp_status` 运行时状态 + project MCP 配置；不新建独立连接池。

**理由**：`reload_mcp_servers()` 已是单一真相源；重复维护会导致状态漂移。`McpServerStatus` 已包含 `Connecting | Connected | Failed | Disabled`。

### D3: WS API — `plugins.*` 命名空间

**选择**：新增面向 UI 的 WS 方法，与现有 `mcp.*` 并存：

| Method | 行为 |
|--------|------|
| `plugins.list` | 返回 `{ plugins: PluginSummary[] }`，含 id、name、scope、enabled、status、toolCount、lastError |
| `plugins.enable` | `{ id }` → 设置 enabled=true，persist，reload |
| `plugins.disable` | `{ id }` → 设置 enabled=false，disconnect，persist |
| `plugins.restart` | `{ id }` → 单 server 重连（reload 子集或 disconnect+connect） |
| `plugins.tools` | `{ id }` → 返回 tools 列表（委托 `mcp.detail` 或直连 client.tools()） |

**事件**：`plugins.status_changed` — 在 `mcp_status` 更新后广播，payload 与 list 项结构一致，便于 store 增量更新。

**替代方案**：前端直接调用 `mcp.status` / `mcp.detail`。

**理由**：`plugins.*` 统一 camelCase 字段、scope、enabled 语义，减少前端拼装；未来可扩展 displayName、icon 而不污染底层 `mcp.*`。

### D4: User / project scope 指示

**选择**：每条插件带 `scope: "user" | "project"`。user 级来自全局 config；project 级来自当前 workspace 的 `.xiaolin/mcp.json` 或 `.cursor/mcp.json`（与 `mcp.detail` 的 `config.source` 一致）。

**enable/disable 持久化**：

- `scope=user` → 更新 `config_live.mcpServers[].enabled`，`persist_config_key("mcpServers", ...)`
- `scope=project` → 更新 workspace 内 mcp.json 的 `disabled` 字段（project 格式用 `disabled: true` 表示禁用）

**冲突**：同 id 在 user 与 project 同时存在时，project 覆盖 user（与 `reload_mcp_servers` 现有 merge 逻辑一致）；列表只显示合并后的有效条目，scope 标为 `project`。

### D5: v1 无 marketplace

**选择**：面板仅管理**已安装**（已配置）的插件。添加新插件仍引导至 Settings → MCP 或编辑配置文件。

**理由**：降低 v1 范围；安装流程涉及命令校验、env 密钥、transport 选择，适合留在 Settings 深配置页。

## Risks / Trade-offs

**[Risk] `plugins.*` 与 `mcp.*` 双 API 维护成本** → handlers 内部共享 `ws/mcp.rs` 辅助函数；集成测试覆盖两条路径。

**[Risk] project 级 enable/disable 写文件失败** → 返回明确错误；UI 回滚 optimistic update。

**[Risk] restart 单 server 无原子 API** → v1 可实现为 `reload_mcp_servers` 全量重载，或 disconnect+reconnect 单 id；文档注明短暂全量抖动可能。

**[Trade-off] 日志查看** → v1「查看连接日志」可打开 gateway 日志过滤（server id）或显示 `status.error` 历史；完整 log tail 可后续迭代。

## Component sketch

```
AppSidebar
  └─ [Plugins] ──click──▶ plugin-store.openPanel()
                              │
                              ▼
                        PluginPanel (overlay)
                          ├─ PluginList
                          │    └─ PluginRow (toggle, badge, scope)
                          └─ PluginDetail (expand: tools, config, actions)
```
