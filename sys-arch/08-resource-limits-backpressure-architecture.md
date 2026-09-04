# Part 08 — Resource Limits & Backpressure Architecture

## Reusable P2P Communication Platform

**Status:** Architecture specification  
**Part:** 08 of 24  
**Primary language:** Rust  
**Primary goals:** bounded resource usage, memory safety under load, deterministic overload behavior, backpressure propagation, fairness, admission control, peer quotas, extension isolation, mobile/embedded efficiency, resilience against abuse and accidental overload

---

# 1. Purpose

A production communication platform must remain stable when:

- many peers connect simultaneously
- one peer sends too much data
- a large file transfer saturates memory
- Bluetooth becomes slow
- DTN storage fills
- a call consumes bandwidth
- the device enters low-memory mode
- Android restricts background execution
- disk becomes nearly full
- third-party extensions behave aggressively
- network conditions collapse
- a malicious peer attempts resource exhaustion

The platform must never rely on:

```text
"we probably won't receive that much traffic"
```

Instead, every subsystem must have explicit, bounded resource contracts.

The core rule is:

> **Every queue, buffer, stream, cache, task class, peer, extension, and storage pool must have a defined upper bound or admission policy.**

When a limit is reached, the system must:

```text
slow producers
defer work
reject work
shed low-priority work
or downgrade quality
```

rather than growing without bound.

---

# 2. Fundamental Architecture

```text
Application
    ↓
Feature Layer
    ↓
Resource Admission
    ↓
Bounded Queues
    ↓
Scheduler / Fairness
    ↓
Transport / Storage / CPU
```

Feedback flows upward:

```text
Transport/Storage Pressure
         ↑
Scheduler
         ↑
Backpressure Signal
         ↑
Producer
```

This architecture must be shared across:

```text
messaging
files
DTN
routing
calls
presence
sync
daemon IPC
plugins/extensions
```

---

# 3. Resource Dimensions

The platform must reason about multiple resource dimensions independently.

Primary dimensions:

```text
Memory
CPU
Storage
Network bandwidth
Concurrent connections
Concurrent streams
Queue depth
File descriptors
Background execution budget
Battery/energy
Thermal headroom
```

Do not collapse all pressure into one generic "busy" flag.

---

# 4. Resource Classes

```rust
pub enum ResourceKind {
    Memory,
    Cpu,
    Storage,
    Network,
    Connections,
    Streams,
    QueueSlots,
    FileDescriptors,
    Energy,
    Thermal,
}
```

Different limits require different reactions.

---

# 5. Resource Policy Layers

Policies should stack:

```text
System hard limits
      ↓
Runtime defaults
      ↓
Application limits
      ↓
Extension limits
      ↓
Peer limits
      ↓
Operation limits
```

A lower layer can tighten limits, but cannot exceed hard safety ceilings.

---

# 6. Hard Safety Limits

Examples:

```text
max frame size
max event size
max manifest size
max concurrent streams
max memory buffers
max DTN storage
max pending outbox operations
```

These protect correctness and availability.

They are not user preferences.

---

# 7. Soft Limits

Soft limits may trigger adaptation:

```text
reduce parallelism
pause background transfers
lower video bitrate
defer sync
evict cache
```

before hard rejection.

---

# 8. Resource Budget

Use explicit resource budgets.

```rust
pub struct ResourceBudget {
    pub memory_bytes: u64,
    pub storage_bytes: u64,
    pub network_bytes_per_sec: Option<u64>,
    pub max_connections: u32,
    pub max_streams: u32,
    pub max_tasks: u32,
}
```

Not every runtime can know exact CPU/energy quantities; coarse classes are acceptable.

---

# 9. Global Runtime Budget

Each `CommunicationRuntime` owns one global budget.

```text
CommunicationRuntime
  ├── Messaging
  ├── Files
  ├── DTN
  ├── Calls
  └── Extensions
```

Subcomponents consume from shared global pools.

This prevents each subsystem from assuming it owns the whole machine.

---

# 10. Sub-Budgets

Example:

```text
Global memory budget: 256 MiB

Files:       96 MiB
Calls:       64 MiB
Messaging:   32 MiB
DTN:         32 MiB
Other:       32 MiB
```

Actual values vary by device class.

Unused budget may be borrowable under policy.

---

# 11. Budget Borrowing

Allow controlled temporary borrowing:

```text
Files borrows from unused Messaging pool
```

but never beyond global hard ceiling.

Borrowing should be revocable under pressure.

---

# 12. Device Resource Profiles

Recommended profiles:

```rust
pub enum ResourceProfile {
    Embedded,
    MobileLow,
    MobileNormal,
    Desktop,
    Server,
}
```

Each profile provides sensible defaults.

---

# 13. Embedded Profile

Example characteristics:

```text
small memory
few streams
low concurrency
small caches
limited relay storage
```

Part 20 can specialize further.

---

# 14. Mobile Low Profile

Optimized for:

```text
4 GB RAM class device
background limits
battery sensitivity
```

---

# 15. Desktop Profile

Allows:

```text
more parallel transfers
larger caches
more connections
```

