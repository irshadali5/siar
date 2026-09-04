# Part 33 — Backup, Restore, Export/Import, Archival & Long-Term Data Portability Architecture

## Reusable P2P Communication Platform

**Status:** Architecture specification  
**Part:** 33  
**Primary language:** Rust  
**Primary purpose:** define a complete, secure, versioned, local-first data backup, restore, migration, export/import, archival, retention, and long-term portability system for the communication platform  
**Primary goals:** encrypted backups, crash-safe snapshots, selective restore, device migration, long-term schema compatibility, conversation/file export, archive-node support, cryptographic erasure, corruption recovery, backup verification, offline portability, and safe restoration of security-sensitive state without ratchet/key reuse

---

# 1. Purpose

The platform now contains:

```text
messages
files
contacts
multi-device identities
E2EE keys
call history
plugins
search indexes
DTN state
settings
device trust
security audit events
```

Users eventually need to:

```text
move to a new phone
replace a laptop
recover from storage failure
restore after accidental deletion
export conversations
archive years of data
move between software versions
move between devices/platforms
```

Backup cannot be treated as:

```text
copy the database folder
```

because some state is:

```text
authoritative
derived
ephemeral
hardware-bound
security-sensitive
unsafe to restore blindly
```

The governing principle is:

> **Back up logical user data and security-safe state, not arbitrary process/runtime internals.**

---

# 2. Architectural Position

```text
Authoritative Local State
        │
        ▼
 Backup Snapshot Builder
        │
        ├── Messages
        ├── Files
        ├── Contacts
        ├── Settings
        ├── Security Metadata
        └── Plugin State
        │
        ▼
 Encryption / Integrity
        │
        ▼
 Backup Container
        │
   ┌────┼─────┐
   │    │     │
 Local  USB  Remote Storage
```

Restore:

```text
Backup Container
        ↓
Verify
        ↓
Decrypt
        ↓
Compatibility / Migration
        ↓
Selective Restore
        ↓
Fresh Security Re-establishment
        ↓
Rebuild Derived State
```

---

# 3. State Classification

Every state category must be classified.

```rust
pub enum BackupClass {
    Authoritative,
    Rebuildable,
    Ephemeral,
    HardwareBound,
    Secret,
}
```

---

# 4. Authoritative State

Examples:

```text
messages
conversation metadata
contact metadata
group metadata
user settings
call history
plugin user data
file references
```

Normally backed up.

---

# 5. Rebuildable State

Examples:

```text
search indexes
thumbnail caches
compiled WASM cache
routing cache
presence cache
diagnostic cache
```

Normally **not** backed up.

Rebuild after restore.

---

# 6. Ephemeral State

Examples:

```text
typing
presence
call media session
active Surface handles
temporary multipath plan
```

Never backed up.

---

# 7. Hardware-Bound State

Examples:

```text
non-exportable Android Keystore private key
TPM key
Secure Enclave key
```

Cannot be copied as ordinary bytes.

Restore must create a new device identity or re-bind through platform-supported secure transfer.

---

# 8. Secret State

Examples:

```text
software private keys
recovery keys
plugin secrets
file encryption keys
```

May be backed up only through explicit encrypted secret-wrapping rules.

---

# 9. Backup Types

```rust
pub enum BackupType {
    FullDevice,
    UserDataOnly,
    ConversationExport,
    Migration,
    Archive,
    OrganizationManaged,
}
```

---

# 10. Full Device Backup

Includes:

```text
messages
contacts
files
settings
plugin state
security metadata
```

but still excludes:

```text
volatile process state
derived caches
unsafe ratchet continuation state
```

---

# 11. User Data Only

Useful for:

```text
export
migration
portability
```

May omit security/runtime details.

---

# 12. Conversation Export

Export one or more conversations.

Possible formats:

```text
portable archive
Markdown/HTML view
JSON metadata
binary attachment directory
```

---

# 13. Migration Backup

Optimized for:

```text
old device → new device
```

Can include:

```text
history
settings
contacts
file data
plugin user data
```

with explicit security re-enrollment.

---

# 14. Archive Backup

Optimized for:

```text
long-term retention
read-mostly access
```

---

# 15. Organization-Managed Backup

May include:

```text
organization-owned data
audit metadata
policy configuration
```

according to managed-account security model.

---

# 16. Backup Container

Recommended logical format:

```text
backup/
├── manifest.ron
├── metadata/
├── messages/
├── contacts/
├── groups/
├── files/
├── settings/
├── plugins/
├── security/
├── audit/
└── integrity/
```

Physical container can be:

```text
single archive
directory bundle
streamed chunked archive
```

---

# 17. Manifest

```rust
pub struct BackupManifest {
    pub format_version: BackupFormatVersion,
    pub created_at: Timestamp,
    pub source_device: DeviceId,
    pub source_app_version: Version,
    pub security_mode: BackupSecurityMode,
    pub sections: Vec<BackupSectionManifest>,
}
```

---

# 18. Backup Format Version

```rust
pub struct BackupFormatVersion(pub u32);
```

