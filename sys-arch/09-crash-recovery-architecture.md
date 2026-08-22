# Part 09 — Crash Recovery Architecture

## Reusable P2P Communication Platform

**Status:** Architecture specification  
**Part:** 09 of 24  
**Primary language:** Rust  
**Primary goals:** deterministic restart, crash-safe persistence, idempotent recovery, state reconciliation, partial-operation repair, corruption containment, fault injection, reusable recovery across messaging/files/DTN/identity/daemon/mobile/headless deployments

---

# 1. Purpose

A production communication platform must assume that the process can terminate at any point.

Possible causes:

```text
power loss
kernel crash
application panic
Android process kill
mobile OS background reclaim
desktop reboot
storage I/O error
forced update
SIGKILL
watchdog termination
out-of-memory kill
unexpected device restart
```

The system must therefore be designed around this rule:

> **Any instruction can stop between two machine operations.**

Correct recovery cannot depend on:

```text
destructor ran
shutdown callback completed
network close succeeded
UI saved state
async task reached final line
```

The architecture must guarantee that durable user operations remain recoverable and that ambiguous external side effects are handled idempotently.

---

# 2. Fundamental Recovery Model

```text
Durable Intent
     ↓
Crash-Safe Commit
     ↓
External Effect
     ↓
Durable Result Marker
```

If crash occurs:

```text
before commit
→ operation did not exist durably

after commit, before effect
→ retry effect

after effect, before result marker
→ retry using same idempotency ID

after result marker
→ continue from completed state
```

This is the core crash-recovery pattern.

---

# 3. Recovery Architecture Position

```text
Persistent Stores
├── Event Log
├── Outbox / Work Queues
├── Transfer Journal
├── Blob Store
├── DTN Store
├── Identity Store
└── Configuration

        ↓

Recovery Coordinator

        ↓

Subsystem Reconcilers
├── Messaging
├── Files
├── DTN
├── Identity
├── Routing
├── Capability Cache
└── Daemon/IPC

        ↓

Runtime Resume
```

Recovery must happen before normal background effects resume.

---

# 4. Recovery Coordinator

Create a central coordinator:

```rust
pub struct RecoveryCoordinator {
    // subsystem reconcilers
}
```

Responsibilities:

- detect previous unclean shutdown
- run storage integrity checks
- run schema migrations
- reconcile subsystem durable state
- repair derived state
- invalidate stale ephemeral state
- classify failures
- choose recovery mode
- release runtime only when minimum invariants hold

---

# 5. Recovery Phases

Recommended sequence:

```text
Phase 0 — Open storage safely
Phase 1 — Verify schema/migrations
Phase 2 — Detect unclean shutdown
Phase 3 — Validate critical stores
Phase 4 — Reconcile durable semantic state
Phase 5 — Repair derived work/projections
Phase 6 — Validate file/blob staging
Phase 7 — Reconcile DTN bundles
Phase 8 — Invalidate sessions/capability caches
Phase 9 — Start local read-only/query services
Phase 10 — Resume network/background work
```

Order matters.

---

# 6. Clean Shutdown Marker

Persist:

```text
runtime_started
runtime_clean_shutdown
```

or equivalent generation marker.

On startup:

```text
last start had no clean close
→ unclean startup path
```

Do not depend on this marker for correctness, only for deciding how much recovery work to run.

---

# 7. Crash Recovery Must Be Idempotent

Recovery itself may crash.

Therefore:

```text
run recovery
crash halfway
run again
```

must be safe.

Every recovery step should be:

```text
idempotent
transactional
or resumable
```

---

# 8. Recovery State Machine

```text
Starting
 ↓
StorageOpened
 ↓
IntegrityChecked
 ↓
Reconciling
 ↓
Recovered
 ↓
RuntimeReady
```

Failure branches:

```text
ReadOnlyRecovery
ManualRepairRequired
FatalCorruption
MigrationFailed
StorageUnavailable
```

---

# 9. Storage Transaction Boundaries

Durable state transitions should use database transactions.

Examples:

```text
append event
+
update projection
+
enqueue outbox
```

one transaction where consistency requires.

Do not create recovery problems by splitting logically atomic state unnecessarily.

---

# 10. WAL

For SQLite-like storage, use WAL appropriately.

Benefits:

- atomic commit
- crash recovery
- concurrent readers
- good mobile/desktop performance

Do not manually implement log semantics already provided by database engine unless needed.

---

# 11. WAL Is Not Enough

WAL protects database atomicity.

It does not automatically reconcile:

```text
database row says file exists
but file rename failed
```

Cross-resource operations need explicit recovery design.

---

# 12. Cross-Store Atomicity

You cannot atomically commit:

```text
SQLite transaction
+
filesystem rename
+
network send
```

as one universal transaction.

Use staged state machines.

Example:

```text
Prepare
 ↓
Persist Intent
 ↓
Perform Filesystem Action
 ↓
Persist Completion
```

Recovery checks intermediate state.

---

# 13. Durable Operation State

Every long-lived operation needs explicit states.

Example file:

```text
Created
Preparing
Transferring
Finalizing
Completed
```

Never rely on:

```text
absence of error
```

to infer completion.

---

# 14. Messaging Recovery

Outbound messaging state:

```text
MessageCreated
MessageQueued
OutboxPending
```

On restart:

```text
reload pending outbox
validate expiry/revocation
retry same MessageId
```

Do not create new MessageId.

---

# 15. Ambiguous Send Result

Crash scenario:

```text
network send succeeded
process crashes before local Sent marker
```

On restart:

```text
send same MessageId again
```

Recipient deduplicates.

This is why stable IDs are required.

---

# 16. Delivery Receipt Recovery

If message delivered but receipt not processed:

```text
recipient may resend receipt
sender applies idempotently
```

Do not rely on one-shot receipts.

---

# 17. Inbox Recovery

Remote message should be ACKed according to durable semantics.

For durable receive:

```text
validate
persist
commit
then ACK
```

Crash before ACK:

```text
sender retries
recipient deduplicates EventId/MessageId
```

---

# 18. File Transfer Recovery

Part 05 transfer state includes:

```text
TransferId
BlobId
verified chunk bitmap/ranges
staging path
state
```

On restart:

```text
load transfer
validate staging
recompute/verify uncertain chunks
resume missing chunks
```

---

# 19. Chunk Commit Ordering

Safe receive sequence:

```text
receive chunk
 ↓
write staging bytes
 ↓
flush as policy requires
 ↓
verify hash/authentication
 ↓
persist chunk-complete state
 ↓
ACK chunk
```

Crash between any two steps must be recoverable.

---

# 20. Chunk State Ambiguity

If bytes were written but completion bit not committed:

```text
treat chunk as unverified
reverify on restart
```

Do not assume.

---

# 21. Finalization Recovery

File finalization:

```text
all chunks verified
 ↓
write Finalizing
 ↓
verify full blob/root
 ↓
atomic rename staging → final
 ↓
persist Completed
```

Crash scenarios:

### Crash before rename

Resume finalization.

### Crash after rename before Completed row

Recovery detects final object exists and verifies it.

### Crash after Completed but object missing

This is corruption/inconsistency; repair or downgrade state.

---

# 22. Atomic Rename

Use atomic rename when filesystem supports it.

Do not assume atomicity across filesystems/mounts.

Staging and final store should live on same filesystem when atomic rename is required.

---

# 23. Temp Files

Temporary files must use opaque IDs.

Recovery scans:

```text
staging/
temp/
```

and classifies:

```text
owned by active transfer
orphan
too new
expired
corrupt
```

---

# 24. Orphan Cleanup

Do not immediately delete every unknown temp file after crash.

Use:

```text
ownership metadata
age threshold
transaction generation
```

then clean safely.

---

# 25. Blob Reference Recovery

Reference counts are derived.

If inconsistent:

```text
rebuild from authoritative references
```

Do not trust cached reference count blindly.

---

# 26. Blob GC Crash Safety

GC should use staged deletion.

Example:

```text
mark GC candidate
 ↓
recheck references
 ↓
delete object
 ↓
delete metadata
```

Recovery reconciles partial state.

---

# 27. DTN Recovery

DTN bundle store is durable.

On restart:

```text
load bundles
remove expired
restore replication budget
restore tombstones
resume encounter scheduler
```

Do not reset all bundle states.

---

# 28. Relay ACK Ambiguity

Crash after durable relay storage but before sending relay ACK:

```text
origin resends
relay deduplicates BundleId
relay sends ACK
```

Safe.

---

# 29. Destination ACK Ambiguity

Destination may have committed payload but crash before sending destination ACK.

On next contact/session:

```text
destination resends ACK
```

ACK must be idempotent.

---

# 30. DTN Forward History Recovery

Forward history can prevent immediate bounce.

On restart, preserve enough:

```text
peer
bundle
last forward time
```

Do not require perfect full path history.

---

# 31. Identity Recovery

Part 02 identity state is security critical.

Recovery must verify:

```text
device-event chain
highest generation
revocation set
root continuity
certificate integrity
```

If identity state is inconsistent:

```text
fail closed for trusted network operations
```

---

# 32. Identity Fork Recovery

If two conflicting same-generation account states appear:

```text
do not auto-merge
```

Enter:

```text
IdentityForkDetected
```

and require explicit reconciliation/security flow.

---

# 33. Revocation Recovery

Revocation must never be lost because process crashed after user approved it.

Safe flow:

```text
append signed revocation
commit
invalidate local sessions
propagate asynchronously
```

Crash after commit:

```text
recovery sees revocation
sessions remain invalid
```

---

# 34. Session Recovery

Authenticated network sessions are ephemeral.

After process crash:

```text
invalidate all in-memory session state
```

Do not persist raw live session objects.

---

# 35. Session Resumption Tokens

If protocol later supports resumption:

```text
persist only cryptographically safe resumption material
```

with:

- expiry
- revocation binding
- account generation binding
- secure storage

Start without complex resumption if unnecessary.

---

# 36. Capability Recovery

Part 07 cached peer capabilities are hints only.

After restart:

```text
cache may help UI
new session must renegotiate
```

