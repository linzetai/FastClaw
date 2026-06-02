## ADDED Requirements

### Requirement: Dedicated ApprovalCard for tool execution approvals
The system SHALL render a dedicated `ApprovalCard` component for `approval_required` events, replacing the generic QuestionPanel for approval flows.

#### Scenario: Shell command approval shows command preview
- **WHEN** an `approval_required` event arrives for a `shell_exec` tool call
- **THEN** the ApprovalCard displays the full command string in a monospace code block
- **THEN** the card shows the risk reason from ExecPolicy or Guardian
- **THEN** the user can choose: Approve / Approve for Session / Deny / Abort

#### Scenario: File write approval shows content preview
- **WHEN** an `approval_required` event arrives for a `write_file` or `edit_file` tool call
- **THEN** the ApprovalCard shows the target file path and a preview of content changes
- **THEN** the user can expand/collapse the full diff

#### Scenario: Approve for Session caches the decision
- **WHEN** the user clicks "Approve for Session" (本次会话允许)
- **THEN** the `ApprovedForSession` decision is sent to the backend
- **THEN** subsequent identical approval requests in the same session are auto-approved without showing the card

### Requirement: ApprovalCard shows risk level indicator
The ApprovalCard SHALL display a visual risk level based on the ExecPolicy decision and Guardian assessment.

#### Scenario: Forbidden command shows red danger indicator
- **WHEN** an approval event has policy decision `Forbidden`
- **THEN** the card shows a red border and "禁止执行" label
- **THEN** only the "查看详情" and "关闭" buttons are available (no approve option)

#### Scenario: Prompt-level command shows yellow caution indicator
- **WHEN** an approval event has policy decision `Prompt`
- **THEN** the card shows an amber border and "需要确认" label
- **THEN** all decision buttons are available

### Requirement: Default ExecPolicy loaded at startup
The Gateway SHALL load a built-in default ExecPolicy file during initialization when no user-configured `exec_policy_path` is provided.

#### Scenario: Gateway starts with default policy
- **WHEN** the Gateway initializes without a user-configured `exec_policy_path`
- **THEN** the built-in `config/exec-policy.json` is loaded into the PolicyEngine
- **THEN** shell commands are evaluated against the default rules

#### Scenario: User-configured policy overrides default
- **WHEN** the user sets `exec_policy_path` in their configuration
- **THEN** the user's policy file is loaded instead of the built-in default

### Requirement: Default approval strategy is Interactive
The system SHALL use `Interactive` as the default approval strategy when no explicit strategy is configured.

#### Scenario: No approval strategy configured
- **WHEN** an agent has no explicit `approval_strategy` in its configuration
- **THEN** the system uses `Interactive` mode (user must approve dangerous operations)
- **THEN** safe operations (read-only tools) proceed without approval
