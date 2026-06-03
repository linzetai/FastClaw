## ADDED Requirements

### Requirement: Header bar layout
AppHeader SHALL 为 44px 高的水平栏，包含以下区域（从左到右）：窗口控制按钮组、导航工具栏、中间标题区、右侧功能区。整个 Header 区域支持窗口拖拽（`-webkit-app-region: drag`），按钮区域设置 `no-drag`。

#### Scenario: Header rendering
- **WHEN** 应用加载完成
- **THEN** AppHeader 渲染在页面顶部，高度 44px，背景色 `--bg-shell`
- **AND** 包含窗口控制、导航工具、标题、右侧功能区四个区域

### Requirement: Window traffic lights (Linux style)
AppHeader 左侧 SHALL 显示三个窗口控制按钮（关闭/最小化/最大化），样式为圆形彩色点（红/黄/绿，直径 12px），间距 8px。

#### Scenario: Traffic lights display
- **WHEN** 应用运行在 Tauri 环境
- **THEN** AppHeader 左侧显示红（#ff5f56）、黄（#ffbd2e）、绿（#27c93f）三个圆形按钮
- **AND** 红色按钮关闭窗口，黄色最小化，绿色最大化/还原

#### Scenario: Non-Tauri environment
- **WHEN** 应用运行在浏览器
- **THEN** 不显示窗口控制按钮

### Requirement: Navigation toolbar
窗口控制右侧 SHALL 显示导航工具栏，包含 Sidebar 切换、Panel 切换、前进、后退四个图标按钮（28x28px，圆角 6px）。

#### Scenario: Sidebar toggle button
- **WHEN** 用户点击 Sidebar 切换按钮
- **THEN** AppSidebar 在展开和折叠之间切换

#### Scenario: WorkspacePanel toggle button
- **WHEN** 用户点击 Panel 切换按钮
- **THEN** WorkspacePanel 在展开和关闭之间切换

### Requirement: Center title area
AppHeader 中间 SHALL 显示当前会话标题（13px, 600 weight）和项目名称（12px, 400 weight, 灰色），两者之间以间隔分隔。右侧有一个更多选项按钮（`···`）。

#### Scenario: Title display
- **WHEN** 用户在某个会话中
- **THEN** AppHeader 中间显示会话标题（粗体）和关联的项目名称（灰色）

#### Scenario: No active session
- **WHEN** 没有活跃会话
- **THEN** 标题区域显示应用名 "XiaoLin"

### Requirement: Right function area
AppHeader 右侧 SHALL 包含（从左到右）：主题切换按钮、下拉箭头、Git 统计数字（`+N -N`，预留）、Commit 按钮（预留，disabled）、布局切换按钮组（单栏/分栏）。

#### Scenario: Theme toggle
- **WHEN** 用户点击主题切换按钮（太阳图标）
- **THEN** 在 light 和 dark 模式之间切换

#### Scenario: Git stats placeholder
- **WHEN** 应用渲染 AppHeader
- **THEN** Git 统计区域显示 `+0 -0`（灰显），Commit 按钮显示但处于 disabled 状态
- **AND** 鼠标悬浮 Commit 按钮时显示 tooltip "Git 集成即将推出"

#### Scenario: Layout toggle
- **WHEN** 用户点击分栏布局按钮
- **THEN** WorkspacePanel 打开（或关闭），实现单栏/分栏切换
