## ADDED Requirements

### Requirement: Bottom drawer mode
SubAgentMonitor SHALL 以底部抽屉形式呈现，叠加在消息区上方，不占据水平布局空间。

#### Scenario: No sub-agents
- **WHEN** 没有活跃的子智能体运行
- **THEN** SubAgentMonitor 完全不渲染，消息区占满水平空间

#### Scenario: Sub-agent starts
- **WHEN** 有子智能体开始运行
- **THEN** 底部抽屉从 StreamFooter 上方滑入，高度默认 240px

### Requirement: Collapsible summary bar
用户 SHALL 能够将抽屉折叠为一行摘要栏。

#### Scenario: Collapse to summary
- **WHEN** 用户点击抽屉顶部的折叠按钮
- **THEN** 抽屉缩小为约 36px 高的摘要栏，显示活跃子智能体数量和当前状态

#### Scenario: Expand from summary
- **WHEN** 用户点击摘要栏
- **THEN** 抽屉恢复到折叠前的高度，显示完整的运行列表

### Requirement: Draggable height
抽屉 SHALL 支持拖拽顶部边缘调整高度，范围 120px 到 400px。

#### Scenario: Drag to resize
- **WHEN** 用户拖拽抽屉顶部边缘向上
- **THEN** 抽屉高度增大，消息区可视区域相应缩小

#### Scenario: Height boundary
- **WHEN** 用户拖拽抽屉高度超过 400px
- **THEN** 高度停留在 400px

### Requirement: Auto-hide after completion
所有子智能体完成运行后 SHALL 在 3 秒延迟后自动收起为摘要栏（非完全隐藏）。

#### Scenario: Auto-collapse after done
- **WHEN** 最后一个活跃子智能体完成
- **THEN** 3 秒后抽屉自动折叠为摘要栏显示完成状态，用户可点击展开查看详情
