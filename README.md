<p align="center">
  <h1 align="center">小林 (XiaoLin)</h1>
  <p align="center">
    全能桌面端 AI 个人助手<br/>
    <em>Tauri Desktop · Multi-agent · 35+ Tools · Memory · MCP · Streaming</em>
  </p>
</p>

<p align="center">
  <a href="#快速开始">快速开始</a> ·
  <a href="#功能特性">功能特性</a> ·
  <a href="#架构概览">架构概览</a> ·
  <a href="#模块技术详解">模块详解</a> ·
  <a href="docs/MANUAL.md">使用手册</a> ·
  <a href="#许可证">许可证</a>
</p>

---

## 什么是小林

小林 (XiaoLin) 是一个用 **Rust** 构建的桌面端 AI 个人助手，基于 **Tauri v2 + React 19** 提供原生桌面体验。内嵌 HTTP/WebSocket 网关、35+ 内置工具、多模型路由、会话持久化、语义记忆和 MCP 集成，开箱即用。

> **当前推荐运行方式**：Tauri 桌面应用 (`cargo tauri dev`)。Docker/K8s 部署方式正在开发中。

### 核心亮点

- **原生桌面体验** — Tauri v2 桌面应用，内嵌 Gateway，零配置启动
- **多 Agent 编排** — Agent 间消息总线、Sub-agent 任务分发
- **丰富的工具生态** — 文件系统、Shell、PTY 终端、代码智能（LSP）、Web 搜索、浏览器自动化等 35+ 内置工具
- **多模型路由** — 支持 OpenAI / Anthropic / DashScope / DeepSeek / Ollama 等，按策略自动路由
- **语义记忆** — 向量检索 + 知识图谱双引擎，支持本地嵌入（无需外部 API）
- **MCP 协议** — MCP Client 连接外部工具生态（Claude Desktop、Cursor 等）
- **安全防护** — Landlock/Seccomp 沙箱、执行策略、Guardian LLM 审查、网络代理
- **自进化** — 自动从对话轨迹中提取可复用技能，持续优化 Agent 能力
- **多渠道接入** — 飞书机器人（WebSocket/Webhook）、微信、HTTP API、WebSocket

---

## 功能特性

| 模块 | 说明 |
|------|------|
| **Desktop App** | Tauri v2 + React 19 原生桌面应用，内嵌 Gateway，流式对话，系统托盘 |
| **Gateway** | Axum HTTP/WebSocket 网关，CORS、速率限制、API Key 认证、gzip 压缩 |
| **Agent Runtime** | 多轮对话执行引擎，流式输出、工具调用、自动重试、成本追踪 |
| **Model Router** | 按策略（Fixed / Fallback / CostOptimized）选择模型，支持 fallback 链 |
| **Session** | SQLite 持久化会话，自动压缩、TTL 过期清理 |
| **Memory** | Episodic + Semantic 双引擎，向量检索（usearch）+ 知识图谱，后台 dreaming |
| **Tool System** | 35+ 内置工具，per-tool 并行调度，Pre/Post Hook，ToolContributor 扩展 |
| **MCP** | 连接外部 MCP Server 的 Client，自动发现并注册工具 |
| **Channels** | 飞书机器人（WebSocket/Webhook）、微信渠道 |
| **Evolution** | Agent 自进化：轨迹记录 → 技能提取 → 自动激活 |
| **Context Engine** | 分层压缩、Token 预算追踪、自动摘要 |
| **Security** | Landlock/Seccomp 沙箱、执行策略、Guardian LLM、Prompt 注入检测、SSRF 防护 |
| **Cron** | 定时任务调度，支持 Agent 聊天触发和 Webhook 触发 |
| **Observability** | Prometheus 指标、结构化日志（JSON/pretty） |

---

## 快速开始

### 系统要求

