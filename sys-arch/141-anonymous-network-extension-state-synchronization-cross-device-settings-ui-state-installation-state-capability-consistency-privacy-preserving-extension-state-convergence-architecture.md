# Core System Architecture Part 141 — Anonymous Network Extension State Synchronization, Cross-Device Settings, UI State, Installation State, Capability Consistency & Privacy-Preserving Extension State Convergence Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 141  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 04, 06, 07, 30, 45, 57, 74–76, 83–85, 123–140

**Primary purpose:** define SIAR's extension state-convergence architecture for cross-device settings, installation state, UI preferences, sync eligibility, capability consistency, conflict handling, offline reconciliation, deletion propagation, version skew, state migration, privacy-aware replication, and anti-resurrection guarantees.

---

# 1. Purpose

Extensions increasingly span multiple devices.

Users may expect:

```text
the same extension installed on several devices
shared extension settings
consistent enable/disable state
cross-device UI preferences
synchronized extension-owned data
```

But blindly syncing extension state is dangerous.

It can accidentally synchronize:

```text
live capability tokens
device-sensitive permissions
secret material
temporary UI state
private usage history
stale installation state
revoked permissions
```

The governing principle is:

> **Only explicitly syncable extension state should converge across devices; device-local authority, secrets, runtime tokens, and sensitive ephemeral state must remain local and be revalidated independently on each device.**

---

# 2. Architectural Position

```text
                EXTENSION LOCAL STATE
                        │
                        ▼
                 STATE CLASSIFIER
                        │
             ┌──────────┼──────────┐
             │          │          │
          LOCAL      SYNCABLE    DERIVED
             │          │          │
             └──────────┼──────────┘
                        ▼
                 SYNC / MERGE ENGINE
                        │
                        ▼
               CROSS-DEVICE CONVERGENCE
                        │
                        ▼
              REVALIDATION / REBINDING
                        │
                        ▼
                DEVICE-LOCAL EFFECTIVE STATE
```

---

# 3. Core Separation

Keep distinct:

```text
extension package state
installation intent
installation fact
user settings
device settings
UI preferences
runtime grants
capability tokens
extension-owned data
sync metadata
```

---

# 4. Non-Goals

Part 141 does not create:

```text
universal state sync
live token replication
cross-device secret copying
behavior-history synchronization
automatic restoration of sensitive permissions
```

---

# 5. State Identity

```rust
pub struct ExtensionStateObjectId(pub [u8; 16]);
```

---

# 6. State Class

```rust
pub enum ExtensionStateClass {
    InstallationIntent,
    InstallationFact,
    UserSetting,
    DeviceSetting,
    UiPreference,
    RuntimeGrantIntent,
    RuntimeGrantToken,
    ExtensionData,
    BackgroundJobDefinition,
    NotificationPreference,
    LocalCache,
    DiagnosticState,
}
```

---

# 7. Sync Eligibility

```rust
pub enum ExtensionStateSyncEligibility {
    Never,
    DeviceLocalOnly,
    Syncable,
    SyncableWithRevalidation,
}
```

---

# 8. Hard Rule

Every state class has explicit sync eligibility.

---

# 9. Example Classification

```text
InstallationIntent → SyncableWithRevalidation
InstallationFact → DeviceLocalOnly
UserSetting → Syncable
DeviceSetting → DeviceLocalOnly
UiPreference → Usually Syncable
RuntimeGrantIntent → SyncableWithRevalidation
RuntimeGrantToken → Never
LocalCache → Never
```

---

# 10. Hard Rule

Sync eligibility cannot be inferred from field name or storage location.

---

# 11. State Descriptor

```rust
pub struct ExtensionStateDescriptor {
    pub class: ExtensionStateClass,
    pub eligibility: ExtensionStateSyncEligibility,
    pub merge_policy: ExtensionStateMergePolicy,
    pub sensitivity: DataSensitivity,
}
```

---

# 12. State Version

```rust
pub struct ExtensionStateVersion(pub u64);
```

---

# 13. Logical Clock

```rust
pub struct ExtensionStateClock {
    pub device: DeviceId,
    pub counter: u64,
}
```

---

# 14. Hard Rule

Wall clock is not the sole conflict-resolution authority.

---

# 15. Sync Record

```rust
pub struct ExtensionStateRecord {
    pub object: ExtensionStateObjectId,
    pub extension: ExtensionId,
    pub class: ExtensionStateClass,
    pub version: ExtensionStateVersion,
    pub clock: ExtensionStateClock,
    pub payload: BrokeredPayload,
}
```

---

# 16. Hard Rule

Payload schema is versioned and bounded.

---

# 17. Installation Intent

Represents user desire:

```text
install extension X
enable extension X
disable extension X
```

across eligible devices.

---

# 18. Installation Fact

Represents physical device reality:

```text
package present
package verified
runtime compatible
```

---

