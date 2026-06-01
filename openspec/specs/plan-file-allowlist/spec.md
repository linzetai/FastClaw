## ADDED Requirements

### Requirement: Plan file path exemption in dispatcher
The dispatcher SHALL allow `write_file` and `edit_file` tool calls in Plan mode when the target `path` argument resolves to the current session's plan file path (`PlanFileStore::plan_path(session_id)`). All other `ToolKind::Edit` tools SHALL remain blocked.

#### Scenario: Agent writes plan file in Plan mode
- **WHEN** Agent calls `write_file` with `path` equal to the session's plan file path while in Plan mode
- **THEN** Dispatcher SHALL allow the call to proceed without blocking

#### Scenario: Agent edits plan file in Plan mode
- **WHEN** Agent calls `edit_file` with `path` equal to the session's plan file path while in Plan mode
- **THEN** Dispatcher SHALL allow the call to proceed without blocking

#### Scenario: Agent writes non-plan file in Plan mode
- **WHEN** Agent calls `write_file` with `path` NOT matching the plan file path while in Plan mode
- **THEN** Dispatcher SHALL block the call with `ExecutionDenied` error

#### Scenario: Path matching uses canonical paths
- **WHEN** Agent provides a relative or symlinked path that resolves to the plan file
- **THEN** Dispatcher SHALL canonicalize both paths before comparison and allow the write

### Requirement: Plan file path injected into dispatch context
The runtime SHALL inject the current session's plan file path into `DispatchContext` so the dispatcher can perform path comparison without accessing `PlanFileStore` directly.

#### Scenario: DispatchContext contains plan path
- **WHEN** A tool dispatch occurs in Plan mode with an active session
- **THEN** `DispatchContext.plan_file_path` SHALL be `Some(path)` where `path` is the result of `PlanFileStore::plan_path(session_id)`

### Requirement: Parallel execution respects Plan mode blocks
`execute_unguarded_standalone` SHALL enforce the same `ToolKind::Edit`/`Execute` blocking as `pre_execution_checks` for all parallel tool executions in Plan mode, including the plan file path exemption for Edit tools.

#### Scenario: Parallel Edit tool blocked in Plan mode
- **WHEN** `exec_command` (parallel, `ToolKind::Execute`) runs in Plan mode
- **THEN** `execute_unguarded_standalone` SHALL block it with `ExecutionDenied` error

#### Scenario: Parallel shell_exec with readonly command allowed
- **WHEN** `shell_exec` (parallel) runs a readonly command (e.g., `ls`) in Plan mode
- **THEN** `execute_unguarded_standalone` SHALL allow it

### Requirement: Mode switch event broadcast unification
`execution.set_mode` (UI toggle path) SHALL broadcast `mode_change` events through the chat stream, consistent with agent-driven mode changes.

#### Scenario: UI toggle broadcasts mode_change
- **WHEN** User toggles Plan mode via StreamFooter and `execution.set_mode` succeeds
- **THEN** Gateway SHALL broadcast a `mode_change` event with `from` and `to` fields on the chat stream

#### Scenario: Plan file update event after UI toggle
- **WHEN** User toggles to Plan mode via UI and a plan context exists
- **THEN** Gateway SHALL also broadcast a `plan_file_update` event with the plan file path and existence status
