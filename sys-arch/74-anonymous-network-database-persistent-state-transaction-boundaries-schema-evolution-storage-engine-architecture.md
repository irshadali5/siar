# Core System Architecture Part 74 — Anonymous Network Database, Persistent State, Transaction Boundaries, Schema Evolution & Storage-Engine Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 74  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 4–6, 9, 31–33, 35, 40, 46–50, 57, 61–73  

**Primary purpose:** define SIAR's database and persistent-state architecture across client devices, provider infrastructure, control-plane services, mailboxes, federation gateways, tenants, and archival systems, including authoritative vs derived state, transactional boundaries, storage-engine abstractions, idempotency, schema evolution, migrations, crash recovery, retention/deletion integration, encryption, replication, and operational correctness.

---

# 1. Purpose

Persistent state is where correctness becomes durable.

A distributed anonymous system may correctly process a message in memory and still fail if persistence semantics are wrong.

Failure examples:

```text
message sent before durable write
duplicate delivery after crash
stale schema during rolling upgrade
deleted object resurrected by old replica
tenant data queried without scope
mailbox ACK returned before storage commit
migration partially applied
cache treated as authority
```

The governing principle is:

> **Persistent state must have explicit ownership, authority, transaction boundaries, versioning, and crash-recovery semantics. Storage engines are replaceable mechanisms; invariants are not.**

---

# 2. Architectural Position

```text
Domain / Protocol State Machine
            │
            ▼
    Persistence Interfaces
            │
     ┌──────┼──────┐
     │      │      │
    SQL     KV    Object Store
     │      │      │
     └──────┼──────┘
            ▼
   Durable / Derived State
```

---

# 3. Core Separation

Keep distinct:

```text
authoritative state
derived state
ephemeral state
cache
index
archive
backup
replica
```

---

# 4. State Authority Classes

```rust
pub enum StateAuthority {
    Authoritative,
    Derived,
    Ephemeral,
    Rebuildable,
}
```

---

# 5. Authoritative

Loss affects correctness.

Examples:

```text
account authority state
outbox
mailbox object metadata
tenant policy
governance policy version
```

---

# 6. Derived

Can be rebuilt from authoritative source.

Examples:

```text
search index
materialized view
cached route score
```

---

# 7. Ephemeral

Never part of durable truth.

Examples:

```text
typing
presence
in-flight retry timer
```

---

# 8. Rebuildable

Persisted for performance but disposable.

Examples:

```text
thumbnail cache
denormalized metrics
```

---

# 9. Hard Rule

No component may treat cache/index state as authoritative unless explicitly declared.

---

# 10. Persistent State Registry

Machine-readable registry.

```rust
pub struct PersistentStateDescriptor {
    pub state_type: StateTypeId,
    pub authority: StateAuthority,
    pub durability: DurabilityRequirement,
    pub lifecycle: LifecyclePolicyId,
    pub encryption: EncryptionRequirement,
}
```

---

# 11. No Undeclared Durable State

Hard rule.

---

# 12. Storage Domains

```rust
pub enum StorageDomain {
    ClientLocal,
    MailboxProvider,
    RelayState,
    Directory,
    ControlPlane,
    Federation,
    Tenant,
    Archive,
}
```

---

# 13. Client Local Store

Stores:

```text
message history
conversation metadata
outbox
receipts
contacts
device state
```

---

# 14. Provider Store

Stores minimum opaque service state.

---

# 15. Control Plane Store

Stores:

```text
infrastructure state
config versions
deployment state
tenant metadata
```

---

# 16. No User Content In Control Plane

Hard rule.

---

# 17. Storage Engine Classes

```rust
pub enum StorageEngineClass {
    Sql,
    KeyValue,
    Object,
    AppendOnlyLog,
}
```

---

# 18. SQL

Best for:

```text
relational metadata
transactions
constraints
queries
migrations
```

---

# 19. KV

Best for:

```text
high-throughput lookup
small scoped records
caches
replay windows
```

---

# 20. Object Storage

Best for:

```text
attachments
encrypted large blobs
backups
archives
```

---

# 21. Append-Only Log

Best for:

```text
audit
event sourcing in narrow domains
transparency logs
```

---

# 22. No One-Database Dogma

Hard rule.

---

# 23. Engine Choice

Driven by invariants.

---

# 24. SQL Baseline

PostgreSQL for infrastructure/control-plane metadata is a strong default.

---

# 25. Client SQL

SQLite/rusqlite-like embedded SQL can be used.

---

# 26. Alternative Embedded DB

Allowed if ACID, migration, corruption-recovery, and encryption requirements are met.

---

# 27. Domain Independence

Core domain crates do not import database driver types.

---

# 28. Repository Trait

