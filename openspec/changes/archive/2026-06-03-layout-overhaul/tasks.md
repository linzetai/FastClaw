## 1. CSS 基础设施

- [x] 1.1 在 `index.css` 中新增布局 CSS 变量（`--bg-shell`、`--bg-card`、`--bg-user-msg`、`--bg-input-border`、`--border`、`--border-subtle`、`--header-h`、`--sidebar-w`、`--panel-w`、`--card-r` 等），light 和 dark 模式各一套，值映射到现有主题变量
- [x] 1.2 确保 7 种 accent preset 切换时新变量颜色正确跟随，不出现颜色断裂

## 2. 布局骨架（AppShell）

- [x] 2.1 创建 `src/components/shell/AppShell.tsx`，实现 AppHeader + AppSidebar + ContentBlock 的四区域骨架布局（flex column + flex row），先用占位色块验证结构
- [x] 2.2 修改 `AppLayout.tsx`，在 onboarding 完成后的主内容分支中调用 `AppShell` 替代当前的 `NavRail + SessionList + MessageStream` 布局
- [x] 2.3 创建 `src/components/shell/ContentBlock.tsx`，实现 ChatPane + WorkspacePanel 的统一卡片容器，支持 WorkspacePanel 开关时的圆角动态切换
- [x] 2.4 在 ChatPane 中实现空状态分支：无消息时渲染 `WelcomeView`（居中标题 + ChatInputBar + 快捷建议卡片），有消息时渲染 `MessageStream`；空状态不显示 WorkspacePanel

## 3. AppHeader

- [x] 3.1 创建 `src/components/shell/AppHeader.tsx`，实现 44px 高的 Header 栏：窗口控制区（Linux 下红黄绿圆点 / 非 Tauri 隐藏）、导航工具栏（Sidebar toggle、Panel toggle、前进、后退）
- [x] 3.2 实现 Header 中间标题区：显示当前会话标题 + 项目名称 + 更多按钮（`···`）
- [x] 3.3 实现 Header 右侧功能区：主题切换按钮、Git 统计（`+0 -0` 灰显预留）、Commit 按钮（disabled + tooltip）、布局切换按钮组（单栏/分栏）
- [x] 3.4 将 Header 的拖拽区域（`-webkit-app-region: drag`）和按钮区域（`no-drag`）正确设置，确保窗口拖拽和按钮点击互不干扰

## 4. AppSidebar

- [x] 4.1 创建 `src/components/shell/AppSidebar.tsx`，实现三区域结构：顶部操作区、中间滚动列表、底部 Settings
- [x] 4.2 实现顶部操作按钮：New chat（调用创建会话）、Search（展开搜索）、Plugins（ComingSoon）、Automations（ComingSoon），每项带 SVG 图标
- [x] 4.3 实现中间列表区的分组渲染："Pinned"（固定会话）、"Projects"（按 workDir 分组的项目列表）、"Chats"（普通会话），每组有标题 + 列表项
- [x] 4.4 实现会话列表项（SessionItem）：图标 + 标题（单行截断）+ 时间标注，hover/active 状态样式，点击切换会话
- [x] 4.5 实现 Sidebar 折叠/展开动画（`width: 0` + `overflow: hidden`，由 AppHeader toggle 控制）
- [x] 4.6 实现 Sidebar 拖拽调整宽度：右边缘 4px 热区，min 180px / max 400px / default 210px，双击重置，宽度持久化到 `useUIStore`

## 5. WorkspacePanel

- [x] 5.1 创建 `src/components/shell/workspace-tabs.ts`，实现标签注册系统：`registerWorkspaceTab({ id, label, icon, component, footerComponent? })` 与 `getWorkspaceTabs()` / 当前激活 tab 状态
- [x] 5.2 创建 `src/components/shell/WorkspacePanel.tsx`，实现面板容器：标签栏 + 可滚动内容区 + 当前标签的 footer 插槽
- [x] 5.3 实现标签栏：各已注册标签的图标按钮、"Review" 为 v1 默认激活、"+" 按钮（预留）、右侧面板操作按钮（最大化、切换面板）
- [x] 5.4 注册 Review 标签并创建 `ReviewTabContent.tsx`；创建 `review-store.ts`（zustand），定义 mock 数据接口（`FileChange`、`DiffHunk`、`StagingState`），初始化 4-5 个模拟文件变更
- [x] 5.5 实现 Review 标签内文件变更列表：Staged/Unstaged 分组、文件名 + 增删统计、文件选择高亮
- [x] 5.6 创建 `src/components/shell/ReviewDiff.tsx`，实现 inline diff 渲染：行号、增/删/上下文行颜色、折叠未修改行（"N unmodified lines"）
- [x] 5.7 实现 Review 标签底部操作栏："Revert all" + "Stage all" 按钮，mock 模式下切换 staged/unstaged 状态
- [x] 5.8 实现 WorkspacePanel 显隐逻辑：新会话默认隐藏；agent 文件变更时自动打开并切到 Review；AppHeader/快捷键显式切换；可选右边缘 hover peek（可配置禁用）

