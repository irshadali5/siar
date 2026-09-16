# Core System Architecture Part 75 — Anonymous Network Event Streaming, Message Bus, Durable Queues, Internal Pub/Sub & Distributed Workflow Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 75  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 4–9, 30–33, 46–50, 61–74  

**Primary purpose:** define SIAR's internal event-streaming, durable queue, message-bus, pub/sub, distributed workflow, command/event separation, ordering, retry, backpressure, saga, timeout, dead-letter, observability, and failure-isolation architecture across clients, providers, control-plane services, tenants, federation domains, and long-running workflows.

---

# 1. Purpose

Distributed systems need asynchronous coordination.

Without a disciplined event architecture, components become coupled through:

```text
direct RPC chains
shared database polling
ad-hoc retry loops
global mutable state
implicit ordering assumptions
```

These patterns are fragile.

The governing principle is:

> **SIAR asynchronous coordination must be explicit, durable where required, idempotent, bounded, privacy-safe, and domain-scoped.**

---

# 2. Architectural Position

```text
Domain State Change
       │
       ▼
Transactional Outbox
       │
       ▼
Durable Event Bus / Queue
       │
  ┌────┼────┐
  │    │    │
Consumer A Consumer B Workflow Engine
  │    │    │
  └────┼────┘
       ▼
Idempotent Side Effects
```

---

# 3. Core Separation

Keep distinct:

```text
command
domain event
integration event
queue job
notification
workflow state
audit event
telemetry event
```

---

# 4. Non-Goals

Part 75 does not create:

```text
one global event bus carrying every user action
event sourcing for every domain
unbounded queues
best-effort delivery for critical state transitions
cross-tenant event fanout by default
```

---

# 5. Event Classes

```rust
pub enum EventClass {
    Domain,
    Integration,
    Workflow,
    Audit,
    Operational,
}
```

---

# 6. Domain Event

Represents completed domain fact.

Example:

```text
MessagePersisted
DeviceRevoked
TenantProvisioned
```

---

# 7. Integration Event

Stable boundary event between services/domains.

---

# 8. Workflow Event

Progress/completion signal in long-running process.

---

# 9. Audit Event

Admin/governance/infrastructure action.

---

# 10. Operational Event

Health/capacity/deployment state.

---

# 11. Commands

Commands express intent.

---

# 12. Command Examples

```text
SendMessage
RotateProviderKey
ProvisionTenant
DrainRelay
```

---

# 13. Hard Rule

Events are facts; commands are requests.

Do not conflate.

---

# 14. Event Envelope

```rust
pub struct EventEnvelope<T> {
    pub event_id: EventId,
    pub event_type: EventTypeId,
    pub schema_version: EventSchemaVersion,
    pub occurred_at: EventTime,
    pub partition_key: PartitionKey,
    pub payload: T,
}
```

---

# 15. Event ID

Random or monotonic per domain, but not global user identity.

---

# 16. No Global User-Correlatable Event ID

Hard rule.

---

# 17. Event Schema Version

Independent from DB/API/protocol versions.

---

# 18. Event Time

Coarsened where privacy-sensitive.

---

# 19. Partition Key

Domain-scoped.

---

# 20. Examples

```text
conversation-local key
tenant-local key
federation-domain key
workflow key
```

---

# 21. No Global Person Key

Hard rule.

---

# 22. Command Envelope

```rust
pub struct CommandEnvelope<T> {
    pub command_id: CommandId,
    pub command_type: CommandTypeId,
    pub idempotency_key: IdempotencyKey,
    pub payload: T,
}
```

---

# 23. Command Idempotency

Mandatory for retried commands.

---

# 24. Command Result

```rust
pub enum CommandResult<T> {
    Accepted,
    Completed(T),
    Rejected(CommandRejection),
}
```

---

# 25. Event Bus Scope

Internal only.

---

# 26. No Direct End-User Subscription

Hard rule.

---

# 27. Client Event Bus

Local process/device scope.

---

# 28. Server Event Bus

Infrastructure/service scope.

---

# 29. Federation Event Bus

Not shared globally.

---

# 30. Multi-Tenant Bus

Tenant-isolated partitions/namespaces.

---

# 31. Delivery Semantics

Preferred default:

```text
at-least-once
```

---

# 32. Why

Exactly-once across distributed systems is expensive/fragile.

---

# 33. Hard Rule

Do not claim globally exactly-once delivery.

---

# 34. Exactly-Once Effect

Achieved via:

```text
at-least-once delivery
+
idempotent consumer
+
transactional state change
```

---

# 35. Consumer Idempotency

Required.

---

# 36. Consumer Receipt

```rust
pub struct ConsumerReceipt {
    pub consumer: ConsumerId,
    pub event: EventId,
    pub status: ConsumerApplyStatus,
}
```

---

# 37. Consumer State

```rust
pub enum ConsumerApplyStatus {
    Applied,
    Duplicate,
    Rejected,
    Deferred,
}
```

---

