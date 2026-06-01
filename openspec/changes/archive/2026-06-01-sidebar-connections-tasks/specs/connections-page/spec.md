## ADDED Requirements

### Requirement: 连接页面展示 MCP 服务器列表
系统 SHALL 在侧边栏「连接」页面展示所有已配置的 MCP 服务器，包括 id、连接状态（connected/failed/disabled/connecting）、提供的工具数量、连接时间和错误信息。

#### Scenario: 正常展示 MCP 列表
- **WHEN** 用户点击侧边栏「连接」按钮
- **THEN** 页面展示所有 MCP 服务器卡片，每张卡片包含状态指示灯（绿色=connected, 红色=failed, 灰色=disabled）、服务器 id、工具数量

#### Scenario: MCP 服务器有错误
- **WHEN** 某个 MCP 服务器状态为 failed 且存在 error 信息
- **THEN** 该卡片显示红色状态灯和错误描述文字

#### Scenario: 无 MCP 服务器
- **WHEN** 没有配置任何 MCP 服务器
- **THEN** MCP 区域展示空状态提示

### Requirement: 添加 MCP 服务器
系统 SHALL 提供一个表单让用户添加新的 MCP 服务器，表单包含 id（标识符）、command（启动命令）、args（参数列表）三个字段。

#### Scenario: 成功添加 MCP 服务器
- **WHEN** 用户填写 id、command、args 并提交
- **THEN** 系统调用 `mcp.add` 命令，成功后刷新 MCP 列表，新服务器出现在列表中

#### Scenario: 添加 MCP 服务器失败
- **WHEN** 用户提交表单但连接失败
- **THEN** 新服务器出现在列表中但状态为 failed，显示错误信息

### Requirement: 删除 MCP 服务器
系统 SHALL 允许用户删除已配置的 MCP 服务器。

#### Scenario: 确认删除 MCP 服务器
- **WHEN** 用户点击 MCP 卡片上的删除按钮并确认
- **THEN** 系统调用 `mcp.remove` 命令，该服务器从列表中移除

### Requirement: 重载所有 MCP 服务器
系统 SHALL 提供一个「重载」按钮，重新加载所有 MCP 服务器连接。

#### Scenario: 重载 MCP 连接
- **WHEN** 用户点击「重载」按钮
- **THEN** 系统调用 `mcp.reload`，所有服务器重新连接，列表刷新为最新状态

### Requirement: 连接页面展示 Channel 列表
系统 SHALL 在「连接」页面的第二个区块展示所有已加载的消息通道插件，包括 id、名称、描述和别名列表。

#### Scenario: 正常展示 Channel 列表
- **WHEN** 配置了飞书和微信通道插件
- **THEN** Channel 区域展示两张卡片，分别显示飞书和微信的名称、描述

#### Scenario: 无 Channel 插件
- **WHEN** 没有配置任何 Channel 插件
- **THEN** Channel 区域展示空状态提示，说明可通过配置文件添加

### Requirement: 移除专家 Tab
系统 SHALL 从侧边栏导航中移除「专家」选项，NavItem 类型中不再包含 `experts`。

#### Scenario: 侧边栏不显示专家按钮
- **WHEN** 应用启动后渲染侧边栏
- **THEN** 侧边栏包含：对话、工作室、任务、文件、连接 五个按钮，不包含「专家」
