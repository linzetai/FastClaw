## ADDED Requirements

### Requirement: useSearchStore Zustand store
前端 SHALL 提供 `useSearchStore`，集中管理全局搜索状态。

#### Scenario: Store initial state
- **WHEN** 应用加载
- **THEN** store 初始值为：`query: ""`, `results: []`, `loading: false`, `filters: {}`, `page: 0`, `total: 0`, `hasMore: false`, `indexStatus: null`

#### Scenario: Store exposes panel visibility
- **WHEN** 需要控制 `SearchPanel` 开关
- **THEN** store 含 `panelOpen: boolean` 与 `openPanel()` / `closePanel()` actions

### Requirement: Debounced search action
store SHALL 提供防抖搜索 action，避免频繁 WS 调用。

#### Scenario: Debounced search triggers WS
- **WHEN** 调用 `setQuery(text)` 且 text 非空
- **THEN** 300ms 后若 query 未变，设置 `loading: true` 并发送 `search.query`
- **AND** 响应到达后更新 `results`, `total`, `hasMore`, `loading: false`

#### Scenario: Cancel stale responses
- **WHEN** 用户在 300ms 内继续输入导致 query 变化
- **THEN** 丢弃旧请求响应（request id 或 abort），仅应用最新 query 的结果

#### Scenario: Clear on empty query
- **WHEN** `setQuery("")`
- **THEN** 取消待发送 debounce、`results` 置空、`loading` 为 false

### Requirement: Filter state management
store SHALL 管理筛选条件并与 `search.query` 请求体同步。

#### Scenario: Set date range filter
- **WHEN** 调用 `setFilters({ dateFrom, dateTo })`
- **THEN** 合并到 `filters` 并在下次 `search` 时附带

#### Scenario: Set work_dir filter
- **WHEN** 调用 `setFilters({ workDir })`
- **THEN** 下次搜索携带 `filters.work_dir`

### Requirement: Navigate-to-result action
store SHALL 提供 `navigateToResult(result)`，协调会话切换与滚动。

#### Scenario: Navigate to result
- **WHEN** 调用 `navigateToResult({ sessionId, turnId })`
- **THEN** 调用 `useChatMetaStore.setActiveChat(sessionId)`
- **AND** 关闭 `panelOpen`
- **AND** 派发 `scrollToTurn(turnId)` 或等价事件供 `MessageStream` 消费

### Requirement: Index status polling
store SHALL 在面板打开或应用启动时拉取索引状态。

#### Scenario: Fetch index status on panel open
- **WHEN** `openPanel()` 被调用
- **THEN** 发送 `search.index_status` 并写入 `indexStatus`
- **AND** 若 `is_indexing` 为 true，每 2s 轮询直至完成（面板关闭时停止轮询）

### Requirement: Pagination action
store SHALL 支持加载更多结果。

#### Scenario: loadMore increments page
- **WHEN** 调用 `loadMore()` 且 `hasMore` 为 true
- **THEN** `page` 加 1 并请求 `search.query` 附加 `page`
- **AND** 将新 `results` 追加到现有列表
