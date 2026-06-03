## ADDED Requirements

### Requirement: Merged header bar
ContentHeader SHALL 被删除，其功能（侧边栏展开按钮、搜索按钮）SHALL 整合到 ChatTabsBar 中，形成统一的顶部栏。

#### Scenario: Single header rendering
- **WHEN** 应用加载完成显示对话视图
- **THEN** 主内容区顶部只有一个 36px 高的 header 栏，包含 Tab 列表和功能按钮

### Requirement: Horizontal scrollable tabs
Tab 栏 SHALL 以横排方式显示所有打开的会话 Tab，当 Tab 总宽度超出容器时支持水平滚动。

#### Scenario: Multiple tabs visible
- **WHEN** 用户打开 3 个会话
- **THEN** 3 个 Tab 横排显示在 header 栏中，每个 Tab 可见标题和关闭按钮

#### Scenario: Overflow scrolling
- **WHEN** 打开的 Tab 总宽度超出 header 栏可用宽度
- **THEN** Tab 栏可水平滚动，两端显示滚动方向箭头

#### Scenario: Active tab auto-scroll
- **WHEN** 用户切换到一个被滚动隐藏的 Tab
- **THEN** Tab 栏自动滚动使激活的 Tab 完全可见

### Requirement: Tab overflow indicator
当打开超过 8 个 Tab 时 SHALL 在末尾显示溢出计数器。

#### Scenario: Overflow count display
- **WHEN** 用户打开 12 个会话 Tab
- **THEN** 可见区域显示尽可能多的 Tab，末尾显示 `+N` 按钮（N 为不可见的 Tab 数），点击展开下拉列表显示所有 Tab

### Requirement: Tab drag reorder
用户 SHALL 能够通过拖拽重新排列 Tab 顺序。

#### Scenario: Drag to reorder
- **WHEN** 用户按住某个 Tab 拖动到另一个位置
- **THEN** Tab 顺序实时更新，拖放指示线显示目标位置

### Requirement: Sidebar toggle in header
侧边栏折叠时 SHALL 在 header 栏左侧显示展开按钮。

#### Scenario: Show toggle when collapsed
- **WHEN** 侧边栏处于折叠状态
- **THEN** header 栏最左侧显示 `PanelLeftOpen` 图标按钮，点击展开侧边栏

#### Scenario: Hide toggle when expanded
- **WHEN** 侧边栏处于展开状态
- **THEN** header 栏不显示侧边栏切换按钮（折叠按钮在 SessionList 内部）

### Requirement: Search button in header
搜索按钮 SHALL 显示在 header 栏右侧。

#### Scenario: Trigger search
- **WHEN** 用户点击 header 栏右侧的搜索图标
- **THEN** 消息区搜索栏展开（行为与现有一致）

### Requirement: Streaming and attention indicators on tabs
每个 Tab SHALL 显示 streaming 和 attention 状态指示器。

#### Scenario: Streaming indicator
- **WHEN** 某个会话正在接收 streaming 响应
- **THEN** 该 Tab 上显示一个蓝色脉冲圆点

#### Scenario: Attention indicator
- **WHEN** 某个后台会话需要用户操作（如回答问题）
- **THEN** 该 Tab 上显示一个橙色脉冲圆点
