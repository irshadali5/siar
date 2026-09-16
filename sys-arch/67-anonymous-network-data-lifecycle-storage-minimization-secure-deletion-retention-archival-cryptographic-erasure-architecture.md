# Core System Architecture Part 67 — Anonymous Network Data Lifecycle, Storage Minimization, Secure Deletion, Retention, Archival & Cryptographic Erasure Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 67  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 33, 35, 40, 47, 50, 55–66  

**Primary purpose:** define SIAR's complete data-lifecycle architecture, including minimization, storage classification, retention, deletion, tombstones, archival, cryptographic erasure, replica/backup handling, legal holds, data-rights integration, lifecycle enforcement, and storage-layer invariants across anonymous clients and infrastructure.

---

# 1. Purpose

Anonymous systems should minimize what they retain.

Every stored record creates:

```text
privacy risk
breach impact
legal exposure
operational cost
correlation potential
```

The governing principle is:

> **If SIAR does not need data, it should not collect it; if it no longer needs data, it should not retain it; and if data must be deleted, deletion semantics must be explicit, verifiable, and honest.**

---

# 2. Architectural Position

```text
Data Creation
    │
    ▼
Classification
    │
    ▼
Retention Policy
    │
    ├── active storage
    ├── ephemeral storage
    ├── archival storage
    └── no-storage path
    │
    ▼
Deletion / Expiry
    │
    ▼
Cryptographic Erasure / Physical Cleanup
```

---

# 3. Core Separation

Keep distinct:

```text
collection
processing
active storage
cache
archive
backup
replica
tombstone
legal hold
deletion
cryptographic erasure
```

---

# 4. Non-Goals

Part 67 does not promise:

```text
magical deletion from another user's device
physical NAND erase guarantees
instant purge from all cold backups
zero metadata retention where protocol correctness requires some
```

---

# 5. Data Lifecycle Classes

```rust
pub enum DataLifecycleClass {
    NeverPersist,
    Ephemeral,
    ShortLived,
    ActiveDurable,
    Archival,
    StatutoryRetention,
    LegalHold,
}
```

---

# 6. NeverPersist

Examples:

```text
plaintext transport buffers
ephemeral session secrets
decoded transient packet state
```

---

# 7. Ephemeral

Examples:

```text
presence cache
typing state
temporary routing state
```

---

# 8. ShortLived

Examples:

```text
replay cache
rate-limit state
temporary upload chunks
```

---

# 9. ActiveDurable

Examples:

```text
local message history
account settings
contact graph
provider configuration
```

---

# 10. Archival

Examples:

```text
user-selected long-term archive
signed governance history
release evidence
```

---

# 11. StatutoryRetention

Only where applicable.

---

# 12. LegalHold

Narrowly scoped preservation of existing records.

---

# 13. Data Classification

```rust
pub enum DataSensitivity {
    Public,
    Internal,
    Sensitive,
    HighlySensitive,
    Secret,
}
```

---

# 14. Lifecycle Policy

```rust
pub struct DataLifecyclePolicy {
    pub class: DataLifecycleClass,
    pub sensitivity: DataSensitivity,
    pub retention: RetentionPolicy,
    pub deletion: DeletionPolicy,
    pub replication: ReplicationPolicy,
}
```

---

# 15. Storage Minimization

Default stance:

```text
do not persist unless needed
```

---

# 16. Hard Rule

A component must justify persistence.

---

# 17. Persistence Declaration

```rust
pub struct PersistenceDeclaration {
    pub data_type: DataTypeId,
    pub purpose: DataPurpose,
    pub lifecycle: DataLifecyclePolicy,
}
```

---

# 18. No Undeclared Persistence

Hard rule.

---

# 19. Data Purpose

```rust
pub enum DataPurpose {
    ServiceOperation,
    Security,
    UserRequestedHistory,
    Backup,
    Archival,
    Compliance,
}
```

---

# 20. Purpose Limitation

Data collected for one purpose cannot silently be reused for another.

---

# 21. Collection Gate

```rust
pub trait DataCollectionGate {
    fn authorize(
        &self,
        data_type: DataTypeId,
        purpose: DataPurpose,
    ) -> Result<(), LifecycleError>;
}
```

---

# 22. Local-First Preference

Keep data on user device when remote storage not necessary.

---

# 23. Remote Storage

Requires explicit service need.

---

# 24. Storage Scope

```rust
pub enum StorageScope {
    MemoryOnly,
    DeviceLocal,
    ProviderLocal,
    RegionBound,
    ReplicatedService,
    ArchiveStore,
}
```

---

# 25. Default

`DeviceLocal` for user history.

---

# 26. Provider Stores Minimum

Prefer:

