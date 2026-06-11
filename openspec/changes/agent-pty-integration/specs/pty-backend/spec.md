## MODIFIED Requirements

### Requirement: PTY I/O streaming
系统 SHALL 通过 broadcast channel 分发 PTY 输出到多个消费者（WebSocket handler、Agent 工具）。WebSocket handler SHALL 从 broadcast channel subscribe 而非直接持有 reader。延迟不超过 16ms（单帧往返）。

#### Scenario: User input forwarded to PTY
- **WHEN** 客户端通过 WebSocket 发送 Binary 帧包含 `ls\n` 字节
- **THEN** 系统将这些字节写入 PTY master fd，shell 执行 `ls` 命令

#### Scenario: PTY output streamed to client via broadcast
- **WHEN** PTY 子进程产生 stdout 输出
- **THEN** reader task 发布到 broadcast channel，WebSocket handler 从 channel 接收并转发给客户端

#### Scenario: Multiple consumers receive same output
- **WHEN** WebSocket handler 和 Agent tool 同时订阅同一 session
- **THEN** 两者独立接收相同的输出数据

#### Scenario: ANSI escape sequences preserved
- **WHEN** PTY 输出包含 ANSI 转义序列（颜色、光标移动等）
- **THEN** 系统原样透传，不做任何过滤或转换

## ADDED Requirements

### Requirement: Session creation supports source tracking
`PtySessionManager::create_session` SHALL accept a `source` parameter indicating whether the session was created by `"user"` (frontend) or `"agent"` (agent tool). This metadata SHALL be available via `list_sessions()`.

#### Scenario: Agent creates session with source tag
- **WHEN** Agent tool calls `create_session` with source `"agent"`
- **THEN** `list_sessions()` includes `source: "agent"` for that session

### Requirement: Programmatic session creation for agent
`PtySessionManager` SHALL provide a method to create a session and immediately return a broadcast subscriber for output reading, without requiring a WebSocket connection.

#### Scenario: Agent creates session and reads output
- **WHEN** Agent tool calls `create_session_with_subscriber(config)`
- **THEN** returns `(session_id, broadcast::Receiver<Vec<u8>>)` for the agent to read output directly
