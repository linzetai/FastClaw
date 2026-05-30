# 设计方案

## 1. 侧边栏彻底隐藏

**方案**: 折叠时 width=0 + overflow hidden，展开按钮移至 ContentHeader 左侧。

改动文件:
- `SessionList.tsx`: collapsed 时 width: 0, opacity: 0
- `AppLayout.tsx`: ContentHeader 添加展开按钮（PanelLeftOpen 图标），仅 collapsed 时显示
- `ui-store.ts`: 无需改动，已有 sidebarCollapsed 状态

交互细节:
- 折叠动画保留 transition（200ms ease-in-out）
- 展开按钮在 ContentHeader 左侧，仅 collapsed 时显示
- 快捷键保留

## 2. 消息交错渲染

**方案**: 在 stream 结束时将 streamSegments 序列化存储到 Chat 对象，
AiMessage 优先使用 segments 渲染（保持交错），fallback 到旧的 toolCalls+content。

改动文件:
- `types.ts`: Chat 增加 `segments?: StreamSegment[]` 字段
- `useMessageStreamChat.ts`: stream 完成时将 segments 写入 chat
- `MessageRenderer.tsx`: AiMessage 改为优先使用 segments 渲染交错视图
- `agent-store.ts / session-store.ts`: addMessage 时保存 segments

关键决策:
- segments 仅存在前端 localStorage，不持久化到后端（后端仍用 toolCalls+content）
- 从后端恢复的历史会话退化为旧的展示方式（可接受）
- StepGroup 在非 streaming 时默认展开（expanded=true）

## 3. 模型选择器实时同步

**方案**: 在 zustand config-store 层新增 models 状态 + refreshModels action。

改动文件:
- `config-store.ts`: 新增 models[] 状态和 refreshModels action
- `StreamFooter.tsx ModelSelector`: 改为从 store 读取，删除内部 useEffect fetch
- `ModelTab.tsx`: 保存/删除模型后调用 refreshModels()
- ModelSelector 打开下拉时也触发一次 refresh（兜底）

## 4. 风格统一

**方案**: 抽取 shared UI 原子组件到 `components/common/FormElements.tsx`，各页面引用。

需要统一的元素:
- Input: 圆角 var(--radius-xs)、padding px-3 py-2、border 0.5px separator-opaque、focus ring tint
- Button: primary (tint bg) / secondary (bg-secondary) / ghost (transparent hover)
- Card: bg-secondary, 0.5px separator border, radius-xs

改动文件:
- 新增 `components/common/FormElements.tsx`
- `SettingsShared.tsx`: 导出引用 common 组件
- Settings 各 Tab: 替换内联样式
