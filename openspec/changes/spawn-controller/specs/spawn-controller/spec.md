## ADDED Requirements

### Requirement: Global slot pool limits total concurrent sub-agents
The system SHALL enforce a global maximum (`max_global`, default 20) on the total number of concurrently running sub-agents across all sessions. When the limit is reached, new spawn requests SHALL wait up to `slot_acquire_timeout` before failing.

#### Scenario: Global limit reached
- **WHEN** `max_global` sub-agents are already running across all sessions
- **AND** a new `spawn_subagent` is requested
- **THEN** the request SHALL wait for a slot to free up, up to `slot_acquire_timeout`
- **AND** if no slot frees within the timeout, the spawn SHALL fail with `SpawnControllerError::GlobalLimitTimeout`

#### Scenario: Global slot released on task completion
- **WHEN** a running sub-agent completes, fails, or is cancelled
- **THEN** its global slot SHALL be released immediately
- **AND** any pending spawn waiting for a global slot SHALL be notified

### Requirement: Per-session slot pool provides isolation
The system SHALL maintain an independent slot pool for each session tree. Each session pool SHALL enforce its own maximum (`max_per_session`, default 5). Sub-agents in different sessions SHALL NOT compete for each other's session slots.

#### Scenario: Session A full does not block Session B
- **WHEN** Session A has `max_per_session` running sub-agents
- **AND** Session B has fewer than `max_per_session` running sub-agents
- **AND** a new spawn is requested for Session B
- **THEN** the spawn SHALL succeed for Session B without waiting

#### Scenario: Session slot pool created on demand
- **WHEN** a spawn is requested for a session that has no existing pool
- **THEN** the system SHALL create a new `SessionSlotPool` for that session

### Requirement: RwLock gate enforces concurrency safety
When `enforce_rw_isolation` is true, the system SHALL use a `tokio::sync::RwLock<()>` per session pool to distinguish between concurrency-safe and non-concurrency-safe sub-agents.

#### Scenario: Multiple concurrency-safe agents run in parallel
- **WHEN** `enforce_rw_isolation` is true
- **AND** multiple sub-agents with `concurrency_safe = true` are spawned in the same session
- **THEN** they SHALL all acquire read guards and run concurrently (up to `max_per_session`)

#### Scenario: Non-concurrency-safe agent runs exclusively
- **WHEN** `enforce_rw_isolation` is true
- **AND** a sub-agent with `concurrency_safe = false` acquires the write guard
- **THEN** no other sub-agent (safe or unsafe) in the same session SHALL run until it releases the guard

#### Scenario: Writer waits for all readers to drain
- **WHEN** `enforce_rw_isolation` is true
- **AND** readers are currently holding the RwLock
- **AND** a `concurrency_safe = false` spawn is requested
- **THEN** the writer SHALL wait for all current readers to complete before acquiring the write guard

#### Scenario: RwLock disabled
- **WHEN** `enforce_rw_isolation` is false
- **THEN** all sub-agents SHALL only be limited by slot counts (no RwLock gating)

### Requirement: SpawnReservation provides RAII slot management
The system SHALL use a `SpawnReservation` struct that holds global slot, session slot, and RwLock guard references. On drop, all resources SHALL be released atomically.

#### Scenario: Normal completion releases reservation
- **WHEN** a sub-agent task completes normally
- **AND** the `SpawnReservation` is dropped
- **THEN** global slot count, session slot count, and RwLock guard SHALL all be released

#### Scenario: Task panic releases reservation
- **WHEN** a sub-agent task panics
- **AND** `SpawnReservation`'s drop handler runs
- **THEN** all slots and guards SHALL still be released

#### Scenario: Task cancellation releases reservation
- **WHEN** a sub-agent task is cancelled via `CancellationToken`
- **AND** the spawned tokio task is dropped
- **THEN** all slots and guards SHALL still be released

### Requirement: Broadcast event notification replaces polling
The system SHALL use `tokio::sync::broadcast` channels to notify subscribers of slot lifecycle events (`Acquired`, `Released`, `Completed`, `Failed`).

#### Scenario: spawn_and_wait receives completion event
- **WHEN** `spawn_and_wait` (formerly `spawn_sync`) is called
- **AND** the sub-agent completes
- **THEN** `spawn_and_wait` SHALL receive the `SlotEvent::Completed` event via broadcast
- **AND** return the result without polling delay

#### Scenario: Multiple subscribers receive same event
- **WHEN** both `spawn_and_wait` and `WaitAgentTool` subscribe to the same session's events
- **AND** a sub-agent completes
- **THEN** both subscribers SHALL receive the `SlotEvent::Completed` event

### Requirement: concurrency_safe flag is read from SubAgentDef
`SubAgentTool` SHALL read the `concurrency_safe` field from the resolved `SubAgentDef` and pass it to `SubAgentManager::spawn()`. If no `SubAgentDef` is found, `concurrency_safe` SHALL default to `false`.

#### Scenario: Explore agent spawns with concurrency_safe=true
- **WHEN** a `spawn_subagent` tool call specifies `type="explore"`
- **AND** the `explore` SubAgentDef has `concurrency_safe = true`
- **THEN** the spawn SHALL acquire a read guard (allowing parallel execution)

#### Scenario: Code agent spawns with concurrency_safe=false
- **WHEN** a `spawn_subagent` tool call specifies `type="code"`
- **AND** the `code` SubAgentDef has `concurrency_safe = false`
- **THEN** the spawn SHALL acquire a write guard (exclusive execution)

### Requirement: Configuration via config.toml
The system SHALL support a `[concurrency]` section in `config.toml` with keys: `max_global` (default 20), `max_per_session` (default 5), `enforce_rw_isolation` (default true), `slot_acquire_timeout_seconds` (default 30).

#### Scenario: Config overrides defaults
- **WHEN** `config.toml` contains `[concurrency] max_per_session = 3`
- **THEN** the `SpawnController` SHALL use `max_per_session = 3` instead of the default 5

#### Scenario: Backward compatibility with SubAgentPolicy
- **WHEN** no `[concurrency]` section exists in `config.toml`
- **THEN** `SpawnConfig` SHALL use `SubAgentPolicy.max_parallel` as `max_per_session`

### Requirement: Idle session pool garbage collection
The system SHALL periodically remove `SessionSlotPool` instances that have had no active sub-agents for longer than a configurable idle duration.

#### Scenario: Idle pool is collected
- **WHEN** a session pool has had zero active sub-agents for longer than `max_idle`
- **AND** `gc_idle_sessions()` is called
- **THEN** the pool SHALL be removed from memory
