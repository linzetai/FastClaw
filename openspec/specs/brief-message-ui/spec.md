## ADDED Requirements

### Requirement: BriefMessage events rendered in chat stream
The system SHALL render `brief_message` events from the agent as lightweight inline cards in the message stream.

#### Scenario: Normal mode brief message
- **WHEN** a `brief_message` event arrives with `mode: "normal"`
- **THEN** a subtle inline card is inserted into the chat stream
- **THEN** the card has a gray left border and info icon
- **THEN** the card does not interrupt or replace the current streaming output

#### Scenario: Proactive mode brief message
- **WHEN** a `brief_message` event arrives with `mode: "proactive"`
- **THEN** a more prominent inline card is inserted with a blue left border and a distinct icon
- **THEN** the card content is rendered as markdown

#### Scenario: Brief message with attachments
- **WHEN** a `brief_message` event includes attachments (images or files)
- **THEN** the card displays attachment thumbnails or file pills below the text content

### Requirement: BriefMessage does not trigger system notifications
Brief messages SHALL NOT trigger Tauri system notifications, as they are informational only.

#### Scenario: Brief message while app is in background
- **WHEN** a `brief_message` event arrives while the app window is not in focus
- **THEN** no system notification is sent
- **THEN** the unread indicator in the session list updates
