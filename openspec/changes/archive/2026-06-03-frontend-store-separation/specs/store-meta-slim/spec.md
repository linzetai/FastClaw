## ADDED Requirements

### Requirement: Slim chat metadata store
The system SHALL provide a `useChatMetaStore` that contains only low-frequency metadata fields per chat: `id`, `localKey`, `title`, `workDir`, `source`, `messageCount`, `open`, `createdAt`, `executionMode`, `planFilePath`, `planFileExists`.

#### Scenario: ChatMeta type definition
- **WHEN** defining the `ChatMeta` type
- **THEN** it SHALL NOT include `stream`, `usage`, `lastSegments`, or `subAgentRuns` fields
- **AND** it SHALL include all fields needed by SessionList and ChatTabsBar

#### Scenario: Store structure
- **WHEN** accessing `useChatMetaStore`
- **THEN** it SHALL expose `chats: Record<string, ChatMeta>`, `chatOrder: string[]`, `activeChatId: string`, and `agents: Agent[]`

### Requirement: Chat CRUD on meta store
Chat create, rename, close, reopen, reorder, and delete operations SHALL modify `useChatMetaStore` only (plus stream store cleanup for delete).

#### Scenario: Creating a new chat
- **WHEN** `newChat(workDir?)` is called
- **THEN** a new `ChatMeta` entry SHALL be added to `useChatMetaStore.chats`
- **AND** the new chat id SHALL be appended to `useChatMetaStore.chatOrder`
- **AND** `useChatMetaStore.activeChatId` SHALL be set to the new chat id
- **AND** an empty stream entry SHALL be initialized in `useStreamStore.streams[newId]`

#### Scenario: Closing a chat
- **WHEN** `closeChat(chatId)` is called
- **THEN** `useChatMetaStore.chats[chatId].open` SHALL be set to false (or entry removed if empty)
- **AND** `useStreamStore.streams[chatId]` SHALL be cleaned up if the chat had no messages

#### Scenario: Renaming a chat
- **WHEN** `renameChat(chatId, title)` is called
- **THEN** only `useChatMetaStore.chats[chatId].title` SHALL change
- **AND** `useStreamStore` SHALL NOT be affected

### Requirement: syncSessionsForAgent updates meta store
Backend session sync SHALL update `useChatMetaStore` with session metadata and initialize empty stream entries in `useStreamStore` for new sessions.

#### Scenario: Syncing sessions from backend
- **WHEN** `syncSessionsForAgent(sessions)` is called
- **THEN** new sessions SHALL create entries in both `useChatMetaStore.chats` and `useStreamStore.streams`
- **AND** existing sessions SHALL only update metadata fields (messageCount, workDir, source)
- **AND** existing stream data SHALL NOT be overwritten

### Requirement: Agents data in meta store
Agent configuration (agents array, activeAgentId, syncAgentsFromBackend, updateAgentProps) SHALL remain in `useChatMetaStore`.

#### Scenario: Syncing agents from backend
- **WHEN** `syncAgentsFromBackend(backendAgents)` is called
- **THEN** `useChatMetaStore.agents` SHALL be updated
- **AND** no stream data SHALL be affected
