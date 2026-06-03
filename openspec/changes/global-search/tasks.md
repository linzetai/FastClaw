## 1. FTS5 schema

- [ ] 1.1 在 `xiaolin-session` 新增 `search_index.rs`：`SearchIndex` 结构体 + `ensure_schema()`（`messages_fts` FTS5 表 + `search_index_meta` 元数据表）
- [ ] 1.2 实现 FTS5 migration：首次启动创建虚拟表，`tokenize = 'unicode61'`，UNINDEXED 列 `session_id`, `turn_id`, `role`, `message_id`
- [ ] 1.3 实现 `SearchIndex::index_row()` / `upsert_row()` 单条写入 API
- [ ] 1.4 实现启动时 `needs_backfill()` 检测与 `search_index_meta` 游标读写

## 2. Indexing pipeline

- [ ] 2.1 在 `EventLog` batch writer 成功 INSERT 后钩子：解析可搜索事件文本并调用 `index_row`
- [ ] 2.2 在 `SessionStore::append_messages` 路径同步索引 user/assistant `content`
- [ ] 2.3 实现 `(session_id, turn_id, role)` 去重更新逻辑（流式 delta 覆盖）
- [ ] 2.4 实现后台 `bulk_index_history()`：按游标批量扫描 `event_log` + `messages`，更新进度到 meta 表
- [ ] 2.5 实现 `SearchIndex::search()`：FTS MATCH + `bm25` 排序 + `snippet()` 片段提取 + JOIN `sessions`
- [ ] 2.6 实现 `SearchIndex::delete_session()` 与会话删除级联清理
- [ ] 2.7 实现 `SearchIndex::rebuild()` 维护入口（开发/修复用）

## 3. WS API

- [ ] 3.1 在 `xiaolin-protocol` 定义 `search.query` / `search.index_status` 请求与响应类型
- [ ] 3.2 新增 `xiaolin-gateway/src/ws/search.rs`：`search.query` handler（q, filters, page, limit，默认 limit=10，2s 超时）
- [ ] 3.3 实现 `search.index_status` handler（indexed_count, total_count, is_indexing）
- [ ] 3.4 在 `ws/mod.rs` 注册路由；gateway 启动时 spawn bulk index 任务并持有 `Arc<SearchIndex>`

## 4. Frontend store

- [ ] 4.1 创建 `useSearchStore.ts`：query, results, loading, filters, page, panelOpen, indexStatus
- [ ] 4.2 实现 300ms debounce `search()` + 陈旧响应丢弃
- [ ] 4.3 实现 `navigateToResult`, `loadMore`, `openPanel`/`closePanel`, index status 轮询

## 5. Search panel

- [ ] 5.1 创建 `SearchPanel.tsx`：输入框、关闭按钮、Esc 处理
- [ ] 5.2 实现结果列表项 UI（标题、项目名、snippet 高亮、时间戳、角色）
- [ ] 5.3 实现筛选 UI：日期范围选择器、工作区/项目下拉（数据来自 sessions work_dir 去重）
- [ ] 5.4 实现 empty / no-results / loading 三态
- [ ] 5.5 实现「加载更多」分页
- [ ] 5.6 集成 `navigateToResult`：切换 session + 扩展 `MessageStream` `scrollToTurn(turnId)` + 高亮动画
- [ ] 5.7 注册 Cmd/Ctrl+K 全局快捷键（`useEffect` + `preventDefault`）

## 6. Sidebar integration

- [ ] 6.1 在 `AppSidebar` / `SessionList` 顶部增加 Search 按钮，点击 `openPanel()`
- [ ] 6.2 侧边栏布局：`SearchPanel` overlay 覆盖会话列表区域，关闭后恢复列表

## 7. Performance

- [ ] 7.1 bulk index 批量事务（如每 500 行 commit 一次），避免单条 INSERT 事务开销
- [ ] 7.2 搜索查询 `LIMIT` + `OFFSET` 分页，禁止无 LIMIT 全表扫描
- [ ] 7.3 面板打开时显示 `search.index_status` 进度；索引完成后停止轮询
- [ ] 7.4 （可选）bulk 进度跨 5% 广播 `search.index_progress` 减少轮询

## 8. Verification

- [ ] 8.1 单元测试：`SearchIndex` snippet 查询与 filter SQL
- [ ] 8.2 gateway 集成测试：`search.query` 命中已知 fixture 消息
- [ ] 8.3 Tauri MCP E2E：Cmd+K 打开面板 → 搜索 → 点击结果跳转并截图验证
