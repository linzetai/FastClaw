## Why

用户无法在跨会话的对话历史中检索内容。随着会话数量增长，要找回某次讨论、某段代码方案或某个工具输出只能靠记忆或翻遍会话列表。当前 `SessionList` 仅在侧边栏用 `fuzzyMatch` 过滤**会话标题**（`chat.title`），`MessageStream` 内的搜索仅覆盖**当前会话**已加载的消息流，均无法搜索 SQLite 中已持久化的历史消息。

Codex 原型侧边栏顶部有独立的 **Search** 入口；用户期望一键打开全局搜索，按关键词、时间、项目/工作区筛选，并跳转到具体消息位置。

## What Changes

- **侧边栏 Search 入口** → 打开全局搜索面板（overlay 或临时替换会话列表区域）
- **跨会话全文检索**：对消息正文进行搜索，不限于当前 session
- **SQLite FTS5 索引层**：`messages_fts` 虚拟表，对 `content` 建全文索引
- **搜索结果展示**：上下文片段（SNIPPET）、会话标题、时间戳、关键词高亮
- **跳转到消息**：点击结果 → 切换 session + 滚动定位到对应 `turn_id` / 消息
- **筛选器**：关键词、日期范围、项目/工作区（`sessions.work_dir`）

## Capabilities

### New Capabilities

- `search-index`: SQLite FTS5 索引层——表创建、迁移、增量写入、启动批量回填、SNIPPET 提取、索引维护
- `search-panel`: 全局搜索 UI——输入框、结果列表、筛选、空态/加载态、跳转导航
- `search-websocket-api`: 搜索相关 WS API——`search.query`、`search.index_status`
- `search-store`: 前端 Zustand store——查询状态、防抖搜索、筛选、导航动作

### Modified Capabilities

- `app-sidebar`: 增加 Search 按钮（放大镜图标），点击打开搜索面板；快捷键 Cmd/Ctrl+K

## Impact

- **后端**：
  - `xiaolin-session`：新增 `SearchIndex` 模块（FTS5 表、索引写入、查询、回填任务）
  - `xiaolin-session::EventLog`：写入路径钩子，对含可搜索文本的事件增量索引
  - `xiaolin-gateway/src/ws/`：新增 `search.rs` handler（`search.query`、`search.index_status`）
  - 启动流程：后台 bulk index 历史 `event_log` / `messages`，广播索引进度
- **前端**：
  - 新增 `SearchPanel` 组件、`useSearchStore`
  - 修改 `SessionList` / `AppSidebar`（或 layout-overhaul 后的 `AppSidebar`）集成 Search 入口
  - 扩展消息流滚动 API，支持按 `session_id` + `turn_id` 定位
- **依赖**：无新增 crate；SQLite FTS5 为内置扩展（与现有 `sqlx` + SQLite 栈一致）
- **兼容性**：索引未就绪时搜索 API 返回部分结果或 `is_indexing: true`；旧数据在首次启动后台回填，不阻塞 UI
- **关系**：可与 `layout-overhaul` 的侧边栏结构对齐；`project-model` 的 `work_dir` 分组可用于「按项目筛选」
