## ADDED Requirements

### Requirement: LLM-based operation review
系统 SHALL 提供 Guardian Agent，使用独立的 LLM session 评估待执行操作是否符合用户意图、风险是否可接受，并返回结构化评估结果。

#### Scenario: 审核通过允许执行
- **WHEN** Guardian Agent 评估命令 `git push origin main` 且用户意图明确为推送代码
- **THEN** Guardian SHALL 返回 `{ decision: "allow", risk_level: "low", rationale: "..." }`

#### Scenario: 审核拒绝阻止执行
- **WHEN** Guardian Agent 评估命令 `rm -rf /` 且无明确用户意图要求删除根目录
- **THEN** Guardian SHALL 返回 `{ decision: "deny", risk_level: "high", rationale: "..." }`
- **AND** ShellTool SHALL 拒绝执行并将 rationale 作为错误信息返回

### Requirement: Fail-closed design
Guardian SHALL 在超时、LLM 调用失败、或返回格式无法解析时默认拒绝操作。

#### Scenario: LLM 调用超时
- **WHEN** Guardian 的 LLM 调用在配置的超时时间（默认 60 秒）内未返回
- **THEN** Guardian SHALL 返回 deny 决策并附带超时说明

#### Scenario: LLM 返回格式异常
- **WHEN** LLM 返回的 JSON 无法解析为有效的 GuardianAssessment
- **THEN** Guardian SHALL 返回 deny 决策并记录解析错误日志

#### Scenario: LLM 服务不可用
- **WHEN** Guardian 的 LLM 调用因网络错误或 API 错误失败
- **THEN** Guardian SHALL 返回 deny 决策并附带错误说明

### Requirement: Intent transcript reconstruction
Guardian SHALL 从对话历史中重建用户意图的紧凑 transcript，限制 token 数量以控制 LLM 调用成本。

#### Scenario: 提取最近的用户消息
- **WHEN** 对话历史包含 20 轮消息
- **THEN** Guardian SHALL 提取最近的用户消息和相关 assistant 回复，总 token 数 SHALL 不超过配置的上限（默认 10000 tokens）

#### Scenario: 包含待审核操作详情
- **WHEN** 构建审核 prompt
- **THEN** prompt SHALL 包含待执行的完整命令、工作目录、以及操作类型描述

### Requirement: Structured assessment output
Guardian 的 LLM session SHALL 返回严格的 JSON 结构化输出，包含决策、风险等级和理由。

#### Scenario: 有效的评估结果
- **WHEN** LLM 返回 `{ "decision": "allow", "risk_level": "low", "rationale": "Command is a standard read operation" }`
- **THEN** Guardian SHALL 解析并返回对应的 GuardianAssessment 结构体

#### Scenario: 风险等级分类
- **WHEN** Guardian 评估一个操作
- **THEN** risk_level SHALL 为 `low`、`medium` 或 `high` 之一

### Requirement: Optional activation
Guardian SHALL 默认关闭，通过配置 `guardian.enabled = true` 启用。未启用时 ExecPolicy 返回 Prompt 的命令直接进入用户确认流程。

#### Scenario: Guardian 未启用时跳过审核
- **WHEN** 配置中 `guardian.enabled = false`（默认）且 ExecPolicy 返回 Prompt
- **THEN** 系统 SHALL 跳过 Guardian 审核，直接进入用户确认流程

#### Scenario: Guardian 启用时自动审核
- **WHEN** 配置中 `guardian.enabled = true` 且 ExecPolicy 返回 Prompt
- **THEN** 系统 SHALL 调用 Guardian Agent 进行自动审核，根据结果决定允许或拒绝
