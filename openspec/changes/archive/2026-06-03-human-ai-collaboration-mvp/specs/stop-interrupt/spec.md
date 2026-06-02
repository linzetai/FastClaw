## ADDED Requirements

### Requirement: Frontend Stop button sends backend Interrupt
The system SHALL send a `chat.cancel` WebSocket message to the Gateway when the user clicks the Stop button, causing the backend to cancel the active turn via `SessionOp::Interrupt`.

#### Scenario: User clicks Stop during streaming
- **WHEN** the user clicks the Stop button while the agent is streaming a response
- **THEN** the frontend sends `chat.cancel` with the current session ID via WebSocket
- **THEN** the backend cancels the active turn (LLM call + running tools)
- **THEN** the frontend receives a `turn_aborted` event and updates the UI

#### Scenario: User clicks Stop during tool execution
- **WHEN** the user clicks Stop while a tool is executing (e.g., shell command)
- **THEN** the backend cancels the tool execution with the 100ms grace period
- **THEN** partial tool output is preserved in the message stream
- **THEN** the stream footer returns to the idle (send) state

#### Scenario: Stop with pending approval
- **WHEN** the user clicks Stop while an approval dialog is pending
- **THEN** the pending approval is dismissed
- **THEN** the turn is aborted and the approval card shows "已取消"

### Requirement: transport.ts exposes chatCancel function
The transport layer SHALL export a `chatCancel(sessionId: string)` function that sends the `chat.cancel` WS message.

#### Scenario: chatCancel sends correct WS payload
- **WHEN** `chatCancel("session-123")` is called
- **THEN** the WebSocket client sends `{ method: "chat.cancel", params: { sessionId: "session-123" } }`