# 19. Hard Rule

Installation intent may sync; installation fact does not.

---

# 20. Device Reconciliation

On receiving installation intent, each device independently checks:

```text
platform compatibility
policy
package availability
signature/provenance
storage/resources
```

---

# 21. Hard Rule

One device's successful install does not prove another device can install.

---

# 22. Installation State

```rust
pub enum ExtensionInstallationState {
    Desired,
    Installing,
    Installed,
    Unsupported,
    BlockedByPolicy,
    Failed,
    Removed,
}
```

---

# 23. Hard Rule

Cross-device UI must distinguish Desired from Installed.

---

# 24. Package Version Preference

User may choose:

```text
stable
beta
pinned
```

---

# 25. Hard Rule

Actual installed package version remains device-local fact.

---

# 26. Update Preference

Can sync.

Actual update completion remains device-local.

---

# 27. Hard Rule

No fake global “updated” state.

---

# 28. User Setting

Examples:

```text
extension mode
feature toggle
preferred external service
default behavior
```

---

# 29. Device Setting

Examples:

```text
camera device
download path
local network adapter
device-specific cache size
```

---

# 30. Hard Rule

Device settings never overwrite another device's local setting.

---

# 31. Settings Schema

```rust
pub struct ExtensionSettingsSchema {
    pub version: SchemaVersion,
    pub fields: BTreeMap<ExtensionSettingKey, ExtensionSettingDescriptor>,
}
```

---

# 32. Setting Descriptor

```rust
pub struct ExtensionSettingDescriptor {
    pub scope: ExtensionSettingScope,
    pub sensitivity: DataSensitivity,
    pub merge: ExtensionStateMergePolicy,
}
```

---

# 33. Setting Scope

```rust
pub enum ExtensionSettingScope {
    AccountSync,
    TenantSync,
    DeviceLocal,
    SessionOnly,
}
```

---

# 34. Hard Rule

SessionOnly is never persisted/synced.

---

# 35. Secret Setting

API keys/tokens are not ordinary settings.

---

# 36. Hard Rule

Secrets use secret broker and are never synchronized as plain setting values.

---

# 37. Secret Reference Sync

May sync opaque intent/reference metadata only if secure key system supports re-provisioning.

---

# 38. Hard Rule

Never copy raw secret material through extension state sync.

---

# 39. UI Preference State

Examples:

```text
collapsed section
selected extension tab
preferred sort mode
```

---

# 40. Hard Rule

Only durable user preference may sync.

---

# 41. Transient UI State

Examples:

```text
scroll position
text field content
focused node
open modal
hover state
```

---

# 42. Hard Rule

Transient UI state is never cross-device synced.

---

# 43. Privacy-Sensitive UI State

Search queries/history should usually remain local or ephemeral.

---

# 44. Hard Rule

No automatic cross-device UI behavioral history.

---

# 45. Notification Preference

May sync if user expects consistency.

---

# 46. Device/OS Notification Permission

Remains device-local.

---

# 47. Hard Rule

Synced “notifications enabled” intent cannot override OS denial.

---

# 48. Background Execution Preference

May sync as intent.

Actual OS scheduling state remains device-local.

---

# 49. Hard Rule

Background permission must be revalidated per device.

---

# 50. Capability Consistency

Need to distinguish:

```text
grant intent
effective grant
runtime token
```

---

# 51. Grant Intent

Represents approved conceptual scope.

---

# 52. Effective Grant

Device-local intersection with:

```text
device policy
OS permission
privacy mode
tenant policy
current package/runtime state
```

---

# 53. Runtime Token

Never syncs.

---

# 54. Hard Rule

Cross-device grant sync transfers intent, not authority.

---

# 55. Grant Intent Record

```rust
pub struct ExtensionGrantIntent {
    pub extension: ExtensionId,
    pub permission: ExtensionPermissionClass,
    pub requested_scope: SyncableScopeIntent,
    pub lifecycle: GrantLifecycleIntent,
}
```

---

# 56. Hard Rule

Device-sensitive permissions require re-approval on each device.

---

# 57. Device-Sensitive Examples

```text
camera
microphone
clipboard
file access
local network
device management
```

---

# 58. Hard Rule

No device silently inherits these from another device.

---

# 59. Message/Conversation Scope

If portable across devices, scope can sync using stable opaque conversation refs.

---

# 60. Hard Rule

Scope sync must not expose raw contact/social graph metadata.

---

# 61. Privacy Mode

Privacy-mode preference may sync if user chooses.

---

# 62. Effective Privacy Mode

May become stricter device-locally.

---

# 63. Hard Rule

A remote lower-privacy preference cannot weaken a device's stricter local policy.

---

# 64. Merge Policy

```rust
pub enum ExtensionStateMergePolicy {
    LastWriterWins,
    AddWinsSet,
    RemoveWinsSet,
    MapPerKey,
    MaxSecurity,
    MinPrivilege,
    Manual,
}
```

