## MODIFIED Requirements

### Requirement: PlanApprovalCard renders execution mode choices
When `PlanApprovalCard` detects `approval_pending` in the tool metadata, it SHALL render action buttons for the user to choose how to proceed. The approval flow SHALL use a dedicated approval card style instead of the generic QuestionPanel, with options for "开始实现" (agent mode) and "继续规划" (plan mode), plus a "记住选择" toggle.

#### Scenario: Plan approval renders as ApprovalCard style
- **WHEN** `exit_plan_mode` returns with `approval_pending: true`
- **THEN** the PlanApprovalCard renders with the same visual language as the new ApprovalCard (risk-level border, action preview)
- **THEN** the plan content preview is expandable inline

#### Scenario: Plan approval supports remember choice
- **WHEN** the user toggles "记住选择" and clicks "开始实现"
- **THEN** subsequent plan exits in the same session auto-approve to agent mode
