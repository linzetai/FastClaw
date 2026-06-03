## 1. CSS 基础设施

- [ ] 1.1 在 `index.css` 中新增布局 CSS 变量（`--bg-shell`、`--bg-card`、`--bg-user-msg`、`--bg-input-border`、`--border`、`--border-subtle`、`--header-h`、`--sidebar-w`、`--panel-w`、`--card-r` 等），light 和 dark 模式各一套，值映射到现有主题变量
- [ ] 1.2 确保 7 种 accent preset 切换时新变量颜色正确跟随，不出现颜色断裂

## 2. 布局骨架（AppShell）

- [ ] 2.1 创建 `src/components/shell/AppShell.tsx`，实现 AppHeader + AppSidebar + ContentBlock 的四区域骨架布局（flex column + flex row），先用占位色块验证结构
- [ ] 2.2 修改 `AppLayout.tsx`，在 onboarding 完成后的主内容分支中调用 `AppShell` 替代当前的 `NavRail + SessionList + MessageStream` 布局
- [ ] 2.3 创建 `src/components/shell/ContentBlock.tsx`，实现 ChatPane + WorkspacePanel 的统一卡片容器，支持 WorkspacePanel 开关时的圆角动态切换
- [ ] 2.4 在 ChatPane 中实现空状态分支：无消息时渲染 `WelcomeView`（居中标题 + ChatInputBar + 快捷建议卡片），有消息时渲染 `MessageStream`；空状态不显示 WorkspacePanel

## 3. AppHeader

- [ ] 3.1 创建 `src/components/shell/AppHeader.tsx`，实现 44px 高的 Header 栏：窗口控制区（Linux 下红黄绿圆点 / 非 Tauri 隐藏）、导航工具栏（Sidebar toggle、Panel toggle、前进、后退）
- [ ] 3.2 实现 Header 中间标题区：显示当前会话标题 + 项目名称 + 更多按钮（`···`）
- [ ] 3.3 实现 Header 右侧功能区：主题切换按钮、Git 统计（`+0 -0` 灰显预留）、Commit 按钮（disabled + tooltip）、布局切换按钮组（单栏/分栏）
- [ ] 3.4 将 Header 的拖拽区域（`-webkit-app-region: drag`）和按钮区域（`no-drag`）正确设置，确保窗口拖拽和按钮点击互不干扰

## 4. AppSidebar

- [ ] 4.1 创建 `src/components/shell/AppSidebar.tsx`，实现三区域结构：顶部操作区、中间滚动列表、底部 Settings
- [ ] 4.2 实现顶部操作按钮：New chat（调用创建会话）、Search（展开搜索）、Plugins（ComingSoon）、Automations（ComingSoon），每项带 SVG 图标
- [ ] 4.3 实现中间列表区的分组渲染："Pinned"（固定会话）、"Projects"（按 workDir 分组的项目列表）、"Chats"（普通会话），每组有标题 + 列表项
- [ ] 4.4 实现会话列表项（SessionItem）：图标 + 标题（单行截断）+ 时间标注，hover/active 状态样式，点击切换会话
- [ ] 4.5 实现 Sidebar 折叠/展开动画（`width: 0` + `overflow: hidden`，由 AppHeader toggle 控制）
- [ ] 4.6 实现 Sidebar 拖拽调整宽度：右边缘 4px 热区，min 180px / max 400px / default 210px，双击重置，宽度持久化到 `useUIStore`

## 5. WorkspacePanel

- [ ] 5.1 创建 `src/components/shell/workspace-tabs.ts`，实现标签注册系统：`registerWorkspaceTab({ id, label, icon, component, footerComponent? })` 与 `getWorkspaceTabs()` / 当前激活 tab 状态
- [ ] 5.2 创建 `src/components/shell/WorkspacePanel.tsx`，实现面板容器：标签栏 + 可滚动内容区 + 当前标签的 footer 插槽
- [ ] 5.3 实现标签栏：各已注册标签的图标按钮、"Review" 为 v1 默认激活、"+" 按钮（预留）、右侧面板操作按钮（最大化、切换面板）
- [ ] 5.4 注册 Review 标签并创建 `ReviewTabContent.tsx`；创建 `review-store.ts`（zustand），定义 mock 数据接口（`FileChange`、`DiffHunk`、`StagingState`），初始化 4-5 个模拟文件变更
- [ ] 5.5 实现 Review 标签内文件变更列表：Staged/Unstaged 分组、文件名 + 增删统计、文件选择高亮
- [ ] 5.6 创建 `src/components/shell/ReviewDiff.tsx`，实现 inline diff 渲染：行号、增/删/上下文行颜色、折叠未修改行（"N unmodified lines"）
- [ ] 5.7 实现 Review 标签底部操作栏："Revert all" + "Stage all" 按钮，mock 模式下切换 staged/unstaged 状态
- [ ] 5.8 实现 WorkspacePanel 显隐逻辑：新会话默认隐藏；agent 文件变更时自动打开并切到 Review；AppHeader/快捷键显式切换；可选右边缘 hover peek（可配置禁用）

## 6. ChatInputBar

- [ ] 6.1 创建 `src/components/shell/ChatInputBar.tsx`，实现新输入框容器：1.5px 边框、12px 圆角、焦点光晕效果
- [ ] 6.2 将现有 `MentionInput` 嵌入新输入框，保持 @ 提及、/ 命令、多行扩展等功能
- [ ] 6.3 实现内联工具栏：附加按钮（+）、权限选择器（placeholder）、刷新按钮、模型选择器（复用现有 `ModelSelector`）、计算等级（placeholder）、附件按钮、发送按钮
- [ ] 6.4 实现输入框下方的 metadata 行："Work locally" chip + "main" branch chip（均为 placeholder）
- [ ] 6.5 创建 `src/components/shell/WelcomeView.tsx`：大标题 "What should we build in {project-name}?"、居中 ChatInputBar、快捷操作建议卡片

## 7. 消息流适配

- [ ] 7.1 调整 `MessageStream` 内的消息样式以匹配原型：用户消息右对齐 + 圆角气泡（`--bg-user-msg`），AI 消息左对齐无气泡
- [ ] 7.2 实现消息流内的文件变更卡片（`FileChangesCard`）：文件数 + 增删统计 + Undo 按钮 + 文件行列表，点击文件行联动 WorkspacePanel Review 标签
- [ ] 7.3 适配 "33 previous messages ›" 折叠提示样式

## 8. Store 与状态管理

- [ ] 8.1 更新 `ui-store.ts`：新增 `workspacePanelOpen` / `workspacePanelWidth` / `workspaceActiveTab` / `workspaceHoverPeekEnabled` 状态，移除 `activeNav`（NavRail 导航改为 Sidebar 内部状态）
- [ ] 8.2 确保 `sidebarWidth` 的默认值从 240px 更新为 210px，`setSidebarWidth` 和 `resetSidebarWidth` 同步更新

## 9. 清理与集成

- [ ] 9.1 在 `AppLayout.tsx` 中移除 NavRail 的渲染和 `activeNav` 的路由逻辑，全部由 AppShell 接管
- [ ] 9.2 标记 `NavRail.tsx`、旧 `TitleBar.tsx`、`ChatTabsBar.tsx` 为 deprecated（添加注释），确认无其他地方直接引用
- [ ] 9.3 确保 Settings / Connections / Tasks 等子页面在新布局中仍可正常访问（通过 Sidebar 按钮路由）
- [ ] 9.4 使用 Tauri MCP 截图验证 light 和 dark 模式下的完整布局（含空状态 WelcomeView 与 WorkspacePanel 打开态），对比原型确认像素级一致性
