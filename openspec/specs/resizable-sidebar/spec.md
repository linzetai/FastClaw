## ADDED Requirements

### Requirement: Sidebar drag handle
SessionList 右侧边缘 SHALL 显示一个可拖拽的手柄，用户拖动时实时调整侧边栏宽度。

#### Scenario: Drag to resize
- **WHEN** 用户在 SessionList 右边缘按下鼠标并水平拖动
- **THEN** SessionList 宽度跟随鼠标实时变化，松开后保持新宽度

#### Scenario: Cursor hint
- **WHEN** 鼠标悬浮在 SessionList 右边缘 4px 区域内
- **THEN** 鼠标光标变为 `col-resize`，手柄区域显示可见的分隔线指示

### Requirement: Width boundaries
侧边栏宽度 SHALL 限制在 180px 到 400px 之间，默认宽度 240px。

#### Scenario: Minimum width enforcement
- **WHEN** 用户拖拽侧边栏宽度到 180px 以下
- **THEN** 宽度停留在 180px，不继续缩小

#### Scenario: Maximum width enforcement
- **WHEN** 用户拖拽侧边栏宽度超过 400px
- **THEN** 宽度停留在 400px，不继续增大

### Requirement: Width persistence
侧边栏宽度 SHALL 通过 `useUIStore` 持久化，重启应用后恢复上次的宽度。

#### Scenario: Restart preserves width
- **WHEN** 用户将侧边栏调整到 300px 后关闭应用
- **THEN** 重新打开应用后侧边栏宽度为 300px

### Requirement: Collapse behavior improvement
侧边栏折叠 SHALL 使用 `width: 0` + `overflow: hidden` 实现，不使用 `opacity: 0`。

#### Scenario: Collapsed state
- **WHEN** 用户点击折叠按钮
- **THEN** 侧边栏动画收缩到 0px 宽度，不渲染内部的展开按钮（展开按钮移到 ContentHeader/ChatTabsBar）

### Requirement: Double-click to reset
用户双击拖拽手柄 SHALL 重置侧边栏宽度为默认值 240px。

#### Scenario: Reset to default
- **WHEN** 用户双击 SessionList 右侧的拖拽手柄
- **THEN** 侧边栏宽度平滑过渡到 240px