```rust
pub trait Repository<T> {
    type Id;

    fn get(&self, id: &Self::Id) -> Result<Option<T>, StorageError>;
}
```

---

# 29. Transaction Abstraction

```rust
pub trait StorageTransaction {
    fn commit(self) -> Result<(), StorageError>;
    fn rollback(self) -> Result<(), StorageError>;
}
```

---

# 30. Transaction Boundary

Defined by domain invariant.

---

# 31. Example: Send Message

Must atomically persist:

```text
message record
outbox entry
attachment references
```

before transmission starts.

---

# 32. Persist-Before-Send

Hard rule.

---

# 33. Message Send Transaction

Conceptually:

```rust
pub struct PersistedSendIntent {
    pub message: MessageRecord,
    pub outbox: OutboxRecord,
    pub attachment_refs: Vec<AttachmentRef>,
}
```

---

# 34. Commit Then Dispatch

```text
begin tx
→ insert message
→ insert outbox
→ update conversation sequence
→ commit
→ signal transport worker
```

---

# 35. No Send Inside Transaction

Preferred.

---

# 36. Why

Network call is nondeterministic/slow.

---

# 37. Transactional Outbox

Core pattern.

---

# 38. Outbox State

```rust
pub enum OutboxState {
    Pending,
    InFlight,
    Delivered,
    FailedRetryable,
    FailedPermanent,
}
```

---

# 39. Crash Recovery

`Pending` survives restart.

---

# 40. InFlight Recovery

On restart:

```text
requeue with idempotency/dedup
```

---

# 41. Delivery ACK

Must map to stable MessageId.

---

# 42. Inbox Pattern

Receiver persists before app-level ACK.

---

# 43. Receive Transaction

```text
validate
→ dedup check
→ persist message
→ persist delivery state
→ commit
→ send app-level ACK
```

---

# 44. ACK-Before-Commit Forbidden

Hard rule.

---

# 45. Idempotency

Mandatory.

---

# 46. Idempotency Key

Protocol-generated stable operation/message ID.

---

# 47. Idempotency Record

```rust
pub struct IdempotencyRecord {
    pub operation: OperationId,
    pub result_digest: Option<ContentDigest>,
}
```

---

# 48. Idempotent Apply

Duplicate operation yields same semantic result.

---

# 49. No Duplicate Side Effect

Hard rule.

---

# 50. Deduplication Scope

Protocol-specific.

---

# 51. Message Dedup

By MessageId.

---

# 52. Payment/Credit Dedup

By SpendId/OperationId.

---

# 53. Admin Command Dedup

By AdminCommandId.

---

# 54. Schema Version

Every durable schema versioned.

```rust
pub struct SchemaVersion(pub u32);
```

---

# 55. Migration Version

Separate.

```rust
pub struct MigrationVersion(pub u32);
```

---

# 56. Data Format Version

Separate from DB schema version.

---

# 57. Protocol Version

Separate from DB schema version.

---

# 58. Hard Rule

Do not conflate API, protocol, and storage schema versions.

---

# 59. Schema Evolution Principle

Use:

```text
expand
→ migrate
→ contract
```

---

# 60. Expand

Add compatible schema first.

---

# 61. Migrate

Backfill/dual-write if necessary.

---

# 62. Contract

Remove old schema only after fleet compatibility window.

---

# 63. Rolling Upgrade

Mixed-version nodes expected.

---

# 64. Backward Read Compatibility

Needed during rollout.

---

# 65. Forward Read Compatibility

Useful where practical.

---

# 66. No Breaking Schema Change In Single Step

Hard rule for shared production stores.

---

# 67. Migration Metadata

```rust
pub struct MigrationDescriptor {
    pub from: SchemaVersion,
    pub to: SchemaVersion,
    pub reversible: bool,
    pub online: bool,
}
```

---

# 68. Reversible

Explicit.

---

# 69. Irreversible Migration

Requires backup/recovery plan.

---

# 70. Migration State

```rust
pub enum MigrationState {
    Pending,
    Running,
    Validating,
    Completed,
    Failed,
}
```

---

# 71. Migration Lock

Prevent concurrent incompatible migration.

---

# 72. Advisory Lock / Leader

Possible.

---

# 73. No Per-Request Auto-Migration

Hard rule.

---

# 74. Client DB Migration

Runs at startup/update.

---

# 75. Client Migration Requirements

```text
transactional where possible
crash-safe
backup or rollback strategy
```

---

# 76. Migration Journal

Durable.

---

# 77. Crash Mid-Migration

Must resume or rollback.

---

# 78. Hard Rule

Unknown partial migration state cannot open app normally.

---

# 79. Read-Only Recovery Mode

Useful.

---

# 80. Schema Compatibility Gate

