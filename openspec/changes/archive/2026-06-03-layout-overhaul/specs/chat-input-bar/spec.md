## CHANGED Requirements (from current StreamFooter → new ChatInputBar)

当前输入区域由 `StreamFooter.tsx` 实现，需要重构为独立的 `ChatInputBar.tsx` 组件，视觉和交互对齐原型 `docs/prototype-codex-layout.html` 的 `.input-wrap` + `.input-box` + `.input-below` 结构。

### Requirement: Input box container

输入框 SHALL 使用独立的 `input-box` 容器，样式如下：

| 属性 | 当前值 | 目标值（原型对齐） |
|------|--------|-------------------|
| border-radius | 18px | **12px** |
| border | 1.5px solid var(--separator) | **1.5px solid var(--bg-input-border)** |
| background | var(--bg-surface) | **var(--bg-card)** |
| 外层 padding | px-4 pb-3 pt-2 | **padding: 6px 28px 12px** |

#### Scenario: Default state
- **WHEN** 输入框未获得焦点
- **THEN** 显示 `var(--bg-input-border)` 颜色的 1.5px 边框，`var(--bg-card)` 背景

#### Scenario: Focused state
- **WHEN** 输入框内任意元素获得焦点（`:focus-within`）
- **THEN** 边框变为 `var(--accent)` 颜色
- **AND** 出现 `box-shadow: 0 0 0 3px rgba(accent, 0.08)` 光晕
- **AND** 过渡动画 `transition: border-color 0.15s, box-shadow 0.15s`

### Requirement: Textarea area

MentionInput 组件 SHALL 复用现有实现（@ 提及、/ 命令、多行扩展、粘贴文件等），但样式需调整：

| 属性 | 当前值 | 目标值 |
|------|--------|--------|
| textarea padding | 由 MentionInput 内部控制 | **11px 14px 6px**（与原型 `.input-ta` 对齐）|
| font-size | 继承 | **13.5px** |
| line-height | 继承 | **1.45** |
| min-height | 继承 | **18px** |
| placeholder | "描述任务，或输入 @ 引用文件、/ 命令..." | 保持中文或可配置 |

#### Scenario: Text input
- **WHEN** 用户在输入区域输入文字
- **THEN** 文本区域自动扩展高度（保持现有 autoGrow 逻辑）
- **AND** 保持现有 MentionInput 的 @ 提及、/ 命令、粘贴图片等功能

### Requirement: Inline toolbar (ib-bar)

文本输入区域下方 SHALL 显示水平排列的工具栏（对齐原型 `.input-bar`）。工具栏分为左侧区域（`ib-left`, flex:1）和右侧区域（`ib-right`）。

**布局**：`display: flex; align-items: center; padding: 3px 10px 8px;`

**左侧区域 (ib-left)** 插槽顺序（与原型严格对齐）：

```
[1] + 附加按钮           — ib-chip, Plus icon (13px), 点击打开文件/context 附加
[2] 🔒 权限选择器 ▾      — ib-chip, Lock icon + "Default permissions" + chevron
                           当前无对应实现，显示 placeholder (tooltip "即将推出")
[3] ↻ 刷新按钮            — ib-chip, RefreshCw icon, 预留
[4] ● 模型选择器 ▾        — ib-chip, 复用现有 ModelSelector，保持彩色圆点 + 模型名 + chevron
[5] ⚡ 计算等级 ▾          — ib-chip, "Extra High" + chevron
                           当前无对应实现，显示 placeholder (tooltip "即将推出")
```

**右侧区域 (ib-right)**：

```
[6] 📎 附件按钮           — ib-icon (26x26, 圆角6px), Paperclip icon (15px, strokeWidth 1.6)
[7] ⬆️ 发送按钮          — send-btn (28x28, 圆形), accent 背景, 白色 ArrowUp icon
                           disabled 时 opacity 0.3, hover 时 scale 1.06 + box-shadow
```

**从当前 toolbar 移除的元素**：
- ❌ `FolderOpen` 工作目录按钮 → 移动到 below-input 的 "Work locally" chip
- ❌ `ModeToggle` (Agent/Plan) → 移动到 below-input 或保留为可选
- ❌ `ContextRing` → 移动到 Header 或保留在输入区但不在 ib-bar 中

#### Scenario: ib-chip 通用样式
- **GIVEN** 任何 ib-chip 元素
- **THEN** 应用样式：`padding: 3px 7px; border-radius: 5px; font-size: 11px; font-weight: 500; color: var(--text-3);`
- **AND** hover 时 `background: var(--bg-hover); color: var(--text-2);`
- **AND** 内含 SVG 图标 `width: 13px; height: 13px; stroke-width: 1.6`
- **AND** chevron 下拉标记 `font-size: 8px; opacity: 0.5; margin-left: 1px`

#### Scenario: ib-icon 通用样式
- **GIVEN** 任何 ib-icon 元素
- **THEN** 应用样式：`width: 26px; height: 26px; border-radius: 6px; background: transparent; color: var(--text-4);`
- **AND** hover 时 `color: var(--text-3); background: var(--bg-hover);`
- **AND** 内含 SVG 图标 `width: 15px; height: 15px; stroke-width: 1.6`