without becoming unbounded.

---

# 16. Server Profile

Allows:

```text
high concurrency
large relay store
many peers
```

with stricter per-tenant/peer quotas.

---

# 17. Bounded Queue Principle

Every queue must declare capacity.

Bad:

```rust
unbounded_channel()
```

for arbitrary network/user data.

Good:

```rust
bounded_channel(capacity)
```

with documented overflow behavior.

---

# 18. Queue Categories

Typical queues:

```text
control
interactive
bulk
background
DTN
notification
projection
IPC
```

Each should have independent capacity.

---

# 19. Priority Queues

Recommended classes:

```rust
pub enum WorkPriority {
    Critical,
    Control,
    Interactive,
    Normal,
    Bulk,
    Background,
}
```

This aligns with Parts 03 and 06.

---

# 20. Queue Capacity by Priority

Reserve slots for higher-priority traffic.

Example:

```text
Critical reserve
Control reserve
Shared normal pool
Bulk limited
Background limited
```

Bulk traffic must not consume capacity needed for receipts/SOS/control frames.

---

# 21. Backpressure Semantics

Backpressure means:

```text
consumer slower than producer
```

The producer must not continue indefinitely.

Possible responses:

```text
await capacity
return Busy
defer operation
drop stale data
reduce production rate
```

Behavior depends on work class.

---

# 22. Durable vs Ephemeral Backpressure

Durable operation:

```text
message/file
```

should generally:

```text
persist
defer
retry later
```

Ephemeral operation:

```text
typing indicator
video frame
presence update
```

can be dropped when stale.

---

# 23. Backpressure Result

```rust
pub enum AdmissionResult {
    Accepted,
    Deferred(DeferredReason),
    Rejected(RejectReason),
    Dropped(DropReason),
}
```

Do not hide overload behind silent failures.

---

# 24. Admission Control

Every expensive operation passes admission before resource allocation.

Example:

```text
incoming file offer
 ↓
check file size
storage reservation
peer quota
global transfer slots
memory budget
 ↓
accept/reject/defer
```

---

# 25. Admission Controller

```rust
pub trait AdmissionController {
    fn admit(
        &self,
        request: &ResourceRequest,
        snapshot: &ResourceSnapshot,
    ) -> AdmissionDecision;
}
```

Keep decision deterministic.

---

# 26. Resource Request

```rust
pub struct ResourceRequest {
    pub owner: ResourceOwner,
    pub priority: WorkPriority,
    pub memory: u64,
    pub storage: u64,
    pub streams: u32,
    pub connections: u32,
    pub bandwidth_class: BandwidthClass,
    pub durable: bool,
}
```

---

# 27. Resource Owner

```rust
pub enum ResourceOwner {
    Core,
    Messaging,
    Files,
    Dtn,
    Calls,
    Extension(ProtocolId),
    Peer(DeviceId),
}
```

This enables accounting.

---

# 28. Hierarchical Accounting

Charge usage to:

```text
runtime
 ↓
extension
 ↓
peer
 ↓
operation
```

Example:

```text
Files
  Bob
    Transfer 123
```

This makes quota enforcement explainable.

---

# 29. Per-Peer Quotas

Recommended limits:

```text
max active streams
max pending messages
max inbound bytes
max staged files
max DTN relay bytes
max requests/sec
```

Unknown peers get smaller defaults.

---

# 30. Trust-Aware Quotas

Quota classes:

```text
Unknown
Known
Verified
Organization
Authority
LocalOwnDevice
```

Higher trust can increase quotas.

Never make trusted peers unlimited.

---

# 31. Per-Extension Quotas

Third-party/custom extensions must not monopolize resources.

Each extension receives:

```text
memory budget
queue budget
stream budget
storage budget
CPU work budget
```

---

# 32. Extension Admission

Extension registration may declare:

```rust
pub struct ExtensionResourceLimits {
    pub max_memory_bytes: u64,
    pub max_queued_ops: u32,
    pub max_streams: u16,
    pub max_storage_bytes: u64,
}
```

Runtime can tighten these.

---

# 33. CPU Budgeting

Precise CPU quotas are difficult in-process.

Use practical controls:

```text
max concurrent CPU-heavy jobs
worker semaphore
task priority
batch size
yield points
```

Examples:

```text
hashing
AV1 software encode
image processing
compression
```

---

# 34. CPU Work Classes

```rust
pub enum CpuWorkClass {
    Critical,
    Interactive,
    Bulk,
    Background,
}
```

Use separate semaphores/pools when needed.

---

# 35. Blocking Work Pool

CPU-heavy/blocking operations should not run on the async executor core threads.

Use bounded:

```text
blocking/compute pool
```

with admission.

---

# 36. Hashing Concurrency

Large-file hashing should be bounded.

Do not hash 20 multi-gigabyte files concurrently because 20 transfers arrived.

---

# 37. AV1 Software Encoding

Software AV1 can consume substantial CPU.

Media subsystem must acquire:

```text
CPU media permits
```

and degrade quality if unavailable.

---

# 38. Memory Pool