```rust
pub trait SchemaCompatibilityGate {
    fn check(
        &self,
        binary: SoftwareVersion,
        schema: SchemaVersion,
    ) -> Result<(), StorageError>;
}
```

---

# 81. Binary Refuses Unsupported Schema

Hard rule.

---

# 82. Data Migration Backfill

Chunked.

---

# 83. Why

Avoid giant transactions.

---

# 84. Backfill Checkpoint

```rust
pub struct MigrationCheckpoint {
    pub migration: MigrationVersion,
    pub cursor: Option<OpaqueCursor>,
}
```

---

# 85. Online Migration

Rate-limited.

---

# 86. Production Traffic Priority

Higher than backfill.

---

# 87. Migration Observability

Aggregate.

---

# 88. No user-content logging.

---

# 89. Transaction Isolation

Choose by invariant.

---

# 90. PostgreSQL Isolation Levels

Possible:

```text
Read Committed
Repeatable Read
Serializable
```

---

# 91. Default

Read Committed may suffice for simple CRUD.

---

# 92. Stronger Invariants

Use:

```text
constraints
locking
serializable transaction
```

---

# 93. Example

One-time credit spend:

```text
must prevent double spend
```

---

# 94. Unique Constraint

Prefer DB-enforced invariant.

---

# 95. Compiler + DB

Use both.

---

# 96. No Application-Only Uniqueness

Hard rule where DB can enforce.

---

# 97. Constraint Types

```text
primary key
unique
foreign key
check
not null
```

---

# 98. Database Constraint

Part of domain safety.

---

# 99. Foreign Keys

Useful, but avoid cross-tenant leakage.

---

# 100. Tenant-Scoped Composite Key

Example:

```text
(tenant_id, object_id)
```

---

# 101. Multi-Tenant Isolation

Part 69.

---

# 102. Tenant Scope Required In Key

For shared stores.

---

# 103. RLS

Defense in depth.

---

# 104. No Query Without Tenant Context

Hard rule.

---

# 105. Connection Pool

Bounded.

---

# 106. Pool Trait

```rust
pub trait ConnectionPool {
    fn acquire(&self) -> Result<DbConnection, StorageError>;
}
```

---

# 107. No Unbounded Pool

Hard rule.

---

# 108. Pool Backpressure

Required.

---

# 109. Transaction Timeout

Required.

---

# 110. Long Transactions

Avoid.

---

# 111. Why

Lock contention/vacuum issues.

---

# 112. Large Object Transfer

Not inside SQL transaction.

---

# 113. Two-Phase App Workflow

Example attachment:

```text
upload encrypted blob
→ persist manifest reference transactionally
```

---

# 114. Orphan Blob

GC later.

---

# 115. No Blob Commit Coupled To Remote Store Transaction

Hard rule.

---

# 116. Saga Pattern

For cross-store operations.

---

# 117. Saga State

```rust
pub enum SagaState {
    Started,
    StepCompleted(u32),
    Compensating,
    Completed,
    Failed,
}
```

---

# 118. Saga Journal

Durable.

---

# 119. Idempotent Steps

Mandatory.

---

# 120. Compensation

Best effort, domain-safe.

---

# 121. No Distributed 2PC Requirement

Preferred.

---

# 122. Why

Complexity/availability.

---

# 123. Use Local Transaction + Outbox + Saga

Recommended.

---

# 124. Object Store Consistency

Must understand provider semantics.

---

# 125. Object Address

Content-addressed or opaque random ID.

---

# 126. Sensitive Mode

Opaque random object ID often better.

---

# 127. Content Addressing

May leak equality.

---

# 128. Strict Privacy

Disable cross-user dedup.

---

# 129. Blob Metadata DB

Authoritative reference metadata in SQL/KV.

---

# 130. Blob Bytes

Object store.

---

# 131. Attachment Transaction

Persist:

```text
manifest
chunk refs
ownership refs
```

---

# 132. Garbage Collection

Part 67.

---

# 133. Refcount

If used, update transactionally.

---

# 134. Refcount Race

Must test.

---

# 135. Prefer Mark/Sweep For Complex Sharing

Where safer.

---

# 136. Mailbox Store

Requires:

```text
durable deposit
bounded TTL
idempotent fetch ACK
```

---

# 137. Mailbox Deposit Transaction

```text
validate quota
→ store ciphertext
→ insert metadata
→ commit
→ return deposit ACK
```

---

# 138. Deposit ACK Before Commit Forbidden

Hard rule.

---

# 139. Mailbox Fetch

Read without deleting immediately.

---

# 140. Fetch ACK

Separate.

---

# 141. Delete After Client Durable Persist

Preferred.

---

# 142. At-Least-Once Delivery

Acceptable with dedup.

---

# 143. Exactly-Once

