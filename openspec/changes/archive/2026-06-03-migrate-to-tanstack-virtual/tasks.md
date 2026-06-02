## 1. 依赖替换

- [x] 1.1 安装 `@tanstack/react-virtual` 依赖
- [x] 1.2 移除 `react-virtuoso` 依赖
- [x] 1.3 清理 `pnpm-lock.yaml` 并验证 `pnpm install` 无报错

## 2. useStreamScroll 重构

- [x] 2.1 移除 `VirtuosoHandle` 类型引用，改为接收 `Virtualizer` 实例和 `scrollContainerRef`
- [x] 2.2 将 `scrollToIndex({index, align, behavior})` 调用迁移为 `virtualizer.scrollToIndex(index, {align, behavior})`
- [x] 2.3 将 `scrollToIndex({index: "LAST"})` 迁移为 `virtualizer.scrollToEnd()`
- [x] 2.4 将 `scrollTo({top})` 迁移为 `scrollContainerRef.current.scrollTop = top`
- [x] 2.5 将 `getState(cb)` 迁移为读取 `virtualizer.range`
- [x] 2.6 将 `startReached` 回调迁移为滚动位置检测（`scrollTop < 200` 时触发加载）
- [x] 2.7 搜索定位逻辑适配新 `scrollToIndex` API

## 3. MessageStream 渲染层重写

- [x] 3.1 移除 `<Virtuoso>` 组件及其 import
- [x] 3.2 创建 `useVirtualizer` hook 配置：`anchorTo: 'end'`、`followOnAppend: true`、`scrollEndThreshold: 120`、`useFlushSync: false`
- [x] 3.3 实现 `getItemKey` 函数：message → `msg-{id}`，brief → `brief-{id}`，streaming → `_streaming_`
- [x] 3.4 实现绝对定位布局：scrollContainer → sizer div → virtual items（`translateY` 定位 + `measureElement` ref）
- [x] 3.5 将 Header/Footer 从 Virtuoso components 迁移为容器内直接渲染的 div
- [x] 3.6 将 `atBottomStateChange` 迁移为在 scroll handler 中调用 `virtualizer.isAtEnd()`
- [x] 3.7 初始加载时调用 `virtualizer.scrollToEnd()` 滚动到底部

## 4. 辅助逻辑适配

- [x] 4.1 适配 `handleRangeChanged`：从 Virtuoso `rangeChanged` 迁移为读取 `virtualizer.range`
- [x] 4.2 适配窗口 resize 时的 scrollToIndex 逻辑
- [x] 4.3 适配 streaming 完成后的 scrollToIndex 逻辑
- [x] 4.4 适配 `scrollToBottom` 函数

## 5. 验证

- [x] 5.1 `pnpm tsc --noEmit` 类型检查通过
- [ ] 5.2 启动 `cargo tauri dev`，Tauri MCP 截图验证消息正常渲染
- [ ] 5.3 验证 end-anchor 行为：新消息到达时底部跟随
- [ ] 5.4 验证 streaming 时底部保持 pinned
- [ ] 5.5 验证向上滚动时 scroll FAB 出现
- [ ] 5.6 验证搜索定位功能正常
- [ ] 5.7 验证切换会话后列表正确重建
- [ ] 5.8 验证代码中无 `react-virtuoso` 残留引用