Do not restore old negotiated contract as authoritative.

---

# 37. Routing Recovery

Part 03 route history is advisory.

Persisted:

```text
last successful transport
known endpoint hints
```

may bootstrap.

Revalidate all route candidates.

---

# 38. Resource Recovery

Part 08 in-memory permits disappear on crash.

After restart:

```text
memory/stream/CPU permits reset
```

Persistent resource reservations:

```text
storage reservations
staging allocations
```

must be reconciled.

---

# 39. Storage Reservation Recovery

For each reservation:

```text
active operation exists?
```

If yes:

```text
restore reservation
```

If no:

```text
expire/release after safety check
```

---

# 40. Outbox Reconciliation

Outbox should be consistent with semantic event state.

On startup:

```text
event says MessageQueued
outbox missing
→ reconstruct

outbox exists
message already Delivered
→ remove
```

This repairs drift.

---

# 41. Projection Recovery

Part 04 projections have checkpoints.

Startup:

```text
read checkpoint
replay committed events after it
```

If projection schema version changed:

```text
rebuild
```

---

# 42. Projection Corruption

If noncritical projection corrupt:

```text
drop/rebuild
```

If critical authoritative store corrupt:

```text
do not continue blindly
```

Classify severity.

---

# 43. Search Index Recovery

Search index is derived.

Recovery can:

```text
mark unavailable
rebuild in background
```

Messaging still works.

---

# 44. Conversation Summary Recovery

If chat-list projection missing:

```text
rebuild from message streams
```

Can prioritize recent streams first for fast startup.

---

# 45. Progressive Recovery

Do not require full years-long rebuild before app becomes usable.

Possible:

```text
recover critical state
 ↓
render recent/local data
 ↓
background rebuild secondary indexes
```

---

# 46. Recovery Priority

Recover first:

```text
identity
security state
event store integrity
pending messages
SOS/critical DTN
active transfers
```

Later:

```text
search
analytics
old thumbnails
```

---

# 47. Read-Only Recovery Mode

If writes are unsafe but reads work:

```text
ReadOnlyRecovery
```

allows:

- view messages
- export data
- inspect devices
- diagnostics

Disallow new durable operations.

---

# 48. Degraded Recovery Mode

Some noncritical subsystem unavailable:

```text
search broken
thumbnail cache broken
```

Runtime can start degraded.

Surface diagnostics.

---

# 49. Fatal Recovery Mode

Examples:

```text
identity key store inaccessible
critical DB unrecoverable
migration partially corrupted
```

Do not start network operations that could make state worse.

---

# 50. Recovery Classification

```rust
pub enum RecoverySeverity {
    Clean,
    Repaired,
    Degraded,
    ReadOnly,
    Fatal,
}
```

---

# 51. Corruption Detection

Mechanisms:

```text
SQLite integrity check
event checksums
blob hashes
manifest authentication
snapshot checksums
identity signatures
```

Use domain-appropriate checks.

---

# 52. Corruption Scope

Classify:

```text
single blob corrupt
one projection corrupt
one event stream corrupt
entire DB corrupt
secure-store failure
```

Do not treat all corruption the same.

---

# 53. Single Blob Corruption

If blob can be reacquired:

```text
mark corrupt
remove final trust
redownload
```

Do not poison whole application.

---

# 54. Transfer Journal Corruption

If transfer journal corrupt but blob chunks are verifiable:

```text
rebuild progress by scanning/verifying chunks
```

possibly slower but safe.

---

# 55. Event Stream Corruption

If immutable authoritative event stream corrupt:

```text
quarantine
attempt restore from backup/peer
```

Do not silently skip events.

---

# 56. Secure Store Failure

If device private keys unavailable:

```text
network identity unavailable
```

Allow local data viewing where possible, but fail trusted communication.

---

# 57. Backup Integration

Recovery can use backups when local repair insufficient.

Backup must be:

```text
authenticated
versioned
integrity-checked
```

---

# 58. Restore Is Not Ordinary Recovery

Restore may resurrect old pending operations.

Therefore after restore:

```text
re-evaluate expiry
delivery state
revocation
peer state
```

before rescheduling effects.

---

# 59. Duplicate Effect Prevention

Every external effect needs stable identity.

Examples:

```text
MessageId
TransferId
BundleId
DeviceEventId
```

Recovery retries with same ID.

---

# 60. Idempotent Receiver

Receiver must tolerate duplicate:

```text
message
chunk
bundle
receipt
device event
```

This is essential for crash safety.

---

# 61. Side Effect Journal

For some operations, keep explicit effect state:

```text
Pending
InFlight
Succeeded
Failed
Deferred
```

Do not rely only on task existence.

---

# 62. InFlight State Semantics

After crash, `InFlight` becomes:

```text
UnknownOutcome
```

Recovery decides:

```text
retry
verify remote state
or wait
```

Do not assume failure.

---

# 63. Unknown Outcome State

```rust
pub enum EffectRecoveryState {
    Pending,
    UnknownOutcome,
    ConfirmedSuccess,
    ConfirmedFailure,
}
```

Useful for non-idempotent external systems.

Within this P2P platform, design protocols to make retries idempotent.