Do not claim globally.

---

# 144. Hard truth.

---

# 145. Directory Store

Mostly signed snapshots/state.

---

# 146. Append-Only History

Useful for audit.

---

# 147. Snapshot State

Current authoritative view.

---

# 148. Transparency Log

Append-only.

---

# 149. Governance Store

Strong consistency.

---

# 150. Quorum decisions.

---

# 151. No Eventually Consistent Governance Authority State

Hard rule.

---

# 152. Control Plane Store

Strongly consistent metadata preferred.

---

# 153. Runtime Nodes

Consume signed cached config.

---

# 154. Control Plane Unavailable

No unsafe write-through fallback.

---

# 155. Federation Store

Separate per peer/domain.

---

# 156. No One Global Federation DB

Hard rule.

---

# 157. Federation Queue

Durable.

---

# 158. Per-Domain Bulkhead.

---

# 159. Retry State

Stored.

---

# 160. No Cross-Domain User Identity Table

Hard rule.

---

# 161. Client Local-First Database

Core offline truth.

---

# 162. Client DB Stores

```text
messages
conversations
contacts
outbox
receipts
local settings
```

---

# 163. Client DB Encryption

Required for sensitive durable state.

---

# 164. Key Source

Device secure store.

---

# 165. No DB Encryption Key In DB

Hard rule.

---

# 166. Database File

Encrypted at rest.

---

# 167. Attachments

Separate encrypted blob store.

---

# 168. Search Index

Derived.

---

# 169. FTS

Can be separate table/index.

---

# 170. Deleted Message

Must remove from FTS.

---

# 171. Embeddings

Derived and lifecycle-bound.

---

# 172. No Durable AI Index Without Lifecycle Policy

Hard rule.

---

# 173. Event Log

Part 4.

---

# 174. Event Sourcing

Use selectively.

---

# 175. Good Candidates

```text
sync history
audit
governance transitions
```

---

# 176. Not Every Domain

Hard rule.

---

# 177. Event Record

```rust
pub struct DurableEvent {
    pub event_id: EventId,
    pub version: EventSchemaVersion,
    pub payload: Bytes,
}
```

---

# 178. Event Schema

Versioned.

---

# 179. Event Replay

Deterministic.

---

# 180. Snapshot

Can accelerate.

---

# 181. Snapshot Must Be Derivable

For event-sourced domain.

---

# 182. No Event Log Containing Sensitive Plaintext By Default

Hard rule.

---

# 183. WAL

Database internal.

---

# 184. Backup Implication

WAL may contain deleted/old data.

---

# 185. Encryption Important.

---

# 186. PostgreSQL WAL

Treat as sensitive durable storage.

---

# 187. SQLite WAL

Likewise.

---

# 188. Checkpoint/Vacuum

Operational lifecycle.

---

# 189. Encryption At Rest

Layered.

---

# 190. Options

```text
database-level encryption
filesystem encryption
application-level field/blob encryption
```

---

# 191. Application-Level Encryption

Needed when storage provider must not see content.

---

# 192. Database Encryption

Protects lost disk.

---

# 193. Both Can Coexist.

---

# 194. Field-Level Encryption

Use carefully.

---

# 195. Searchability Tradeoff

Encrypted fields harder to query.

---

# 196. Prefer Minimal Server-Side Query On User Content

Hard rule.

---

# 197. Key Rotation

Part 66.

---

# 198. Rewrap DEKs when possible.

---

# 199. Database Key Rotation

Online where supported.

---

# 200. No Full-DB Downtime If Avoidable

Goal.

---

# 201. Backup Architecture

Part 33/67.

---

# 202. Logical Backup

Preferred for portability.

---

# 203. Physical Backup

Useful for infrastructure recovery.

---

# 204. Physical Backup Contains Engine internals.

---

# 205. User Content

Encrypted.

---

# 206. Backup Consistency

Use snapshot/transaction boundaries.

---

# 207. No Fuzzy Inconsistent Backup

Hard rule.

---

# 208. Point-In-Time Recovery

Useful for PostgreSQL.

---

# 209. PITR

Must respect deletion anti-resurrection.

---

# 210. Restore Filter/Deletion Ledger

Part 67.

---

# 211. No Restore To Old State Without Applying Revocations/deletions.

---

# 212. Hard rule.

---

# 213. Replication

Engine-level or application-level.

---

# 214. PostgreSQL Replication

Good for control-plane HA.

---

# 215. Mailbox Replication

May be application-aware.

---

# 216. Client Sync

Application-level.

---

# 217. Replication Factor

Policy.

---

# 218. No More Replication Than Needed

Privacy/cost.

---

# 219. Strong Consistency

Use where authority requires.

---

# 220. Eventual Consistency

