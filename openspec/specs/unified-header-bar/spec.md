## MODIFIED Requirements

### Requirement: Merged header bar
ContentHeader 已被删除。原 ChatTabsBar 的水平 Tab 功能 SHALL 被 AppHeader 的中间标题区域替代。Header 不再显示横排 Tab，改为显示当前活跃会话的标题和项目名称。会话切换通过 AppSidebar 进行。

#### Scenario: Single header rendering
- **WHEN** 应用加载完成显示对话视图
- **THEN** 顶部只有一个 44px 高的 AppHeader，包含窗口控制、导航工具、标题、功能区
- **AND** 没有独立的 ChatTabsBar 组件

### Requirement: Sidebar toggle in header
侧边栏折叠时 SHALL 在 AppHeader 导航工具栏中通过 Sidebar 切换按钮控制展开（图标为带左竖线的矩形）。

#### Scenario: Show toggle always
- **WHEN** 应用渲染 AppHeader
- **THEN** 导航工具栏始终显示 Sidebar 切换按钮
- **AND** 点击切换 AppSidebar 的展开/折叠状态

## REMOVED Requirements

### Requirement: Horizontal scrollable tabs
**Reason**: 新布局不使用水平 Tab 栏，会话切换改为通过 AppSidebar 列表完成
**Migration**: 用户通过 AppSidebar 的会话列表切换会话，不再需要横排 Tab

### Requirement: Tab overflow indicator
**Reason**: 水平 Tab 栏已移除，不再需要溢出计数器
**Migration**: 所有会话在 AppSidebar 的可滚动列表中显示

### Requirement: Tab drag reorder
**Reason**: 水平 Tab 栏已移除
**Migration**: 会话排序通过 AppSidebar 中的 Pinned/时间排序实现

### Requirement: Streaming and attention indicators on tabs
**Reason**: Tab 栏已移除，但 streaming 和 attention 状态指示器 SHALL 移到 AppSidebar 的会话列表项中
**Migration**: 在 AppSidebar 的会话列表项中，streaming 会话显示蓝色脉冲点，需要 attention 的会话显示橙色脉冲点
