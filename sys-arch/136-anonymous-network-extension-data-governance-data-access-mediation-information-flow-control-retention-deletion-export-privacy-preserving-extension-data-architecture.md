# Core System Architecture Part 136 — Anonymous Network Extension Data Governance, Data Access Mediation, Information-Flow Control, Retention, Deletion, Export & Privacy-Preserving Extension Data Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 136  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 37, 42, 56, 67, 74, 81, 84–85, 94–95, 123–135

**Primary purpose:** define SIAR's extension data-governance architecture for mediated data access, information-flow labeling, source/sink control, extension-owned storage, derived data, retention, deletion, export, backup/restore, anti-resurrection, privacy-aware synchronization, uninstall/revocation cleanup, and auditability without creating a hidden data lake or surveillance path.

---

# 1. Purpose

Extension permissions are only half of the problem.

Even when an extension is allowed to read or write data, the platform must still answer:

```text
What kind of data is being accessed?
Where may it flow?
How long may it be retained?
Can it be exported?
Can it be synced?
Can derived data outlive the source?
What happens after uninstall?
What happens after user deletion?
Can backups resurrect deleted extension data?
```

The governing principle is:

> **Extension data access should be mediated by explicit data-class and information-flow policy, with retention, export, deletion, synchronization, and derived-data rules enforced independently from mere permission possession.**

---

# 2. Architectural Position

```text
                  PLATFORM / USER DATA
                         │
                         ▼
                 DATA ACCESS BROKER
                         │
              ┌──────────┼──────────┐
              │          │          │
           LABEL       POLICY      SCOPE
              │          │          │
              └──────────┼──────────┘
                         ▼
                 EXTENSION RUNTIME
                         │
              ┌──────────┼──────────┐
              │          │          │
           STORAGE     NETWORK    EXPORT
              │          │          │
              └──────────┼──────────┘
                         ▼
                 LIFECYCLE GOVERNANCE
                         │
                         ▼
             RETAIN / DELETE / MIGRATE
```

---

# 3. Core Separation

Keep distinct:

```text
permission
data class
data source
data sink
retention policy
derived data
storage ownership
export right
sync right
deletion obligation
```

---

# 4. Non-Goals

Part 136 does not create:

```text
extension-accessible raw data lake
unbounded metadata warehouse
permission = indefinite retention
derived-data exemption from deletion
shared extension storage by default
silent cloud sync of local extension data
```

---

# 5. Data Identity

```rust
pub struct ExtensionDataObjectId(pub [u8; 16]);
```

Opaque and scoped.

---

# 6. Data Class

```rust
pub enum ExtensionDataClass {
    Public,
    OperationalMetadata,
    ContactMetadata,
    ConversationMetadata,
    MessageContent,
    AttachmentMetadata,
    AttachmentContent,
    Presence,
    DeviceMetadata,
    LocationCoarse,
    Diagnostics,
    TenantAdministrative,
    SecretReference,
}
```

---

# 7. Data Sensitivity

```rust
pub enum DataSensitivity {
    Public,
    Internal,
    Private,
    Sensitive,
    Secret,
}
```

---

# 8. Hard Rule

Sensitivity and permission are distinct.

---

# 9. Data Ownership Domain

```rust
pub enum DataOwnershipDomain {
    UserOwned,
    TenantOwned,
    ExtensionOwned,
    PlatformOperational,
}
```

---

# 10. Hard Rule

Ownership affects export/deletion authority.

---

# 11. Data Source

```rust
pub enum ExtensionDataSource {
    Conversation,
    Message,
    Attachment,
    Contact,
    Presence,
    FilePicker,
    Clipboard,
    Camera,
    Microphone,
    TenantDirectory,
    ExtensionStorage,
    ExternalNetwork,
}
```

---

# 12. Data Sink

```rust
pub enum ExtensionDataSink {
    ExtensionMemory,
    ExtensionStorage,
    ExternalNetwork,
    FileExport,
    Clipboard,
    Notification,
    TenantRecord,
    UserVisibleUi,
}
```

---

# 13. Hard Rule

Source permission does not imply sink permission.

---

# 14. Information-Flow Label

```rust
pub struct ExtensionDataLabel {
    pub class: ExtensionDataClass,
    pub sensitivity: DataSensitivity,
    pub ownership: DataOwnershipDomain,
    pub retention: RetentionClass,
}
```

---

# 15. Retention Class

