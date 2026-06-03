## ADDED Requirements

### Requirement: Independent stream store
The system SHALL provide an independent Zustand store (`useStreamStore`) that manages all per-chat message stream data, including `streams`, `usage`, `lastSegments`, and `subAgentRuns`.

#### Scenario: Stream store creation
- **WHEN** the application initializes
- **THEN** `useStreamStore` SHALL be a standalone `create()` store, separate from any chat metadata store

#### Scenario: Stream data structure
- **WHEN** accessing stream data
- **THEN** the store SHALL expose `streams: Record<string, StreamItem[]>`, `usage: Record<string, ChatUsage>`, `lastSegments: Record<string, ChatStreamSegment[]>`, and `subAgentRuns: Record<string, Record<string, SubAgentRunUI>>`

### Requirement: addMessage writes to stream store only
The `addMessage` action SHALL write new messages exclusively to `useStreamStore.streams[chatId]` without modifying or triggering updates on any chat metadata store entry.

#### Scenario: Adding a user message
- **WHEN** `addMessage(chatId, { role: "user", content: "hello" })` is called
- **THEN** a new `StreamItem` SHALL be appended to `useStreamStore.streams[chatId]`
- **AND** `useChatMetaStore` SHALL have its `messageCount` incremented synchronously
- **AND** `useChatMetaStore.chats` reference SHALL change but `streams` reference SHALL remain unchanged from the meta store's perspective

#### Scenario: Adding an assistant message
- **WHEN** `addMessage(chatId, { role: "assistant", content: "..." })` is called
- **THEN** the stream item SHALL be appended to `useStreamStore.streams[chatId]`
- **AND** components subscribed only to `useChatMetaStore` SHALL NOT re-render

### Requirement: appendStreamDelta writes to stream store only
The `appendStreamDelta` action SHALL update `useStreamStore.streams[chatId]` without creating a new `chatList` or `chatMeta` reference.

#### Scenario: Appending delta during streaming
- **WHEN** `appendStreamDelta(chatId, "token")` is called
- **THEN** the last assistant message in `useStreamStore.streams[chatId]` SHALL have its content appended
- **AND** no chat metadata store subscriber SHALL be notified

### Requirement: Usage and segments isolation
`updateChatUsage` and `setChatLastSegments` SHALL write exclusively to `useStreamStore.usage` and `useStreamStore.lastSegments` respectively.

#### Scenario: Updating usage after turn_end
- **WHEN** `updateChatUsage(chatId, usageData)` is called
- **THEN** `useStreamStore.usage[chatId]` SHALL be updated
- **AND** the chat metadata store SHALL NOT be affected

#### Scenario: Setting last segments
- **WHEN** `setChatLastSegments(chatId, segments)` is called
- **THEN** `useStreamStore.lastSegments[chatId]` SHALL be updated
- **AND** only components subscribing to `useStreamStore` with that chatId SHALL re-render

### Requirement: SubAgent state isolation
All sub-agent operations (`subAgentStart`, `subAgentDelta`, `subAgentToolStart`, `subAgentToolDone`, `subAgentComplete`) SHALL write to `useStreamStore.subAgentRuns[chatId]`.

#### Scenario: SubAgent delta update
- **WHEN** `subAgentDelta(chatId, runId, content)` is called
- **THEN** `useStreamStore.subAgentRuns[chatId][runId].content` SHALL be appended
- **AND** the chat metadata store SHALL NOT be notified

### Requirement: loadChatStream populates stream store
The `loadChatStream` action SHALL write backend messages to `useStreamStore.streams[chatId]` and update `useChatMetaStore.chats[chatId].messageCount`.

#### Scenario: Loading chat history from backend
- **WHEN** `loadChatStream(chatId, backendMessages)` is called
- **THEN** `useStreamStore.streams[chatId]` SHALL be set to the parsed messages
- **AND** `useChatMetaStore.chats[chatId].messageCount` SHALL be updated to match