Use where tolerant.

---

# 221. Consistency Requirement

```rust
pub enum ConsistencyRequirement {
    Strong,
    Causal,
    Eventual,
    LocalOnly,
}
```

---

# 222. Governance

Strong.

---

# 223. Tenant policy

Strong.

---

# 224. Presence

Ephemeral/eventual.

---

# 225. Search Index

Eventual.

---

# 226. Message Local History

Local strong.

---

# 227. Multi-Device Sync

Conflict-resolved eventual.

---

# 228. Conflict Resolution

Domain-specific.

---

# 229. No Generic Last-Write-Wins For All Data

Hard rule.

---

# 230. Why

Clock skew/data loss.

---

# 231. Conflict Policy Registry

```rust
pub enum ConflictPolicy {
    AppendOnly,
    MonotonicVersion,
    DomainMerge,
    TombstoneWins,
    LastWriterWinsAllowed,
}
```

---

# 232. TombstoneWins

For deletion.

---

# 233. MonotonicVersion

For policy/config.

---

# 234. AppendOnly

For audit/event log.

---

# 235. LWW

Only low-risk fields.

---

# 236. Timestamp

Not sole ordering authority.

---

# 237. Prefer

```text
sequence
epoch
version
```

---

# 238. Schema Ownership

Each crate/domain owns its schema.

---

# 239. No Giant Shared SQL Schema Ownership

Hard rule.

---

# 240. Migration Module

Lives with owning crate.

---

# 241. Cross-Domain Foreign Keys

Use sparingly.

---

# 242. Why

Tight coupling.

---

# 243. Integration Boundary

Use IDs/events.

---

# 244. Monolithic Workspace

Still modular schema ownership.

---

# 245. Storage Adapter

Per domain.

---

# 246. Example Layout

```text
crates/
  messaging-store/
  identity-store/
  mailbox-store/
  governance-store/
```

---

# 247. Database Connection Sharing

Infrastructure optimization.

---

# 248. Domain interface still separate.

---

# 249. Repository Error Model

```rust
pub enum StorageError {
    NotFound,
    Conflict,
    ConstraintViolation,
    Unavailable,
    Corruption,
    UnsupportedSchema,
    TransactionAborted,
    EncryptionFailure,
    Internal,
}
```

---

# 250. No Raw DB Error Leakage

Hard rule.

---

# 251. Error Normalization

Required.

---

# 252. Corruption Detection

Critical.

---

# 253. Checksums

Use where engine provides.

---

# 254. Application-Level Integrity

For encrypted blobs/manifests.

---

# 255. Corruption State

```rust
pub enum CorruptionState {
    None,
    Suspected,
    Confirmed,
}
```

---

# 256. Confirmed Corruption

Read-only/quarantine affected store.

---

# 257. No Continue Blindly

Hard rule.

---

# 258. Recovery

Restore/rebuild depending authority class.

---

# 259. Derived Store Corrupt

Rebuild.

---

# 260. Authoritative Store Corrupt

Recover from replica/backup.

---

# 261. Corruption Incident

Part 65/70.

---

# 262. Storage Health

Separate from service health.

---

# 263. Health Metrics

```text
commit latency
pool saturation
replication lag
disk usage
error rate
```

---

# 264. No User Query Logging

Hard rule.

---

# 265. Slow Query Logging

Use normalized query fingerprint.

---

# 266. No parameter values.

---

# 267. Query Plan Regression

Monitor.

---

# 268. Index Strategy

Evidence-driven.

---

# 269. No Over-Indexing Sensitive Fields

Hard rule.

---

# 270. Index Leakage

Indexes can reveal:

```text
existence
cardinality
frequency
```

---

# 271. Server-Side User Content Indexing

Minimize.

---

# 272. Client Search

Preferred.

---

# 273. Partitioning

Infrastructure DB.

---

# 274. Partition By

```text
tenant
time
provider
```

depending domain.

---

# 275. No Partition By Global User ID

Hard rule.

---

# 276. Mailbox Partition

By opaque mailbox shard.

---

# 277. Tenant DB Shard

TenantId.

---

# 278. Federation Queue Partition

FederationDomainId.

---

# 279. Shard Movement

Explicit migration.

---

# 280. No Ad-Hoc Resharding During Incident

Hard rule.

---

# 281. Capacity Planning

Part 63.

---

# 282. DB Sizing Inputs

```text
writes/s
reads/s
rows
bytes
retention
index overhead
replication
```

---

# 283. Storage Headroom

Required.

---

# 284. Disk Full

Must fail safely.

---

# 285. Mailbox Disk Full

Reject new deposits before corruption.

---

# 286. Client Disk Full

Keep existing data safe.

---

# 287. Migration Disk Headroom

Account for temporary duplication.

