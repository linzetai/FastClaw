## ADDED Requirements

### Requirement: Four-region layout architecture
应用 SHALL 采用四区域布局架构：AppHeader（顶部）+ AppSidebar（左侧）+ ChatPane（中间）+ WorkspacePanel（右侧），其中 ChatPane 和 WorkspacePanel 包裹在统一的 ContentBlock 容器内。

#### Scenario: Default layout rendering
- **WHEN** 应用启动并完成加载，且活跃会话有消息
- **THEN** 渲染 AppHeader（44px 高）+ AppSidebar（210px 宽）+ ContentBlock（flex-1，内含 ChatPane + WorkspacePanel）
- **AND** ContentBlock 使用 `--bg-card` 背景色，AppSidebar 和 AppHeader 使用 `--bg-shell` 背景色

#### Scenario: Layout without WorkspacePanel
- **WHEN** WorkspacePanel 处于关闭状态
- **THEN** ContentBlock 仅包含 ChatPane，占满剩余宽度
- **AND** ContentBlock 的 border-radius 改为四角圆角 `var(--card-r)`

### Requirement: ContentBlock unified card surface
ChatPane 和 WorkspacePanel SHALL 共享同一个卡片容器（ContentBlock），使用统一的 `--bg-card` 背景和 `border-radius: var(--card-r) 0 0 var(--card-r)` 圆角。WorkspacePanel 与 ChatPane 之间使用 `border-left: 1px solid var(--border)` 分隔。

#### Scenario: Visual separation between Chat and Workspace
- **WHEN** WorkspacePanel 打开
- **THEN** ChatPane 和 WorkspacePanel 在同一白色卡片内，WorkspacePanel 左侧有 1px 的 `--border` 色分隔线
- **AND** 外层 ContentBlock 的右侧无圆角（贴合窗口右边缘）

### Requirement: Empty state layout
当活跃会话无消息（或没有选中会话）时，ContentBlock SHALL 仅显示 ChatPane，不显示 WorkspacePanel，且 ChatPane 内渲染居中 WelcomeView 而非 MessageStream。

#### Scenario: New chat empty state
- **WHEN** 活跃会话没有任何消息（或没有选中会话）
- **THEN** ContentBlock 渲染 ChatPane，内含居中 WelcomeView
- **AND** 不显示 WorkspacePanel
- **AND** ContentBlock 使用四角圆角 `var(--card-r)`

#### Scenario: WelcomeView content
- **WHEN** 处于空状态布局
- **THEN** WelcomeView 显示大标题 "What should we build in {project-name}?"（project-name 来自当前 session 的 work_dir 目录名，无 work_dir 时显示通用标题）
- **AND** 居中放置 ChatInputBar（复用同一组件，居中位置而非底部固定）
- **AND** 输入框下方显示 2-3 个快捷操作建议卡片

#### Scenario: Suggestion cards data source
- **WHEN** WelcomeView 需要渲染建议卡片
- **THEN** v1 使用硬编码的静态建议列表（3 项，如 "开始构建新功能"、"审查最近的代码变更"、"连接插件扩展能力"）
- **AND** 每个建议卡片包含图标 + 一句话描述，点击后将描述文字填入 InputBar
- **AND** 后续可替换为动态数据源（基于 project 上下文、历史对话、memory 推荐）

#### Scenario: Transition to message stream
- **WHEN** 用户在空状态会话中发送第一条消息
- **THEN** WelcomeView 切换为 MessageStream
- **AND** WorkspacePanel 仍保持隐藏，直至有文件变更或用户显式打开

### Requirement: Shell background layer
AppSidebar 和 AppHeader SHALL 使用 `--bg-shell` 背景色（light 模式下为浅灰，dark 模式下为深色），形成与 ContentBlock 的白色卡片之间的视觉层次。

#### Scenario: Light mode background contrast
- **WHEN** 主题为 light 模式
- **THEN** AppSidebar 和 AppHeader 背景为 `--bg-shell`（浅灰），ContentBlock 为 `--bg-card`（白色），两者形成清晰的层次对比

#### Scenario: Dark mode background contrast
- **WHEN** 主题为 dark 模式
- **THEN** AppSidebar 和 AppHeader 背景为 `--bg-shell`（深色），ContentBlock 为 `--bg-card`（略浅深色），保持层次对比

### Requirement: Window resize handles
应用 SHALL 在非最大化状态下提供窗口边缘和角落的拖拽调整大小功能（保持现有 WindowResizeHandles 行为）。

#### Scenario: Resize handle presence
- **WHEN** 窗口未最大化
- **THEN** 窗口四边和四角显示不可见的拖拽手柄（hit area 5px）

#### Scenario: Maximized state
- **WHEN** 窗口最大化
- **THEN** 不渲染拖拽手柄

### Requirement: CSS variable integration
新增的布局变量 SHALL 映射到现有主题系统的 CSS 变量，确保 accent preset 切换时自动跟随。

#### Scenario: Theme variables defined
- **WHEN** 应用加载时
- **THEN** 以下 CSS 变量在 `[data-theme]` 选择器下可用：`--bg-shell`、`--bg-card`、`--bg-user-msg`、`--bg-hover`、`--bg-active`、`--bg-elevated`、`--bg-code`、`--bg-input-border`、`--border`、`--border-subtle`、`--header-h`（44px）、`--sidebar-w`（210px）、`--panel-w`（360px）、`--card-r`（12px）
