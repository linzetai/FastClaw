## Why

当前 XiaoLin 采用传统 IM 风格布局（NavRail + SessionList + MessageStream 三栏），与项目的 AI Coding Agent 定位不匹配。我们已根据行业最佳实践完成了一版高保真原型（`docs/prototype-codex-layout.html`），需要将现有前端布局改造为四区域布局：Header + Sidebar + ChatPane + WorkspacePanel。这次改造只涉及前端 UI 层的布局和组件结构，不涉及后端 API 或 Rust 侧的变更。后端能力（Git 集成、Project 模型等）将在后续独立的 change 中处理。

## What Changes

- **移除 NavRail**：取消左侧 48px 的图标导航栏，其导航功能合并到新 Sidebar 中
- **重构 Header**：从简单的 TitleBar 改为功能丰富的 Header，包含窗口控制、导航工具栏、会话标题、项目名称、统计信息区（预留 git stats 位置）、Commit 按钮（预留）、布局切换按钮
- **重构 Sidebar**：从 SessionList 改为全功能 Sidebar，包含顶部操作区（New chat / Search / Plugins / Automations）、分组列表（Pinned / Projects / Chats）、底部 Settings 入口
- **新增 WorkspacePanel**：主内容区右侧新增多用途工作区面板，通过标签页系统承载多种内容类型；v1 实现 Review 标签（Git diff、stage/revert，mock 数据），Terminal / Browser / Files / Summary 为后续 change 注册的占位标签
- **空状态布局**：新会话或无消息时，ChatPane 显示居中 WelcomeView，不显示 WorkspacePanel；有消息或显式切换后才显示面板
- **重构 InputBar**：输入框从当前的 StreamFooter 改造为新风格，增加权限选择器、模型选择器、计算等级选择器、附件按钮的内联布局，底部增加执行环境（Work locally）和分支选择器（预留）
- **统一内容区域为 ContentBlock**：ChatPane 和 WorkspacePanel 共享同一个白色卡片背景（`bg-card`），Sidebar 和 Header 使用 shell 背景色（`bg-shell`），形成标志性的视觉层次

## Capabilities

### New Capabilities
- `app-shell-layout`: 整体应用外壳布局——Header + Sidebar + ContentBlock（ChatPane + WorkspacePanel）的四区域架构，包含空状态 WelcomeView、窗口控制、拖拽区域、响应式断点适配
- `app-sidebar`: 全功能侧边栏——顶部操作区、分组列表（Pinned/Projects/Chats）、折叠/展开、底部 Settings；替代当前 NavRail + SessionList
- `app-header`: 新 Header 栏——窗口控制、导航工具、会话标题与项目名、统计信息区、Commit 按钮（预留）、布局切换
- `workspace-panel`: 右侧多用途工作区面板——标签注册系统、标签栏、v1 Review 标签（文件变更列表、内联 diff、Stage/Revert）；预留 Terminal / Browser / Files / Summary 由后续 change 注册
- `chat-input-bar`: 新输入栏——权限/模型/计算等级选择器内联、附件、执行环境与分支选择（预留）

### Modified Capabilities
- `unified-header-bar`: 原 header 的 Tab 横排系统将被新 Header 设计替代，Tab 概念移入 Sidebar 或保留为 Header 内的会话选择器
- `resizable-sidebar`: 拖拽调整宽度的逻辑保留，但容器从 SessionList 变为新 Sidebar，宽度范围和默认值可能调整
- `workspace-grouped-sessions`: 会话分组逻辑保留，但渲染位置从 SessionList 移到新 Sidebar 的 Chats 区域，增加 Pinned 和 Projects 分组

## Impact

- **前端组件**：`AppLayout.tsx`、`TitleBar.tsx`、`NavRail.tsx`、`SessionList.tsx`、`StreamFooter.tsx`、`ChatTabsBar.tsx` 将被重构或替换
- **CSS/Token**：`index.css` 中新增配色变量（`--bg-shell`、`--bg-card`、`--bg-user-msg` 等），可能需要扩展现有主题系统
- **Store**：`ui-store.ts` 需要新增 `workspacePanelOpen` / `workspacePanelWidth` / `workspaceActiveTab` 等状态；移除 NavRail 相关的 `activeNav` 导航（或改为 Sidebar 内的子状态）
- **依赖**：无新增外部依赖，所有组件使用现有 React + Tailwind + Lucide 栈
- **兼容性**：此变更是 **BREAKING**——现有布局组件的公开接口和组件树将完全重构，无法渐进兼容