For high-throughput paths, use bounded buffer pools.

```rust
BufferPool {
    max_bytes,
    block_sizes,
}
```

Borrowing buffer waits or fails according to priority.

---

# 39. Buffer Ownership

Prefer explicit RAII permits:

```rust
let permit = memory_pool.acquire(bytes).await?;
```

Dropping permit releases accounting.

This reduces leaks.

---

# 40. Memory Permit

```rust
pub struct MemoryPermit {
    bytes: u64,
    owner: ResourceOwner,
}
```

No manual "release()" ideally.

---

# 41. Stream Permit

Same pattern:

```rust
let stream_permit = stream_limiter.acquire(peer).await?;
```

Release on drop.

---

# 42. Connection Limits

Global:

```text
max active peer sessions
```

Per peer:

```text
max parallel connections
```

Transport should generally reuse sessions.

---

# 43. File Descriptor Limits

Desktop/server may exhaust OS FDs.

Bound:

```text
open blob readers
open staging files
sockets
watchers
```

Use pooled/closed resources.

---

# 44. Storage Budgets

Separate:

```text
durable user data
cache
staging
relay
temporary
```

Do not use one undifferentiated disk quota.

---

# 45. Storage Classes

```rust
pub enum StorageClass {
    Durable,
    Cache,
    Staging,
    Relay,
    Temporary,
}
```

Each class has:

```text
hard quota
soft watermark
eviction policy
```

---

# 46. Storage Watermarks

Example:

```text
Normal < 70%
Elevated 70–85%
Critical 85–95%
Full > 95%
```

Exact thresholds configurable.

---

# 47. Storage Pressure Actions

Normal:

```text
full functionality
```

Elevated:

```text
reduce prefetch
start cache cleanup
```

Critical:

```text
pause background bulk
reject large relay bundles
aggressive cache eviction
```

Full:

```text
accept only minimal critical durable data if reserved space exists
```

---

# 48. Critical Storage Reserve

Reserve bytes for:

```text
identity changes
small messages
SOS
delivery ACKs
security events
```

Bulk files must not consume this reserve.

---

# 49. Storage Reservation

Before accepting large file:

```text
reserve expected bytes
```

This avoids overcommit.

Reservation itself must expire if transfer never starts.

---

# 50. Storage Reservation Record

```rust
pub struct StorageReservation {
    pub id: ReservationId,
    pub bytes: u64,
    pub owner: ResourceOwner,
    pub expires_at: Timestamp,
}
```

Persist if required for crash recovery.

---

# 51. Network Bandwidth Budget

Traffic classes:

```text
Realtime
Interactive
Normal
Bulk
Background
```

Calls and control traffic should receive reserved capacity.

---

# 52. Traffic Shaping

Use token-bucket or scheduler-based shaping.

Example:

```text
Bulk capped while call active
Background capped always on metered
```

---

# 53. Token Bucket

Conceptual:

```rust
TokenBucket {
    rate,
    burst,
}
```

Useful for:

```text
per-peer bytes/sec
per-extension bandwidth
unknown-peer intake
```

---

# 54. Bandwidth Fairness

One transfer must not monopolize link.

Use:

```text
weighted fair scheduling
```

across peers/extensions/priority classes.

---

# 55. Weighted Fair Queueing

Example weights:

```text
Control 8
Interactive 6
Normal 4
Bulk 2
Background 1
```

Critical traffic gets special bounded preemption.

---

# 56. Strict Priority Risks

Pure strict priority can starve bulk forever.

Use:

```text
weighted fairness
+
critical reserve
```

rather than absolute priority for all classes.

---

# 57. Emergency Preemption

Emergency can temporarily preempt:

```text
background sync
bulk file chunks
relay cache work
```

but not violate hard safety/resource bounds.

---

# 58. Realtime Drop Policy

Realtime media frames become useless after deadline.

If queue full:

```text
drop stale frame
```

not:

```text
block and accumulate seconds of latency
```

---

# 59. Typing/Presence Drop Policy

Ephemeral state should coalesce.

Example:

```text
typing=true
typing=true
typing=true
```

keep latest state, not all events.

---

# 60. Coalescing

Useful for:

```text
progress updates
presence
network metrics
typing
capability deltas
```

Do not queue every redundant update.

---

# 61. Latest-Value Channel

For state-like updates use:

```text
watch/latest-value
```

rather than FIFO event queue where appropriate.

---

# 62. Messaging Backpressure

If outbound message queue is full:

```text
persist message
mark deferred
```

Do not lose durable message.

UI can show:

```text
Waiting to send
```

---

# 63. Message Inbound Limit

Incoming messages must obey:

```text
max message size
max messages/sec
max pending unprocessed
```

Unknown peers stricter.

---

# 64. File Backpressure

File reader must slow when:

```text
network queue full
encryptor busy
memory pool exhausted
```

Do not read ahead unboundedly.

---

# 65. Transfer Slots

Global:

```text
max active transfers
```

Per peer:

```text
max active transfers per peer
```

Extra transfers enter durable deferred state.

---

# 66. DTN Backpressure

DTN relay intake checks:

