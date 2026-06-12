## MODIFIED Requirements

### Requirement: Override takes effect on next turn
权限变更 SHALL 支持在当前 turn 立即生效的场景：当用户通过 `PermissionSelector` 切换到 "full-auto" 预设时，SHALL 立即设置当前 turn 的 `ApprovalCache.global_approved = true`，使后续工具调用自动批准。

#### Scenario: Mid-turn permission change to full-auto takes effect immediately
- **WHEN** session 正在执行 turn（Agent 正在调用工具）
- **AND** 用户通过 PermissionSelector 切换到 "full-auto" 预设
- **THEN** 后端立即设置当前 turn 的 `ApprovalCache.global_approved = true`
- **AND** 当前 turn 后续工具调用自动批准，无需弹窗
- **AND** 前端不再显示 "权限将在下一轮对话生效" 提示

#### Scenario: Mid-turn permission change to restrictive preset
- **WHEN** session 正在执行 turn
- **AND** 用户通过 PermissionSelector 切换到 "ask-edit"（更严格的预设）
- **THEN** 当前 turn 继续使用已有审批策略
- **AND** 下一个 turn 使用新的严格预设
