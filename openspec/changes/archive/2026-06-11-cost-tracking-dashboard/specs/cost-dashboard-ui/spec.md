## ADDED Requirements

### Requirement: CostDashboard page accessible from sidebar
前端 SHALL 在侧边栏提供「成本」入口，点击后导航到 CostDashboard 页面。

#### Scenario: Navigate to cost dashboard
- **WHEN** 用户点击侧边栏「成本」图标
- **THEN** 主内容区显示 CostDashboard 页面

### Requirement: Date range selector
CostDashboard SHALL 提供日期范围选择器，支持预设（近 7 天、近 30 天）和自定义范围。

#### Scenario: Select last 7 days
- **WHEN** 用户选择「近 7 天」
- **THEN** 图表数据范围为今天往前 7 天

### Requirement: Token consumption trend chart
CostDashboard SHALL 显示按天的 token 消耗折线图，X 轴为日期，Y 轴为 token 数，按模型分色显示。

#### Scenario: Multiple models in range
- **WHEN** 查询范围内使用了 claude-3 和 gpt-4
- **THEN** 折线图有两条线（不同颜色），各自显示每天的 total tokens

### Requirement: Model cost breakdown
CostDashboard SHALL 显示模型成本占比图（饼图或水平柱状图），展示各模型的 cost_usd 占比。

#### Scenario: Display proportions
- **WHEN** claude-3 花费 $2.5，gpt-4 花费 $0.5
- **THEN** 饼图显示 claude-3 占 83%，gpt-4 占 17%

### Requirement: Tool health table
CostDashboard SHALL 显示工具健康度表格，列出每个工具的 total_calls、success_rate、avg_duration_ms。

#### Scenario: Tool with high failure rate
- **WHEN** shell_exec 有 100 次调用，30 次失败
- **THEN** 表格显示 success_rate = 70%，可视为需要优化

### Requirement: Budget progress bar
CostDashboard SHALL 在配置了 budget 时显示 budget 进度条。

#### Scenario: Budget at 60%
- **WHEN** budget_limit = $10, total_cost = $6
- **THEN** 进度条填充 60%，颜色为正常（绿/蓝）

#### Scenario: Budget at 85%
- **WHEN** budget_limit = $10, total_cost = $8.5
- **THEN** 进度条填充 85%，颜色变为警告色（橙/黄）

### Requirement: Real-time session cost in header
AppHeader SHALL 可选地显示当前 session 的成本摘要（小数字），由 CostUpdated 事件实时更新。

#### Scenario: Cost updates during session
- **WHEN** CostUpdated 事件将 total_cost_usd 更新为 $0.42
- **THEN** Header 显示 "$0.42"
