# Core System Architecture Part 57 — Anonymous Network Identity Recovery, Account Portability, Device Re-Provisioning & Privacy-Preserving Continuity Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 57  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 02, 33, 35, 43–56  

**Primary purpose:** define privacy-preserving identity recovery, account portability, device re-provisioning, lost-device revocation, social-graph continuity, mailbox re-establishment, provider migration, and secure recovery semantics without restoring stale session secrets or creating a universal recovery identifier.

---

# 1. Purpose

Identity recovery is one of the highest-risk areas in a privacy system.

A poorly designed recovery flow can destroy years of privacy engineering by introducing:

```text
permanent recovery identifiers
centralized recovery accounts
reused device secrets
stale ratchet/session restoration
provider-visible account linkage
social graph reconstruction
```

The governing principle is:

> **Recovery should restore authority and durable user state, not resurrect old transport/session secrets or weaken identity separation.**

---

# 2. Architectural Position

```text
Durable Account Authority
          │
          ▼
Recovery Authorization
          │
          ▼
Fresh Device Enrollment
          │
          ▼
Re-Provisioned Identity State
          │
          ├── social graph continuity
          ├── mailbox replacement
          ├── capability rotation
          ├── provider re-binding
          └── local data restore
```

---

# 3. Core Separation

Keep distinct:

```text
account authority
device identity
transport identity
mailbox identity
relationship identity
recovery identity
backup encryption key
session/ratchet state
```

---

# 4. Non-Goals

Recovery must not require:

```text
restoring old session keys
restoring one-time reply capabilities
restoring transport tickets
restoring stale mailbox secrets
restoring old device identity verbatim
```

---

# 5. Account Authority

Long-lived logical authority.

```rust
pub struct AccountAuthorityId(pub [u8; 32]);
```

This may identify an account cryptographically to its trusted devices.

---

# 6. Device Identity

Per-device.

```rust
pub struct DeviceIdentityId(pub [u8; 32]);
```

---

# 7. Fresh Device Rule

A recovered/re-provisioned device receives:

```text
new DeviceIdentityId
new transport identity
new mailbox credentials
new session keys
```

---

# 8. No Device Clone

Hard rule.

A replacement device must not impersonate the cryptographic runtime identity of the lost device.

---

# 9. Recovery Authority

Recovery proves:

```text
the claimant may create a fresh authorized device
```

not:

```text
the claimant is entitled to restore every old secret
```

---

# 10. Recovery Modes

```rust
pub enum IdentityRecoveryMode {
    ExistingDeviceApproval,
    RecoveryKey,
    ThresholdRecovery,
    ManagedRecovery,
    OfflineExportRecovery,
}
```

---

# 11. Existing Device Approval

Preferred when at least one trusted device remains.

---

# 12. Recovery Key

User-held high-entropy recovery material.

---

# 13. Threshold Recovery

Multiple independent recovery shares.

---

# 14. Managed Recovery

Organization-assisted, explicit lower-privacy trust model.

---

# 15. Offline Export Recovery

Restore from encrypted portable backup/export.

---

# 16. Recovery Key

```rust
pub struct RecoverySecret(SecretBytes);
```

Never stored plaintext remotely.

---

# 17. Recovery Identifier

Avoid stable global recovery username.

---

# 18. Recovery Locator

If remote recovery service is used, use scoped opaque locator.

```rust
pub struct RecoveryLocator(pub [u8; 32]);
```

---

# 19. Locator Privacy

Should not equal:

```text
AccountId
DeviceId
email
phone
```

---

# 20. Recovery Package

```rust
pub struct RecoveryPackage {
    pub version: RecoveryPackageVersion,
    pub encrypted_state: Bytes,
    pub account_binding: EncryptedAccountBinding,
    pub recovery_policy: RecoveryPolicy,
}
```

---

# 21. Recovery Package Contents

May include:

```text
account authority material
trusted-device roster
contact graph
verification state
user preferences
backup metadata
provider-independent durable data
```

---

# 22. Recovery Package Exclusions

Must exclude or invalidate:

```text
live session ratchets
single-use reply capabilities
transport tickets
short-lived quota credentials
active relay credentials
old mailbox receive secrets
```

---

# 23. Recovery Data Classification

```rust
pub enum RecoveryStateClass {
    DurableAuthority,
    DurableUserData,
    Rebuildable,
    Ephemeral,
    MustRotate,
    MustNotRestore,
}
```

---

# 24. DurableAuthority

Examples:

```text
account authority
recovery policy
trusted recovery roots
```

---

# 25. DurableUserData

Examples:

```text
contacts
local labels
verification notes
settings
```

---

# 26. Rebuildable

Examples:

```text
search index
provider health cache
```

---

# 27. Ephemeral

Examples:

```text
typing state
presence cache
call state
```

---

# 28. MustRotate

Examples:

```text
mailbox identity
device capabilities
provider access credentials
```

---

# 29. MustNotRestore

Examples:

```text
spent one-time capabilities
expired transport session material
```

---

# 30. Recovery Policy

```rust
pub struct RecoveryPolicy {
    pub mode: IdentityRecoveryMode,
    pub minimum_approvals: u8,
    pub require_device_revocation: bool,
    pub rotate_relationship_capabilities: bool,
}
```

---

# 31. Existing Device Approval Flow

```text
new device generates keypair
→ displays pairing request
→ existing device verifies
→ account authority authorizes
→ fresh device added
```

---

# 32. Device Addition Certificate

```rust
pub struct DeviceAuthorization {
    pub account: AccountAuthorityId,
    pub device: DeviceIdentityId,
    pub issued_at: Timestamp,
    pub expires_at: Option<Timestamp>,
    pub signature: AccountAuthoritySignature,
}
```

---

# 33. Device Authorization

Does not expose global provider identity.

---

# 34. QR/NFC Bootstrap

Can reuse Part 15/45 bootstrap.

---

# 35. Recovery-Key Flow

```text
new device
→ derive recovery decrypt key
→ decrypt recovery package
→ validate account binding
→ create fresh device identity
→ rotate required credentials
```

---

# 36. Recovery Key Derivation

Use memory-hard KDF where passphrase involved.

---

# 37. Recovery Phrase

If product offers mnemonic:

```text
must have sufficient entropy
must not be generated from weak user password
```

---

# 38. Password-Only Recovery

Discouraged for high-value account authority.

---

# 39. Threshold Recovery

Example:

```text
2-of-3
```

shares.

---

# 40. Share Holders

Could be:

```text
user hardware token
trusted offline backup
organization recovery authority
```

---

# 41. Social Recovery

Potential future.

---

# 42. Social Recovery Privacy Risk

Trusted contacts may learn:

```text
they are recovery trustees
possibly account continuity events
```

---

# 43. Recommendation

Do not make social recovery default.

---

# 44. Managed Recovery

Enterprise/school/org-controlled account recovery.

---

# 45. Managed Recovery Boundary

Organization may authorize fresh device.

---

# 46. Managed Recovery Must Not

Automatically decrypt:

```text
personal private messages
device-local secrets
```

unless product explicitly uses managed escrow.

---

# 47. No Hidden Escrow

Hard rule.

---

# 48. Lost Device

Need revocation.

---

# 49. Lost Device State

```rust
pub enum DeviceSecurityState {
    Active,
    SuspectedLost,
    Revoked,
    Retired,
}
```

---

# 50. Device Revocation

Account authority signs revocation.

---

# 51. Device Revocation Record

```rust
pub struct DeviceRevocation {
    pub device: DeviceIdentityId,
    pub reason: DeviceRevocationReason,
    pub effective_at: Timestamp,
    pub signature: AccountAuthoritySignature,
}
```

---

# 52. Revocation Reasons

```rust
pub enum DeviceRevocationReason {
    Lost,
    Stolen,
    Compromised,
    Retired,
}
```

---

# 53. Revocation Effects

Must revoke:

```text
device receiving capabilities
device mailbox capability
device group sender keys
device presence credentials
device call credentials
```

---

# 54. Group Integration

Part 43:

