## Context

XiaoLin 当前前端布局采用三栏架构：

```
┌──────────────────────────────────────────────────┐
│ TitleBar (36px, drag region, window controls)    │
├──┬────────────┬──────────────────────────────────┤
│48│ SessionList │ MessageStream (chat + footer)    │
│px│  (240px)    │                                  │
│  │ collapsible │                                  │
│Na│             │                                  │
│vR│             │                                  │
│ai│             │                                  │
│l │             │                                  │
└──┴────────────┴──────────────────────────────────┘
```

组件树：`AppLayout → TitleBar + NavRail + SessionList + MessageStream`

主题系统：双层（`data-theme` + `data-accent`），CSS 变量通过 `index.css` 定义，已支持 light/dark + 7 种 accent 预设。

状态管理：`ui-store.ts` 管理 `sidebarCollapsed / sidebarWidth / activeNav / layoutTier`。

目标是改造为四区域布局（参照 `docs/prototype-codex-layout.html`）：

```
┌───────────────────────────────────────────────────────────────┐
│ AppHeader (44px) - controls / title / stats / commit / layout │
├──────────┬───────────────────────────────┬────────────────────┤
│ AppSide- │ ChatPane                      │ WorkspacePanel     │
│ bar      │ (flex-1)                      │ (360px)            │
│ (210px)  │                               │                    │
│          │  [empty] WelcomeView          │ tabs: Review/+     │
│ New chat │  OR MessageStream             │ tab content area   │
│ Search   │  user msg (right-align)       │ (Review: diff etc) │
│ Plugins  │  AI response                 │ optional footer    │
│ Automats │  file changes card           │                    │
│ ──────── │  ┌─────────────────────────┐  │                    │
│ Pinned   │  │ ChatInputBar           │  │                    │
│ Projects │  │ perms/model/level/send  │  │                    │
│ Chats    │  │ local · branch          │  │                    │
│ Settings │  └─────────────────────────┘  │                    │
└──────────┴───────────────────────────────┴────────────────────┘
```

## Goals / Non-Goals

**Goals:**

- 前端布局 1:1 还原原型设计，包括间距、字体大小、配色、圆角等细节
- 保持现有功能完整性：会话管理、消息流、模型切换、附件等
- 复用现有组件的内部逻辑（`MessageStream` 的消息渲染、`MentionInput` 的输入处理等），只重构外层容器
- 适配现有主题系统（light/dark + accent presets），新增的 CSS 变量遵循已有命名规范
- WorkspacePanel 采用可扩展的多标签架构；v1 仅实现 Review 标签（mock 数据），预留 Terminal / Browser / Files / Summary 由后续 change 注册
- 新会话空状态显示居中 WelcomeView，不显示 WorkspacePanel

**Non-Goals:**

- 不实现 Git 后端功能（diff / stage / commit 的真实操作）
- 不实现 Terminal、Browser、Files、Summary 标签的真实内容（由 terminal-panel 等后续 change 注册）
- 不引入 Project 数据模型（后端 project entity）
- 不实现 Automations / Plugins 页面的真实功能（保留入口，点击显示 ComingSoon）
- 不做移动端 / 响应式小屏适配（保持现有 compact tier 的基本处理）
- 不替换 CSS 技术栈（继续使用 Tailwind CSS v4 + CSS 变量）

## Decisions

### Decision 1: 整体重构而非渐进改造

**选择**：以原型为蓝本全新编写 `AppShell.tsx`，取代 `AppLayout.tsx` 的布局逻辑。

**替代方案**：在现有 `AppLayout` 上渐进修改——保留 NavRail，右侧加 Panel。

**理由**：原型的布局架构（Header-top + Sidebar-left + Content-block 内含 Chat+Workspace）与当前结构差异过大。NavRail 是 48px 图标栏，新 Sidebar 是 210px 全功能面板，两者的交互模型完全不同。渐进改造会产生大量条件分支和兼容代码，最终复杂度更高。新建组件文件可以与旧代码并存，通过一个顶层 flag 切换，方便回退。

### Decision 2: ContentBlock 作为统一卡片容器

**选择**：ChatPane 和 WorkspacePanel 放在同一个 `div.content-block` 内，共享 `bg-card` 背景和 `border-radius`，中间用 `border-left` 分隔。

**替代方案**：ChatPane 和 WorkspacePanel 各自独立卡片。

**理由**：原型中 ContentBlock 是一个整体的白色圆角卡片（左上角圆角 12px，右侧无圆角贴边），Workspace Panel 的左边框是 `border-left` 而非独立卡片的边框。这种设计让 Chat 和 Workspace 视觉上形成一个整体工作区，与 shell 背景形成层次对比。

### Decision 3: 新增 CSS 变量映射到现有主题系统

**选择**：在 `index.css` 中新增一组语义变量（如 `--bg-shell`、`--bg-card`、`--bg-user-msg`），值映射到已有主题变量。

```css
[data-theme="light"] {
  --bg-shell: var(--bg-secondary);   /* 原型: #e8eaef */
  --bg-card: var(--bg-primary);      /* 原型: #ffffff */
  --bg-user-msg: var(--bg-tertiary); /* 原型: #f0f1f4 */
}
```

**理由**：原型使用了自定义变量名（`--bg-shell` 等），直接硬编码会脱离主题系统。通过映射层，accent preset 切换时这些变量也会自动跟随变化。同时保留了原型变量名的语义清晰度。

### Decision 4: WorkspacePanel 多标签注册架构

