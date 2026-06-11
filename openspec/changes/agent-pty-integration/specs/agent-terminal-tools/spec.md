## ADDED Requirements

### Requirement: terminal_open tool creates PTY session
The system SHALL provide a `terminal_open` tool that creates a persistent interactive PTY session using the shared `xiaolin_pty::PtySessionManager`. The tool SHALL accept optional `cwd` (working directory) and `name` (display name) parameters. It SHALL return the `session_id` and initial terminal output (shell prompt).

#### Scenario: Agent opens a terminal with default settings
- **WHEN** Agent calls `terminal_open` with no parameters
- **THEN** system creates a PTY session in the project root directory, returns a `session_id` and the shell prompt output

#### Scenario: Agent opens a terminal with custom cwd and name
- **WHEN** Agent calls `terminal_open` with `cwd: "/tmp"` and `name: "Build Server"`
- **THEN** system creates a PTY session in `/tmp`, the session appears in Shell tab as "Build Server", and returns `session_id` with prompt

#### Scenario: Agent exceeds max session limit
- **WHEN** Agent already has 3 open PTY sessions and calls `terminal_open`
- **THEN** system returns an error indicating the agent session limit is reached

### Requirement: terminal_input sends input and reads output
The system SHALL provide a `terminal_input` tool that writes input to an existing PTY session and returns the resulting output. It SHALL support `wait_ms` (timeout) and `wait_for` (text pattern) parameters to control when output collection stops.

#### Scenario: Agent sends a command and waits for output
- **WHEN** Agent calls `terminal_input` with `session_id`, `input: "echo hello\n"`, `wait_ms: 2000`
- **THEN** system writes the input to the PTY, waits up to 2000ms, and returns all output received during that period

#### Scenario: Agent waits for specific text pattern
- **WHEN** Agent calls `terminal_input` with `input: "npm run dev\n"` and `wait_for: "ready on port"`
- **THEN** system writes the input and returns output as soon as "ready on port" appears (or after `wait_ms` timeout)

#### Scenario: Agent sends input to non-existent session
- **WHEN** Agent calls `terminal_input` with an invalid `session_id`
- **THEN** system returns an error indicating the session was not found

### Requirement: terminal_close terminates PTY session
The system SHALL provide a `terminal_close` tool that closes an existing PTY session. The session SHALL be removed from the frontend Shell tab after closure.

#### Scenario: Agent closes an active session
- **WHEN** Agent calls `terminal_close` with a valid `session_id`
- **THEN** system kills the PTY process, removes the session, and notifies the frontend

#### Scenario: Agent closes already-closed session
- **WHEN** Agent calls `terminal_close` with a session that has already exited
- **THEN** system returns success (idempotent)

### Requirement: Tool registration in agent context
The `terminal_open`, `terminal_input`, and `terminal_close` tools SHALL be registered in the `ToolRegistry` and receive the shared `Arc<xiaolin_pty::PtySessionManager>` at construction time via `state/builder.rs`.

#### Scenario: Tools available in agent tool list
- **WHEN** Agent requests its available tools
- **THEN** `terminal_open`, `terminal_input`, and `terminal_close` appear in the tool list with correct schemas

### Requirement: Tool descriptions guide LLM selection
Each terminal tool description SHALL clearly distinguish its use case from `shell_exec` to help the LLM make correct tool selection decisions.

#### Scenario: LLM reads tool descriptions
- **WHEN** LLM sees both `shell_exec` and `terminal_open` in available tools
- **THEN** `shell_exec` description emphasizes "single command, quick execution" while `terminal_open` emphasizes "persistent session, interactive processes, multi-step workflows"