```text
relay quota
priority
peer quota
storage pressure
```

Low-priority bundles may be rejected/evicted first.

---

# 67. DTN Encounter Budget

During short contact:

```text
max bytes to exchange
max bundle count
```

This prevents one peer from consuming entire encounter.

---

# 68. Routing Backpressure

Routing should not create unlimited path acquisition attempts.

Limit:

```text
concurrent connects
active discovery jobs
retry timers
```

---

# 69. Connection Storm Protection

If 1000 peers appear:

```text
do not connect to all simultaneously
```

Use admission + priority + queue.

---

# 70. Discovery Backpressure

Nearby discovery results can be noisy.

Coalesce/update peer state instead of queueing every advertisement.

---

# 71. Capability Negotiation Limits

Part 07 negotiation must bound:

```text
capability count
parameter bytes
concurrent negotiations
```

A peer cannot hold unlimited negotiation sessions.

---

# 72. Event Log Backpressure

Part 04 append path needs:

```text
max batch size
bounded writer queue
projection lag thresholds
```

If storage cannot keep up, producers receive backpressure.

---

# 73. Projection Lag

Define:

```text
Healthy
Lagging
Critical
```

If critical projection lags too far:

```text
slow writes
shed secondary work
```

Do not let projection memory backlog grow without bound.

---

# 74. Outbox Backpressure

Outbox can grow durably, but must have:

```text
max count
max bytes
retention/expiry
```

If user queues too much offline:

```text
defer/reject new bulk
```

while preserving critical messaging reserve.

---

# 75. Daemon IPC Backpressure

Part 16 daemon clients must not receive unbounded event streams.

Per client:

```text
bounded IPC send queue
snapshot/resync mechanism
```

If GUI falls behind:

```text
drop/coalesce noncritical updates
request fresh snapshot
```

---

# 76. FFI Backpressure

Part 19 callbacks must not be called infinitely fast.

Use:

```text
poll API
bounded event queue
coalescing
```

depending on host language.

---

# 77. Plugin Backpressure

Part 24 plugins/extensions receive:

```text
bounded mailboxes
resource budgets
timeouts
```

A slow plugin cannot block core runtime.

---

# 78. Admission by Cost Estimate

Operations declare expected cost.

Example:

```text
5 GB file
```

can be rejected before hashing if:

```text
staging quota insufficient
```

---

# 79. Cost Estimate Accuracy

Estimates may be imperfect.

Use:

```text
reservation
+
runtime accounting
```

to correct.

Do not trust application-supplied estimate blindly.

---

# 80. Resource Snapshot

```rust
pub struct ResourceSnapshot {
    pub memory: ResourceUsage,
    pub storage: StorageUsage,
    pub connections: CountUsage,
    pub streams: CountUsage,
    pub network: NetworkUsage,
    pub pressure: PressureState,
}
```

---

# 81. Pressure State

```rust
pub enum PressureState {
    Normal,
    Elevated,
    Critical,
    Exhausted,
}
```

This is generic presentation.

Subsystem-specific actions remain typed.

---

# 82. Memory Pressure Sources

Platform may report:

```text
Android trim memory
iOS memory warning
OS low-memory
internal budget exhaustion
```

Combine into runtime pressure state.

---

# 83. Memory Pressure Response

Elevated:

```text
shrink caches
reduce read-ahead
```

Critical:

```text
pause bulk
release previews
reduce buffer pools
drop ephemeral state
```

Exhausted:

```text
reject new noncritical work
preserve core control
```

---

# 84. Android Memory Integration

Kotlin reports:

```text
trim-memory level
```

Rust translates into policy.

Do not duplicate resource policy in Kotlin.

---

# 85. iOS Memory Integration

Platform adapter reports memory warnings.

Rust performs same shared degradation logic.

---

# 86. CPU Pressure

Detect via:

```text
worker queue depth
task latency
thermal signal
media encoder overload
```

Use coarse pressure state.

---

# 87. CPU Pressure Response

```text
reduce parallel hashes
lower AV1 encode complexity
pause background compression
reduce indexing concurrency
```

---

# 88. Thermal Pressure

Thermal state can force:

```text
lower media quality
reduce Wi-Fi heavy transfer
reduce CPU jobs
```

Critical communication remains available.

---

# 89. Battery/Energy Budget

Part 13 will define detailed policy.

Part 08 needs general integration:

```text
background work budget
radio activation budget
CPU-heavy work budget
```

---

# 90. Resource Reservation Hierarchy

For operation:

```text
admit
 ↓
reserve
 ↓
execute
 ↓
release
```

Reservations should be RAII where possible.

---

# 91. Multi-Resource Reservation

Some operations need multiple resources atomically.

Example file transfer:

```text
1 transfer slot
2 stream slots
8 MiB memory
500 MiB storage reservation
```

Avoid deadlocks from acquiring resources in arbitrary order.

---

# 92. Reservation Order

Define canonical order:

```text
Global admission
Storage
Memory
Connections
Streams
CPU
```

or use central allocator that grants bundle atomically.

---

# 93. Deadlock Avoidance

Never:

```text
hold memory permit
wait forever for stream permit
while another task holds stream and waits for memory
```

Use:

- fixed acquisition order
- timeout
- combined reservation
- release-and-retry

---

# 94. Resource Permit Bundle

```rust
pub struct ResourcePermitBundle {
    pub memory: Option<MemoryPermit>,
    pub storage: Option<StoragePermit>,
    pub streams: Vec<StreamPermit>,
    pub task: Option<TaskPermit>,
}
```

---

# 95. Permit Expiry

Reservations before work starts may expire.

Active permits tied to live operation.

---

# 96. Cancellation Safety

When operation is cancelled:

```text
drop permits
cancel tasks
release reserved storage if unused
```

RAII simplifies this.

---

# 97. Panic Safety

If task panics:

```text
permits release on drop
```

Do not require explicit cleanup only.

Core runtime should catch boundary panics where appropriate.

---

# 98. Fair Semaphores

Use fairness to prevent one task repeatedly reacquiring permits.

Tokio/futures primitives may need wrapper behavior depending on guarantees.

---

# 99. Starvation Detection

Track wait time.

If low-priority task waits too long:

```text
age its priority slightly
```

without outranking critical safety work.

---

# 100. Aging

```text
effective priority = base + bounded wait-age boost
```

Useful for bulk transfer fairness.

---

# 101. Load Shedding

When critically overloaded:

```text
drop/reject low-value work
```

Priority order:

```text
ephemeral
background
bulk
normal
interactive
control
critical
```

Durable work should be deferred rather than silently dropped where possible.

---

# 102. Shed Examples

Drop:

```text
old typing state
stale presence
old diagnostics update
stale video frame
```

Defer:

```text
large file
background sync
thumbnail prefetch
```

Preserve:

```text
device revocation
SOS
small text
delivery ACK
```

---

# 103. Overload Mode

```rust
pub enum OverloadMode {
    Normal,
    Constrained,
    CriticalOnly,
}
```

Runtime can switch based on combined pressure.

---

# 104. Critical-Only Mode

Accept only:

```text
security events
small critical messages
SOS
essential ACKs
shutdown/recovery control
```

when resources are exhausted.

---

# 105. User Feedback

Normal users should see:

```text
Waiting for storage
Waiting for Wi-Fi
Device is low on storage
Background transfer paused
```

not internal queue codes.

---

# 106. Developer Diagnostics

Advanced view:

```text
Memory: 72/128 MiB
Streams: 14/32
Transfers: 3/4
DTN relay: 420/512 MiB
Bulk queue: 48/64
Pressure: Elevated
```

---

# 107. Resource Metrics

Track:

```text
current usage
peak usage
admission rejects
deferred count
queue wait
permit wait
evictions
shed operations
```

---

# 108. Cardinality Discipline

Do not export metrics with unbounded labels like raw peer ID.

Use:

```text
trust class
extension
priority
resource class
```

---

# 109. Per-Peer Diagnostics

Local debugging can inspect one peer specifically.

Do not export such detail to telemetry by default.

---

# 110. Resource Leak Detection

Debug builds/testing can track:

```text
permits not released
tasks still alive
staging reservations orphaned
```

---

# 111. Timeout Policy

Resource waits should have operation-specific timeout.

Examples:

```text
video frame: milliseconds
text send admission: seconds/local persist
background file: may wait indefinitely durably
```

---

# 112. Wait vs Fail

Durable background transfer:

```text
wait/defer
```

Interactive call setup:

```text
fail fast with fallback
```

Policy varies by class.

---

# 113. Messaging Limits

Recommended configurable limits:

```text
max message bytes
max attachments
max pending outbound messages
max inbound rate per peer
max unresolved remote events
```

---

# 114. File Limits

```text
max file size
max chunk size
max parallel chunks
max active transfers
max staging bytes
max manifest entries
```

---

# 115. DTN Limits

```text
max bundle size
max relay bytes
max bundles
max replication budget
max hop limit
max peer intake/sec
```

---

# 116. Routing Limits

```text
max candidates per destination
max concurrent path acquisitions
max retry timers
max discovery escalations
```

---

# 117. Capability Limits

```text
max capabilities
max extension count
max parameter bytes
max dynamic updates/sec
```

---

# 118. Call Limits

```text
max active calls
max video tracks
max decoder buffers
max encoder workers
```

---

# 119. Group Limits

```text
max group members
max simultaneous group fan-out
max group update backlog
```

---

# 120. Sync Limits

```text
max events per batch
max concurrent stream syncs
max history backfill bytes
```

---

# 121. Limit Negotiation

Part 07 can negotiate peer-facing maxima.

Effective limit:

```text
min(
    local hard limit,
    local policy,
    negotiated remote limit,
    current pressure limit
)
```

---

# 122. Runtime Tightening

Negotiated:

```text
parallel chunks = 8
```

Current pressure:

```text
effective = 2
```

No renegotiation required for local throttling unless peer behavior depends on it.

---

# 123. Inbound Limit Enforcement

Always enforce before allocation.

Example:

```text
declared frame = 100 MB
local max = 1 MB
→ reject header
```

Do not allocate then reject.

---

# 124. Length Conversion Safety

Never cast untrusted:

```rust
u64 -> usize
```

before bounds validation.

---

# 125. Decompression Limits

If compression exists:

```text
max decompressed size
max ratio
streaming limits
```

Protect against decompression bombs.

---

# 126. Recursive Structure Limits

Bound:

```text
nesting depth
list length
map length
```

in protocol/application payloads.

---

# 127. Hash/CPU DoS

An attacker can send many objects requiring hashing/signature verification.

Use:

```text
verification queue
per-peer CPU quota
rate limits
```

---

# 128. Signature Verification Budget

Batch/parallelize carefully.

Unknown peers should not consume unlimited crypto verification CPU.

---

# 129. Unknown Peer Admission

Before expensive crypto work, perform cheap checks:

```text
length
syntax
rate limit
cookie/challenge
```

where protocol permits.

---

# 130. Connection Admission

Before full session establishment:

```text
global connection slot
per-source rate
handshake budget
```

Protect against connection floods.

---

# 131. Handshake Budget

Separate:

```text
half-open
authenticated
active
```

connection pools.

Half-open sessions receive smallest budget.

---

# 132. Abuse Escalation

Repeated violations:

```text
rate limit
temporary quarantine
disconnect
block
```

Security/abuse subsystem informs admission.

---

# 133. Queue Poisoning Protection

A peer must not fill high-priority queue with fake critical work.

Priority requests are mapped through authorization policy.

---

# 134. Priority Authorization

Example:

```text
Unknown peer asks Critical
→ downgrade to Normal or reject
```

Only trusted authority/user-local action can claim certain classes.

---

# 135. Resource Accounting Across Retries

Retry should not leak reservations.

Each retry:

```text
reuse or reacquire cleanly
```

Do not accumulate storage/memory permits.

---

# 136. Resource Accounting Across Path Switch

File transfer switching path keeps:

```text
transfer slot
storage reservation
```

but may release/acquire stream/connection permits.

---

# 137. Resource Accounting Across Process Restart

Durable reservations:

```text
storage
```

may need reconstruction.

In-memory permits:

```text
memory
streams
CPU
```

are rebuilt.

---

# 138. Crash Recovery Integration

Part 09 should reconcile:

```text
orphan storage reservation
staging file
work queue
active operation state
```

after abnormal shutdown.

---

# 139. Event Log Integration

Part 04 records meaningful overload transitions only where useful.

Do not journal every permit acquisition.

Examples worth recording:

```text
TransferDeferredStorage
TransferPausedResourcePressure
```

---

# 140. Resource State as Projection

Operational resource state is mostly runtime/diagnostic.

Persistent settings:

```text
user quota
app policy
```

live in config/storage.

---

# 141. Resource Config

```rust
pub struct ResourceConfig {
    pub profile: ResourceProfile,
    pub memory: MemoryLimits,
    pub storage: StorageLimits,
    pub network: NetworkLimits,
    pub peers: PeerLimits,
    pub queues: QueueLimits,
}
```

Validate at startup.

---

# 142. Config Validation

Reject inconsistent config.

Examples:

```text
critical reserve > total storage quota
per-peer streams > global streams
bulk queue > global queue memory budget
```

---

# 143. Dynamic Config

Admins/users may change:

```text
relay storage quota
max parallel transfers
data saver
```

Runtime applies safely.

---

# 144. Enterprise Tenant Quotas

For multi-tenant/server:

```text
per tenant
per org
per user
per peer
```

hierarchical accounting is required.

---

# 145. Tenant Isolation

Tenant A cannot consume Tenant B's reserved critical budget.

Shared spare capacity can be borrowable under policy.

---

# 146. Server Overload

Server mode should degrade:

```text
reject new bulk
limit unknown peers
preserve authenticated control
```

rather than crash.

---

# 147. Headless Relay Mode

Relay-specific budget:

```text
relay storage
relay bandwidth
peer count
bundle intake
```

No UI memory reserved.

---

# 148. Mobile Background Mode

When backgrounded:

```text
reduce queues
pause bulk
reduce CPU jobs
keep essential durable/control
```

OS policy may further restrict.

---

# 149. Foreground Mode

Can temporarily increase:

```text
discovery
transfer concurrency
UI cache
```

within hard limits.

---

# 150. Battery Saver Mode

Part 13 integrates by tightening:

```text
CPU permits
network bulk rates
scan concurrency
```

---

# 151. Emergency Mode

May reallocate resources:

```text
more critical queue reserve
less background cache
more DTN critical reserve
```

but not exceed global hard ceilings.

---

# 152. Call Active Mode

Reserve:

```text
bandwidth
CPU
memory
audio/video buffers
```

and throttle:

```text
bulk transfer
background sync
```

---

# 153. Reservation Preemption

Some low-priority reservations can be preemptible.

Example:

```text
background thumbnail cache
```

High-priority operation can reclaim.

Do not preempt resources that would corrupt active durable work without safe pause.

---

# 154. Preemptible Permit