```text
device revoke
→ group epoch/key update where required
```

---

# 55. Social Graph Integration

Part 45:

```text
relationship remains
device capability changes
```

---

# 56. Contact Identity Continuity

Contact should see:

```text
same logical account
new device
```

if account authority is preserved.

---

# 57. Device Verification

New device may require explicit re-verification.

---

# 58. Trust Level

```rust
pub enum RecoveryTrustOutcome {
    AccountContinuityVerified,
    NewDeviceUnverified,
    NewDeviceVerified,
    SecurityChanged,
}
```

---

# 59. Contact UX

Possible:

```text
"New device added"
"Security changed"
```

---

# 60. No Silent Device Replacement

Hard rule.

---

# 61. Account Key Rotation

Sometimes account authority itself may need rotation.

---

# 62. Triggers

```text
account key compromise
recovery root compromise
cryptographic migration
```

---

# 63. Account Rotation

High-risk operation.

---

# 64. Rotation Certificate

```rust
pub struct AccountKeyTransition {
    pub old: AccountAuthorityId,
    pub new: AccountAuthorityId,
    pub effective_at: Timestamp,
    pub proof: AccountTransitionProof,
}
```

---

# 65. Normal Rotation

Old account key signs new.

---

# 66. Compromise Rotation

May require recovery authority instead.

---

# 67. Trust Continuity

Contacts receive signed transition proof.

---

# 68. Compromise Warning

If old key suspected compromised, UI must distinguish.

---

# 69. No Blind Trust

Hard rule.

---

# 70. Relationship Capability Rotation

After recovery, optionally rotate:

```text
messaging capability
presence capability
call capability
file capability
```

---

# 71. Maximum Privacy

Prefer rotate all sensitive relationship capabilities.

---

# 72. Cost

Requires contact re-synchronization.

---

# 73. Recovery Epoch

```rust
pub struct RecoveryEpoch(pub u64);
```

Monotonic.

---

# 74. Recovery Epoch Use

Can invalidate stale recovery artifacts.

---

# 75. Old Recovery Package

After successful recovery and epoch advance:

```text
may be outdated
```

---

# 76. Anti-Rollback

Client stores highest accepted recovery epoch.

---

# 77. Recovery Package Rollback

Reject lower epoch if unsafe.

---

# 78. Mailbox Recovery

Old mailbox should not simply be restored on new device.

---

# 79. Preferred Flow

```text
create new mailbox identity
→ publish new receiving descriptor
→ dual receive briefly
→ retire old
```

---

# 80. Lost Device Mailbox

If compromised/lost:

```text
revoke immediately
```

---

# 81. No Dual Receive For Compromised Secret

Hard rule.

---

# 82. Mailbox Migration State

```rust
pub enum MailboxRecoveryState {
    CreatingNew,
    Publishing,
    DualReceive,
    RetiringOld,
    Complete,
}
```

---

# 83. Reply Capability Recovery

Single-use reply capabilities are not restored.

---

# 84. New Reply Capability Pool

Generate fresh.

---

# 85. Mixnet Identity Recovery

Transport-level identifiers rotate.

---

# 86. No Stable Transport Identity Portability

Hard rule.

---

# 87. Provider Binding Recovery

Provider credentials should be re-issued.

---

# 88. Payment Wallet Recovery

Part 47 rules apply.

---

# 89. Bearer Credit Caveat

Cannot naively clone unspent bearer credits.

---

# 90. Recovery Result

May require issuer reissue/reconciliation.

---

# 91. Provider Account Portability

SIAR should minimize provider lock-in.

---

# 92. Portable State

Provider-independent representation.

---

# 93. Examples

```text
contacts
message history
group membership metadata
settings
verification state
```

---

# 94. Provider-Specific State

Examples:

```text
mailbox capability
quota token
relay credential
```

must be recreated.

---

# 95. Account Portability

User should be able to move:

```text
device
provider
region
deployment
```

without changing logical account unnecessarily.

---

# 96. Portable Account Bundle

