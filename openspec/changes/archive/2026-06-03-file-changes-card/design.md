# FileChangesCard — 技术设计

## 架构

```
MessageRenderer (assistant turn)
  ├── ToolCalls / StepIndicators  (现有)
  ├── MarkdownContent             (现有)
  ├── AiReactionBar               (现有)
  └── FileChangesCard  ← NEW
        ├── fc-top: "N files changed +X -Y" + Undo
        └── fc-row[] × N: 文件名 + 增删统计 + 橙色点 + 展开箭头
```

## 数据流

1. `MessageRenderer` 已有 `toolCalls` 或 `savedSegments` 数据
2. 新增 `useFileChangeSummary(toolCalls | segments)` hook，从中筛选 `edit_file` 类型的工具调用，聚合为:
   ```ts
   interface FileChangeSummary {
     totalFiles: number;
     totalAdded: number;
     totalRemoved: number;
     files: Array<{
       path: string;
       linesAdded: number;
       linesRemoved: number;
       replacements: number;
     }>;
   }
   ```
3. 如果 `totalFiles > 0`，渲染 `FileChangesCard` 组件

## 组件设计

### FileChangesCard

- **位置**: 在 `AiReactionBar` 之前渲染
- **展开/折叠**: 默认展示文件列表；超过 5 个文件时只显示前 5 个 + "show N more"
- **Undo 按钮**: 本期仅 UI 占位，点击 `console.warn("Undo not yet implemented")`
- **文件行点击**: 触发 `xiaolin:open-review` 事件，携带 `{ path }` 参数，联动 WorkspacePanel

### 与现有 DiffCard 的关系

- `DiffCard` 保留，仍在 `StepIndicator` 内渲染单个工具调用的 diff
- `FileChangesCard` 是 AI 回复级别的聚合视图，两者互补
- 可考虑未来用 `FileChangesCard` 替代多个独立 `DiffCard`

## 样式规格（对齐原型 `.fc`）

| 元素 | 样式 |
|------|------|
| 卡片容器 | `border: 1px solid var(--border); border-radius: 12px; overflow: hidden; margin: 10px 0 16px;` |
| fc-top | `display: flex; align-items: center; padding: 8px 14px; font-size: 12px; font-weight: 500; color: var(--text-2);` |
| fc-top stats | `font-family: var(--font-mono); font-size: 11px; margin-left: 6px;` 绿色/红色着色 |
| fc-top undo | `margin-left: auto; font-size: 11px; color: var(--text-3);` hover `color: var(--accent)` |
| fc-row | `display: flex; align-items: center; gap: 6px; padding: 6px 14px; font-size: 12px; font-family: var(--font-mono); color: var(--text-2); border-top: 1px solid var(--border-subtle);` |
| fc-row hover | `background: var(--bg-hover)` |
| fc-row 橙色点 | `width: 6px; height: 6px; border-radius: 50%; background: var(--orange)` |
| fc-row 展开箭头 | `color: var(--text-4); font-size: 14px;` |
