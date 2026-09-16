# Core System Architecture Part 137 — Anonymous Network Extension Inter-Process Communication, Event Bus, Host Calls, Shared Memory, Streaming, Backpressure & Privacy-Preserving Runtime Communication Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 137  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 19, 24, 68, 75, 79, 130, 133–136

**Primary purpose:** define SIAR's extension runtime-communication architecture for host calls, native IPC, WASM host-function calls, event buses, request/response envelopes, streaming, shared-memory transfer, backpressure, cancellation, flow control, crash recovery, versioning, fairness, and privacy-preserving runtime communication.

---

# 1. Purpose

Extension runtimes need efficient communication with the host, but communication channels are themselves trust boundaries.

Poor IPC architecture can cause:

```text
unbounded memory growth
host deadlocks
cross-extension data leakage
stale capability use
correlation leaks
oversized copies
replay
queue starvation
```

The governing principle is:

> **All extension runtime communication should be typed, versioned, bounded, capability-aware, cancellation-safe, backpressure-aware, privacy-minimized, and unable to expose host internals or cross extension boundaries.**

---

# 2. Architectural Position

```text
                   EXTENSION RUNTIME
                          │
                          ▼
                  COMMUNICATION ADAPTER
                          │
          ┌───────────────┼───────────────┐
          │               │               │
       HOST CALLS       EVENT BUS       STREAMS
          │               │               │
          └───────────────┼───────────────┘
                          ▼
                 CAPABILITY / POLICY GATE
                          │
                          ▼
                      HOST SERVICES
```

---

# 3. Core Separation

Keep distinct:

```text
command
query
event
stream
shared-memory transfer
control message
diagnostic message
```

---

# 4. Non-Goals

Part 137 does not create:

```text
raw internal actor access
unbounded publish/subscribe
shared global event bus
host memory exposure
cross-extension direct sockets
```

---

# 5. Communication Domain

```rust
pub enum ExtensionCommDomain {
    HostCall,
    Event,
    Stream,
    SharedMemory,
    Control,
    Diagnostics,
}
```

---

# 6. Transport Class

```rust
pub enum ExtensionTransportClass {
    InProcessWasm,
    LocalIpc,
    SharedMemory,
}
```

---

# 7. Hard Rule

Transport class does not change authorization semantics.

---

# 8. Canonical Envelope

```rust
pub struct ExtensionEnvelope {
    pub version: ExtensionCommVersion,
    pub runtime: ExtensionRuntimeId,
    pub correlation: CorrelationId,
    pub kind: ExtensionEnvelopeKind,
    pub payload: Vec<u8>,
}
```

---

# 9. Bounded Payload

Hard rule.

---

# 10. Envelope Kind

```rust
pub enum ExtensionEnvelopeKind {
    Request,
    Response,
    Event,
    StreamControl,
    StreamChunk,
    Control,
    Error,
}
```

---

# 11. Postcard

Preferred binary encoding for IPC/control messages.

---

# 12. Hard Rule

JSON is not used internally unless required for external interoperability.

---

# 13. Communication Version

```rust
pub struct ExtensionCommVersion(pub u32);
```

---

# 14. Version Negotiation

Part 129 compatibility rules apply.

---

# 15. Hard Rule

No silent fallback to an incompatible IPC version.

---

# 16. Request Identity

```rust
pub struct ExtensionRequestId(pub [u8; 16]);
```

---

# 17. Correlation Identity

```rust
pub struct CorrelationId(pub [u8; 16]);
```

---

# 18. Hard Rule

Correlation IDs are scoped to runtime/session and never reused as global user identifiers.

---

# 19. Request Envelope

```rust
pub struct ExtensionRequestEnvelope {
    pub request_id: ExtensionRequestId,
    pub runtime: ExtensionRuntimeId,
    pub host_call: HostCallId,
    pub capability_token: ExtensionCapabilityTokenRef,
    pub deadline: Option<Timestamp>,
    pub payload: BrokeredPayload,
}
```

---

# 20. Response Envelope

```rust
pub struct ExtensionResponseEnvelope {
    pub request_id: ExtensionRequestId,
    pub result: ExtensionHostCallResult,
}
```

---

# 21. Host Call Registry

```rust
pub struct HostCallRegistry {
    pub version: HostCallRegistryVersion,
    pub calls: BTreeMap<HostCallId, HostCallDescriptor>,
}
```

---

# 22. Descriptor