---

# 288. No Migration Without Space Preflight

Hard rule.

---

# 289. Vacuum/Compaction

Engine-specific.

---

# 290. Maintenance Scheduling

Low priority.

---

# 291. Compaction Privacy

No user-level metrics.

---

# 292. KV Compaction

Can create write amplification.

---

# 293. Benchmark.

---

# 294. Storage Engine Capability Model

```rust
pub struct StorageCapabilities {
    pub transactions: bool,
    pub snapshot_reads: bool,
    pub ttl: bool,
    pub atomic_compare_swap: bool,
    pub encryption_at_rest: bool,
}
```

---

# 295. Domain Requirement Check

```rust
pub trait StorageCapabilityValidator {
    fn validate(
        &self,
        requirements: &StorageRequirements,
        capabilities: &StorageCapabilities,
    ) -> Result<(), StorageError>;
}
```

---

# 296. No Engine Used If It Cannot Meet Invariant

Hard rule.

---

# 297. Pluggable Storage

Possible.

---

# 298. But

Not lowest-common-denominator semantics.

---

# 299. Domain States Requirements Explicitly.

---

# 300. Example

Mailbox requires:

```text
durable atomic metadata write
TTL or retention scheduler
idempotent key
```

---

# 301. Embedded Store Requirement

Client requires:

```text
ACID
crash recovery
migration
encryption
```

---

# 302. Store Replacement

Migration/export-import.

---

# 303. No Direct File Copy Between Different Engines

Hard rule.

---

# 304. Canonical Logical Export

Versioned.

---

# 305. Import Validation

Strict.

---

# 306. Store Adapter Version

```rust
pub struct StorageAdapterVersion(pub u16);
```

---

# 307. Data Access Layer

Use typed queries.

---

# 308. Raw SQL

Contained.

---

# 309. No SQL String Construction From User Input

Hard rule.

---

# 310. Prepared Queries

Preferred.

---

# 311. SQLx/Diesel/rusqlite

Implementation choice.

---

# 312. Pure Rust Preference

Use pure-Rust client libs where practical.

---

# 313. Native SQLite

May use FFI depending crate.

---

# 314. FFI isolated.

---

# 315. Migration Testing

Required.

---

# 316. Test Matrix

```text
fresh install
N-1→N
N-2→N where supported
crash mid migration
rollback
mixed version
```

---

# 317. Data Fixtures

Synthetic.

---

# 318. No Production DB Snapshot In CI

Hard rule.

---

# 319. Property Tests

Examples:

```text
persist-before-send
ACK-after-commit
tombstone beats stale write
tenant scope preserved
```

---

# 320. Fuzzing

Fuzz:

```text
migration metadata
logical export/import
event payload
storage adapter input
```

---

# 321. Fault Injection

Inject:

```text
disk full
fsync failure
connection loss
transaction abort
replica lag
```

---

# 322. Crash Tests

After each transaction step.

---

# 323. Concurrency Tests

Examples:

```text
double spend
duplicate message
delete vs update
migration vs request
```

---

# 324. Loom

Useful for in-memory transaction coordination.

---

# 325. Kani

Useful for version/constraint logic.

---

# 326. TLA+

Useful for:

```text
outbox/inbox retry
replicated delete
schema rolling upgrade
```

---

# 327. Benchmarking

Part 63.

---

# 328. Benchmark Realistic Security

Encryption enabled.

---

# 329. DB Performance Tests

Measure:

```text
commit p99
pool saturation
migration throughput
recovery time
```

---

# 330. Storage SLO

Per domain.

---

# 331. Data Lifecycle Integration

Part 67.

---

# 332. Every table/object namespace maps to lifecycle policy.

---

# 333. Retention Job

Uses lifecycle engine.

---

# 334. No ad-hoc cron delete logic.

---

# 335. Privacy Integration

Part 56.

---

# 336. Consent-controlled state has scope.

---

# 337. Legal Hold Integration

Part 55/67.

---

# 338. Hold only scoped records.

---

# 339. Multi-Tenant Integration

Part 69.

---

# 340. Tenant-scoped repositories.

---

# 341. Supply-Chain Integration

Part 73.

---

# 342. Migration binary/artifact exact and signed.

---

# 343. Attestation Integration

Part 71.

---

# 344. Critical DB host state verified.

---

# 345. Disaster Recovery Integration

Part 70.

---

# 346. Restore drills validate database recovery.

---

# 347. Database Secret Management

Part 61/72.

---

# 348. DB credentials scoped.

---

# 349. No Shared Superuser Credential

Hard rule.

---

# 350. Role Separation

```text
application role
migration role
backup role
admin role
```

---

# 351. Application Role

Least privilege.

---

# 352. Migration Role

Elevated, temporary.