```rust
pub enum RetentionClass {
    Ephemeral,
    Session,
    ShortLived,
    UserControlled,
    TenantPolicy,
    Durable,
}
```

---

# 16. Hard Rule

Retention class travels with data unless explicitly transformed.

---

# 17. Taint Model

Data entering extension runtime receives a label.

---

# 18. Hard Rule

Labels are platform-owned metadata, not extension claims.

---

# 19. Information-Flow Decision

```rust
pub enum InformationFlowDecision {
    Allow,
    AllowWithTransformation,
    Deny,
    RequireConsent,
    RequireAdminApproval,
}
```

---

# 20. Information-Flow Rule

```rust
pub struct InformationFlowRule {
    pub source_class: ExtensionDataClass,
    pub sink: ExtensionDataSink,
    pub decision: InformationFlowDecision,
}
```

---

# 21. Hard Rule

Flow policy evaluated independently from capability grant.

---

# 22. Example

```text
MessageContent → ExtensionMemory = allow with grant
MessageContent → ExtensionStorage = require retention policy
MessageContent → ExternalNetwork = enhanced approval/certification
SecretReference → ExternalNetwork = deny
```

---

# 23. Data Access Broker

```rust
pub trait ExtensionDataAccessBroker {
    async fn read(
        &self,
        request: ExtensionDataReadRequest,
    ) -> Result<LabeledData, ExtensionDataGovernanceError>;

    async fn write(
        &self,
        request: ExtensionDataWriteRequest,
    ) -> Result<(), ExtensionDataGovernanceError>;
}
```

---

# 24. Hard Rule

Extensions do not directly access platform persistence.

---

# 25. Read Request

```rust
pub struct ExtensionDataReadRequest {
    pub runtime: ExtensionRuntimeId,
    pub source: ExtensionDataSource,
    pub object: DataObjectRef,
    pub purpose: DataUsePurpose,
}
```

---

# 26. Data Use Purpose

```rust
pub enum DataUsePurpose {
    Display,
    Transform,
    Send,
    Archive,
    Sync,
    Export,
    AnalyzeLocal,
}
```

---

# 27. Purpose Binding

Hard rule.

---

# 28. No "Any Purpose"

Hard rule.

---

# 29. Purpose Change

Requires policy reevaluation.

---

# 30. Labeled Data

```rust
pub struct LabeledData {
    pub label: ExtensionDataLabel,
    pub payload: BrokeredPayload,
}
```

---

# 31. Brokered Payload

Prefer handles/streams for large content.

---

# 32. Hard Rule

Inline values are bounded.

---

# 33. Data Minimization

Broker may return:

```text
subset
redacted version
coarse timestamp
pseudonymous ID
aggregated representation
```

instead of the full source object.

---

# 34. Data Projection

```rust
pub enum DataProjection {
    Full,
    MetadataOnly,
    Redacted,
    Pseudonymous,
    Aggregated,
}
```

---

# 35. Hard Rule

Minimal necessary projection is the default.

---

# 36. Contact / Presence / Device Metadata

Prefer:

```text
opaque contact refs
relationship-scoped presence
normalized device metadata
```

instead of raw source metadata.

---

# 37. Network Metadata

Raw IP/topology data is unavailable unless explicitly authorized and compatible with active privacy mode.

---

# 38. Data Sink Broker

```rust
pub trait ExtensionDataSinkBroker {
    async fn emit(
        &self,
        runtime: ExtensionRuntimeId,
        data: LabeledData,
        sink: ExtensionDataSink,
    ) -> Result<(), ExtensionDataGovernanceError>;
}
```

---

# 39. External Network Sink

Highest scrutiny for sensitive data.

---

# 40. Exfiltration Policy

```rust
pub struct ExfiltrationPolicy {
    pub source: ExtensionDataClass,
    pub destination: NetworkOrigin,
    pub allowed: bool,
    pub approval: ExfiltrationApproval,
}
```

---

# 41. Exfiltration Approval

```rust
pub enum ExfiltrationApproval {
    None,
    UserConsent,
    AdminApproval,
    CertificationRequired,
}
```

---

# 42. Hard Rule

External destinations are origin-bound where practical.

---

# 43. Secret Data

Secret-class material cannot use ordinary external sinks.

---

# 44. Secret References

Extensions should receive use-without-reveal handles/capabilities where possible.

---

# 45. Extension Storage

