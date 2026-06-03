# FileChangesCard Tasks

## 1. useFileChangeSummary hook

从 `toolCalls` 或 `savedSegments` 中筛选 `edit_file` 类型并聚合统计数据。

- [x] 1.1 创建 `useFileChangeSummary.ts` hook，接收 toolCalls/segments，返回 `FileChangeSummary | null`
- [x] 1.2 实现同路径合并逻辑（同文件多次编辑，增删数累加）
- [x] 1.3 复用 `DiffCard.tsx` 中的 `parseEditResult` 函数（提取为共享 util）

## 2. FileChangesCard 组件

实现聚合卡片的 UI 渲染。

- [x] 2.1 创建 `FileChangesCard.tsx`，渲染 fc-top（N files changed + stats + Undo 占位）
- [x] 2.2 渲染 fc-row 列表（文件名 + 增删统计 + 橙色点 + 展开箭头）
- [x] 2.3 超过 5 个文件时折叠，显示 "show N more" 按钮
- [x] 2.4 文件行点击触发 `xiaolin:open-review` 事件
- [x] 2.5 对齐原型样式（border-radius, padding, font, hover 等）

## 3. 集成到 MessageRenderer

- [x] 3.1 在 `MessageRenderer` 中调用 `useFileChangeSummary`，在 AiReactionBar 之前渲染 FileChangesCard
- [x] 3.2 TS 编译 + 截图验证

## 4. 与现有 DiffCard 的关系优化

- [x] 4.1 评估是否在 FileChangesCard 存在时隐藏 StepIndicator 中的独立 DiffCard（避免重复信息）