---

# 353. Backup Role

Read only where possible.

---

# 354. Admin Role

Human/admin only.

---

# 355. No App Running As DB Owner

Hard rule.

---

# 356. Schema Ownership

Migration service/role.

---

# 357. Credential Rotation

Routine.

---

# 358. Connection Pool Reacts to rotation.

---

# 359. No Long-Lived Static Password

Preferred.

---

# 360. TLS

DB connections encrypted.

---

# 361. Mutual auth where practical.

---

# 362. Database Network

Private/internal.

---

# 363. No Public DB Port

Hard rule.

---

# 364. Client DB

Local only.

---

# 365. Database Observability

Safe metrics:

```text
query class latency
pool usage
replication lag
disk usage
migration state
```

---

# 366. Forbidden telemetry

No:

```text
message text
contact IDs
raw mailbox IDs
SQL parameter values
```

---

# 367. Database Audit

Infrastructure/admin actions.

---

# 368. No user query content.

---

# 369. Storage Privacy Invariants

Mandatory.

---

# 370. Examples

```text
server does not need plaintext to route encrypted message
search stays client-side where possible
cross-user blob dedup disabled strict mode
```

---

# 371. Initial Production Storage Profile

Recommended:

```text
Client:
  encrypted embedded SQL
  encrypted blob store
  derived FTS/index

Infrastructure:
  PostgreSQL for control-plane/relational authority
  KV where bounded high-throughput state needs it
  object storage for encrypted blobs/backups
```

---

# 372. No Mandatory Single Vendor

Hard rule.

---

# 373. Storage Portability

Logical schemas/adapters.

---

# 374. Data Export

RON/Postcard can be used for internal logical snapshots where appropriate.

---

# 375. JSON

Only external interoperability.

---

# 376. Postcard

Good for compact binary internal records.

---

# 377. RON

Good for human-reviewed metadata/config, not giant datasets.

---

# 378. DB Record Encoding

Native SQL columns preferred for queryable metadata.

---

# 379. Blob Column

Use for opaque versioned payload where appropriate.

---

# 380. No Whole Domain Row As Unversioned Blob

Hard rule.

---

# 381. Versioned Payload

```rust
pub struct VersionedBlob {
    pub version: u16,
    pub bytes: Vec<u8>,
}
```

---

# 382. Decode Bounds

Mandatory.

---

# 383. Migration Of Blob Formats

Separate from SQL schema migration.

---

# 384. Persistent State Ownership Graph

Machine-readable.

---

# 385. State Owner

```rust
pub struct StateOwnership {
    pub state_type: StateTypeId,
    pub owning_crate: CrateId,
    pub schema_version: SchemaVersion,
}
```

---

# 386. No Shared Anonymous Ownership

Hard rule.

---

# 387. Data Store Inventory

Part 67/73 style.

---

# 388. Inventory Includes

```text
engine
location
authority
encryption
retention
backup
```

---

# 389. Storage Inventory

No user data.

---

# 390. Storage Incident Classes

```rust
pub enum StorageIncident {
    Corruption,
    CapacityExhaustion,
    ReplicationFailure,
    MigrationFailure,
    CredentialCompromise,
    UnauthorizedAccess,
    DataResurrection,
}
```

---

# 391. Data Resurrection

Privacy/security critical.

---

# 392. Migration Failure

Freeze writes if needed.

---

# 393. Capacity Exhaustion

Degrade safely.

---

# 394. Unauthorized Access

Rotate credentials/investigate.

---

# 395. DB Compromise

Assume server-side metadata exposure.

---

# 396. Application-level encrypted payloads limit content exposure.

---

# 397. No Server DB Compromise Should Reveal E2EE Plaintext

Hard rule.

---

# 398. Storage Testkit

Dedicated.

---

# 399. Test Engines

At least:

```text
embedded SQL adapter
PostgreSQL adapter
object-store mock
```

---

# 400. Contract Tests

Same repository semantics across adapters.

---

# 401. Storage Contract

```rust
pub trait StorageContractTest {
    fn run<A: StorageAdapter>(
        adapter: A,
    ) -> ContractTestReport;
}
```

---

# 402. Contract Scenarios

```text
atomicity
idempotency
rollback
crash recovery
migration
```

---

# 403. No Adapter Accepted Without Contract Suite

Hard rule.

---

# 404. Crate Layout

Recommended:

```text
crates/
├── siar-storage-core/
├── siar-storage-registry/
├── siar-storage-sql/
├── siar-storage-kv/
├── siar-storage-object/
├── siar-transaction/
├── siar-outbox/
├── siar-inbox/
├── siar-schema/
├── siar-migration/
├── siar-storage-recovery/
├── siar-storage-observability/
└── siar-storage-testkit/
```