```text
opaque ciphertext
routing-minimal metadata
```

---

# 27. No Central User Profile Warehouse

Hard rule.

---

# 28. Active Data

Data currently needed for operation.

---

# 29. Archive Data

Separated from active DB.

---

# 30. Why Separate

Archive may have:

```text
different retention
different access
different encryption
different backup policy
```

---

# 31. Archive Store

```rust
pub struct ArchivePolicy {
    pub retention: RetentionPolicy,
    pub encryption_profile: CryptoProfile,
    pub access: ArchiveAccessPolicy,
}
```

---

# 32. No Automatic Archival

Hard rule for user content.

---

# 33. User-Selected Archive

Explicit.

---

# 34. Infrastructure Archive

Allowed for:

```text
governance
release evidence
audit
```

---

# 35. Retention Policy

```rust
pub enum RetentionPolicy {
    None,
    UntilProcessed,
    Duration(Duration),
    UntilEpoch(Epoch),
    UntilUserDeletes,
    IndefiniteByDesign,
}
```

---

# 36. `IndefiniteByDesign`

Only for narrow records like:

```text
public governance history
```

---

# 37. No Indefinite By Default

Hard rule.

---

# 38. Retention Deadline

Explicit.

---

# 39. Retention Clock

Part 60.

---

# 40. Retention Uses

```text
wall clock
epoch
monotonic TTL
```

depending class.

---

# 41. Retention Evaluator

```rust
pub trait RetentionEvaluator {
    fn status(
        &self,
        policy: &RetentionPolicy,
        context: &LifecycleTimeContext,
    ) -> RetentionStatus;
}
```

---

# 42. Retention Status

```rust
pub enum RetentionStatus {
    Active,
    Expired,
    Held,
    Unknown,
}
```

---

# 43. Unknown

Security/compliance policy decides.

---

# 44. Expiry

Expired data enters deletion queue.

---

# 45. Deletion Policy

```rust
pub enum DeletionPolicy {
    Immediate,
    Deferred(Duration),
    TombstoneThenDelete,
    CryptographicErase,
    LegalHoldAware,
}
```

---

# 46. Immediate

Delete promptly.

---

# 47. Deferred

Grace period.

---

# 48. TombstoneThenDelete

Needed for replicated/sync state.

---

# 49. CryptographicErase

Destroy decryption key.

---

# 50. LegalHoldAware

Deletion blocked only for specific held records.

---

# 51. Deletion State Machine

```rust
pub enum DeletionState {
    Requested,
    Eligible,
    Tombstoned,
    KeyRevoked,
    PhysicallyDeleted,
    Verified,
    BlockedByHold,
    Failed,
}
```

---

# 52. Deletion ID

Infrastructure/local object only.

---

# 53. No Global User Deletion ID

Hard rule.

---

# 54. Tombstones

Needed for sync convergence.

---

# 55. Tombstone Principle

Tombstone stores:

```text
minimal deletion marker
version/epoch
```

not deleted content.

---

# 56. Tombstone Type

```rust
pub struct Tombstone {
    pub object_ref: OpaqueObjectRef,
    pub deletion_epoch: u64,
    pub expires_at: Option<Timestamp>,
}
```

---

# 57. Tombstone Lifetime

Bounded.

---

# 58. No Permanent Tombstone Unless Protocol Requires

Hard rule.

---

# 59. Tombstone Privacy

Must not preserve original sensitive metadata unnecessarily.

---

# 60. Replicated Deletion

Need convergence.

---

# 61. Replica Delete Flow

```text
mark tombstone
→ propagate delete
→ confirm replicas
→ GC tombstone after safety window
```

---

# 62. Replica Acknowledgement

```rust
pub struct ReplicaDeletionAck {
    pub replica: ReplicaId,
    pub deletion_epoch: u64,
}
```

---

# 63. Partial Replica Failure

Deletion remains pending.

---

# 64. No False Completion

Hard rule.

---

# 65. Backup Deletion

Harder.

---

# 66. Backup Reality

Immutable backups may retain deleted ciphertext until backup expiry.

---

# 67. Honest Semantics

User-facing deletion should distinguish:

```text
active deleted
backup pending expiry
legal retention
```

---

# 68. Backup Tombstone

Can ensure deleted records are not restored.

---

# 69. Restore Filter

```rust
pub trait RestoreDeletionFilter {
    fn allow_restore(
        &self,
        object: OpaqueObjectRef,
        backup_epoch: BackupEpoch,
    ) -> bool;
}
```

---

# 70. No Resurrection

Hard rule.

---

# 71. Deletion Ledger

Minimal.

---

# 72. Purpose

Prevent restore of deleted/revoked state.

---

# 73. No Content In Ledger

