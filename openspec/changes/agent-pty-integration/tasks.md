## 1. PTY Output Broadcast 基础设施

- [ ] 1.1 在 `xiaolin_pty::PtySession` 中添加 `broadcast::Sender<Vec<u8>>` 字段和 `subscribe()` 方法
- [ ] 1.2 修改 `PtySession::spawn` 启动后台 reader task 将 PTY 输出 publish 到 broadcast channel
- [ ] 1.3 添加 `source: String` 字段到 `PtySession` 和 `PtySessionConfig`
- [ ] 1.4 新增 `create_session_with_subscriber(config) -> Result<(String, broadcast::Receiver), String>` 方法
- [ ] 1.5 修改 `PtySessionManager::list_sessions()` 返回 `source` 字段

## 2. Gateway PTY Route 适配

- [ ] 2.1 修改 `routes/pty.rs` WebSocket handler 从 broadcast subscriber 读取输出（替代 `get_reader()`）
- [ ] 2.2 验证前端终端功能不受 broadcast 改造影响（输入/输出/resize/cwd 跟踪）

## 3. Agent Terminal 工具实现

- [ ] 3.1 创建 `crates/xiaolin-agent/src/builtin_tools/terminal.rs` 模块
- [ ] 3.2 实现 `TerminalOpenTool`：创建 PTY session，subscribe broadcast，返回 session_id + 初始输出
- [ ] 3.3 实现 `TerminalInputTool`：write input，从 broadcast 收集输出（支持 wait_ms / wait_for）
- [ ] 3.4 实现 `TerminalCloseTool`：关闭 session
- [ ] 3.5 在 `builtin_tools/mod.rs` 中添加注册函数 `register_terminal_tools`

## 4. Agent 工具注入

- [ ] 4.1 在 `state/builder.rs` 中将 `strm.pty_manager` 传递给 `register_terminal_tools`
- [ ] 4.2 限制 Agent 最多创建 3 个 PTY session（工具层面校验）
- [ ] 4.3 验证 `terminal_*` 工具出现在 agent tool list 中

## 5. 前端联动

- [ ] 5.1 在 `PtySession` store 中添加 `source: "user" | "agent"` 字段
- [ ] 5.2 通过 chat WebSocket 发送 `pty_session_opened` 事件（包含 session_id、name、source）
- [ ] 5.3 前端收到事件后自动在 Shell tab 添加 agent session 并建立 PTY WebSocket 连接
- [ ] 5.4 Agent session tab 显示 "Agent" 标识（robot icon 或 A 标签）
- [ ] 5.5 用户可切换到 agent session 并输入（共享 PTY）

## 6. 废弃旧工具

- [ ] 6.1 在 `exec_command`/`write_stdin` 的 tool description 中添加 deprecated 说明
- [ ] 6.2 Agent prompt 中引导使用 `terminal_*` 替代 `exec_command`

## 7. 验证

- [ ] 7.1 E2E 测试：Agent 调用 terminal_open，前端 Shell tab 出现新会话
- [ ] 7.2 E2E 测试：Agent 调用 terminal_input 发送命令，前端实时显示输出
- [ ] 7.3 E2E 测试：用户在 agent session 中输入，Agent 下次 terminal_input 能看到用户输入的结果
- [ ] 7.4 E2E 测试：Agent 调用 terminal_close，前端会话标记为 exited
