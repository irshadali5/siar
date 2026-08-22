# Part 04 — Offline Event Log Architecture

## Reusable P2P Communication Platform

**Status:** Architecture specification  
**Part:** 04 of 24  
**Primary language:** Rust  
**Primary goals:** durable offline-first state, crash recovery, replayable history, deterministic projections, synchronization, idempotency, auditability, reuse across messaging/files/identity/DTN/emergency/custom products

---

# 1. Purpose

A resilient communication platform must keep working when Internet access, peers, radios, or even the process itself disappear.

The offline event log is the durable local backbone that lets the system accept work first, persist it, and perform remote side effects later.

Core rule:

> **Persist accepted intent before depending on the network.**

The event log supports:

- messaging outbox
- delivery state
- file-transfer lifecycle
- multi-device identity changes
- group membership
- emergency reports
- DTN bundle lifecycle
- synchronization
- future product-specific domains

It must remain reusable and independent of Dioxus, Android, Kotlin, Iroh, or any one database.

---

# 2. Do Not Event-Source Everything

Use an event log where history and recovery matter:

```text
messages
device lifecycle
group membership
transfer lifecycle
emergency state
DTN state
replication state
security-sensitive transitions
```

Use ordinary projections/tables for:

```text
chat-list summaries
search indexes
cached counters
temporary routing metrics
ephemeral presence
```

Recommended model:

```text
Durable Event Journal
        ↓
Projections / Materialized Views
        ↓
UI / Query APIs / Work Queues
```

---

# 3. Command vs Event

A command asks for change:

```text
SendMessage
StartFileTransfer
RevokeDevice
MarkRead
CreateSOS
```

An event records an accepted transition:

```text
MessageQueued
TransferCreated
DeviceRevoked
ReadAdvanced
EmergencyReportCreated
```

Commands may fail.

Committed events are immutable historical facts.

---

# 4. Core Event Envelope

```rust
pub struct EventEnvelope {
    pub event_id: EventId,
    pub stream_id: StreamId,
    pub stream_version: u64,
    pub event_type: EventTypeId,
    pub schema_version: u16,
    pub created_at: Timestamp,
    pub origin: EventOrigin,
    pub correlation_id: Option<CorrelationId>,
    pub causation_id: Option<EventId>,
    pub payload: Bytes,
}
```

Use strong IDs:

```rust
pub struct EventId([u8; 16]);
pub struct StreamId([u8; 32]);
pub struct CorrelationId([u8; 16]);
pub struct EventTypeId(u32);
```

---

# 5. Streams

Events belong to logical streams:

```text
conversation/<id>
transfer/<id>
account/<id>
device/<id>
group/<id>
dtn/<id>
emergency/<id>
```

Each stream has its own monotonic version:

```text
1
2
3
...
```

This gives deterministic local ordering without depending on wall-clock time.

---

# 6. Local Global Offset

Also maintain a device-local append offset:

```rust
pub struct LocalLogOffset(u64);
```

This is useful for:

- projections
- incremental backup
- local replay
- checkpointing

It is never global truth across devices.

---

# 7. Event Origin

```rust
pub enum EventOrigin {
    LocalDevice(DeviceId),
    RemoteDevice(DeviceId),
    Imported,
    Recovery,
    System,
}
```

Remote-origin events are trusted only after identity, authorization, and protocol validation.

---

# 8. Correlation and Causation

Correlation ties one workflow together:

```text
MessageCreated
MessageQueued
MessageSent
MessageDelivered
```

Causation records which event led to which event.

This improves:

- diagnostics
- recovery analysis
- audit trails
- testability

---

# 9. Versioned Event Schemas

Never permanently serialize current domain structs.

Use explicit schemas:

```text
MessageQueuedV1
TransferCreatedV1
DeviceRevokedV1
```

If semantics change:

```text
V1 → V2
```

or upcast old data deterministically.

Do not silently reinterpret old bytes using changed Rust structs.

---

# 10. Append-Only Semantics

Committed events should not be modified in ordinary operation.

Correction occurs through new events:

```text
MessageCreated
MessageEdited
```

not by rewriting the old record.

Storage compaction and legal/privacy deletion are separate retention operations.

---

# 11. Atomic Append

A durable append should be transactional:

```text
BEGIN
  verify expected stream version
  insert event
  update stream head
  assign local offset
COMMIT
```

