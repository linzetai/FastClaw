## ADDED Requirements

### Requirement: User can send steer instructions during streaming
The system SHALL allow the user to type and send additional instructions while the agent is actively streaming, injected into the current turn via `chat.steer`.

#### Scenario: User sends steer message while agent is streaming
- **WHEN** the agent is actively streaming a response
- **THEN** the input placeholder changes to "追加指令..."
- **WHEN** the user types a message and presses Enter
- **THEN** the message is sent as `chat.steer` instead of `chat.send`
- **THEN** the agent receives the steer input as additional context in the current turn

#### Scenario: Steer message displayed in chat stream
- **WHEN** a steer message is sent during streaming
- **THEN** the message appears in the chat stream as a user message with a "追加" badge
- **THEN** the badge visually distinguishes it from a new turn's user message

#### Scenario: Steer when no active streaming
- **WHEN** the agent is not streaming (idle state)
- **THEN** the input box behaves normally (sends `chat.send` to start a new turn)
- **THEN** the placeholder shows the default "描述任务，或输入 @ 引用文件、/ 命令..."

### Requirement: transport.ts exposes chatSteer function
The transport layer SHALL export a `chatSteer(sessionId, messages)` function that sends the `chat.steer` WS message.

#### Scenario: chatSteer sends correct WS payload
- **WHEN** `chatSteer("session-123", [{ role: "user", content: "换个方向" }])` is called
- **THEN** the WebSocket client sends `{ method: "chat.steer", params: { sessionId: "session-123", messages: [...] } }`