```rust
pub struct HostCallDescriptor {
    pub capability: Option<SdkCapability>,
    pub request_schema: SchemaId,
    pub response_schema: SchemaId,
    pub max_request_bytes: u64,
    pub max_response_bytes: u64,
    pub timeout: Duration,
}
```

---

# 23. Hard Rule

Every privileged host call has an explicit capability requirement.

---

# 24. Host Call Categories

Examples:

```text
messaging
storage
network
file
device
notification
diagnostics
background jobs
```

---

# 25. WASM Host Calls

WASM uses imported host functions mapped onto the same canonical host-call registry.

---

# 26. Hard Rule

WASM convenience imports cannot bypass broker authorization.

---

# 27. Native IPC

Native extensions communicate over authenticated local IPC.

---

# 28. Preferred Transport

Platform-specific options:

```text
Unix domain sockets
Windows named pipes
local QUIC only if strong reason exists
```

---

# 29. Hard Rule

IPC endpoint is private to the extension runtime.

---

# 30. Peer Authentication

Host and native runtime mutually authenticate using short-lived local workload credentials.

---

# 31. Hard Rule

No unauthenticated local socket.

---

# 32. Local IPC Session

```rust
pub struct ExtensionIpcSession {
    pub session_id: ExtensionIpcSessionId,
    pub runtime: ExtensionRuntimeId,
    pub protocol_version: ExtensionCommVersion,
    pub grant_epoch: ExtensionGrantEpoch,
}
```

---

# 33. Session Binding

Bound to:

```text
runtime
package
process/workload identity
grant epoch
```

---

# 34. Hard Rule

A restarted process cannot reuse a prior IPC session blindly.

---

# 35. Event Bus

Extension event delivery uses a scoped event bus.

---

# 36. No Global Bus

Hard rule.

---

# 37. Event Class

```rust
pub enum ExtensionEventClass {
    ConversationChanged,
    MessageChanged,
    TransferProgress,
    ConnectivityChanged,
    PermissionChanged,
    PolicyChanged,
    RuntimeLifecycle,
}
```

---

# 38. Event Subscription

```rust
pub struct ExtensionEventSubscription {
    pub runtime: ExtensionRuntimeId,
    pub class: ExtensionEventClass,
    pub scope: ExtensionEventScope,
    pub queue_policy: EventQueuePolicy,
}
```

---

# 39. Event Scope

```rust
pub enum ExtensionEventScope {
    RuntimeLocal,
    Conversation(SdkConversationId),
    Tenant(TenantId),
    Integration(IntegrationId),
}
```

---

# 40. Hard Rule

Subscription scope cannot exceed current grants.

---

# 41. Event Envelope

```rust
pub struct ExtensionEventEnvelope {
    pub event_id: ExtensionEventId,
    pub sequence: u64,
    pub class: ExtensionEventClass,
    pub scope: ExtensionEventScope,
    pub payload: BrokeredPayload,
}
```

---

# 42. Event IDs

Not globally correlatable across extensions.

---

# 43. Hard Rule

No cross-extension stable event ID.

---

# 44. Event Sequence

Per-subscription or per-stream sequence.

---

# 45. Hard Rule

Do not expose global activity order unnecessarily.

---

# 46. Durable vs Ephemeral Events

```rust
pub enum ExtensionEventDurability {
    Ephemeral,
    Replayable,
}
```

---

# 47. Replayable

Only for events with safe bounded replay semantics.

---

# 48. Hard Rule

Typing/presence-style events should usually remain ephemeral.

---

# 49. Event Queue Policy

```rust
pub enum EventQueuePolicy {
    LosslessBounded,
    Coalescing,
    LatestOnly,
    DropOldest,
}
```

---

# 50. Hard Rule

Policy is explicit per event class.

---

# 51. Lossless Bounded

Used only where correctness requires it.

---

# 52. Hard Rule

If bounded queue fills, producer does not allocate without limit.

---

# 53. Coalescing

Useful for:

```text
progress
connectivity
presence
runtime metrics
```

---

# 54. Hard Rule

Coalescing must preserve documented semantics.

---

# 55. Snapshot + Delta

Preferred for stateful UI/integration data.

---

# 56. Hard Rule

If deltas are dropped, extension can request a fresh snapshot.

---

# 57. Backpressure

First-class.

---

# 58. Backpressure State

```rust
pub enum BackpressureState {
    Normal,
    Elevated,
    Saturated,
    Disconnected,
}
```

