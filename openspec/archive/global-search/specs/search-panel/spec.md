## ADDED Requirements

### Requirement: Global search panel entry
系统 SHALL 提供全局搜索面板组件 `SearchPanel`，由侧边栏 Search 按钮或快捷键打开。

#### Scenario: Open search panel from sidebar
- **WHEN** 用户点击侧边栏 Search 按钮（放大镜图标）
- **THEN** 显示 `SearchPanel`（overlay 或替换侧边栏列表区域）
- **AND** 搜索输入框获得焦点

#### Scenario: Open via keyboard shortcut
- **WHEN** 用户按下 Cmd+K（macOS）或 Ctrl+K（Windows/Linux）
- **THEN** 打开 `SearchPanel` 并聚焦输入框
- **AND** 快捷键不与浏览器默认行为冲突（在 WebView 内 `preventDefault`）

#### Scenario: Close search panel
- **WHEN** 用户按 Escape 或点击面板外关闭控件
- **THEN** 关闭 `SearchPanel` 并恢复侧边栏会话列表

### Requirement: Search input with debounce
`SearchPanel` SHALL 提供搜索输入框，通过 store 触发防抖查询。

#### Scenario: Debounced query submission
- **WHEN** 用户在输入框键入关键词
- **THEN** 300ms 无新输入后才调用 `search.query`
- **AND** 输入清空时不发起查询并清空结果列表

#### Scenario: Loading indicator during query
- **WHEN** `search.query` 请求进行中
- **THEN** 输入框旁或结果区显示 loading 状态（spinner 或骨架屏）

### Requirement: Search results list
`SearchPanel` SHALL 展示分页搜索结果，每项包含会话上下文与消息片段。

#### Scenario: Render result item
- **WHEN** `search.query` 返回非空 `results`
- **THEN** 每项显示：会话标题、`work_dir` 项目名（若有）、角色标签、时间戳、snippet（含 `<b>` 高亮）
- **AND** 列表最多展示当前页条数（默认 10）

#### Scenario: Highlight keywords in snippet
- **WHEN** snippet 包含 `<b>`…`</b>` 标记
- **THEN** 前端安全渲染为高亮样式（dangerouslySetInnerHTML 或解析后 React 节点），不执行脚本

#### Scenario: Load more results
- **WHEN** 响应 `has_more` 为 true
- **THEN** 显示「加载更多」或自动加载下一页（`page + 1`）

### Requirement: Navigate to search result
用户点击结果项 SHALL 跳转到对应对话位置。

#### Scenario: Click result navigates to message
- **WHEN** 用户点击某条搜索结果
- **THEN** 关闭或收起 `SearchPanel`
- **AND** 切换到 `session_id` 对应会话
- **AND** 消息流滚动到 `turn_id` 对应消息并短暂高亮

#### Scenario: Navigation failure
- **WHEN** 目标 session 不存在或 turn 无法在流中定位（3s 超时）
- **THEN** 显示非阻塞 toast「无法定位到该消息」

### Requirement: Search filters
`SearchPanel` SHALL 支持日期范围与项目/工作区筛选。

#### Scenario: Filter by date range
- **WHEN** 用户选择开始日期与结束日期
- **THEN** 后续 `search.query` 请求携带 `filters.date_from` 与 `filters.date_to`
- **AND** 仅返回该时间范围内的匹配

#### Scenario: Filter by project workspace
- **WHEN** 用户选择某一 `work_dir` / 项目名称
- **THEN** 请求携带 `filters.work_dir`
- **AND** 仅返回该工作区下会话的匹配

#### Scenario: Clear filters
- **WHEN** 用户清除筛选
- **THEN** 以仅关键词条件重新搜索

### Requirement: Empty and no-results states
`SearchPanel` SHALL 对空输入、无匹配、索引中状态提供明确 UI。

#### Scenario: Empty query state
- **WHEN** 输入框为空且面板刚打开
- **THEN** 显示引导文案（如「搜索所有对话中的消息」），不显示结果列表

#### Scenario: No results state
- **WHEN** 查询完成且 `results` 为空数组
- **THEN** 显示「未找到匹配」及当前筛选摘要

#### Scenario: Indexing in progress state
- **WHEN** `search.index_status` 报告 `is_indexing: true`
- **THEN** 面板顶部显示索引进度条或文案「正在索引历史消息 (N/M)」
- **AND** 仍可对已索引部分发起搜索