This version is separate from:

```text
app version
database schema
wire protocol version
security suite
```

---

# 19. Backup Section

```rust
pub struct BackupSectionManifest {
    pub name: BackupSection,
    pub version: u32,
    pub digest: Digest,
    pub encrypted: bool,
}
```

---

# 20. Backup Sections

```rust
pub enum BackupSection {
    Messages,
    Contacts,
    Groups,
    Files,
    Settings,
    Plugins,
    SecurityMetadata,
    Audit,
}
```

---

# 21. Independent Section Versions

Each section evolves independently.

This allows:

```text
messages schema v4
contacts schema v2
plugin schema v7
```

inside one backup container.

---

# 22. Backup Encryption

Backup must be encrypted before leaving the trusted device.

Use a dedicated backup encryption key.

---

# 23. Backup Key

```rust
pub struct BackupKey(SecretBytes);
```

Never reuse:

```text
account identity key
message ratchet key
file content key
```

as backup key.

---

# 24. Backup Key Sources

Possible:

```text
random recovery key
user recovery secret + strong KDF
trusted-device transfer
organization recovery key
```

---

# 25. Password-Based Backup

If user chooses password:

```text
strong memory-hard KDF
```

must derive wrapping key.

Never use:

```text
SHA256(password)
```

directly.

---

# 26. Recovery Key

Preferred high-security option:

```text
random high-entropy secret
```

represented as:

```text
mnemonic
QR
recovery file
```

---

# 27. Multi-Recipient Backup Encryption

A backup key may be wrapped to:

```text
user recovery key
trusted device
organization key
```

without encrypting backup data multiple times.

---

# 28. Backup Envelope

```rust
pub struct BackupKeyEnvelope {
    pub recipient_kind: BackupRecipientKind,
    pub wrapped_key: Bytes,
}
```

---

# 29. Integrity

Every section/chunk must be authenticated.

Corruption must be detected before restore.

---

# 30. Merkle / Digest Tree

For large backup:

```text
per chunk digest
per section root
overall manifest root
```

allows partial verification.

---

# 31. Streaming Backup

Do not require:

```text
entire 100 GB archive in RAM
```

Use streaming.

---

# 32. Chunked Backup

```rust
pub struct BackupChunk {
    pub section: BackupSection,
    pub index: u64,
    pub ciphertext: Bytes,
    pub digest: Digest,
}
```

---

# 33. Resume

Interrupted backup upload/copy can resume at chunk boundary.

---

# 34. Incremental Backup

After initial full snapshot:

```text
only changed logical records/files
```

can be added.

---

# 35. Incremental Backup Model

Use:

```text
base snapshot
+
incremental generations
```

---

# 36. Backup Generation

```rust
pub struct BackupGeneration(pub u64);
```

---

# 37. Incremental Source

Can consume:

```text
Part 04 durable event log
```

or revision checkpoints.

---

# 38. Checkpoint

```rust
pub struct BackupCheckpoint {
    pub source_revision: u64,
}
```

---

# 39. Crash-Safe Snapshot

Backup snapshot must represent a consistent logical point.

Options:

```text
database transaction snapshot
MVCC snapshot
event revision watermark
```

---

# 40. No Half-Message Backup

Do not back up:

```text
message row
without attachment key metadata
```

if they are logically atomic.

---

# 41. Snapshot Revision

Manifest records:

```text
snapshot_revision
```

for consistency.

---

# 42. Backup While App Active

Allowed if storage backend supports consistent snapshot semantics.

Do not require app shutdown.

---

# 43. Backup During File Transfer

Back up only:

```text
completed verified file
or
explicit resumable transfer state
```

according to policy.

---

# 44. Partial Download

Normally not worth backing up.

Can restart/resume from source after restore.

---

# 45. Outbox

Pending outbound messages may be backed up carefully.

---

# 46. Outbox Restore Safety

Restoring old unsent messages can cause duplicate sends.

Each message uses stable:

```text
MessageId
```

and idempotency.

---

# 47. Pending Send Policy

On restore:

```text
show as pending
require normal dedup/send pipeline
```

---

# 48. Ratchet State Danger

Do not simply restore old message-ratchet state.

This can create:

```text
key reuse
nonce reuse
forked sessions
replay confusion
```

---

# 49. Security Restore Rule

Restore:

```text
identity metadata
trusted contacts
conversation history
```

but re-establish:

```text
live session ratchets
device incarnation
ephemeral session keys
```

---

# 50. Device Migration Security

Old trusted device can authorize new device.

Flow:

```text
New device creates fresh DeviceId
    ↓
Old device verifies
    ↓
Secure migration channel
    ↓
History/settings/files transfer
    ↓
New device authorized
    ↓
Fresh ratchets
```

---

# 51. Never Clone Device Identity Blindly

A new phone should not become an exact clone of old phone's hardware-bound DeviceId unless the security design explicitly supports it.

Preferred:

```text
new DeviceId
```

---

# 52. Old Device Retirement

After migration:

```text
keep both
or
revoke old
```

user chooses.

---