# 38. Transactional Consumer

Flow:

```text
begin tx
→ check event_id
→ apply domain change
→ persist consumer receipt
→ commit
```

---

# 39. ACK After Commit

Hard rule.

---

# 40. Transactional Outbox

Part 74.

---

# 41. Publisher Flow

```text
begin tx
→ domain state write
→ outbox event write
→ commit
→ async dispatcher publishes
```

---

# 42. Hard Rule

No publish-before-domain-commit for durable domain events.

---

# 43. Outbox Record

```rust
pub struct OutboxEventRecord {
    pub event: EventEnvelope<Bytes>,
    pub publish_state: OutboxPublishState,
}
```

---

# 44. Publish State

```rust
pub enum OutboxPublishState {
    Pending,
    Publishing,
    Published,
    FailedRetryable,
    FailedPermanent,
}
```

---

# 45. Crash Recovery

`Pending/Publishing` safely retried.

---

# 46. Inbox Pattern

Consumer dedup/inbox table.

---

# 47. Inbox Record

```rust
pub struct InboxEventRecord {
    pub event_id: EventId,
    pub consumer: ConsumerId,
    pub received_at: EventTime,
}
```

---

# 48. No Duplicate Side Effect

Hard rule.

---

# 49. Durable Queue

For work requiring eventual execution.

---

# 50. Queue Job

```rust
pub struct QueueJob<T> {
    pub job_id: JobId,
    pub job_type: JobTypeId,
    pub priority: QueuePriority,
    pub payload: T,
    pub retry: RetryPolicy,
}
```

---

# 51. Priority Classes

Reuse:

```text
Emergency
SecurityControl
Realtime
Interactive
Background
Bulk
```

---

# 52. Queue Priority

Must not bypass privacy/security.

---

# 53. Queue Budget

Bounded.

---

# 54. No Unbounded Durable Queue

Hard rule.

---

# 55. Queue Capacity Dimensions

```text
items
bytes
age
```

---

# 56. Queue Limit

```rust
pub struct QueueCapacity {
    pub max_items: u64,
    pub max_bytes: u64,
    pub max_age: Option<Duration>,
}
```

---

# 57. Backpressure

Upstream producer must slow/reject.

---

# 58. No Infinite Retry Growth

Hard rule.

---

# 59. Retry Policy

```rust
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub jitter: bool,
}
```

---

# 60. Exponential Backoff

Recommended.

---

# 61. Jitter

Required for distributed retry.

---

# 62. Retry Classification

```rust
pub enum RetryClass {
    Never,
    ImmediateLimited,
    Exponential,
    Scheduled,
}
```

---

# 63. Retryable Error

Explicit.

---

# 64. No Retry On Permanent Validation Error

Hard rule.

---

# 65. Poison Message

Malformed or repeatedly failing job.

---

# 66. Dead-Letter Queue

Use carefully.

---

# 67. Dead-Letter Record

```rust
pub struct DeadLetterRecord {
    pub job_id: JobId,
    pub reason: FailureClass,
    pub failure_count: u32,
}
```

---

# 68. Privacy

No sensitive payload copied indefinitely.

---

# 69. Hard Rule

Dead-letter storage must not become permanent sensitive-data archive.

---

# 70. Dead-Letter Payload

Prefer:

```text
opaque reference
digest
minimal failure metadata
```

---

# 71. Quarantine Queue

Alternative to full payload DLQ.

---

# 72. Workflow

Long-running multi-step process.

---

# 73. Examples

```text
tenant provisioning
provider migration
key rotation
account deletion
backup restore
release rollout
```

---

# 74. Workflow State

```rust
pub enum WorkflowState {
    Pending,
    Running,
    Waiting,
    Compensating,
    Completed,
    Failed,
    Cancelled,
}
```

---

# 75. Workflow ID

Scoped.

---

# 76. No Global User Workflow ID

Hard rule.

---

# 77. Workflow Step

```rust
pub struct WorkflowStep {
    pub step_id: WorkflowStepId,
    pub command: CommandEnvelope<Bytes>,
    pub compensation: Option<CommandEnvelope<Bytes>>,
}
```

---

# 78. Saga

Preferred for cross-service operations.

---

# 79. Saga Principle

Local transactions + durable events + compensation.

---

# 80. No Distributed 2PC Requirement

Hard rule.

---

# 81. Workflow Journal

Durable.

---

# 82. Workflow Journal Entry

```rust
pub struct WorkflowJournalEntry {
    pub workflow: WorkflowId,
    pub step: WorkflowStepId,
    pub state: WorkflowStepState,
}
```

---

# 83. Step State

```rust
pub enum WorkflowStepState {
    Pending,
    Dispatched,
    Applied,
    Compensated,
    Failed,
}
```

---

# 84. Idempotent Steps

Mandatory.

---

# 85. Compensation

Best-effort, domain-safe.

---

# 86. Compensation Is Not Rollback

Hard truth.

---

# 87. Timeout

Explicit per step.

