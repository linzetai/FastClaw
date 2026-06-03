# FileChangesCard Spec

## Requirement: 聚合文件变更卡片

在 AI 回复结束后 SHALL 显示一个聚合文件变更卡片，汇总本轮所有 `edit_file` 工具调用的变更。

### Scenario: AI 回复包含文件编辑
- **GIVEN** AI 的本轮回复中有 ≥1 个 `edit_file` 工具调用且 result 为成功
- **WHEN** 消息渲染完成
- **THEN** 在 markdown 内容与 AiReactionBar 之间显示 FileChangesCard
- **AND** 顶栏显示 "N files changed" + `+X -Y` 增删统计（绿/红着色）
- **AND** 每个编辑过的文件显示为一行，包含：文件名（mono 字体）、增删统计、橙色修改指示点、展开箭头

### Scenario: AI 回复无文件编辑
- **GIVEN** AI 的本轮回复中没有 `edit_file` 工具调用
- **WHEN** 消息渲染完成
- **THEN** 不显示 FileChangesCard

### Scenario: 文件行点击
- **GIVEN** FileChangesCard 已渲染
- **WHEN** 用户点击某个文件行
- **THEN** 触发 `xiaolin:open-review` 自定义事件，携带 `{ path: string }`
- **AND** 如果 WorkspacePanel 已打开，自动切换到 Review tab 并定位到该文件

### Scenario: Undo 按钮
- **GIVEN** FileChangesCard 顶栏有 Undo 按钮
- **WHEN** 用户点击 Undo
- **THEN** 本期仅打印提示 "Undo not yet implemented"（后续 spec 定义完整 undo 流程）

### Scenario: 同文件多次编辑
- **GIVEN** AI 对同一个文件执行了多次 `edit_file`
- **WHEN** 聚合统计时
- **THEN** 同路径合并为一行，增删数累加

## Requirement: 样式对齐原型

FileChangesCard SHALL 对齐 `docs/prototype-codex-layout.html` 中 `.fc` 组件的视觉规格。

| 属性 | 值 |
|------|-----|
| 卡片 border | `1px solid var(--border, var(--separator))` |
| 卡片 border-radius | `12px` |
| 卡片 margin | `10px 0 16px` |
| 顶栏 padding | `8px 14px` |
| 顶栏 font-size | `12px` |
| 文件行 font | `var(--font-mono)` |
| 文件行 font-size | `12px` |
| 文件行 hover | `background: var(--bg-hover)` |
| 橙色点 | `6px × 6px, border-radius: 50%, var(--orange)` |
| 增行数颜色 | `var(--green-text, var(--green))` |
| 删行数颜色 | `var(--red-text, var(--red))` |