# 53. Direct Device Migration

Best UX:

```text
QR pair
local Wi-Fi/QUIC
encrypted bulk transfer
```

No cloud required.

---

# 54. Migration over USB

Optional.

Use same encrypted migration container.

---

# 55. Migration through Archive Node

Trusted personal node can provide history to new device.

---

# 56. Remote Cloud Backup

Optional storage destination.

Server sees:

```text
encrypted chunks
manifest metadata minimized
```

---

# 57. Cloud Provider Independence

Backend trait:

```rust
pub trait BackupStorage {
    async fn put_chunk(...);
    async fn get_chunk(...);
    async fn list_generations(...);
}
```

---

# 58. Storage Destinations

```rust
pub enum BackupDestination {
    LocalDisk,
    RemovableMedia,
    PeerDevice,
    PersonalNode,
    CloudObjectStore,
    OrganizationStore,
}
```

---

# 59. Local Disk

Useful desktop default.

---

# 60. Removable Media

Important for:

```text
offline
air-gapped
emergency
```

---

# 61. Personal Node

Part 20 embedded node can act as encrypted backup target.

---

# 62. Cloud Object Store

Use simple object storage semantics.

Avoid coupling backup format to provider.

---

# 63. Organization Store

Managed deployment can provide own storage.

---

# 64. Backup Scheduling

```rust
pub enum BackupSchedule {
    Manual,
    Daily,
    Weekly,
    OnCharging,
    OnTrustedNetwork,
}
```

---

# 65. Mobile Backup Policy

Heavy backup should prefer:

```text
charging
Wi-Fi/unmetered
thermal okay
```

---

# 66. Background Limits

Part 31 lifecycle scheduler handles mobile background opportunities.

---

# 67. Incremental Mobile Backup

Keep background job small.

---

# 68. Large Attachment Policy

Options:

```text
include all
include recent
metadata only
exclude
```

---

# 69. User Control

User can choose backup scope.

---

# 70. Backup Scope

```rust
pub struct BackupScope {
    pub messages: bool,
    pub files: BackupFilePolicy,
    pub contacts: bool,
    pub settings: bool,
    pub plugins: bool,
}
```

---

# 71. Plugin State Backup

Plugins declare:

```text
backupable state
non-backupable cache
secret state
```

---

# 72. Plugin Backup Manifest

Part 24 manifest can include:

```text
backup schema version
export/import hooks
```

---

# 73. Plugin Restore Failure

Must not prevent core restore.

Plugin state can remain:

```text
quarantined/unrestored
```

---

# 74. Plugin Missing

Keep plugin data namespace if policy says retain.

When compatible plugin installed later:

```text
restore/import
```

---

# 75. Search Index

Part 32 index is excluded by default.

Rebuild after restore.

---

# 76. Notification State

Part 31 notification projection mostly excluded.

Unread/read durable state remains included.

---

# 77. Presence/Typing

Never backed up.

---

# 78. Active Calls

Never backed up as resumable session.

Call history is backed up.

---

# 79. DTN Store

Two categories:

```text
own durable pending bundles
relay-only third-party bundles
```

---

# 80. Relay-Only DTN Bundles

Do not include in user backup by default.

They are transit data.

---

# 81. Own Pending Bundles

May be included if safe/idempotent.

---

# 82. Archive Format Portability

Do not serialize raw internal Rust structs directly as long-term backup format.

Use stable backup DTOs.

---

# 83. Backup DTO

```rust
pub struct BackupMessageV1 {
    pub message_id: MessageId,
    pub conversation_id: ConversationId,
    pub sender: AccountId,
    pub timestamp: Timestamp,
    pub content: BackupMessageContentV1,
}
```

---

# 84. Schema Migration

Each backup section has migrators.

```text
V1 → V2 → V3
```

---

# 85. Migration Direction

Support:

```text
old backup → current app
```

for declared support window.

---

# 86. Downgrade Restore

Current backup → old app is not generally guaranteed.

Should fail safely.

---

# 87. Long-Term Support Window

Document:

```text
minimum backup format versions supported
```

---

# 88. Archive Compatibility

Long-term archive should be more stable than internal DB schema.

---

# 89. Export vs Backup

Backup:

```text
optimized for restore
encrypted
complete machine-readable state
```

Export:

```text
optimized for user portability/readability
```

---

# 90. Conversation Export Formats

Recommended:

```text
Markdown
HTML
JSON
portable binary archive
```

---

# 91. Markdown Export

Good for:

```text
human-readable
version control
simple archival
```

---

# 92. HTML Export

Useful for rich browsing.

---

# 93. JSON Export

Useful for external tools.

---

# 94. Binary Portable Export

Preserves:

```text
attachments
metadata
rich message types
```

efficiently.

---

# 95. Export Privacy

Exported plaintext is no longer protected by E2EE at rest unless user encrypts the export.

Warn clearly.

---

# 96. Encrypted Export

Offer:

```text
encrypted portable archive
```

option.

---

# 97. Attachment Export

Use stable filenames with collision-safe mapping.

---

# 98. Path Safety