---

# 88. Workflow Timeout

```rust
pub struct WorkflowTimeout {
    pub step: WorkflowStepId,
    pub deadline: Duration,
}
```

---

# 89. Time Source

Part 60.

---

# 90. Scheduled Job

Timer-driven work.

---

# 91. Scheduler

Part 34 integration.

---

# 92. Durable Timer

Stored.

---

# 93. Timer Fire

Idempotent.

---

# 94. Crash

Timer recovers.

---

# 95. No In-Memory-Only Critical Timer

Hard rule.

---

# 96. Internal Pub/Sub

Best for transient notifications.

---

# 97. Pub/Sub Classes

```rust
pub enum PubSubClass {
    Ephemeral,
    Durable,
}
```

---

# 98. Ephemeral Pub/Sub

Examples:

```text
UI refresh
local state invalidation
noncritical cache update
```

---

# 99. Durable Pub/Sub

For integration events.

---

# 100. Hard Rule

Critical state transition cannot depend solely on ephemeral pub/sub.

---

# 101. Topic

Logical domain namespace.

---

# 102. Topic ID

```rust
pub struct TopicId(pub String);
```

---

# 103. Topic Naming

Domain scoped.

---

# 104. Avoid

```text
users.*
messages.*
```

global patterns.

---

# 105. Prefer

```text
tenant.<opaque>.policy
federation.<domain>.events
```

internally.

---

# 106. But

Topic names still should avoid sensitive IDs where logs expose them.

---

# 107. Topic Alias

Opaque internal identifier.

---

# 108. Subscription

```rust
pub struct Subscription {
    pub consumer: ConsumerId,
    pub topic: TopicId,
    pub mode: SubscriptionMode,
}
```

---

# 109. Subscription Mode

```rust
pub enum SubscriptionMode {
    CompetingConsumer,
    Fanout,
}
```

---

# 110. Competing Consumer

One worker gets job.

---

# 111. Fanout

Each subscriber gets event.

---

# 112. Fanout Cost

Bounded.

---

# 113. No Unlimited Dynamic Fanout

Hard rule.

---

# 114. Consumer Group

Useful.

---

# 115. Consumer Group ID

Service-scoped.

---

# 116. Ordering

Need explicit guarantees.

---

# 117. Ordering Levels

```rust
pub enum OrderingGuarantee {
    None,
    Partition,
    Total,
}
```

---

# 118. Default

Partition ordering.

---

# 119. Total Ordering

Rare and expensive.

---

# 120. Hard Rule

Do not assume total ordering unless explicitly guaranteed.

---

# 121. Partition Ordering

Examples:

```text
conversation
tenant policy
workflow
```

---

# 122. Partition Sequence

```rust
pub struct PartitionSequence(pub u64);
```

---

# 123. Sequence Monotonicity

Per partition.

---

# 124. Cross-Partition Ordering

Undefined.

---

# 125. Reordering

Consumers must tolerate where allowed.

---

# 126. Event Versioning

Schema evolution.

---

# 127. Expand Compatibility

Add optional fields.

---

# 128. Breaking Event Change

New event type/version.

---

# 129. No Semantic Reuse Of Old Field

Hard rule.

---

# 130. Event Compatibility Matrix

Current/previous.

---

# 131. Consumer Compatibility

Declared.

---

# 132. Consumer Capability

```rust
pub struct ConsumerCapability {
    pub event_type: EventTypeId,
    pub versions: VersionRange,
}
```

---

# 133. Unknown Event

Safe handling.

---

# 134. Critical Consumer

May block deployment if incompatible.

---

# 135. Event Payload

Minimal.

---

# 136. No Full Domain Object By Default

Hard rule.

---

# 137. Event Projection

Purpose-specific.

---

# 138. Good Event

```text
MessagePersisted {
    message_ref,
    conversation_ref,
}
```

---

# 139. Bad Event

```text
EntireMessageIncludingPlaintext
```

unless strictly local/trusted boundary.

---

# 140. Server Bus

Should never carry E2EE plaintext.

---

# 141. Hard rule.

---

# 142. Privacy Event Classification

```rust
pub enum EventPrivacyClass {
    Public,
    Internal,
    SensitiveReference,
    ForbiddenRemote,
}
```

---

# 143. ForbiddenRemote

Never leaves local trust boundary.

---

# 144. Bus Broker

Enforces class.

---

# 145. Event Encryption

At transport layer.

---

# 146. Sensitive Internal Event

May also encrypt payload/application layer.

---

# 147. Broker Should Not Need Plaintext User Content

Hard rule.

---

# 148. Multi-Tenant Isolation

Part 69.

---

# 149. Tenant Namespace

Queue/topic isolated.

---

# 150. No Cross-Tenant Consumer Group

Hard rule.

---

# 151. Tenant Event Context

```rust
pub struct TenantEventContext {
    pub tenant: TenantId,
}
```

---

# 152. Explicit.

---

# 153. No Missing Tenant Context For Tenant Event

