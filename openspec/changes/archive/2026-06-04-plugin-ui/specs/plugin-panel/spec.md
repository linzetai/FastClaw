## ADDED Requirements

### Requirement: Plugin panel entry
系统 SHALL 提供 `PluginPanel` 组件，以 modal/overlay 形式展示，由 Sidebar「Plugins」按钮或等效 action 打开，由 ESC、关闭按钮或点击遮罩关闭。

#### Scenario: Open plugin panel from sidebar
- **WHEN** 用户点击 AppSidebar 顶部的「Plugins」按钮
- **THEN** `PluginPanel` overlay 显示在当前视图之上
- **AND** 面板标题为「Plugins」或「MCP Plugins」

#### Scenario: Close plugin panel
- **WHEN** 用户按下 ESC 或点击关闭按钮或遮罩层
- **THEN** `PluginPanel` 关闭且主界面恢复交互

### Requirement: Plugin list display
`PluginPanel` SHALL 展示已配置 MCP 插件列表；每项包含：显示名称（id）、状态徽章、scope 标签（user / project）、启用/禁用 toggle。

#### Scenario: List shows configured plugins
- **WHEN** `plugins.list` 返回至少一条插件
- **THEN** 列表渲染所有插件行，按名称排序
- **AND** 每行显示 id 作为标题、scope 标签、状态徽章、enable toggle

#### Scenario: Status badge connected
- **WHEN** 插件 `status` 为 `connected`
- **THEN** 显示绿色徽章（文案如「Connected」）

#### Scenario: Status badge error
- **WHEN** 插件 `status` 为 `failed` 且存在 `lastError`
- **THEN** 显示红色徽章（文案如「Error」）

#### Scenario: Status badge disabled
- **WHEN** 插件 `enabled` 为 false 或 `status` 为 `disabled`
- **THEN** 显示灰色徽章（文案如「Disabled」）

#### Scenario: Status badge connecting
- **WHEN** 插件 `status` 为 `connecting`
- **THEN** 显示中性/黄色进行中指示（可选 spinner）

### Requirement: Enable/disable toggle
每行 SHALL 提供 enable/disable toggle；切换时调用 `plugins.enable` 或 `plugins.disable`，显示 loading 状态直至响应或失败回滚。

#### Scenario: Disable connected plugin
- **WHEN** 用户将已连接插件的 toggle 设为 off
- **THEN** 调用 `plugins.disable { id }`
- **AND** 成功后该行状态变为 disabled，徽章变灰

#### Scenario: Enable disabled plugin
- **WHEN** 用户将已禁用插件的 toggle 设为 on
- **THEN** 调用 `plugins.enable { id }`
- **AND** 成功后触发重连，状态经 `connecting` 变为 `connected` 或 `failed`

#### Scenario: Toggle failure rollback
- **WHEN** enable/disable WS 调用返回错误
- **THEN** toggle 回滚到操作前状态
- **AND** 显示错误 toast 或行内错误提示

### Requirement: Plugin detail expand
用户 SHALL 可展开某插件行查看详情：可用 tools 列表、连接信息（command、args 摘要、transport、url）、`lastError`、`connectedAt`、`toolCount`。

#### Scenario: Expand row loads detail
- **WHEN** 用户展开某插件行
- **THEN** 若本地无缓存则调用 `plugins.tools { id }` 或复用 list 中已有字段
- **AND** 展示 tools 列表（name + description 截断）
- **AND** 展示 connection info（env 值已脱敏）

#### Scenario: Collapse detail
- **WHEN** 用户再次点击展开控件
- **THEN** 详情区域收起

### Requirement: Quick actions
详情区或行操作区 SHALL 提供「Restart」与「View logs」快捷操作。

#### Scenario: Restart plugin
- **WHEN** 用户点击「Restart」
- **THEN** 调用 `plugins.restart { id }`
- **AND** 该行进入 connecting 状态直至 `plugins.status_changed` 或 list 刷新

#### Scenario: View connection logs
- **WHEN** 用户点击「View logs」
- **THEN** 展示该插件最近错误信息（`lastError`）及指向完整日志的说明（v1 可为行内 expandable 文本，非必须 tail 实时日志）

### Requirement: Empty state
当无任何已配置插件时，面板 SHALL 显示空状态文案与引导（例如「在 Settings 中添加 MCP 服务器」或「编辑 .xiaolin/mcp.json」）。

#### Scenario: No plugins configured
- **WHEN** `plugins.list` 返回空数组
- **THEN** 显示空状态插图/文案
- **AND** 提供跳转 Settings MCP 区域的链接或按钮（若 Settings 已支持）

### Requirement: Link to settings for add
v1 SHALL 不在面板内实现完整「添加插件」表单；可提供「在 Settings 中管理」入口。

#### Scenario: Open settings from panel
- **WHEN** 用户点击「Manage in Settings」
- **THEN** 关闭或保持 PluginPanel，并打开 Settings 至 MCP 相关 Tab