Failure means no partial logical event.

---

# 12. Optimistic Concurrency

Support:

```rust
append(stream, expected_version, events)
```

If current version differs:

```text
ConcurrencyConflict
```

This protects local concurrent writers.

---

# 13. Local-First Command Flow

Correct:

```text
User action
 ↓
validate
 ↓
append event
 ↓ COMMIT
update projection
 ↓
show UI
 ↓
network effect later
```

Wrong:

```text
try network
 ↓
persist only after success
```

---

# 14. Transactional Outbox

Recommended hybrid:

```text
event log = semantic history
outbox table = efficient pending-work queue
```

One transaction may commit:

```text
MessageQueued
OutboxOperation
message projection
conversation summary
```

If the process dies, the outbox still exists.

---

# 15. Event Log Is Not a Job Queue

Do not scan millions of old events to find work.

Use dedicated projections/work tables for:

```text
outbox
retry schedule
transfer chunks
pending DTN forwards
```

The log preserves meaning. The work queue optimizes execution.

---

# 16. Projection Architecture

```text
Event Journal
    ↓
Projection Runner
    ↓
Materialized Views
```

Examples:

```text
messages
conversation_summary
transfer_state
device_directory
group_state
dtn_bundle_state
emergency_state
```

Projections must be:

- deterministic
- idempotent
- rebuildable
- versioned

---

# 17. Projection Checkpoints

```rust
pub struct ProjectionCheckpoint {
    pub projection_id: ProjectionId,
    pub last_log_offset: LocalLogOffset,
    pub projection_version: u16,
}
```

After restart, a projector resumes from its checkpoint.

---

# 18. Read-Your-Writes

For core UX, accepted local operations should become queryable immediately.

Example:

```text
SendMessage succeeds locally
→ conversation immediately shows message
```

This generally means critical projections update in the same transaction as the event.

---

# 19. Event Store Backend

Recommended initial backend:

```text
SQLite
```

because it provides:

- ACID transactions
- WAL
- recovery
- indexes
- mature mobile/desktop support
- migration tooling

Keep the architecture backend-neutral.

---

# 20. Event Store Trait

```rust
pub trait EventStore: Send + Sync {
    async fn append(
        &self,
        request: AppendRequest,
    ) -> Result<AppendResult, EventStoreError>;

    async fn read_stream(
        &self,
        stream: StreamId,
        from_version: u64,
        limit: usize,
    ) -> Result<Vec<StoredEvent>, EventStoreError>;

    async fn read_log(
        &self,
        from_offset: LocalLogOffset,
        limit: usize,
    ) -> Result<Vec<StoredEvent>, EventStoreError>;
}
```

---

# 21. Batch Append

Support multiple events in one transaction:

```text
MessageCreated
MessageQueued
AttachmentReferenced
```

This is both faster and more consistent.

---

# 22. Integrity

Optional integrity mechanisms:

```text
payload checksum
record checksum
per-stream hash chain
```

Security-critical domains such as device identity may already use signed/hash-chained state from Part 02.

Do not sign every trivial local event unless there is a reason.

---

# 23. Remote Event Ingestion

Safe flow:

```text
receive
 ↓
protocol validation
 ↓
identity verification
 ↓
authorization
 ↓
deduplication
 ↓
domain validation
 ↓
append
 ↓
projection
 ↓
durable ACK
```

For durable delivery semantics, ACK only after persistence.

---

# 24. Idempotency

Remote events may arrive repeatedly.

Require stable:

```text
EventId
```

and enforce uniqueness in storage.

Duplicate valid input should normally become an idempotent no-op.

---

# 25. Out-of-Order Events

Remote events can arrive out of order.

Possible handling:

```text
hold unresolved
request missing predecessor
apply only after dependency satisfied
```

Do not silently discard valid future events.

---

# 26. Gap Detection

If a stream expects:

```text
version 44
```

but receives:

```text
46
```

mark the stream incomplete and request reconciliation.

---

# 27. Logical Clocks

Wall clocks are useful for display, not universal ordering.

Use:

```text
stream version
generation
domain sequence
logical clock
```

where necessary.

A hybrid logical clock can be added later if cross-device approximate chronology becomes valuable.

---

# 28. Offline IDs

IDs must be generated locally without server coordination.