Hard rule.

---

# 154. Federation

Separate bus per domain/peer.

---

# 155. No Global Inter-Federation Bus

Hard rule.

---

# 156. Integration Event

Cross-domain only through explicit gateway.

---

# 157. Gateway validates/reprojects.

---

# 158. No Raw Internal Event Bridging

Hard rule.

---

# 159. Local Client Event Bus

Can be in-process.

---

# 160. Tokio broadcast/mpsc

Possible.

---

# 161. But

Critical durable events still persisted.

---

# 162. UI Events

Ephemeral.

---

# 163. Domain Events

May be durable.

---

# 164. Bounded Channels

Mandatory.

---

# 165. Tokio Channel

No unbounded hot path.

---

# 166. Queue Fairness

Per tenant/domain/class.

---

# 167. Fair Scheduling

Possible:

```text
deficit round robin
weighted fair queue
```

---

# 168. No One Tenant Starvation

Hard rule.

---

# 169. Priority Inversion

Prevent.

---

# 170. Security-control event

Must not wait behind bulk backlog.

---

# 171. Resource Budget

Part 63.

---

# 172. Queue Storage

Database/KV-backed.

---

# 173. Durable Queue Storage Requirements

```text
atomic enqueue
lease/claim
ack
retry metadata
```

---

# 174. Queue Lease

```rust
pub struct QueueLease {
    pub job: JobId,
    pub consumer: ConsumerId,
    pub expires_at: LeaseDeadline,
}
```

---

# 175. Lease Expiry

Job becomes visible again.

---

# 176. No Permanent Hidden InFlight Job

Hard rule.

---

# 177. Visibility Timeout

Configured by job type.

---

# 178. Heartbeat

Optional for long jobs.

---

# 179. Long Job

Prefer workflow step with checkpoint.

---

# 180. Queue Claim

Atomic compare-and-set/transaction.

---

# 181. Duplicate Claim

Prevented or harmless via idempotency.

---

# 182. Queue Sharding

Partition by:

```text
tenant
workflow
service
```

---

# 183. No Global User Shard Key

Hard rule.

---

# 184. Broker Technology

Possible:

```text
PostgreSQL queue
embedded durable queue
NATS JetStream
Kafka-like system
custom Rust queue
```

---

# 185. No Mandatory Broker Product

Hard rule.

---

# 186. Selection Based On

```text
ordering
durability
throughput
operational complexity
privacy
```

---

# 187. Initial Recommendation

Use simplest engine that satisfies invariants.

---

# 188. Small/Self-Hosted

PostgreSQL-backed durable queue can suffice.

---

# 189. Large Deployment

Dedicated streaming broker may be useful.

---

# 190. Client

Embedded queue in local DB.

---

# 191. Broker Abstraction

```rust
pub trait EventBus {
    fn publish(
        &self,
        event: EventEnvelope<Bytes>,
    ) -> Result<(), BusError>;

    fn subscribe(
        &self,
        subscription: Subscription,
    ) -> Result<Box<dyn EventStream>, BusError>;
}
```

---

# 192. Durable Queue Trait

```rust
pub trait DurableQueue {
    fn enqueue(
        &self,
        job: QueueJob<Bytes>,
    ) -> Result<(), QueueError>;

    fn claim(
        &self,
        consumer: ConsumerId,
    ) -> Result<Option<LeasedJob>, QueueError>;
}
```

---

# 193. Workflow Engine Trait

```rust
pub trait WorkflowEngine {
    fn start(
        &self,
        workflow: WorkflowDefinition,
    ) -> Result<WorkflowId, WorkflowError>;

    fn signal(
        &self,
        workflow: WorkflowId,
        event: WorkflowSignal,
    ) -> Result<(), WorkflowError>;
}
```

---

# 194. Workflow Definition

Versioned.

---

# 195. Workflow Version

```rust
pub struct WorkflowVersion(pub u16);
```

---

# 196. In-Flight Workflow Upgrade

Hard problem.

---

# 197. Recommendation

Pin workflow version at start.

---

# 198. New Workflow Uses New Version.

---

# 199. Migration

Explicit for long-lived workflow.

---

# 200. No Silent Definition Swap Mid-Workflow

Hard rule.

---

# 201. Workflow Compensation

Versioned.

---

# 202. Workflow Replay

Can reconstruct state from journal.

---

# 203. Deterministic Workflow Logic

Preferred.

---

# 204. External Side Effect

Must be recorded.

---

# 205. No Nondeterministic Hidden Side Effect During Replay

Hard rule.

---

# 206. Scheduled Workflow

Timer events durable.

---

# 207. Clock Skew

Part 60.

---

# 208. Timer semantics

Use monotonic/epoch where possible.

---

# 209. Wall-clock scheduled tasks

Need skew policy.

---

# 210. Message Bus Security

Authenticate producers/consumers.

---

# 211. Producer Identity

Service-scoped.

---

# 212. Consumer Identity

Service-scoped.

