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
AppHeader 右侧 SHALL 包含（从左到右）：主题切换按钮、下拉箭头按钮、Git 统计数字（`+N -N`）、Commit 按钮、布局切换按钮组（单栏/分栏两个按钮）。Linux 环境下在最右侧附加窗口控制按钮（最小化/最大化/关闭）。

#### Scenario: Theme toggle
- **WHEN** 用户点击主题切换按钮（太阳/月亮图标）
- **THEN** 在 light 和 dark 模式之间切换

#### Scenario: Git stats display
- **WHEN** 应用渲染 AppHeader
- **THEN** Git 统计区域显示 `+N` (使用 `--green-text` 颜色) 和 `-N` (使用 `--red-text` 颜色)，使用 monospace 字体 11px
- **AND** 无 git 变更时显示 `+0` `-0`，仍保持绿色/红色着色（低 opacity 可接受）

#### Scenario: Commit button styling
- **WHEN** 渲染 Commit 按钮
- **THEN** 按钮使用绿色描边样式：`border: 1.5px solid var(--green-text)`，文字颜色 `var(--green-text)`，背景透明
- **AND** 按钮包含菱形图标 + "Commit" 文字 + 下拉箭头
- **AND** hover 时背景 `rgba(52,199,89,0.08)`
- **AND** 无 git 数据时按钮为 disabled 状态，保持灰色边框和文字

#### Scenario: Layout toggle buttons
- **WHEN** 渲染布局切换区域
- **THEN** 显示两个按钮：单栏视图（矩形图标）和分栏视图（竖线分割矩形图标）
- **AND** 当前激活的布局按钮高亮显示
- **AND** 点击分栏按钮打开 WorkspacePanel，点击单栏按钮关闭