---

# 59. Hard Rule

Saturated does not become unbounded buffering.

---

# 60. Producer Behavior

When consumer slows:

```text
coalesce
pause
reject
drop according to policy
disconnect
```

---

# 61. Consumer Credits

Credit-based flow control is recommended for high-rate streams.

---

# 62. Credit Window

```rust
pub struct FlowCredit {
    pub messages: u32,
    pub bytes: u64,
}
```

---

# 63. Hard Rule

Both message count and byte count are bounded.

---

# 64. Stream Identity

```rust
pub struct ExtensionStreamId(pub [u8; 16]);
```

---

# 65. Stream Type

```rust
pub enum ExtensionStreamType {
    AttachmentRead,
    AttachmentWrite,
    Media,
    Export,
    Import,
    Diagnostics,
}
```

---

# 66. Stream Open

```rust
pub struct StreamOpenRequest {
    pub runtime: ExtensionRuntimeId,
    pub stream_type: ExtensionStreamType,
    pub capability: SdkCapability,
    pub expected_length: Option<u64>,
}
```

---

# 67. Hard Rule

Stream creation requires authorization before any payload transfer.

---

# 68. Stream State

```rust
pub enum ExtensionStreamState {
    Opening,
    Open,
    HalfClosedLocal,
    HalfClosedRemote,
    Closing,
    Closed,
    Failed,
}
```

---

# 69. Hard Rule

No data after Closed.

---

# 70. Stream Chunk

```rust
pub struct ExtensionStreamChunk {
    pub stream: ExtensionStreamId,
    pub offset: u64,
    pub data: BrokeredBytes,
}
```

---

# 71. Chunk Size

Bounded.

---

# 72. Hard Rule

Large streams do not become giant IPC messages.

---

# 73. Streaming Checks

Validate:

```text
offset
length
quota
integrity if required
authorization epoch
```

---

# 74. Hard Rule

Authorization is revalidated for long-lived streams.

---

# 75. Stream Revocation

Revoking source capability closes stream.

---

# 76. Hard Rule

No stream survives revoked authority.

---

# 77. Stream Cancellation

```rust
pub struct StreamCancel {
    pub stream: ExtensionStreamId,
    pub reason: StreamCancelReason,
}
```

---

# 78. Cancellation Reasons

```rust
pub enum StreamCancelReason {
    CallerCancelled,
    DeadlineExceeded,
    PermissionRevoked,
    ResourceLimit,
    RuntimeStopping,
    Error,
}
```

---

# 79. Hard Rule

Cancellation is idempotent.

---

# 80. Request Cancellation

Host calls may support cancellation.

---

# 81. Hard Rule

Cancellation does not imply rollback of already committed side effects.

---

# 82. Idempotency

Write host calls that may retry should use idempotency keys where appropriate.

---

# 83. Example

```rust
pub struct ExtensionIdempotencyKey(pub [u8; 16]);
```

---

# 84. Hard Rule

Retry safety is defined per host call.

---

# 85. Deadline

Every potentially blocking host call has a bounded deadline.

---

# 86. Hard Rule

No infinite request wait.

---

# 87. Timeout Semantics

Timeout yields Unknown/Uncertain when commit status cannot be proven.

---

# 88. Hard Rule

Do not report failure if side effect may have committed.

---

# 89. Commit Receipt

For durable write calls:

```rust
pub struct HostCallCommitReceipt {
    pub operation: ExtensionRequestId,
    pub committed: bool,
    pub durable_ref: Option<OpaqueDurableRef>,
}
```

---

# 90. Shared Memory

Used only for high-volume data when measured benefit exists.

---

# 91. Use Cases

```text
large attachments
media frames
bulk import/export
```

---

# 92. Hard Rule

No shared memory for secrets unless specifically justified.

---

# 93. Shared Memory Region

```rust
pub struct ExtensionSharedMemoryRegion {
    pub region_id: SharedMemoryRegionId,
    pub owner_runtime: ExtensionRuntimeId,
    pub size: u64,
    pub access: SharedMemoryAccess,
}
```

---

# 94. Access

```rust
pub enum SharedMemoryAccess {
    HostReadExtensionWrite,
    HostWriteExtensionRead,
    Bidirectional,
}
```

---

# 95. Prefer Unidirectional

Hard rule.

---

# 96. Shared Memory Lifecycle