Hard rule.

---

# 74. Cryptographic Erasure

Primary secure-deletion tool.

---

# 75. Principle

Encrypt data under scoped DEK.

Delete DEK to render ciphertext unreadable.

---

# 76. DEK

Data Encryption Key.

---

# 77. KEK

Key Encryption Key.

---

# 78. Key Hierarchy

```text
KEK
  └── DEK
       └── ciphertext
```

---

# 79. Per-Object DEK

High isolation.

---

# 80. Per-Batch DEK

Better performance, larger blast radius.

---

# 81. No One Global Data Key

Hard rule.

---

# 82. Crypto Erasure Type

```rust
pub struct CryptoEraseRequest {
    pub key_ref: DataKeyRef,
    pub reason: ErasureReason,
}
```

---

# 83. Erasure Reason

```rust
pub enum ErasureReason {
    UserDeletion,
    RetentionExpiry,
    Compromise,
    ArchiveExpiry,
    PolicyChange,
}
```

---

# 84. Key Destruction

Must include replicas/escrow copies.

---

# 85. HSM Keys

Delete key slot/version.

---

# 86. Secure Store

Destroy record.

---

# 87. Backup Key Copies

Need lifecycle.

---

# 88. Hard Rule

Crypto erasure is incomplete if usable key copies remain.

---

# 89. Physical Deletion

Still valuable.

---

# 90. Database Delete

Remove row/object.

---

# 91. Filesystem Delete

Best effort.

---

# 92. SSD Caveat

Flash remapping means physical erase is not guaranteed.

---

# 93. Therefore

Encryption + key destruction is stronger semantic guarantee.

---

# 94. Memory Deletion

Zeroize secrets where practical.

---

# 95. No Full RAM Guarantee

Hard truth.

---

# 96. Swap

Avoid secrets in unencrypted swap.

---

# 97. Core Dumps

Disable/restrict for secret-bearing processes.

---

# 98. Temporary Files

Use encrypted/tmpfs when possible.

---

# 99. No Plaintext Spill

Hard rule.

---

# 100. Logs

Part 51.

---

# 101. Log Retention

Short.

---

# 102. User Data In Logs

Forbidden.

---

# 103. Audit Logs

Separate retention class.

---

# 104. Audit Log Purpose

Infrastructure/governance/admin events.

---

# 105. No Message Content

Hard rule.

---

# 106. Audit Retention

Longer if justified.

---

# 107. Privacy Logs

Aggregate only.

---

# 108. Search Index

Rebuildable.

---

# 109. Deletion

Deleted content must be removed from:

```text
FTS
embeddings
cache
preview index
```

---

# 110. Search Index Deletion

```rust
pub trait SearchIndexLifecycle {
    fn remove(
        &self,
        object: OpaqueObjectRef,
    ) -> Result<(), LifecycleError>;
}
```

---

# 111. No Orphan Search Copy

Hard rule.

---

# 112. Cache

Short-lived.

---

# 113. Cache Invalidation

Immediate after deletion.

---

# 114. CDN

Generally avoid user-sensitive cleartext.

---

# 115. Encrypted Blob Cache

Still requires expiry.

---

# 116. Attachment Lifecycle

Part 40.

---

# 117. Attachment Delete Flow

```text
delete manifest reference
→ revoke/deallocate DEK
→ GC ciphertext chunks
```

---

# 118. Content-Addressed Storage

Problem:

```text
same blob may have multiple references
```

---

# 119. Reference Counting

Needed.

---

# 120. Refcount Privacy

Local/provider-internal.

---

# 121. No Cross-User Dedup In Strict Mode

Hard rule.

---

# 122. Why

Cross-user dedup leaks existence.

---

# 123. GC

Only when no live references.

---

# 124. Blob GC

```rust
pub trait BlobGarbageCollector {
    fn collect(
        &self,
        policy: GcPolicy,
    ) -> Result<GcReport, LifecycleError>;
}
```

---

# 125. GC Safety

No live-data deletion.

---

# 126. Staged GC

```text
mark
→ quarantine
→ sweep
```

---

# 127. Quarantine Window

Allows recovery from GC bugs.

---

# 128. Quarantine Data

Still encrypted.

---

# 129. No User-Facing "deleted" Until Policy Threshold Met

Hard rule depending semantics.

---

# 130. Message Deletion

Local message delete.

---

# 131. Remote Recipient

Cannot guarantee.

---

# 132. Conversation Delete

Deletes user's local copy.

---

# 133. "Delete For Everyone"

Only if protocol supports cooperative deletion.

---

# 134. Honest UI

Must explain limitations.

---

# 135. Group Deletion

Each member device may retain copy.

---

