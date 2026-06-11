## ADDED Requirements

### Requirement: Token usage persisted to SQLite daily
系统 SHALL 在每次 LLM 调用完成后，将 token 用量异步写入 SQLite `token_usage_daily` 表，按 (date, model) 聚合累加。

#### Scenario: First call of the day for a model
- **WHEN** 当天该 model 无记录
- **THEN** INSERT 新行，各 token 字段为本次调用的值，call_count = 1

#### Scenario: Subsequent call same day same model
- **WHEN** 当天该 model 已有记录
- **THEN** UPDATE 累加 input_tokens, output_tokens, cache_read_tokens, cache_creation_tokens, cost_usd, call_count

#### Scenario: Write failure does not block LLM response
- **WHEN** SQLite 写入失败（磁盘满、锁超时等）
- **THEN** 仅记录 tracing::warn，不影响 CostTracker 内存统计和 LLM streaming

### Requirement: Batch flush for write efficiency
系统 SHALL 使用 batch flush 机制，将 1-2 秒内的多次 record 合并为单次 SQLite 事务写入。

#### Scenario: Multiple calls within flush window
- **WHEN** 1 秒内有 5 次 LLM 调用完成
- **THEN** 合并为 1 次事务包含 5 条 UPSERT 语句

### Requirement: Session cost summary maintained
系统 SHALL 在 session 开始时 INSERT `session_cost_summary`，每个 turn 结束后 UPDATE 累加字段，session 结束时写入 ended_at。

#### Scenario: Session starts
- **WHEN** 新 session 创建
- **THEN** INSERT session_cost_summary 行（started_at = now, 其余为 0）

#### Scenario: Turn completes
- **WHEN** 一个 turn 的 LLM 调用完成
- **THEN** UPDATE total_cost_usd += delta, total_input_tokens += input, total_output_tokens += output, turn_count += 1