```rust
pub enum SharedMemoryState {
    Allocated,
    Mapped,
    Active,
    Revoking,
    Unmapped,
}
```

---

# 97. Hard Rule

Memory region invalidated on runtime stop/revocation.

---

# 98. Ownership

Host controls allocation.

---

# 99. Hard Rule

Extension cannot map arbitrary host memory.

---

# 100. Size Bound

Hard rule.

---

# 101. Shared Memory Descriptor

Pass handle/FD, not raw pointer.

---

# 102. Hard Rule

No pointer values cross trust boundary.

---

# 103. Zero-Copy

Use only when:

```text
ownership explicit
lifetime explicit
mutability explicit
revocation safe
```

---

# 104. Hard Rule

Zero-copy is optimization, not architecture default.

---

# 105. Ring Buffer

Optional for high-throughput media.

---

# 106. Ring Buffer Descriptor

```rust
pub struct ExtensionRingBufferConfig {
    pub region: SharedMemoryRegionId,
    pub slots: u32,
    pub slot_size: u32,
}
```

---

# 107. Hard Rule

Producer cannot overwrite unread critical data silently unless queue policy explicitly permits it.

---

# 108. Shared Memory Integrity

For untrusted native extension, host validates metadata and lengths.

---

# 109. Hard Rule

Never trust extension-provided offsets without bounds checks.

---

# 110. WASM Memory

WASM linear memory already sandboxed.

---

# 111. Host Borrowing

Avoid long-lived host references into WASM memory.

---

# 112. Hard Rule

Copy or pin only within well-defined call lifetime.

---

# 113. Reentrancy

Host-call callbacks into same extension are restricted.

---

# 114. Hard Rule

No uncontrolled recursive host→extension→host call chains.

---

# 115. Reentrancy Policy

```rust
pub enum ReentrancyPolicy {
    Forbidden,
    Deferred,
    AllowedForListedCalls,
}
```

---

# 116. Hard Rule

Default to Forbidden/Deferred.

---

# 117. Deadlock Prevention

Avoid host locks across extension calls.

---

# 118. Hard Rule

No internal mutex held while waiting on extension-controlled response.

---

# 119. Threading

Host communication layer presents async interface.

---

# 120. Hard Rule

Blocking native operations isolated from async runtime.

---

# 121. Bounded Worker Pools

Hard rule.

---

# 122. Priority Classes

```rust
pub enum ExtensionCommPriority {
    Control,
    Interactive,
    Data,
    Background,
    Bulk,
}
```

---

# 123. Priority Principle

Host-critical control always outranks extension bulk traffic.

---

# 124. Hard Rule

Extension cannot self-label arbitrary traffic as Control.

---

# 125. Priority Assignment

Host registry determines maximum priority per call/event class.

---

# 126. Hard Rule

No starvation of lower classes indefinitely.

---

# 127. Fair Scheduling

Use weighted fair queue or equivalent bounded scheduler.

---

# 128. Hard Rule

One extension cannot monopolize communication channels.

---

# 129. Per-Extension Queue Budget

```rust
pub struct ExtensionQueueBudget {
    pub max_messages: u32,
    pub max_bytes: u64,
}
```

---

# 130. Hard Rule

Budget enforced per runtime and globally.

---

# 131. Global Queue Budget

Protects host.

---

# 132. Hard Rule

Overload may throttle extensions before core messaging.

---

# 133. Cross-Extension Isolation

Each extension gets independent queues.

---

# 134. Hard Rule

No event from extension A delivered to B unless explicit brokered contract exists.

---

# 135. Inter-Extension Communication

If enabled:

```text
typed
explicit
capability-scoped
versioned
```

---

# 136. Hard Rule

No shared global topic namespace.

---

# 137. Event Topic Identity

```rust
pub struct ExtensionTopicId(pub [u8; 16]);
```

---

# 138. Hard Rule

Topic IDs are scoped and non-enumerable.

---

# 139. Topic Publication

Publisher capability separate from subscriber capability.

---

# 140. Hard Rule

Publish does not imply subscribe.

---

# 141. Event Metadata

Minimize:

```text
global timestamps
hostnames
process IDs
peer IDs
```

---

# 142. Hard Rule

Do not leak host topology through IPC metadata.

---

# 143. Timestamps

Use monotonic/relative time where enough.

---

# 144. Hard Rule

Wall clock not exposed unnecessarily.

---

# 145. Anonymous Mode