# 136. Provider Ciphertext

Can expire/delete separately.

---

# 137. Mailbox Retention

Part 35.

---

# 138. Mailbox Object TTL

Short.

---

# 139. After Fetch/Persist

Provider may remove according policy.

---

# 140. Store-And-Forward

Should not become indefinite archive.

---

# 141. Replay Cache

Part 60.

---

# 142. Aggressive expiry.

---

# 143. Presence Data

Never durable.

---

# 144. Typing Data

Memory-only.

---

# 145. Call Metadata

Minimized.

---

# 146. Call Logs

Local optional.

---

# 147. Relay Logs

No participant identity/content.

---

# 148. Federation Data

Part 58.

---

# 149. Peering Queues

Short retention.

---

# 150. No Global Cross-Domain Delivery History

Hard rule.

---

# 151. Naming Data

Part 59.

---

# 152. Old Handle History

Local only/minimized.

---

# 153. Resolver Query Logs

Forbidden.

---

# 154. Payment Data

Part 47.

---

# 155. Settlement Records

Aggregate.

---

# 156. User Spend Mapping

Forbidden.

---

# 157. Legal Hold

Part 55.

---

# 158. Principle

Legal hold preserves existing specifically scoped records.

---

# 159. It Must Not

```text
start new surveillance
expand collection
disable encryption
```

---

# 160. Legal Hold Scope

```rust
pub struct LegalHoldScope {
    pub data_types: BTreeSet<DataTypeId>,
    pub object_scope: HoldObjectScope,
    pub expires_at: Option<Timestamp>,
}
```

---

# 161. Hold State

```rust
pub enum HoldState {
    Active,
    Released,
    Expired,
}
```

---

# 162. Held Record

Deletion transitions to:

```text
BlockedByHold
```

---

# 163. Hold Release

Deletion resumes.

---

# 164. No Indefinite Silent Hold

Hard rule.

---

# 165. Hold Review

Periodic.

---

# 166. Hold Audit

Restricted.

---

# 167. No User Content In General Audit Log

Hard rule.

---

# 168. Data Rights

Part 56.

---

# 169. Deletion Request

Coordinates:

```text
active local
remote account data
backup state
integrations
```

---

# 170. Data Rights Result

```rust
pub enum DataDeletionResult {
    Deleted,
    Scheduled,
    BackupPendingExpiry,
    RetainedByLegalPolicy,
    NotHeld,
    PeerControlled,
    ProviderUnavailable,
}
```

---

# 171. No "Complete" If Peer-Controlled Copies Exist

Hard rule.

---

# 172. Account Deletion

Broader workflow.

---

# 173. Account Deletion State

```rust
pub enum AccountDeletionState {
    Requested,
    Authenticated,
    RevokingAccess,
    DeletingServiceData,
    ProcessingBackups,
    WaitingRetention,
    Completed,
    CompletedWithRetention,
}
```

---

# 174. Revoke First

Stop new activity.

---

# 175. Then Delete

Service data.

---

# 176. Remove Push Tokens

Yes.

---

# 177. Remove Provider Credentials

Yes.

---

# 178. Revoke Mailbox Capability

Yes.

---

# 179. Social Graph

Local/remote appropriate.

---

# 180. Public Handle

Unpublish.

---

# 181. Discovery Index

Remove.

---

# 182. No Handle Graveyard Exposing History

Hard rule.

---

# 183. Backup Expiry

Maybe later than account deletion.

---

# 184. Explain.

---

# 185. Deletion Verification

Need evidence.

---

# 186. Deletion Verification Level

```rust
pub enum DeletionVerification {
    Logical,
    ReplicaConfirmed,
    CryptoErased,
    PhysicalBestEffort,
}
```

---

# 187. Logical

No longer addressable.

---

# 188. ReplicaConfirmed

All required replicas acknowledge.

---

# 189. CryptoErased

Key material gone.

---

# 190. PhysicalBestEffort

Storage deleted/GC'd.

---

# 191. No Claim Stronger Than Evidence

Hard rule.

---

# 192. Lifecycle Metadata

Store minimal.

---

# 193. Lifecycle Record

```rust
pub struct LifecycleRecord {
    pub object: OpaqueObjectRef,
    pub policy_id: LifecyclePolicyId,
    pub state: LifecycleState,
    pub expiry: Option<LifecycleExpiry>,
}
```

---

# 194. Lifecycle State

```rust
pub enum LifecycleState {
    Active,
    Expired,
    PendingDeletion,
    Held,
    Deleted,
    Archived,
}
```

---

# 195. Opaque Object Ref

Not global user ID.

---

# 196. Retention Engine