---

# 64. Network Effect Recovery

For network send:

```text
UnknownOutcome
→ retry same ID
```

preferred.

---

# 65. File Export Recovery

Exporting plaintext to user filesystem may be non-idempotent.

Use:

```text
temp destination
atomic commit/rename
```

or explicit user-visible duplicate policy.

---

# 66. Notification Recovery

Do not treat notification display as authoritative state.

If crash causes duplicate notification:

```text
deduplicate by notification/event ID
```

where platform allows.

---

# 67. Android Process Death

Assume app can die immediately after backgrounding.

Therefore:

```text
persist before returning success to UI
```

Long-running work should be represented durably and resumed through allowed platform mechanisms.

---

# 68. Android Foreground Service Recovery

If service stops unexpectedly:

```text
Rust durable state remains source of truth
```

Kotlin service restart asks runtime what work remains.

Do not let Kotlin maintain parallel hidden transfer state.

---

# 69. iOS Recovery

iOS background execution may end unpredictably.

Same principle:

```text
durable state in Rust/storage
platform callback is only execution opportunity
```

---

# 70. Desktop Daemon Recovery

Part 16 daemon owns persistent runtime.

On crash:

```text
system supervisor restarts daemon
recovery runs
GUI reconnects
```

GUI should not attempt to repair DB independently.

---

# 71. IPC Client Recovery

When daemon restarts:

```text
GUI/CLI IPC connection breaks
```

Client:

```text
reconnect
request fresh state snapshot
resubscribe
```

Do not assume incremental event stream continuity.

---

# 72. Headless Node Recovery

Headless relay/server should be able to restart unattended.

Requirements:

```text
no UI prompt for routine recovery
automatic bundle/file/outbox resume
clear logs/health state
```

Fatal key/corruption cases may still require admin intervention.

---

# 73. Supervision

Long-lived workers need supervisor.

```text
Supervisor
├── OutboxWorker
├── TransferScheduler
├── DtnScheduler
├── ProjectionRunner
└── DiscoveryService
```

Worker panic does not crash unrelated state if recoverable.

---

# 74. Worker Restart Policy

```text
bounded exponential backoff
```

Repeated failure:

```text
mark subsystem degraded
stop restart storm
```

---

# 75. Panic Boundaries

Rust panic should not unwind across FFI.

At service boundaries:

```text
catch/report where appropriate
```

Do not use panic as ordinary error control.

---

# 76. Process-Wide Panic Strategy

For unrecoverable invariant violation:

```text
abort/restart
```

may be safer than continuing corrupted shared state.

Persistent recovery then restores from durable state.

---

# 77. OOM Considerations

OOM may abort process without cleanup.

Part 08 resource limits should make OOM less likely.

Crash recovery must still assume:

```text
no cleanup happened
```

---

# 78. Disk Full During Transaction

Database transaction should fail.

Application must not claim operation accepted.

Recovery later may:

```text
evict cache
free temp
retry if user requests
```

---

# 79. Disk Full During Blob Write

Transfer state remains incomplete.

Do not mark chunk complete before durable write succeeded.

---

# 80. Disk Full During Finalization

Keep transfer in:

```text
Finalizing/FailedStorage
```

and allow user action.

---

# 81. Partial Database Migration

Migration architecture should be transactional where possible.

Maintain:

```text
schema version
migration state
```

If crash mid-migration:

```text
resume or rollback deterministically
```

---

# 82. Migration Lock

Only one process performs migration.

Daemon mode simplifies this.

---

# 83. Migration Backup

Before risky destructive migration:

```text
backup/checkpoint
```

according to storage size/policy.

---

# 84. Event Schema vs DB Schema

Event schemas remain immutable.

DB migration may change:

```text
indexes
projection tables
metadata layout
```

without rewriting event meaning.

---

# 85. Recovery Version Compatibility

New binary should understand old durable state.

Do not release incompatible persistence changes without migration path.

---

# 86. Downgrade Compatibility

Running older binary against newer DB may be unsafe.

Use:

```text
min_reader_version
min_writer_version
```

metadata.

Reject unsafe downgrade.

---

# 87. Recovery Metadata

Maintain:

```rust
pub struct RecoveryMetadata {
    pub last_clean_shutdown: Option<Timestamp>,
    pub schema_version: u32,
    pub recovery_generation: u64,
    pub last_recovery_status: RecoverySeverity,
}
```

---

# 88. Recovery Generation

Increment every successful recovery/start generation.

Useful for:

```text
stale temp ownership
diagnostics
reservation cleanup
```

---

# 89. Orphan Resource Ownership

Temp/staging records can include:

```text
created_generation
operation_id
```

Recovery can distinguish stale orphan from current operation.

---

# 90. Stale Locks

Never rely on process-lifetime lock files without stale recovery semantics.

Prefer OS locks or DB locks.

If using lock file:

```text
PID alone is insufficient
```

because PID can be reused.

---

# 91. Monotonic vs Wall Time

Timeout/age decisions across restart use persisted wall time with tolerance.

Within process use monotonic clock.

Do not persist monotonic timestamps directly.

---