- Rust 1.82+（MSRV）
- Node.js 18+、pnpm
- SQLite 3
- Tauri v2 系统依赖（参见 [Tauri Prerequisites](https://v2.tauri.app/start/prerequisites/)）
- （可选）GCC 12+ / C++17 编译器（usearch 向量检索后端）

### 从源码运行

```bash
git clone git@github.com:linzetai/XiaoLin.git
cd XiaoLin

# 安装前端依赖
cd crates/xiaolin-app && pnpm install && cd ../..

# 开发模式运行（前端 HMR + Rust 热编译）
cd crates/xiaolin-app && pnpm tauri dev
```

### 首次配置

首次启动时桌面应用会显示 Onboarding 引导，帮助你配置 LLM API Key 和基本设置。

也可以手动配置：

```bash
mkdir -p ~/.xiaolin/config
cp config/default.json ~/.xiaolin/config/default.json
# 编辑 ~/.xiaolin/config/default.json 填写 LLM API Key
```

### 生产构建

```bash
cd crates/xiaolin-app
pnpm tauri build
```

构建产物：
- Linux: `target/release/bundle/deb/*.deb` / `target/release/bundle/appimage/*.AppImage`
- Windows: `target/release/bundle/nsis/*.exe`
- macOS: `target/release/bundle/macos/*.app`

### Docker 部署（WIP）

> Docker 和 Kubernetes 部署方式正在开发中。当前 Dockerfile 和 K8s manifest 已标记为 WIP。

---

## 架构概览

```
┌────────────────────────────────────────────────────────────┐
│               XiaoLin Desktop (Tauri v2)                   │
│                                                            │
│  ┌──────────────────────────────────────────────────────┐  │
│  │     React 19 + TailwindCSS Frontend                 │  │
│  │  ┌──────────┐ ┌────────┐ ┌───────┐ ┌────────────┐  │  │
│  │  │ Chat UI  │ │Agents  │ │Config │ │ MCP Panel  │  │  │
│  │  │(Stream)  │ │Manager │ │Panel  │ │            │  │  │
│  │  └────┬─────┘ └────┬───┘ └───┬───┘ └─────┬──────┘  │  │
│  │       └──────────────┴────────┴────────────┘         │  │
│  │                   Tauri IPC / WebSocket               │  │
│  └──────────────────────┬────────────────────────────────┘  │
│                         ▼                                   │
│  ┌──────────────────────────────────────────────────────┐  │
│  │              Embedded XiaoLin Gateway                │  │
│  │  Session → Agent Router → Model Router → LLM         │  │
│  │     ↕           ↕             ↕                      │  │
│  │  Memory    Tool Executor   Cost Tracker              │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                            │
│  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────────────────┐  │
│  │Session │ │Memory  │ │  MCP   │ │    Security Stack   │  │
│  │(SQLite)│ │(Vector │ │Client  │ │ Sandbox+ExecPolicy  │  │
│  │        │ │+Graph) │ │        │ │ +Guardian+NetProxy  │  │
│  └────────┘ └────────┘ └────────┘ └────────────────────┘  │
│                                                            │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  Extensions: Feishu Bot · WeChat · Cron · Agent Bus  │  │
│  └──────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────┘
```

---

## 模块技术详解

### xiaolin-core

核心基础库，定义所有 crate 共享的类型和 trait。

| 子模块 | 职责 |
|--------|------|
| `config` | 配置数据模型（`XiaoLinConfig`），支持 JSON5、`$include` 引用、多 profile |
| `agent_config` | Agent 定义（模型、System Prompt、工具权限、MCP 绑定） |
| `tool` | 工具 trait（`Tool`）、注册表（`ToolRegistry`）、ToolContributor 扩展、DynamicTool |
| `skill` | 技能 trait 和注册表，支持 SKILL.md 文件加载 |
| `bus` | Agent 消息总线：跨 Agent 异步消息传递和请求-响应模式 |
| `channel` | 渠道抽象（`ChannelPlugin` trait），统一飞书/微信/Webhook 等入站消息 |
| `routing` | Agent 路由器：按渠道、关键词、默认规则将消息分发到正确的 Agent |
| `complexity` | 任务复杂度评估（`ComplexityTier`: tiny/small/medium/large/frontier） |
| `workspace` | Agent 工作目录管理，沙箱路径隔离 |

### xiaolin-agent

Agent 运行时引擎，负责完整的对话循环。

| 子模块 | 职责 |
|--------|------|
| `runtime` | 核心执行循环：消息构建 → LLM 调用 → 流式输出 → 工具调用 → 递归 |
| `llm` | LLM Provider 抽象层：OpenAI / Anthropic / DashScope 等多种协议 |
| `builtin_tools` | 35+ 内置工具实现（文件系统、Shell、PTY、LSP、搜索、浏览器等） |
| `prompt_engine` | Prompt 组装引擎：静态段 + 动态段，按 token 预算裁剪 |
| `stream_engine` | SSE 流式输出引擎：增量 token、工具调用、状态事件 |
| `tool_executor` | 工具并行调度器：per-tool 并行声明、Pre/Post Hook、批量执行 |

### xiaolin-gateway

HTTP/WebSocket 网关层。

| 子模块 | 职责 |
|--------|------|
| `routes` | Axum 路由定义：REST API + WebSocket + Webhook |
| `state` | 应用状态管理：五阶段初始化，热重载 Agent 配置 |
| `ws` | WebSocket 实时聊天：`xiaolin-ws/2` JSON-RPC 协议，双向流式通信 |
| `chat_pipeline` | 完整聊天流水线：认证 → 路由 → 模型选择 → Agent 执行 → 流式响应 |

### xiaolin-session / xiaolin-session-actor

SQLite 持久化会话管理和 Actor 模型。

| 能力 | 说明 |
|------|------|
| 会话 CRUD | 创建/查询/删除会话及消息 |
| TTL 清理 | 定时清理过期会话（默认 7 天） |
| DM 作用域 | 支持 main / per-peer / per-channel-peer / per-account-channel-peer 隔离 |
| 对话 Trace | 记录每轮对话的完整执行轨迹 |

### xiaolin-memory

双引擎记忆系统：向量检索 + 知识图谱。

| 子模块 | 职责 |
|--------|------|
| `episodic` | 情景记忆：存储对话中的关键事件 |
| `semantic` | 语义记忆：事实 + 关系图谱 |
| `embedding` | 嵌入层：本地（hypembed 纯 Rust）或远程（OpenAI API） |
| `importance` | 重要性评分：基于规则的记忆重要性自动打分 |
| `dreaming` | 梦境周期：后台定期回顾记忆，提取事实、补充嵌入 |
| `working` | 工作记忆：当前会话的短期记忆缓冲 |

### xiaolin-security / xiaolin-sandbox / xiaolin-execpolicy / xiaolin-guardian

多层安全防护栈。

| 组件 | 职责 |
|------|------|
| `xiaolin-security` | API Key 认证、速率限制、Prompt 注入检测、SSRF 防护 |
| `xiaolin-sandbox` | Landlock/Seccomp（Linux）沙箱抽象层 |
| `xiaolin-linux-sandbox` | Linux 沙箱辅助进程二进制 |
| `xiaolin-execpolicy` | Shell 命令执行策略引擎（白/黑名单、危险模式检测） |
| `xiaolin-guardian` | Guardian LLM 审查：高风险操作的二次确认 |
| `xiaolin-network-proxy` | 网络代理：沙箱内的出站流量控制 |
| `xiaolin-hardening` | 进程早期安全加固 |

### 其他模块

| 模块 | 职责 |
|------|------|
| `xiaolin-protocol` | WebSocket 协议类型定义（`xiaolin-ws/2`） |
| `xiaolin-model-router` | 多模型路由与预算追踪 |
| `xiaolin-context` | 上下文引擎（分层压缩、预算、折叠） |
| `xiaolin-evolution` | Agent 自进化（轨迹记录 → 技能提取 → 自动激活） |
| `xiaolin-self-iter` | 自迭代优化（诊断 → 沙箱 → 修复） |
| `xiaolin-mcp` | MCP Client 连接外部 MCP Server |
| `xiaolin-cron` | 定时任务调度器 |
| `xiaolin-treesitter` | Tree-sitter 代码分析 |
| `xiaolin-observe` | 可观测性（Prometheus 指标、日志） |
| `xiaolin-path` | 路径解析/绝对化工具 |

---

## 项目结构

```
XiaoLin/
├── crates/
│   ├── xiaolin-core/            # 核心类型、配置、路由、工具 trait
│   ├── xiaolin-agent/           # Agent 运行时、35+ 内置工具
│   ├── xiaolin-gateway/         # HTTP/WebSocket 网关
│   ├── xiaolin-protocol/        # WebSocket 协议类型（xiaolin-ws/2）
│   ├── xiaolin-session/         # 会话持久化（SQLite）
│   ├── xiaolin-session-actor/   # 会话 Actor 模型
│   ├── xiaolin-memory/          # 向量记忆 + 知识图谱
│   ├── xiaolin-model-router/    # 多模型路由与预算追踪
│   ├── xiaolin-context/         # 上下文引擎（压缩、预算、折叠）
│   ├── xiaolin-mcp/             # MCP Client
│   ├── xiaolin-security/        # 认证、速率限制、Prompt 注入检测
│   ├── xiaolin-sandbox/         # 沙箱抽象层
│   ├── xiaolin-linux-sandbox/   # Linux 沙箱辅助进程
│   ├── xiaolin-execpolicy/      # Shell 命令执行策略
│   ├── xiaolin-guardian/        # Guardian LLM 审查
│   ├── xiaolin-network-proxy/   # 沙箱网络代理
│   ├── xiaolin-hardening/       # 进程安全加固
│   ├── xiaolin-path/            # 路径工具
│   ├── xiaolin-observe/         # 可观测性（Prometheus、日志）
│   ├── xiaolin-evolution/       # Agent 自进化
│   ├── xiaolin-self-iter/       # 自迭代优化
│   ├── xiaolin-cron/            # 定时任务调度器
│   ├── xiaolin-treesitter/      # Tree-sitter 代码分析
│   └── xiaolin-app/             # Tauri v2 桌面应用
├── extensions/
│   ├── feishu/                  # 飞书机器人扩展
│   └── wechat/                  # 微信渠道扩展
├── config/                      # 默认配置和 Agent 定义
│   ├── default.json
│   └── agents/main.json
├── prompts/                     # System prompt 模板库
├── deploy/kubernetes/           # K8s 部署清单（WIP）
├── Dockerfile                   # Docker 构建（WIP）
├── docker-compose.yml           # Docker Compose（WIP）
└── Cargo.toml                   # Workspace 根 manifest
```

---

## CLI 命令参考（计划中）

> **注意**：独立的 `xiaolin` CLI 二进制尚未实现。当前工作区仅产出 Tauri 桌面应用（`xiaolin-app`）和 Linux 沙箱辅助进程（`xiaolin-linux-sandbox`）。
>
> 以下是计划中的 CLI 功能，将在后续版本中实现。

<details>
<summary>展开查看计划中的 CLI 命令</summary>

```
xiaolin <COMMAND>

Commands:
  setup        交互式首次配置
  serve        启动网关（前台）
  health       健康检查
  doctor       环境诊断
  tui          终端交互界面
  config       配置管理
  gateway      网关管理
  sessions     会话管理
  agents       Agent 管理
  tools        工具管理
  mcp-server   启动 MCP Server（stdio）
  completions  生成 Shell 补全脚本
```
</details>

---

## 支持的 LLM 提供商

| 提供商 | 协议 | 默认模型 | 备注 |
|--------|------|----------|------|
| OpenAI | OpenAI API | gpt-4o | Vision / Tool Calling |
| Anthropic | Anthropic API | claude-sonnet-4-20250514 | Reasoning / Vision |
| DashScope (Qwen) | OpenAI 兼容 | qwen3.5-plus | 阿里云通义千问 |
| DeepSeek | OpenAI 兼容 | deepseek-chat | |
| Google Gemini | OpenAI 兼容 | gemini-2.5-flash | |
| Ollama | OpenAI 兼容 | llama3.1:8b | 本地推理，无需 API Key |
| Custom | OpenAI 兼容 | 自定义 | 任意 OpenAI 兼容端点 |

---

## 内置工具一览

<details>
<summary>展开查看 35+ 内置工具</summary>

**文件系统**
- `read_file` — 读取文件内容
- `write_file` — 写入文件
- `edit_file` — 字符串替换编辑
- `multi_edit` — 批量多文件编辑
- `apply_patch` — 应用 unified diff 补丁
- `glob` — 文件名模式搜索
- `search_in_files` — 正则表达式搜索
- `list_directory` — 列出目录内容

**Shell 与终端**
- `shell_exec` — 沙箱化 Shell 命令执行
- `exec_command` — PTY 交互式终端会话
- `write_stdin` — 向 PTY 会话发送输入
- `terminal_capture` — 终端输出捕获

**代码智能**
- `lsp` — 统一 LSP 工具：Go to Definition / Find References / Workspace Symbols
- `file_outline` — 文件结构大纲
- `code_sections` — 语义代码分段
- `browser` — Chrome/CDP 浏览器自动化

**网络**
- `web_search` — 网页搜索（Tavily / SearXNG / Google / Baidu / Bing）
- `web_fetch` — 获取网页内容
- `http_fetch` — HTTP 请求

**记忆**
- `memory` — 统一记忆工具（搜索 + 存储）

**目标与任务管理**
- `get_goal` / `create_goal` / `update_goal` — 目标管理
- `todo_write` / `todo_read` — 待办事项
- `task_create` / `task_list` / `task_get` / `task_update` / `task_stop` — 子任务管理

**交互与权限**
- `ask_question` — 向用户提问
- `confirm` — 确认操作
- `send_user_message` — 发送中间消息
- `request_permissions` — 请求额外权限

**计划模式**
- `enter_plan_mode` / `exit_plan_mode` — 执行模式切换

**会话管理**
- `sessions_spawn` / `sessions_send` — 会话生成与消息发送

**实用工具**
- `get_current_time` — 获取当前时间
- `sleep` — 等待指定时间
- `tool_search` — BM25 模糊搜索可用工具
- `screenshot` — 屏幕截图
- `notebook_edit` — Jupyter Notebook 编辑
- `skill` — 技能管理（列出/读取/写入）
- `identity` — Agent 身份配置管理
- `snip` — 上下文片段压缩
- `git` — Git 版本控制操作

**媒体生成**
- `image_generate` — 图像生成（需 API 配置）
- `text_to_speech` — 文本转语音（需 API 配置）

</details>

---

## API 端点

完整 OpenAPI 规范可通过 `GET /api/v1/openapi.json` 获取。主要端点：

| 方法 | 路径 | 说明 |
|------|------|------|
| `GET` | `/ws` | WebSocket 实时聊天（`xiaolin-ws/2` JSON-RPC） |
| `POST` | `/api/v1/chat` | 聊天补全（支持流式） |
| `POST` | `/api/v1/chat/completions` | OpenAI 兼容聊天接口 |
| `POST` | `/api/v1/chat/resolve-approval` | 解决审批请求 |
| `GET/POST` | `/api/v1/agents` | Agent CRUD |
| `GET/PUT/DELETE` | `/api/v1/agents/:agent_id` | 单个 Agent 管理 |
| `GET/PUT` | `/api/v1/agents/:agent_id/tools` | Agent 工具配置 |
| `GET` | `/api/v1/tools` | 列出所有工具 |
| `GET` | `/api/v1/skills` | 列出所有技能 |
| `GET` | `/api/v1/sessions` | 列出会话 |
| `GET/DELETE` | `/api/v1/sessions/:session_id` | 会话管理 |
| `GET` | `/api/v1/sessions/:session_id/messages` | 获取会话消息 |
| `GET` | `/api/v1/memory/episodes` | 情景记忆列表 |
| `GET` | `/api/v1/memory/episodes/search` | 情景记忆搜索 |
| `GET/POST` | `/api/v1/memory/facts` | 语义事实管理 |
| `GET` | `/api/v1/memory/facts/search` | 事实搜索 |
| `DELETE` | `/api/v1/memory/facts/:fact_id` | 删除事实 |
| `GET/POST` | `/api/v1/cron/jobs` | 定时任务管理 |
| `GET/DELETE` | `/api/v1/cron/jobs/:job_id` | 单个任务管理 |
| `GET` | `/api/v1/channels` | 列出渠道 |
| `GET/POST` | `/api/v1/routes` | 路由规则管理 |
| `PUT/DELETE` | `/api/v1/routes/:id` | 单条路由管理 |
| `GET` | `/api/v1/subagents/defs` | 子 Agent 定义 |
| `GET` | `/api/v1/subagents/runs` | 子 Agent 运行实例 |
| `GET/DELETE` | `/api/v1/subagents/runs/:run_id` | 子 Agent 实例管理 |
| `GET` | `/api/v1/subagents/concurrency` | 子 Agent 并发信息 |
| `GET/POST` | `/api/v1/llm-plugins` | LLM 插件管理 |
| `GET/PUT/DELETE` | `/api/v1/llm-plugins/:id` | 单个 LLM 插件 |
| `POST` | `/api/v1/llm-plugins/:id/test` | 测试 LLM 插件 |
| `POST` | `/api/v1/evolution/feedback` | 提交 Agent 反馈 |
| `GET` | `/api/v1/evolution/candidates/:agent_id` | 候选技能 |
| `GET` | `/api/v1/bus/agents` | Agent 总线 |
| `POST` | `/api/v1/bus/send` | 发送总线消息 |
| `GET` | `/api/v1/traces` | 对话 Trace |
| `POST` | `/webhook/:channel_id` | 渠道 Webhook |
| `POST` | `/api/v1/channels/wechat/login/start` | 微信登录 |
| `GET` | `/api/v1/channels/wechat/login/status/:session_key` | 微信登录状态 |
| `GET` | `/api/v1/channels/wechat/accounts` | 微信账号列表 |
| `GET` | `/health` | 健康检查 |
| `GET` | `/ready` | 就绪探针 |
| `GET` | `/metrics` | Prometheus 指标 |

---

## 配置说明

配置文件路径：`~/.xiaolin/config/default.json`（支持 JSON5 注释）

主要配置段落参见 [docs/MANUAL.md](docs/MANUAL.md)。

---

## 开发

```bash
# 运行测试
cargo test --workspace

# 运行 Clippy
cargo clippy --workspace --all-targets

# 桌面应用开发模式
cd crates/xiaolin-app && pnpm tauri dev
```

---

## 许可证

[MIT License](LICENSE) &copy; 2026 linzetai