```rust
pub trait RetentionEngine {
    fn evaluate(
        &self,
        object: &LifecycleRecord,
    ) -> Result<RetentionDecision, LifecycleError>;
}
```

---

# 197. Retention Decision

```rust
pub enum RetentionDecision {
    Keep,
    Delete,
    Archive,
    Hold,
}
```

---

# 198. Deletion Coordinator

```rust
pub trait DeletionCoordinator {
    fn delete(
        &self,
        request: DeletionRequest,
    ) -> Result<DeletionReport, LifecycleError>;
}
```

---

# 199. Crypto Erasure Service

```rust
pub trait CryptoErasureService {
    fn erase(
        &self,
        key_ref: DataKeyRef,
    ) -> Result<CryptoErasureReceipt, LifecycleError>;
}
```

---

# 200. Archive Manager

```rust
pub trait ArchiveManager {
    fn archive(
        &self,
        object: OpaqueObjectRef,
        policy: &ArchivePolicy,
    ) -> Result<ArchiveReceipt, LifecycleError>;
}
```

---

# 201. Lifecycle Policy Registry

Machine-readable.

---

# 202. Policy Registry

```rust
pub struct LifecyclePolicyRegistry {
    pub version: LifecyclePolicyVersion,
    pub policies: BTreeMap<DataTypeId, DataLifecyclePolicy>,
}
```

---

# 203. Policy Version

Monotonic.

---

# 204. Hard Rule

No data type without lifecycle policy.

---

# 205. CI Gate

New persistent data type requires policy entry.

---

# 206. Schema Integration

DB migration must declare lifecycle impact.

---

# 207. Field-Level Retention

Possible.

---

# 208. Avoid If Overcomplicated

Prefer record-class boundaries.

---

# 209. Storage Engine Adapters

Need lifecycle operations.

---

# 210. SQL Adapter

Supports:

```text
delete
tombstone
archive
retention query
```

---

# 211. KV Adapter

Supports:

```text
TTL
namespace delete
```

---

# 212. Object Store Adapter

Supports:

```text
lifecycle rule
object delete
version cleanup
```

---

# 213. Backend Capability

```rust
pub struct StorageLifecycleCapabilities {
    pub ttl: bool,
    pub tombstone: bool,
    pub atomic_delete: bool,
    pub versioned_delete: bool,
    pub key_destroy: bool,
}
```

---

# 214. Lifecycle Semantics

Must account for backend behavior.

---

# 215. Object Store Versioning

Delete marker may not delete old versions.

---

# 216. Hard Rule

Versioned object stores require explicit old-version lifecycle.

---

# 217. Database MVCC

Old row versions may survive until vacuum.

---

# 218. Logical Delete vs Physical Page

Distinguish.

---

# 219. Cryptographic Erasure Helps.

---

# 220. SQLite

Deleted pages may remain until vacuum/secure-delete settings.

---

# 221. Client Device

Encrypted DB recommended.

---

# 222. File System Snapshots

May retain blocks.

---

# 223. Backup/Volume Snapshots

Lifecycle-aware.

---

# 224. Snapshot Policy

```rust
pub struct SnapshotRetentionPolicy {
    pub max_age: Duration,
    pub max_count: u32,
}
```

---

# 225. No Infinite Snapshots

Hard rule.

---

# 226. Snapshot Encryption

Separate keys where feasible.

---

# 227. Snapshot Restore

Deletion ledger/filter applied.

---

# 228. Archive Encryption

Long-term profile.

---

# 229. PQ Migration

Part 66.

---

# 230. Archive Re-encryption

Scheduled.

---

# 231. Cold Storage

Can be offline.

---

# 232. Access Rare.

---

# 233. Archive Index

Minimal.

---

# 234. No User Search Analytics

Hard rule.

---

# 235. Data Lifecycle and Telemetry

Metrics safe:

```text
expired records count
deletion backlog
archive bytes
hold count
```

---

# 236. Forbidden Metrics

No:

```text
user deletion history
message retention by identity
contact-specific lifecycle
```

---

# 237. Cohort Aggregation

Minimum thresholds.

---

# 238. Lifecycle SLOs

Examples:

```text
expired ephemeral data removed within target
deletion backlog below threshold
crypto erase completion
backup expiry compliance
```

---

# 239. Deletion SLO

Per data class.

---

# 240. No Universal Deletion Deadline

Hard rule.

---

# 241. Retention Incident

Examples:

```text
expired data not deleted
backup retained too long
search index stale copy
```

---

# 242. Privacy Incident

Treat seriously.

---

# 243. Incident Playbook

Part 65.

---

# 244. Lifecycle Drift

Config says 30 days, store keeps 90.

---

# 245. Detect.

---

# 246. Retention Audit

Periodic.

---

