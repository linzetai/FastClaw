## Why

XiaoLin 桌面应用的整体布局存在多个结构性问题：侧边栏宽度固定无法调整、ContentHeader 与 ChatTabsBar 功能重叠浪费垂直空间、多 Tab 改为下拉菜单后丧失一览性、SubAgentMonitor 面板始终占据右侧空间、消息区 `maxWidth: 720px` 硬编码导致宽屏下空间浪费、窄窗口无响应式适配、代码块缺少行号。这些问题影响信息密度和操作效率，需要系统性优化。

## What Changes

- **侧边栏可拖拽调整宽度**：SessionList 增加拖拽手柄，支持 180px–400px 范围内自由调整，宽度通过 `useUIStore` 持久化
- **合并 ContentHeader 到 ChatTabsBar**：消除重复的 44px 标题栏，将搜索按钮和侧边栏展开按钮整合到 ChatTabsBar 中
- **恢复多 Tab 横排布局**：从下拉菜单恢复为横向可滚动的 Tab 栏，溢出时显示省略指示器，保留拖拽排序
- **SubAgentMonitor 改为 overlay/drawer**：从固定右侧面板改为底部抽屉或浮动面板，无活跃子智能体时完全不占空间
- **响应式适配**：定义三个断点（≤700px 紧凑 / 700–1200px 标准 / ≥1200px 宽屏），NavRail 在紧凑模式下可折叠，消息区 maxWidth 随窗口宽度自适应
- **代码块行号**：为 markdown 渲染的代码块添加行号 gutter，支持通过设置开关

## Capabilities

### New Capabilities
- `resizable-sidebar`: 侧边栏拖拽调整宽度功能，包括拖拽手柄、宽度持久化和边界限制
- `unified-header-bar`: 合并 ContentHeader 与 ChatTabsBar 为统一的顶部栏，恢复多 Tab 横排
- `responsive-layout`: 响应式布局系统，包括断点定义、NavRail 折叠、消息区自适应宽度
- `floating-subagent-panel`: SubAgentMonitor 改为浮动/抽屉式面板
- `code-line-numbers`: 代码块行号渲染

### Modified Capabilities
- `tanstack-virtual-chat`: 消息区 maxWidth 从硬编码改为响应式计算，影响虚拟列表项宽度估算

## Impact

- **组件文件**：`AppLayout.tsx`（删除 ContentHeader）、`SessionList.tsx`（添加 resize 逻辑）、`ChatTabsBar.tsx`（完全重写为横排 Tab）、`SubAgentMonitor.tsx`（改为 overlay）、`MessageRenderer.tsx`（maxWidth 响应式）、`NavRail.tsx`（折叠模式）、`MarkdownContent.tsx`（行号渲染）
- **Store**：`useUIStore` 新增 `sidebarWidth`、`navRailCollapsed` 状态字段
- **CSS**：`index.css` 新增响应式断点变量和行号样式
- **测试**：`ChatTabsBar.test.tsx` 需要配合新的 Tab 栏 UI 重写