# 92. Retry Timers Recovery

Persist:

```text
next_attempt_at
attempt_count
expiry
```

For long-lived durable work.

After restart:

```text
if next_attempt_at passed
→ eligible now
```

---

# 93. Backoff Recovery

Do not reset every retry backoff to zero after crash.

Could cause thundering herd.

Persist coarse retry state where useful.

---

# 94. Thundering Herd Prevention

On restart with 10k pending operations:

```text
do not resume all immediately
```

Use:

```text
priority
jitter
resource admission
batching
```

---

# 95. Recovery Scheduler

```text
Critical pending
 ↓
Interactive
 ↓
Normal
 ↓
Bulk
 ↓
Background
```

Reuse Part 08 fairness.

---

# 96. SOS Recovery

If pending SOS exists and not expired/cancelled:

```text
resume highest priority
```

after identity/storage safety checks.

---

# 97. Revocation Recovery Priority

Identity revocations should be applied before ordinary sends.

A message queued to now-revoked device should be re-evaluated.

---

# 98. Policy Re-Evaluation

Durable pending work may outlive policy changes.

On restart:

```text
recheck current policy
```

Examples:

```text
mobile data now disabled
peer revoked
file size policy changed
```

---

# 99. Capability Re-Evaluation

Old peer capability cache may be stale.

Pending operation can remain queued until new session renegotiates.

Do not assume prior capability.

---

# 100. Routing Re-Evaluation

Every restart reconstructs routes.

Do not persist:

```text
"send via Bluetooth"
```

as durable semantic truth.

Persist:

```text
delivery intent
```

then Part 03 chooses current path.

---

# 101. DTN vs Direct Re-Evaluation

Pending DTN message may later have direct Internet path.

Recovery can:

```text
prefer direct
```

while preserving BundleId/MessageId semantics and deduplication.

---

# 102. Recovery of Multi-Path Transfer

Part 12 future multipath:

```text
persist verified chunk ownership/progress
```

not transient path assignment.

After restart, scheduler can choose new paths.

---

# 103. Recovery of Media Calls

Realtime calls generally do not survive process crash as live sessions.

On restart:

```text
mark call interrupted/ended
```

Do not attempt to resume RTP state blindly.

---

# 104. Call History

Semantic event:

```text
CallInterrupted
```

may be added.

This is different from attempting live media recovery.

---

# 105. Presence Recovery

Presence/typing is ephemeral.

After restart:

```text
discard stale presence
```

Recompute current presence.

---

# 106. Discovery Recovery

Nearby discovery state is ephemeral.

Persist only long-lived trusted peer/endpoints where appropriate.

Rescan after restart.

---

# 107. Capability Dynamic State Recovery

Dynamic hardware/battery capability resets.

Requery platform.

---

# 108. Resource Pressure Recovery

Do not persist:

```text
memory pressure currently critical
```

as authoritative.

Recompute from current environment.

Persistent storage pressure derives from disk state.

---

# 109. Cleanup Order

After unclean shutdown:

```text
1. critical DB integrity
2. identity
3. event/outbox
4. transfer staging
5. DTN
6. cache/temp cleanup
```

Do not aggressively GC before establishing authoritative references.

---

# 110. Quarantine

Suspicious/corrupt artifacts can move to:

```text
quarantine/
```

instead of immediate deletion.

Bound quarantine size.

---

# 111. Manual Repair Tools

Provide CLI/admin tooling:

```text
verify database
verify blobs
rebuild projections
list orphan staging
export diagnostics
```

Do not require user to hand-edit SQLite.

---

# 112. `comm doctor`

A headless diagnostic command could provide:

```text
communication-node doctor
```

checks:

- DB integrity
- event projection lag
- blob store consistency
- DTN store consistency
- key-store accessibility
- schema version
- orphan temp files

---

# 113. Repair Command Safety

Repair operations should be:

```text
dry-run capable
logged
bounded
backup-aware
```

Avoid destructive "fix everything" automatically.

---

# 114. Automatic Repair Policy

Safe automatic:

```text
rebuild cache
rebuild projection
delete expired temp
reverify partial chunk
```

Manual/admin:

```text
drop corrupt authoritative event
reset identity chain
```

---

# 115. Recovery Diagnostics

Store last recovery report:

```rust
pub struct RecoveryReport {
    pub severity: RecoverySeverity,
    pub repaired_items: Vec<RecoveryAction>,
    pub warnings: Vec<RecoveryWarning>,
}
```

Do not include secret material.

---

# 116. User-Facing Recovery UX

Normal:

```text
Recovering your messages…
```

but avoid long blocking if not necessary.

If issue:

```text
Some file transfers were paused and will resume.
```

For serious:

```text
App is in read-only recovery mode.
```

---

# 117. Developer Diagnostics

Show:

```text
Unclean shutdown: yes
Event log: healthy
Projection rebuild: 1200 events
Transfers resumed: 3
Orphan temp cleaned: 7
DTN bundles restored: 12
Identity: verified generation 41
```

---

# 118. Recovery Metrics

Track:

```text
recovery duration
unclean startups
repaired projections
resumed transfers
orphan bytes reclaimed
corruption count
read-only starts
```