```rust
pub struct PortableAccountBundle {
    pub version: PortableAccountBundleVersion,
    pub account_state: EncryptedAccountState,
    pub integrity: BundleIntegrityProof,
}
```

---

# 97. Portable Bundle Contents

Only durable logical state.

---

# 98. Portable Bundle Encryption

User-controlled recovery key.

---

# 99. Provider Independence

Hard rule.

---

# 100. Import

```text
validate
→ decrypt
→ migrate schema
→ create fresh device
→ rotate provider/session credentials
```

---

# 101. Export

Part 33 integrates.

---

# 102. Account Export vs Backup

Different semantics.

---

# 103. Backup

For recovery.

---

# 104. Export

For portability/user data access.

---

# 105. Recovery Metadata

May include:

```text
schema version
account authority version
recovery epoch
```

---

# 106. No Network Trace History

Hard rule.

---

# 107. Social Graph Portability

Can restore local contact graph.

---

# 108. Contact Capability Rehydration

Old live receiving capabilities may need refresh.

---

# 109. Relationship Continuity

Use stable local relationship IDs only if safe locally.

---

# 110. Provider-Facing IDs

Rotate.

---

# 111. Group Membership Portability

Account remains member.

---

# 112. Device Re-Provisioning

New device requests group device authorization.

---

# 113. Group Sender Key

Fresh.

---

# 114. Group Read History

Per policy:

```text
None
SinceJoin
ExplicitShare
FullHistory
```

---

# 115. No Automatic Full-History Restore

Hard rule unless policy explicitly permits.

---

# 116. Presence Recovery

New presence identity/capability.

---

# 117. Discovery Recovery

Handle/account identity may continue.

---

# 118. Nearby Beacon

Fresh.

---

# 119. Realtime Call Identity

Fresh relay/session credentials.

---

# 120. Backup Recovery

Part 33 rules dominate.

---

# 121. Secure Recovery State Machine

```rust
pub enum RecoveryState {
    Initiated,
    Authorized,
    PackageValidated,
    FreshDeviceCreated,
    CredentialsRotating,
    ServicesRebound,
    Completed,
    Failed,
}
```

---

# 122. No Completion Before Rotation

Hard rule.

---

# 123. Recovery Checkpoint

```rust
pub struct RecoveryCheckpoint {
    pub state: RecoveryState,
    pub completed_steps: BTreeSet<RecoveryStep>,
}
```

---

# 124. Crash Safety

Recovery resumable.

---

# 125. Idempotency

Repeated step must not duplicate device or capabilities.

---

# 126. Recovery Transaction

Some steps distributed.

Use saga-style orchestration.

---

# 127. Recovery Saga

```text
authorize
→ create device
→ revoke/rotate
→ bind services
→ sync durable data
→ finalize
```

---

# 128. Compensating Actions

If failure:

```text
revoke partially created device
invalidate temporary capabilities
```

---

# 129. Partial Recovery

Never leave ambiguous authorization.

---

# 130. Recovery Authorization Trait

```rust
pub trait RecoveryAuthorizer {
    fn authorize(
        &self,
        request: RecoveryRequest,
    ) -> Result<RecoveryAuthorization, RecoveryError>;
}
```

---

# 131. Recovery Request

```rust
pub struct RecoveryRequest {
    pub mode: IdentityRecoveryMode,
    pub fresh_device_key: DevicePublicKey,
    pub recovery_epoch: RecoveryEpoch,
}
```

---

# 132. Device Provisioner

```rust
pub trait FreshDeviceProvisioner {
    fn provision(
        &self,
        authorization: RecoveryAuthorization,
    ) -> Result<DeviceAuthorization, RecoveryError>;
}
```

---

# 133. Credential Rotation Service

```rust
pub trait RecoveryCredentialRotator {
    fn rotate_for_device(
        &self,
        device: DeviceIdentityId,
    ) -> Result<RotationReport, RecoveryError>;
}
```

---

# 134. Portability Service

