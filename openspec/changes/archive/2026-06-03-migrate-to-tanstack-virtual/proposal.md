## Why

react-virtuoso 4.18.5+ 在 React 19 + WebKitGTK（Linux Tauri WebView）环境下存在致命渲染 bug：Virtuoso 初始化时内部列表停留在 `visibility: hidden` 状态且不渲染任何子元素（0 children），导致聊天消息完全不可见。此 bug 已在 [react-virtuoso#1391](https://github.com/petyosi/react-virtuoso/issues/1391) 报告但尚无修复版本。

@tanstack/react-virtual 是 headless 虚拟化库，提供一等公民的 Chat UI 支持（`anchorTo: 'end'`、`followOnAppend`、streaming 增长跟踪），且对 React 19 有明确兼容方案（`useFlushSync: false`）。迁移可彻底消除当前的渲染死锁问题。

## What Changes

- **移除** `react-virtuoso` 依赖
- **新增** `@tanstack/react-virtual` 依赖
- **重写** `MessageStream.tsx` 中的虚拟列表渲染层，从 `<Virtuoso>` 组件式 API 迁移为 `useVirtualizer()` hook + 绝对定位布局
- **重构** `useStreamScroll.ts` 滚动管理逻辑，适配 TanStack Virtual 的 API（`scrollToIndex`、`scrollToEnd`、`isAtEnd`、`range`）
- **适配** `StreamFooter.tsx`、`useMessageStreamChat.ts` 等文件中对 `VirtuosoHandle` 的引用
- **移除** Virtuoso 特有的 `visibility: hidden` 初始化机制依赖

## Capabilities

### New Capabilities

- `tanstack-virtual-chat`: 基于 @tanstack/react-virtual 的聊天消息虚拟滚动，覆盖 end-anchoring、follow-on-append、streaming 动态增长、历史消息 prepend 稳定性

### Modified Capabilities

（无现有 spec 级别的需求变更，此迁移为纯实现层替换）

## Impact

- **前端代码**：`MessageStream.tsx`、`useStreamScroll.ts`、`StreamFooter.tsx`、`useMessageStreamChat.ts`
- **依赖变更**：移除 `react-virtuoso`，新增 `@tanstack/react-virtual`
- **bundle 体积**：从 ~17KB（react-virtuoso）降至 ~10-15KB（@tanstack/react-virtual）
- **风险**：
  - 绝对定位布局需要正确处理 `measureElement` 回调，否则动态高度消息可能闪烁
  - `anchorTo: 'end'` 需要稳定的 item key（当前 `displayData` 的 key 需确认唯一性）
  - 搜索高亮滚动定位逻辑需要适配新 API
  - 分页加载（startReached → 手动 scrollTop 检测）需要重新实现
