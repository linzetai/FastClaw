## MODIFIED Requirements

### Requirement: QuickActionBar sends input to Gateway
The QuickActionBar SHALL send user input to the Gateway via WebSocket to create a new conversation or append to the current one, instead of only logging to console.

#### Scenario: User submits input via QuickActionBar
- **WHEN** the user types a message in the QuickActionBar and presses Enter
- **THEN** the input is sent to the Gateway via `chat.send` WebSocket message
- **THEN** a new session is created (or the active session is used) and the agent starts processing
- **THEN** the main window is shown and focused on the active conversation

#### Scenario: QuickActionBar with empty input
- **WHEN** the user presses Enter with an empty input
- **THEN** no message is sent and the QuickActionBar remains open