Requirements:

- collision resistant
- offline
- portable
- stable across retries

A time-sortable 128-bit identifier can improve database locality, but correctness must not depend on accurate wall time.

---

# 29. Pure Decision Functions

Where practical:

```rust
fn decide(
    state: &State,
    command: Command,
) -> Result<Vec<DomainEvent>, DomainError>
```

This makes domain logic easy to test.

Effects remain separate.

---

# 30. Effect Processing

Events may schedule external work:

```text
MessageQueued
 → send effect

TransferCreated
 → transfer effect

EmergencyReportCreated
 → routing/DTN effect
```

Effects are asynchronous, retryable, and idempotent.

---

# 31. Exactly-Once Is Not the Goal

Across distributed systems, use:

```text
at-least-once attempts
+
stable IDs
+
receiver deduplication
=
effectively-once logical outcome
```

Do not claim true exactly-once network delivery.

---

# 32. Retry Event Granularity

Do not append one durable event per tiny transport retry.

Permanent history should record meaningful transitions:

```text
MessageDeferred
RouteChanged
TransferPaused
TransferFailed
```

Low-level attempts belong in tracing/metrics.

---

# 33. Messaging Events

Typical semantic events:

```text
MessageCreated
MessageQueued
MessageReceived
MessageDelivered
MessageRead
MessageEdited
MessageDeleted
ReactionAdded
ReactionRemoved
```

---

# 34. File Events

Typical:

```text
TransferCreated
TransferAccepted
TransferStarted
TransferPaused
TransferResumed
TransferCompleted
TransferCancelled
TransferFailed
BlobVerified
```

Chunk completion should usually live in a resumable transfer journal/projection, not as millions of permanent events.

---

# 35. Identity Events

Part 02 naturally maps to durable events:

```text
DeviceAdded
DeviceRevoked
DeviceSuspended
RootRotated
RecoveryUsed
```

These may require signatures and stronger retention.

---

# 36. DTN Events

Possible:

```text
BundleCreated
BundleStored
BundleForwarded
DestinationReached
BundleExpired
BundleEvicted
```

Do not permanently log every proximity encounter.

---

# 37. Emergency Events

Examples:

```text
EmergencyReportCreated
EmergencyReportUpdated
SOSCancelled
AuthorityAlertReceived
EmergencyDeliveryConfirmed
```

Emergency events may have stronger audit and retention policy.

---

# 38. Snapshotting

Large streams can use snapshots:

```text
events 1..10000
 ↓
snapshot @ 10000
 ↓
replay 10001+
```

Snapshot is derived state, not authoritative history.

---

# 39. Snapshot Structure

```rust
pub struct Snapshot {
    pub stream_id: StreamId,
    pub stream_version: u64,
    pub projection_version: u16,
    pub state: Bytes,
    pub checksum: Hash,
}
```

If invalid:

```text
discard
rebuild from log
```

---

# 40. Compaction

Retention is domain-specific.

Possible classes:

```rust
pub enum RetentionClass {
    Permanent,
    SecurityAudit,
    UserControlled,
    OperationalShortTerm,
    Ephemeral,
}
```

Examples:

```text
device revocation → long retention
completed transfer operational noise → compactable
typing status → never journal
```

---

# 41. Privacy and Deletion

Append-only design must not become an excuse to ignore deletion requirements.

Options include:

```text
logical deletion event
projection deletion
physical retention policy
crypto-erasure for sensitive retained payloads
```

Exact behavior depends on product/legal requirements.

---

# 42. Event Encryption

Local database encryption is recommended.

Particularly sensitive event payloads may also be application-encrypted.

Keep cryptographic key management outside event-store internals.

---

# 43. Blob References

Large binary data never belongs directly in the journal.

Events should reference:

```text
BlobId
ContentHash
```

Part 05 will define the blob subsystem.

---

# 44. Search as Projection

Full-text search is a rebuildable projection:

```text
MessageCreated
MessageEdited
MessageDeleted
 ↓
FTS index
```

If search corrupts, rebuild it.

---

# 45. Replay Must Not Re-run Side Effects

Projection replay:

```text
must not resend messages
must not restart transfers
must not rebroadcast SOS
```

Effect execution and projection replay are separate.

---

# 46. Replay Modes

```rust
pub enum ReplayMode {
    ProjectionOnly,
    Recovery,
    Live,
}
```