```rust
pub trait AccountPortabilityService {
    fn export_bundle(
        &self,
        policy: PortabilityPolicy,
    ) -> Result<PortableAccountBundle, RecoveryError>;

    fn import_bundle(
        &self,
        bundle: PortableAccountBundle,
    ) -> Result<ImportReport, RecoveryError>;
}
```

---

# 135. Recovery Privacy Modes

```rust
pub enum RecoveryPrivacyMode {
    Standard,
    MinimizeLinkability,
    Strict,
}
```

---

# 136. Standard

May reuse some stable account-level provider associations.

---

# 137. MinimizeLinkability

Rotates provider-facing identities.

---

# 138. Strict

Rotates:

```text
mailbox IDs
provider credentials
presence IDs
transport identities
quota credentials
```

---

# 139. Recovery Timing Correlation

A recovery event can be observable.

---

# 140. Mitigation

Where practical:

```text
jitter provider re-registration
batch descriptor updates
avoid immediate global provider migration
```

---

# 141. Lost-Device Urgency

Security revocation overrides timing-obfuscation delay.

---

# 142. Recovery Event Leakage

Contacts may infer:

```text
device changed
```

---

# 143. Honest Tradeoff

Continuity requires some relationship-level notification.

---

# 144. No Provider Global Recovery Event

Hard rule.

---

# 145. Multi-Device Continuity

Remaining devices can continue operating.

---

# 146. Account Device Roster

```rust
pub struct AuthorizedDeviceRecord {
    pub device: DeviceIdentityId,
    pub state: DeviceSecurityState,
    pub authorized_at: Timestamp,
}
```

---

# 147. Device Roster Privacy

E2EE account-level state.

---

# 148. Provider Should Not Know Full Device Count

Hard rule where avoidable.

---

# 149. Device Revocation Sync

High-priority account control event.

---

# 150. Offline Device

Revocation applies when reconnecting.

---

# 151. Revocation Sequence

Monotonic.

---

# 152. Device Revocation Epoch

```rust
pub struct DeviceRevocationEpoch(pub u64);
```

---

# 153. Rollback Protection

Required.

---

# 154. Recovery Notification

User should see:

```text
new device
revoked device
recovery completed
```

---

# 155. Suspicious Recovery

If unexpected:

```text
revoke new device
rotate recovery secret
rotate account authority if needed
```

---

# 156. Recovery Attack Model

Threats:

```text
stolen recovery phrase
malicious recovery server
old-backup replay
device cloning
social recovery coercion
provider correlation
insider managed recovery
```

---

# 157. Stolen Recovery Secret

Recovery alone may be enough depending policy.

---

# 158. Stronger Policy

Require:

```text
recovery secret + existing device
or
threshold approvals
```

---

# 159. High-Security Mode

```rust
pub struct HighSecurityRecoveryPolicy {
    pub require_threshold: bool,
    pub require_delay: bool,
    pub notify_existing_devices: bool,
}
```

---

# 160. Recovery Delay

Can help detect theft.

---

# 161. But

May be harmful in emergency.

---

# 162. User-Configurable

Where appropriate.

---

# 163. Managed Recovery Insider

Needs:

```text
audit
separation of duties
threshold approval
```

---

# 164. No Silent Admin Recovery

Hard rule.

---

# 165. Managed Recovery Audit

```rust
pub struct ManagedRecoveryAudit {
    pub request_id: RecoveryRequestId,
    pub approvers: Vec<ManagedRecoveryApproverId>,
    pub timestamp: Timestamp,
}
```

---

# 166. Recovery Server

If used, stores encrypted package only.

---

# 167. Zero-Knowledge Storage Goal

Server should not have recovery decryption key.

---

# 168. Package Metadata

Minimize.

---

# 169. No Plain Email/Phone Index By Default

Hard rule for strict recovery service.

---

# 170. Recovery Locator Derivation

Use random or privacy-preserving derivation.

---

# 171. Rate Limiting

Protect recovery endpoint.

---

# 172. Recovery Brute Force

If passphrase-based, memory-hard KDF.

---

# 173. Online Guess Limiting

Also needed.

---

# 174. No Password Hint

Hard rule.

---

# 175. Recovery Key Rotation