---

# 213. No User Messaging Identity

Hard rule.

---

# 214. Authorization

Topic/queue scoped.

---

# 215. Producer ACL

Explicit.

---

# 216. Consumer ACL

Explicit.

---

# 217. No Wildcard `*` For Critical Service

Hard rule.

---

# 218. Least Privilege

Each service reads/writes only required topics.

---

# 219. Broker Credential

Short-lived/scoped.

---

# 220. Credential Rotation

Part 61/66.

---

# 221. Encryption In Transit

Required.

---

# 222. Broker At Rest

Encrypted where sensitive metadata exists.

---

# 223. No Plaintext Secret In Event Payload

Hard rule.

---

# 224. Audit

Broker admin actions only.

---

# 225. No User Event Body Logging

Hard rule.

---

# 226. Observability

Safe metrics:

```text
queue depth
event rate
consumer lag
retry count
dead-letter count
```

---

# 227. Forbidden Metrics

No:

```text
message text
contact IDs
conversation names
raw mailbox IDs
```

---

# 228. Consumer Lag

Partition/service aggregate.

---

# 229. No Per-User Lag Dashboard

Hard rule.

---

# 230. Event Tracing

Privacy-sensitive.

---

# 231. Correlation ID

Scoped.

---

# 232. No Global Cross-Service User Correlation ID

Hard rule.

---

# 233. Workflow Correlation

WorkflowId only.

---

# 234. Trace Sampling

Infrastructure synthetic/aggregate.

---

# 235. No Full User Event Trace In Production

Hard rule.

---

# 236. Backpressure States

```rust
pub enum QueuePressure {
    Normal,
    Elevated,
    Saturated,
}
```

---

# 237. Saturated

Reject/defer low-priority producers.

---

# 238. No Memory Spill Explosion

Hard rule.

---

# 239. Spill To Disk

Only within bounded quota.

---

# 240. Queue Disk Full

Fail safely.

---

# 241. Security-Control Queue

Reserved capacity.

---

# 242. Emergency Queue

Reserved capacity.

---

# 243. Bulk Queue

First to throttle.

---

# 244. Queue Aging

Avoid starvation.

---

# 245. Priority Aging

Controlled.

---

# 246. But

Bulk never outranks security controls.

---

# 247. Retry Storm Protection

Central.

---

# 248. Circuit Breaker

For failing downstream.

---

# 249. Breaker State

```rust
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}
```

---

# 250. Open

Stop immediate retries.

---

# 251. HalfOpen

Probe.

---

# 252. No Retry Storm Across Fleet

Hard rule.

---

# 253. Rate Limits

Per producer/topic.

---

# 254. Queue Admission Control

Based on capacity.

---

# 255. No Unlimited Producer

Hard rule.

---

# 256. Workflow Examples

Tenant provisioning:

```text
allocate namespace
→ create key root
→ allocate storage
→ apply policy
→ activate
```

---

# 257. Provider Migration

```text
provision target
→ replicate state
→ dual receive
→ cutover
→ retire old
```

---

# 258. Account Deletion

```text
revoke access
→ delete active data
→ revoke keys
→ process backups
→ complete
```

---

# 259. Key Rotation

```text
generate next
→ distribute
→ overlap
→ prefer next
→ retire old
```

---

# 260. Release Rollout

```text
canary
→ verify
→ wave
→ verify
→ complete
```

---

# 261. Each Workflow

Durable, idempotent, restart-safe.

---

# 262. No In-Memory Orchestration For Critical Multi-Step Flow

Hard rule.

---

# 263. Event Store

Do not automatically store every event forever.

---

# 264. Retention

Part 67.

---

# 265. Event Retention Class

```rust
pub enum EventRetentionClass {
    Ephemeral,
    Short,
    Operational,
    Audit,
}
```

---

# 266. Domain Event Retention

Only as needed.

---

# 267. Integration Event

Until all consumers/retention window.

---

# 268. Audit Event

Longer if justified.

---

# 269. No Infinite Event History By Default

Hard rule.

---

# 270. Compaction

Possible for monotonic state.

---

# 271. But

Do not compact audit semantics incorrectly.

---

# 272. Event Replay

Allowed within retention.

---

# 273. Consumer Reset

High-risk.

---

# 274. Replay Could Duplicate side effects.

---

# 275. Replay Requires Idempotent consumer.

---

# 276. Hard rule.

---

# 277. Schema Registry

Event types/version registry.

---

# 278. Event Type ID

Stable.

---

# 279. No Dynamic Arbitrary Payload Type

Hard rule.

---

# 280. Registry Entry

```rust
pub struct EventSchemaDescriptor {
    pub event_type: EventTypeId,
    pub versions: VersionRange,
    pub privacy: EventPrivacyClass,
    pub retention: EventRetentionClass,
}
```

---

# 281. CI Gate

Every durable event has registry entry.

---

# 282. Unknown Durable Event

Consumer handles explicitly.

---

# 283. Integration Boundary