## 6. ChatInputBar 改版

重构现有 `StreamFooter.tsx` → 独立 `ChatInputBar.tsx`，对齐原型 `.input-wrap` + `.input-box` + `.input-below` 结构。

- [x] 6.1 **input-box 容器样式**：创建 `ChatInputBar.tsx`，实现外层容器 `input-wrap`（`padding: 6px 28px 12px`）+ 内层 `input-box`（`border: 1.5px solid var(--bg-input-border); border-radius: 12px; background: var(--bg-card);`），`:focus-within` 时 `border-color: var(--accent); box-shadow: 0 0 0 3px rgba(accent, 0.08)`
- [x] 6.2 **嵌入 MentionInput**：将现有 MentionInput 移入 input-box，textarea 区域 padding 调整为 `11px 14px 6px`，`font-size: 13.5px; line-height: 1.45; min-height: 18px`。保持 @ 提及、/ 命令、粘贴文件等全部功能
- [x] 6.3 **ib-bar 工具栏（左区）**：实现 `display: flex; align-items: center; padding: 3px 10px 8px;` 布局。左侧 ib-left 包含 5 个 chip 插槽：① `+` 附加按钮 ② 🔒 权限选择器 ▾ (placeholder) ③ ↻ 刷新 (预留) ④ ● 模型选择器 ▾ (复用 ModelSelector) ⑤ Extra High ▾ (placeholder)。chip 统一 `padding: 3px 7px; border-radius: 5px; font-size: 11px; font-weight: 500; color: var(--text-3);` hover `bg: var(--bg-hover)`
- [x] 6.4 **ib-bar 工具栏（右区）**：右侧 ib-right 包含 ① 📎 附件按钮 (`ib-icon` 26x26, Paperclip 15px) ② ⬆️ 发送按钮 (`send-btn` 28x28 圆形, accent 背景, 白色 ArrowUp)。流式输出时发送按钮变为红色 Stop 按钮（保持现有逻辑）
- [x] 6.5 **below-input 元数据行**：input-box 下方显示 `padding: 4px 4px 0` 行，包含 ① 💻 "Work locally ▾" chip（点击复用现有目录选择逻辑）② 🔀 "main ▾" chip（预留）③ 右侧 Agent/Plan ModeToggle（移自 ib-bar）。chip 样式 `font-size: 11px; color: var(--text-4);`
- [x] 6.6 **迁移保留功能**：确保 AttachedFiles 预览条、MessageQueue 指示器、PendingQuestion/ApprovalCard、Plan 模式条、拖放区域等全部在新组件中正常工作
- [x] 6.7 **替换 StreamFooter**：在 MessageStream 和 StreamEmptyState 中用 ChatInputBar 替换 StreamFooter 调用，删除或标记 StreamFooter 为 deprecated

## 7. 消息流样式适配

- [x] 7.1 **用户消息气泡**：用户消息改为右对齐气泡样式 — `background: var(--bg-user-msg); border-radius: 14px; padding: 10px 16px; font-size: 14px; line-height: 1.5; margin-left: auto; max-width: 70%; width: fit-content;`
- [x] 7.2 **AI 消息样式**：AI 消息左对齐无气泡 — `font-size: 13.5px; line-height: 1.7; color: var(--text-2);` 段落间 `margin-bottom: 8px;` 行内代码 `font-family: var(--font-mono); font-size: 12px; background: var(--bg-code); padding: 2px 5px; border-radius: 4px; color: var(--purple);`
- [x] 7.3 **文件变更卡片 (FileChangesCard)**：在 AI 回复后显示 — 卡片 `border: 1px solid var(--border); border-radius: 12px;`，顶栏 "N files changed +X -Y" + Undo 按钮，文件行 `font-family: var(--font-mono); font-size: 12px;` + 增删统计着色 + 橙色变更点 + 展开箭头，hover `bg: var(--bg-hover)`，点击联动 WorkspacePanel Review
- [x] 7.4 **折叠提示**：适配 "33 previous messages ›" 样式 — `font-size: 13px; color: var(--text-3);` hover `color: var(--text-2);`
- [x] 7.5 **文件路径链接**：消息中的文件路径用 `.fp` 样式渲染 — `font-family: var(--font-mono); font-size: 12px; background: var(--accent-bg); color: var(--accent); border-radius: 4px;` 前缀 `■` 标记

