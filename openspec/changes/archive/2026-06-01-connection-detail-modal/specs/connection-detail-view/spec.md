## ADDED Requirements

### Requirement: MCP 详情后端接口
系统 SHALL 提供 `mcp.detail` WebSocket 方法，接受 `{ id: string }` 参数，返回指定 MCP 服务器的完整信息。

返回数据 SHALL 包含：
- `id`, `status`, `error`, `toolCount`, `connectedAt`（来自 McpServerStatus）
- `config`: `{ command, args, transport, env, url }`（来自 McpServerConfig，env 中的值做掩码处理）
- `tools`: `Array<{ name, description }>`（来自 McpClient.tools()）

#### Scenario: 查询已连接的 MCP 服务器详情
- **WHEN** 前端发送 `mcp.detail` 请求，id 为已连接的服务器
- **THEN** 返回完整状态、配置信息和 tool 列表

#### Scenario: 查询不存在的 MCP 服务器
- **WHEN** 前端发送 `mcp.detail` 请求，id 不存在
- **THEN** 返回 error，message 为 "server not found"

### Requirement: Channel 详情后端接口
系统 SHALL 提供 `channels.detail` WebSocket 方法，接受 `{ id: string }` 参数，返回指定 Channel 的完整信息。

返回数据 SHALL 包含：
- `id`, `name`, `description`, `aliases`, `status`, `connectionMode`, `capabilities`（同 list）
- `config`: 脱敏后的配置信息（app_id 完整显示，app_secret/encrypt_key/token 仅前4字符 + "****"）
- `tools`: `Array<{ name, description }>`（来自 ChannelPlugin.tools()）

#### Scenario: 查询已注册的 Channel 详情
- **WHEN** 前端发送 `channels.detail` 请求，id 为已注册的 channel
- **THEN** 返回完整元数据、脱敏配置和 tool 列表

#### Scenario: 查询未注册但已知的 Channel
- **WHEN** 前端发送 `channels.detail` 请求，id 为已知但未注册的 channel（如 "wechat"）
- **THEN** 返回基础元数据和 "available"/"configured" 状态，tools 为空数组

### Requirement: MCP 详情弹窗
前端 SHALL 在用户点击 MCP 服务器卡片时打开 McpDetailModal，展示完整信息。

弹窗 SHALL 包含以下区域：
1. 标题栏：服务器 ID + 状态标识 + 关闭按钮
2. 配置区：启动命令、参数、传输类型
3. 工具列表：所有 tool 的名称和描述，支持滚动
4. 错误区（条件显示）：错误详情
5. 操作栏：重载、删除按钮

#### Scenario: 打开 MCP 详情弹窗
- **WHEN** 用户点击 MCP 服务器卡片（非操作按钮区域）
- **THEN** 弹窗打开，显示加载状态后展示完整信息

#### Scenario: 弹窗中查看工具列表
- **WHEN** 弹窗打开且服务器已连接
- **THEN** 展示完整 tool 列表，每个 tool 显示名称和描述

#### Scenario: 弹窗中删除服务器
- **WHEN** 用户在弹窗中点击删除按钮并确认
- **THEN** 服务器被删除，弹窗关闭，列表刷新

### Requirement: Channel 详情弹窗
前端 SHALL 在用户点击 Channel 卡片时打开 ChannelDetailModal，展示完整信息。

弹窗 SHALL 包含以下区域：
1. 标题栏：Channel 名称 + 状态标签 + 连接模式 + 关闭按钮
2. 元数据区：描述、别名
3. 能力区：能力列表及说明
4. 配置区（条件显示）：脱敏后的配置信息
5. 工具列表：Channel 提供的 tools
6. 操作栏：连接/断开按钮

#### Scenario: 打开已连接 Channel 的详情弹窗
- **WHEN** 用户点击已连接的 Channel 卡片
- **THEN** 弹窗打开，展示完整信息，操作栏显示「断开」按钮

#### Scenario: 打开未连接 Channel 的详情弹窗
- **WHEN** 用户点击未连接的 Channel 卡片
- **THEN** 弹窗打开，展示基础信息，操作栏显示「连接」按钮

#### Scenario: 弹窗中断开 Channel
- **WHEN** 用户在弹窗中点击断开按钮
- **THEN** Channel 被断开，弹窗关闭，列表刷新

### Requirement: 卡片点击交互
MCP 和 Channel 卡片 SHALL 支持整体点击打开详情，同时保持操作按钮的独立功能。

#### Scenario: 点击卡片空白区域
- **WHEN** 用户点击卡片的非按钮区域
- **THEN** 打开对应的详情弹窗

#### Scenario: 点击卡片上的操作按钮
- **WHEN** 用户点击卡片上的删除/断开按钮
- **THEN** 执行按钮对应的操作，不打开详情弹窗（事件冒泡被阻止）