Use integration event, not DB CDC by default.

---

# 284. CDC

Change Data Capture may leak schema/internal data.

---

# 285. Hard rule

Do not expose raw CDC across trust boundaries.

---

# 286. Internal CDC

Possible for analytics/rebuildable systems.

---

# 287. But

Apply privacy review.

---

# 288. Analytics Bus

Separate from operational bus.

---

# 289. No User Content.

---

# 290. Command Bus

Could be internal.

---

# 291. But

Commands requiring immediate auth should be validated at ingress.

---

# 292. Command Handler

Checks:

```text
auth
capability
policy
idempotency
```

---

# 293. Command Cannot Grant Itself Authority

Hard rule.

---

# 294. External API

Part 68 translates request → command.

---

# 295. Internal Event

Never directly trusted as auth proof.

---

# 296. Hard rule.

---

# 297. Distributed Locking

Avoid if possible.

---

# 298. Prefer

```text
single partition owner
DB constraint
lease
compare-and-swap
```

---

# 299. Global Lock

Hard rule against.

---

# 300. Leader Election

Needed for some workflows.

---

# 301. Scope leader by domain.

---

# 302. No Global Coordinator If Avoidable

Hard rule.

---

# 303. Queue Lease + fencing token

Useful.

---

# 304. Fencing Token

```rust
pub struct FencingToken(pub u64);
```

---

# 305. Monotonic.

---

# 306. Prevent stale worker write.

---

# 307. Hard rule for exclusive workflows.

---

# 308. Workflow Ownership

Lease.

---

# 309. Lease Expiry

Another worker resumes.

---

# 310. State in durable journal.

---

# 311. Exactly-Once External Side Effect

Use external idempotency key if provider supports.

---

# 312. Otherwise

Need compensating/reconciliation.

---

# 313. No False Exactly-Once Claim

Hard rule.

---

# 314. Reconciliation

Part 46.

---

# 315. Workflow Reconciler

Checks stuck/inconsistent state.

---

# 316. Reconciliation Event

Could repair.

---

# 317. No Blind Auto-Repair For High-Risk Security State

Hard rule.

---

# 318. Human approval for critical correction.

---

# 319. Event Bus Failure

Critical.

---

# 320. Failure Modes

```text
broker down
partition
storage full
consumer lag
duplicate flood
schema incompatibility
```

---

# 321. Broker Down

Producer outbox continues locally until bounded limit.

---

# 322. No memory-only buffering.

---

# 323. Hard rule.

---

# 324. Consumer Down

Lag grows bounded.

---

# 325. Queue Saturated

Admission control.

---

# 326. Schema Incompatibility

Consumer quarantines event or deployment blocked.

---

# 327. No Silent Drop

Hard rule.

---

# 328. Event Corruption

Digest/checksum detect.

---

# 329. Corrupted Event

Do not apply.

---

# 330. Dead-letter/quarantine minimal metadata.

---

# 331. Broker Replication

Depends on criticality.

---

# 332. Control-plane event bus

HA.

---

# 333. Client local bus

single device.

---

# 334. Federation queue

per peer.

---

# 335. Durability Requirement

```rust
pub enum EventDurability {
    Ephemeral,
    LocalDurable,
    ReplicatedDurable,
}
```

---

# 336. Security-control events

Replicated where infrastructure requires.

---

# 337. UI events

Ephemeral.

---

# 338. Work queue jobs

Local/replicated durable.

---

# 339. Event Encryption

Could use payload envelope.

---

# 340. Key Scope

Service/tenant.

---

# 341. No Shared Global Event Encryption Key

Hard rule.

---

# 342. Multi-Tenant Bus

Tenant-specific encryption key if required.

---

# 343. Queue Metadata Leakage

Minimize.

---

# 344. Broker may see:

```text
topic
size
timing
```

---

# 345. Do not put sensitive semantics in topic name.

---

# 346. Hard rule.

---

# 347. Topic IDs opaque in production.

---

# 348. Event Size

Bounded.

---

# 349. Large Payload

Store object separately; event contains reference.

---

# 350. No Multi-MB Event Payload

Hard rule.

---

# 351. Max Event Size

```rust
pub struct EventSizePolicy {
    pub max_bytes: usize,
}
```

---

# 352. Attachment/Event

Use blob ref.

---

# 353. Serialization

Postcard preferred internally.

---

# 354. JSON only external interop.

---

# 355. Versioned envelope.

---

# 356. Strict decode bounds.

---

# 357. No Serde Untagged Ambiguity For Security-Critical Events

Preferred hard rule.

---

# 358. Event Registry

Compile-time/generated enum where feasible.

---

# 359. Performance

Part 63.

---

# 360. Measure:

```text
publish latency
consumer lag
queue throughput
retry rate
workflow completion time
```

---

# 361. Tail Latency

Important.

---

# 362. But

Throughput cannot justify dropping security events.

---

# 363. Batch Publish

Allowed.

---

