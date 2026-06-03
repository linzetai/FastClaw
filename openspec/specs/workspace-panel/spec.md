## ADDED Requirements

### Requirement: WorkspacePanel container
WorkspacePanel SHALL 为 ContentBlock 右侧的多用途固定宽度面板（`--panel-w`，默认 360px），采用多标签页架构。面板包含：标签栏（顶部）、内容区（flex-1, 可滚动）、各标签可选的底部操作栏（footer）。左侧有 `border-left: 1px solid var(--border)` 与 ChatPane 分隔。

#### Scenario: Panel rendering
- **WHEN** WorkspacePanel 处于打开状态
- **THEN** 渲染 360px 宽的面板，紧贴 ContentBlock 右侧
- **AND** 包含标签栏、可滚动内容区、当前激活标签的底部操作栏（如有）

### Requirement: Tab bar
标签栏 SHALL 显示所有已注册的标签页。v1 仅包含 "Review" 标签。标签栏包含：各标签的图标按钮、"+" 按钮（添加标签，预留）、右侧面板控制按钮组（最大化、切换面板位置）。激活的标签使用 `--bg-active` 背景和 `--text-1` 文字色。

#### Scenario: Active tab display
- **WHEN** 打开 WorkspacePanel
- **THEN** "Review" 标签默认激活（`--text-1` 颜色 + `--bg-active` 背景）

#### Scenario: Tab switching
- **WHEN** 用户点击另一个已注册的标签图标
- **THEN** 切换到该标签，内容区渲染对应标签的 component

#### Scenario: Add tab placeholder
- **WHEN** 用户点击 "+" 按钮
- **THEN** 暂无操作（预留功能）

### Requirement: Tab registration system
WorkspacePanel SHALL 通过标签注册系统渲染内容。每个标签声明 `{ id, label, icon, component, footerComponent? }`。标签由各功能 change 注册（如 git-integration 注册 Review、terminal-panel 注册 Terminal）。WorkspacePanel 仅负责渲染当前激活标签的 component 和 footerComponent。

**技术方案**：使用 `workspace-tabs.ts` 模块导出一个 Zustand slice（或独立 store）管理 tab 注册与激活状态。

```typescript
interface WorkspaceTab {
  id: string;           // "review" | "terminal" | "browser" | ...
  label: string;        // 标签栏显示文字
  icon: ComponentType;  // Lucide icon 组件
  component: ComponentType;       // 标签内容组件
  footerComponent?: ComponentType; // 可选底部操作栏
  badge?: number | boolean;       // 通知徽章（数字或圆点）
  order?: number;       // 排列顺序（默认按注册顺序）
}

// 注册 API（在各功能模块的初始化中调用）
registerWorkspaceTab(tab: WorkspaceTab): void;
unregisterWorkspaceTab(id: string): void;

// 状态
activeTabId: string;
registeredTabs: WorkspaceTab[];
setActiveTab(id: string): void;
```

各 change 在对应组件的 `useEffect` 中调用 `registerWorkspaceTab()` 注册自己的标签。WorkspacePanel 读取 `registeredTabs` 渲染标签栏，读取 `activeTabId` 渲染内容区。

#### Scenario: Review tab registered in v1
- **WHEN** layout-overhaul 实现完成
- **THEN** Review 标签通过 `registerWorkspaceTab({ id: "review", label: "Review", ... })` 注册
- **AND** Terminal、Browser、Files、Summary 标签尚未注册（由后续 change 添加）

#### Scenario: Future tab registration
- **WHEN** 后续 change 调用 `registerWorkspaceTab()` 注册 Terminal / Browser / Files / Summary 标签
- **THEN** 标签栏自动显示对应图标按钮（按 order 排序）
- **AND** 点击后切换至该标签内容

#### Scenario: Tab badge notification
- **WHEN** 某标签的 `badge` 字段被更新（如 Review tab 有新的 git 变更）
- **THEN** 标签图标旁显示通知徽章
- **AND** 切换到该标签后徽章清除

### Requirement: Review tab content
Review 标签（v1，使用 git mock 数据）SHALL 显示：文件变更列表（Staged / Unstaged 分组）、选中文件的 inline diff、底部 Stage all / Revert all 操作栏。Git diff 相关行为限定在 Review 标签内，不再占用整个 WorkspacePanel。