---

# 65. Hard Rule

Security/permission state does not use naive last-writer-wins.

---

# 66. MinPrivilege

For permission-like state:

```text
more restrictive state wins
```

until explicit re-approval.

---

# 67. MaxSecurity

For security/privacy settings:

```text
stricter setting wins
```

under conflict.

---

# 68. Hard Rule

Conflict resolution can only preserve/increase safety automatically.

---

# 69. LastWriterWins

Allowed only for low-risk cosmetic settings.

---

# 70. Hard Rule

Use logical clock/version, not untrusted sender wall clock alone.

---

# 71. Set Merge

```rust
pub enum ExtensionStateSetMerge {
    AddWins,
    RemoveWins,
}
```

---

# 72. Hard Rule

For revocation/deletion state, RemoveWins is preferred.

---

# 73. Installation Intent Merge

If one device removes extension and another adds concurrently, policy should be explicit.

---

# 74. Recommended Baseline

RemoveWins for high-risk enterprise policy.

Manual/user-confirmed resolution for personal install intent where ambiguity matters.

---

# 75. Hard Rule

No silent reinstall after explicit removal if removal tombstone is newer.

---

# 76. Deletion Tombstone

```rust
pub struct ExtensionStateDeletionTombstone {
    pub object: ExtensionStateObjectId,
    pub epoch: ExtensionDeletionEpoch,
    pub clock: ExtensionStateClock,
}
```

---

# 77. Hard Rule

Older replicas cannot resurrect deleted state.

---

# 78. Uninstall Tombstone

Separate from extension-data tombstones if needed.

---

# 79. Hard Rule

Uninstall prevents stale device from restoring extension intent unexpectedly.

---

# 80. Reinstall

Explicit new installation intent with higher epoch.

---

# 81. Hard Rule

Reinstall does not automatically restore prior high-risk grants.

---

# 82. State Convergence

Goal:

```text
eligible durable intent converges
device facts remain local
security-sensitive effective authority is rederived
```

---

# 83. Hard Rule

Convergence does not mean byte-identical device state.

---

# 84. Convergence View

```rust
pub struct ExtensionStateConvergence {
    pub logical_state: ExtensionLogicalState,
    pub device_states: BTreeMap<DeviceId, ExtensionDeviceStateSummary>,
}
```

---

# 85. Hard Rule

Device summaries minimize sensitive activity details.

---

# 86. Sync Envelope

```rust
pub struct ExtensionStateSyncEnvelope {
    pub schema: SchemaVersion,
    pub records: Vec<ExtensionStateRecord>,
    pub tombstones: Vec<ExtensionStateDeletionTombstone>,
}
```

---

# 87. Hard Rule

Envelope is bounded.

---

# 88. Serialization

Postcard preferred internally.

RON for human-readable debug/export metadata where suitable.

---

# 89. Hard Rule

JSON only external interoperability where necessary.

---

# 90. Transport Encryption

State sync is encrypted/authenticated.

---

# 91. Hard Rule

Server cannot modify state undetectably.

---

# 92. End-to-End Encryption

Use where account/device sync model requires it.

---

# 93. Hard Rule

Sync infrastructure should not learn private extension settings unnecessarily.

---

# 94. Device Membership

Only authorized account/tenant devices can receive eligible state.

---

# 95. Hard Rule

Removed device loses future sync authority.

---

# 96. Device Revocation

Triggers:

```text
session revoke
sync key rotation if required
state rekey/re-encryption where applicable
```

---

# 97. Hard Rule

Revoked device cannot keep receiving new state.

---

# 98. State Key Scope

Prefer:

```text
account extension key
tenant extension key
device extension key
```

depending state class.

---

# 99. Hard Rule

No one universal extension-sync key across all tenants/accounts.

---

# 100. Cross-Tenant Sync

Forbidden.

---

# 101. Hard Rule

Tenant ID is explicit in sync context.

---

# 102. Federation

Extension state does not federate by default.

---

# 103. Hard Rule

Federation requires explicit interoperability contract.

---

# 104. Offline Operation

Devices may edit syncable state offline.

---

# 105. Hard Rule

Offline edits carry logical clocks/versions.

---

# 106. Reconnect

On reconnect:

```text
authenticate
fetch remote delta
apply tombstones
merge local/remote
revalidate effective state
publish new delta
```

---

# 107. Hard Rule

Revalidation happens after merge before runtime authority changes.

---

# 108. Conflict State

```rust
pub enum ExtensionStateConflict {
    ConcurrentWrite,
    SchemaMismatch,
    PolicyMismatch,
    DeletedVsModified,
    UnsupportedValue,
}
```

---

# 109. Hard Rule

Security-sensitive unresolved conflict does not default to permissive value.

---

# 110. Manual Conflict Resolution

Used where semantics cannot safely merge.

