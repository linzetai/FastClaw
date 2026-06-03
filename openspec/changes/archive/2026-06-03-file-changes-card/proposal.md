# File Changes Card

## Summary

在 AI 回复中添加聚合文件变更卡片（FileChangesCard），汇总一轮 AI 对话中所有 `edit_file` 工具调用的变更统计，提供"N files changed +X -Y"顶栏、Undo 按钮、按文件列表展示增删统计，点击联动 WorkspacePanel Review 面板。

## Motivation

当前 `DiffCard` 是每个 `edit_file` 工具调用结果的独立渲染。当 AI 一轮对话中编辑了多个文件时，用户缺少一个全局视角来快速了解整体变更范围。原型 `docs/prototype-codex-layout.html` 中的 `.fc` 组件提供了这种聚合视图。

## Scope

- **IN**: 聚合卡片 UI 组件、数据聚合逻辑、与 WorkspacePanel 的联动
- **OUT**: Undo 功能的后端实现（本期仅做 UI 占位）、diff 详情展开（复用现有 DiffCard 的 InlineDiff）

## Dependencies

- layout-overhaul change (已完成大部分)
- 现有 `DiffCard.tsx` 中的 `isEditResult` / `parseEditResult` 工具函数
