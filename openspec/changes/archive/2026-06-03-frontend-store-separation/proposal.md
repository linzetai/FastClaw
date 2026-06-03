## Why

前端唯一的 Zustand store（`useAgentStore`）将聊天元数据、消息流、子 Agent 运行状态、消息队列、UI 状态全部嵌套在 `agentChats[agentId].chatList[n]` 里。每次 streaming delta、消息追加、subAgent 更新都需要展开整条 `agentChats → chatList → stream` 链路创建新引用，导致侧边栏、标签栏、输入区等不相关组件级联重渲染。在长对话场景下，`chatList.map()` 的 GC 压力和重渲染开销显著影响用户体验。

## What Changes

- 将「高频变化的消息流数据」（stream、usage、lastSegments、subAgentRuns）从 `chatList` 中分离到独立的 `useStreamStore`，使消息更新不触发侧边栏和标签栏重渲染
- 将「中频变化的消息队列」从 `agentChats` 中分离到独立的 `useQueueStore`
- `chatList` 退化为纯元数据容器（id、title、messageCount、workDir 等），重构为 `useChatMetaStore`
- UI 状态（sidebarCollapsed、activeNav、detailOpen）保留在独立的 `useUIStore`
- 所有消费者（`MessageStream`、`SessionList`、`StreamFooter` 等）迁移到按需订阅对应 store 的精确 selector

## Capabilities

### New Capabilities
- `store-stream-isolation`: 将消息流数据（stream、usage、segments、subAgentRuns）分离到独立 store，实现与元数据的渲染隔离
- `store-meta-slim`: 聊天元数据 store 只包含低频变化的 metadata 字段，侧边栏和标签栏精准订阅
- `store-consumer-migration`: 所有组件和 hook 迁移到新的多 store 架构，用精确 selector 订阅

### Modified Capabilities

## Impact

- `src/lib/stores/` — 全部 store 文件重构（session-store.ts、agent-store.ts、ui-store.ts、selectors.ts、index.ts、types.ts）
- `src/components/message-stream/MessageStream.tsx` — 改为订阅 streamStore
- `src/components/message-stream/useMessageStreamChat.ts` — 写入拆分到 streamStore 和 chatMetaStore
- `src/components/message-stream/MessageRenderer.tsx` — props 精简
- `src/components/session-list/SessionList.tsx` — 改为订阅 chatMetaStore
- `src/components/message-stream/StreamFooter.tsx` — 改为订阅精确字段
- `src/components/message-stream/SubAgentMonitor.tsx` — 改为订阅 streamStore
- `src/lib/store.ts` (GatewayStore) — syncBackendData 拆分写入
- `src/lib/stores/persistence.ts` — 适配新 store 结构
- 所有 `__tests__/` 测试文件适配