---

# 111. Hard Rule

Conflict UI shows semantic difference, not raw internal blob.

---

# 112. State Migration

Schema evolves across extension versions.

---

# 113. Hard Rule

Migration occurs before merge where needed.

---

# 114. Migration Plan

```rust
pub struct ExtensionStateMigrationPlan {
    pub extension: ExtensionId,
    pub from: SchemaVersion,
    pub to: SchemaVersion,
    pub reversible: bool,
}
```

---

# 115. Hard Rule

Unknown future schema never decoded as old schema.

---

# 116. Version Skew

Different devices may run different extension versions.

---

# 117. Hard Rule

State compatibility explicitly declared by extension version.

---

# 118. Compatibility Matrix

```rust
pub struct ExtensionStateCompatibility {
    pub writer_version: ExtensionVersion,
    pub reader_version: ExtensionVersion,
    pub relation: CompatibilityRelation,
}
```

---

# 119. Hard Rule

Older device cannot overwrite newer unknown fields if preservation strategy requires them.

---

# 120. Forward-Compatible Envelope

May preserve opaque unknown extension-owned payload sections.

---

# 121. Hard Rule

Security-sensitive fields are never blindly preserved without policy awareness.

---

# 122. Read-Only Compatibility

Old device may read state but not write if write would destroy newer semantics.

---

# 123. Hard Rule

Read-only downgrade is explicit.

---

# 124. Unsupported Device

If extension unavailable on a device:

```text
store syncable intent metadata
do not execute
do not fabricate installed state
```

---

# 125. Hard Rule

Unsupported device never receives runtime grant tokens.

---

# 126. Selective Sync

User/admin may choose which extension state classes sync.

---

# 127. Hard Rule

Disabling sync does not delete local state unless separately requested.

---

# 128. Sync Preference

```rust
pub struct ExtensionSyncPreference {
    pub extension: ExtensionId,
    pub enabled_classes: BTreeSet<ExtensionStateClass>,
}
```

---

# 129. Hard Rule

Sensitive classes may be forced Never regardless of preference.

---

# 130. Sync Pause

Device/account can pause extension state sync.

---

# 131. Hard Rule

Pause does not silently enable local-only weaker privacy.

---

# 132. State Snapshot

Periodic compact snapshot may accelerate new-device bootstrap.

---

# 133. Hard Rule

Snapshot contains only sync-eligible state.

---

# 134. Snapshot Identity

```rust
pub struct ExtensionStateSnapshot {
    pub extension: ExtensionId,
    pub snapshot_version: u64,
    pub schema: SchemaVersion,
    pub digest: Digest,
}
```

---

# 135. Hard Rule

Snapshot is integrity-protected.

---

# 136. Incremental Delta

```rust
pub struct ExtensionStateDelta {
    pub base_snapshot: Option<u64>,
    pub records: Vec<ExtensionStateRecord>,
    pub tombstones: Vec<ExtensionStateDeletionTombstone>,
}
```

---

# 137. Hard Rule

Delta application idempotent.

---

# 138. Duplicate Delivery

Safe.

---

# 139. Hard Rule

Exactly-once transport is not required.

---

# 140. At-Least-Once Sync

Use idempotent state application.

---

# 141. Hard Rule

Sync message replay cannot recreate removed authority.

---

# 142. State Hash

Optional for integrity/convergence verification.

---

# 143. Hard Rule

Do not expose hash as cross-user tracking ID.

---

# 144. Anti-Entropy

Periodic bounded reconciliation may compare version vectors/digests.

---

# 145. Hard Rule

No continuous high-frequency state chatter.

---

# 146. Battery Awareness

Mobile sync batches when possible.

---

# 147. Hard Rule

Noncritical extension preference sync can defer.

---

# 148. Privacy-Critical Revocation

Prioritized.

---

# 149. Hard Rule

Revocation/deletion state should not wait behind cosmetic settings sync.

---

# 150. Sync Priority

```rust
pub enum ExtensionStateSyncPriority {
    SecurityCritical,
    Permission,
    Installation,
    UserSetting,
    UiPreference,
    Cosmetic,
}
```

---

# 151. Hard Rule

Extension cannot self-assign SecurityCritical priority.

---

# 152. Background Sync

Uses Part 138 scheduler.

---

# 153. Hard Rule

No always-on polling.

---

# 154. Trigger

Prefer:

```text
local state change
push/wake hint
connectivity restored
periodic anti-entropy
```

---

# 155. Hard Rule

Push payload remains opaque/minimal.

---

# 156. Capability Consistency

A synced grant intent may yield different effective capability on different devices.

---

# 157. Example

```text
Device A: microphone permission allowed
Device B: microphone OS permission denied
```

Logical intent same, effective grant differs.

---

# 158. Hard Rule

UI must not imply all devices have same effective capability.

---

# 159. Device Capability Summary

