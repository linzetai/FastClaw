## ADDED Requirements

### Requirement: usePluginStore structure
前端 SHALL 提供 `usePluginStore`（Zustand），管理插件面板 UI 状态与 MCP 插件数据。

#### Scenario: Store initial state
- **WHEN** 应用启动且 store 未初始化
- **THEN** `plugins` 为空数组、`loading` 为 false、`panelOpen` 为 false、`error` 为 null

#### Scenario: Store holds plugin list fields
- **WHEN** list 数据已加载
- **THEN** 每条 `PluginEntry` 包含：`id`、`scope`（user|project）、`enabled`、`status`、`toolCount`、`lastError?`、`connectedAt?`

### Requirement: Load plugin list
Store SHALL 提供 `fetchPlugins()` action，调用 `plugins.list` WS API 并更新 `plugins` 与 `loading`。

#### Scenario: Fetch on panel open
- **WHEN** 用户打开 PluginPanel（`openPanel()`）
- **THEN** `panelOpen` 设为 true
- **AND** 自动调用 `fetchPlugins()`
- **AND** `loading` 在请求期间为 true

#### Scenario: Fetch success
- **WHEN** `plugins.list` 返回成功
- **THEN** `plugins` 更新为响应中的列表
- **AND** `loading` 设为 false，`error` 清空

#### Scenario: Fetch failure
- **WHEN** `plugins.list` 失败
- **THEN** `loading` 设为 false
- **AND** `error` 保存错误消息供面板展示

### Requirement: Enable and disable actions
Store SHALL 提供 `enablePlugin(id)` 与 `disablePlugin(id)`，调用对应 WS API 并乐观更新本地 `enabled` 字段。

#### Scenario: Optimistic enable
- **WHEN** 调用 `enablePlugin(id)`
- **THEN** 立即将对应项 `enabled` 设为 true
- **AND** 发送 `plugins.enable { id }`
- **AND** 失败时回滚 `enabled` 并设置 `error`

#### Scenario: Optimistic disable
- **WHEN** 调用 `disablePlugin(id)`
- **THEN** 立即将对应项 `enabled` 设为 false，`status` 可设为 `disabled`
- **AND** 发送 `plugins.disable { id }`

### Requirement: Restart action
Store SHALL 提供 `restartPlugin(id)`，调用 `plugins.restart` 并将该项 `status` 设为 `connecting` 直至 WS 事件更新。

#### Scenario: Restart sets connecting
- **WHEN** 调用 `restartPlugin(id)`
- **THEN** 对应项 `status` 更新为 `connecting`
- **AND** 发送 `plugins.restart { id }`

### Requirement: WS sync for status changes
Store SHALL 订阅 `plugins.status_changed`（及必要时在 `mcp.status` 广播时刷新），合并更新单条插件状态，无需全量 refetch。

#### Scenario: Status changed event
- **WHEN** 收到 `plugins.status_changed { id, status, toolCount, lastError, ... }`
- **THEN** 合并更新 `plugins` 中匹配 `id` 的条目
- **AND** 若面板未打开，仍更新 store 以驱动 Sidebar 徽章

#### Scenario: Connected count for badge
- **WHEN** `plugins` 数组更新
- **THEN** `connectedCount` selector 返回 `status === "connected"` 的条目数量

### Requirement: Panel open state
Store SHALL 管理 `panelOpen` 与 `openPanel` / `closePanel` actions。

#### Scenario: Close panel clears loading only
- **WHEN** 调用 `closePanel()`
- **THEN** `panelOpen` 为 false
- **AND** 保留已缓存的 `plugins` 数据供下次快速打开

### Requirement: Tools detail cache
Store SHALL 可选缓存 `plugins.tools` 结果于 `toolsById: Record<string, ToolSummary[]>`，避免重复请求。

#### Scenario: Fetch tools for detail
- **WHEN** UI 请求某 id 的 tools 且缓存未命中
- **THEN** 调用 `plugins.tools { id }` 并写入 `toolsById[id]`