Runtime communication should not expose:

```text
raw peer addresses
relay path
mix route
exact implementation versions
stable external identifiers
```

---

# 146. Hard Rule

Extension-facing events use privacy-preserving projections.

---

# 147. Correlation Privacy

Correlation IDs are random, local, short-lived.

---

# 148. Hard Rule

No global tracing ID propagated into anonymous network traffic.

---

# 149. Diagnostics

Communication tracing can include:

```text
call type
latency
result class
queue depth
```

---

# 150. Hard Rule

No private payload by default.

---

# 151. Debug Payload Logging

Disabled by default.

---

# 152. Hard Rule

Never log secrets.

---

# 153. Trace Sampling

Aggregate/technical only.

---

# 154. Hard Rule

No per-user communication timeline.

---

# 155. IPC Authentication Failure

Immediate channel closure.

---

# 156. Hard Rule

Repeated failures may quarantine native runtime.

---

# 157. Replay Protection

Control messages include session context / nonce or sequence where needed.

---

# 158. Hard Rule

Stale request from previous runtime session rejected.

---

# 159. Sequence Number

```rust
pub struct ExtensionSessionSequence(pub u64);
```

---

# 160. Hard Rule

Sequence state is session-scoped.

---

# 161. Message Integrity

Authenticated IPC channel + envelope validation.

---

# 162. Hard Rule

No custom cryptography.

---

# 163. Compression

Allowed for large streams, not small control messages by default.

---

# 164. Hard Rule

Compression bombs bounded.

---

# 165. Maximum Decompressed Size

Explicit.

---

# 166. Hard Rule

Schema Validation

All incoming messages are schema-validated and bounds-checked before dispatch.

---

# 167. Unknown Message Type

Reject safely.

---

# 168. Hard Rule

No default fallthrough to privileged action.

---

# 169. Unknown Field

Handled by schema-version policy.

---

# 170. Hard Rule

Compatibility policy applies.

---

# 171. Host Call Error

```rust
pub enum HostCallError {
    InvalidRequest,
    CapabilityDenied,
    ScopeDenied,
    DeadlineExceeded,
    Cancelled,
    Backpressured,
    ResourceLimit,
    VersionMismatch,
    RuntimeStopping,
    Internal,
}
```

---

# 172. Error Detail

Minimized.

---

# 173. Hard Rule

Error messages must not reveal hidden resource existence where authorization would deny enumeration.

---

# 174. Backpressure Error

May include coarse retry hint.

---

# 175. Hard Rule

No precise host capacity leakage in anonymous mode.

---

# 176. Runtime Restart

Host reconstructs queues/subscriptions from durable policy, not stale in-memory state.

---

# 177. Hard Rule

Ephemeral subscriptions are not silently resurrected.

---

# 178. Replayable Subscriptions

Must resume from explicit checkpoint.

---

# 179. Checkpoint

```rust
pub struct ExtensionEventCheckpoint {
    pub subscription: ExtensionSubscriptionId,
    pub sequence: u64,
}
```

---

# 180. Hard Rule

Checkpoint valid only for same extension/scope/version.

---

# 181. Event Gap

If replay unavailable, extension receives explicit resync-required state.

---

# 182. Hard Rule

Never silently pretend no event was missed.

---

# 183. Crash Semantics

In-flight calls become:

```text
cancelled
uncertain
or completed with durable receipt
```

depending operation.

---

# 184. Hard Rule

Crash recovery never duplicates non-idempotent side effects silently.

---

# 185. Request Journal

For selected durable host calls only.

---

# 186. Hard Rule

Do not journal every IPC payload.

---

# 187. Durable Call State

```rust
pub enum DurableCallState {
    Accepted,
    Committed,
    Failed,
    Unknown,
}
```

---

# 188. Hard Rule

Unknown remains distinct from Failed.

---

# 189. Shutdown

Communication shutdown order:

```text
stop accepting new requests
cancel background/bulk streams
drain bounded critical responses
revoke new host calls
close channels
```

---

# 190. Hard Rule

Extension cannot hold host shutdown indefinitely.

---

# 191. Shutdown Deadline

Bounded.

---

# 192. Hard Rule

After deadline, force close.

---

# 193. Event Ordering

Only guaranteed where explicitly documented.

---

# 194. Hard Rule

No global total order.

---

# 195. Per-Stream Order

May be guaranteed.

---

# 196. Hard Rule

Cross-stream ordering not inferred.

