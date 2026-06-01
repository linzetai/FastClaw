## 1. 侧边栏精简

- [x] 1.1 从 `NavItem` 类型移除 `experts`，更新为 `"chat" | "workspace" | "tasks" | "files" | "connections"`
- [x] 1.2 从 `NavRail.tsx` 的 `TOP_ITEMS` 数组移除专家项
- [x] 1.3 从 `AppLayout.tsx` 的 `COMING_SOON_TITLES` 移除 `experts` 映射
- [x] 1.4 验证侧边栏只显示 5 个导航按钮

## 2. 连接页面 - Channel 列表 API

- [x] 2.1 在 `transport.ts` 新增 `getChannels()` 函数，通过 HTTP `GET /api/v1/channels` 获取 Channel 列表
- [x] 2.2 在 `api.ts` 新增 `listChannels()` 导出封装

## 3. 连接页面 - UI 组件

- [x] 3.1 创建 `ConnectionsPage.tsx` 组件骨架，分 MCP 服务器和消息通道两个区块
- [x] 3.2 实现 MCP 服务器列表展示（卡片：id、状态灯、工具数、连接时间、错误信息）
- [x] 3.3 实现 MCP 空状态展示
- [x] 3.4 实现 Channel 列表展示（卡片：名称、描述、别名）
- [x] 3.5 实现 Channel 空状态展示
- [x] 3.6 实现「添加 MCP 服务器」按钮和 Modal 表单（id、command、args）
- [x] 3.7 实现「删除 MCP 服务器」按钮（带确认）
- [x] 3.8 实现「全部重载」按钮，调用 `reloadMcpServers()`
- [x] 3.9 在 `AppLayout.tsx` 中集成 lazy-loaded `ConnectionsPage`，替换 connections 的 ComingSoon

## 4. 任务页面 - UI 组件

- [x] 4.1 创建 `TasksPage.tsx` 组件骨架，展示 Cron Job 列表
- [x] 4.2 实现 Cron Job 卡片（名称、cron 表达式、人类可读描述、状态灯、运行/错误计数、上次/下次运行时间）
- [x] 4.3 实现空状态展示（含「创建第一个任务」按钮）
- [x] 4.4 实现「创建/编辑任务」Modal 表单（name、schedule 预设 + 自定义、action 类型切换、enabled 开关）
- [x] 4.5 实现 cron 表达式人类可读翻译工具函数
- [x] 4.6 实现「删除任务」按钮（带确认弹窗）
- [x] 4.7 实现启用/禁用切换（调用 `cronUpsertJob` 更新 enabled 字段）
- [x] 4.8 实现点击卡片展开运行历史面板（调用 `cronListRuns`，显示时间、状态、耗时、输出/错误）
- [x] 4.9 在 `AppLayout.tsx` 中集成 lazy-loaded `TasksPage`，替换 tasks 的 ComingSoon

## 5. 验证

- [x] 5.1 `cargo tauri dev` 启动，验证侧边栏 5 个按钮正确显示
- [x] 5.2 验证「连接」页面 MCP 列表正确展示，添加/删除/重载操作正常
- [x] 5.3 验证「任务」页面创建/编辑/删除/启用禁用/运行历史功能正常