# 364. Preserve per-partition ordering where required.

---

# 365. Batch Consume

Allowed.

---

# 366. Transaction boundaries clear.

---

# 367. Batch Failure

Retry idempotently.

---

# 368. No Partial Ack Ambiguity

Hard rule.

---

# 369. Consumer Concurrency

Per partition.

---

# 370. Parallelism bounded.

---

# 371. Work-Stealing

Possible for independent partitions.

---

# 372. No Cross-Tenant fairness collapse.

---

# 373. Broker Choice Criteria

```text
durability
ordering
operational simplicity
Rust client quality
offline/self-host support
```

---

# 374. Recommendation

Do not introduce Kafka-class infrastructure before scale requires it.

---

# 375. Small deployments

PostgreSQL durable queue + local Tokio channels.

---

# 376. Larger deployments

NATS JetStream/Kafka-like options can be adapter implementations.

---

# 377. Core must remain broker-agnostic.

---

# 378. Event Bus Capability Model

```rust
pub struct BusCapabilities {
    pub durable: bool,
    pub partition_ordering: bool,
    pub consumer_groups: bool,
    pub delayed_delivery: bool,
    pub transactions: bool,
}
```

---

# 379. Domain Requirement

Explicit.

---

# 380. No Adapter Used If Missing Required Capability

Hard rule.

---

# 381. Observability Integration

Part 51.

---

# 382. Queue Metrics

Aggregate.

---

# 383. Alert

```text
lag high
retry storm
dead-letter growth
disk saturation
```

---

# 384. Alert Must Link Runbook

Part 65.

---

# 385. No payload in alert.

---

# 386. Testing

Dedicated event/workflow testkit.

---

# 387. Test Scenarios

```text
duplicate delivery
reordering
consumer crash
producer crash
broker outage
queue full
workflow restart
```

---

# 388. Duplicate Test

One semantic effect.

---

# 389. Producer Crash

Committed outbox publishes later.

---

# 390. Consumer Crash

Unacked event redelivered.

---

# 391. ACK Crash Race

No lost event.

---

# 392. Reordering Test

Consumer remains correct if order guarantee says none.

---

# 393. Partition Order Test

Sequence preserved per partition.

---

# 394. Queue Full Test

Producer backpressure/rejects safely.

---

# 395. Retry Storm Test

Backoff/jitter works.

---

# 396. Dead-Letter Test

No indefinite sensitive payload retention.

---

# 397. Workflow Crash Test

Resume from journal.

---

# 398. Compensation Test

Idempotent.

---

# 399. Timeout Test

Correct state transition.

---

# 400. Tenant Isolation Test

Tenant A event cannot reach B consumer.

---

# 401. Federation Isolation Test

Remote domain gets only integration event projection.

---

# 402. Schema Evolution Test

N-1 consumer handles supported version.

---

# 403. Large Payload Test

Rejected/reference path used.

---

# 404. Security Event Priority Test

Security queue not starved by bulk.

---

# 405. Fuzzing

Fuzz:

```text
event envelope
command envelope
workflow journal
queue job
schema registry
```

---

# 406. Property Tests

Properties:

```text
ack never precedes durable consumer commit
same event applied twice has one semantic effect
workflow state never advances past failed required step
tenant event never escapes tenant scope
```

---

# 407. Formal Verification Targets

Strong candidates:

```text
outbox/inbox delivery
queue lease/fencing
workflow/saga state machine
retry/timeout state
```

---

# 408. TLA+ Candidate

At-least-once delivery + consumer crash.

---

# 409. Kani Candidate

queue state/lease/fencing arithmetic.

---

# 410. Loom Candidate

concurrent consumer claim/ack.

---

# 411. Chaos

Part 64/70.

---

# 412. Broker Partition

Exercise.

---

# 413. Region Loss

Queue continuity/failover.

---

# 414. Control Plane Loss

Local outbox continues bounded.

---

# 415. No silent event loss.

---

# 416. Disaster Recovery

Event bus state backup depending authority.

---

# 417. Queue Recovery

Must not replay beyond retention incorrectly.

---

# 418. Workflow Recovery

Journal authoritative.

---

# 419. No reconstruct from logs alone.

---

# 420. Supply Chain

Part 73.

---

# 421. Broker client/runtime dependencies pinned.

---

# 422. Crate Layout

Recommended:

```text
crates/
├── siar-event-core/
├── siar-command-core/
├── siar-event-registry/
├── siar-event-bus/
├── siar-durable-queue/
├── siar-outbox/
├── siar-inbox/
├── siar-workflow/
├── siar-scheduler/
├── siar-event-observability/
└── siar-event-testkit/
```

---

# 423. `siar-event-core`

Owns:

```text
event IDs
envelopes
ordering
durability
errors
```

---

# 424. `siar-command-core`

Command envelopes/idempotency.

---

# 425. `siar-event-registry`

Schema/version/privacy/retention metadata.

---

# 426. `siar-event-bus`

Broker abstraction/adapters.

