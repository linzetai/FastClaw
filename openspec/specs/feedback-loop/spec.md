## ADDED Requirements

### Requirement: Thumbs up/down writes to Evolution API
The system SHALL send user feedback (thumbs up/down) to the backend Evolution system for agent improvement.

#### Scenario: User clicks thumbs up on assistant message
- **WHEN** the user clicks the thumbs-up button on an assistant message
- **THEN** a `POST /api/v1/evolution/feedback` request is sent with `{ sessionId, turnId, rating: "positive" }`
- **THEN** the button shows a filled state indicating the feedback was recorded

#### Scenario: User clicks thumbs down on assistant message
- **WHEN** the user clicks the thumbs-down button on an assistant message
- **THEN** a `POST /api/v1/evolution/feedback` request is sent with `{ sessionId, turnId, rating: "negative" }`
- **THEN** the button shows a filled state indicating the feedback was recorded

#### Scenario: User changes feedback
- **WHEN** the user clicks thumbs-up after previously clicking thumbs-down (or vice versa)
- **THEN** a new feedback request is sent with the updated rating
- **THEN** the UI reflects the new selection

### Requirement: Retry button re-executes the turn
The system SHALL allow the user to retry a failed or unsatisfactory assistant turn.

#### Scenario: User clicks retry on assistant message
- **WHEN** the user clicks the retry button on an assistant message
- **THEN** the system sends a retry request to the backend
- **THEN** the original assistant message is replaced with a new streaming response
- **THEN** the streaming indicator appears as with a normal turn

#### Scenario: Retry during streaming
- **WHEN** the user attempts to retry while another turn is streaming
- **THEN** the retry action is disabled (grayed out)

### Requirement: DenialTracker prevents repeated approval requests
The system SHALL track user denials and prevent the agent from repeatedly requesting approval for the same type of operation within a session.

#### Scenario: User denies a shell command
- **WHEN** the user denies approval for `rm -rf ./temp`
- **THEN** subsequent `rm` commands in the same session are auto-denied without showing the approval card
- **THEN** the agent receives a denial with reason "用户已拒绝此类操作"

#### Scenario: Denial tracking resets on new session
- **WHEN** a new chat session is started
- **THEN** the denial tracker is cleared
- **THEN** previously denied operation types can be approved again