```rust
pub struct ExtensionDeviceCapabilitySummary {
    pub device: DeviceId,
    pub permission: ExtensionPermissionClass,
    pub effective: bool,
    pub reason: CapabilityAvailabilityReason,
}
```

---

# 160. Privacy Minimization

Expose only to authorized device-management UI.

---

# 161. Hard Rule

No public/device-fingerprint leakage.

---

# 162. Installation Consistency

User may request:

```text
InstallOnAllEligibleDevices
InstallOnSelectedDevices
ThisDeviceOnly
```

---

# 163. Install Intent Scope

```rust
pub enum ExtensionInstallationIntentScope {
    ThisDevice,
    SelectedDevices(BTreeSet<DeviceId>),
    AllEligibleOwnedDevices,
}
```

---

# 164. Hard Rule

AllEligibleOwnedDevices still respects local platform/tenant/device policy.

---

# 165. No Forced Install Across Managed Boundary

Hard rule.

---

# 166. Tenant-Managed Install

Separate administrative intent.

---

# 167. Hard Rule

Tenant admin cannot install into personal context.

---

# 168. Extension Disable State

May be:

```text
device-local disable
account-wide disable
tenant-wide disable
```

---

# 169. Hard Rule

Scope explicit.

---

# 170. Disable Conflict

Stricter disable wins until explicit re-enable.

---

# 171. Hard Rule

Remote enable cannot override local quarantine/security block.

---

# 172. Quarantine State

Device-local security fact.

---

# 173. Hard Rule

Quarantine may sync advisory/security intent, but remote peer cannot clear local quarantine automatically.

---

# 174. Revocation State

Platform/package revocation is authoritative and restrictive.

---

# 175. Hard Rule

No extension state merge can override package revocation.

---

# 176. Marketplace Integration

Part 133 provides package/revocation/compatibility metadata.

---

# 177. Runtime Integration

Part 134 derives runtime from local effective state.

---

# 178. Permission Integration

Part 135 syncs intent, not live token.

---

# 179. Data Governance Integration

Part 136 extension-owned data follows separate sync/retention/deletion policy.

---

# 180. IPC Integration

Part 137 state changes notify runtime through bounded events.

---

# 181. Background Integration

Part 138 performs deferred sync/reconciliation jobs.

---

# 182. Notification Integration

Part 139 can notify about important conflicts/unsupported devices.

---

# 183. UI Integration

Part 140 displays logical vs device-local state clearly.

---

# 184. Hard Rule

No one layer conflates logical/account state with actual device state.

---

# 185. New Device Bootstrap

Sequence:

```text
establish fresh device identity
authorize sync membership
fetch signed/encrypted snapshot
apply tombstones
migrate schemas
restore sync-eligible intent/settings
revalidate device-local permissions/policy
materialize effective state
```

---

# 186. Hard Rule

No live runtime tokens restored.

---

# 187. Account Recovery

Same principle as Part 57.

---

# 188. Hard Rule

Recovery restores durable intent/settings, not old device authority.

---

# 189. Backup Restore

Backup may restore extension state history/snapshot.

---

# 190. Hard Rule

Newer tombstones/revocation epochs still dominate restored state.

---

# 191. Anti-Resurrection

Covers:

```text
uninstall
permission revoke
state delete
extension-data delete
device revoke
```

---

# 192. Hard Rule

Older backup/offline device cannot silently resurrect any of these.

---

# 193. State Deletion

User can delete syncable setting/history where meaningful.

---

# 194. Deletion Receipt

```rust
pub struct ExtensionStateDeletionReceipt {
    pub object: ExtensionStateObjectId,
    pub epoch: ExtensionDeletionEpoch,
    pub acknowledged_devices: BTreeSet<DeviceId>,
}
```

---

# 195. Hard Rule

Acknowledgement is not required from permanently revoked device.

---

# 196. Deletion Completion

May be:

```rust
pub enum ExtensionStateDeletionStatus {
    LocalDeleted,
    Propagating,
    ConvergedAmongActiveDevices,
}
```

---

# 197. Hard Rule

Do not falsely claim physically deleted from unreachable external systems.

---

# 198. Sync Metadata

Minimize:

```text
device ID
record class
version
clock
digest
```

---

# 199. Hard Rule

No sync metadata contains private message content.

---

# 200. Timing Metadata

Coarsen where possible.

---

# 201. Hard Rule

Do not turn sync logs into detailed device activity timelines.

---

# 202. Server Role

Sync server may store encrypted envelopes and minimal routing metadata.

---

# 203. Hard Rule

Server is not global extension-state analytics warehouse.

---

# 204. State Index

Can index:

```text
account/tenant envelope ownership
extension ID
state class
epoch/version
```

---

# 205. Hard Rule

No value-content indexing for encrypted private settings.

---

# 206. Sync Quota

Bound:

```text
record count
bytes
delta frequency
snapshot size
```

---

# 207. Hard Rule

One extension cannot dominate sync channel.

---

# 208. State Compaction

Remove superseded records after safe snapshot/tombstone retention.

---

# 209. Hard Rule

Do not compact away deletion evidence before anti-resurrection horizon.

---

# 210. Tombstone Retention

Long enough to outlive plausible offline-device/backup return window.

---

# 211. Hard Rule

Retention horizon explicit.

---

# 212. Permanently Offline Device

After policy cutoff, returning device may require full resync/re-enrollment.

---

# 213. Hard Rule

Very stale device cannot blindly merge ancient state.

---

# 214. Staleness State

```rust
pub enum ExtensionReplicaFreshness {
    Fresh,
    Stale,
    RebootstrapRequired,
}
```

---

# 215. Hard Rule

RebootstrapRequired blocks writes until snapshot/tombstone refresh.

---

# 216. Replica Identity

```rust
pub struct ExtensionStateReplicaId(pub [u8; 16]);
```

---

# 217. Hard Rule

Replica ID scoped to account/tenant extension domain, not public identity.

---

# 218. Conflict UX

User should see:

```text
setting conflict
unsupported device
permission unavailable
installation blocked
```

---

# 219. Hard Rule

Do not expose raw CRDT/vector-clock internals in normal UI.

---

# 220. Manual Resolution

For security-sensitive conflict, choices are explicit and restrictive.

---

# 221. Hard Rule

No “use latest” shortcut for all classes.

---

# 222. State Sync Service

```rust
pub trait ExtensionStateSyncService {
    async fn push(
        &self,
        delta: ExtensionStateDelta,
    ) -> Result<ExtensionSyncReceipt, ExtensionStateSyncError>;

    async fn pull(
        &self,
        cursor: ExtensionSyncCursor,
    ) -> Result<ExtensionStateDelta, ExtensionStateSyncError>;
}
```

---

# 223. Merge Service

```rust
pub trait ExtensionStateMergeService {
    fn merge(
        &self,
        local: ExtensionStateRecord,
        remote: ExtensionStateRecord,
    ) -> Result<ExtensionStateMergeResult, ExtensionStateSyncError>;
}
```

---

# 224. Revalidation Service

```rust
pub trait ExtensionStateRevalidationService {
    fn materialize_effective_state(
        &self,
        logical: &ExtensionLogicalState,
        device: DeviceId,
    ) -> Result<ExtensionDeviceEffectiveState, ExtensionStateSyncError>;
}
```

---

# 225. State Registry

```rust
pub trait ExtensionStateRegistry {
    fn descriptor(
        &self,
        class: ExtensionStateClass,
    ) -> Result<ExtensionStateDescriptor, ExtensionStateSyncError>;
}
```

---

# 226. Error Taxonomy

```rust
pub enum ExtensionStateSyncError {
    StateClassNotSyncable,
    SchemaUnsupported,
    MergeConflict,
    TombstoneDominates,
    DeviceRevoked,
    ReplicaTooStale,
    PolicyDenied,
    CrossTenantSync,
    CapabilityRevalidationFailed,
    QuotaExceeded,
    IntegrityFailure,
    Unauthorized,
    Internal,
}
```

---

# 227. Observability

Safe metrics:

```text
delta bytes
merge conflicts
rebootstrap count
tombstone applications
sync latency class
```

---

# 228. Forbidden:

```text
setting values
secret material
user behavior
detailed device usage history
```

---

# 229. Hard Rule

No per-user behavioral sync analytics.

---

# 230. Sync SLOs

Examples:

```text
security revocation propagates within target
ordinary settings converge within target when devices online
rebootstrap succeeds within target
tombstone application backlog bounded
```

---

# 231. Security SLO

```text
0 runtime capability tokens synchronized
0 revoked device receives new state
0 older state overrides newer revocation/deletion epoch
```

---

# 232. Privacy SLO

```text
0 secret values in extension settings sync
0 transient UI history synchronized by default
0 sync server indexes private setting contents
```

---

# 233. Failure Modes

```text
conflicting settings
stale device return
version skew
partial install
revocation race
```

---

# 234. Conflicting Settings

Merge by declared policy; manual if unsafe.

---

# 235. Stale Device Return

Rebootstrap required past horizon.

---

# 236. Version Skew

Read-only/transform-required/unsupported compatibility handling.

---

# 237. Partial Install

Logical Desired remains; device fact reports failure/blocked state.

---

# 238. Revocation Race

Revocation/deletion epoch dominates concurrent permissive write.

---

# 239. Testing

Need extension-state-sync testkit.

Required scenarios:

```text
offline setting edit
multi-device install intent
device-sensitive permission
extension uninstall/reinstall
stale device rejoin
```

---

# 240. Classification Test

RuntimeGrantToken classified Never sync.

---

# 241. Setting Merge Test