# 247. Audit Sample

Synthetic/metadata-level.

---

# 248. No Need To Inspect User Content

Hard rule.

---

# 249. Lifecycle Reconciliation

```rust
pub trait LifecycleReconciler {
    fn reconcile(
        &self,
        scope: StorageScope,
    ) -> Result<LifecycleReconciliationReport, LifecycleError>;
}
```

---

# 250. Reconciliation Detects

```text
expired-but-live
deleted-but-indexed
backup-over-retention
orphan blob
```

---

# 251. Reconciliation Report

Aggregate.

---

# 252. No User Identifiers In General Report

Hard rule.

---

# 253. Deletion Queue

Durable.

---

# 254. Why

Deletion must survive crash.

---

# 255. Queue Item

```rust
pub struct DeletionQueueItem {
    pub object: OpaqueObjectRef,
    pub required_actions: Vec<DeletionAction>,
}
```

---

# 256. Deletion Action

```rust
pub enum DeletionAction {
    RemovePrimary,
    RemoveReplica,
    RemoveIndex,
    RevokeKey,
    ExpireBackup,
    RemoveCache,
}
```

---

# 257. Idempotent

Mandatory.

---

# 258. Crash-Safe

Mandatory.

---

# 259. Retry

Bounded/jittered.

---

# 260. Permanent Failure

Escalate.

---

# 261. No Silent Deletion Drop

Hard rule.

---

# 262. Data Archive vs Backup

Different.

---

# 263. Backup

Recovery-oriented.

---

# 264. Archive

Long-term preservation.

---

# 265. Archive Can Outlive Active Account

Only with explicit policy/consent/governance basis.

---

# 266. No Default User Archive

Hard rule.

---

# 267. Export

Part 33/56.

---

# 268. Export Copy

Once delivered to user:

```text
outside service deletion control
```

---

# 269. Explain.

---

# 270. Plugins/Integrations

Part 24/56.

---

# 271. Third-Party Data

Deletion cannot always propagate.

---

# 272. Integration Revocation

Stop future access.

---

# 273. Deletion Request

Send if supported.

---

# 274. Track Outcome

Honest.

---

# 275. No Claim "deleted everywhere"

Hard rule.

---

# 276. AI Processing

External AI data lifecycle explicit.

---

# 277. Default

No persistent external copy unless provider contract/policy says.

---

# 278. Local AI

Preferred for sensitive content.

---

# 279. Data Minimization By API Design

Do not pass full objects where subset suffices.

---

# 280. DTO Principle

```text
minimum fields
minimum lifetime
minimum scope
```

---

# 281. Typed Redaction

```rust
pub struct MinimalView<TPurpose> {
    // purpose-specific projection
}
```

---

# 282. Full Record Access

Restricted.

---

# 283. No Generic `to_json()` Everywhere

Hard rule.

---

# 284. Storage Encryption

Always for sensitive durable data.

---

# 285. Database At Rest

Encrypted.

---

# 286. Object Storage

Encrypted before provider where possible.

---

# 287. Key Separation

Part 66.

---

# 288. Data Lifecycle Privacy Budget

Useful concept.

---

# 289. Storage Footprint Budget

```rust
pub struct StoragePrivacyBudget {
    pub max_retention: Duration,
    pub max_replication: u8,
    pub allowed_scopes: BTreeSet<StorageScope>,
}
```

---

# 290. Enforced By Policy.

---

# 291. Data Residency

Part 55.

---

# 292. Retention + Residency

Both must pass.

---

# 293. Cross-Region Replication

If residency forbids:

```text
do not replicate
```

---

# 294. No Availability Override

Hard rule.

---

# 295. Deletion Under Partition

Queue locally.

---

# 296. Replica unreachable

State:

```text
pending
```

---

# 297. Do Not Mark Complete

Hard rule.

---

# 298. Delete Race

Read concurrently with deletion.

---

# 299. Need semantics.

---

# 300. Recommended

Once deletion accepted:

```text
deny new reads
```

then cleanup.

---

# 301. Deletion Barrier

```rust
pub struct DeletionBarrier {
    pub object: OpaqueObjectRef,
    pub deletion_epoch: u64,
}
```

---

# 302. Readers

Check barrier.

---

# 303. Restore

Check barrier.

---

# 304. Sync

Propagate barrier.

---

# 305. No Zombie Read

Hard rule.

---

# 306. Data Resurrection

Serious bug.

---

# 307. Anti-Resurrection Invariant

Deleted epoch dominates older writes.

---

# 308. CRDT/Sync Integration

Deletion tombstone must win according policy.

---

# 309. Aequora-Like Sync

If used, deletion must be first-class op.

---

# 310. Delete Operation

