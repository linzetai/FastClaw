## ADDED Requirements

### Requirement: PtySession supports multiple output consumers via broadcast
The `PtySession` SHALL use a `tokio::sync::broadcast::Sender<Vec<u8>>` to distribute PTY output to multiple concurrent consumers. The broadcast channel SHALL have a buffer capacity of 256 messages.

#### Scenario: Two consumers subscribe to the same session
- **WHEN** a frontend WebSocket handler and an Agent tool both subscribe to the same session's output
- **THEN** both receive identical output data independently without blocking each other

#### Scenario: Slow consumer falls behind
- **WHEN** a consumer does not read fast enough and the broadcast buffer is full
- **THEN** the slow consumer receives a `Lagged` error and can skip to the latest available data without affecting other consumers

### Requirement: PTY reader task publishes to broadcast
A dedicated background task SHALL read from the PTY master fd and publish each chunk to the broadcast channel. This replaces the current direct reader clone model.

#### Scenario: PTY output is published to broadcast
- **WHEN** the shell process writes output to the PTY
- **THEN** the reader task reads the data and sends it to the broadcast channel within one read cycle

#### Scenario: PTY process exits
- **WHEN** the shell process terminates and the PTY reader encounters EOF
- **THEN** the reader task stops and all broadcast subscribers receive channel closure

### Requirement: Subscribers can join at any time
New subscribers to the broadcast channel SHALL receive only output produced after their subscription. They SHALL NOT receive historical output.

#### Scenario: Late subscriber
- **WHEN** a subscriber joins after the session has already produced output
- **THEN** the subscriber receives only new output from the point of subscription onward