Low-risk cosmetic setting converges via LWW/logical clock.

---

# 242. Security Merge Test

Permission revoke beats concurrent grant extension.

---

# 243. Device Permission Test

Microphone grant intent does not become effective without local OS/user approval.

---

# 244. Install Intent Test

Unsupported device reports Desired+Unsupported, not Installed.

---

# 245. Uninstall Tombstone Test

Offline old device cannot reintroduce removed install intent.

---

# 246. Reinstall Test

New explicit intent can supersede tombstone without restoring old high-risk grants.

---

# 247. Backup Test

Restored older snapshot obeys newer tombstones.

---

# 248. Revoked Device Test

Removed device cannot receive future sync.

---

# 249. Schema Skew Test

Old extension version cannot destructively rewrite newer state.

---

# 250. Privacy Test

Transient search/scroll/focus state never appears in sync delta.

---

# 251. Fuzzing

Fuzz:

```text
state records
merge policies
logical clocks
tombstones
sync envelopes
```

---

# 252. Property Tests

Properties:

```text
Never-sync state can never enter outgoing sync envelope
runtime token can never be materialized from remote sync data
deletion/revocation epoch can never be overwritten by older permissive state
cross-tenant state can never merge
```

---

# 253. Formal Verification Targets

Strong candidates:

```text
merge convergence
tombstone dominance
permission intent→effective state
stale replica rebootstrap
```

---

# 254. Kani Candidate

class eligibility/merge/revocation invariants.

---

# 255. TLA+ Candidate

```text
offline edit → reconnect → merge → revoke/delete → stale device return
```

---

# 256. Loom Candidate

Concurrent:

```text
local setting write
remote revoke
sync apply
runtime revalidation
```

---

# 257. Performance

State sync should be lightweight.

---

# 258. Target

```text
incremental deltas
bounded snapshots
batched mobile sync
indexed tombstones
```

---

# 259. Hard Rule

No full extension-state upload on every small change.

---

# 260. Debounce

Cosmetic settings may debounce.

---

# 261. Hard Rule

Security revocation does not debounce beyond policy target.

---

# 262. Compression

Possible for snapshots.

---

# 263. Hard Rule

Bound decompressed size.

---

# 264. Storage

Separate:

```text
syncable logical state
device-local effective state
logical clocks
tombstones
replica cursors
snapshot metadata
```

---

# 265. Hard Rule

Secrets/tokens remain in dedicated secure stores.

---

# 266. Partitioning

By:

```text
account or tenant
extension
state class
replica
```

---

# 267. Hard Rule

No behavior-history partition.

---

# 268. Crate Layout

Recommended:

```text
crates/
├── siar-extension-state-core/
├── siar-extension-state-registry/
├── siar-extension-state-merge/
├── siar-extension-state-sync/
├── siar-extension-state-install/
├── siar-extension-state-permission/
├── siar-extension-state-migration/
├── siar-extension-state-revalidation/
├── siar-extension-state-observability/
└── siar-extension-state-testkit/
```

---

# 269. `siar-extension-state-core`

Owns:

```text
ExtensionStateObjectId
ExtensionStateClass
ExtensionStateSyncEligibility
ExtensionStateSyncError
```

---

# 270. `siar-extension-state-registry`

Class descriptors/sync eligibility/merge policy.

---

# 271. `siar-extension-state-merge`

Logical clocks, CRDT-like merge policies, conflict detection.

---

# 272. `siar-extension-state-sync`

Encrypted delta/snapshot transport.

---

# 273. `siar-extension-state-install`

Installation intent/fact/device reconciliation.

---

# 274. `siar-extension-state-permission`

Grant-intent synchronization and local effective revalidation.

---

# 275. `siar-extension-state-migration`

Version-skew/schema transform handling.

---

# 276. `siar-extension-state-revalidation`

Device-local materialization of effective state.

---

# 277. `siar-extension-state-observability`

Aggregate convergence health only.

---

# 278. `siar-extension-state-testkit`

merge/tombstone/version/privacy/formal tests.

---