User can rotate recovery secret.

---

# 176. Recovery Key Rotation Flow

```text
authorize
→ write new recovery package
→ increment epoch
→ invalidate old locator/package
```

---

# 177. Package Redundancy

Can store multiple encrypted copies.

---

# 178. Multi-Provider Recovery Storage

Possible.

---

# 179. Cross-Provider Linkability

Use independent locators.

---

# 180. No Shared Recovery Object ID

Hard rule.

---

# 181. Recovery Backup Verification

Periodic local check.

---

# 182. Recovery Drill

User can test recovery package without activating recovery.

---

# 183. Dry-Run Recovery

```rust
pub enum RecoveryDryRunResult {
    Valid,
    MissingShare,
    InvalidPackage,
    OutdatedPackage,
}
```

---

# 184. Dry Run

Must not create device.

---

# 185. Backup Freshness

UI can show:

```text
recovery package last updated
```

---

# 186. No Remote Telemetry Of Recovery Status

Maximum Anonymity default.

---

# 187. Portability Between Deployments

User may move from:

```text
self-hosted
→ managed
managed
→ self-hosted
provider A
→ provider B
```

---

# 188. Logical Account Continuity

Preserve account authority.

---

# 189. Provider Credentials

Always rebind.

---

# 190. Provider Data Migration

Only ciphertext/durable logical state.

---

# 191. No Provider Internal IDs In Export

Hard rule.

---

# 192. Hosted-to-Local Migration

Portable bundle imported locally.

---

# 193. Local-to-Hosted Migration

User explicitly uploads encrypted remote state.

---

# 194. Consent

Required if introducing new remote storage.

---

# 195. Account Fork

User may intentionally create new identity instead of recover.

---

# 196. Fork Semantics

No continuity proof.

---

# 197. Migration vs Fork

Must be explicit.

---

# 198. Privacy Benefit Of Fork

Breaks linkage.

---

# 199. Cost

Contacts/groups require re-introduction.

---

# 200. User Choice

Architecture supports both.

---

# 201. Recovery UX

Normal recovery flow:

```text
Choose recovery method
Verify authorization
Create fresh device
Revoke lost device if needed
Restore durable data
Re-establish private services
Review security changes
```

---

# 202. Security Summary

Show:

```text
new device ID
lost device status
credentials rotated
mailbox recreated
```

in human language.

---

# 203. No Technical Secret Exposure

Hard rule.

---

# 204. Recovery Progress

Typed stages.

---

# 205. Partial Failure UX

Explain:

```text
account recovered
mailbox still re-registering
```

---

# 206. No False Success

Hard rule.

---

# 207. Contact Notification UX

Could show:

```text
This contact added a new device.
```

---

# 208. Compromise UX

Stronger:

```text
Security credentials changed. Re-verification recommended.
```

---

# 209. Recovery Event History

Local/account E2EE.

---

# 210. No Public Recovery History

Hard rule.

---

# 211. Privacy-Safe Observability

Safe aggregate:

```text
recovery success rate
migration failure class
package version
```

---

# 212. Forbidden Telemetry

No:

```text
AccountAuthorityId
RecoveryLocator
contact graph
device roster
```

---

# 213. Support Bundle

Can include:

```text
recovery state
error class
schema version
```

---

# 214. Exclude

```text
recovery secret
package contents
device keys
mailbox capability
```

---

# 215. Recovery SLOs

Examples:

```text
fresh-device provisioning success
revocation propagation
provider rebinding success
```

---

# 216. Privacy SLOs

Examples:

```text
zero stale session restoration
zero cloned device identity
zero recovery secret telemetry
```

---

# 217. Recovery Testkit

Need deterministic recovery simulator.

---

# 218. Scenarios

```text
remaining trusted device
all devices lost
lost device later returns
stolen recovery secret
old backup replay
provider outage during recovery
group rekey failure
mailbox migration failure
```

---

# 219. Existing-Device Approval Test

Fresh device authorized correctly.

---

# 220. Device Clone Test

Old DeviceIdentityId cannot be reused.

---

# 221. Lost Device Test