Never trust attachment filename as filesystem path.

Sanitize:

```text
../
absolute path
reserved names
```

---

# 99. Export Manifest

Map:

```text
MessageId → attachment files
```

---

# 100. Export Integrity

Include digests.

---

# 101. Selective Export

Filters:

```text
conversation
date range
sender
files only
messages only
```

---

# 102. Selective Restore

Restore user may choose:

```text
all
messages only
contacts only
settings only
specific conversations
```

---

# 103. Dependency-Aware Restore

If restoring message references file key:

```text
restore required metadata
```

or mark attachment unavailable cleanly.

---

# 104. Restore Plan

Before writing:

```rust
pub struct RestorePlan {
    pub sections: Vec<RestoreSectionPlan>,
    pub migrations: Vec<MigrationStep>,
    pub conflicts: Vec<RestoreConflict>,
}
```

---

# 105. Dry Run

```text
backup restore --dry-run
```

shows:

```text
version compatibility
space required
conflicts
missing plugins
security consequences
```

---

# 106. Restore Staging

Never write directly into live DB as data is parsed.

Use:

```text
staging
validation
transaction/import
```

---

# 107. Atomic Restore

For full restore:

```text
new database directory
```

can be built and swapped atomically where platform allows.

---

# 108. Merge Restore

Selective/import restore merges into existing account.

Harder.

Requires conflict rules.

---

# 109. Conflict Types

```rust
pub enum RestoreConflictKind {
    ExistingMessage,
    ExistingConversation,
    ContactMismatch,
    GroupMismatch,
    PluginSchemaMismatch,
}
```

---

# 110. Message Conflict

Same `MessageId`:

```text
same digest → dedup
different content → security/data-integrity conflict
```

Do not silently overwrite.

---

# 111. Contact Conflict

Merge:

```text
trust state
aliases
notes
```

according to explicit policy.

---

# 112. Group Conflict

Group cryptographic state cannot be arbitrarily merged.

Treat history vs current security membership separately.

---

# 113. Restore Security Metadata

Can restore:

```text
verified contact fingerprints
historical device records
revocations
security audit history
```

but live ratchets are re-established.

---

# 114. Device Authorization

Restored history does not automatically authorize current device.

Device enrollment is separate.

---

# 115. Recovery-Only Restore

If user lost all devices:

```text
recovery key
→ restore account material
→ create new device
→ re-establish trust
```

according to Part 28.

---

# 116. Organization Recovery

Managed account can use organization-approved recovery.

Must remain audited.

---

# 117. Backup Verification

A backup is not useful until verified.

---

# 118. Verify Command

```text
comm backup verify file.backup
```

Checks:

```text
manifest
signature/auth tag
chunk digests
section consistency
schema versions
```

---

# 119. Periodic Verification

Long-term archive should be periodically verified.

---

# 120. Bit Rot

Storage can corrupt silently.

Per-chunk digests detect.

---

# 121. Repair

If redundant backup generations exist:

```text
repair missing chunk from older generation
```

where content-addressed chunking permits.

---

# 122. Backup Deduplication

Encrypted chunks can be deduplicated locally if deterministic mapping does not leak unacceptable information.

---

# 123. Privacy Trade-Off

Global server-side dedup can leak equality.

Default:

```text
per-account backup namespace
```

---

# 124. Content-Addressed Backup Store

Within one user's encrypted backup, content addressing is useful.

---

# 125. Chunk Compression

Compress before encryption where safe.

---

# 126. Compression Bomb on Restore

Bound decompression ratio/output.

---

# 127. Secret Compression

Avoid mixed-secret/attacker-controlled compression contexts that create side channels.

Backup is local batch data, but still keep security domains separate.

---

# 128. Retention Policy

```rust
pub struct BackupRetentionPolicy {
    pub keep_daily: u32,
    pub keep_weekly: u32,
    pub keep_monthly: u32,
}
```

---

# 129. Example Retention

```text
7 daily
4 weekly
12 monthly
```

product configurable.

---

# 130. Pruning

Delete old generations only after:

```text
new generation verified
```

---

# 131. Backup Storage Quota

Bound total backup usage.

---

# 132. Low Storage

If quota reached:

```text
prune safe generations
pause
notify user
```

Never silently delete only good backup.

---

# 133. Backup Health

```rust
pub enum BackupHealth {
    Healthy,
    Stale,
    Incomplete,
    Corrupt,
    Unverified,
    Disabled,
}
```

---

# 134. Last Successful Backup

Show:

```text
date
destination
verified/unverified
```

---

# 135. Backup Diagnostics

Part 18 can expose:

```text
last backup
last verify
generation count
bytes
destination health
pending chunks
```

---

# 136. Notification Integration

Part 31 can notify:

```text
backup failed
backup storage full
restore complete
```

Only user-actionable failures.

---

# 137. Quiet Background Failure

Transient network failure:

```text
retry
```

without noisy notification.

---

# 138. Backup Failure Escalation

After repeated failure/stale age threshold:

```text
notify user
```

---

# 139. Restore Progress

