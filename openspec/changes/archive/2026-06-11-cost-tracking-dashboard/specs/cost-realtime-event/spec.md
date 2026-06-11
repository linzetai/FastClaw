## ADDED Requirements

### Requirement: CostUpdated event pushed after each turn
系统 SHALL 在每个 turn 的 LLM 调用完成后，通过 WebSocket 向前端推送 CostUpdated 事件。

#### Scenario: Normal turn completion
- **WHEN** agent turn 完成，LLM response 包含 token_usage
- **THEN** 推送 CostUpdated 事件包含 session_id, turn_index, delta_cost_usd, total_cost_usd, model, input_tokens, output_tokens

#### Scenario: Turn with no LLM call (tool-only)
- **WHEN** turn 没有产生 LLM 调用（纯工具执行）
- **THEN** 不推送 CostUpdated 事件

### Requirement: Frontend receives and displays real-time cost
前端 cost-store SHALL 监听 CostUpdated 事件并实时更新当前 session 的成本显示。

#### Scenario: Multiple turns in rapid succession
- **WHEN** 收到连续 3 次 CostUpdated 事件
- **THEN** cost-store 中 total_cost_usd 等于最后一次事件的 total_cost_usd 值
