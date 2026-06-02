## ADDED Requirements

### Requirement: System notification for pending approvals
The system SHALL send a Tauri system notification when an `approval_required` or `ask_question` event arrives and the app window is not in the foreground.

#### Scenario: Approval needed while app is in background
- **WHEN** an `approval_required` event arrives
- **WHEN** the app window is not focused (user is in another application)
- **THEN** a system notification is sent with title "小林需要确认" and body summarizing the action
- **THEN** clicking the notification brings the app window to the foreground and scrolls to the approval card

#### Scenario: Question asked while app is in background
- **WHEN** an `ask_question` event arrives
- **WHEN** the app window is not focused
- **THEN** a system notification is sent with title "小林有问题要问你" and the question text as body

#### Scenario: App is in foreground — no notification
- **WHEN** an `approval_required` or `ask_question` event arrives
- **WHEN** the app window is focused
- **THEN** no system notification is sent (the in-app UI is sufficient)

### Requirement: Tray icon indicates pending interaction
The system tray icon SHALL visually indicate when agent interaction is pending.

#### Scenario: Tray icon shows attention indicator
- **WHEN** one or more chats have pending approvals or questions
- **THEN** the system tray icon shows a small dot or badge overlay
- **WHEN** all pending interactions are resolved
- **THEN** the tray icon returns to its default state