```rust
pub struct DeleteOperation {
    pub object: OpaqueObjectRef,
    pub deletion_epoch: u64,
    pub authority: DeletionAuthority,
}
```

---

# 311. Authority

```rust
pub enum DeletionAuthority {
    User,
    RetentionPolicy,
    SecurityPolicy,
    LegalPolicy,
}
```

---

# 312. Conflict Resolution

Older update cannot resurrect.

---

# 313. Hard Rule.

---

# 314. Device Reinstall

Old local backup can contain deleted object.

---

# 315. Restore Filter

Must prevent resurrection.

---

# 316. Multi-Device

Deletion sync E2EE.

---

# 317. Offline Device

On reconnect, applies deletion before old updates.

---

# 318. Hard Rule.

---

# 319. Deletion Receipt

Local/user-facing.

---

# 320. Receipt

```rust
pub struct DeletionReceipt {
    pub scope: DeletionScope,
    pub outcome: DataDeletionResult,
    pub completed_at: Option<Timestamp>,
}
```

---

# 321. No Global Tracking Receipt ID

Hard rule.

---

# 322. Lifecycle Testing

Need dedicated testkit.

---

# 323. Test Scenarios

```text
retention expiry
delete under partition
backup restore
legal hold
replica outage
search index lag
blob orphan
```

---

# 324. Retention Test

Expired object queued.

---

# 325. Delete Test

Active store denies new read immediately after barrier.

---

# 326. Replica Test

Completion waits for required replicas.

---

# 327. Backup Test

Deleted item not restored.

---

# 328. Tombstone GC Test

No premature removal.

---

# 329. Legal Hold Test

Scoped record retained, unrelated data deleted.

---

# 330. Hold Release Test

Deletion resumes.

---

# 331. Index Test

Deleted item disappears from FTS/embedding/cache.

---

# 332. Blob GC Test

Live reference prevents deletion.

---

# 333. Cross-User Dedup Test

Strict mode does not dedup across users.

---

# 334. Crypto Erase Test

Data unreadable after key destruction.

---

# 335. Key Copy Test

All required key replicas gone.

---

# 336. Account Delete Test

Push/mailbox/discovery/provider credentials revoked.

---

# 337. Offline Device Test

Old local update cannot resurrect.

---

# 338. Clock Test

Retention handles skew safely.

---

# 339. Storage Backend Test

Object versions/vacuum/snapshot semantics covered.

---

# 340. Fuzzing

Fuzz:

```text
lifecycle policy
deletion queue item
tombstone
restore filter
archive manifest
```

---

# 341. Property Tests

Properties:

```text
deleted epoch dominates older writes
expired ephemeral data cannot become active again
cryptographically erased object cannot decrypt without recreated forbidden key
undeclared data type cannot persist
```

---

# 342. Formal Verification Targets

Strong candidates:

```text
deletion state machine
tombstone convergence
legal hold vs expiry
restore anti-resurrection
```

---

# 343. TLA+ Candidate

Multi-replica delete/partition/reconnect.

---

# 344. Kani Candidate

retention/deletion policy merge.

---

# 345. Loom Candidate

concurrent read/delete/index update.

---

# 346. Performance Tests

Measure:

```text
deletion backlog
GC throughput
archive migration
crypto erase
index cleanup
```

---

# 347. Do Not Optimize By Skipping Verification

Hard rule.

---

# 348. Batch Delete

Allowed.

---

# 349. But

Each batch still respects policy/hold boundaries.

---

# 350. Background GC

Low priority.

---

# 351. Security Delete

High priority.

---

# 352. Compromise Erasure

Urgent.

---

# 353. Crate Layout

Recommended:

```text
crates/
├── siar-lifecycle-core/
├── siar-data-classification/
├── siar-retention/
├── siar-deletion/
├── siar-tombstone/
├── siar-crypto-erasure/
├── siar-archive/
├── siar-lifecycle-reconciliation/
├── siar-lifecycle-observability/
└── siar-lifecycle-testkit/
```

---

# 354. `siar-lifecycle-core`

Owns:

```text
lifecycle classes
states
errors
```

---

# 355. `siar-data-classification`

Persistence declarations/purpose/sensitivity.

---

# 356. `siar-retention`

Retention evaluation.

---

# 357. `siar-deletion`

Deletion state/queue/coordinator.

---

# 358. `siar-tombstone`

Replication/sync deletion markers.

---

# 359. `siar-crypto-erasure`

DEK/KEK destruction receipts.

---

# 360. `siar-archive`

Archive policy/storage.

---

# 361. `siar-lifecycle-reconciliation`

Drift/orphan/restore checks.

---

# 362. `siar-lifecycle-observability`

