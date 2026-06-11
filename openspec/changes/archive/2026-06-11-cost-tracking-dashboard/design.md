## Context

XiaoLin 已有完整的实时成本追踪基础设施：
- `CostTracker`（`runtime/cost_tracker.rs`）：per-model 累计 token + USD，支持 budget alert
- `MetricsCollector`（`xiaolin-observe`）：Prometheus 格式输出 counters/histograms
- `RuntimeObserver`（`runtime/observer.rs`）：per-session tool call 追踪（name, success, duration）
- `GoalPanel.tsx` / `GoalStatusCard.tsx`：前端已有 goal 状态展示组件

缺失的是**历史持久化**（当前数据在进程重启后丢失）和**前端成本页面**。

参考：Claude Code 的 `statsCache.ts` 使用 JSON 文件按天聚合 `DailyActivity` + `DailyModelTokens`，支持增量合并和 session 恢复。XiaoLin 选择 SQLite（已有基础设施）而非 JSON 文件。

## Goals / Non-Goals

**Goals:**
- 每次 LLM 调用的 token 消耗持久化到 SQLite，按天+模型聚合
- 每次 tool call 的成功/失败/耗时持久化，按天+工具聚合
- 提供 REST API 查询历史数据（分天/分模型/分工具/分 session）
- Turn 结束后通过 WebSocket 推送 CostUpdated 事件
- 前端提供 CostDashboard 页面展示趋势和统计
- Goal 模式有 budget 时到达上限 pause，无 budget 时不限制

**Non-Goals:**
- 不做 ToolProfile 自适应（不根据失败率自动 demote 工具）
- 不做按轮次的细粒度持久化（只按天聚合）
- 不做跨实例同步（单机 SQLite 足够）
- 不做计费/付费系统

## Decisions

### D1: SQLite 按天聚合而非按 turn 记录

**选择**：使用 `INSERT OR REPLACE` / `ON CONFLICT UPDATE` 将同一天同一模型的 token 累加到一行。

**替代方案**：
- 每个 LLM 调用插一行 → 数据量膨胀快，查询成本高
- JSON 文件（Claude Code 方式）→ 无事务保证，并发不安全

**理由**：按天聚合大幅减少数据量（一天最多 model_count 行），查询高效，符合"追溯分天消耗"的需求。

### D2: 异步写入，不阻塞 LLM 响应流

**选择**：CostTracker.record() 返回后，通过 tokio::spawn 异步写入 SQLite。

**理由**：持久化是 best-effort，不能影响 streaming 性能。即使写入偶尔失败（磁盘满等），内存统计仍然准确。

### D3: 复用现有 SQLite 连接池

XiaoLin 已有 `unified-sqlite-pool` spec，复用同一个 pool 来避免 WAL 锁冲突。

### D4: WebSocket CostUpdated 事件结构

```rust
CostUpdated {
    session_id: String,
    turn_index: u32,
    delta_cost_usd: f64,
    total_cost_usd: f64,
    model: String,
    input_tokens: u32,
    output_tokens: u32,
}
```

每个 turn 结束后推送一次，前端据此更新实时显示。

### D5: 前端图表库选择

**选择**：使用轻量级 SVG 图表（手写或 `recharts`），不引入重量级 chart 框架。

**理由**：XiaoLin 前端已使用 React + TailwindCSS，recharts 体积小且 React 原生集成。Dashboard 图表需求不复杂（折线图 + 饼图 + 表格），不需要 D3.js 级别的灵活性。

### D6: Goal Budget 行为

- 有 budget 设置 → 达到时注入 budget_limit prompt + 自动 pause goal + 推送 GoalPaused 事件
- 无 budget 设置 → 无限制，直到 goal complete 或用户手动 stop

### D7: 工具失败记录维度

按天+工具名聚合：success_count, failure_count, total_duration_ms。用途：
1. 开发者在 Dashboard 查看哪些工具失败率高
2. 后续手动优化 tool description
3. 不自动 demote——只提供数据供人决策

## Schema

```sql
-- 按天+模型聚合的 token 消耗
CREATE TABLE IF NOT EXISTS token_usage_daily (
    date                  TEXT NOT NULL,
    model                 TEXT NOT NULL,
    input_tokens          INTEGER NOT NULL DEFAULT 0,
    output_tokens         INTEGER NOT NULL DEFAULT 0,
    cache_read_tokens     INTEGER NOT NULL DEFAULT 0,
    cache_creation_tokens INTEGER NOT NULL DEFAULT 0,
    cost_usd              REAL NOT NULL DEFAULT 0.0,
    call_count            INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (date, model)
);

-- 按天+工具聚合的调用统计
CREATE TABLE IF NOT EXISTS tool_call_daily (
    date              TEXT NOT NULL,
    tool_name         TEXT NOT NULL,
    success_count     INTEGER NOT NULL DEFAULT 0,
    failure_count     INTEGER NOT NULL DEFAULT 0,
    total_duration_ms INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (date, tool_name)
);

-- Session 级成本汇总
CREATE TABLE IF NOT EXISTS session_cost_summary (
    session_id         TEXT PRIMARY KEY,
    started_at         TEXT NOT NULL,
    ended_at           TEXT,
    total_cost_usd     REAL NOT NULL DEFAULT 0.0,
    total_input_tokens INTEGER NOT NULL DEFAULT 0,
    total_output_tokens INTEGER NOT NULL DEFAULT 0,
    turn_count         INTEGER NOT NULL DEFAULT 0,
    model_breakdown    TEXT  -- JSON: { "model_name": { input, output, cost } }
);
```

## REST API

| Method | Path | Response |
|--------|------|----------|
| GET | `/api/v1/cost/daily?from=YYYY-MM-DD&to=YYYY-MM-DD` | `[{ date, model, input_tokens, output_tokens, cost_usd, call_count }]` |
| GET | `/api/v1/cost/tools?from=YYYY-MM-DD&to=YYYY-MM-DD` | `[{ tool_name, success_count, failure_count, avg_duration_ms }]` |
| GET | `/api/v1/cost/sessions?limit=N` | `[{ session_id, started_at, total_cost_usd, turn_count }]` |
| GET | `/api/v1/cost/summary` | `{ total_cost_usd, today_cost_usd, budget_limit, budget_used_pct }` |

## Risks / Trade-offs

- **SQLite 写入性能** → 高频 LLM 调用时大量 INSERT：使用异步 batch 写入（攒 1-2 秒后批量 flush）
- **数据量增长** → 长期运行后表膨胀：按天聚合天然限制行数（365 天 × N 模型 ≈ 千行级），无需清理
- **进程崩溃丢数据** → 异步写入窗口内的数据可能丢失：可接受，内存统计为权威源
- **图表库体积** → recharts ~50KB gzip：相比完整 bundle 可接受
- **时区问题** → "今天"的定义：统一使用本地时区的 YYYY-MM-DD
