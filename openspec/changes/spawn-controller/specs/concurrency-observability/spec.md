## ADDED Requirements

### Requirement: HTTP endpoint exposes concurrency snapshot
The system SHALL provide `GET /api/v1/subagents/concurrency` returning a `ConcurrencySnapshot` JSON object.

#### Scenario: No active sub-agents
- **WHEN** no sub-agents are running
- **THEN** the response SHALL contain `global_active: 0`, `global_max: <configured>`, and empty `sessions` array

#### Scenario: Active sub-agents across sessions
- **WHEN** Session A has 2 running sub-agents and Session B has 1
- **THEN** the response SHALL contain `global_active: 3`
- **AND** `sessions` array SHALL contain two entries with `active: 2` and `active: 1` respectively

#### Scenario: RwState reflects lock status
- **WHEN** a session has 3 concurrency-safe agents running
- **THEN** the session's `rw_state` SHALL be `{"Reading": 3}`
- **WHEN** a non-concurrency-safe agent is running exclusively
- **THEN** the session's `rw_state` SHALL be `{"Writing": "<run_id>"}`

### Requirement: WebSocket op exposes concurrency snapshot
The system SHALL handle `sub_agents.concurrency` WebSocket operation returning the same `ConcurrencySnapshot` as the HTTP endpoint.

#### Scenario: WS client requests concurrency status
- **WHEN** a WebSocket client sends `{"method": "sub_agents.concurrency"}`
- **THEN** the response SHALL contain the `ConcurrencySnapshot` JSON

### Requirement: ConcurrencySnapshot includes active agent details
Each session in the snapshot SHALL include an `agents` array with `run_id`, `def_id`, `concurrency_safe`, `started_at`, and `elapsed_ms` for each active sub-agent.

#### Scenario: Agent details are accurate
- **WHEN** an explore agent (concurrency_safe=true) has been running for 5 seconds
- **THEN** its entry in the `agents` array SHALL contain `concurrency_safe: true` and `elapsed_ms` approximately 5000

### Requirement: Snapshot includes queued spawn count
Each session in the snapshot SHALL include a `queued_spawns` count indicating how many spawn requests are waiting for a slot.

#### Scenario: Spawns queued when session is full
- **WHEN** a session is at `max_per_session` capacity
- **AND** 2 additional spawns are waiting for slots
- **THEN** the session's `queued_spawns` SHALL be `2`
