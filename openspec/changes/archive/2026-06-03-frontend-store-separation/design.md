## Context

XiaoLin 前端使用单一 Zustand store（`useAgentStore`）管理所有状态。核心数据结构是 `agentChats: Record<string, AgentChats>`，其中每个 `AgentChats` 包含 `chatList: Chat[]`，而每个 `Chat` 又嵌套了 `stream: StreamItem[]`、`subAgentRuns`、`usage`、`lastSegments` 等高频变化字段。

当前架构下，任何 stream 变化都需要重建 `chatList` 数组，导致订阅 `agentChats` 的所有组件（SessionList、ChatTabsBar、StreamFooter 等）不必要地重渲染。这在 AI streaming 场景下尤为突出——`turn_end` 时的 `addMessage` 和 `updateChatUsage` 会连续触发多次全链路更新。

技术栈：React 19 + Zustand 5 + Vite 8 + TypeScript 6 + TailwindCSS 4，运行在 Tauri v2 WebView 中。

## Goals / Non-Goals

**Goals:**
- 消除消息流更新对侧边栏（SessionList）和标签栏（ChatTabsBar）的级联重渲染
- 降低 `addMessage`、`updateChatUsage`、`subAgentDelta` 等操作的 GC 压力
- 保持渐进式迁移能力——可以分 Phase 实施，每个 Phase 独立可验证
- 保持现有功能完全不变（纯重构，无行为变更）

**Non-Goals:**
- 不引入 Immer 或 Mutative（可在后续独立优化中引入）
- 不重构 streaming 机制本身（当前 `segmentsRef` + RAF 批量刷新已经是正确做法）
- 不改变持久化策略（当前只持久化 openChats/activeChat 信息）
- 不改变 WebSocket 通信层
- 不做 UI 变更

## Decisions

### D1: 多独立 store vs 单 store 多 slice

**选择**：多个独立 `create()` store

**替代方案**：单 store 使用 `subscribeWithSelector` + 精确 selector

**理由**：Zustand 5 的 selector 基于 `Object.is` 浅比较。单 store 中即使 selector 只取 `state.someSlice`，只要 store root 发生任何变化，selector 函数就会被调用（虽然可能因为返回值相同而不触发重渲染）。多个独立 store 则完全隔离——`useStreamStore` 的变化不会导致 `useChatMetaStore` 的任何 subscriber 被通知。

对于本项目，stream 更新频率远高于 meta 更新频率，完全隔离的收益明显。

### D2: 数据结构选择 — Map vs Record vs 数组

**选择**：仍使用 `Record<string, T>` + 有序 ID 数组

**替代方案**：`Map<string, T>`

**理由**：
- 当前代码全部使用 `Record` 和 `Array`，迁移到 Map 需要改动所有访问方式
- Map 不能直接 JSON 序列化，影响 `localStorage` 持久化和调试
- Record 的 `{...old, [key]: val}` 展开在 V8 引擎中已经足够高效
- 可以用 chatOrder 数组维护有序性：`chatOrder: string[]`，`chats: Record<string, ChatMeta>`

### D3: Store 划分边界

**选择**：4 个 store

| Store | 职责 | 更新频率 |
|---|---|---|
| `useChatMetaStore` | 聊天元数据（id, title, messageCount, workDir, open, executionMode, createdAt）+ activeChatId + agents | 低（用户操作/后端同步） |
| `useStreamStore` | 消息流（streams）、token 用量（usage）、保存的 segments（lastSegments）、subAgent 运行态 | 高（每次 turn_end、每次 addMessage） |
| `useQueueStore` | 消息队列（enqueue/dequeue/reorder） | 中（用户排队操作） |
| `useUIStore` | sidebarCollapsed, activeNav, detailOpen | 极低（UI 切换） |

**替代方案**：只拆 stream + 其余合一

**理由**：Queue 虽然更新频率不高，但语义上独立、与 stream/meta 无关联。拆出后 3 个 store 各自职责清晰，也便于未来为 queue 添加持久化。

### D4: 跨 store 一致性保证

**选择**：在 action 中同步调用多个 store

```ts
// streamStore.addMessage 内部同步递增 chatMetaStore 的 messageCount
addMessage: (chatId, msg) => {
  set(state => { /* 更新 streams */ });
  useChatMetaStore.getState().incrementMessageCount(chatId);
}
```

**替代方案**：事件驱动（store A 发事件，store B 订阅）

**理由**：
- Zustand `getState()` 是同步的，不会有中间状态问题
- React 19 的自动 batching 保证两次 `set` 在同一个事件循环中合并为一次重渲染
- 比事件驱动更直观、更易调试
- 对于 `turn_end` 同时更新 stream + usage + meta 的场景，所有 `set` 调用在同一个 microtask 中完成

### D5: selector 设计策略

**选择**：为每个常见的订阅模式提供预定义 selector hook

```ts
// stream-selectors.ts
export function useActiveStream() {
  const activeChatId = useChatMetaStore(s => s.activeChatId);
  return useStreamStore(s => s.streams[activeChatId] ?? EMPTY_STREAM);
}

export function useChatUsage(chatId: string) {
  return useStreamStore(s => s.usage[chatId]);
}
```

**理由**：
- 避免组件内写跨 store 逻辑
- `EMPTY_STREAM` 常量避免每次返回新空数组引用
- 集中管理便于后续优化

## Risks / Trade-offs

**[跨 store 操作复杂度增加]** → 通过预定义 helper 函数（如 `addMessageToStream`）封装跨 store 写入，对外暴露统一 API

**[调试时需要查看多个 store]** → 在 dev mode 给每个 store 加 `devtools` middleware，Redux DevTools 中可以分别看到各 store 的变化

**[持久化适配]** → 当前只持久化 `agentOpenChats` 和 `agentActiveChats`，迁移到 chatMetaStore 后逻辑不变，只是引用来源改变

**[测试迁移]** → 现有 4 个测试文件引用 `useAgentStore`，需要适配。mock 方式从 mock 单 store 改为 mock 对应 store

**[代码膨胀风险]** → 4 个 store 文件 + selector 文件 + helper 文件，比现在的 2 个文件多。但每个文件更短更聚焦，净可维护性提升

**[渐进迁移期的过渡态]** → Phase 1 只抽 streamStore，保留 chatList 中的 meta 字段。此期间 chatMetaStore 和 agentStore 共存。Phase 2 完成后删除旧 store
