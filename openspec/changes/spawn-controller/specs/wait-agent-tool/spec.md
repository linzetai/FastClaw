## ADDED Requirements

### Requirement: WaitAgentTool enables LLM to batch-wait sub-agents
The system SHALL provide a `wait_agent` tool that allows the LLM to wait for one or more sub-agent runs to reach a terminal state (completed, failed, or cancelled).

#### Scenario: Wait-all returns when all agents complete
- **WHEN** `wait_agent` is called with `mode="all"` and `run_ids=["r1","r2","r3"]`
- **AND** all three agents eventually complete
- **THEN** the tool SHALL return a result containing the status and output of all three agents

#### Scenario: Wait-any returns on first completion
- **WHEN** `wait_agent` is called with `mode="any"` and `run_ids=["r1","r2","r3"]`
- **AND** agent r1 completes while r2 and r3 are still running
- **THEN** the tool SHALL return immediately with r1's result
- **AND** r2 and r3 SHALL continue running (not cancelled)

#### Scenario: Wait with timeout returns partial results
- **WHEN** `wait_agent` is called with `timeout_seconds=10`
- **AND** agent r1 completes within 10 seconds but r2 does not
- **THEN** the tool SHALL return r1's result plus a `timed_out: true` indicator for r2

### Requirement: WaitAgentTool handles already-completed agents
The tool SHALL check if any requested agents have already reached a terminal state before subscribing to events.

#### Scenario: All agents already completed
- **WHEN** `wait_agent` is called with run_ids of agents that have already completed
- **THEN** the tool SHALL return immediately without waiting for broadcast events

#### Scenario: Mixed completed and running agents
- **WHEN** `wait_agent` is called with run_ids where some are completed and some are running
- **AND** `mode="all"`
- **THEN** the tool SHALL include already-completed results and wait only for running agents

### Requirement: WaitAgentTool result format
The tool SHALL return a JSON object containing a `results` map (run_id → status/result/error) and a `timed_out` boolean.

#### Scenario: Successful wait-all result
- **WHEN** all agents complete successfully
- **THEN** the result SHALL contain each agent's `run_id`, `status: "completed"`, and `result` text
- **AND** `timed_out` SHALL be `false`

#### Scenario: Agent failure included in result
- **WHEN** one agent fails during a wait-all
- **THEN** its entry SHALL contain `status: "failed"` and `error` message
- **AND** other completed agents SHALL still have their results

### Requirement: WaitAgentTool validates run_ids
The tool SHALL validate that all provided run_ids correspond to existing sub-agent runs.

#### Scenario: Unknown run_id
- **WHEN** `wait_agent` is called with a run_id that does not exist
- **THEN** the tool SHALL return an error indicating the unknown run_id

### Requirement: WaitAgentTool is non-parallel
The tool SHALL declare `supports_parallel() == false` to prevent multiple wait calls from running concurrently within the same turn.

#### Scenario: Tool declaration
- **WHEN** the tool registry queries `wait_agent` capabilities
- **THEN** `supports_parallel()` SHALL return `false`
- **AND** `kind()` SHALL return `ToolKind::System`
