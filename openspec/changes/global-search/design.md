## Context

当前消息持久化有两条路径，全局搜索需覆盖用户可见的对话文本：

```
┌─────────────────────────────────────────────────────────────────┐
│                        Gateway / Agent                          │
└────────────────────────────┬────────────────────────────────────┘
                             │
         ┌───────────────────┴───────────────────┐
         ▼                                       ▼
┌─────────────────┐                   ┌─────────────────┐
│  SessionStore   │                   │    EventLog     │
│  messages 表    │                   │  event_log 表   │
│  role, content  │                   │  event_json     │
│  (LLM 上下文)   │                   │  (流式回放)      │
└─────────────────┘                   └─────────────────┘
         │                                       │
         │  当前仅 title fuzzy                   │  无 FTS
         ▼                                       ▼
   SessionList 本地过滤                    无跨 session 检索
```

- **`SessionList`**（`SessionList.tsx`）：顶部输入框 + `fuzzyMatch(query, chat.title)`，仅过滤标题
- **`MessageStream`**：Cmd+F 式当前会话内 DOM 搜索，依赖内存中的 `stream`，不查库
- **`event_log`**：`session_id`, `turn_id`, `event_type`, `event_json`——含 `MessageDelta` / 最终消息等可提取文本
- **`messages`**：`session_id`, `role`, `content`——结构化对话行，有 `id` 但无 `turn_id`

原型（`docs/prototype-codex-layout.html`）侧边栏顶部操作区含 **Search** 按钮，与 New chat / Plugins 并列，暗示全局搜索而非会话列表内嵌过滤。

## Goals / Non-Goals

**Goals:**

- 跨会话全文搜索消息正文，结果按相关性排序并带上下文片段
- FTS5 索引与现有 SQLite 同库，增量更新 + 启动时后台回填历史
- 搜索结果可跳转到对应 session 与 turn（或等价消息锚点）
- 前端 300ms 防抖、每页最多 10 条；支持日期、工作区筛选
- 大批量历史首次索引时显示进度，不阻塞应用启动

**Non-Goals:**

- 不索引工具原始 JSON、二进制附件、图片 OCR（仅可搜索文本字段）
- 不做语义/向量搜索（本变更仅 FTS5 关键词）
- 不替换 `SessionList` 标题过滤（保留为轻量本地 filter；全局搜索为独立面板）
- 不搜索 memory facts、文件系统、git（另有专用 API）

## Decisions

### D1: SQLite FTS5 作为全文引擎

**决策**：使用 SQLite FTS5 虚拟表，不引入 Meilisearch / Tantivy 等外部依赖。

**理由**：项目已用 `sqlx` + SQLite（`SessionStore` pool 与 `EventLog` 共享）；FTS5 内置 SNIPPET、BM25 排序；部署零额外服务。

**替代方案**：应用内 Tantivy 索引文件。否决——增加备份/迁移复杂度，与单文件 SQLite 模型不一致。

### D2: FTS 表结构

**决策**：

```sql
CREATE VIRTUAL TABLE IF NOT EXISTS messages_fts USING fts5(
    content,
    session_id UNINDEXED,
    turn_id UNINDEXED,
    role UNINDEXED,
    message_id UNINDEXED,  -- 可选，messages 表 id，用于无 turn 的回填行
    tokenize = 'unicode61'
);
```

- **INDEXED**：仅 `content`
- **UNINDEXED**：元数据列，用于过滤与跳转，不参与分词

**理由**：FTS5 建议仅对需搜索列分词；`session_id` / `turn_id` 用于 JOIN 与跳转。

### D3: 增量索引与启动回填

**决策**：

1. **增量**：`EventLog` batch writer 在 INSERT `event_log` 成功后，对可索引事件类型（如含 `content` / `text` 的 message 类事件）解析文本并 `INSERT INTO messages_fts`；`SessionStore::append_messages` 对 user/assistant 行同步写入（双写保证 LLM 消息与流事件不遗漏）
2. **回填**：应用启动后 spawn 后台任务，按 `event_log.id` 或 `messages.id` 游标批量索引未入库行；维护 `search_index_meta` 表记录 `last_event_log_id` / `last_message_id`
3. **去重**：同一 `(session_id, turn_id, role)` 多次 delta 时 UPSERT 或 DELETE+INSERT 保留最终 content