Storage is namespaced.

```rust
pub struct ExtensionDataNamespace {
    pub extension: ExtensionId,
    pub tenant: Option<TenantId>,
    pub device: Option<DeviceId>,
}
```

---

# 46. Hard Rules

```text
no cross-extension namespace access
no cross-tenant namespace access
```

---

# 47. Storage Class

```rust
pub enum ExtensionStorageClass {
    Ephemeral,
    Cache,
    DurableLocal,
    DurableSynced,
}
```

---

# 48. DurableSynced

Requires explicit sync eligibility.

---

# 49. Local-First Default

Extension-owned data defaults to local storage unless synchronization is explicitly part of the capability contract.

---

# 50. Stored Object

```rust
pub struct ExtensionStoredObject {
    pub id: ExtensionDataObjectId,
    pub label: ExtensionDataLabel,
    pub created_at: Timestamp,
    pub expires_at: Option<Timestamp>,
}
```

---

# 51. Retention Policy

```rust
pub struct ExtensionRetentionPolicy {
    pub class: ExtensionDataClass,
    pub max_duration: Option<Duration>,
    pub delete_on_uninstall: bool,
    pub user_configurable: bool,
}
```

---

# 52. Hard Rule

Permission does not imply retention.

---

# 53. Retention Deadline

Enforced automatically.

---

# 54. No "Forever" Default

Hard rule.

---

# 55. User / Tenant Controls

Users may shorten retention.

Tenant policy may impose stricter retention.

---

# 56. Derived Data

Derived data includes:

```text
summaries
indexes
embeddings
thumbnails
transcodes
aggregates
```

---

# 57. Derived Data Record

```rust
pub struct ExtensionDerivedData {
    pub id: ExtensionDataObjectId,
    pub sources: BTreeSet<ExtensionDataObjectId>,
    pub derivation: DerivationClass,
    pub label: ExtensionDataLabel,
}
```

---

# 58. Derivation Class

```rust
pub enum DerivationClass {
    Index,
    Summary,
    Thumbnail,
    Transform,
    Aggregate,
    Embedding,
}
```

---

# 59. Hard Rules

```text
derived sensitivity does not automatically decrease
derived retention defaults to no longer than source retention
source deletion propagates to derived data
```

---

# 60. Explicit Declassification

Only reviewed transformations may lower sensitivity.

---

# 61. Anti-Resurrection

Deleted source must not reappear from:

```text
cache
index
sync replica
backup
derived artifact
```

---

# 62. Deletion Epoch

```rust
pub struct ExtensionDeletionEpoch(pub u64);
```

Monotonic.

---

# 63. Deletion Tombstone

```rust
pub struct ExtensionDeletionTombstone {
    pub object: ExtensionDataObjectId,
    pub epoch: ExtensionDeletionEpoch,
    pub deleted_at: Timestamp,
}
```

---

# 64. Hard Rule

Older synchronized/restored state can never override a newer deletion epoch.

---

# 65. Deletion Reason

```rust
pub enum ExtensionDeletionReason {
    UserRequest,
    RetentionExpired,
    PermissionRevoked,
    ExtensionUninstalled,
    TenantPolicy,
    AccountDeletion,
}
```

---

# 66. Deletion Workflow

```text
create tombstone
stop new access
delete primary object
delete derived objects
clear caches/indexes
propagate deletion
verify
```

---

# 67. Deletion Receipt

```rust
pub struct ExtensionDeletionReceipt {
    pub object: ExtensionDataObjectId,
    pub epoch: ExtensionDeletionEpoch,
    pub verified_components: BTreeSet<DeletionComponent>,
}
```

---

# 68. Deletion Components

```rust
pub enum DeletionComponent {
    PrimaryStore,
    Cache,
    Index,
    DerivedData,
    SyncReplica,
    ExportStaging,
}
```

---

# 69. Cryptographic Erasure

Supported for encrypted extension data where appropriate.

---

# 70. Data Export

User/tenant-owned extension data should be exportable where the product contract promises portability.

---

# 71. Export Scope

```rust
pub enum ExtensionExportScope {
    AllExtensionData,
    SelectedObjects,
    TenantScoped,
    UserScoped,
}
```

---

# 72. Export Format

Versioned and documented.

Preferred internal bundle:

```text
RON manifest
+
Postcard data
```

JSON only where external interoperability requires it.

---

# 73. Export Manifest

