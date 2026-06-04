## MODIFIED Requirements

### Requirement: Top action buttons
顶部操作区 SHALL 包含四个文字+图标按钮（垂直排列，每项 padding 6px 10px，圆角 6px）：New chat、Search、Plugins、Automations。

#### Scenario: New chat action
- **WHEN** 用户点击 "New chat" 按钮
- **THEN** 创建新会话并切换到该会话

#### Scenario: Search action
- **WHEN** 用户点击 "Search" 按钮
- **THEN** 展开搜索输入框，可以搜索会话历史

#### Scenario: Plugins action
- **WHEN** 用户点击 "Plugins" 按钮
- **THEN** 调用 `usePluginStore.getState().openPanel()`
- **AND** 不切换 `activeNav` 至全屏 ComingSoon 视图

#### Scenario: Automations opens management panel
- **WHEN** 用户点击 "Automations" 按钮
- **THEN** 系统 SHALL 打开 AutomationPanel overlay（由 automation-panel capability 提供）
- **AND** SHALL NOT 显示 ComingSoon 占位页
- **AND** SHALL NOT 导航到全页 TasksPage 路由

### Requirement: Automations button placement
Automations 按钮 SHALL 位于侧栏顶部操作区，顺序在 Plugins 之后、Pinned 分组之前，与原型 `docs/prototype-codex-layout.html` 一致。

#### Scenario: Visual order in sidebar
- **WHEN** AppSidebar 渲染顶部操作区
- **THEN** 按钮顺序为：New chat → Search → Plugins → Automations
- **AND** Automations 使用与原型一致的时钟/循环图标（refresh/cycle 语义）

#### Scenario: Automations while panel open
- **WHEN** AutomationPanel overlay 已打开，用户再次点击 Automations
- **THEN** 实现 MAY 关闭 overlay（toggle）或保持打开；行为须在实现中统一并保持一致

### Requirement: Sidebar structure unchanged
除 Automations 点击行为外，AppSidebar 的三段结构（顶部操作、中间滚动列表、底部 Settings）SHALL 保持不变。

#### Scenario: Other sidebar regions unaffected
- **WHEN** 用户打开或关闭 AutomationPanel
- **THEN** Pinned / Projects / Chats 列表与 Settings 按钮行为 SHALL 不受影响

### Requirement: Plugins button opens plugin panel
AppSidebar 顶部操作区的「Plugins」按钮 SHALL 打开 `PluginPanel`（通过 `usePluginStore.openPanel()`），不再显示 `ComingSoon` 占位页。

#### Scenario: Plugins button click
- **WHEN** 用户点击「Plugins」按钮
- **THEN** 调用 `usePluginStore.getState().openPanel()`
- **AND** 不切换 `activeNav` 至全屏 ComingSoon 视图

#### Scenario: Plugins button visual
- **WHEN** AppSidebar 渲染顶部操作区
- **THEN** 「Plugins」按钮保持与原型一致的文字+图标样式（padding 6px 10px，圆角 6px）

### Requirement: Connected plugins badge
「Plugins」按钮 SHALL 在存在已连接 MCP 插件时显示数字徽章，表示 `status === "connected"` 的插件数量。

#### Scenario: Badge hidden when zero connected
- **WHEN** `usePluginStore` 的 connected 数量为 0
- **THEN** 不显示徽章或显示隐藏状态

#### Scenario: Badge shows connected count
- **WHEN** 有 N 个插件状态为 connected（N > 0）
- **THEN** 在 Plugins 按钮右侧或角标显示 N
- **AND** N 上限显示为 「9+」当 N > 9（可选）

#### Scenario: Badge updates on status change
- **WHEN** 收到 `plugins.status_changed` 或 list 刷新导致 connected 数量变化
- **THEN** 徽章数字在 300ms 内更新，无需重新打开面板

### Requirement: Store subscription on sidebar mount
AppSidebar SHALL 在挂载时确保 `usePluginStore` 已订阅 WS 事件；若 `plugins` 为空，可后台调用一次 `fetchPlugins()` 以初始化徽章（不打开面板）。

#### Scenario: Background prefetch for badge
- **WHEN** AppSidebar 挂载且 `plugins` 为空
- **THEN** 静默调用 `fetchPlugins()` 一次
- **AND** 不设置 `panelOpen` 为 true