UI/CLI shows:

```text
verify
decrypt
migrate
import
rebuild indexes
```

---

# 140. Dioxus Backup UI

Screens:

```text
Backup status
Create backup
Restore
Export conversations
Recovery key
Storage destinations
Retention
```

---

# 141. Headless CLI

```text
comm backup create
comm backup verify
comm backup list
comm backup restore
comm export conversation
comm import archive
```

---

# 142. Daemon API

Part 16 daemon exposes high-level commands.

---

# 143. FFI

Part 19 exposes:

```text
start_backup
start_restore
backup_progress
export_conversation
```

with operation handles.

---

# 144. Operation Handle

Long backup is asynchronous and cancellable.

---

# 145. Cancellation

Backup:

```text
safe to cancel between chunks
```

Restore:

```text
safe while staging
```

After final atomic commit begins, cancellation semantics become restricted.

---

# 146. Progress Model

```rust
pub struct BackupProgress {
    pub phase: BackupPhase,
    pub completed_bytes: u64,
    pub total_bytes: Option<u64>,
}
```

---

# 147. Backup Phase

```rust
pub enum BackupPhase {
    Snapshot,
    Serialize,
    Encrypt,
    Write,
    Verify,
    Complete,
}
```

---

# 148. Restore Phase

```rust
pub enum RestorePhase {
    Open,
    Verify,
    Decrypt,
    Migrate,
    Stage,
    Commit,
    RebuildDerived,
    Complete,
}
```

---

# 149. Resource Limits

Part 08 applies:

```text
CPU
memory
I/O
network
temporary disk
```

---

# 150. Streaming Buffers

Use bounded reusable buffers.

---

# 151. No Whole-Archive Memory Load

Hard invariant.

---

# 152. Backup Priority

Lower than:

```text
active call
message delivery
security event
```

---

# 153. Pause During Call

Heavy backup can pause during realtime call on mobile.

---

# 154. Thermal Pressure

Pause compression/encryption if necessary.

---

# 155. Battery Saver

Pause heavy background backup.

---

# 156. Charging Policy

Large archive creation can prefer charging.

---

# 157. Encryption Performance

Use streaming AEAD/chunk encryption.

---

# 158. Crypto Parallelism

Parallelize chunks carefully.

Bound memory and CPU.

---

# 159. File Reuse

If attachment chunk already in backup generation/store:

```text
reference existing encrypted backup chunk
```

when safe.

---

# 160. Backup Destination Trait

```rust
pub trait BackupStorage {
    async fn begin_generation(...);
    async fn put_chunk(...);
    async fn get_chunk(...);
    async fn commit_generation(...);
    async fn delete_generation(...);
}
```

---

# 161. Atomic Generation Commit

Destination must not expose incomplete generation as current.

---

# 162. Commit Marker

Upload/write:

```text
chunks
manifest
verification root
commit marker last
```

---

# 163. Interrupted Upload

No commit marker:

```text
generation incomplete
```

can resume or GC.

---

# 164. Remote Listing

Only committed generations shown to restore UI.

---

# 165. Backup Lock

One backup writer per profile/account at a time.

---

# 166. Restore Lock

Restore requires exclusive data mutation phase.

---

# 167. Read Access During Restore

For full restore:

```text
app may enter maintenance/read-only mode
```

---

# 168. Hot Selective Import

Small conversation import may be transactional without full maintenance mode.

---

# 169. Corrupt Local DB Recovery

If local DB corrupt but backup healthy:

```text
create fresh DB
restore
rebuild derived state
```

---

# 170. Partial Recovery

If one backup section corrupt:

```text
restore healthy sections
```

only if user chooses and dependencies allow.

---

# 171. Security Section Corrupt

Do not improvise.

Restore user history while requiring fresh device/account recovery.

---

# 172. File Section Missing

Messages can restore with:

```text
attachment unavailable
```

metadata.

---

# 173. Plugin Section Missing

Core restore still succeeds.

---

# 174. Search Rebuild

After restore:

```text
Part 32 index = empty/rebuild
```

---

# 175. Thumbnail Rebuild

Regenerate lazily.

---

# 176. Presence State

Starts fresh.

---

# 177. Notification State

Recompute from unread/message state.

---

# 178. Outbox

Pending operations require idempotency review before automatic resume.

---

# 179. Calls

Historical call records restore.

No live call state.

---

# 180. Device Trust

Restore historical trust records, but current device must be newly authorized/recovered.

---

# 181. Backup Export of Keys

Default:

```text
no raw private-key dump
```

---

# 182. Software Key Backup

If required:

```text
wrap with backup recovery key
```

and mark high sensitivity.

---

# 183. Hardware Key Backup

Usually impossible by design.

---

# 184. Recovery Key Rotation

If recovery key changes:

```text
new backups use new wrapping key
```

Old backups may remain tied to old recovery key unless rewrapped.

---

# 185. Rewrap

Can rewrap only backup master key envelopes without re-encrypting all chunks, if architecture supports.

---

# 186. Lost Recovery Key

If no trusted device/organization recovery:

```text
encrypted backup is unrecoverable
```

This is expected.

---

# 187. Recovery Key Validation

When user saves recovery key:

```text
ask to confirm
```

before claiming backup recoverable.

---

# 188. Recovery Key UX

Avoid storing screenshot automatically.

Offer:

```text
copy
print
save encrypted file
QR
```

with warnings.

---

# 189. Export Key UX

Plaintext export requires explicit confirmation.

---

# 190. Archive Reader

Long-term export should be readable by a standalone reader where possible.

---

# 191. `comm-archive-reader`

Optional separate Rust tool:

```text
open encrypted archive
verify
decrypt
browse
export
```

---

# 192. Archive Reader Independence

Useful even if full application no longer installed.

---

# 193. Stable Archive Spec

Publish format spec if long-term portability is a product goal.

---

# 194. External Interoperability

Part 23 can define:

```text
backup/export format conformance
```

for third-party readers/importers.

---

# 195. JSON/Markdown Export

External tools need no Rust dependency.

---

# 196. Native Import

Third-party importers can target public logical import API.

---

# 197. Import API

```rust
pub trait DataImporter {
    fn inspect(&self, source: &ImportSource) -> Result<ImportPlan, ImportError>;
    async fn import(&self, plan: ImportPlan) -> Result<ImportResult, ImportError>;
}
```

---

# 198. External Messenger Import

Future adapters could import:

```text
generic JSON
other messenger export
CSV contacts
```

into logical domain model.

---

# 199. Import Trust

Imported messages are not automatically cryptographically verified as native historical messages.

Mark provenance.

---

# 200. Provenance

```rust
pub enum DataProvenance {
    NativeVerified,
    NativeBackup,
    ExternalImport,
    UserCreated,
}
```

---

# 201. Imported Identity Claims

Do not allow imported file to assert:

```text
verified peer identity
```

without real cryptographic evidence.

---

# 202. Imported Security State

External import cannot create trusted device certificates or authority roles.

---

# 203. Export Provenance

Optional signed export can prove it was produced by user's device, but do not overclaim third-party authenticity.

---

# 204. Legal/Compliance Export

Managed organizations may need:

```text
data subject export
retention export
```

Use explicit managed policy.

---

# 205. Retention Policy

User/organization can define:

```text
keep forever
keep N days
keep metadata only
delete attachment after N days
```

---

# 206. Retention Is Separate from Backup

Deleting local data may or may not delete backup copy.

Policy must define both.

---

# 207. Backup Retention vs Data Retention

Examples:

```text
message deleted locally
backup older generation still contains it
```

User should understand.

---

# 208. Purge Policy

For strict deletion:

```text
delete local
delete backup references/generations
destroy relevant keys
```

---

# 209. Cryptographic Erasure

Best mechanism for encrypted backup:

```text
destroy wrapping/content key
```

where possible.

---

# 210. Immutable Backup Trade-Off

Immutable backups improve ransomware resistance but conflict with deletion requirements.

Offer policy choices.

---

# 211. Ransomware Resistance

Use:

```text
append-only generations
delayed deletion
offline copy
```

for high-value deployments.

---

# 212. Organization Backup

Can require:

```text
immutable retention
```

subject to legal policy.

---

# 213. Personal Backup

User controls.

---

# 214. Backup Verification Schedule

Example:

```text
verify newest generation after creation
periodically sample older generations
```

---

# 215. Restore Drill

For enterprise/high-value use:

```text
periodic test restore
```

is more meaningful than backup success alone.

---

# 216. Test Restore

Restore into:

```text
isolated temporary profile
```

and run integrity checks.

---

# 217. Backup Health Score

Could derive:

```text
freshness
verification
destination redundancy
recovery key confirmed
```

---

# 218. Multiple Destinations

Support:

```text
local external drive
+
personal node
+
cloud
```

for redundancy.

---

# 219. Destination Independence

One failed destination should not invalidate others.

---

# 220. Replication Policy

```rust
pub struct BackupReplicationPolicy {
    pub minimum_successful_destinations: u8,
}
```

---

# 221. Backup Quorum

For enterprise:

```text
require 2 destinations
```

before pruning old generation.

---

# 222. Personal Node Sync

Encrypted chunks can replicate opportunistically when node online.

---

# 223. DTN Backup

Not recommended for arbitrary huge backups through mesh.

Could carry:

```text
small emergency recovery metadata
```

only if explicitly designed.

---

# 224. Offline Archive

USB/external SSD remains important.

---

# 225. Archive File Naming

Use deterministic:

```text
product-account-date-generation.backup
```

without exposing sensitive contact names.

---

# 226. Manifest Metadata Privacy

Even encrypted backup filename/manifest may reveal:

```text
date
size
device
```

Encrypt manifest where practical.

---

# 227. Public Header

Minimal unencrypted header:

```text
magic
format version
KDF parameters
key envelopes
```

---

# 228. Encrypted Manifest Body

Detailed sections/metadata encrypted.

---

# 229. Magic

```text
COMMBAK
```

or project-specific marker.