**理由**：热路径与 EventLog 批量写入对齐，避免每事件单独事务；历史数据一次性补齐。

**风险**：大库首次回填耗时长 → 见 R1。

### D4: 查询 API 与 SNIPPET

**决策**：`search.query` 执行 FTS5 查询，使用 `snippet(messages_fts, 0, '<b>', '</b>', '…', 32)` 生成高亮片段；`ORDER BY bm25(messages_fts)` 排序；JOIN `sessions` 取 `title`, `work_dir`, `updated_at`。

响应单项示例：

```json
{
  "session_id": "...",
  "turn_id": "...",
  "role": "assistant",
  "session_title": "...",
  "work_dir": "/path/to/project",
  "snippet": "...matching <b>keyword</b>...",
  "timestamp": "2026-06-01T12:00:00Z",
  "rank": -1.23
}
```

**理由**：SNIPPET 为 FTS5 原生能力，减少前端截断逻辑。

### D5: 跳转导航

**决策**：结果携带 `session_id` + `turn_id`（必填）及可选 `message_id`。前端 `navigateToResult`：

1. `setActiveChat(session_id)` 加载会话
2. 等待 stream 就绪后，调用已有 scroll-to-turn 机制（或扩展 `MessageStream` 暴露 `scrollToTurn(turnId)`）
3. 短暂高亮目标消息块

若仅有 `message_id`（历史 messages 回填），映射到最近 `turn_id` 或按 message index 滚动。

**理由**：与 `AgentEvent::turn_id()` 模型一致，便于对齐 EventLog 回放粒度。

### D6: 速率限制与分页

**决策**：

- 前端：`useSearchStore` 内 300ms debounce；空查询不发起请求
- 后端：默认 `limit = 10`，`page` 从 0 起；单次查询超时 2s；同一连接并发 search 请求合并或拒绝（可选 mutex）

**理由**：避免每次键入打穿 SQLite；侧边栏面板空间有限，10 条/页足够。

### D7: 筛选器

**决策**：

| 筛选 | 实现 |
|------|------|
| 关键词 `q` | FTS5 `MATCH`，支持引号短语 |
| 日期范围 | JOIN `sessions.updated_at` 或索引行 `created_at`（需在索引表增加 `indexed_at` UNINDEXED 列） |
| 项目/工作区 | `sessions.work_dir = ?` 或 `work_dir LIKE ?` |

**理由**：`work_dir` 已在 `sessions` 表；与 `project-model` 分组一致。

### D8: 搜索面板呈现

**决策**：点击侧边栏 Search 或 Cmd/Ctrl+K 打开**全屏或侧边栏内 overlay 面板**（覆盖会话列表区域，保留 AppShell 其余部分）。面板含：搜索框、筛选 chips、结果列表、Esc 关闭。

**理由**：对齐 Codex 原型独立 Search 入口；与会话列表内嵌搜索框（当前 `SessionList` 顶部）职责分离——后者可保留为「过滤当前列表标题」或后续移除。

## Risks / Trade-offs

**[R1] 首次 bulk index 慢** → 后台任务 + `search.index_status` 进度；UI 显示「正在索引 N/M」；搜索在部分索引下仍可用（已索引子集）。

**[R2] EventLog JSON 与 messages 双源重复** → 索引层按 `(session_id, turn_id, role)` 去重；优先 EventLog 最终文本，messages 作回填补充。

**[R3] FTS5 中文分词弱** → `unicode61` tokenizer 对 CJK 按字符切分，可接受；后续可加 `porter` 或自定义 tokenizer（非本变更）。

**[R4] jump-to-turn 时 stream 未加载** → 先 `session.load` / 切换 session 再 poll `scrollToTurn`；超时 3s 显示 toast「无法定位消息」。

**[R5] 与 layout-overhaul 侧边栏结构冲突** → `app-sidebar` spec 定义 Search 按钮行为；实现时合并 layout-overhaul 的顶部四按钮布局。

## Open Questions

- 是否在索引中包含 `reasoning_content`？（默认否，仅用户可见 `content`）
- 会话列表顶部现有「搜索会话」输入框是否保留或改为仅打开全局面板？（建议保留标题 filter，全局 Search 按钮打开面板）