---

# 197. Causal Metadata

Avoid unless product semantics require it.

---

# 198. Hard Rule

Do not expose unnecessary correlation structure.

---

# 199. Large Message Transfer

Use streams/shared memory rather than increasing envelope limits.

---

# 200. Hard Rule

Message size caps remain strict.

---

# 201. Attachment Transfer

Preferred:

```text
handle
→ stream/shared memory
→ broker verification
```

---

# 202. Hard Rule

No giant Vec copied through JNI/native IPC.

---

# 203. Media Transfer

Use shared memory/ring buffer where measured beneficial.

---

# 204. Hard Rule

Media path separate from control path.

---

# 205. Control Plane Isolation

Control messages must not compete with bulk data in same unbounded queue.

---

# 206. Hard Rule

Separate logical channels/priorities.

---

# 207. Communication Quotas

Per extension:

```text
requests/sec
bytes/sec
streams
subscriptions
queued bytes
```

---

# 208. Hard Rule

Quota exhaustion yields explicit typed result.

---

# 209. Abuse Detection

Based on technical rate/resource violations.

---

# 210. Hard Rule

No payload-content inspection for ordinary rate enforcement.

---

# 211. Communication Policy

```rust
pub struct ExtensionCommunicationPolicy {
    pub max_requests_per_second: u32,
    pub max_streams: u32,
    pub queue_budget: ExtensionQueueBudget,
    pub allowed_priorities: BTreeSet<ExtensionCommPriority>,
}
```

---

# 212. Signed/versioned.

---

# 213. Policy Epoch

```rust
pub struct ExtensionCommunicationPolicyEpoch(pub u64);
```

---

# 214. Anti-Rollback

Hard rule.

---

# 215. Policy Tightening

Can reduce quotas immediately.

---

# 216. Hard Rule

No permissive fail-open on stale policy.

---

# 217. Communication Observability

Safe metrics:

```text
host-call latency by class
queue depth
stream bytes
backpressure count
cancellation count
crash-disconnect count
```

---

# 218. Forbidden:

```text
private payloads
user identity
contact graph
cross-extension correlation graph
```

---

# 219. Hard Rule

Observability is extension/runtime scoped and aggregate.

---

# 220. SLOs

Examples:

```text
control host-call p99 latency
bounded queue saturation recovery
revocation-to-channel-close latency
stream cancellation latency
```

---

# 221. Security SLO

```text
0 unauthenticated native IPC sessions
0 cross-extension event leakage
0 host memory pointer exposure
0 revoked stream continuing privileged transfer
```

---

# 222. Privacy SLO

```text
0 global correlation ID propagated through extension events
0 anonymous route metadata exposed via runtime IPC
0 private payload logging by default
```

---

# 223. Failure Modes

```text
event flood
slow consumer
dead native process
partial shared-memory write
stream revocation race
```

---

# 224. Event Flood

Throttle/coalesce/drop by policy.

---

# 225. Slow Consumer

Bound queues + resync.

---

# 226. Dead Native Process

Session closed, runtime marked Failed.

---

# 227. Partial Shared Memory Write

Use explicit ownership/state markers and integrity validation.

---

# 228. Revocation Race

Epoch check before privileged commit/stream continuation.

---

# 229. Testing

Need extension-communication testkit.

---

# 230. Test Scenarios

```text
WASM host call
native IPC request
event coalescing
stream cancellation
shared-memory transfer
```

---

# 231. Envelope Bounds Test

Oversized payload rejected.

---

# 232. Auth Test

Unauthenticated native IPC rejected.

---

# 233. Version Test

Unsupported communication version rejected.

---

# 234. Queue Test

Slow consumer cannot cause unbounded memory.

---

# 235. Coalescing Test

Progress events preserve latest state.

---

# 236. Resync Test

Dropped deltas cause explicit resync requirement.

---

# 237. Cancellation Test

Cancelled request cannot leak resources.

---

# 238. Revocation Test

Running stream stops after capability revocation.

---

# 239. Shared Memory Bounds Test

Invalid offset/length rejected.

---

# 240. Crash Test

Native process exit closes session without host crash.

---

# 241. Replay Test

Old-session request rejected.

---

# 242. Cross-Extension Isolation Test

Runtime A cannot subscribe to B's private topic.

---

# 243. Priority Test

Bulk extension traffic cannot starve control traffic.

---

# 244. Shutdown Test

