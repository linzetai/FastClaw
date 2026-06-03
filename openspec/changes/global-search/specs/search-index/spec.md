## ADDED Requirements

### Requirement: FTS5 virtual table creation
系统 SHALL 在应用 SQLite 数据库中创建 FTS5 虚拟表 `messages_fts`，用于消息正文全文检索。

#### Scenario: First-run migration creates messages_fts
- **WHEN** `SearchIndex::ensure_schema()` 在启动时执行
- **AND** `messages_fts` 表不存在
- **THEN** 创建 `messages_fts` 虚拟表，列包含可索引的 `content` 与 UNINDEXED 元数据列 `session_id`, `turn_id`, `role`, `message_id`（可选）
- **AND** 使用 `tokenize = 'unicode61'`

#### Scenario: Schema already exists
- **WHEN** `messages_fts` 已存在
- **THEN** 迁移为 no-op，不删除已有索引数据

### Requirement: Index metadata tracking
系统 SHALL 维护 `search_index_meta` 表（或等价键值存储），记录回填游标与索引状态。

#### Scenario: Track bulk indexing cursor
- **WHEN** 后台 bulk index 处理一批 `event_log` 行
- **THEN** 更新 `last_event_log_id`（及/或 `last_message_id`）游标
- **AND** `search.index_status` 可据此计算 `indexed_count` 与 `total_count`

### Requirement: Incremental indexing on new messages
系统 SHALL 在消息持久化热路径上增量写入 FTS 索引，无需等待批量任务。

#### Scenario: Index on EventLog append
- **WHEN** `EventLog` batch writer 成功 INSERT 一条可搜索事件（含用户或助手可见文本内容）
- **THEN** 解析 `event_json` 提取纯文本 `content`
- **AND** 将 `(session_id, turn_id, role, content)` 写入或更新 `messages_fts`

#### Scenario: Index on SessionStore append_messages
- **WHEN** `SessionStore::append_messages` 写入 `role` 为 `user` 或 `assistant` 且 `content` 非空的行
- **THEN** 同步索引该行；`turn_id` 使用当前 turn（若可用）或留空并由 `message_id` 关联

#### Scenario: Skip non-searchable events
- **WHEN** 事件类型为工具进度、心跳、无文本 payload
- **THEN** 不写入 `messages_fts`

#### Scenario: Deduplicate by session and turn
- **WHEN** 同一 `(session_id, turn_id, role)` 收到多次 content 更新（流式 delta）
- **THEN** 索引层保留该键的最新 `content`，不产生重复搜索结果行

### Requirement: Startup bulk indexing for existing messages
系统 SHALL 在应用启动后于后台对历史数据进行一次性（可 resumed）回填，不阻塞 gateway ready。

#### Scenario: Background bulk index on startup
- **WHEN** 应用启动且 `search_index_meta` 显示仍有未索引历史
- **THEN** spawn 低优先级异步任务，按游标批量读取 `event_log` 与/或 `messages`
- **AND** 批量 INSERT 到 `messages_fts`
- **AND** gateway 与 UI 可正常使用，不等待回填完成

#### Scenario: Resume interrupted bulk index
- **WHEN** 上次 bulk index 中途退出
- **THEN** 从 `search_index_meta` 保存的游标继续，不重复索引已处理行

#### Scenario: Bulk index completion
- **WHEN** 所有历史行已处理
- **THEN** 设置 `is_indexing = false`
- **AND** `indexed_count` 等于 `total_count`

### Requirement: SNIPPET-based context extraction
搜索查询 SHALL 使用 FTS5 `snippet()` 函数生成带标记的上下文片段。

#### Scenario: Generate highlighted snippet
- **WHEN** 执行 FTS 查询且某行匹配关键词 `q`
- **THEN** 使用 `snippet(messages_fts, …, '<b>', '</b>', '…', 32)` 生成片段
- **AND** 片段在 API 响应中以 `snippet` 字段返回，供前端渲染高亮

### Requirement: Index maintenance
系统 SHALL 提供索引维护能力，应对损坏、膨胀或 schema 变更。

#### Scenario: Rebuild FTS index
- **WHEN** 管理员或内部命令触发 `SearchIndex::rebuild()`
- **THEN** 清空 `messages_fts` 并重置 `search_index_meta` 游标
- **AND** 重新执行 bulk index

#### Scenario: Vacuum after large rebuild
- **WHEN** rebuild 完成
- **THEN** 可选执行 `VACUUM` 或 `INSERT INTO messages_fts(messages_fts) VALUES('optimize')` 压缩 FTS 段

#### Scenario: Cascade delete on session removal
- **WHEN** 某 `session_id` 的会话被删除
- **THEN** 从 `messages_fts` 删除该 `session_id` 的所有索引行
