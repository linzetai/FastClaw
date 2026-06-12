## MODIFIED Requirements

### Requirement: Batch approval option
审批卡片 SHALL 提供"本轮全部允许"选项，点击后自动批准当前 turn 中**所有工具类型**的后续操作，而非仅限同类型工具。

#### Scenario: Approve all for session covers all tool types
- **WHEN** 用户点击 "本轮全部允许"
- **THEN** 前端发送 `decision: "approved_all_for_session"` 给后端
- **AND** 后端在 `ApprovalCache` 中设置 `global_approved = true`
- **AND** 当前 turn 中所有后续工具调用（无论工具类型）自动批准
- **AND** 下一个 turn 恢复正常审批流程（`global_approved` 被重置）

#### Scenario: Approval timeout prevents indefinite blocking
- **WHEN** 后端发送审批请求到前端
- **AND** 前端在 5 分钟内未响应
- **THEN** 后端自动将该审批决策设为 `TimedOut`
- **AND** 对应工具调用被跳过，agent 继续执行