```rust
pub struct ExtensionExportManifest {
    pub extension: ExtensionId,
    pub version: ExtensionExportVersion,
    pub objects: Vec<ExtensionExportEntry>,
    pub generated_at: Timestamp,
}
```

---

# 74. Export Entry

```rust
pub struct ExtensionExportEntry {
    pub object: ExtensionDataObjectId,
    pub class: ExtensionDataClass,
    pub digest: Digest,
}
```

---

# 75. Hard Rule

Export authority is separate from ordinary read permission.

---

# 76. Bulk Export

High sensitivity; explicit approval/audit where required.

---

# 77. Export Staging

Temporary encrypted storage with automatic expiry.

---

# 78. Import

Imports validate:

```text
schema
ownership
size bounds
integrity
compatibility
deletion epochs
```

---

# 79. Hard Rule

Import cannot resurrect an object deleted at a newer epoch.

---

# 80. Backup Eligibility

```rust
pub enum ExtensionBackupEligibility {
    Excluded,
    LocalBackupOnly,
    EncryptedBackup,
}
```

---

# 81. Hard Rules

```text
no live capability tokens in backup
no plaintext secrets in backup
restore restores data, not authority
```

---

# 82. Retired Extension Restore

Old data can become:

```text
migrated
export-only
quarantined
```

Never auto-run retired code.

---

# 83. Uninstall Data Disposition

```rust
pub enum ExtensionUninstallDataDisposition {
    DeleteImmediately,
    RetainFor(Duration),
    ExportThenDelete,
}
```

---

# 84. Uninstall Ordering

```text
revoke runtime grants
cancel background work
stop runtime
apply data disposition
verify cleanup
```

---

# 85. Revoked Extension

Security revocation may preserve data in quarantine while denying the extension runtime all access.

---

# 86. Platform-Controlled Export During Quarantine

Permitted if policy allows.

---

# 87. Extension Data Sync

Optional and explicit.

---

# 88. Sync Policy

```rust
pub struct ExtensionSyncPolicy {
    pub allowed: bool,
    pub scope: ExtensionSyncScope,
    pub encryption_required: bool,
}
```

---

# 89. Sync Scope

```rust
pub enum ExtensionSyncScope {
    DeviceSet,
    Tenant,
    UserAccount,
}
```

---

# 90. Hard Rules

```text
device-local data does not silently sync
capability tokens never sync
cross-tenant sync is forbidden
```

---

# 91. Sync Conflict Strategy

```rust
pub enum ExtensionDataConflictStrategy {
    LastWriterWins,
    MergeSet,
    MergeMap,
    ApplicationResolved,
    RejectConcurrent,
}
```

---

# 92. Hard Rule

Last-writer-wins is allowed only where semantics tolerate it.

---

# 93. Sync Encryption

Sensitive extension data uses E2EE where required by data classification.

---

# 94. Federation

Extension data does not federate by default.

---

# 95. External SaaS Boundary

An authorized third-party service is an external sink.

---

# 96. Hard Rule

SIAR cannot promise deletion from a third party after authorized disclosure; that boundary must be shown truthfully.

---

# 97. Data Processing Declaration

Each extension declares:

```text
classes read
classes stored
classes sent externally
retention
```

---

# 98. Data Declaration

```rust
pub struct ExtensionDataDeclaration {
    pub reads: BTreeSet<ExtensionDataClass>,
    pub stores: BTreeSet<ExtensionDataClass>,
    pub external_sinks: BTreeSet<ExtensionDataClass>,
    pub retention: BTreeMap<ExtensionDataClass, RetentionClass>,
}
```

---

# 99. Runtime Enforcement

Observed brokered flow cannot exceed declared flow.

---

# 100. Update Diff

A new data class, storage behavior, sink, or retention expansion requires review.

---

# 101. Marketplace / Consent Integration

Install/update UX should display:

```text
what the extension reads
what it stores
what it sends outside SIAR
how long it retains data
```

---

# 102. Data Flow Graph

Logical technical graph only.

Nodes:

```text
source
transform
store
sink
```

---

# 103. No User Social Graph

Hard rule.

---

# 104. Data Flow Edge

```rust
pub struct ExtensionDataFlowEdge {
    pub source: DataFlowNodeId,
    pub sink: DataFlowNodeId,
    pub data_class: ExtensionDataClass,
    pub policy: InformationFlowDecision,
}
```

---

