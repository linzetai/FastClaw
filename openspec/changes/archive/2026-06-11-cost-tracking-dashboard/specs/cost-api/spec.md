## ADDED Requirements

### Requirement: Daily cost query endpoint
系统 SHALL 提供 `GET /api/v1/cost/daily?from=YYYY-MM-DD&to=YYYY-MM-DD` 端点，返回指定日期范围内按天+模型的 token 消耗数据。

#### Scenario: Query 7 days with multiple models
- **WHEN** GET /api/v1/cost/daily?from=2026-06-04&to=2026-06-11
- **THEN** 返回 JSON 数组，每项包含 date, model, input_tokens, output_tokens, cache_read_tokens, cost_usd, call_count
- **AND** 结果按 date ASC 排序

#### Scenario: No data in range
- **WHEN** 指定范围内无数据
- **THEN** 返回空数组 []

### Requirement: Tool stats query endpoint
系统 SHALL 提供 `GET /api/v1/cost/tools?from=YYYY-MM-DD&to=YYYY-MM-DD` 端点，返回工具调用统计。

#### Scenario: Query tool stats
- **WHEN** GET /api/v1/cost/tools?from=2026-06-01&to=2026-06-11
- **THEN** 返回 JSON 数组，每项包含 tool_name, success_count, failure_count, total_duration_ms
- **AND** 结果按 (success_count + failure_count) DESC 排序

### Requirement: Session cost list endpoint
系统 SHALL 提供 `GET /api/v1/cost/sessions?limit=N` 端点，返回最近 N 个 session 的成本汇总。

#### Scenario: Query recent sessions
- **WHEN** GET /api/v1/cost/sessions?limit=10
- **THEN** 返回最近 10 个 session 的 session_id, started_at, total_cost_usd, turn_count
- **AND** 按 started_at DESC 排序

### Requirement: Cost summary endpoint
系统 SHALL 提供 `GET /api/v1/cost/summary` 端点，返回全局成本摘要。

#### Scenario: With budget configured
- **WHEN** 配置了 budget_limit_usd = 10.0
- **THEN** 返回 { total_cost_usd, today_cost_usd, budget_limit: 10.0, budget_used_pct: X }

#### Scenario: Without budget
- **WHEN** 未配置 budget
- **THEN** 返回 { total_cost_usd, today_cost_usd, budget_limit: null, budget_used_pct: null }