#### Scenario: Send button states
- **WHEN** 输入框有内容或有附件
- **THEN** 发送按钮 opacity 为 1，hover 时 `transform: scale(1.06); box-shadow: 0 2px 8px rgba(accent, 0.2)`
- **WHEN** 输入框为空且无附件
- **THEN** 发送按钮 opacity 为 0.3，不可点击
- **WHEN** 正在流式输出中
- **THEN** 发送按钮变为红色圆形停止按钮（保持现有 stopStream 逻辑）

#### Scenario: Model selector
- **WHEN** 用户点击模型选择器 chip
- **THEN** 弹出模型列表下拉菜单（复用现有 ModelSelector 逻辑和样式）

#### Scenario: Permission selector (placeholder)
- **WHEN** 用户点击权限选择器 chip
- **THEN** 显示 tooltip "权限管理即将推出"

#### Scenario: Compute level (placeholder)
- **WHEN** 用户点击计算等级 chip
- **THEN** 显示 tooltip "计算等级设置即将推出"

### Requirement: Below-input metadata row

输入框下方 SHALL 显示执行环境和分支信息行（对齐原型 `.input-below`）。

**布局**：`display: flex; align-items: center; gap: 2px; padding: 4px 4px 0;`

**插槽**：

```
[1] 💻 Work locally ▾   — below-chip, Monitor icon + "Work locally" + chevron
                          点击弹出工作目录选择器（复用现有 FolderOpen 对话框逻辑）
                          如已设置 workDir，显示缩短路径替代 "Work locally"
[2] 🔀 main ▾           — below-chip, GitBranch icon + branch name + chevron
                          当前无分支数据，显示 "main"（预留）
```

#### Scenario: below-chip 通用样式
- **GIVEN** 任何 below-chip 元素
- **THEN** 应用样式：`padding: 3px 7px; border-radius: 5px; font-size: 11px; color: var(--text-4);`
- **AND** hover 时 `background: var(--bg-hover); color: var(--text-3);`
- **AND** 内含 SVG 图标 `width: 12px; height: 12px; stroke-width: 1.8`

#### Scenario: Work locally chip
- **WHEN** 用户点击 "Work locally" chip
- **THEN** 打开目录选择器对话框（复用现有 Tauri `open` 对话框逻辑）
- **AND** 选择后更新 workDir 状态，chip 显示缩短的目录路径

#### Scenario: Branch chip (placeholder)
- **WHEN** 用户点击分支 chip
- **THEN** 暂无操作（预留 Git 集成）

### Requirement: Preserve existing features

以下现有功能 SHALL 在重构中保留：

1. **MentionInput** — @ 提及、/ 命令、fuzzy 搜索弹出、粘贴文件、剪贴板图片读取
2. **ModelSelector** — 模型列表、彩色圆点、切换逻辑
3. **AttachedFiles** — 文件预览条、图片预览、删除按钮
4. **MessageQueue** — QueueIndicator + QueuePanel 展开
5. **PlanMode** — Plan 模式指示条（可移到 below-input 或保留在 input-box 内部）
6. **PendingQuestion / ApprovalCard** — 工具问答和审批卡片（在 input-box 上方显示）
7. **Stop button** — 流式输出时的红色停止按钮替换发送按钮
8. **Drag-and-drop** — 全屏拖放区域

### Requirement: ModeToggle position

Agent/Plan 模式切换 SHALL 保留，但位置需要调整：
- 默认在 below-input row 的右侧显示（与 "Work locally" 和 "main" 同行）
- 或者作为 ib-chip 放在 inline toolbar 的计算等级右侧
- 具体位置在实现时根据空间决定，优先保持原型风格的简洁

### Requirement: File changes card in chat

ChatPane 中 AI 回复后 SHALL 可以显示 "文件变更卡片"（对齐原型 `.fc` 系列样式）：

| 元素 | 样式 |
|------|------|
| 卡片容器 | `margin: 10px 0 16px; border: 1px solid var(--border); border-radius: 12px; overflow: hidden;` |
| 顶部栏 | `padding: 8px 14px; font-size: 12px; font-weight: 500;` 显示 "N files changed" + stats + Undo 按钮 |
| 文件行 | `padding: 6px 14px; font-size: 12px; font-family: var(--font-mono);` 显示文件名 + 增删统计 + 橙色点 + 展开箭头 |
| 文件行 hover | `background: var(--bg-hover);` |

#### Scenario: File changes card rendering
- **WHEN** AI 完成文件修改操作
- **THEN** 在消息流中显示文件变更卡片
- **AND** 卡片顶部显示 "N files changed +X -Y"，右侧有 Undo 按钮
- **AND** 每个文件行显示文件名、增删统计（绿/红着色）、橙色变更点、展开箭头

#### Scenario: File changes card interaction
- **WHEN** 用户点击文件行
- **THEN** 在 WorkspacePanel 的 Review 标签中展开该文件的 diff