# 105. Practical Information-Flow Baseline

Use platform labels on brokered values/handles plus sink validation.

Full dynamic taint tracking is not required for the first production version.

---

# 106. High-Assurance Extensions

May require stronger static/dynamic flow analysis.

---

# 107. Local AI Processing

Allowed only within declared source/sink/data-class policy.

---

# 108. External AI API

Treated as an external network sink.

---

# 109. No Silent Cloud Fallback

Hard rule.

---

# 110. Cache Governance

Caches inherit source labels.

Cache TTL ≤ source retention.

---

# 111. Cache Invalidation

Triggered by:

```text
source deletion
permission revocation
tenant policy change
privacy mode change
```

---

# 112. Index Governance

Indexes inherit source sensitivity and deletion obligations.

---

# 113. Derived Artifact Cleanup

The following must be considered during deletion:

```text
search indexes
embeddings
thumbnails
previews
transcodes
summaries
```

---

# 114. Observability Data

Extension runtime/technical metrics are separate from extension user data.

---

# 115. Metrics Cardinality

Bounded.

No user IDs by default.

---

# 116. Audit

Audit high-value events such as:

```text
bulk export
high-risk external sink approval
retention policy expansion
deletion failure
```

Do not log every normal read by default.

---

# 117. Data Governance Policy

```rust
pub struct ExtensionDataGovernancePolicy {
    pub version: ExtensionDataPolicyVersion,
    pub flow_rules: Vec<InformationFlowRule>,
    pub retention_rules: Vec<ExtensionRetentionPolicy>,
}
```

Signed and versioned.

---

# 118. Policy Epoch

```rust
pub struct ExtensionDataPolicyEpoch(pub u64);
```

Monotonic and anti-rollback.

---

# 119. Policy Tightening

A stricter policy can immediately reduce future flow and trigger reconciliation of existing stored data.

---

# 120. Reconciliation Action

```rust
pub enum ExtensionDataReconciliationAction {
    Keep,
    Relabel,
    Delete,
    Quarantine,
    RequireExport,
}
```

---

# 121. Extension Data Inventory

```rust
pub struct ExtensionDataInventory {
    pub extension: ExtensionId,
    pub classes: BTreeMap<ExtensionDataClass, ExtensionDataInventoryStats>,
}
```

---

# 122. Inventory Statistics

```rust
pub struct ExtensionDataInventoryStats {
    pub object_count: u64,
    pub bytes: u64,
    pub oldest: Option<Timestamp>,
    pub newest: Option<Timestamp>,
}
```

Aggregate only; no private-content enumeration.

---

# 123. Quota Pressure

Quota exhaustion may:

```text
reject new writes
evict caches first
prompt export/delete
```

It must not silently delete durable user-owned data.

---

# 124. Data Integrity

```rust
pub enum ExtensionDataIntegrityState {
    Verified,
    Suspect,
    Corrupt,
    Unknown,
}
```

Unknown is not Verified.

Corrupt data is quarantined rather than silently served.

---

# 125. Schema Migration

```rust
pub struct ExtensionDataMigrationPlan {
    pub extension: ExtensionId,
    pub from_schema: SchemaVersion,
    pub to_schema: SchemaVersion,
    pub reversible: bool,
}
```

---

# 126. Migration Requirements

```text
dry-run
bounded impact estimate
checkpoint
integrity verification
rollback/forward-recovery semantics
```

---

# 127. Forward-Only Migration

Requires backup/checkpoint and stronger qualification.

---

# 128. Data Rights UX

Users/admins should be able to inspect:

```text
approximate stored size
data classes
retention
external-sharing declaration
export action
delete action
```

without exposing internal schema complexity.

---

# 129. Delete-All Extension Data

Platform-controlled action.

---

# 130. Honest Delete UX

Explain:

```text
what SIAR will delete
what may remain in backups until expiry
what an external third party may still retain
```

Never claim “deleted everywhere” unless that is actually verified.

---

# 131. Account / Tenant Deletion

Extension-owned user/tenant data participates in the same deletion orchestration.

---

# 132. Relationship Revocation

May require reevaluating retained relationship-derived data.

---

# 133. Privacy-Mode Change

May trigger:

```text
access reduction
cache purge
derived-data purge
sync restriction
```

---

# 134. Maximum Anonymity

Avoid durable peer-sensitive metadata unless explicitly necessary and allowed.