#### Scenario: Unstaged files display
- **WHEN** Review 标签激活且存在未暂存的文件变更（mock 数据）
- **THEN** 显示 "Unstaged" 分组，列出所有变更文件名
- **AND** 每个文件名旁显示增删行数统计（绿色 +N / 红色 -N）

#### Scenario: Staged files display
- **WHEN** Review 标签激活且存在已暂存的文件变更（mock 数据）
- **THEN** 显示 "Staged" 分组，列出已暂存文件名及增删统计

#### Scenario: File selection
- **WHEN** 用户在 Review 标签中点击某个文件名
- **THEN** 内容区下方显示该文件的 diff 内容

#### Scenario: Diff line display
- **WHEN** 选中一个有变更的文件
- **THEN** 显示 monospace 字体的 inline diff 内容
- **AND** 新增行（`+`）使用 `--green-line` 背景 + `--green-text` 文字色
- **AND** 删除行（`-`）使用 `--red-line` 背景 + `--red-text` 文字色
- **AND** 上下文行使用 `--text-3` 灰色文字
- **AND** 每行左侧显示行号（10px, 灰色, 右对齐, 32px 宽）

#### Scenario: Collapsed unchanged lines
- **WHEN** diff 中有连续未修改的行
- **THEN** 显示 "N unmodified lines" 的折叠提示，两侧有水平分隔线

#### Scenario: Review footer actions
- **WHEN** Review 标签激活且有文件变更数据
- **THEN** 底部显示 "↺ Revert all" 和 "+ Stage all" 两个按钮
- **AND** "Stage all" 按钮有绿色边框和文字色

#### Scenario: Mock mode interaction
- **WHEN** 用户点击 "Stage all" 或 "Revert all"
- **THEN** 按钮显示短暂的按下反馈动画
- **AND** mock 数据状态更新（文件在 staged/unstaged 之间移动）

### Requirement: WorkspacePanel toggle
WorkspacePanel SHALL 支持通过 AppHeader 中的布局按钮或键盘快捷键打开/关闭。关闭时不渲染 WorkspacePanel，ContentBlock 只包含 ChatPane。

#### Scenario: Toggle WorkspacePanel
- **WHEN** 用户点击 AppHeader 的分栏布局按钮或触发快捷键
- **THEN** WorkspacePanel 在打开和关闭之间切换
- **AND** 切换时有平滑的宽度过渡动画

### Requirement: Auto-show behavior
WorkspacePanel SHALL 默认在新会话（无消息）时隐藏。当 agent 产生文件变更时，面板自动打开并切换到 Review 标签（后续由 Git integration change 联动真实数据）。

#### Scenario: Hidden on new chat
- **WHEN** 活跃会话没有任何消息
- **THEN** WorkspacePanel 不显示
- **AND** ContentBlock 仅渲染 ChatPane（含居中 WelcomeView）

#### Scenario: Auto-open on file changes
- **WHEN** agent 在会话中产生文件变更（mock 或后续 Git 事件）
- **THEN** WorkspacePanel 自动打开
- **AND** 激活 Review 标签

#### Scenario: Explicit toggle overrides auto-hide
- **WHEN** 用户通过 AppHeader 或快捷键显式打开 WorkspacePanel
- **THEN** 即使会话无消息，面板也可显示（用户主动选择）

### Requirement: Panel hover trigger
WorkspacePanel SHALL 支持可选的右侧边缘悬停触发（hover gutter）：鼠标移入 ContentBlock 右边缘窄条区域时短暂预览面板。该行为 MUST 可通过配置禁用。

#### Scenario: Hover peek enabled
- **WHEN** hover trigger 已启用且 WorkspacePanel 处于关闭状态
- **THEN** 鼠标移入右边缘悬停区域时，面板以 peek 模式短暂显示
- **AND** 鼠标移出后面板收起（除非用户已显式 pin 打开）

#### Scenario: Hover peek disabled
- **WHEN** 用户在设置中禁用 hover trigger
- **THEN** 右边缘悬停区域不响应，面板仅通过按钮/快捷键/自动行为控制
