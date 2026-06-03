## ADDED Requirements

### Requirement: Input box container
输入框 SHALL 使用 1.5px 边框（`--bg-input-border` 颜色）、12px 圆角的容器。获取焦点时边框变为 `--accent` 颜色，外围有 3px 的半透明光晕。

#### Scenario: Default state
- **WHEN** 输入框未获得焦点
- **THEN** 显示 `--bg-input-border` 颜色的 1.5px 边框，`--bg-card` 背景

#### Scenario: Focused state
- **WHEN** 输入框获得焦点
- **THEN** 边框变为 `--accent` 颜色
- **AND** 外围出现 3px 的 `rgba(accent, 0.08)` box-shadow

### Requirement: Textarea area
输入框顶部 SHALL 为文本输入区域（padding 11px 14px 6px），placeholder 为 "Ask for follow-up changes"，13.5px 字号，min-height 18px，支持多行自动扩展。

#### Scenario: Text input
- **WHEN** 用户在输入区域输入文字
- **THEN** 文本区域自动扩展高度
- **AND** 保持现有 MentionInput 的 @ 提及和 / 命令功能

### Requirement: Inline toolbar
文本输入区域下方 SHALL 显示水平排列的工具栏。工具栏分为左侧区域（ib-left, flex:1）和右侧区域（ib-right）。

**插槽顺序规范**（与原型 `docs/prototype-codex-layout.html` 对齐）：

```
左侧区域 (ib-left):
  [1] + 附加按钮           — 文件/context 附加
  [2] 🔒 权限选择器 ▾      — permission-presets change 激活
  [3] ↻ 刷新按钮            — 预留
  [4] 🟢 模型选择器 ▾      — 复用现有 ModelSelector
  [5] ⚡ 计算等级 ▾        — compute-level change 激活

右侧区域 (ib-right):
  [6] 📎 附件按钮           — 文件上传
  [7] 🎤 语音按钮           — 预留（已有 STT 后端）
  [8] ⬆️ 发送按钮          — 圆形, accent 背景, 白色箭头
```

各插槽在对应 change 未实现时 SHALL 显示 placeholder（点击显示 tooltip "即将推出"）或直接隐藏。已实现的功能（如 ModelSelector）保持现有交互。

#### Scenario: Model selector
- **WHEN** 用户点击模型选择器
- **THEN** 弹出模型列表下拉菜单（复用现有 ModelSelector 逻辑）

#### Scenario: Permission selector (placeholder)
- **WHEN** 用户点击权限选择器
- **THEN** 暂无操作（显示 tooltip "权限管理即将推出"）

#### Scenario: Send button states
- **WHEN** 输入框有内容
- **THEN** 发送按钮 opacity 为 1，hover 时 scale 1.06 + box-shadow
- **WHEN** 输入框为空
- **THEN** 发送按钮 opacity 为 0.3，不可点击

### Requirement: Below-input metadata
输入框下方 SHALL 显示执行环境和分支信息：执行位置（"💻 Work locally ▾"，预留）和分支选择器（"🔀 main ▾"，预留）。

#### Scenario: Metadata display
- **WHEN** 输入框渲染完成
- **THEN** 下方显示 "Work locally" 和 "main" 两个 chip，均为 11px 灰色文字
- **AND** 两个 chip 点击暂无操作（预留功能）

### Requirement: File changes card in chat
ChatPane 中 AI 回复后 SHALL 可以显示 "文件变更卡片"，包含：变更文件数 + 增删统计、文件列表（每行：文件名 + 增删数 + 变更点 + 展开箭头）、Undo 按钮。

#### Scenario: File changes card rendering
- **WHEN** AI 完成文件修改操作
- **THEN** 在消息流中显示文件变更卡片
- **AND** 卡片顶部显示 "N files changed +X -Y"，右侧有 Undo 按钮
- **AND** 每个文件行显示文件名、增删统计、橙色变更点、展开箭头

#### Scenario: File changes card interaction
- **WHEN** 用户点击文件行
- **THEN** 在 WorkspacePanel 的 Review 标签中展开该文件的 diff（如果 WorkspacePanel 已打开）
