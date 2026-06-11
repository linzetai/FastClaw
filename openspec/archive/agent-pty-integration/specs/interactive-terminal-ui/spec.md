## ADDED Requirements

### Requirement: Agent-created sessions appear in Shell tab
When Agent creates a PTY session via `terminal_open`, the session SHALL automatically appear in the user's Shell tab without user action. The frontend SHALL receive a notification event through the chat WebSocket and establish a PTY WebSocket connection to the new session.

#### Scenario: Agent opens a terminal
- **WHEN** Agent calls `terminal_open` and the backend creates a PTY session
- **THEN** frontend receives a `pty_session_opened` event via chat WebSocket, adds the session to Shell tab, and connects to it

#### Scenario: Agent terminal appears with correct name
- **WHEN** Agent opens a terminal with `name: "Dev Server"`
- **THEN** the session tab displays "Dev Server" with an Agent badge

### Requirement: Agent session badge
Sessions created by the Agent SHALL display a visual "Agent" indicator (badge/icon) in the session tab to distinguish them from user-created sessions.

#### Scenario: Visual distinction
- **WHEN** Shell tab has both user-created and agent-created sessions
- **THEN** agent sessions show a distinguishing badge (e.g., robot icon or "A" tag) while user sessions show the normal terminal icon

### Requirement: User can intervene in agent sessions
Users SHALL be able to type input into agent-created PTY sessions. Both agent input and user input are sent to the same PTY; the session is shared.

#### Scenario: User types in agent session
- **WHEN** user clicks on an agent-created session tab and types a command
- **THEN** the input is sent to the PTY and both agent and user see the resulting output

### Requirement: PtySession source tracking
The `PtySession` store SHALL include a `source` field with values `"user"` or `"agent"` to track session origin.

#### Scenario: Store tracks session source
- **WHEN** a session is created by the agent
- **THEN** `usePtyStore` stores `source: "agent"` for that session