Privacy-safe lifecycle metrics.

---

# 363. `siar-lifecycle-testkit`

Partition/restore/hold/delete simulator.

---

# 364. Error Taxonomy

```rust
pub enum LifecycleError {
    PersistenceUndeclared,
    RetentionPolicyMissing,
    DeletionBlockedByHold,
    ReplicaUnavailable,
    KeyDestructionFailed,
    ArchivePolicyInvalid,
    RestoreWouldResurrect,
    BackendCapabilityMissing,
    ReconciliationFailure,
    Internal,
}
```

---

# 365. Security & Privacy Invariants

Mandatory:

```text
1. No persistent data type exists without an explicit lifecycle policy.
2. Data minimization is default; persistence requires declared purpose.
3. Retention is bounded by explicit policy unless indefinite retention is intentionally designed and justified.
4. Deletion immediately prevents new logical reads before physical cleanup completes.
5. Deleted state cannot be resurrected by stale replicas, offline devices, backups, or old sync operations.
6. Tombstones contain minimal metadata and are garbage-collected after their safety window.
7. Cryptographic erasure is not considered complete while usable key copies remain.
8. Search indexes, caches, previews, embeddings, and replicas must follow primary-object deletion.
9. Legal holds preserve only existing scoped records and never expand collection.
10. User-facing deletion results distinguish active deletion, backup retention, legal retention, peer-controlled copies, and unavailable providers.
11. Cross-user deduplication is disabled in strict privacy modes where it would reveal content equality/existence.
12. Lifecycle telemetry and audits never become a per-user retention/deletion history.
```

---

# 366. Initial Production Scope

Implement first:

```text
machine-readable lifecycle policy registry
persistence declarations
active/ephemeral/archive classes
bounded retention policies
durable deletion queue
deletion barriers
tombstones
replica deletion acknowledgements
restore anti-resurrection filter
search/cache/index cleanup
DEK/KEK crypto-erasure abstraction
backup-expiry semantics
legal-hold integration
lifecycle reconciliation
privacy-safe deletion/retention metrics
```

Then add:

```text
automated archive tiering
storage-backend lifecycle capability negotiation
formal multi-replica deletion verification
advanced key-erasure attestations
cross-provider deletion orchestration
long-term archive re-encryption automation
```

---

# 367. Definition of Done

Part 67 is complete when:

- every persistent data class has purpose, sensitivity, scope, retention, deletion, and replication policy
- ephemeral data is explicitly non-durable
- user content is not automatically archived
- retention expiry feeds durable deletion workflow
- logical deletion precedes physical cleanup
- tombstones are bounded and privacy-minimal
- replica deletion convergence is defined
- backups cannot resurrect deleted state
- cryptographic erasure and physical deletion are distinguished
- DEK/KEK key hierarchy supports scoped erasure
- legal hold is narrow and does not expand collection
- account deletion coordinates mailbox, push, discovery, provider credentials, backups, and integrations
- search/index/cache copies follow deletion
- offline devices and stale sync operations cannot resurrect data
- lifecycle reconciliation detects drift/orphans
- deletion/retention/restore/hold/fuzz/formal tests are specified

---

# 368. Final Architecture

```text
                       DATA CREATION
                            │
                            ▼
                      CLASSIFICATION
                            │
                            ▼
                    LIFECYCLE POLICY
                            │
            ┌───────────────┼───────────────┐
            │               │               │
        Ephemeral        Durable          Archive
            │               │               │
            └───────────────┼───────────────┘
                            ▼
                     RETENTION ENGINE
                            │
                            ▼
                    DELETION COORDINATOR
                            │
             ┌──────────────┼──────────────┐
             │              │              │
          Primary        Replicas      Index/Cache
             │              │              │
             └──────────────┼──────────────┘
                            ▼
                    CRYPTOGRAPHIC ERASURE
                            │
                            ▼
                      PHYSICAL CLEANUP
```

Data-lifecycle safety model:

```text
minimize collection
+
explicit retention
+
logical delete barrier
+
replica convergence
+
backup anti-resurrection
+
cryptographic erasure
+
honest user-facing semantics
```

not:

```text
delete one database row and assume the data is gone everywhere
```

---

# 369. Final Principle

Data privacy depends as much on **what the system stops retaining** as on what it encrypts.

The correct lifecycle model is:

```text
collect less
+
store narrowly
+
retain deliberately
+
delete comprehensively
+
erase keys when appropriate
+
never resurrect stale data
```

This architecture gives SIAR a storage-minimizing, retention-aware, cryptographically erasable data lifecycle across clients, providers, replicas, indexes, backups, archives, and federated infrastructure while preserving the privacy guarantees established across Parts 34–66.