```rust
pub enum PermitClass {
    Hard,
    Preemptible,
}
```

Use carefully.

---

# 155. Safe Pause

File transfer can pause at chunk boundary.

This makes it preemptible.

Identity update cannot be partially paused mid-commit.

---

# 156. Scheduler Architecture

```text
Ingress
 ↓
Admission
 ↓
Priority queues
 ↓
Fair scheduler
 ↓
Resource permit acquisition
 ↓
Worker
 ↓
Feedback
```

---

# 157. Scheduler Shards

Potentially separate:

```text
network scheduler
CPU scheduler
storage scheduler
```

Coordinated by admission layer.

Avoid one giant lock.

---

# 158. Lock Contention

Resource accounting hot paths should use:

```text
atomics
sharded counters
small critical sections
```

where profiling justifies.

Correctness first.

---

# 159. Async Cancellation

All waits on:

```text
queue
permit
storage reservation
```

must be cancellable.

---

# 160. Structured Concurrency

Operations should own child tasks.

Cancel parent:

```text
children cancel
permits release
```

Avoid detached task leaks.

---

# 161. Task Registry

Runtime may track:

```text
active task count
owner
priority
age
```

for diagnostics.

---

# 162. No Fire-and-Forget by Default

Do not spawn untracked long-lived tasks.

Long-lived services belong to supervised runtime.

---

# 163. Supervisor

```text
RuntimeSupervisor
├── routing worker
├── event projector
├── transfer scheduler
├── DTN engine
└── capability updater
```

Supervisor enforces lifecycle/resource limits.

---

# 164. Restart Policy

A failed worker may restart with bounded backoff.

Repeated crash:

```text
mark subsystem degraded
```

Do not infinite restart loop.

---

# 165. Resource Pressure Notifications

Internal event:

```rust
pub enum ResourceEvent {
    PressureChanged(ResourceKind, PressureState),
    AdmissionRejected(ResourceOwner),
    QuotaExceeded(ResourceOwner),
}
```

UI/application can subscribe to high-level state.

---

# 166. Rate-Limited Diagnostics

Do not emit resource event per packet.

Coalesce pressure changes.

---

# 167. UI Mapping

Examples:

```text
Storage nearly full
Large transfers paused
Low-memory mode
Too many active transfers
```

Normal users should not see semaphore counts.

---

# 168. Developer View

Advanced:

```text
Memory pool 86%
Bulk queue 64/64
DTN relay 91%
CPU heavy workers 4/4
Peer Bob streams 8/8
```

---

# 169. Resource Error Model

```rust
pub enum ResourceError {
    HardLimitExceeded,
    QuotaExceeded,
    Busy,
    Deferred,
    StorageFull,
    MemoryPressure,
    TooManyConnections,
    TooManyStreams,
    Cancelled,
}
```

---

# 170. Retryability

Classify:

```text
Busy → retryable
StorageFull → user action/policy
HardLimitExceeded → permanent for request
Cancelled → terminal
```

---

# 171. No Generic String Errors

Public reusable API must return typed errors.

---

# 172. Testing Strategy

Unit tests:

```text
quota accounting
queue capacity
priority scheduling
permit release
watermark transitions
```

Integration:

```text
file + call contention
DTN + storage pressure
many peers
```

Fault:

```text
worker crash
permit cancellation
disk full
low-memory signal
```

---

# 173. Property Tests

Invariants:

```text
usage never exceeds hard budget
released permit returns capacity
critical reserve cannot be consumed by bulk
peer quota <= global quota
duplicate reservation release does not underflow
```

---

# 174. Concurrency Tests

Run many tasks acquiring/releasing resources.

Assert:

```text
no deadlock
no starvation beyond policy
no accounting drift
```

---

# 175. Backpressure Test

Producer generates 1M items.

Consumer slow.

Expected:

```text
bounded memory
producer slowed/rejected
```

not OOM.

---

# 176. File Saturation Test

Start:

```text
100 large transfers
```

with limit 4.

Expected:

```text
4 active
96 deferred
bounded staging/memory
```

---

# 177. Call Priority Test

Start bulk transfer.

Then start video call.

Expected:

```text
call reserves resources
bulk throttles
call quality protected
```

---

# 178. Emergency Preemption Test

Fill normal/bulk queues.

Inject SOS.

Expected:

```text
SOS admitted through reserved critical capacity
```

---

# 179. Unknown Peer Flood Test

Unknown peer sends:

```text
many file offers
many bundle offers
many handshakes
```

Expected:

```text
rate limited
bounded CPU/memory/storage
```

---

# 180. Storage Full Test

Fill disk.

Expected:

```text
cache eviction
bulk rejected
critical reserve preserved
no corrupt partial commits
```

---

# 181. Low-Memory Test

Trigger pressure.

Expected:

```text
cache shrinks
parallelism drops
ephemeral queues coalesce/drop
durable state survives
```

---

# 182. Cancellation Test

Cancel task while waiting for multiple permits.

Expected:

```text
no leaked permits
```

---

# 183. Panic Test

Worker panics while holding permits.

Expected:

```text
RAII releases
supervisor handles worker failure
```

---

# 184. Fuzz Targets

Fuzz:

```text
resource config parser
quota update messages if network-exposed
admission request decoding
limit negotiation
```

---

# 185. Benchmarking

Benchmark:

```text
permit acquisition
queue operations
scheduler throughput
1000 peers
10k deferred ops
```

Resource layer must remain cheap relative to network/storage work.

---

# 186. Suggested Crate Structure

```text
crates/comm-resource/
├── src/
│   ├── lib.rs
│   ├── budget.rs
│   ├── profile.rs
│   ├── owner.rs
│   ├── request.rs
│   ├── admission.rs
│   ├── permit.rs
│   ├── memory.rs
│   ├── storage.rs
│   ├── network.rs
│   ├── connection.rs
│   ├── stream.rs
│   ├── queue.rs
│   ├── scheduler.rs
│   ├── fairness.rs
│   ├── pressure.rs
│   ├── quota.rs
│   ├── metrics.rs
│   ├── diagnostics.rs
│   └── error.rs
└── Cargo.toml
```

---

# 187. Core Public API

```rust
let permit = resources
    .admit(ResourceRequest::file_transfer(...))
    .await?;

run_transfer(permit).await?;
```

High-level feature APIs should hide most manual resource plumbing.

---

# 188. Feature Integration API

Messaging/file/DTN layers can request typed permits:

```rust
resources.messaging().acquire_message(...)
resources.files().acquire_transfer(...)
resources.dtn().reserve_bundle(...)
```

This keeps policies domain-aware.

---

# 189. Initial Production Scope

Implement first:

```text
global runtime budget
per-feature budgets
per-peer quotas
bounded queues
priority classes
critical reserve
memory permits
stream/connection permits
storage reservation
transfer concurrency
token-bucket rate limiting
pressure states
load shedding
diagnostics
```

Defer initially:

```text
complex adaptive borrowing
fine-grained CPU accounting
advanced predictive scheduling
cross-process cgroup integration
```

---

# 190. Implementation Phases

## Phase 1 — Resource Model

```text
ResourceKind
ResourceProfile
ResourceBudget
ResourceOwner
PressureState
```

## Phase 2 — Permits

```text
memory
connections
streams
tasks
```

## Phase 3 — Queues

```text
bounded queues
priority
critical reserve
```

## Phase 4 — Storage

```text
quotas
reservations
watermarks
eviction signals
```

## Phase 5 — Fairness

```text
peer
extension
priority
bandwidth shaping
```

## Phase 6 — Platform Pressure

```text
Android memory
iOS memory
thermal
background
```

## Phase 7 — Hardening

```text
flood tests
deadlock tests
panic/cancel tests
benchmarks
```

---

# 191. Definition of Done

Part 08 is complete when:

- no unbounded queue exists on external/high-volume data paths
- every large allocation is preceded by validated limits
- files cannot consume all memory/storage
- one peer cannot monopolize streams or bandwidth
- one extension cannot monopolize runtime resources
- bulk traffic cannot starve control/critical traffic
- SOS/security events have reserved capacity
- durable work defers rather than disappears under overload
- stale ephemeral work can be dropped/coalesced
- storage has separate durable/cache/staging/relay quotas
- active calls can reserve realtime resources
- Android/iOS pressure signals tighten shared Rust policy
- cancellation/panic releases permits safely
- admission behavior is deterministic and observable
- unknown-peer floods remain bounded
- process restart can reconcile persistent reservations
- resource metrics and diagnostics are available
- property/concurrency/flood/storage-full tests exist

---

# 192. Relationship to Earlier Parts

Part 08 builds on:

```text
01 — Protocol Extension System
02 — Multi-Device Identity
03 — Transport & Routing Policy Engine
04 — Offline Event Log
05 — Robust File / Blob Subsystem
06 — DTN / Store-Carry-Forward
07 — Capability Negotiation
```

It directly supports:

```text
09 — Crash Recovery
10 — Protocol Fuzzing & Test Suite
11 — Relay / Self-Hosted Infrastructure
12 — Multipath Networking
13 — Battery-Aware Scheduling
14 — Proximity Abstraction
16 — Daemon & Headless Runtime
17 — Emergency Priority Architecture
18 — Network Diagnostics & Path Visualization
19 — C ABI / FFI
20 — Embedded Linux Node
22 — Third-Party Protocol Extensions
24 — Plugin / Module Ecosystem
```

---

# 193. Final Principle

A communication platform is not production-ready merely because it is fast under normal load.

It is production-ready when overload is **predictable and bounded**.

The system must be able to say:

```text
This transfer is deferred.
This peer has reached its quota.
This queue is full.
This background work is paused.
This stale media frame is dropped.
This SOS still has reserved capacity.
```

instead of:

```text
keep allocating until the process crashes
```

The resource-limit and backpressure layer is therefore a platform-wide safety system.

It ensures that messaging, files, DTN, calls, routing, headless nodes, and third-party extensions can coexist on mobile, desktop, server, and embedded devices without one subsystem destroying the reliability of the rest.
