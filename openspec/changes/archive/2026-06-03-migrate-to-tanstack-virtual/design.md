## Context

当前聊天消息列表使用 `react-virtuoso`（v4.18.7）实现虚拟滚动。在 React 19.2.5 + WebKitGTK（Linux Tauri）环境下，Virtuoso 内部列表在初始化时卡在 `visibility: hidden` 状态，0 个子元素被渲染，导致消息完全不可见。

涉及 react-virtuoso 的文件仅 2 个：
- `MessageStream.tsx`：使用 `<Virtuoso>` 组件渲染消息列表
- `useStreamScroll.ts`：通过 `VirtuosoHandle` API 管理滚动行为

数据模型：`StreamItem = { type: "message", data: ChatMessage } | { type: "brief", data: BriefMessageData }`，每条消息有唯一 `data.id`。

## Goals / Non-Goals

**Goals:**
- 消除 react-virtuoso 的 `visibility: hidden` 渲染死锁 bug
- 保持现有滚动行为：底部跟随、streaming 时自动滚动、scroll FAB、搜索定位
- 保持分页加载（向上滚动加载历史消息）
- 使用 headless API 获得完全渲染控制，避免黑盒行为

**Non-Goals:**
- 不改变消息数据模型或 store 结构
- 不改变 `MessageRendererRow` 的渲染逻辑
- 不改变 UI 外观或交互行为
- 不引入新的滚动行为

## Decisions

### 1. 使用 `@tanstack/react-virtual` 的 `useVirtualizer` hook

**选择**：`useVirtualizer` + 绝对定位布局

**替代方案**：
- 方案 B：移除虚拟化，用 `div.map()` — 简单但消息多时性能差
- 方案 C：降级 react-virtuoso — 只是暂时绕过，React 19 tearing 风险

**理由**：TanStack Virtual 的 Chat guide 提供了我们需要的全部能力（`anchorTo: 'end'`、`followOnAppend`、streaming 增长跟踪），且 headless 设计不存在 `visibility: hidden` 黑盒机制。

### 2. 布局方案：绝对定位 + `measureElement`

```
┌─ scrollContainer (ref=parentRef, overflow:auto, flex:1) ─┐
│  ┌─ sizer div (height = virtualizer.getTotalSize()) ─┐   │
│  │  ┌─ item (position:absolute, translateY) ──────┐  │   │
│  │  │  <MessageRendererRow ... />                  │  │   │
│  │  └─ ref={virtualizer.measureElement} ──────────┘  │   │
│  │  ┌─ item ─────────────────────────────────────┐   │   │
│  │  │  ...                                        │  │   │
│  │  └────────────────────────────────────────────┘   │   │
│  └───────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────┘
```

每个 virtual item 使用 `ref={virtualizer.measureElement}` 实现动态高度测量，`data-index` 属性用于映射。

### 3. `getItemKey` 策略

```typescript
getItemKey: (index) => {
  const item = displayData[index];
  if (!item) return index;
  if ('key' in item && item.key === '_streaming_') return '_streaming_';
  if (item.type === 'message') return `msg-${item.data.id}`;
  if (item.type === 'brief') return `brief-${item.data.id}`;
  return index;
}
```

稳定的 key 是 `anchorTo: 'end'` prepend 补偿的基础，必须使用消息 ID 而非数组索引。

### 4. 分页加载（startReached 替代方案）

TanStack Virtual 没有 `startReached` 回调。改用滚动位置检测：

```typescript
const onScroll = useCallback(() => {
  const el = parentRef.current;
  if (!el || !hasMore) return;
  if (el.scrollTop < 200) {
    // 触发加载更多
    loadMore();
  }
}, [hasMore]);
```

加载后 `anchorTo: 'end'` 会自动保持可视消息位置不变，不需要手动补偿 scrollTop。

### 5. useStreamScroll API 映射

| react-virtuoso API | TanStack Virtual API | 说明 |
|---|---|---|
| `scrollToIndex({index, align, behavior})` | `virtualizer.scrollToIndex(index, {align, behavior})` | 参数顺序变化 |
| `scrollToIndex({index: "LAST"})` | `virtualizer.scrollToEnd()` | 专用方法 |
| `scrollTo({top})` | `parentRef.current.scrollTop = top` | 直接操作 DOM |
| `getState(cb)` | `virtualizer.range` | 同步访问 |
| `atBottomStateChange` | `virtualizer.isAtEnd()` | 在 scroll handler 中检测 |
| `followOutput` | `followOnAppend: true` + `anchorTo: 'end'` | 声明式配置 |

### 6. React 19 兼容配置

```typescript
useFlushSync: false  // 避免 React 19 flushSync 警告
```

## Risks / Trade-offs

- **[动态高度闪烁]** → `measureElement` 测量在 DOM 渲染后发生，首次渲染可能有轻微跳动。缓解：设置合理的 `estimateSize`（如 80px）减少误差
- **[streaming 消息尺寸变化]** → 最后一条消息 token 流入时高度持续增长。缓解：`anchorTo: 'end'` 原生支持此场景
- **[搜索滚动定位]** → 当前通过 `scrollToIndex` + `mark.scrollIntoView` 双重定位。TanStack Virtual 的 `scrollToIndex` 语义略有不同（使用 `measureElement` 的实际高度），需要验证定位精度
- **[Header/Footer]** → react-virtuoso 的 `components.Header/Footer` 在 TanStack Virtual 中不存在。改为在滚动容器内部、sizer div 外部直接渲染
- **[分页加载 race condition]** → 滚动检测可能在快速滚动时多次触发。缓解：用 `loadingRef` 防止并发加载
