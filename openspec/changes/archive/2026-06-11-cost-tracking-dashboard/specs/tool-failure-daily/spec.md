## ADDED Requirements

### Requirement: Tool call results persisted daily
系统 SHALL 在每次 tool call 完成后，将结果（成功/失败/耗时）异步写入 SQLite `tool_call_daily` 表，按 (date, tool_name) 聚合。

#### Scenario: Successful tool call
- **WHEN** tool call 成功执行
- **THEN** 对应行 success_count += 1, total_duration_ms += 实际耗时

#### Scenario: Failed tool call
- **WHEN** tool call 执行失败
- **THEN** 对应行 failure_count += 1, total_duration_ms += 实际耗时

#### Scenario: First call of the day for a tool
- **WHEN** 当天该 tool 无记录
- **THEN** INSERT 新行

### Requirement: Tool failure data available for developer inspection
前端 CostDashboard 的工具健康度表格 SHALL 展示失败率排行，帮助开发者识别需要优化的工具。

#### Scenario: Identify problematic tool
- **WHEN** web_fetch 过去 7 天失败率 40%
- **THEN** 表格中 web_fetch 排在高失败率位置，开发者可据此优化 tool description 或修复 bug