Only recovery/live mode may schedule external work according to durable pending state.

---

# 47. Startup Recovery

Recommended:

```text
open DB
 ↓
verify migrations
 ↓
resume projections
 ↓
reconcile work queues
 ↓
load local read models
 ↓
render UI
 ↓
start networking
```

No remote connection is required before local state becomes usable.

---

# 48. Work Queue Reconciliation

If an event indicates a pending operation but the optimized work table is missing/corrupt:

```text
recovery reconciler
```

can reconstruct it.

This is a key advantage of retaining semantic history.

---

# 49. Replication Scope

Not every local event should leave the device.

```rust
pub enum ReplicationScope {
    LocalOnly,
    OwnDevices,
    ConversationPeers,
    GroupMembers,
    ExplicitRecipients,
    PublicSigned,
}
```

Examples:

```text
local diagnostic → LocalOnly
read state → OwnDevices/ConversationPeers
device update → OwnDevices + authorized peers
SOS → Explicit/PublicSigned depending mode
```

---

# 50. Own-Device Sync

Suitable event classes:

```text
messages
read state
device state
group state
selected settings
```

Draft sync should remain optional.

---

# 51. Peer and Group Sync

Remote replication must check:

```text
authorization
membership
epoch/version
history visibility policy
```

before sending or appending events.

---

# 52. Local Storage Envelope vs Network Envelope

Do not require:

```text
StoredEvent == WireEvent
```

Recommended:

```text
LocalStoredEvent
      ↓ transform
ReplicationEventV1
      ↓
E2EE / protocol
```

This keeps storage evolution independent from wire compatibility.

---

# 53. Sync Cursors

Where stream ordering supports it:

```rust
pub struct SyncCursor {
    pub peer: DeviceId,
    pub stream: StreamId,
    pub acknowledged_version: u64,
}
```

For more complex sparse state, later add Merkle/set reconciliation.

---

# 54. Conflicts Are Domain-Specific

Do not build one generic conflict resolver.

Examples:

```text
two new messages → both valid
two device revocations → merge conservatively
two profile edits → explicit rule
group membership → authenticated state machine
```

The event store records facts; the domain resolves meaning.

---

# 55. Event Size Limits

Every event type must have a maximum size.

Do not allow:

```text
huge file
huge recursive object
unbounded metadata
```

inside a journal event.

---

# 56. Durability Classes

```rust
pub enum DurabilityClass {
    Critical,
    Durable,
    BestEffort,
}
```

Examples:

```text
DeviceRevoked → Critical
MessageQueued → Durable
typing → not journaled / BestEffort
```

Durability policy must balance safety with mobile battery/performance.

---

# 57. SQL Schema

Conceptual SQLite schema:

```sql
events(
    local_offset INTEGER PRIMARY KEY,
    event_id BLOB UNIQUE NOT NULL,
    stream_id BLOB NOT NULL,
    stream_version INTEGER NOT NULL,
    event_type INTEGER NOT NULL,
    schema_version INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    origin BLOB,
    correlation_id BLOB,
    causation_id BLOB,
    payload BLOB NOT NULL,
    checksum BLOB,
    UNIQUE(stream_id, stream_version)
)
```

Supporting tables:

```text
stream_heads
projection_checkpoints
snapshots
outbox
```

---

# 58. Indexes

Minimum:

```text
event_id
(stream_id, stream_version)
local_offset
```

Avoid indexing everything by default.

---

# 59. Memory Discipline

Replay in bounded batches:

```text
read batch
 ↓
apply
 ↓
discard
 ↓
next batch
```

Never read years of history into memory.

---

# 60. Projection Isolation

One broken secondary projection must not stop durable appends.

Example:

```text
search index projector fails
```

Messaging still works.

Projection catches up later from checkpoint.

---

# 61. Internal Event Notifications

After commit:

```text
wake projectors/effect workers
```

But durability never depends on the in-memory notification.

If the signal is missed, checkpointed readers catch up later.

---

# 62. Unknown Events

Unknown event handling must be explicit.

Possible:

```text
optional unknown → store/ignore safely
required semantic unknown → block stream until upgrade
```

Never deserialize into arbitrary current types.

---

# 63. Namespaced Custom Events

Third-party products may register:

```text
com.example.erp.approval_created
```