Revoked device denied future capabilities.

---

# 222. Offline Lost Device Test

Reconnect sees revocation epoch.

---

# 223. Old Backup Test

Rollback blocked.

---

# 224. Mailbox Recovery Test

New mailbox identity.

---

# 225. Reply Capability Test

Old one-time caps absent.

---

# 226. Group Test

New device gets fresh sender key.

---

# 227. Relationship Test

Logical relationship retained.

---

# 228. Strict Recovery Test

Provider-facing IDs rotated.

---

# 229. Payment Wallet Test

No bearer token duplication.

---

# 230. Managed Recovery Test

Threshold/admin audit enforced.

---

# 231. Provider Migration Test

Account portable without old provider IDs.

---

# 232. Recovery Crash Tests

Crash after every stage.

---

# 233. Idempotency Test

Resume does not duplicate device/capability.

---

# 234. Fuzzing

Fuzz:

```text
recovery package
device authorization
revocation record
portable account bundle
migration checkpoint
```

---

# 235. Property Tests

Properties:

```text
recovered device always has fresh DeviceIdentityId
revoked device never regains authority without new explicit authorization
MustNotRestore state never appears in recovered store
recovery epoch never decreases
```

---

# 236. Formal Verification Targets

Strong candidates:

```text
recovery state machine
device revocation
package anti-rollback
managed threshold authorization
```

---

# 237. TLA+ Candidate

Device lost/recovery/revocation races.

---

# 238. Kani Candidate

Recovery state classification enforcement.

---

# 239. Loom Candidate

Concurrent device authorization/revocation.

---

# 240. Performance Tests

Measure:

```text
package decrypt
schema migration
social graph restore
provider rebinding
```

---

# 241. Large Account Test

Synthetic:

```text
100k contacts
large message history
many groups
```

---

# 242. Recovery Throttling

Avoid flooding providers after restore.

---

# 243. Rebinding Queue

Prioritize:

```text
security/revocation
mailbox
relationships
groups
background providers
```

---

# 244. Battery-Aware Mobile Recovery

Android should batch background re-provisioning.

---

# 245. Desktop

Daemon can continue recovery after UI closes.

---

# 246. Local-First Recovery

Core account/data restoration should work offline where possible.

---

# 247. Online Steps

Required for:

```text
device authorization propagation
provider rebinding
revocation distribution
group updates
```

---

# 248. Offline Recovery

Can restore local data but remain:

```text
NotNetworkProvisioned
```

---

# 249. Provisioning State

```rust
pub enum NetworkProvisioningState {
    LocalRecovered,
    PartiallyProvisioned,
    FullyProvisioned,
}
```

---

# 250. Security Rule

Do not send network traffic until required fresh credentials exist.

---

# 251. Crate Layout

Recommended:

```text
crates/
├── siar-recovery-core/
├── siar-account-authority/
├── siar-device-provisioning/
├── siar-device-revocation/
├── siar-recovery-package/
├── siar-recovery-policy/
├── siar-account-portability/
├── siar-recovery-migration/
├── siar-service-rebinding/
├── siar-recovery-observability/
└── siar-recovery-testkit/
```

---

# 252. `siar-recovery-core`

Owns:

```text
recovery states
epochs
modes
errors
```

---

# 253. `siar-account-authority`

Account authority/transition proofs.

---

# 254. `siar-device-provisioning`

Fresh-device creation.

---

# 255. `siar-device-revocation`

Lost/stolen/retired device handling.

---

# 256. `siar-recovery-package`

Encrypted recovery state.

---

# 257. `siar-recovery-policy`

Threshold/managed/high-security policy.

---

# 258. `siar-account-portability`

Portable logical account bundles.

---

# 259. `siar-recovery-migration`

Schema/import/checkpoint logic.

---

# 260. `siar-service-rebinding`

Mailbox/provider/group capability recreation.

---

# 261. `siar-recovery-observability`

Privacy-safe recovery metrics.

---

# 262. `siar-recovery-testkit`

Crash/replay/loss/compromise simulator.

---

# 263. Error Taxonomy