## 8. Store 与状态管理

- [x] 8.1 更新 `ui-store.ts`：新增 `workspacePanelOpen` / `workspacePanelWidth` / `workspaceActiveTab` / `workspaceHoverPeekEnabled` 状态，移除 `activeNav`（NavRail 导航改为 Sidebar 内部状态）
- [x] 8.2 确保 `sidebarWidth` 的默认值从 240px 更新为 210px，`setSidebarWidth` 和 `resetSidebarWidth` 同步更新

## 9. 清理与集成

- [x] 9.1 在 `AppLayout.tsx` 中移除 NavRail 的渲染和 `activeNav` 的路由逻辑，全部由 AppShell 接管
- [x] 9.2 标记 `NavRail.tsx`、旧 `TitleBar.tsx`、`ChatTabsBar.tsx` 为 deprecated（添加注释），确认无其他地方直接引用
- [x] 9.3 确保 Settings / Connections / Tasks 等子页面在新布局中仍可正常访问（通过 Sidebar 按钮路由）
- [x] 9.4 使用 Tauri MCP 截图验证 light 和 dark 模式下的完整布局（含空状态 WelcomeView 与 WorkspacePanel 打开态），对比原型确认像素级一致性

## 10. 回归修复（原型对照）

以下任务基于原型 `docs/prototype-codex-layout.html` 与当前实现的逐像素对比得出。

### P0 — 布局结构

- [x] 10.1 **ContentBlock 包裹修正**：将 `WorkspacePanel` 从 `AppShell` 中移入 `ContentBlock` 内部，使 ChatPane 和 WorkspacePanel 共享同一个白色卡片容器。`ContentBlock` 的 `flex-direction` 改为 `row`，内含 `children`（ChatPane, flex-1）+ `WorkspacePanel`（固定宽度）
- [x] 10.2 **ContentBlock 间距修正**：移除 `ContentBlock` 的 `margin-right` 和 `margin-bottom`（当前 `margin: "0 var(--gap-shell) var(--gap-shell) 0"`），content-block 直接延伸到窗口右边缘。仅保留底部 `var(--gap-shell)` 间距

### P0 — Sidebar 会话项样式

- [x] 10.3 **会话列表项改为紧凑单行**：移除 28x28 圆形图标容器和双行（标题+副标题）布局，改为原型的紧凑样式：14px SVG 图标（无背景框）+ 标题（13px, 单行截断）+ 时间标注（11px, 灰色, 靠右），整体 `padding: 5px 10px`，`border-radius: 6px`
- [x] 10.4 **分组标题去大写**：移除 `text-transform: uppercase` 和 `letter-spacing: 0.04em`，使用小写文字（首字母大写），`font-weight: 500`，`padding: 12px 10px 4px`

### P1 — Header 补全

- [x] 10.5 **添加 Panels toggle 按钮**：在 sidebar toggle 和 back/forward 之间增加 Panels 切换按钮（矩形+水平分割线图标），点击切换 WorkspacePanel 开关
- [x] 10.6 **添加标题区 "···" 菜单按钮**：在项目名称右侧添加更多选项按钮（`···`），使用 18px 字号，颜色 `--text-4`
- [x] 10.7 **Commit 按钮绿色边框样式**：将 Commit 按钮从纯文本改为绿色描边样式：`border: 1.5px solid var(--green-text)`，文字 `var(--green-text)`，hover `rgba(52,199,89,0.08)`，包含菱形 icon + "Commit" + 下拉箭头
- [x] 10.8 **Git 统计着色**：将 `+N` 显示为 `--green-text` 颜色，`-N` 显示为 `--red-text` 颜色（不再是统一灰色）
- [x] 10.9 **布局切换改为双按钮**：将单个 Columns2 按钮改为两个按钮组：单栏（矩形图标）+ 分栏（竖线分割矩形），当前布局高亮
- [x] 10.10 **添加下拉箭头按钮**：在主题切换按钮右侧添加下拉 chevron 按钮

### P2 — 细节打磨

- [x] 10.11 **Sidebar 操作按钮样式微调**：将顶部操作按钮的 padding 从 `0 10px, height 32px, border-radius 8px` 调整为 `6px 10px, border-radius 6px`，更贴近原型
- [x] 10.12 **Sidebar section header 操作图标**：在 "Chats" 分组标题右侧添加折叠/排序操作图标（`⊟` `≡`）
- [x] 10.13 **WorkspacePanel tab bar 左侧文件图标**：在标签栏最左侧添加文件图标按钮（原型中 tab bar 以此开头）
- [x] 10.14 **Sidebar 滚动条宽度**：确保 sidebar 列表区域的滚动条为 3px（原型指定），当前可能使用全局 4px