---

# 119. Privacy

Recovery logs should not include:

```text
message plaintext
file names unless needed
private keys
full contact graph
```

---

# 120. Crash Injection Testing

This is mandatory.

Inject crashes at deterministic points.

Examples:

```text
before DB commit
after DB commit
before network send
after network send
before result marker
during chunk write
during final rename
during DTN persistence
during projection checkpoint
```

---

# 121. Failpoint Framework

Add internal failpoints:

```rust
failpoint!("after_message_commit");
failpoint!("before_blob_rename");
```

Enabled only in tests/debug builds.

This allows reproducible crash scenarios.

---

# 122. Process-Kill Tests

Do not only simulate exceptions.

Run integration tests that:

```text
start process
perform operation
SIGKILL
restart
assert state
```

This validates real OS/database behavior.

---

# 123. Power-Loss Approximation

Hard to reproduce exactly, but can approximate:

```text
fsync boundaries
kill timing
filesystem fault injection
```

Test on target filesystems where practical.

---

# 124. Database Fault Injection

Simulate:

```text
busy
I/O error
disk full
corrupt page
migration failure
```

---

# 125. Network Fault Injection

Crash combined with:

```text
duplicate send
delayed ACK
lost ACK
peer restart
```

must remain idempotent.

---

# 126. File Fault Injection

Simulate:

```text
partial write
short write
rename failure
permission change
source file disappears
```

---

# 127. DTN Fault Injection

Simulate:

```text
relay stores then crashes
relay ACK lost
destination ACK delayed
tombstone lost and rebuilt
```

---

# 128. Identity Fault Injection

Simulate:

```text
revocation committed then crash
root rotation interrupted
stale state presented
secure store unavailable
```

---

# 129. Projection Fault Injection

Crash:

```text
after projection row update
before checkpoint
```

On restart:

```text
event may replay
projection must remain idempotent
```

---

# 130. Property Tests

Important invariants:

```text
committed durable intent is never lost
uncommitted intent never appears as committed
duplicate effect does not duplicate logical state
completed blob never becomes incomplete due only to restart
revoked device never becomes active after recovery
expired DTN bundle never resumes forwarding
projection rebuild equals committed history
```

---

# 131. Recovery Determinism

Given same durable state, recovery result should be deterministic.

This greatly simplifies debugging.

---

# 132. Recovery Should Not Depend on Network

Minimum local recovery must complete without Internet.

Network sync happens afterward.

This is critical for offline/emergency environments.

---

# 133. Peer-Assisted Repair

After local recovery, peers may help restore:

```text
missing messages
missing blobs
device directory
```

but local system must not require them just to open.

---

# 134. Authoritative vs Reconstructible Data

Classify all storage.

## Authoritative

```text
event log
identity state
user-owned blobs
```

## Reconstructible

```text
search index
conversation summary
route cache
capability cache
thumbnail cache
```

Recovery strategy depends on class.

---

# 135. Storage Catalog

Maintain architecture doc/table for each data set:

```text
authoritative?
rebuild source?
retention?
backup?
encryption?
recovery method?
```

This prevents accidental misuse.

---

# 136. Recovery Ownership

Each subsystem owns its reconciler.

```rust
pub trait RecoverableSubsystem {
    async fn recover(
        &self,
        ctx: &RecoveryContext,
    ) -> Result<SubsystemRecoveryReport, RecoveryError>;
}
```

Coordinator orchestrates order.

---

# 137. Recovery Context

```rust
pub struct RecoveryContext {
    pub generation: u64,
    pub unclean_shutdown: bool,
    pub mode: RecoveryMode,
}
```

---

# 138. Recovery Mode

```rust
pub enum RecoveryMode {
    Normal,
    Thorough,
    ReadOnly,
    Repair,
}
```

Normal startup may run lightweight checks.

Unclean shutdown may run Thorough.

---

# 139. Fast Startup vs Thorough Check

Do not run full database integrity scan every normal launch if too expensive.

Strategy:

```text
clean previous shutdown
→ fast checks

unclean
→ targeted thorough checks

periodic maintenance
→ full checks
```

---

# 140. Periodic Scrubbing

Blob store can periodically verify:

```text
random/sample hashes
```

to detect latent corruption.

Not every startup.

---

# 141. Bit Rot

Long-lived file store may experience corruption.

Content hashes allow detection.

Repair from:

```text
other device
backup
authorized peer
```

if available.

---

# 142. Database Backup Strategy

For SQLite:

```text
online backup API
```

or consistent snapshot method.

Never copy live DB files blindly without understanding WAL.

---

# 143. Backup Recovery Test

Regularly test:

```text
create backup
restore to fresh environment
rebuild projections
verify logical state
```

A backup not tested is not a recovery plan.

---

# 144. Disaster Recovery vs Crash Recovery

Crash recovery:

```text
same device/storage mostly intact
```

Disaster recovery:

```text
device/storage lost
restore from backup/other device
```

Part 09 focuses on crash recovery but must integrate with backup.

---

# 145. Multi-Device Recovery

If one device loses local state:

```text
new/recovered device
```

may bootstrap from:

```text
own trusted device
```

after identity authorization.

Do not assume server backup exists.

---

# 146. Partial History Recovery

A device may restore only:

```text
recent messages
metadata
on-demand blobs
```

Full history is policy-dependent.

---

# 147. Recovery Security

Never accept peer-supplied "repair" state without:

```text
identity
authorization
integrity
version checks
```

Repair paths are security-sensitive.

---

# 148. Rollback Attack Recovery

Attacker restores old local database backup.

Identity/account generation must detect:

```text
stale state
```

if newer trusted state exists elsewhere/local secure metadata.

Part 02 rollback protection applies.

---

# 149. Secure Monotonic Anchor

Where platform supports, store minimal:

```text
highest identity generation
```

in secure storage to make rollback harder.

Optional.

---

# 150. Recovery and Encryption Keys

Recovery must distinguish:

```text
data exists
keys exist
```

Encrypted blob without key is not usable.

Do not mark fully recovered if cryptographic material is missing.

---

# 151. Key Backup Policy

Device private keys may intentionally be non-exportable.

Then device restore may require:

```text
new device identity
```

rather than restoring same key.

Document product semantics.

---

# 152. Session Key Recovery

Do not restore ephemeral session keys after crash unless protocol explicitly supports secure resumption.

Default:

```text
new session handshake
```

---

# 153. Group Key Recovery

Group state may require current epoch keys.

If local group key state missing:

```text
request authorized resync
```

Do not derive from stale membership.

---

# 154. Corrupt Config

Configuration file parse failure should not destroy data.

Use:

```text
last-known-good
defaults
safe mode
```

depending on setting.

---

# 155. Config Transactions

Critical config updates should be atomic:

```text
write temp
fsync
rename
```

or stored transactionally in DB.

---

# 156. RON Config

If using RON:

```text
version config
validate strictly
keep defaults explicit
```

Do not accept malformed config silently.

---

# 157. Postcard Durable Data

Version durable Postcard schemas.

Do not decode old persistent bytes into changed current struct without migration/upcast.

---

# 158. Recovery of Third-Party Extensions

Part 22/24 extensions must declare:

```text
durable state version
recovery hook
resource cleanup
```

One extension failure should not corrupt core.

---

# 159. Extension Quarantine

If extension recovery repeatedly fails:

```text
disable extension
start core degraded
```

where safe.

Do not prevent messaging/files if unrelated custom plugin breaks.

---

# 160. Recovery API Boundary

Core applications should call:

```rust
runtime.recover().await?;
```

not manually orchestrate subsystem repair.

---

# 161. Recovery Report API

```rust
let report = runtime.recovery_report();
```

Useful for UI/admin.

---

# 162. Health State After Recovery

```rust
pub struct RuntimeHealth {
    pub recovery: RecoverySeverity,
    pub storage: HealthState,
    pub identity: HealthState,
    pub messaging: HealthState,
    pub files: HealthState,
    pub dtn: HealthState,
}
```

---

# 163. Admission After Recovery

Before recovery complete:

```text
reject/defer new durable operations
```

Read-only queries may be allowed earlier.

---

# 164. Network Start Barrier

Do not start inbound network listeners before:

```text
identity
critical store
dedup state
```

are ready.

Otherwise duplicate/security handling may be wrong.

---

# 165. Listener Startup Order

Recommended:

```text
identity ready
event store ready
dedup/outbox ready
DTN store ready
then transport listeners
```

---

# 166. Lazy Secondary Recovery

Search/index/cache can continue after network starts if isolated.

---

# 167. Versioned Recovery Protocol

If daemon/GUI versions differ:

```text
IPC schema/version negotiation
```

should prevent GUI from assuming unavailable recovered state.

---

# 168. Rolling Upgrade

Server/headless deployments may restart nodes one by one.

Persisted wire/session state should not require cluster-wide simultaneous upgrade where avoidable.

---

# 169. Recovery and Relay Cluster

Part 11 may add relay cluster state.

User-message E2EE state remains client-side.

Relay crash recovery should restore:

```text
sessions/queues only as designed
```

without plaintext dependence.

---

# 170. Recovery and Multipath

Path assignments are ephemeral.

Persistent transfer state is:

```text
which chunks verified
```

not:

```text
chunk 8 assigned to path Wi-Fi
```

This makes multipath restart simple.

---

# 171. Recovery and Battery Scheduling

After restart, recompute:

```text
battery
charging
thermal
```

then schedule pending work accordingly.

Do not immediately resume everything at maximum throughput.

---

# 172. Recovery and Proximity

Nearby encounter state is transient.

Durable DTN bundle store survives.

When proximity returns:

```text
forwarding resumes
```

---

# 173. Recovery and QR/NFC Linking

If crash during device linking:

```text
link state machine
```

must either:

```text
resume safely
or expire/restart
```

Never leave half-authorized device.

---

# 174. Device Link Commit Point

Only after:

```text
certificate/device event committed
```

is device considered linked.

Ephemeral QR/NFC handshake alone does not authorize.

---

# 175. Recovery and Emergency Priority