Anonymous-session data may be ephemeral-only.

---

# 135. Cross-Session Correlation

Prohibited unless explicitly required and privacy-compatible.

---

# 136. Export In Anonymous Mode

Must not leak hidden route, topology, peer, or implementation metadata.

---

# 137. Governance API

```rust
pub trait ExtensionDataGovernanceService {
    fn policy_for(
        &self,
        extension: ExtensionId,
    ) -> Result<ExtensionDataGovernancePolicy, ExtensionDataGovernanceError>;

    fn inventory(
        &self,
        extension: ExtensionId,
    ) -> Result<ExtensionDataInventory, ExtensionDataGovernanceError>;
}
```

---

# 138. Deletion Service

```rust
pub trait ExtensionDeletionService {
    async fn delete(
        &self,
        extension: ExtensionId,
        scope: ExtensionDeletionScope,
        reason: ExtensionDeletionReason,
    ) -> Result<ExtensionDeletionReceipt, ExtensionDataGovernanceError>;
}
```

---

# 139. Export Service

```rust
pub trait ExtensionExportService {
    async fn export(
        &self,
        extension: ExtensionId,
        scope: ExtensionExportScope,
    ) -> Result<ExtensionExportBundle, ExtensionDataGovernanceError>;
}
```

---

# 140. Information-Flow Service

```rust
pub trait ExtensionInformationFlowService {
    fn evaluate(
        &self,
        label: &ExtensionDataLabel,
        sink: &ExtensionDataSink,
        purpose: DataUsePurpose,
    ) -> Result<InformationFlowDecision, ExtensionDataGovernanceError>;
}
```

---

# 141. Error Taxonomy

```rust
pub enum ExtensionDataGovernanceError {
    DataClassDenied,
    PurposeDenied,
    SinkDenied,
    RetentionExceeded,
    ExportNotAllowed,
    DeletionBlocked,
    TombstoneConflict,
    PolicyStale,
    CrossTenantAccess,
    IntegrityFailure,
    ScopeViolation,
    Unauthorized,
    Internal,
}
```

---

# 142. Observability

Safe metrics:

```text
bytes by extension/data class
deletion backlog
retention expirations
export count
policy denial class
```

No user identity.

---

# 143. Data Governance SLOs

Examples:

```text
retention expiry applied within target
deletion propagation within target
tombstone reconciliation within target
high-risk export receipts complete
```

---

# 144. Security SLO

```text
0 secret-class data exported through ordinary sink
0 cross-tenant extension data access
0 deleted data resurrected from sync/backup
```

---

# 145. Privacy SLO

```text
0 extension data lake outside declared namespace
0 derived private data retained past source deletion without explicit policy
0 user identity in extension technical metrics by default
```

---

# 146. Failure Modes

```text
retention drift
derived-data orphaning
backup resurrection
external exfiltration
cross-tenant mixup
```

---

# 147. Retention Drift

Handled by reconciliation job + policy epoch.

---

# 148. Derived-Data Orphaning

Handled by source lineage + delete propagation.

---

# 149. Backup Resurrection

Prevented by deletion epochs/tombstones.

---

# 150. External Exfiltration

Prevented by source/sink policy + origin binding.

---

# 151. Cross-Tenant Mixup

Prevented by explicit tenant context in every storage key/query.

---

# 152. Testing

Need `siar-extension-data-testkit`.

Required scenarios:

```text
message read without network-export permission
attachment export
retention expiry
extension uninstall
backup restore after deletion
privacy-mode escalation
```

---

# 153. Source/Sink Test

`MessageContent → ExternalNetwork` fails without enhanced authorization.

---

# 154. Retention Test

Expired object is deleted within policy target.

---

# 155. Derived Data Test

Source deletion removes linked index/embedding/thumbnail.

---

# 156. Tombstone Test

Older synchronized replica cannot resurrect deleted object.

---

# 157. Backup Test

Restore respects newer deletion epoch.

---

# 158. Cross-Tenant Test

Tenant A extension cannot read Tenant B namespace.

---

# 159. Export Test

Bulk export requires explicit export authority.

---

# 160. Quarantine Test

Revoked extension cannot access preserved quarantined data.

---

# 161. Reinstall Test

Retained data does not silently reactivate runtime access.

---

# 162. Policy Change Test

New stricter policy reconciles already-stored objects.

---

# 163. Privacy Mode Test

