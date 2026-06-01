## ADDED Requirements

### Requirement: 任务页面展示 Cron Job 列表
系统 SHALL 在侧边栏「任务」页面展示所有定时任务，每个任务显示名称、cron 表达式（及人类可读描述）、状态（idle/running/failed/disabled）、运行次数、错误次数、上次和下次运行时间。

#### Scenario: 正常展示任务列表
- **WHEN** 用户点击侧边栏「任务」按钮
- **THEN** 页面展示所有 Cron Job 卡片，按创建时间倒序排列

#### Scenario: 空任务列表
- **WHEN** 没有创建任何 Cron Job
- **THEN** 展示空状态提示和「创建第一个任务」按钮

#### Scenario: 任务状态指示
- **WHEN** 任务列表中有不同状态的任务
- **THEN** idle 显示灰色、running 显示蓝色动画、failed 显示红色、disabled 显示暗灰色

### Requirement: 创建 Cron Job
系统 SHALL 提供一个 Modal 表单让用户创建新的定时任务，表单字段包括：name、schedule（cron 表达式，带常用预设选择）、action 类型（Prompt 或 Webhook）及对应参数、enabled 开关。

#### Scenario: 使用预设创建 Prompt 类型任务
- **WHEN** 用户选择「每天 9:00」预设，填写任务名称和 prompt 文本，选择 agent，提交
- **THEN** 系统调用 `cron.upsert_job`，任务出现在列表中，状态为 idle

#### Scenario: 自定义 cron 表达式
- **WHEN** 用户选择「自定义」并输入 cron 表达式
- **THEN** 表单显示该表达式的人类可读翻译（如 "0 */2 * * *" → "每 2 小时"）

#### Scenario: 创建 Webhook 类型任务
- **WHEN** 用户选择 Webhook 类型，填写 URL 和方法
- **THEN** 系统验证 URL 后调用 `cron.upsert_job`，任务出现在列表中

### Requirement: 编辑 Cron Job
系统 SHALL 允许用户编辑已有的定时任务，打开与创建相同的表单 Modal 并预填当前值。

#### Scenario: 编辑任务调度
- **WHEN** 用户点击任务卡片的编辑按钮
- **THEN** 弹出 Modal，预填当前任务配置，用户可修改后提交

### Requirement: 删除 Cron Job
系统 SHALL 允许用户删除定时任务，操作前需要确认。

#### Scenario: 确认删除任务
- **WHEN** 用户点击删除按钮并在确认弹窗中确认
- **THEN** 系统调用 `cron.delete_job`，任务从列表中移除

### Requirement: 启用/禁用 Cron Job
系统 SHALL 允许用户切换任务的启用/禁用状态。

#### Scenario: 禁用任务
- **WHEN** 用户点击启用中任务的开关
- **THEN** 任务状态变为 disabled，不再按计划执行

#### Scenario: 启用任务
- **WHEN** 用户点击已禁用任务的开关
- **THEN** 任务状态变为 idle，恢复按计划执行

### Requirement: 查看 Cron Job 运行历史
系统 SHALL 在用户展开某个任务时展示其最近的运行记录，包括运行时间、状态（成功/失败）、耗时、输出和错误信息。

#### Scenario: 展开运行历史
- **WHEN** 用户点击任务卡片
- **THEN** 卡片展开显示最近 20 条运行记录，按时间倒序

#### Scenario: 无运行历史
- **WHEN** 新创建的任务还未执行过
- **THEN** 展示「暂无运行记录」提示
