## ADDED Requirements

### Requirement: Three-tier responsive breakpoints
布局 SHALL 根据主内容区宽度自动适配三个等级：紧凑（< 600px）、标准（600-900px）、宽屏（> 900px）。

#### Scenario: Standard layout
- **WHEN** 主内容区宽度为 750px
- **THEN** NavRail 显示（54px）、SessionList 可见、消息区 maxWidth 为 720px

#### Scenario: Wide layout
- **WHEN** 主内容区宽度为 1100px
- **THEN** NavRail 显示（54px）、SessionList 可见、消息区 maxWidth 扩展为 min(860px, 容器宽度-40px)

#### Scenario: Compact layout
- **WHEN** 主内容区宽度小于 600px
- **THEN** NavRail 折叠为 hamburger 按钮、SessionList 折叠为 overlay、消息区 maxWidth 为 100%-32px

### Requirement: NavRail compact mode
紧凑模式下 NavRail SHALL 折叠，通过 hamburger 菜单按钮访问导航项。

#### Scenario: NavRail collapses in compact
- **WHEN** 窗口宽度导致主内容区小于 600px
- **THEN** NavRail 不占据布局空间，左上角显示一个 hamburger 图标按钮

#### Scenario: Hamburger menu opens nav
- **WHEN** 用户在紧凑模式下点击 hamburger 按钮
- **THEN** 导航项以 overlay 弹出（类似移动端侧栏），点击导航项后自动关闭

### Requirement: SessionList overlay in compact
紧凑模式下 SessionList SHALL 以 overlay 方式呈现而非占据布局空间。

#### Scenario: SessionList as overlay
- **WHEN** 窗口处于紧凑模式且用户点击 hamburger 或侧边栏切换按钮
- **THEN** SessionList 以 overlay 形式从左侧滑入，带半透明背景遮罩

#### Scenario: Overlay auto-close
- **WHEN** 用户在 overlay 模式的 SessionList 中选择一个会话
- **THEN** SessionList overlay 自动关闭，主内容区显示选中的会话

### Requirement: Message area adaptive width
消息区 maxWidth SHALL 根据布局等级自适应。

#### Scenario: Wide screen utilization
- **WHEN** 窗口足够宽（主内容区 > 900px）
- **THEN** 消息内容 maxWidth 扩展到 860px 以充分利用空间

#### Scenario: Narrow content area
- **WHEN** 主内容区宽度 < 600px
- **THEN** 消息内容占满容器宽度（减去 16px 两侧 padding）

### Requirement: ResizeObserver-based detection
响应式断点 SHALL 通过 `ResizeObserver` 监听主内容区实际宽度实现，不使用 CSS media query。

#### Scenario: Dynamic resize detection
- **WHEN** 用户拖拽调整窗口大小，主内容区从 800px 缩小到 550px
- **THEN** 布局自动从标准模式切换到紧凑模式，无需刷新
