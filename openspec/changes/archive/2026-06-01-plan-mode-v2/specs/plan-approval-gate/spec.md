## ADDED Requirements

### Requirement: exit_plan_mode returns pending approval state
`exit_plan_mode` tool SHALL NOT directly transition the session to Agent mode. Instead, it SHALL return a `ToolResult` with `metadata.approval_pending = true` and remain in Plan mode until the user explicitly approves.

#### Scenario: Successful exit triggers approval pending
- **WHEN** Agent calls `exit_plan_mode` with `all_steps_completed = true` (or default)
- **THEN** Tool SHALL return `success: true` with `metadata: { "approval_pending": true, "plan_path": "<path>", "plan_exists": <bool> }` and the session SHALL remain in Plan mode

#### Scenario: Incomplete steps still block exit
- **WHEN** Agent calls `exit_plan_mode` with `plan_summary` set and `all_steps_completed = false`
- **THEN** Tool SHALL return the existing "INCOMPLETE" warning without any approval metadata

#### Scenario: Tool result text indicates waiting for approval
- **WHEN** `exit_plan_mode` returns pending approval
- **THEN** The output text SHALL include "等待用户审批" or "Waiting for user approval" and SHALL NOT say "All tools are now available"

### Requirement: execution.approve_plan RPC endpoint
A new WebSocket RPC method `execution.approve_plan` SHALL accept `{ sessionId, mode }` where `mode` is `"agent"`. It SHALL transition the session from Plan to the requested mode and broadcast `mode_change`.

#### Scenario: User approves plan
- **WHEN** Frontend calls `execution.approve_plan` with `{ sessionId: "...", mode: "agent" }`
- **THEN** Backend SHALL transition the session to Agent mode, broadcast `mode_change` and `plan_file_update` events, and return `{ ok: true }`

#### Scenario: Approve when not in Plan mode
- **WHEN** Frontend calls `execution.approve_plan` but the session is already in Agent mode
- **THEN** Backend SHALL return `{ ok: true }` as a no-op

### Requirement: PlanApprovalCard renders execution mode choices
When `PlanApprovalCard` detects `approval_pending` in the tool metadata, it SHALL render action buttons for the user to choose how to proceed.

#### Scenario: Approval card shows action buttons
- **WHEN** `exit_plan_mode` result has `metadata.approval_pending = true`
- **THEN** `PlanApprovalCard` SHALL render: "开始实现" (calls `execution.approve_plan`) and "继续规划" (dismisses the card, stays in Plan mode)

#### Scenario: Plan preview in approval card
- **WHEN** Approval card is rendered and a plan file exists
- **THEN** Card SHALL show expandable Markdown preview of the plan file content (reuse existing `getPlanFile` transport)

#### Scenario: After approval, card shows completed state
- **WHEN** User clicks "开始实现" and `execution.approve_plan` succeeds
- **THEN** Card SHALL update to show "已切换到 Agent 模式" confirmation and disable buttons

### Requirement: StepIndicator passes onImplement to PlanApprovalCard
`StepIndicator` SHALL pass an `onImplement` callback to `PlanApprovalCard` that calls `execution.approve_plan` and updates the chat's execution mode in the store.

#### Scenario: onImplement wired in StepIndicator
- **WHEN** `StepIndicator` renders `PlanApprovalCard` for an `exit_plan_mode` result
- **THEN** It SHALL pass `onImplement` callback that invokes `transport.approvePlan(sessionId)` and calls `setChatExecutionMode(agentId, chatId, "agent")`