Pending emergency events are restored before bulk work.

This ordering must be explicit.

---

# 176. Recovery and Diagnostics

Part 18 can visualize:

```text
what was recovered
what remains degraded
what operation resumed
```

---

# 177. Recovery and FFI

Part 19 host app may crash independently.

Core durable runtime/daemon remains source of truth.

On reconnect, FFI host requests snapshot.

---

# 178. Recovery and WASM

WASM components may use different persistence semantics.

Shared recovery model still applies:

```text
durable intent
idempotent effects
projection rebuild
```

---

# 179. Recovery and Interoperability

Part 23 tests should include:

```text
duplicate after reconnect
retransmit after lost ACK
resume transfer after restart
```

across implementations.

---

# 180. Suggested Crate Structure

```text
crates/comm-recovery/
├── src/
│   ├── lib.rs
│   ├── coordinator.rs
│   ├── phase.rs
│   ├── context.rs
│   ├── report.rs
│   ├── health.rs
│   ├── storage.rs
│   ├── outbox.rs
│   ├── projection.rs
│   ├── files.rs
│   ├── dtn.rs
│   ├── identity.rs
│   ├── cleanup.rs
│   ├── quarantine.rs
│   └── error.rs
└── Cargo.toml
```

---

# 181. Recovery Error Model

```rust
pub enum RecoveryError {
    StorageUnavailable,
    IntegrityFailure,
    MigrationFailure,
    IdentityFailure,
    ProjectionFailure,
    BlobFailure,
    DtnFailure,
    SecureStoreFailure,
    UnsupportedVersion,
    FatalInvariant,
}
```

---

# 182. Initial Production Scope

Implement first:

```text
unclean-shutdown detection
recovery coordinator
outbox reconciliation
projection resume/rebuild
file staging scan
chunk re-verification
blob finalization repair
DTN bundle restore
identity integrity/revocation restore
storage reservation cleanup
ephemeral session invalidation
typed recovery report
fault injection
process-kill tests
```

Defer initially:

```text
complex peer-assisted database repair
automatic authoritative-event surgery
cluster-wide distributed recovery
```

---

# 183. Implementation Phases

## Phase 1 — Recovery Metadata

```text
shutdown marker
recovery generation
health states
```

## Phase 2 — Event/Outbox

```text
projection checkpoints
outbox reconciliation
idempotent resend
```

## Phase 3 — Files

```text
staging scan
chunk verify
finalization recovery
orphan cleanup
```

## Phase 4 — DTN

```text
bundle restore
expiry
tombstones
forward history
```

## Phase 5 — Identity

```text
generation
revocation
fork detection
secure store validation
```

## Phase 6 — Resource Cleanup

```text
storage reservations
stale temp
runtime permits reset
```

## Phase 7 — Hardening

```text
failpoints
SIGKILL tests
disk full
corruption
migration interruption
```

---

# 184. Definition of Done

Part 09 is complete when:

- process can be killed at any documented failpoint without logical corruption
- committed message intent survives restart
- uncommitted message intent does not appear as committed
- ambiguous network send retries with same stable ID
- receiver deduplicates retries
- transfer progress survives process death
- incomplete chunks are reverified rather than trusted blindly
- blob finalization recovers after crash around rename/commit
- DTN relay bundles survive crash
- relay/destination ACK ambiguity is idempotent
- device revocation survives crash immediately
- stale sessions/capabilities/routes are invalidated
- storage reservations and temp files are reconciled
- projections can resume/rebuild
- derived indexes can fail without losing authoritative data
- read-only recovery mode exists
- severe corruption fails closed
- recovery does not require Internet
- recovery itself is idempotent
- process-kill/fault-injection/property tests exist

---

# 185. Relationship to Earlier Parts

Part 09 builds on:

```text
01 — Protocol Extension System
02 — Multi-Device Identity
03 — Transport & Routing Policy Engine
04 — Offline Event Log
05 — Robust File / Blob Subsystem
06 — DTN / Store-Carry-Forward
07 — Capability Negotiation
08 — Resource Limits & Backpressure
```

It directly supports:

```text
10 — Protocol Fuzzing & Test Suite
11 — Relay / Self-Hosted Infrastructure
12 — Multipath Networking
13 — Battery-Aware Scheduling
14 — Proximity Abstraction
15 — QR / NFC Bootstrap Pairing
16 — Daemon & Headless Runtime
17 — Emergency Priority Architecture
18 — Network Diagnostics & Path Visualization
19 — C ABI / FFI
20 — Embedded Linux Node
22 — Third-Party Protocol Extensions
23 — External Interoperability Suite
24 — Plugin / Module Ecosystem
```

---

# 186. Final Principle

The crash-recovery architecture should make this statement true:

> **A crash changes execution timing, not logical truth.**

If the system accepted a durable operation before the crash, recovery finds it.

If an external side effect may have happened, recovery retries safely using the same stable identity.

If only derived state was lost, it is rebuilt.

If authoritative state is corrupt, the system fails closed rather than inventing data.

That is the standard required for a production-grade local-first P2P platform operating on mobile, desktop, headless nodes, and unreliable disaster networks.