**选择**：WorkspacePanel 作为容器，通过标签注册表（`registerWorkspaceTab({ id, label, icon, component, footerComponent? })`）渲染各标签内容。layout-overhaul 仅注册 Review 标签；terminal-panel change 注册 Terminal，git-integration 提供 Review 的真实数据源等。

**替代方案**：在 WorkspacePanel 内硬编码所有标签的 switch/case。

**理由**：面板是多用途容器，各标签由不同 change 拥有。注册系统避免 layout-overhaul 耦合所有未来标签的实现，且与 OpenSpec 中 terminal-panel、git-integration 等 change 的分工一致。Review 标签 v1 仍使用 `review-store.ts`（zustand）管理 mock 数据，子组件为 `ReviewTabContent`、`FileChangeList`、`InlineDiff`、`ReviewFooter`。

### Decision 5: Sidebar 导航替代 NavRail

**选择**：移除 NavRail 组件，其导航功能（chat / workspace / tasks / files / connections / settings）通过 Sidebar 顶部按钮和底部 Settings 实现。"chat" 是默认视图，其他视图通过 Sidebar 按钮切换为全屏内容（类似当前行为）。

**替代方案**：保留 NavRail 作为极窄的图标条。

**理由**：新设计中没有独立的 NavRail，侧边栏本身承担了所有导航功能。保留 NavRail 会与 Sidebar 功能重复。Sidebar 的 Plugins / Automations 按钮可以路由到对应的全屏页面，与当前 NavRail 的 `activeNav` 逻辑一致。

### Decision 6: Header 中 Commit 按钮和 Git Stats 预留但灰显

**选择**：Header 渲染 Commit 按钮和 `+N -N` 统计区域，但使用 `disabled` 状态 + tooltip "Git 集成即将推出"。

**理由**：保持与原型的视觉一致性，让用户看到完整的 UI 形态。禁用状态不会产生误导，且后续 Git 集成时只需移除 disabled 属性。

### Decision 7: 空状态居中 WelcomeView，隐藏 WorkspacePanel

**选择**：当活跃会话无消息（或没有选中会话）时，ChatPane 渲染 `WelcomeView`（大标题 + 居中 ChatInputBar + 快捷建议卡片），不渲染 WorkspacePanel，ContentBlock 使用四角圆角 `var(--card-r)`。

**替代方案**：空状态仍显示 MessageStream 的空列表 + 底部输入框；WorkspacePanel 默认打开。

**理由**：原型中新会话是聚焦的「开始构建」体验，右侧面板在无工作内容时会造成干扰。面板仅在用户显式打开、agent 产生文件变更等有意义时出现。WelcomeView 复用 ChatInputBar 组件，避免两套输入 UI。

### Decision 8: 组件文件组织

```
src/components/
├── shell/                    # 新的布局组件
│   ├── AppShell.tsx          # 顶层布局（替代 AppLayout 的核心布局）
│   ├── AppHeader.tsx         # Header 栏
│   ├── AppSidebar.tsx        # 左侧边栏
│   ├── ContentBlock.tsx      # Chat + Workspace 容器
│   ├── WorkspacePanel.tsx    # 右侧多用途工作区面板
│   ├── workspace-tabs.ts     # 标签注册表与类型
│   ├── ReviewTabContent.tsx  # Review 标签内容
│   ├── ReviewDiff.tsx        # diff 渲染组件
│   ├── WelcomeView.tsx       # 空状态欢迎视图
│   └── ChatInputBar.tsx      # 输入栏（重构自 StreamFooter）
├── layout/                   # 保留，旧组件标记 deprecated
│   ├── AppLayout.tsx         # 改为包装 AppShell
│   ├── TitleBar.tsx          # deprecated → AppHeader
│   └── NavRail.tsx           # deprecated → AppSidebar
```

**理由**：新组件放在 `shell/` 目录下，与旧组件共存。`AppLayout.tsx` 作为入口保留，内部调用 `AppShell`。这样 `App.tsx` 不需要修改，迁移对外层透明。

## Risks / Trade-offs

**[Risk] 大量组件重写导致回归 Bug** → 迁移分阶段进行：Phase 1 布局骨架 + 静态展示，Phase 2 接入交互逻辑，Phase 3 细节打磨。每阶段用 Tauri MCP 截图验证。

**[Risk] 原型的像素细节在不同 DPI / 平台上不一致** → 使用 CSS 变量控制所有尺寸值，不硬编码绝对像素。对关键间距定义语义 token（`--header-h: 44px`、`--sidebar-w: 210px`、`--panel-w: 360px`）。

**[Risk] Review 标签的 mock 数据与未来 Git API 结构不匹配** → `review-store` 中定义清晰的 TypeScript 接口（`FileChange`、`DiffHunk`、`StagingState`），设计时参考 `git2` crate 的输出结构。

**[Risk] 移除 NavRail 后，非 chat 页面（tasks / connections / settings）的入口不明显** → Sidebar 顶部保留 Plugins / Automations 等入口按钮，Settings 在底部。Tasks 和 Connections 通过 Sidebar 的按钮进入，行为与当前一致，只是视觉形式从图标变为文字按钮。

**[Trade-off] ContentBlock 圆角动态切换** → WorkspacePanel 打开时 `border-radius: var(--card-r) 0 0 var(--card-r)`（右侧贴边无圆角）；关闭或空状态时四角 `var(--card-r)`。需用 CSS 或 data-attribute 根据 `workspacePanelOpen` 和会话消息数动态调整。

**[Trade-off] 标签注册 vs 首屏 bundle** → 注册表在运行时收集标签，v1 仅 Review 一条，后续 change 在模块加载时注册，避免 layout-overhaul 依赖未实现的 Terminal/Browser 代码。