through a stable namespace/event registry.

This lets the event log remain reusable.

---

# 64. Multi-Tenant Isolation

For multi-tenant consumers:

```text
TenantId
```

must participate in storage/stream namespace.

One tenant must never see another tenant's streams or projections.

---

# 65. Multiple Identities

Personal/work identities on one device require isolated:

```text
stream namespace
projections
replication policy
outbox
```

Do not accidentally share event state.

---

# 66. Security

Protect against:

```text
malformed imported event
duplicate/replay
rollback
oversized payload
unauthorized remote event
projection poisoning
local corruption
```

Security-critical event domains may use signatures/hash chains.

---

# 67. Event Store Errors

```rust
pub enum EventStoreError {
    ConcurrencyConflict,
    DuplicateEvent,
    StorageFull,
    Corrupt,
    ReadOnly,
    MigrationRequired,
    Io,
    Serialization,
}
```

Keep domain errors separate.

---

# 68. Storage Full Behavior

If storage is full:

```text
do not report "queued"
```

unless the durable append succeeded.

The UI should expose a recoverable storage error.

---

# 69. Read-Only Recovery Mode

If the database is damaged:

```text
read-only mode
```

may allow:

- viewing
- export
- diagnostics

without risking additional corruption.

---

# 70. Backup

Incremental backup can use:

```text
LocalLogOffset
```

to export new events since last backup.

Restore should:

```text
verify
restore log
restore blobs
rebuild projections
reconcile pending work carefully
```

---

# 71. Restore Safety

Do not blindly re-run all previously pending effects after restore.

Check:

```text
expiry
delivery state
revocation
already-completed operation IDs
```

first.

---

# 72. Analytics Separation

Never upload the raw event journal as analytics.

Analytics must use a privacy-filtered projection.

---

# 73. Dioxus Boundary

Dioxus sends:

```text
ApplicationCommand
```

and reads:

```text
ViewModel / projection
```

It never writes event records directly.

---

# 74. Kotlin / iOS Boundary

Platform code may report:

```text
NetworkChanged
AppBackgrounded
LowMemory
```

Rust decides whether a durable domain event is required.

Do not duplicate event semantics in Kotlin or Swift.

---

# 75. Daemon Compatibility

Part 16 may move event ownership into a daemon.

Then:

```text
daemon = sole writer
GUI/CLI = command/query clients
```

Same event APIs should support in-process and daemon modes.

---

# 76. Headless Compatibility

A headless node can use the log for:

```text
file transfers
DTN bundles
identity state
relay operations
```

without any UI.

---

# 77. Routing Integration

Part 03 routing consumes pending durable work.

Typical flow:

```text
MessageQueued
 ↓
router selects path
 ↓
send attempt
 ↓
meaningful result event
```

Do not persist every RTT/path probe as semantic history.

---

# 78. File Integration

Part 05 should use:

```text
semantic transfer events
+
high-frequency chunk journal/projection
```

not one permanent event per chunk.

---

# 79. DTN Integration

Part 06 should persist:

```text
bundle creation
store
forward
delivery
expiry
```

while keeping peer-encounter telemetry mostly operational.

---

# 80. Emergency Integration

Emergency reports must be persisted **before** radio/network attempts.

Example:

```text
SOS created
 ↓ COMMIT
device loses connectivity
 ↓
restart
 ↓
SOS still pending until delivered/expired/cancelled
```

---

# 81. Diagnostics

Expose:

```text
latest local offset
event-store health
projection lag
pending effect count
last successful compaction
```

without exposing content.

---

# 82. Metrics

Useful local metrics:

```text
append latency
commit latency
events/sec
projection lag
replay rate
event-store size
duplicate rate
projection failures
```

---

# 83. Property Tests

Important invariants:

```text
duplicate event does not duplicate projection state
projection rebuild equals live projection
stream versions strictly increase
failed transaction creates no partial event
checkpoint never advances past committed projection
expired pending work is not resurrected
```

---

# 84. Crash Injection Tests

Inject process failure:

```text
before append
inside transaction
after event before projection
after projection before network effect
after network effect before success marker
```

Then verify deterministic recovery.

---

# 85. Fuzzing

Fuzz:

```text
event envelope decoder
event payload decoders
snapshot decoder
backup/import
event upcasters
```

All allocations must be bounded.