Extension cannot prevent host shutdown.

---

# 245. Privacy Test

Anonymous-mode event contains no route/IP/build fingerprint.

---

# 246. Fuzzing

Fuzz:

```text
envelopes
host-call payloads
event frames
stream control messages
shared-memory descriptors
```

---

# 247. Property Tests

Properties:

```text
no queue can grow beyond configured message/byte budget
revoked capability can never authorize new stream data
extension A can never receive event scoped only to extension B
unknown message types can never dispatch privileged host calls
```

---

# 248. Formal Verification Targets

Strong candidates:

```text
stream lifecycle
request/cancel/commit state
queue backpressure
grant-revocation propagation
```

---

# 249. Kani Candidate

Bounds/priority/authorization invariants.

---

# 250. TLA+ Candidate

request → authorize → execute → respond/cancel/revoke.

---

# 251. Loom Candidate

Concurrent:

```text
stream write
capability revoke
runtime shutdown
queue drain
```

---

# 252. Performance

Communication overhead should remain small relative to extension work.

---

# 253. Baseline

```text
Postcard for control
handles/streams for large data
shared memory only where measured
bounded queues everywhere
```

---

# 254. Hard Rule

Never optimize by exposing internal memory ownership unsafely.

---

# 255. Batching

Allowed for:

```text
events
small storage calls
telemetry
```

when semantics permit.

---

# 256. Hard Rule

Batch limits explicit by count/bytes/wait time.

---

# 257. Copy Reduction

Preferred order:

```text
small message copy
stream
shared memory
```

Do not jump directly to complex zero-copy.

---

# 258. Hard Rule

Correctness and isolation outrank zero-copy performance.

---

# 259. Storage

Separate metadata:

```text
IPC sessions
subscriptions
stream state
communication policy
durable-call journal for selected calls
```

---

# 260. Hard Rule

Do not persist arbitrary event payloads.

---

# 261. Partitioning

By:

```text
runtime
extension
stream
subscription
```

No user/person analytics partition.

---

# 262. Crate Layout

Recommended:

```text
crates/
├── siar-extension-comm-core/
├── siar-extension-hostcall/
├── siar-extension-ipc/
├── siar-extension-eventbus/
├── siar-extension-stream/
├── siar-extension-shmem/
├── siar-extension-backpressure/
├── siar-extension-comm-policy/
├── siar-extension-comm-observability/
└── siar-extension-comm-testkit/
```

---

# 263. `siar-extension-comm-core`

Owns:

```text
ExtensionEnvelope
ExtensionCommVersion
CorrelationId
ExtensionCommError
```

---

# 264. `siar-extension-hostcall`

Host-call registry/request-response/cancellation.

---

# 265. `siar-extension-ipc`

Authenticated native-process IPC.

---

# 266. `siar-extension-eventbus`

Scoped subscriptions, event projections, replay/resync.

---

# 267. `siar-extension-stream`

Streaming lifecycle, credits, cancellation.

---

# 268. `siar-extension-shmem`

Shared-memory/ring-buffer transport.

---

# 269. `siar-extension-backpressure`

Queue budgets, flow credits, coalescing, fairness.

---

# 270. `siar-extension-comm-policy`

Signed quotas/priorities/policy epochs.

---

# 271. `siar-extension-comm-observability`

Aggregate technical communication health only.

---

# 272. `siar-extension-comm-testkit`

IPC/event/stream/shmem/backpressure/privacy tests.

---