Maximum anonymity purges prohibited peer-sensitive cache.

---

# 164. Sync Test

Raw capability tokens never synchronize with extension data.

---

# 165. Fuzzing

Fuzz:

```text
data labels
flow rules
retention policies
export manifests
deletion tombstones
```

---

# 166. Property Tests

Properties:

```text
source permission can never imply sink permission
derived data can never outlive source retention unless explicitly allowed
deleted object can never reappear from older sync/backup state
cross-tenant namespace access can never succeed
```

---

# 167. Formal Verification Targets

Strong candidates:

```text
source→sink flow policy
retention expiry
deletion epoch anti-resurrection
cross-tenant namespace isolation
```

---

# 168. Kani Candidate

Label/sink/retention/tombstone invariants.

---

# 169. TLA+ Candidate

```text
read → store → derive → sync → delete → restore
```

---

# 170. Loom Candidate

Concurrent:

```text
read/write
deletion
policy refresh
sync reconciliation
```

---

# 171. Performance

Data-governance checks sit on broker paths.

Target:

```text
precompiled flow table
bounded labels
indexed retention metadata
batched expiry work
```

---

# 172. Hard Rule

No unbounded provenance traversal on the hot path.

---

# 173. Derived-Lineage Store

Maintained asynchronously for deletion propagation.

---

# 174. Storage

Separate:

```text
extension namespaces
data labels
retention metadata
derived lineage
deletion tombstones
export manifests
sync metadata
```

Secrets remain separate.

---

# 175. No Shared User Data Warehouse

Hard rule.

---

# 176. Partitioning

By:

```text
extension
tenant
device
data class
```

No behavioral-analytics partition.

---

# 177. Crate Layout

Recommended:

```text
crates/
├── siar-extension-data-core/
├── siar-extension-data-broker/
├── siar-extension-data-labels/
├── siar-extension-infoflow/
├── siar-extension-storage/
├── siar-extension-retention/
├── siar-extension-deletion/
├── siar-extension-export/
├── siar-extension-data-sync/
├── siar-extension-data-observability/
└── siar-extension-data-testkit/
```

---

# 178. `siar-extension-data-core`

Owns:

```text
ExtensionDataClass
DataSensitivity
DataOwnershipDomain
ExtensionDataGovernanceError
```

---

# 179. `siar-extension-data-broker`

Mediated reads/writes/projections.

---

# 180. `siar-extension-data-labels`

Labels/retention/source/sink metadata.

---

# 181. `siar-extension-infoflow`

Source→sink policy and data-flow evaluation.

---

# 182. `siar-extension-storage`

Namespaced local/synced extension storage.

---

# 183. `siar-extension-retention`

Expiry/reconciliation policy.

---

# 184. `siar-extension-deletion`

Tombstones, derived deletion, anti-resurrection.

---

# 185. `siar-extension-export`

Portable bundles and export authorization.

---

# 186. `siar-extension-data-sync`

Optional encrypted sync/conflict handling.

---

# 187. `siar-extension-data-observability`

Aggregate storage/deletion/flow health only.

---

# 188. `siar-extension-data-testkit`

Flow/retention/deletion/export/privacy tests.

---