---

# 86. Golden Event Tests

Stable event schemas should have golden encodings.

Changing bytes for a stable schema should require explicit protocol/storage review.

---

# 87. Recovery Acceptance Test

```text
MessageQueued committed
process killed
restart
projection restored
outbox reconstructed
route found
same MessageId sent
recipient deduplicates
MessageDelivered committed
```

This is a key production gate.

---

# 88. Multi-Device Offline Test

```text
Phone offline
Laptop offline

Phone appends events
Laptop appends independent events

Later reconnect
 ↓
authorized events replicate
 ↓
domain conflict rules apply
 ↓
both converge
```

No central clock is required.

---

# 89. Suggested Crate Structure

```text
crates/comm-event-log/
├── src/
│   ├── lib.rs
│   ├── event.rs
│   ├── envelope.rs
│   ├── stream.rs
│   ├── store.rs
│   ├── append.rs
│   ├── codec.rs
│   ├── registry.rs
│   ├── projection.rs
│   ├── snapshot.rs
│   ├── checkpoint.rs
│   ├── retention.rs
│   ├── replay.rs
│   ├── diagnostics.rs
│   └── error.rs
└── Cargo.toml
```

Optional backend crate:

```text
comm-event-log-sqlite
```

---

# 90. Public API

Keep small:

```text
EventStore
EventAppender
EventReader
ProjectionRunner
SnapshotStore
```

Most products should use domain services rather than appending arbitrary events directly.

---

# 91. Initial Production Scope

Implement first:

```text
SQLite event store
stream versioning
global local offset
unique event IDs
batch append
critical projections
outbox integration
projection checkpoints
replay
basic snapshots
schema versioning
```

Defer initially:

```text
Merkle replication
complex crypto-erasure
segment-file backend
full compliance mode
```

---

# 92. Implementation Phases

## Phase 1

```text
EventId
StreamId
EventEnvelope
LocalLogOffset
EventStore trait
```

## Phase 2

```text
SQLite backend
events
stream heads
transactions
indexes
```

## Phase 3

```text
messaging
files
identity integration
```

## Phase 4

```text
projections
checkpoints
rebuild
```

## Phase 5

```text
outbox
effects
retry
recovery
```

## Phase 6

```text
replication scopes
stream heads
sync hooks
```

## Phase 7

```text
snapshots
compaction
migration tests
crash injection
fuzzing
benchmarks
```

---

# 93. Definition of Done

Part 04 is complete when:

- accepted local commands survive process death
- operations can be accepted without Internet
- IDs require no central server
- stream versions provide deterministic ordering
- duplicate remote events are idempotent
- projections rebuild deterministically
- checkpoints recover after crashes
- UI reads optimized projections
- large data is referenced via blobs
- message outbox survives restart
- file semantic state survives restart
- device lifecycle remains auditable
- SOS is persisted before transmission
- DTN lifecycle is durable
- replication scope is explicit
- event schemas are versioned
- replay never accidentally re-runs external effects
- storage-full is handled safely
- no external side effect occurs before durable commit
- crash/property/fuzz tests exist
- the subsystem works outside the messenger

---

# 94. Relationship to Other Parts

Part 04 builds on:

```text
01 — Protocol Extension System
02 — Multi-Device Identity
03 — Transport & Routing Policy Engine
```

It directly supports:

```text
05 — Robust File / Blob Subsystem
06 — DTN / Store-Carry-Forward
07 — Capability Negotiation Expansion
08 — Resource Limits & Backpressure
09 — Crash Recovery
10 — Protocol Fuzzing & Test Suite
13 — Battery-Aware Scheduling
16 — Daemon & Headless Runtime
17 — Emergency Priority Architecture
18 — Network Diagnostics & Path Visualization
20 — Embedded Linux Node
23 — External Interoperability Suite
```

---

# 95. Final Principle

The offline event log should make this guarantee true:

> **If the application tells the user that a durable operation has been accepted locally, that intent survives network loss and process termination.**

The architecture is:

```text
User action
 ↓
validate
 ↓
append durable event
 ↓ COMMIT
update projection
 ↓
show local result
 ↓
perform network/file/DTN effects asynchronously
```

This is the foundation that turns the platform into a genuinely local-first, crash-recoverable, offline-capable communication system rather than an online application that merely caches some state.