---

# 230. Format Detection

Reader can reject unsupported files cleanly.

---

# 231. Restore Version Check

Before decryption-heavy work:

```text
validate basic format/header
```

---

# 232. KDF Parameter Bounds

Prevent malicious backup file from requesting absurd:

```text
memory
iterations
```

and causing DoS.

Clamp/validate.

---

# 233. Archive Bomb Protection

Bound:

```text
chunk count
declared sizes
compression ratio
nested archive depth
```

---

# 234. Import Fuzzing

Part 10 fuzz:

```text
backup header
manifest
section parser
chunk table
migration code
```

---

# 235. Restore Security

Treat backup file as untrusted input even if user supplied it.

---

# 236. Path Traversal

Export/import extractor rejects:

```text
../
absolute paths
symlink escape
```

---

# 237. Migration Fuzzing

Old-version DTO → current model.

Must not panic.

---

# 238. Property Tests

Examples:

```text
backup→restore preserves logical messages
search index excluded and rebuilds
duplicate restore is idempotent where merge mode allows
corrupt chunk detected
ratchet state never blindly resumed
```

---

# 239. Round-Trip Test

```text
source profile
→ backup
→ fresh profile
→ restore
```

Compare logical data.

---

# 240. Cross-Version Test

```text
v1 backup
→ current
```

for every supported format generation.

---

# 241. Selective Restore Test

Restore only:

```text
contacts + one conversation
```

No unrelated data appears.

---

# 242. Missing Plugin Test

Plugin state retained/quarantined.

Core succeeds.

---

# 243. Corrupt Attachment Test

Messages restore.

Attachment marked unavailable.

---

# 244. Corrupt Security Section Test

History restores only under safe recovery mode.

No silent new trust.

---

# 245. Wrong Password Test

Fail authentication cleanly.

Do not leak partial plaintext.

---

# 246. Tamper Test

Modify one encrypted chunk.

Verification fails.

---

# 247. Interrupted Backup Test

No committed generation appears.

Resume/cleanup works.

---

# 248. Interrupted Restore Test

Live profile remains unchanged until commit.

---

# 249. Low Disk Test

Preflight detects insufficient staging disk.

---

# 250. Huge Archive Test

Streaming memory stays bounded.

---

# 251. Long-Term Test Corpus

Keep archived backups from old releases in CI.

---

# 252. Conformance

Part 23 should include:

```text
backup format vectors
manifest parsing
migration fixtures
export examples
```

if public portability is desired.

---

# 253. Performance Targets

Backup should be limited primarily by:

```text
storage
encryption
compression
network
```

not excessive allocation.

---

# 254. Parallelism

Use bounded worker pools for:

```text
compression
encryption
hashing
```

---

# 255. Preserve Responsiveness

Backup runs at lower priority than interactive messaging/calls.

---

# 256. Suggested Workspace

```text
crates/
├── comm-backup-core/
├── comm-backup-format/
├── comm-backup-crypto/
├── comm-backup-snapshot/
├── comm-backup-storage/
├── comm-backup-restore/
├── comm-backup-migrate/
├── comm-export/
├── comm-import/
├── comm-archive/
├── comm-backup-diagnostics/
└── comm-backup-testkit/
```

---

# 257. `comm-backup-core`

Owns:

```text
backup types
scope
progress
retention
health
```

---

# 258. `comm-backup-format`

Owns:

```text
container
manifest
section versions
chunk layout
```

---

# 259. `comm-backup-crypto`

Owns:

```text
backup key
KDF
key envelopes
chunk AEAD
integrity tree
```

---

# 260. `comm-backup-snapshot`

Owns:

```text
consistent source revision
section extraction
incremental checkpoint
```

---

# 261. `comm-backup-storage`

Owns storage backend trait.

---

# 262. `comm-backup-restore`

Owns:

```text
inspect
verify
stage
merge/full restore
commit
```

---

# 263. `comm-backup-migrate`

Owns section schema migrations.

---

# 264. `comm-export`

Owns:

```text
Markdown
HTML
JSON
portable export
```

---

# 265. `comm-import`

Owns:

```text
native archive
external adapters
provenance
conflicts
```

---

# 266. `comm-archive`

Owns long-term reader/archive profile.

---

# 267. `comm-backup-testkit`

Provides:

```text
synthetic profiles
old backup versions
corrupt chunks
wrong password
partial storage
```

---

# 268. Public Backup API

```rust
pub trait BackupService {
    async fn create(
        &self,
        request: BackupRequest,
    ) -> Result<BackupOperationId, BackupError>;

    async fn verify(
        &self,
        source: BackupSource,
    ) -> Result<BackupVerification, BackupError>;

    async fn restore(
        &self,
        request: RestoreRequest,
    ) -> Result<RestoreOperationId, BackupError>;
}
```

---

# 269. Backup Request

```rust
pub struct BackupRequest {
    pub scope: BackupScope,
    pub destination: BackupDestination,
    pub security: BackupSecurityMode,
}
```

---

# 270. Restore Request

