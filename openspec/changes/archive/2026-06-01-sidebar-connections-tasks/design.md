## Context

侧边栏除「对话」外的 5 个 Tab 全部显示 Coming Soon。后端已完整实现：
- **MCP 管理**：WS 消息 `mcp.status`/`mcp.reload`/`mcp.add`/`mcp.remove`，transport 层已封装
- **Cron 管理**：WS 消息 `cron.list_jobs`/`cron.get_job`/`cron.upsert_job`/`cron.delete_job`/`cron.list_runs`，transport 层已封装
- **Channel 列表**：HTTP `GET /api/v1/channels`，WS 事件 `channels.changed` 已有监听

前端只需要新增 UI 组件，不需要后端改动。

## Goals / Non-Goals

**Goals:**
- 实现「连接」页面：展示 MCP 服务器状态列表 + Channel 插件列表，支持添加/删除 MCP 服务器、重载连接
- 实现「任务」页面：展示 Cron Job 列表，支持 CRUD，展示运行历史
- 移除「专家」Tab，精简侧边栏为 5 个导航项
- 页面设计风格与现有 UI token 系统和深色主题保持一致

**Non-Goals:**
- Channel 的动态添加/配置（仍通过 config 文件管理）
- 像素风工作室页面（后续单独做）
- 文件汇总页面（复杂度高，后续迭代）
- MCP 服务器的高级配置（环境变量、timeout 等）

## Decisions

### D1: 页面组件架构

采用 lazy-loaded 页面组件，在 `AppLayout.tsx` 中按 `activeNav` 切换渲染：

```
AppLayout
├─ activeNav === "chat" → 现有聊天面板
├─ activeNav === "connections" → <ConnectionsPage />  (lazy)
├─ activeNav === "tasks" → <TasksPage />  (lazy)
├─ activeNav === "workspace" → <ComingSoon />  (保留)
└─ activeNav === "files" → <ComingSoon />  (保留)
```

**理由**：lazy loading 避免首屏加载不必要的组件代码，与现有 `SettingsPanel` 的加载模式一致。

### D2: 数据获取策略

使用现有 transport 层的 WS 通信，不新增 HTTP API：
- `getMcpStatus()` → MCP 服务器列表
- `cronListJobs()` → Cron 任务列表
- `cronListRuns(jobId)` → 某个任务的运行历史
- Channel 列表通过新增 `getChannels()` WS 消息获取（或复用 HTTP `GET /api/v1/channels`）

页面进入时获取数据，通过 WS 事件订阅实时更新。不引入额外的状态管理 store，页面内部用 `useState`/`useEffect` 管理。

**理由**：这两个页面数据量小、更新频率低，不需要全局 store。保持简单。

### D3: 侧边栏精简

从 `NavItem` 类型中移除 `experts`，保留 `chat | workspace | tasks | files | connections`（5 项）。`NavRail.tsx` 的 `TOP_ITEMS` 数组对应调整。

**理由**：专家管理功能未来可能融入其他页面（如设置或工作室），目前没有独立页面的必要。

### D4: 连接页面布局

分两个区块：
1. **MCP 服务器** — 卡片列表，每张卡片展示：id、状态指示灯、工具数、连接时间、错误信息。操作按钮：重连/删除。顶部有「添加」按钮和「全部重载」按钮。
2. **消息通道** — 卡片列表，每张卡片展示：名称、描述、别名。纯展示（Channel 通过配置文件管理）。

### D5: 任务页面布局

主区域为 Cron Job 列表卡片，每张卡片展示：名称、cron 表达式（人类可读）、状态、上次运行时间、下次运行时间、运行/错误计数。操作按钮：编辑/删除/启用禁用。

点击卡片展开运行历史面板。顶部有「新建任务」按钮，弹出表单 modal。

### D6: 添加 MCP 服务器表单

简化表单：id（标识符）、command（启动命令）、args（参数列表，逗号分隔）。提交后调用 `addMcpServer()`。

### D7: 创建 Cron Job 表单

Modal 表单字段：
- name（任务名称）
- schedule（cron 表达式，提供常用预设：每小时/每天/每周）
- action 类型选择：Prompt（指定 agent + prompt 文本）或 Webhook（URL + method）
- enabled（默认开启）

## Risks / Trade-offs

- **[数据实时性]** Channel 列表通过 HTTP 获取，不像 MCP 状态有 WS 推送 → 可接受，Channel 变更频率极低，页面进入时 fetch 即可
- **[Cron 表达式输入]** 用户可能不熟悉 cron 语法 → 提供常用预设下拉 + cron 表达式的人类可读翻译
- **[MCP 添加的简化]** 只支持 command + args，不支持环境变量和高级配置 → 满足 80% 场景，高级配置留在 config 文件
