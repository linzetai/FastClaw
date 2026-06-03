## ADDED Requirements

### Requirement: SessionList subscribes to meta store only
`SessionList` component SHALL subscribe to `useChatMetaStore` for chat list rendering and SHALL NOT subscribe to any stream data.

#### Scenario: Stream update does not re-render SessionList
- **WHEN** a new message is added to a chat via `addMessage`
- **THEN** `SessionList` SHALL NOT re-render
- **AND** `SessionList` SHALL only re-render when chat metadata changes (title, open state, order)

#### Scenario: SessionList reads chatOrder and chats
- **WHEN** `SessionList` renders
- **THEN** it SHALL use `useChatMetaStore(s => s.chatOrder)` and `useChatMetaStore(s => s.chats)` to list chats

### Requirement: MessageStream subscribes to stream store
`MessageStream` component SHALL subscribe to `useStreamStore` for the active chat's stream data.

#### Scenario: MessageStream reads active stream
- **WHEN** `MessageStream` renders
- **THEN** it SHALL read the active chat's stream via `useStreamStore(s => s.streams[activeChatId])` with `activeChatId` sourced from `useChatMetaStore`

#### Scenario: Chat metadata change does not re-render message content
- **WHEN** a different chat is renamed
- **THEN** `MessageStream` SHALL NOT re-render (only metadata store changed for a non-active chat)

### Requirement: useMessageStreamChat writes to correct stores
The `useMessageStreamChat` hook SHALL write streaming events to `useStreamStore` and chat ID updates to `useChatMetaStore`.

#### Scenario: turn_end event handling
- **WHEN** a `turn_end` event is received
- **THEN** `addMessage` SHALL write the final assistant message to `useStreamStore`
- **AND** `updateChatBackendId` SHALL update `useChatMetaStore` (if session ID changed)
- **AND** `updateChatUsage` SHALL write to `useStreamStore.usage`

#### Scenario: Sub-agent events
- **WHEN** `sub_agent_start`, `sub_agent_delta`, or `sub_agent_complete` events are received
- **THEN** all writes SHALL go to `useStreamStore.subAgentRuns`

### Requirement: Selector hooks for cross-store access
The system SHALL provide pre-built selector hooks that combine data from multiple stores without causing unnecessary re-renders.

#### Scenario: useActiveStream selector
- **WHEN** `useActiveStream()` is called
- **THEN** it SHALL return `useStreamStore.streams[activeChatId]` where `activeChatId` comes from `useChatMetaStore`
- **AND** it SHALL return a stable `EMPTY_STREAM` constant when no stream exists (avoiding new reference creation)

#### Scenario: useActiveChatMeta selector
- **WHEN** `useActiveChatMeta()` is called
- **THEN** it SHALL return `useChatMetaStore.chats[activeChatId]`

### Requirement: GatewayStore sync adapted
`syncBackendData` in `store.ts` SHALL write session metadata to `useChatMetaStore` and agent data to `useChatMetaStore`.

#### Scenario: Initial sync on connect
- **WHEN** WebSocket connection is established and `syncBackendData` runs
- **THEN** sessions SHALL be written to `useChatMetaStore` via `syncSessionsForAgent`
- **AND** agents SHALL be written to `useChatMetaStore` via `syncAgentsFromBackend`
- **AND** `useStreamStore` SHALL NOT be populated until individual chats are opened/loaded

### Requirement: Persistence adapts to new store
The persistence layer SHALL read from `useChatMetaStore` for open chats and active chat information.

#### Scenario: saveUIState
- **WHEN** `saveUIState` is triggered
- **THEN** it SHALL read `useChatMetaStore.chats` and `useChatMetaStore.activeChatId` to persist open/active state
- **AND** it SHALL NOT need to read stream data

### Requirement: Tests adapted to new stores
All existing tests SHALL be updated to mock the appropriate store(s) instead of the single `useAgentStore`.

#### Scenario: session-store tests
- **WHEN** running `session-store.test.ts`
- **THEN** tests SHALL interact with both `useChatMetaStore` and `useStreamStore` as appropriate
- **AND** all existing test assertions SHALL continue to pass