# 279. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. Extension logical state, device-local fact, installation intent, installation fact, user setting, device setting, UI preference, permission intent, effective permission, runtime token, extension-owned data, and sync metadata are distinct state classes with explicit synchronization eligibility.
2. Runtime capability tokens, live sessions, plaintext secrets, device-sensitive permissions, transient UI state, local caches, and private behavioral history can never enter cross-device extension state sync.
3. Synced permission state represents intent only; each device independently recomputes effective authority from current package/runtime compatibility, OS permission, device policy, tenant policy, privacy mode, and local approval before creating any runtime capability.
4. Security/privacy/revocation/deletion conflicts resolve toward the more restrictive state automatically; naive last-writer-wins is reserved for low-risk cosmetic settings where semantic loss is acceptable.
5. Installation intent can converge across devices, but installation fact remains local; an unsupported, blocked, failed, quarantined, or policy-denied device can never be represented as successfully installed merely because another device is.
6. Deletion, uninstall, permission revocation, device revocation, and security tombstones use monotonic epochs/causal state that older offline replicas, restored backups, or delayed messages cannot override or resurrect.
7. Extension-state schemas are versioned and compatibility-aware; old extension versions may become read-only or require migration rather than destructively rewriting state they do not understand.
8. Cross-device state sync is authenticated, encrypted, tenant/account scoped, device-membership aware, anti-replay/anti-rollback protected, and cannot cross tenant/federation boundaries without explicit interoperability policy.
9. Recovered/new devices restore only sync-eligible durable intent/settings, then perform fresh device-local revalidation; old runtime sessions, device capability tokens, background grants, and sensitive OS permissions are never restored as active authority.
10. State snapshots, deltas, logs, indexes, metrics, and observability are minimized and cannot become a detailed device activity timeline, extension-use profile, setting-value warehouse, or private-content analytics system.
11. Security-critical state propagation—revocation, deletion, quarantine intent, restrictive policy—receives higher synchronization priority than cosmetic UI/settings state, while extensions cannot self-escalate arbitrary state to security-critical priority.
12. Extension state convergence integrates with marketplace installation/revocation, runtime sandboxing, permissions, data governance, IPC, background jobs, notifications, UI, backup/restore, account/device recovery, compatibility, and audit without creating a side channel around SIAR's security, privacy, anonymity, local-first, or tenant-isolation guarantees.
```

---

# 280. Initial Production Scope

Implement first:

```text
typed extension state-class registry
sync eligibility per class
user/device/session setting scopes
installation intent vs fact
UI preference classification
permission intent vs effective grant
Never-sync runtime tokens/secrets
logical clocks
LWW for low-risk cosmetic settings
remove-wins/min-privilege for revocation state
deletion/uninstall tombstones
encrypted Postcard sync deltas
snapshot bootstrap
offline merge/reconnect
state schema compatibility
device revalidation
stale replica rebootstrap
sync priority classes
backup/recovery anti-resurrection
privacy-safe convergence metrics
extension-state testkit
```

Then add:

```text
full CRDT library for selected state classes
cross-device installation orchestration
device-group extension profiles
tenant-managed extension-state templates
formal convergence proofs
privacy-preserving anti-entropy summaries
portable state migration bundles
```

---

# 281. Definition of Done

Part 141 is complete when:

- every extension state class has explicit sync eligibility;
- runtime tokens/secrets/transient UI state are never synced;
- installation intent and installation fact are separate;
- synced permission intent is locally revalidated;
- low-risk and security-sensitive merge policies differ;
- delete/revoke/uninstall tombstones dominate stale state;
- schema/version skew is explicit;
- stale replicas can require rebootstrap;
- backup/recovery cannot resurrect authority;
- security-critical deltas outrank cosmetic sync;
- server metadata remains minimized;
- no sync telemetry becomes device/user behavioral surveillance;
- merge/tombstone/version/privacy/fuzz/formal tests are specified.

---

# 282. Final Architecture

```text
               EXTENSION LOCAL STATE
                       │
                       ▼
                 STATE REGISTRY
                       │
            ┌──────────┼──────────┐
            │          │          │
         LOCAL      SYNCABLE    NEVER-SYNC
            │          │          │
            └──────────┼──────────┘
                       ▼
                 MERGE / TOMBSTONE
                       │
                       ▼
                ENCRYPTED STATE SYNC
                       │
                       ▼
              DEVICE-LOCAL REVALIDATION
                       │
                       ▼
                 EFFECTIVE EXTENSION
                       │
                       ▼
                 RUNTIME / UI / JOBS
```

State-convergence safety model:

```text
explicit state classes
+
sync eligibility
+
logical clocks
+
restrictive merge rules
+
deletion tombstones
+
encrypted deltas
+
schema compatibility
+
device-local revalidation
```

not:

```text
copy the whole extension database between devices, sync live permission tokens and secrets, and assume every device has identical authority
```

---

# 283. Final Principle

Cross-device extension state is trustworthy when SIAR synchronizes **durable intent**, not **ambient authority**.

The correct model is:

```text
classify every state item
+
sync only eligible durable intent
+
keep device facts local
+
merge cosmetic state safely
+
resolve security state restrictively
+
propagate tombstones
+
handle version skew explicitly
+
revalidate authority on every device
+
never synchronize live secrets or runtime tokens
```

This architecture gives SIAR a privacy-preserving extension state-convergence foundation for installation intent, settings, UI preferences, permission consistency, cross-device synchronization, offline reconciliation, tombstones, schema migration, and device-local authority while preserving the anonymity, local-first, least-authority, runtime, permission, data-governance, background, notification, UI, and anti-surveillance guarantees established across Parts 34–140.