---

# 427. `siar-durable-queue`

Lease/claim/ack/retry.

---

# 428. `siar-outbox`

Transactional publish intent.

---

# 429. `siar-inbox`

Consumer dedup/receipt.

---

# 430. `siar-workflow`

Saga/workflow/journal.

---

# 431. `siar-scheduler`

Durable timers/delayed jobs.

---

# 432. `siar-event-observability`

Privacy-safe lag/queue metrics.

---

# 433. `siar-event-testkit`

Crash/duplicate/reorder/fault simulation.

---

# 434. Error Taxonomy

```rust
pub enum BusError {
    Unavailable,
    Unauthorized,
    TopicDenied,
    SchemaUnsupported,
    EventTooLarge,
    PublishFailed,
    Internal,
}

pub enum QueueError {
    QueueFull,
    LeaseConflict,
    JobNotFound,
    AckInvalid,
    RetryExhausted,
    Internal,
}

pub enum WorkflowError {
    WorkflowNotFound,
    VersionMismatch,
    InvalidTransition,
    StepFailed,
    CompensationFailed,
    Timeout,
    Internal,
}
```

---

# 435. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. Durable domain events are published through transactional outbox semantics after the corresponding domain commit.
2. Consumers ACK only after durable idempotent apply/receipt commit.
3. At-least-once delivery is the default; exactly-once semantic effects are achieved through idempotency, not marketing claims.
4. Every critical queue is bounded by count, bytes, and/or age; overload causes backpressure rather than unbounded memory/disk growth.
5. Commands, events, jobs, and workflow states are distinct typed concepts.
6. Critical multi-step workflows are journaled durably and survive process/node restart.
7. Tenant/federation boundaries are explicit in queue/topic/consumer scope; there is no global cross-tenant bus.
8. Event payloads are purpose-specific and minimal; server-side buses never carry E2EE plaintext.
9. Dead-letter/quarantine handling cannot become an indefinite archive of sensitive payloads.
10. Ordering guarantees are explicit and scoped; total ordering is never assumed globally.
11. Security-control/emergency queues retain reserved capacity and cannot be starved by bulk/background traffic.
12. Broker/runtime failures never silently authorize privacy downgrade or direct-routing bypass.
```

---

# 436. Initial Production Scope

Implement first:

```text
typed event/command envelopes
event schema registry
transactional outbox/inbox
embedded client durable queue
PostgreSQL-backed server durable queue
bounded Tokio local channels
partition ordering
retry/backoff/jitter
queue leases
dead-letter minimal metadata
durable workflow journal
durable timers
tenant-scoped queue/topic namespaces
privacy-safe queue observability
event/workflow contract tests
```

Then add:

```text
dedicated streaming broker adapters
distributed workflow sharding
advanced delayed scheduling
formal queue/workflow verification
cross-region event replication
adaptive queue fairness
```

---

# 437. Definition of Done

Part 75 is complete when:

- commands, events, jobs, audit records, and workflow state are clearly separated
- durable domain events use transactional outbox
- consumers are idempotent and ACK after commit
- queue/topic scopes are bounded and tenant-aware
- partition ordering rules are explicit
- retries are bounded and jittered
- durable timers survive restart
- critical workflows use durable journals/sagas
- dead-letter handling is minimal and lifecycle-bound
- server buses never carry E2EE plaintext
- event schema/privacy/retention metadata is machine-readable
- security/emergency queues cannot be starved
- broker choice is replaceable behind capability-validated adapters
- duplicate/reordering/crash/partition/tenant/schema/fuzz/formal tests are specified

---

# 438. Final Architecture

```text
                      DOMAIN TRANSACTION
                              │
                              ▼
                       OUTBOX RECORD
                              │
                              ▼
                    DURABLE EVENT BUS
                              │
          ┌───────────────────┼───────────────────┐
          │                   │                   │
      Consumer A          Consumer B         Workflow Engine
          │                   │                   │
          ▼                   ▼                   ▼
      Inbox/Dedup         Inbox/Dedup        Workflow Journal
          │                   │                   │
          └───────────────────┼───────────────────┘
                              ▼
                     IDEMPOTENT SIDE EFFECTS
```

Distributed-workflow safety model:

```text
transactional outbox
+
at-least-once delivery
+
idempotent consumers
+
bounded durable queues
+
explicit ordering
+
durable workflow journals
+
privacy-scoped event payloads
```

not:

```text
fire events into an unbounded bus and assume every consumer sees them once and in order
```

---

# 439. Final Principle

The internal event fabric should decouple services without becoming a global metadata spine.

The correct model is:

```text
explicit commands
+
durable facts
+
bounded queues
+
idempotent consumers
+
scoped workflows
+
privacy-safe observability
```

This architecture gives SIAR a reliable asynchronous coordination layer for messaging, tenants, federation, provisioning, recovery, key rotation, deployment, and long-running workflows while preserving the transaction, privacy, and isolation guarantees established across Parts 34–74.