# 189. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. Extension permission, data class, data source, data sink, purpose, ownership, retention, export, synchronization, and deletion authority are distinct concepts and can never be inferred from one another implicitly.
2. Every platform→extension read and extension→sink write is mediated by brokers that apply active authorization, data-class labels, purpose binding, scope, information-flow policy, and privacy-mode restrictions.
3. Reading sensitive data never implies permission to persist, export, sync, notify, copy to clipboard, or send it externally; source and sink authority are evaluated independently.
4. Extension storage is namespaced by extension and tenant/device context, with no cross-extension or cross-tenant access and local-first storage as the default unless sync is explicitly authorized.
5. Retention is explicit, bounded, and independent of read permission; durable copies, caches, indexes, thumbnails, embeddings, summaries, and other derived objects inherit source retention/sensitivity unless a reviewed declassification policy says otherwise.
6. Deletion creates monotonic tombstone/epoch state that propagates to primary stores, caches, derived data, sync replicas, export staging, and backup restore logic so older state cannot resurrect deleted extension data.
7. Uninstall, revocation, account deletion, tenant deletion, privacy-mode changes, and retention expiry all have explicit data-disposition behavior; revoked code cannot retain access merely because data is preserved for export or forensic purposes.
8. Exports are separate privileged operations with versioned manifests, scoped authorization, encrypted temporary staging, bounded lifetime, and truthful disclosure of any data already transferred to external third parties.
9. Backups and restores preserve extension data only according to explicit eligibility and never restore live capability tokens, plaintext secrets, retired extension execution authority, or state older than a deletion epoch.
10. External network, AI, webhook, file, clipboard, and other sinks are treated as explicit information-flow boundaries; no extension may silently move local/private data to a broader sink than the manifest, consent, policy, and certification permit.
11. Extension data inventories, metrics, audit, and observability are aggregate and technical; they cannot become a shared data lake, behavioral profile, private-content index, user surveillance system, or developer/publisher scoring mechanism.
12. Extension data governance integrates with permissions, runtime brokers, marketplace declarations, consent UX, compatibility, sync, backup/restore, deletion, export, assurance, audit/compliance, risk/PIR, tenant policy, and privacy modes without creating a side channel around SIAR's security, privacy, anonymity, or local-first guarantees.
```

---

# 190. Initial Production Scope

Implement first:

```text
typed extension data classes/sensitivity/ownership
source/sink model
brokered labeled reads/writes
data minimization projections
source→sink information-flow policy
origin-bound external network flows
namespaced local extension storage
storage quotas
retention classes/policies
derived-data lineage
deletion tombstones/epochs
cache/index/thumbnail/embedding delete propagation
uninstall/revocation data disposition
portable export bundles
encrypted export staging
backup eligibility/restore rules
optional encrypted extension-data sync
cross-tenant isolation
policy reconciliation
aggregate data inventory
privacy-safe data metrics
extension-data testkit
```

Then add:

```text
richer information-flow labels
formal declassification rules
portable cross-device extension data migration
high-assurance dynamic tainting for selected extensions
data-flow visualization
privacy-preserving retention proofs
formal anti-resurrection verification
```

---

# 191. Definition of Done

Part 136 is complete when:

- extension data classes/sensitivity/ownership are typed;
- all reads/writes go through data brokers;
- source and sink authority are separate;
- storage namespaces isolate extension/tenant contexts;
- retention is explicit and enforced;
- derived data inherits source lifecycle by default;
- deletion tombstones prevent resurrection;
- cache/index/embedding/thumbnail cleanup is covered;
- export is a separate privileged path;
- backups restore data but not stale authority;
- extension sync is explicit and encrypted where required;
- external sinks obey flow policy;
- no hidden extension data lake or behavioral profile exists;
- flow/retention/deletion/export/privacy/fuzz/formal tests are specified.

---

# 192. Final Architecture

```text
                 PLATFORM / USER DATA
                        │
                        ▼
                  DATA ACCESS BROKER
                        │
             ┌──────────┼──────────┐
             │          │          │
           LABEL      PURPOSE     SCOPE
             │          │          │
             └──────────┼──────────┘
                        ▼
                  EXTENSION RUNTIME
                        │
             ┌──────────┼──────────┐
             │          │          │
          STORAGE     NETWORK     EXPORT
             │          │          │
             └──────────┼──────────┘
                        ▼
                  DATA LIFECYCLE
                        │
                        ▼
             RETAIN / DELETE / MIGRATE
```

Extension-data safety model:

```text
mediated data access
+
explicit source/sink policy
+
data-class labels
+
namespaced storage
+
bounded retention
+
derived-data lineage
+
anti-resurrection deletion
+
scoped export/sync
```

not:

```text
let a read permission imply permanent storage and internet export, keep derived copies forever, restore deleted data from backups, and call it extension flexibility
```

---

# 193. Final Principle

Extension data governance is trustworthy when the platform can answer not only **who may read data**, but also **where that data may go, how long it may live, what may be derived from it, and how completely it can be deleted**.

The correct model is:

```text
classify the data
+
mediate every access
+
separate source from sink authority
+
minimize what is exposed
+
bound storage and retention
+
track derived data
+
propagate deletion everywhere
+
export/sync only through explicit policy
+
never let extensions accumulate a hidden data lake
```

This architecture gives SIAR a privacy-preserving extension data foundation for mediated access, information-flow control, storage, retention, deletion, export, sync, derived-data lifecycle, and anti-resurrection guarantees while preserving the anonymity, local-first, least-authority, marketplace, extension-runtime, permission, and anti-surveillance guarantees established across Parts 34–135.
