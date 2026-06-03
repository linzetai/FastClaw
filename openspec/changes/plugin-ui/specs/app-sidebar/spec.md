## MODIFIED Requirements

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
