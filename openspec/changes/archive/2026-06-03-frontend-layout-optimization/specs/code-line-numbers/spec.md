## ADDED Requirements

### Requirement: Line number gutter
Markdown 渲染的代码块 SHALL 在左侧显示行号 gutter。

#### Scenario: Code block with line numbers
- **WHEN** 消息中包含一段 10 行的代码块
- **THEN** 代码块左侧显示 1-10 的行号，行号列宽固定 40px，颜色为 `var(--fill-quaternary)`

#### Scenario: Line number alignment
- **WHEN** 代码块超过 99 行
- **THEN** 行号右对齐，列宽自动适应位数（如 3 位数时约 48px）

### Requirement: Line numbers not copyable
行号 SHALL 使用 CSS `user-select: none` 确保复制代码时不包含行号。

#### Scenario: Copy without line numbers
- **WHEN** 用户选中代码块内容并复制
- **THEN** 剪贴板中只包含代码文本，不包含行号

### Requirement: Toggle setting
行号显示 SHALL 可通过 `useConfigStore.display.showLineNumbers` 开关控制，默认开启。

#### Scenario: Disable line numbers
- **WHEN** 用户在设置中关闭"显示代码行号"
- **THEN** 所有代码块不显示行号，代码区域占满原行号 gutter 空间

#### Scenario: Default enabled
- **WHEN** 用户首次使用应用
- **THEN** 代码块默认显示行号

### Requirement: Streaming code blocks
正在 streaming 的代码块 SHALL 也显示行号，行号随新行实时增加。

#### Scenario: Streaming line numbers
- **WHEN** AI 正在 streaming 输出一段代码
- **THEN** 行号随每一行的出现实时递增