---

# 405. `siar-storage-core`

Owns:

```text
authority classes
capabilities
errors
```

---

# 406. `siar-storage-registry`

Persistent state ownership/requirements.

---

# 407. `siar-storage-sql`

SQL adapters.

---

# 408. `siar-storage-kv`

KV adapters.

---

# 409. `siar-storage-object`

Blob/object adapters.

---

# 410. `siar-transaction`

Transaction/saga abstractions.

---

# 411. `siar-outbox`

Transactional outbox.

---

# 412. `siar-inbox`

Dedup/persist/ACK.

---

# 413. `siar-schema`

Schema versions/compatibility.

---

# 414. `siar-migration`

Migration engine/checkpoints.

---

# 415. `siar-storage-recovery`

Corruption/restore/rebuild.

---

# 416. `siar-storage-observability`

Privacy-safe metrics.

---

# 417. `siar-storage-testkit`

Crash/fault/contract/migration tests.

---

# 418. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. Every durable state type has an explicit owner, authority class, lifecycle policy, and storage requirements.
2. Message send uses persist-before-send transactional outbox semantics.
3. Message/mailbox ACKs are never emitted before required durable commit.
4. All externally retried side effects are idempotent and deduplicated by stable operation/message IDs.
5. Cache/search/index state is never silently treated as authoritative.
6. Database/API/protocol schema versions are independent and migration-compatible.
7. Shared production schema changes follow expand→migrate→contract and support mixed-version rollout.
8. Deleted/tombstoned state cannot be resurrected by stale replicas, backups, or offline writes.
9. Tenant-scoped stores enforce tenant context structurally and through database policy where available.
10. Critical server compromise must not reveal E2EE plaintext stored only as encrypted payloads.
11. Application services do not run with database-owner/superuser privileges.
12. Storage-engine replacement is allowed only when the new engine satisfies declared domain invariants and passes the storage contract test suite.
```

---

# 419. Initial Production Scope

Implement first:

```text
persistent-state registry
embedded encrypted SQL client store
PostgreSQL infrastructure store
transactional outbox/inbox
stable MessageId/OperationId dedup
bounded connection pools
typed repositories
schema/version registry
expand-migrate-contract migration engine
migration checkpoints
object-store adapter
lifecycle/deletion hooks
tenant-scoped repository layer
backup/restore anti-resurrection
storage contract testkit
```

Then add:

```text
specialized KV adapters
online backfill controller
advanced cross-store saga engine
formal migration verification
automated storage capability negotiation
multi-engine portability tooling
```

---

# 420. Definition of Done

Part 74 is complete when:

- all durable state has an explicit authority classification
- domain crates are independent from specific database drivers
- message send/receive persistence is crash-safe
- outbox/inbox/idempotency semantics are explicit
- provider mailbox ACKs follow durable commit
- schema, migration, protocol, and API versions are separate
- expand→migrate→contract supports rolling upgrades
- migrations are crash-safe and checkpointed
- derived indexes/caches are rebuildable
- lifecycle/deletion/tombstone rules integrate with storage
- tenant-scoped repository isolation is structural
- storage encryption and credential separation are defined
- backups/restores cannot resurrect deleted/revoked state
- corruption/recovery procedures are explicit
- SQL/KV/object-store adapters are capability-validated and contract-tested
- database observability avoids user-content leakage

---

# 421. Final Architecture

```text
                     DOMAIN / PROTOCOL LOGIC
                              │
                              ▼
                    PERSISTENCE INTERFACES
                              │
               ┌──────────────┼──────────────┐
               │              │              │
              SQL            KV         Object Store
               │              │              │
               └──────────────┼──────────────┘
                              ▼
                  AUTHORITY / LIFECYCLE LAYER
                              │
                 ┌────────────┼────────────┐
                 │            │            │
             Outbox/Inbox  Migration    Retention
                 │            │            │
                 └────────────┼────────────┘
                              ▼
                       DURABLE STATE
```

Persistent-state safety model:

```text
explicit authority
+
typed repositories
+
transactional boundaries
+
idempotency
+
versioned migrations
+
encryption
+
lifecycle integration
+
crash recovery
```

not:

```text
write some rows, call the network, and hope retries or upgrades do not duplicate or lose state
```

---

# 422. Final Principle

The database is not the architecture.

The architecture is the set of durable invariants that every storage engine must preserve.

The correct model is:

```text
domain-owned state
+
explicit transaction boundaries
+
idempotent distributed workflows
+
versioned schemas
+
safe migration
+
privacy-aware storage
```

This architecture gives SIAR a durable, local-first, multi-engine storage foundation that supports clients, mailboxes, control-plane services, tenants, federation, backups, and archives without tying correctness to a single database product.
