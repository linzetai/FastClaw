## Context

XiaoLin 的 Agent 系统有三层 shell 执行能力：

1. **`shell_exec`**（ShellRuntime）— 一次性命令，`sh -c "cmd"`，有 dangerous-ops 审批 + 沙箱，适合 ls/grep/build 等
2. **`exec_command`/`write_stdin`**（legacy PtySessionManager）— 管道式伪交互，`Command::new().stdin(Piped)`，无终端模拟
3. **`xiaolin_pty`**（新 PTY 系统）— 真正伪终端，已服务前端 Shell tab，支持 resize/信号/cwd 跟踪

Agent 当前无法使用第三层。需要打通。

前端交互式终端（Shell tab）通过 WebSocket 直连 `xiaolin_pty::PtySessionManager`。Agent 工具需要复用同一个 manager 实例，使得 Agent 创建的会话自动出现在用户 Shell tab。

## Goals / Non-Goals

**Goals:**
- Agent 可创建、操控、关闭持久化 PTY 会话
- Agent 操作的终端对用户实时可见（Shell tab 共享）
- 用户可介入 Agent 的终端会话
- `terminal_input` 支持 "wait_for" 模式（等待特定输出或超时）
- LLM 能清晰区分 `shell_exec`（一次性）和 `terminal_*`（持久交互）的使用场景

**Non-Goals:**
- 不在本次为 `terminal_*` 添加 dangerous-ops 审批（后续迭代）
- 不支持 Agent 附着到用户已有的 Shell 会话（仅 Agent 自行创建）
- 不改动 `shell_exec` 的行为
- 不做跨平台 PTY 差异适配（macOS/Windows 留后续）

## Decisions

### D1: Output 分发机制 — Broadcast Channel

**选择**: 在 `PtySession` 中添加 `tokio::sync::broadcast::Sender<Vec<u8>>`

**替代方案**:
- A) 每次读用 `try_clone_reader()` — 只能有一个 reader 活跃，无法共享
- B) 用 `watch` channel — 只保留最新值，不适合流式数据
- C) 用 `mpsc` + 手动 fan-out — 实现复杂

**理由**: broadcast channel 天然支持多消费者、丢失策略可配（lagged），且语义清晰。每个 subscriber（前端 WS handler、Agent tool）独立消费同一 output 流。

### D2: Agent 工具不走 RuntimeRegistry（暂不审批）

**选择**: `terminal_*` 工具直接注册到 `ToolRegistry`，不经过 `RuntimeRegistry` 的 dangerous-ops 审批

**理由**: 
- `terminal_open` 本身只是打开一个 shell（等同于用户在 Shell tab 点 "+"）
- `terminal_input` 发送的具体命令由 LLM 决定，内容审批应在更上层处理
- 后续可通过 Guardian hook 添加审批

### D3: 通知前端新会话 — 通过 Agent event stream

**选择**: Agent 执行 `terminal_open` 后，通过现有的 `AgentEvent::ToolProgress` 或新增 `AgentEvent::PtySessionCreated` 事件通知前端

**替代方案**:
- A) 前端轮询 `/api/v1/pty/sessions` — 延迟高，浪费资源
- B) 独立 WebSocket channel — 过度设计

**理由**: chat WebSocket 已承载 agent 事件流，复用现有通道最简单。前端收到事件后自动建立到该 session 的 PTY WebSocket 连接。

### D4: terminal_input 的输出读取 — subscribe + timeout

**选择**: `terminal_input` 发送输入后，从 broadcast channel subscribe 并等待输出，支持两种退出条件：
1. 超时（`wait_ms`，默认 2000ms）
2. 匹配到指定文本（`wait_for`）

**理由**: Agent 需要知道命令执行完成，但 PTY 输出是流式的。超时兜底保证不阻塞，`wait_for` 允许精确等待（如 "Server ready on port 3000"）。

### D5: 工具命名和 LLM 引导

**选择**: 三个工具 `terminal_open` / `terminal_input` / `terminal_close`，description 明确标注使用场景

**LLM 选择引导**（通过 tool description）：
- `shell_exec`: "Execute a command and get output. Best for quick commands that finish within 2 minutes."
- `terminal_open`: "Open a persistent interactive terminal visible to the user. Use for REPLs, dev servers, debuggers, or multi-step workflows requiring sequential commands in the same session."

## Risks / Trade-offs

- **[Broadcast buffer overflow]** → 设置合理 buffer size（256 条），lagged subscriber 跳过旧数据
- **[Agent 占用所有 PTY slots]** → 限制 Agent 最多创建 3 个 session（manager 总上限 8）
- **[LLM 误选工具]** → 通过 tool description + system prompt 引导；若持续误选，可在 orchestrator 层添加启发式重路由
- **[用户介入导致 Agent 输出解析失败]** → Agent 读取输出时应容忍意外内容，不做严格解析
