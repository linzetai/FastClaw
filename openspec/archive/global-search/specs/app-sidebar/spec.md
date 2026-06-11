## MODIFIED Requirements

### Requirement: Top action buttons include Search
AppSidebar（或 layout-overhaul 后的等效侧边栏）顶部操作区 SHALL 包含 **Search** 按钮，图标为放大镜，行为为打开全局搜索面板。

#### Scenario: Search button visible
- **WHEN** 侧边栏处于展开状态
- **THEN** 顶部操作区显示 Search 按钮（与 New chat、Plugins、Automations 等并列，顺序对齐 Codex 原型）
- **AND** 按钮含图标与「Search」或「搜索」文案

#### Scenario: Search button opens global panel
- **WHEN** 用户点击 Search 按钮
- **THEN** 调用 `useSearchStore.openPanel()` 打开 `SearchPanel`
- **AND** 不将侧边栏内嵌标题搜索框当作跨会话全文检索入口

### Requirement: Global search keyboard shortcut
系统 SHALL 注册全局快捷键 Cmd/Ctrl+K 打开搜索面板。

#### Scenario: Keyboard shortcut opens panel
- **WHEN** 用户在应用主窗口按下 Cmd+K 或 Ctrl+K
- **THEN** 打开 `SearchPanel` 并聚焦搜索输入
- **AND** 在 `SearchPanel` 已打开时，同一快捷键聚焦输入框而非重复创建面板

#### Scenario: Shortcut does not conflict with in-stream search
- **WHEN** `MessageStream` 内会话级搜索（若存在独立快捷键）与全局 Search 并存
- **THEN** Cmd/Ctrl+K 保留给全局搜索；会话内搜索使用不同快捷键（如 Cmd/Ctrl+F）

### Requirement: Search panel overlay behavior
打开搜索面板时，侧边栏 SHALL 以 overlay 或临时替换方式展示 `SearchPanel`，不破坏 AppShell 其余布局。

#### Scenario: Overlay covers session list region
- **WHEN** `SearchPanel` 打开
- **THEN** 会话分组列表区域被搜索面板覆盖或替换
- **AND** 侧边栏宽度保持 `--sidebar-w`，AppHeader 与主内容区不变

#### Scenario: Restore session list on close
- **WHEN** `SearchPanel` 关闭
- **THEN** 恢复显示 Pinned / Projects / Chats 会话列表

## ADDED Requirements

### Requirement: Distinction from session title filter
侧边栏 MAY 保留 `SessionList` 顶部标题过滤输入框；其与全局 Search 职责分离。

#### Scenario: Title filter remains local
- **WHEN** 用户在会话列表顶部输入框键入文字
- **THEN** 仅 `fuzzyMatch` 过滤当前已加载会话标题
- **AND** 不调用 `search.query`
