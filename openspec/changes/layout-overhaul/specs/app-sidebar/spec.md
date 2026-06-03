## ADDED Requirements

### Requirement: Sidebar structure
AppSidebar SHALL 包含三个区域：顶部操作区（固定）、中间滚动列表区、底部固定区（Settings）。整体宽度由 `--sidebar-w`（默认 210px）控制，背景色 `--bg-shell`。

#### Scenario: Sidebar rendering
- **WHEN** AppSidebar 处于展开状态
- **THEN** 渲染 210px 宽的侧边栏，包含顶部操作按钮、中间分组列表、底部 Settings 按钮
- **AND** 中间列表区域可滚动，滚动条宽 3px

### Requirement: Top action buttons
顶部操作区 SHALL 包含四个文字+图标按钮（垂直排列，每项 padding 6px 10px，圆角 6px）：New chat、Search、Plugins、Automations。

#### Scenario: New chat action
- **WHEN** 用户点击 "New chat" 按钮
- **THEN** 创建新会话并切换到该会话

#### Scenario: Search action
- **WHEN** 用户点击 "Search" 按钮
- **THEN** 展开搜索输入框，可以搜索会话历史

#### Scenario: Plugins and Automations
- **WHEN** 用户点击 "Plugins" 或 "Automations" 按钮
- **THEN** 显示 ComingSoon 占位页面

### Requirement: Grouped list sections
中间列表区 SHALL 支持分组显示，包含 "Pinned"（固定的会话）、"Projects"（项目列表）、"Chats"（普通会话）三个分组。每个分组有标题（11px, 500 weight, 灰色）。

#### Scenario: Pinned sessions display
- **WHEN** 存在被固定的会话
- **THEN** 在 "Pinned" 分组下显示这些会话，每项包含图标、标题（可截断）、时间标注

#### Scenario: Projects display
- **WHEN** 存在关联了工作目录的会话
- **THEN** 在 "Projects" 分组下按工作目录分组显示，每项有彩色圆点和项目名称

#### Scenario: Active session highlight
- **WHEN** 用户选中某个会话
- **THEN** 该项使用 `--bg-active` 背景高亮，字色 `--text-1`，font-weight 500

### Requirement: Session list item
每个会话列表项 SHALL 显示：图标（16px 宽居中）、标题（flex-1, 单行截断）、时间标注（11px, 灰色, 靠右）。整体 padding 5px 10px，圆角 6px，hover 时 `--bg-hover` 背景。

#### Scenario: Session item interaction
- **WHEN** 用户点击会话列表项
- **THEN** 切换到该会话，ChatPane 加载对应的消息流
- **AND** 该项变为 active 高亮状态

### Requirement: Bottom settings entry
底部区域 SHALL 显示 Settings 按钮，上方有 `1px solid var(--border)` 分隔线。点击打开 SettingsPanel。

#### Scenario: Open settings
- **WHEN** 用户点击 Settings 按钮
- **THEN** 打开 SettingsPanel 对话框（复用现有组件）

### Requirement: Sidebar collapse
AppSidebar SHALL 支持折叠到 0px 宽度，通过 AppHeader 中的 toggle 按钮控制。折叠使用 `width: 0` + `overflow: hidden` 实现，带过渡动画。

#### Scenario: Collapse sidebar
- **WHEN** 用户点击 AppHeader 中的 Sidebar toggle 按钮
- **THEN** AppSidebar 动画收缩到 0px，ContentBlock 扩展占满空间

#### Scenario: Expand sidebar
- **WHEN** AppSidebar 处于折叠状态，用户点击 toggle 按钮
- **THEN** AppSidebar 动画展开到 `--sidebar-w` 宽度
