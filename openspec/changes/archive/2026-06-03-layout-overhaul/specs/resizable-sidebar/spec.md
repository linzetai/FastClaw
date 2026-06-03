## MODIFIED Requirements

### Requirement: Sidebar drag handle
AppSidebar 右侧边缘 SHALL 显示一个可拖拽的手柄，用户拖动时实时调整侧边栏宽度。容器从原 SessionList 变为 AppSidebar。

#### Scenario: Drag to resize
- **WHEN** 用户在 AppSidebar 右边缘按下鼠标并水平拖动
- **THEN** AppSidebar 宽度跟随鼠标实时变化，松开后保持新宽度

#### Scenario: Cursor hint
- **WHEN** 鼠标悬浮在 AppSidebar 右边缘 4px 区域内
- **THEN** 鼠标光标变为 `col-resize`，手柄区域显示可见的分隔线指示

### Requirement: Width boundaries
侧边栏宽度 SHALL 限制在 180px 到 400px 之间，默认宽度调整为 210px（对齐原型 `--sidebar-w`）。

#### Scenario: Default width
- **WHEN** 首次加载应用（无持久化宽度）
- **THEN** AppSidebar 宽度为 210px

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

### Requirement: Double-click to reset
用户双击拖拽手柄 SHALL 重置侧边栏宽度为默认值 210px。

#### Scenario: Reset to default
- **WHEN** 用户双击 AppSidebar 右侧的拖拽手柄
- **THEN** 侧边栏宽度平滑过渡到 210px

### Requirement: Collapse behavior improvement
侧边栏折叠 SHALL 使用 `width: 0` + `overflow: hidden` 实现。展开/折叠按钮位于 AppHeader 的导航工具栏中。

#### Scenario: Collapsed state
- **WHEN** 用户点击 AppHeader 中的 Sidebar 折叠按钮
- **THEN** AppSidebar 动画收缩到 0px 宽度
- **AND** ContentBlock 扩展占满剩余空间