```rust
pub struct RestoreRequest {
    pub source: BackupSource,
    pub mode: RestoreMode,
    pub selected_sections: Vec<BackupSection>,
}
```

---

# 271. Restore Modes

```rust
pub enum RestoreMode {
    ReplaceProfile,
    Merge,
    Selective,
}
```

---

# 272. Replace Profile

Safest for full migration/recovery.

---

# 273. Merge

Useful for importing history but requires conflict logic.

---

# 274. Selective

User-controlled sections/conversations.

---

# 275. Implementation Phases

## Phase 1 — Full Local Backup

```text
messages
contacts
settings
files
manifest
encryption
verification
```

## Phase 2 — Full Restore

```text
staging
migration
atomic profile replacement
```

## Phase 3 — Device Migration

```text
QR
local P2P transfer
new DeviceId
fresh sessions
```

## Phase 4 — Incremental Backup

```text
generation
checkpoint
resume
retention
```

## Phase 5 — Export / Import

```text
Markdown
JSON
portable archive
```

## Phase 6 — Personal Node / Remote Storage

```text
storage backend
encrypted remote chunks
```

## Phase 7 — Selective Restore

```text
conversation
contacts
plugin state
```

## Phase 8 — Long-Term Archive

```text
stable reader
format spec
verification
```

## Phase 9 — Hardening

```text
corruption
old versions
low disk
tampering
fuzzing
restore drills
```

---

# 276. Initial Production Recommendation

For v1, ship:

```text
encrypted local backup
encrypted removable-media backup
full restore
new-device migration
conversation Markdown/JSON export
backup verification
rebuild search/index after restore
```

Then add:

```text
incremental generations
personal-node backup
cloud-object-store backends
selective restore
standalone archive reader
```

Do not start with a complex cloud backup service.

---

# 277. Definition of Done

Part 33 is complete when:

- authoritative, derived, ephemeral, hardware-bound, and secret state are classified
- backups contain logical user data rather than arbitrary runtime files
- backup format is versioned independently of internal DB schema
- backups are encrypted before leaving the trusted device
- backup keys are separate from account/message/file keys
- large backups stream with bounded memory
- chunk integrity/tampering is detectable
- interrupted backups are resumable or safely discarded
- committed generations are atomic
- search indexes/caches are excluded and rebuilt after restore
- active presence/calls/runtime handles are never restored
- a new device normally receives a fresh `DeviceId`
- old ratchet/session state is never blindly resumed from backup
- full restore supports staging and atomic commit
- merge/selective restore has explicit conflict rules
- messages with missing/corrupt files restore safely
- plugin restore failure cannot block core restore
- backup verification exists as a first-class operation
- retention/pruning never deletes the only verified recovery point silently
- plaintext exports warn about loss of at-rest E2EE protection
- import cannot fabricate trusted identities/security state
- old supported backup formats migrate to current format
- fuzz, tamper, wrong-key, partial, low-disk, cross-version, crash, and round-trip tests exist
- backup/recovery can work fully offline without mandatory cloud infrastructure

---

# 278. Relationship to Earlier Parts

Part 33 integrates with:

```text
02 — Multi-Device Identity
04 — Offline Event Log
05 — File / Blob Subsystem
06 — DTN
08 — Resource Limits
09 — Crash Recovery
10 — Fuzzing
13 — Battery Scheduling
16 — Daemon / Headless
18 — Diagnostics
19 — C ABI / FFI
20 — Embedded Node
21 — Protocol Extensions
22 — WASM Components
23 — Interoperability
24 — Plugin Ecosystem
28 — E2EE / Key Management
30 — Receipts / Read State
31 — Background Delivery
32 — Search / Indexing
```

---

# 279. Final Architecture

```text
                 AUTHORITATIVE LOCAL STATE
                           │
                           ▼
                    Snapshot Revision
                           │
            ┌──────────────┼──────────────┐
            │              │              │
         Messages         Files       Security Meta
            │              │              │
            └──────────────┼──────────────┘
                           │
                   Serialize / Migrate
                           │
                     Chunk / Compress
                           │
                        Encrypt
                           │
                   Integrity Manifest
                           │
              ┌────────────┼────────────┐
              │            │            │
          Local Disk      USB      Personal Node
```

Restore:

```text
Backup
  ↓
Verify
  ↓
Decrypt
  ↓
Version Migrate
  ↓
Stage
  ↓
Security-Safe Re-enrollment
  ↓
Atomic Commit
  ↓
Rebuild Search / Cache / Presence
```

---

# 280. Final Principle

A reliable backup system does not preserve every byte of the old runtime.

It preserves:

```text
the user's durable truth
+
the minimum security-safe recovery state
```

and regenerates everything else.

The correct model is:

```text
logical data backup
+
strong encryption
+
versioned format
+
integrity verification
+
safe device re-enrollment
+
rebuildable derived state
```

not:

```text
copy the application directory and hope it starts
```

Part 33 therefore gives the communication platform a durable long-term ownership story: users can recover, migrate, archive, export, and preserve their data without weakening the security and local-first guarantees of the system.