# 273. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. Extension commands, queries, events, streams, shared-memory transfers, control messages, and diagnostics are distinct typed communication classes with explicit schema/version/size limits.
2. Every native IPC session is authenticated, runtime/package/session-bound, grant-epoch-aware, and invalid after process/runtime restart; no unauthenticated local socket or stale session may invoke host services.
3. WASM host imports and native IPC host calls map to the same canonical host-call registry and capability-policy path, so transport choice cannot bypass authorization.
4. Event subscriptions are scoped, bounded, capability-checked, extension-isolated, and use explicit loss/coalescing/replay semantics; no global event bus or implicit cross-extension topic access exists.
5. Every queue has both message-count and byte limits; saturation results in explicit coalescing, rejection, pause, drop, resync, or disconnect behavior rather than unbounded buffering.
6. Streams and shared-memory regions are authorized before creation, bounded by quota, revalidated during long-lived use, and invalidated on permission revocation, runtime stop, policy change, or session death.
7. Shared memory uses host-controlled regions/handles with explicit direction, bounds, ownership, and lifecycle; raw host pointers or arbitrary host memory mappings never cross the trust boundary.
8. Cancellation, timeout, retry, idempotency, and crash recovery preserve truthful operation state; Unknown remains distinct from Failed when side-effect commit cannot be proven.
9. Control/authorization traffic is isolated and prioritized above extension bulk traffic, while fair scheduling prevents one extension from starving peers or core host services.
10. Correlation IDs, event IDs, timestamps, error details, and diagnostics are session/runtime scoped and privacy-minimized; they cannot become global user identifiers, cross-extension tracing graphs, or anonymous-network metadata leaks.
11. Runtime communication logs and metrics contain technical envelope/latency/queue/result information only by default and never private payloads, secrets, social graphs, raw peer addresses, or global tracing identifiers.
12. Extension runtime communication integrates with permissions, data governance, runtime supervision, compatibility, SDKs, marketplace revocation, secret management, background scheduling, audit, assurance, and anonymous-mode privacy without creating a side channel around SIAR's security, privacy, local-first, or tenant-isolation guarantees.
```

---

# 274. Initial Production Scope

Implement first:

```text
typed Postcard communication envelope
versioned host-call registry
WASM host-call adapter
authenticated native local IPC
runtime/session/grant-epoch binding
bounded request/response payloads
scoped event bus
coalescing/latest-only event policies
snapshot+delta resync
bounded per-extension queues
flow-control credits
streaming API
stream cancellation/deadlines
idempotency keys for retryable writes
selected durable call receipts
shared-memory descriptor model
host-owned bounded shared-memory regions
priority/fair scheduling
signed communication policy
privacy-safe runtime communication metrics
communication testkit
```

Then add:

```text
shared-memory ring buffers for media
adaptive flow control
cross-process zero-copy attachment streaming
formal stream/revocation proofs
high-assurance event ordering models
portable IPC transport abstraction for additional platforms
```

---

# 275. Definition of Done

Part 137 is complete when:

- extension envelopes are typed/versioned/bounded;
- WASM and native IPC use the same host-call policy path;
- native IPC is authenticated;
- event subscriptions are scoped and isolated;
- every queue has byte/message limits;
- backpressure behavior is explicit;
- streams support credits/cancel/revocation;
- shared memory is host-owned/bounded/handle-based;
- stale sessions and replay are rejected;
- bulk traffic cannot starve control traffic;
- timeout/uncertain commit semantics are truthful;
- anonymous-mode IPC does not expose route/IP/build metadata;
- no private payload logging or cross-extension tracing exists;
- IPC/event/stream/shmem/privacy/fuzz/formal tests are specified.

---

# 276. Final Architecture

```text
                 EXTENSION RUNTIME
                        │
                        ▼
                 COMMUNICATION LAYER
                        │
         ┌──────────────┼──────────────┐
         │              │              │
      HOST CALLS      EVENTS        STREAMS
         │              │              │
         └──────────────┼──────────────┘
                        ▼
             AUTHZ / POLICY / QUOTAS
                        │
                        ▼
                HOST SERVICE BROKERS
                        │
                        ▼
              CORE / STORAGE / NETWORK
```

Runtime-communication safety model:

```text
typed envelopes
+
authenticated IPC
+
versioned host calls
+
scoped event subscriptions
+
bounded queues
+
credit-based streams
+
host-owned shared memory
+
revocation-aware cancellation
+
privacy-minimized metadata
```

not:

```text
one global event bus, giant messages, raw pointers, unbounded queues, unauthenticated local sockets, and global trace IDs across anonymous traffic
```

---

# 277. Final Principle

Extension communication is trustworthy when every byte crosses a boundary with known authority, known lifetime, known size, and known backpressure behavior.

The correct model is:

```text
type every message
+
authenticate every native channel
+
scope every subscription
+
bound every queue
+
stream large data
+
use shared memory only where justified
+
revalidate long-lived authority
+
cancel safely
+
preserve truthful commit state
+
never trade metadata privacy or isolation for convenience
```

This architecture gives SIAR a privacy-preserving runtime-communication foundation for host calls, IPC, event delivery, streams, shared memory, backpressure, cancellation, crash recovery, flow control, and fair scheduling while preserving the anonymity, local-first, least-authority, extension-runtime, permission, data-governance, and anti-surveillance guarantees established across Parts 34–136.