```rust
pub enum RecoveryError {
    AuthorizationFailed,
    RecoverySecretInvalid,
    ThresholdInsufficient,
    PackageInvalid,
    PackageRollback,
    DeviceAlreadyExists,
    DeviceRevoked,
    RotationFailed,
    ProviderRebindFailed,
    MigrationFailed,
    ManagedApprovalInsufficient,
    Internal,
}
```

---

# 264. Security & Privacy Invariants

Mandatory:

```text
1. Recovery creates a fresh device identity; it never clones the lost device identity.
2. Session ratchets, reply capabilities, transport tickets, relay credentials, and equivalent ephemeral secrets are never blindly restored.
3. Recovery authority proves permission to create a new authorized device, not permission to resurrect every old secret.
4. Lost/compromised devices can be revoked independently from logical account continuity.
5. Recovery packages and portable bundles are encrypted under user-controlled or explicitly managed recovery policy.
6. Recovery epochs and device-revocation epochs are monotonic and anti-rollback protected.
7. Provider-facing mailbox, transport, presence, and quota identities rotate in strict recovery mode.
8. No universal provider-visible recovery identifier is required.
9. Managed recovery is explicit, auditable, and cannot silently escrow private message keys.
10. Account portability preserves logical user state while recreating provider/session-specific credentials.
11. Bearer payment credits are not duplicated by naïve backup restore.
12. Recovery telemetry/support bundles never contain recovery secrets, device keys, contact graph, or mailbox capabilities.
```

---

# 265. Initial Production Scope

Implement first:

```text
existing-device approval
high-entropy recovery key
encrypted recovery package
fresh-device provisioning
device revocation
recovery/device epochs
social graph restore
group device re-provisioning
fresh mailbox creation
provider credential rebinding
portable account export/import
crash-safe recovery saga
local recovery progress/status
```

Then add:

```text
threshold recovery
managed multi-approver recovery
multi-provider encrypted recovery storage
account-authority compromise rotation
privacy-preserving social recovery
advanced recovery dry-run verification
```

---

# 266. Definition of Done

Part 57 is complete when:

- account authority and device identity are explicitly separated
- recovered devices always receive fresh runtime identities
- stale session/ratchet/transport/reply secrets are classified as non-restorable
- device loss, theft, compromise, and retirement have explicit revocation semantics
- recovery packages are versioned, encrypted, and anti-rollback protected
- account/social-graph continuity survives device replacement
- mailbox, presence, transport, provider, and group device credentials are re-provisioned
- strict recovery rotates provider-facing identities
- managed recovery is auditable and explicit
- account portability works across provider/self-hosted deployments
- bearer credit/wallet recovery obeys Part 47 constraints
- recovery is resumable, idempotent, and crash-safe
- UI clearly distinguishes account continuity, device continuity, and security changes
- recovery privacy/SLO/telemetry boundaries are defined
- replay, crash, lost-device, compromise, provider-outage, fuzz, and formal tests are specified

---

# 267. Final Architecture

```text
                  DURABLE ACCOUNT AUTHORITY
                            │
                            ▼
                    RECOVERY AUTHORIZATION
                            │
                            ▼
                    FRESH DEVICE IDENTITY
                            │
             ┌──────────────┼──────────────┐
             │              │              │
         Restore         Rotate         Rebind
        durable data    credentials      services
             │              │              │
             └──────────────┼──────────────┘
                            ▼
                  PRIVACY-SAFE CONTINUITY
```

Recovery safety model:

```text
restore durable authority
+
restore durable user data
+
create fresh device
+
rotate ephemeral/provider credentials
+
revoke lost devices
+
anti-rollback
```

not:

```text
copy the old device wholesale
```

---

# 268. Final Principle

Identity continuity and device continuity are different things.

The correct recovery model is:

```text
preserve the user's durable authority and relationships
while deliberately creating fresh runtime identities and secrets
```

This architecture lets SIAR survive device loss, migration, provider changes, and account recovery without turning recovery into a permanent tracking handle or a mechanism for reviving stale cryptographic state.
