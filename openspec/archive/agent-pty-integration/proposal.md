## Why

Agent 当前只能通过 `shell_exec` 执行一次性命令，无法与 REPL、调试器、dev server 等交互式进程持续交互。现有的 `exec_command`/`write_stdin` 工具使用管道 stdin/stdout 模拟交互，不是真正的 PTY，缺少终端模拟、信号处理、resize 等能力，且与前端交互式终端完全隔离。需要将 Agent 接入已有的 `xiaolin_pty` 系统，让 Agent 操作的终端对用户实时可见、可介入。

## What Changes

- 新增 `terminal_open` / `terminal_input` / `terminal_close` 三个 Agent 工具，基于 `xiaolin_pty::PtySessionManager`
- 为 `xiaolin_pty::PtySession` 添加 output broadcast channel，支持多消费者（前端 WebSocket + Agent 工具同时读取）
- 将 `pty_manager` 注入到 Agent 工具注册链路（`state/builder.rs`）
- Agent 创建的 PTY 会话通过 chat WebSocket 通知前端，自动出现在 Shell tab（带 "Agent" 标识）
- 标记 `exec_command`/`write_stdin` 为 deprecated

## Capabilities

### New Capabilities
- `agent-terminal-tools`: Agent 侧的 terminal_open/terminal_input/terminal_close 工具实现、注册、以及与 PTY manager 的集成
- `pty-output-broadcast`: PTY session 的 output broadcast channel，支持多消费者并发读取同一 session 的输出流

### Modified Capabilities
- `interactive-terminal-ui`: Shell tab 需要支持接收 agent 创建的会话通知，显示 "Agent" 标识，并允许用户介入
- `pty-backend`: PtySession 需要新增 broadcast channel 支持多 reader

## Impact

- `crates/xiaolin-pty/src/session.rs` — 新增 broadcast channel
- `crates/xiaolin-pty/src/manager.rs` — 创建 session 时初始化 broadcast
- `crates/xiaolin-agent/src/builtin_tools/` — 新增 terminal.rs 模块
- `crates/xiaolin-gateway/src/state/builder.rs` — 注入 pty_manager 到 agent 工具
- `crates/xiaolin-gateway/src/routes/pty.rs` — WebSocket handler 改为从 broadcast 读取
- `crates/xiaolin-app/src/lib/stores/pty-store.ts` — 新增 source 字段
- `crates/xiaolin-app/src/components/shell/TerminalTabContent.tsx` — Agent 会话标识
- `crates/xiaolin-tools-fs/src/exec_command.rs` — 标记 deprecated
