## ADDED Requirements

### Requirement: search.query WS API
后端 SHALL 提供 `search.query` WebSocket 方法，对 `messages_fts` 执行全文检索并返回排序结果。

#### Scenario: Basic keyword search
- **WHEN** 收到 `search.query { q: "rust async" }`
- **AND** `q` 经 trim 后非空
- **THEN** 对 `messages_fts` 执行 FTS5 `MATCH` 查询
- **AND** 返回 `{ results: [...], total, has_more, page: 0 }`
- **AND** 每条 result 含 `session_id`, `turn_id`, `role`, `session_title`, `snippet`, `timestamp`, `work_dir`（可 null）, `rank`

#### Scenario: Search with pagination
- **WHEN** 收到 `search.query { q: "error", page: 1, limit: 10 }`
- **THEN** 返回第 11–20 条匹配（按 bm25 排序）
- **AND** `has_more` 反映是否仍有后续页

#### Scenario: Search with filters
- **WHEN** 收到 `search.query { q: "fix", filters: { date_from: "2026-01-01", date_to: "2026-06-30", work_dir: "/home/user/proj" } }`
- **THEN** SQL 在 JOIN `sessions` 后应用日期与 `work_dir` 约束
- **AND** 仅返回满足筛选的命中

#### Scenario: Empty query rejected
- **WHEN** 收到 `search.query { q: "" }` 或 `q` 仅空白
- **THEN** 返回 `{ results: [], total: 0, has_more: false }` 或 validation 错误，不扫描全表

#### Scenario: Default limit cap
- **WHEN** 请求未指定 `limit` 或 `limit` 大于 10
- **THEN** 使用 `limit = 10` 作为上限

#### Scenario: Query timeout
- **WHEN** FTS 查询超过 2 秒
- **THEN** 返回错误 `{ error: "search_timeout" }`，不挂起连接

### Requirement: search.index_status WS API
后端 SHALL 提供 `search.index_status` 方法，报告全文索引进度与规模。

#### Scenario: Get status while indexing
- **WHEN** 收到 `search.index_status`
- **AND** 后台 bulk index 正在进行
- **THEN** 返回 `{ indexed_count, total_count, is_indexing: true }`
- **AND** `indexed_count` ≤ `total_count`

#### Scenario: Get status when complete
- **WHEN** bulk index 已完成
- **THEN** 返回 `{ indexed_count, total_count, is_indexing: false }`
- **AND** `indexed_count` 等于 `total_count`

#### Scenario: Status with zero messages
- **WHEN** 数据库无历史消息
- **THEN** 返回 `{ indexed_count: 0, total_count: 0, is_indexing: false }`

### Requirement: WS handler registration
`search.query` 与 `search.index_status` SHALL 注册到 gateway WebSocket 方法路由，与现有 `compute_level.*` / `git.*` 模式一致。

#### Scenario: Handler dispatch
- **WHEN** 客户端发送 JSON-RPC 风格 WS 消息 `method: "search.query"`
- **THEN** `xiaolin-gateway/src/ws/search.rs` 处理器被调用
- **AND** 使用共享 `SqlitePool` / `SearchIndex` 实例

#### Scenario: Index status broadcast optional
- **WHEN** bulk index 进度跨越大阈值（如每 5%）
- **THEN** 可选广播 `search.index_progress` 事件供前端更新进度条（实现阶段可选）
