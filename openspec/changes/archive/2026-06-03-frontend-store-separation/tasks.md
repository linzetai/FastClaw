## 1. 类型定义与新 Store 创建

- [x] 1.1 在 `src/lib/stores/types.ts` 中新增 `ChatMeta` 类型（从 `Chat` 中去掉 stream/usage/lastSegments/subAgentRuns），同时保留原 `Chat` 类型不删除（过渡期共存）
- [x] 1.2 创建 `src/lib/stores/stream-store.ts`，定义 `StreamState` 接口和 `useStreamStore = create<StreamState>()`，包含 `streams`/`usage`/`lastSegments`/`subAgentRuns` 四个 Record 字段和对应的 action
- [x] 1.3 创建 `src/lib/stores/chat-meta-store.ts`，定义 `ChatMetaState` 接口和 `useChatMetaStore = create<ChatMetaState>()`，包含 `chats`/`chatOrder`/`activeChatId`/`agents` 字段和 CRUD action
- [x] 1.4 创建 `src/lib/stores/queue-store.ts`，定义 `QueueState` 和 `useQueueStore`，将 `enqueueMessage`/`dequeueMessage`/`updateQueuedMessage`/`removeQueuedMessage`/`clearQueue`/`reorderQueue` 迁移过来
- [x] 1.5 重构 `src/lib/stores/ui-store.ts` 为独立 `useUIStore = create()`，不再作为 buildUISlice 注入

## 2. Store Action 迁移

- [x] 2.1 将 `addMessage` 实现迁移到 `stream-store.ts`，写入 `streams[chatId]`，并同步调用 `useChatMetaStore.getState().incrementMessageCount(chatId)`
- [x] 2.2 将 `appendStreamDelta` 迁移到 `stream-store.ts`，只更新 `streams[chatId]`
- [x] 2.3 将 `updateChatUsage` 迁移到 `stream-store.ts`，写入 `usage[chatId]`，同时更新对应 stream 中最后一条 assistant message 的 usage
- [x] 2.4 将 `setChatLastSegments` 迁移到 `stream-store.ts`，写入 `lastSegments[chatId]`
- [x] 2.5 将 5 个 subAgent action（start/delta/toolStart/toolDone/complete）迁移到 `stream-store.ts`，写入 `subAgentRuns[chatId]`
- [x] 2.6 将 `addBriefMessage` 迁移到 `stream-store.ts`
- [x] 2.7 将 `loadChatStream` 迁移到 `stream-store.ts`，写入 `streams[chatId]`，同步更新 `useChatMetaStore` 的 `messageCount`
- [x] 2.8 将 `newChat`/`setActiveChat`/`closeChat`/`reopenChat`/`renameChat`/`reorderChats`/`setWorkDir`/`clearUnread` 迁移到 `chat-meta-store.ts`
- [x] 2.9 将 `syncSessionsForAgent` 迁移到 `chat-meta-store.ts`，并在其中初始化新 session 的空 stream 条目
- [x] 2.10 将 `updateChatBackendId` 拆分：`chat-meta-store.ts` 更新 chat id 映射，`stream-store.ts` 更新 streams key
- [x] 2.11 将 `setChatExecutionMode`/`setChatPlanFile` 迁移到 `chat-meta-store.ts`
- [x] 2.12 将 agents 相关 action（syncAgentsFromBackend/updateAgentProps/removeAgent/setActiveAgent）迁移到 `chat-meta-store.ts`

## 3. Selector Hooks

- [x] 3.1 更新 `src/lib/stores/selectors.ts`：新增 `useActiveStream()`、`useActiveChatMeta()`、`useChatUsage(chatId)` 等跨 store selector，使用 `EMPTY_STREAM` 常量避免空数组新引用
- [x] 3.2 更新 `src/lib/stores/index.ts`：导出所有新 store 和 selector，保留 `useAgentStore` 的 re-export（过渡期兼容）

## 4. 消费者组件迁移

- [x] 4.1 迁移 `MessageStream.tsx`：从 `useStreamStore` 读取 stream/subAgentRuns，从 `useChatMetaStore` 读取 activeChatId/activeChat metadata
- [x] 4.2 迁移 `useMessageStreamChat.ts`：所有 store action 调用改为从对应 store 获取（stream 写入 → streamStore，chat id 更新 → chatMetaStore，queue 操作 → queueStore）
- [x] 4.3 迁移 `MessageRenderer.tsx`：props 中的 `subAgentRuns` 改为从 streamStore selector 获取
- [x] 4.4 迁移 `SessionList.tsx`：改为订阅 `useChatMetaStore`（chats + chatOrder + activeChatId），不再引用 stream 数据
- [x] 4.5 迁移 `StreamFooter.tsx`：queue 相关操作改为 `useQueueStore`，streaming 状态从 props 传入
- [x] 4.6 迁移 `SubAgentMonitor.tsx`：从 `useStreamStore.subAgentRuns` 读取
- [x] 4.7 迁移 `AppLayout.tsx`：UI 状态从 `useUIStore` 读取，agents 从 `useChatMetaStore` 读取
- [x] 4.8 迁移 `NavRail.tsx`/`TitleBar.tsx`/`ContentHeader`：使用 `useUIStore`
- [x] 4.9 迁移 `ChatTabsBar.tsx`：从 `useChatMetaStore` 读取 chatOrder + chats
- [x] 4.10 迁移 `StreamEmptyState.tsx`/`PlanApprovalCard.tsx`/`SecurityTab.tsx` 等使用 `useAgentStore` 的组件
- [x] 4.11 迁移 `store.ts`（GatewayStore）：`syncBackendData` 改为调用 `useChatMetaStore` 的 sync 方法

## 5. 持久化与清理

- [x] 5.1 更新 `persistence.ts`：`saveUIState` 从 `useChatMetaStore` 读取 open/active 状态，`_persisted` 初始化逻辑适配
- [x] 5.2 更新 `index.ts` 中的 `subscribe` 逻辑：监听 `useChatMetaStore` 变化来触发持久化（替代当前监听 `agentChats` 的方式）
- [x] 5.3 删除旧 `useAgentStore`（在所有消费者迁移完成后），清理 `session-store.ts` 和 `agent-store.ts` 中已被迁移的代码

## 6. 测试适配

- [x] 6.1 更新 `session-store.test.ts`：改为测试 `useChatMetaStore` + `useStreamStore` 的交互
- [x] 6.2 更新 `agent-store-persistence.test.ts`：适配新的持久化读取源
- [x] 6.3 更新 `chat-stream-events.test.ts`：适配 stream 事件写入 `useStreamStore` 的路径
- [x] 6.4 更新 `transport.test.ts`：如有 store 引用需适配
- [x] 6.5 更新 `ChatTabsBar.test.tsx` 和 `ToolCallCard.test.tsx`：mock 改为对应 store
- [x] 6.6 运行 `pnpm test` 确认全部测试通过

## 7. 验证

- [x] 7.1 运行 `tsc && vite build` 确认编译无错误
- [x] 7.2 启动 `cargo tauri dev`，手动验证基本流程：新建聊天 → 发送消息 → streaming → 侧边栏不闪烁 → 切换 tab → 关闭 tab
- [x] 7.3 使用 React DevTools Profiler 对比重构前后 SessionList 在 streaming 时的重渲染次数
