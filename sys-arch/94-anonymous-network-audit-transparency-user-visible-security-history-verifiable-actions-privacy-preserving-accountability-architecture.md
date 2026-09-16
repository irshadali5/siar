# Core System Architecture Part 94 — Anonymous Network Audit, Transparency, User-Visible Security History, Verifiable Actions & Privacy-Preserving Accountability Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 94  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 28, 42, 46, 51, 53, 55–56, 61, 67, 69, 72–75, 81–93

**Primary purpose:** define SIAR's audit, transparency, user-visible security history, action receipt, evidence, append-only accountability, tamper-evident logging, scoped non-repudiation, tenant/operator accountability, federation transparency, dispute evidence, retention, anti-surveillance, and verifiable action architecture.

---

# 1. Purpose

Accountability is necessary for:

```text
security changes
administrative actions
policy changes
key rotations
device enrollment
account recovery
managed organization changes
federation changes
critical infrastructure operations
```

But conventional audit systems often become surveillance systems because they record:

```text
every message
every user action
every contact edge
every search
every session
every route
```

The governing principle is:

> **SIAR audit systems should prove that important actions occurred, who or what authority performed them, and which policy authorized them—without becoming a comprehensive history of private user behavior.**

---

# 2. Architectural Position

```text
Security / Admin / Governance Action
              │
              ▼
       Audit Classification
              │
              ▼
      Evidence Construction
              │
       ┌──────┼──────┐
       │      │      │
   Local Log  Receipt Transparency
       │      │      │
       └──────┼──────┘
              ▼
      Verification / Review
```

---

# 3. Core Separation

Keep distinct:

```text
audit event
operational log
security event
user-visible security history
transparency record
legal evidence
telemetry
message history
```

---

# 4. Non-Goals

Part 94 does not create:

```text
a universal immutable user activity log
a global message metadata ledger
a central contact graph
permanent device tracking
audit-based deanonymization
```

---

# 5. Audit Event Classes

```rust
pub enum AuditEventClass {
    Security,
    Administrative,
    Governance,
    KeyManagement,
    AccountLifecycle,
    TenantManagement,
    Federation,
    Deployment,
    Compliance,
}
```

---

# 6. Security

Examples:

```text
passkey added
session revoked
recovery started
device identity changed
```

---

# 7. Administrative

Examples:

```text
role granted
policy changed
tenant setting modified
```

---

# 8. Governance

Examples:

```text
authority policy signed
trust root rotated
emergency policy activated
```

---

# 9. Key Management

Examples:

```text
HSM signing operation
key rotation
key destruction
```

---

# 10. Account Lifecycle

Examples:

```text
suspension
deactivation
deletion
recovery
```

---

# 11. Tenant Management

Examples:

```text
admin grant
membership offboarding
managed-policy change
```

---

# 12. Federation

Examples:

```text
peer agreement created
federation trust changed
peer revoked
```

---

# 13. Deployment

Examples:

```text
release promoted
rollback executed
artifact revoked
```

---

# 14. Compliance

Examples:

```text
legal hold applied
retention policy changed
```

---

# 15. Forbidden Audit Classes

By default, do not centrally audit:

```text
ordinary message send/read
typing indicators
presence
search queries
feed reads
contact browsing
local profile changes
```

---

# 16. Hard Rule

Audit captures authority-changing or security-relevant actions, not ordinary private behavior.

---

# 17. Audit Event

```rust
pub struct AuditEvent {
    pub event_id: AuditEventId,
    pub class: AuditEventClass,
    pub action: AuditAction,
    pub actor: AuditActorRef,
    pub target: AuditTargetRef,
    pub outcome: AuditOutcome,
    pub policy_version: Option<PolicyVersion>,
    pub occurred_at: CoarseTimestamp,
}
```

---

# 18. Event ID

Random/opaque.

---

# 19. No User Account Encoding

Hard rule.

---

# 20. Audit Actor

```rust
pub enum AuditActorRef {
    UserScoped(UserAuditRef),
    AdminScoped(AdminAuditRef),
    WorkloadScoped(WorkloadAuditRef),
    GovernanceAuthority(AuthorityAuditRef),
    FederationPeer(FederationAuditRef),
    System(SystemAuditRef),
}
```

---

# 21. Scoped Actor Reference

Only meaningful in authorized review context.

---

# 22. No Global Audit Identity

Hard rule.

---

# 23. Audit Target

```rust
pub enum AuditTargetRef {
    Account(AccountAuditRef),
    Tenant(TenantAuditRef),
    Policy(PolicyAuditRef),
    Key(KeyAuditRef),
    Device(DeviceAuditRef),
    FederationPeer(FederationAuditRef),
    Release(ReleaseAuditRef),
}
```

---

# 24. Target References Are Opaque

Hard rule.

---

# 25. Audit Action

Strongly typed.

---

# 26. Example

```rust
pub enum AuditAction {
    GrantRole,
    RevokeRole,
    AddCredential,
    RevokeCredential,
    RecoverAccount,
    RotateKey,
    ChangePolicy,
    PublishRelease,
    RevokePeer,
}
```

---

# 27. No Free-Form Action String In Core

Preferred hard rule.

---

# 28. Outcome

```rust
pub enum AuditOutcome {
    Success,
    Denied,
    Failed,
    Cancelled,
}
```

---

# 29. Denied Events

Useful for high-risk operations.

---

# 30. But

Do not flood audit with ordinary denied reads.

---

# 31. Hard rule.

---

# 32. Evidence Payload

Minimal.

---

# 33. Evidence Descriptor

```rust
pub struct AuditEvidence {
    pub event: AuditEventId,
    pub action_digest: Digest,
    pub policy_digest: Option<Digest>,
    pub actor_proof: Option<ActorProof>,
    pub signatures: SignatureBundle,
}
```

---

# 34. No Raw Secret Material

Hard rule.

---

# 35. No Message Content

Hard rule.

---

# 36. No Password/Token Values

Hard rule.

---

# 37. Action Digest

Canonical digest of security-relevant action.

---

# 38. Canonical Encoding

Required.

---

# 39. Postcard/internal canonical format.

---

# 40. Signature over canonical bytes.

---

# 41. No JSON signature ambiguity.

---

# 42. Hard rule.

---

# 43. Audit Record Integrity

Tamper-evident.

---

# 44. Hash Chain

Possible.

---

# 45. Append-only sequence.

---

# 46. Example

```rust
pub struct AuditChainRecord {
    pub sequence: u64,
    pub previous_digest: Digest,
    pub event_digest: Digest,
}
```

---

# 47. Hash Chain Provides

```text
tamper detection
reordering detection
deletion detection within chain scope
```

---

# 48. It Does Not Provide

```text
global truth
real-time availability
identity correctness by itself
```

---

# 49. Hard truth.

---

# 50. Audit Chains Scoped

Per:

```text
tenant
authority
service
device
```

---

# 51. No One Global Chain

Hard rule.

---

# 52. Why

Global chain creates correlation and scaling problems.

---

# 53. User Security History

User-visible subset of account/security audit events.

---

# 54. Examples

```text
new device added
passkey added
session revoked
recovery started
security setting changed
```

---

# 55. User Security History Entry

```rust
pub struct UserSecurityHistoryEntry {
    pub event_id: AuditEventId,
    pub kind: UserSecurityEventKind,
    pub occurred_at: CoarseTimestamp,
    pub source_class: SecurityEventSourceClass,
    pub verification: SecurityHistoryVerification,
}
```

---

# 56. Security History Verification

```rust
pub enum SecurityHistoryVerification {
    Local,
    ServerSigned,
    DeviceSigned,
    AuthoritySigned,
}
```

---

# 57. User Can Distinguish Verification Level

Hard rule.

---

# 58. No Fake "Verified" Label

---

# 59. Security History Should Not Show

```text
precise IP address
precise location
full user-agent
```

by default.

---

# 60. Hard rule.

---

# 61. Coarse Device Description

Allowed.

---

# 62. Example

```text
Android phone
Linux desktop
```

---

# 63. No Stable Fingerprint

Hard rule.

---

# 64. User Security History Retention

Bounded.

---

# 65. Example:

```text
30–180 days
```

depending class.

---

# 66. Critical security changes can retain longer if user/account policy requires.

---

# 67. No Lifetime Account Surveillance Timeline

Hard rule.

---

# 68. Local Security History

Can retain more privately on device if user chooses.

---

# 69. Multi-Device Sync

Encrypted.

---

# 70. No sender/third party access.

---

# 71. Hard rule.

---

# 72. User Action Receipts

Important for sensitive actions.

---

# 73. Receipt Examples

```text
account deletion requested
device revoked
tenant role changed
backup export created
key rotated
```

---

# 74. Action Receipt

```rust
pub struct ActionReceipt {
    pub receipt_id: ReceiptId,
    pub action: AuditAction,
    pub target: AuditTargetRef,
    pub result: AuditOutcome,
    pub committed_at: CoarseTimestamp,
    pub action_digest: Digest,
    pub signer: ReceiptSignerRef,
    pub signature: SignatureBytes,
}
```

---

# 75. Receipt Purpose

User/operator can later verify what was committed.

---

# 76. Receipt Is Not Secret

But can contain sensitive metadata.

---

# 77. Store encrypted locally.

---

# 78. Hard rule.

---

# 79. Action Receipt Verification

```rust
pub trait ActionReceiptVerifier {
    fn verify(
        &self,
        receipt: &ActionReceipt,
    ) -> Result<VerifiedActionReceipt, AuditError>;
}
```

---

# 80. Receipt Includes Exact Action Digest

---

# 81. Prevents "receipt says success but action differs."

---

# 82. Hard rule.

---

# 83. Receipt Signing Key

Separate from account auth key.

---

# 84. Service-specific/authority-specific.

---

# 85. No One Universal Receipt Key

Hard rule.

---

# 86. Key Rotation

Old receipt remains verifiable via archived public key/cert chain.

---

# 87. Transparency Logs

For public/governance artifacts.

---

# 88. Suitable records:

```text
release manifests
governance policy
trust root changes
federation root changes
provider signing keys
```

---

# 89. Not suitable:

```text
user messages
private account actions
contact graph
private group membership
```

---

# 90. Hard rule.

---

# 91. Transparency Log Entry

```rust
pub struct TransparencyEntry {
    pub log_index: u64,
    pub entry_type: TransparencyEntryType,
    pub object_digest: Digest,
    pub timestamp_epoch: CoarseEpoch,
}
```

---

# 92. Append-only.

---

# 93. Merkle Tree

Recommended for public transparency.

---

# 94. Supports:

```text
inclusion proofs
consistency proofs
equivocation detection
```

---

# 95. No custom Merkle crypto.

---

# 96. Hard rule.

---

# 97. Transparency Log Identity

Per governance/release/federation domain.

---

# 98. No global universal transparency log required.

---

# 99. Hard rule.

---

# 100. Transparency Object

Canonical digest.

---

# 101. Inclusion Proof

```rust
pub struct InclusionProof {
    pub log_root: Digest,
    pub leaf_index: u64,
    pub proof: Vec<Digest>,
}
```

---

# 102. Consistency Proof

Optional.

---

# 103. Clients can verify append-only growth.

---

# 104. Transparency Witnesses

Optional independent observers.

---

# 105. Useful for:

```text
release transparency
governance transparency
federation key transparency
```

---

# 106. No user social data.

---

# 107. Hard rule.

---

# 108. Equivocation Detection

If different roots shown to different clients.

---

# 109. Witness/gossip can detect.

---

# 110. Client Stores Recent Roots

Local.

---

# 111. No user identity required.

---

# 112. Hard rule.

---

# 113. Key Transparency

Can help detect unexpected key changes.

---

# 114. But

Must not expose contact graph.

---

# 115. Possible model

lookup by scoped capability/name only.

---

# 116. No global public account key directory by default.

---

# 117. Hard rule.

---

# 118. Device Key Change Transparency

User/contact-specific.

---

# 119. Local continuity records.

---

# 120. Optional signed consistency proofs.

---

# 121. No global device identity log.

---

# 122. Hard rule.

---

# 123. Non-Repudiation

Use narrowly.

---

# 124. Not every action should be non-repudiable.

---

# 125. Why

Strong non-repudiation can harm privacy and deniability.

---

# 126. Suitable for:

```text
governance signatures
release signing
tenant admin changes
legal hold actions
high-value key ceremonies
```

---

# 127. Unsuitable for:

```text
ordinary private messages
casual reactions
typing
presence
```

---

# 128. Hard rule.

---

# 129. Messaging Deniability

Preserve where protocol requires.

---

# 130. Do Not Add Global Signature Receipts To Every Message

Hard rule.

---

# 131. Verifiable Administrative Action

Admin change receipt.

---

# 132. Example:

```rust
pub struct AdminActionProof {
    pub admin_scope: AdminScope,
    pub capability_digest: Digest,
    pub action_digest: Digest,
    pub policy_version: PolicyVersion,
    pub signature_bundle: SignatureBundle,
}
```

---

# 133. Proves

```text
which scoped authority
which capability
which policy version
which action
```

---

# 134. Does not prove human intent beyond credential control.

---

# 135. Hard truth.

---

# 136. Multi-Party Approval

For high-risk actions.

---

# 137. Examples:

```text
tenant deletion
root key rotation
governance emergency action
```

---

# 138. Approval Record

```rust
pub struct ApprovalRecord {
    pub proposal_digest: Digest,
    pub approvals: Vec<ApprovalSignature>,
    pub threshold: ApprovalThreshold,
}
```

---

# 139. Threshold Rule

Must be satisfied before commit.

---

# 140. No post-hoc fake approval.

---

# 141. Hard rule.

---

# 142. Approval Authority

Separate from execution service.

---

# 143. Separation of duties.

---

# 144. No self-approval if policy forbids.

---

# 145. Hard rule.

---

# 146. Audit Storage Classes

```rust
pub enum AuditStorageClass {
    DeviceLocal,
    AccountEncrypted,
    TenantScoped,
    GovernancePublic,
    OperatorScoped,
}
```

---

# 147. DeviceLocal

Personal security history.

---

# 148. AccountEncrypted

Own-account synced history.

---

# 149. TenantScoped

Managed admin audit.

---

# 150. GovernancePublic

Public transparency artifacts.

---

# 151. OperatorScoped

Infrastructure admin actions.

---

# 152. Hard Rule

Different audit classes use different stores.

---

# 153. No Central Universal Audit Warehouse

Hard rule.

---

# 154. Audit Store Interface

```rust
pub trait AuditStore {
    fn append(
        &self,
        event: &AuditEvent,
        evidence: &AuditEvidence,
    ) -> Result<AuditCommitReceipt, AuditError>;
}
```

---

# 155. Append-Only Semantics

Preferred.

---

# 156. Corrections

Use superseding event, not rewrite.

---

# 157. Hard rule.

---

# 158. Audit Commit Receipt

```rust
pub struct AuditCommitReceipt {
    pub event: AuditEventId,
    pub sequence: u64,
    pub chain_digest: Digest,
}
```

---

# 159. Durable.

---

# 160. No commit receipt before persistence.

---

# 161. Hard rule.

---

# 162. Audit Event Ordering

Per scope.

---

# 163. No global total order.

---

# 164. Hard rule.

---

# 165. Time

Use coarse wall time + monotonic sequence.

---

# 166. No security decision based only on timestamp.

---

# 167. Part 60 integration.

---

# 168. Clock skew

Handled.

---

# 169. Audit Sequence

Monotonic within scope.

---

# 170. Rollback/restore must preserve sequence floor.

---

# 171. Hard rule.

---

# 172. Audit Anti-Rollback

Important.

---

# 173. Store latest chain head securely.

---

# 174. Optional TPM/HSM sealing for critical authority logs.

---

# 175. No host-only guarantee.

---

# 176. Hard truth.

---

# 177. Audit Root Anchoring

For high-value logs

periodically anchor digest in:

```text
transparency log
separate witness
offline archive
```

---

# 178. No public user event anchoring.

---

# 179. Hard rule.

---

# 180. Tenant Audit

Managed organization.

---

# 181. Events:

```text
role changes
policy changes
device-management actions
managed exports
managed retention
```

---

# 182. Not:

```text
personal messages
personal contacts
private personal profile
```

---

# 183. Hard rule.

---

# 184. Tenant Audit Identity

Organization-scoped.

---

# 185. Same admin in two tenants gets separate references.

---

# 186. No cross-tenant global audit identity.

---

# 187. Hard rule.

---

# 188. Tenant Audit Viewer

Scoped capability.

---

# 189. Roles:

```text
Auditor
SecurityAdmin
ComplianceReviewer
```

---

# 190. Least privilege.

---

# 191. No tenant owner automatic access to personal data.

---

# 192. Hard rule.

---

# 193. Operator Audit

Infrastructure admin actions.

---

# 194. Examples:

```text
node quarantine
deployment rollout
secret rotation
emergency block
```

---

# 195. No user content.

---

# 196. Hard rule.

---

# 197. Operator Identity

Separate from messaging/account identity.

---

# 198. Individual accounts only.

---

# 199. No shared admin identity.

---

# 200. Hard rule.

---

# 201. Federation Audit

Records:

```text
peer trust creation
capability grant
policy change
peer revocation
```

---

# 202. No remote user activity logs.

---

# 203. Hard rule.

---

# 204. Federation Audit Scope

Per peer/domain.

---

# 205. No federation-wide superlog.

---

# 206. Hard rule.

---

# 207. Governance Audit

High-assurance.

---

# 208. Events:

```text
policy proposal
approval
signature
activation
revocation
root rollover
```

---

# 209. Public transparency for non-sensitive metadata.

---

# 210. Secret details excluded.

---

# 211. Hard rule.

---

# 212. HSM / Ceremony Audit

Part 72.

---

# 213. Ceremony record includes:

```text
ceremony ID
roles
proposal digest
device/HSM identifiers scoped
signatures
result
```

---

# 214. No private key material.

---

# 215. Hard rule.

---

# 216. Supply-Chain Audit

Part 73.

---

# 217. Release artifacts/provenance/transparency.

---

# 218. No end-user identity.

---

# 219. Hard rule.

---

# 220. Audit And Analytics Separation

Audit proves critical actions.

Analytics measures aggregates.

---

# 221. Hard Rule

Never use audit store as product analytics source.

---

# 222. Why

Audit data has richer identity context.

---

# 223. Audit And Logging Separation

Operational log is ephemeral troubleshooting.

Audit is durable evidence.

---

# 224. Hard rule.

---

# 225. Audit And Notification Separation

Security history entry may generate notification.

---

# 226. Notification is not audit record.

---

# 227. Hard rule.

---

# 228. Audit And Authorization

Audit records decision context.

---

# 229. But audit record does not grant authority.

---

# 230. Hard rule.

---

# 231. Audit Authorization

Reading audit logs requires capability.

---

# 232. Separate actions:

```text
ReadOwnSecurityHistory
ReadTenantAudit
ReadGovernanceTransparency
ReadOperatorAudit
```

---

# 233. No generic `ReadAllAudit`.

---

# 234. Hard rule.

---

# 235. Audit Query

Scoped.

---

# 236. Example

```rust
pub struct AuditQuery {
    pub scope: AuditQueryScope,
    pub class: Option<AuditEventClass>,
    pub range: AuditSequenceRange,
    pub limit: u16,
}
```

---

# 237. No arbitrary cross-scope joins.

---

# 238. Hard rule.

---

# 239. Audit Search

User history local.

---

# 240. Tenant audit exact/scoped.

---

# 241. No full-text free-form search over secrets.

---

# 242. Hard rule.

---

# 243. Audit Export

Allowed for authorized scope.

---

# 244. Export manifest.

---

# 245. Signed.

---

# 246. Sensitive references redacted.

---

# 247. Hard rule.

---

# 248. Audit Export Format

Human:

```text
RON / JSON external / CSV maybe
```

Machine:

```text
Postcard bundle
```

---

# 249. Signed manifest

canonical.

---

# 250. No ambiguous export.

---

# 251. Hard rule.

---

# 252. Evidence Bundle

```rust
pub struct EvidenceBundle {
    pub manifest: EvidenceManifest,
    pub events: Vec<AuditEvent>,
    pub receipts: Vec<ActionReceipt>,
    pub proofs: Vec<TransparencyProof>,
}
```

---

# 253. Bundle Scope

Explicit.

---

# 254. No hidden extra data.

---

# 255. Hard rule.

---

# 256. Evidence Manifest

```rust
pub struct EvidenceManifest {
    pub scope: EvidenceScope,
    pub event_count: u64,
    pub root_digest: Digest,
    pub generated_at: CoarseTimestamp,
}
```

---

# 257. Verification Tool

Offline capable.

---

# 258. No need to trust source server at verification time.

---

# 259. Good property.

---

# 260. Dispute Handling

User/admin can verify receipt vs current state.

---

# 261. Example

```text
Did I revoke this device?
Did admin remove this role?
Was this policy actually signed?
```

---

# 262. Receipt/history can answer.

---

# 263. No support agent "trust me."

---

# 264. Hard rule.

---

# 265. User-Visible Transparency

Security center can show:

```text
what changed
when
which device/admin class
verification status
```

---

# 266. Avoid:

```text
precise IP
hidden risk score
internal correlation ID
```

---

# 267. Hard rule.

---

# 268. Explainability

Each critical event can provide:

```text
action
scope
result
source class
verification
```

---

# 269. No opaque "security event occurred."

---

# 270. Hard rule.

---

# 271. Security History Alerting

High-risk changes trigger notification.

---

# 272. History remains source of truth.

---

# 273. Notification can be dismissed.

---

# 274. History entry persists per retention.

---

# 275. Hard rule.

---

# 276. User Acknowledgement

Optional.

---

# 277. Acknowledgement is local state.

---

# 278. Not evidence that user agrees.

---

# 279. Hard rule.

---

# 280. Tamper Detection

Possible indicators:

```text
chain break
signature failure
sequence gap
unexpected root
```

---

# 281. Audit Verification Status

```rust
pub enum AuditVerificationStatus {
    Verified,
    Incomplete,
    Tampered,
    Unknown,
}
```

---

# 282. Unknown ≠ Verified

Hard rule.

---

# 283. Missing Event

Gap detected.

---

# 284. No silent repair.

---

# 285. Hard rule.

---

# 286. Recovery From Corrupt Audit Store

Restore last verified snapshot.

---

# 287. Record recovery epoch.

---

# 288. Do not silently splice histories.

---

# 289. Hard rule.

---

# 290. Audit Recovery Epoch

```rust
pub struct AuditRecoveryEpoch(pub u64);
```

---

# 291. Verification acknowledges discontinuity.

---

# 292. Important.

---

# 293. Retention

Different classes.

---

# 294. Suggested:

```rust
pub enum AuditRetentionClass {
    ShortSecurity,
    StandardAdministrative,
    LongGovernance,
    LegalScoped,
}
```

---

# 295. User Security

Months.

---

# 296. Tenant Admin

Months/years per policy.

---

# 297. Governance

Longer.

---

# 298. LegalScoped

Only as required.

---

# 299. No indefinite default for all classes.

---

# 300. Hard rule.

---

# 301. Deletion

Audit data is tricky.

---

# 302. Account deletion

May remove personal security history.

---

# 303. But tenant/governance records can retain scoped admin evidence.

---

# 304. No operational account remains active.

---

# 305. Hard rule.

---

# 306. Pseudonymization

Retained audit actor refs can be detached from deleted account where legal/operationally sufficient.

---

# 307. No permanent hidden direct account link if unnecessary.

---

# 308. Hard rule.

---

# 309. Audit Tombstone

Marks retained evidence without active identity linkage.

---

# 310. Example

```rust
pub enum AuditSubjectState {
    ActiveScoped,
    Detached,
    Deleted,
}
```

---

# 311. Detached

Evidence remains, mapping removed.

---

# 312. Privacy-friendly.

---

# 313. Hard rule.

---

# 314. Legal Hold

Applies only scoped evidence.

---

# 315. No broad freeze of entire user audit history.

---

# 316. Hard rule.

---

# 317. Backup

Audit backup separate by class.

---

# 318. Public transparency logs replicated.

---

# 319. Tenant audit encrypted backup.

---

# 320. User history encrypted backup optional.

---

# 321. Deleted/expired audit entries not resurrected.

---

# 322. Hard rule.

---

# 323. Audit Backup Anti-Rollback

Restore must verify:

```text
sequence
chain head
recovery epoch
retention ledger
```

---

# 324. No rollback to old log head.

---

# 325. Hard rule.

---

# 326. Multi-Region

Audit storage may replicate.

---

# 327. Replication order consistent per scope.

---

# 328. No global ordering required.

---

# 329. Hard rule.

---

# 330. Tenant Audit Consistency

Strong enough that admin action and audit receipt do not diverge.

---

# 331. Transaction Pattern

```text
validate action
→ commit domain change
→ write audit outbox
→ commit
→ append audit evidence
```

---

# 332. But

For high-risk actions receipt may require synchronous evidence reservation.

---

# 333. Need explicit action class.

---

# 334. Hard rule.

---

# 335. High-Assurance Action

Possible pattern:

```text
prepare action
→ produce approval/evidence
→ commit domain change
→ append audit record
→ return receipt
```

---

# 336. If append fails after commit

durable recovery queue.

---

# 337. Never return "audited" receipt before append durability.

---

# 338. Hard rule.

---

# 339. Audit Outbox

```rust
pub struct AuditOutboxEntry {
    pub event: AuditEvent,
    pub evidence: AuditEvidence,
    pub state: AuditOutboxState,
}
```

---

# 340. State

```rust
pub enum AuditOutboxState {
    Pending,
    Appending,
    Appended,
    FailedRetryable,
    FailedPermanent,
}
```

---

# 341. Idempotent append.

---

# 342. Same event ID

same record.

---

# 343. No duplicate evidence.

---

# 344. Hard rule.

---

# 345. Audit Record Serialization

Canonical Postcard.

---

# 346. Human export RON/JSON.

---

# 347. External standards optional later.

---

# 348. No signing over non-canonical JSON.

---

# 349. Hard rule.

---

# 350. Hash Algorithm

Part 66 registry.

---

# 351. Signature Algorithm

Part 66.

---

# 352. No hard-coded one algorithm forever.

---

# 353. Hard rule.

---

# 354. Cryptographic Agility

Audit records include suite ID.

---

# 355. Old signatures remain verifiable.

---

# 356. Migration supported.

---

# 357. No rewrite of historical event content merely to change algorithm.

---

# 358. Hard rule.

---

# 359. PQ Migration

Future.

---

# 360. Public transparency signatures can migrate with dual-sign period.

---

# 361. No homegrown PQ scheme.

---

# 362. Hard rule.

---

# 363. Trusted Time

Audit timestamp not sole evidence.

---

# 364. Sequence + signed epoch + chain.

---

# 365. No NTP-only trust.

---

# 366. Hard rule.

---

# 367. Transparency Log Time

Coarse.

---

# 368. Avoid unnecessary precise timing of sensitive governance actions if not needed.

---

# 369. Hard rule.

---

# 370. Audit Privacy Threat Model

Attacks:

```text
social graph reconstruction
administrator stalking
timeline correlation
audit exfiltration
cross-tenant correlation
```

---

# 371. Social Graph Reconstruction

Do not log ordinary communication edges.

---

# 372. Admin Stalking

Scoped access/audit viewer actions.

---

# 373. Timeline Correlation

coarse time/minimal actor refs.

---

# 374. Audit Exfiltration

encryption/access control.

---

# 375. Cross-Tenant Correlation

separate scoped refs.

---

# 376. Hard rule.

---

# 377. Audit Access Is Audited

High-risk.

---

# 378. Example

tenant auditor exports records.

---

# 379. Record audit access/export event.

---

# 380. But

Avoid recursive logging explosion.

---

# 381. Hard rule.

---

# 382. Audit-Of-Audit

Only significant events:

```text
audit export
retention change
viewer grant
log recovery
```

---

# 383. Not every read/page fetch.

---

# 384. Hard rule.

---

# 385. Privacy-Preserving Audit Query

Use bounded filters.

---

# 386. No free cross-scope joins.

---

# 387. No contact/message IDs.

---

# 388. Hard rule.

---

# 389. Operator Forensics

Separate from normal audit.

---

# 390. Incident-specific evidence collection.

---

# 391. Must not retroactively transform audit system into surveillance.

---

# 392. Hard rule.

---

# 393. Incident Expansion

Requires:

```text
incident ID
scope
approval
TTL
```

---

# 394. Auto-expire.

---

# 395. No permanent broad logging after incident.

---

# 396. Hard rule.

---

# 397. Public Transparency Portal

Can publish:

```text
release roots
governance roots
federation authority changes
security advisories
```

---

# 398. No user data.

---

# 399. Hard rule.

---

# 400. Client Verification

Clients can pin/verify recent transparency roots.

---

# 401. No login required.

---

# 402. Good.

---

# 403. Witness Protocol

Optional.

---

# 404. Multiple independent witnesses.

---

# 405. No witness sees user event data.

---

# 406. Hard rule.

---

# 407. User-Visible Verification

Security center can verify:

```text
current device list
credential changes
recovery changes
account state changes
```

---

# 408. Each event can include verification badge.

---

# 409. No hidden "trusted by server" claim.

---

# 410. Hard rule.

---

# 411. Device Security History

Device-local.

---

# 412. Examples:

```text
app lock changed
local DB rekeyed
backup restored
```

---

# 413. Not necessarily server-synced.

---

# 414. Hard rule.

---

# 415. Local Action Receipt

Can be device signed.

---

# 416. Example backup export.

---

# 417. No remote server needed.

---

# 418. Good local-first property.

---

# 419. Managed Device Audit

Only managed profile actions.

---

# 420. Personal profile excluded.

---

# 421. Hard rule.

---

# 422. Role Assignment Audit

Records:

```text
granter scope
grantee scoped ref
role/capability
scope
result
```

---

# 423. No unnecessary personal identity.

---

# 424. Hard rule.

---

# 425. Secret Access Audit

Part 80.

---

# 426. Record:

```text
secret class
consumer service
purpose
result
```

---

# 427. Do not record secret value.

---

# 428. Hard rule.

---

# 429. HSM Signing Audit

Record typed signing purpose.

---

# 430. No arbitrary digest-only unexplained signing.

---

# 431. Hard rule.

---

# 432. Release Audit

Part 73.

---

# 433. Records:

```text
artifact digest
verification manifest
promotion state
signatures
```

---

# 434. Publicly verifiable.

---

# 435. No user data.

---

# 436. Hard rule.

---

# 437. Deployment Audit

Records:

```text
who authorized
artifact digest
target class
rollout policy
result
```

---

# 438. No host secrets.

---

# 439. Hard rule.

---

# 440. Federation Peer Audit

Records:

```text
domain identity
agreement digest
trust mode
activation/revocation
```

---

# 441. No remote member activity.

---

# 442. Hard rule.

---

# 443. Governance Policy Audit

Records:

```text
policy digest
approval signatures
activation epoch
superseded version
```

---

# 444. Public where safe.

---

# 445. Hard rule.

---

# 446. Experimentation Audit

Part 93.

---

# 447. Record only:

```text
experiment approved
experiment paused
guardrail rollback
```

---

# 448. Not participant assignment.

---

# 449. Hard rule.

---

# 450. Analytics Audit

Part 92.

---

# 451. Record:

```text
metric schema changes
retention changes
DP budget policy changes
```

---

# 452. Not individual metric contributions.

---

# 453. Hard rule.

---

# 454. Audit Observability

Safe metrics:

```text
append latency
verification failures
chain gaps
export count
retention jobs
```

---

# 455. Forbidden:

```text
user event volume by identity
admin-to-user interaction graph
```

---

# 456. Hard rule.

---

# 457. Audit SLOs

Examples:

```text
critical audit append durability
chain verification success
receipt generation latency
transparency consistency
```

---

# 458. Security SLO

```text
0 unsigned critical governance records
0 undetected chain rollback
```

---

# 459. Privacy SLO

```text
0 ordinary message/contact/search/feed events in central audit
0 cross-tenant identity correlation
```

---

# 460. Failure Modes

```text
audit store outage
signature failure
chain corruption
transparency split view
retention job failure
```

---

# 461. Audit Store Outage

High-risk actions may fail closed if audit required synchronously.

---

# 462. Low-risk audited actions can commit + durable outbox.

---

# 463. Explicit by action class.

---

# 464. Hard rule.

---

# 465. Signature Failure

Critical evidence not accepted.

---

# 466. No unsigned fallback.

---

# 467. Hard rule.

---

# 468. Chain Corruption

Mark verification status Tampered/Incomplete.

---

# 469. Preserve evidence.

---

# 470. No silent repair.

---

# 471. Hard rule.

---

# 472. Transparency Split View

Witness/gossip detects.

---

# 473. Security incident.

---

# 474. Freeze affected trust update if necessary.

---

# 475. Hard rule.

---

# 476. Retention Job Failure

Alert.

---

# 477. Retry.

---

# 478. Do not retain forever silently.

---

# 479. Hard rule.

---

# 480. Testing

Need dedicated audit/transparency testkit.

---

# 481. Test Scenarios

```text
admin role grant
account recovery
chain tamper
transparency inclusion
audit export
```

---

# 482. Append Test

Committed event produces exactly one audit record.

---

# 483. Duplicate Test

Retry idempotent.

---

# 484. Signature Test

Invalid signature rejected.

---

# 485. Chain Test

Deleted/reordered event detected.

---

# 486. Recovery Test

Recovery epoch preserves discontinuity.

---

# 487. Receipt Test

Receipt digest matches committed action.

---

# 488. User History Test

Only allowed security events visible.

---

# 489. Privacy Test

Ordinary message/search/contact events absent.

---

# 490. Tenant Test

Tenant A cannot query B audit.

---

# 491. Managed Profile Test

Org cannot see personal security history.

---

# 492. Federation Test

No user activity in peer audit.

---

# 493. Transparency Test

Inclusion/consistency proofs verify.

---

# 494. Split-View Test

Witness detects conflicting roots.

---

# 495. Retention Test

Expired events deleted/tombstoned per policy.

---

# 496. Backup Test

Expired records not resurrected.

---

# 497. Deletion Test

Deleted account mapping detached where required.

---

# 498. HSM Test

Signing event records typed purpose, no key material.

---

# 499. Experiment Test

No participant IDs in experiment audit.

---

# 500. Analytics Test

No individual contribution in analytics audit.

---

# 501. Fuzzing

Fuzz:

```text
audit record
evidence bundle
transparency proof
receipt parser
audit export manifest
```

---

# 502. Property Tests

Properties:

```text
audit chain sequence never decreases
critical record without valid signature can never be Verified
ordinary user behavior class can never enter central audit store
tenant-scoped audit reference cannot authorize cross-tenant query
```

---

# 503. Formal Verification Targets

Strong candidates:

```text
append-only chain
multi-party approval
receipt/action binding
retention + backup anti-resurrection
```

---

# 504. Kani Candidate

audit class/storage-class/privacy invariants.

---

# 505. TLA+ Candidate

domain commit → audit outbox → append → crash → retry → receipt.

---

# 506. Loom Candidate

concurrent append/retention/export.

---

# 507. Performance

Audit is not hot path for ordinary messaging.

---

# 508. Critical admin actions can pay higher cost.

---

# 509. User security history low volume.

---

# 510. Public transparency append batched.

---

# 511. No per-message audit overhead.

---

# 512. Hard rule.

---

# 513. Storage

Separate stores:

```text
user security history
tenant audit
operator audit
governance transparency
federation audit
```

---

# 514. No universal table.

---

# 515. Hard rule.

---

# 516. Partitioning

By:

```text
tenant
authority
service
federation peer
```

---

# 517. Never by global user graph.

---

# 518. Hard rule.

---

# 519. Indexing

Audit query index by:

```text
event class
action class
coarse time
scope
```

---

# 520. No full-text index of sensitive notes.

---

# 521. Hard rule.

---

# 522. Free-Form Notes

Avoid.

---

# 523. Structured reason codes.

---

# 524. If notes required

encrypted/restricted/retention-bounded.

---

# 525. Hard rule.

---

# 526. Crate Layout

Recommended:

```text
crates/
├── siar-audit-core/
├── siar-audit-event/
├── siar-audit-chain/
├── siar-action-receipt/
├── siar-security-history/
├── siar-transparency-log/
├── siar-audit-export/
├── siar-audit-retention/
├── siar-audit-observability/
└── siar-audit-testkit/
```

---

# 527. `siar-audit-core`

Owns:

```text
event IDs
classes
actions
outcomes
errors
```

---

# 528. `siar-audit-event`

Typed event/evidence construction.

---

# 529. `siar-audit-chain`

Append-only sequence/hash-chain verification.

---

# 530. `siar-action-receipt`

Signed user/admin receipts.

---

# 531. `siar-security-history`

User-visible security timeline.

---

# 532. `siar-transparency-log`

Merkle append-only public transparency.

---

# 533. `siar-audit-export`

Signed evidence bundles.

---

# 534. `siar-audit-retention`

Retention/deletion/legal-hold integration.

---

# 535. `siar-audit-observability`

Audit pipeline health only.

---

# 536. `siar-audit-testkit`

tamper/receipt/transparency/privacy/tenant tests.

---

# 537. Error Taxonomy

```rust
pub enum AuditError {
    InvalidEvent,
    UnsupportedAction,
    SignatureInvalid,
    ChainBroken,
    SequenceRollback,
    ReceiptMismatch,
    TransparencyProofInvalid,
    UnauthorizedAuditAccess,
    RetentionViolation,
    ScopeMismatch,
    StorageUnavailable,
    Internal,
}
```

---

# 538. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. Central audit captures security-, governance-, administrative-, and authority-changing events—not ordinary private user behavior.
2. Audit actor/target references are scope-bound and cannot become global user, device, contact, or cross-tenant identifiers.
3. Audit records contain digests, typed action metadata, outcomes, and signatures—not message plaintext, credentials, tokens, or private content.
4. User-visible security history is a privacy-minimized subset of security/account events and does not expose precise location, IP history, or device fingerprints by default.
5. Critical action receipts are cryptographically bound to the exact committed action and cannot be generated before durable evidence exists.
6. Public transparency logs contain governance/release/federation trust artifacts only and never private account, membership, message, or social-graph data.
7. Non-repudiation is applied narrowly to administrative/governance/high-value actions and is not imposed on ordinary private messaging or ephemeral interactions.
8. Tenant, operator, federation, governance, and personal audit stores remain separate; there is no universal audit superstore or cross-scope query.
9. Audit chains, sequence numbers, recovery epochs, and transparency proofs detect rollback, omission, reordering, tampering, or equivocation where the relevant assurance model supports it.
10. Audit retention is bounded by event class; expired/deleted records cannot be resurrected by backups, replays, or old replicas.
11. Audit access/export, retention-policy changes, and log recovery are themselves accountable high-risk actions, without recursively logging every read operation.
12. Audit data is never reused as product analytics, behavioral profiling, advertising, social-graph reconstruction, or generalized surveillance data.
```

---

# 539. Initial Production Scope

Implement first:

```text
typed AuditEventClass/AuditAction
scoped actor/target refs
canonical audit event encoding
signed AuditEvidence
per-scope append-only hash chains
transactional audit outbox
idempotent append
signed ActionReceipt
user-visible security history
tenant/operator/governance audit separation
basic public transparency log
inclusion/consistency verification
scoped audit query capabilities
bounded retention/deletion
backup anti-rollback
privacy-safe audit metrics
audit/transparency testkit
```

Then add:

```text
independent transparency witnesses
advanced key transparency
multi-party evidence bundles
offline verification CLI
threshold-signature/approval adapters
formal append-only/transparency verification
```

---

# 540. Definition of Done

Part 94 is complete when:

- only security/admin/governance authority-changing events are centrally audited
- actor/target references are scope-bound and privacy-safe
- audit records never contain secrets/private content
- user security history is separately modeled and minimized
- action receipts verify exact committed actions
- audit chains are append-only/tamper-evident
- public transparency logs contain only public trust artifacts
- non-repudiation is narrow, not universal
- tenant/operator/federation/governance stores stay isolated
- retention and backup anti-resurrection work
- critical audit failures fail safely according to action class
- exports are signed/scoped/redacted
- audit access is capability-controlled
- analytics cannot consume audit data as behavior telemetry
- tamper/receipt/transparency/tenant/fuzz/formal tests are specified

---

# 541. Final Architecture

```text
              SECURITY / ADMIN ACTION
                       │
                       ▼
                AUDIT CLASSIFIER
                       │
                       ▼
                 EVIDENCE BUILDER
                       │
          ┌────────────┼────────────┐
          │            │            │
      HASH CHAIN   ACTION RECEIPT  TRANSPARENCY
          │            │            │
          └────────────┼────────────┘
                       ▼
                VERIFICATION LAYER
                       │
          ┌────────────┼────────────┐
          │            │            │
     USER HISTORY   TENANT AUDIT   PUBLIC TRUST
```

Accountability safety model:

```text
typed critical events
+
scoped identities
+
canonical evidence
+
signed receipts
+
append-only chains
+
public transparency where appropriate
+
bounded retention
+
strict privacy boundaries
```

not:

```text
record everything every user does forever and call the resulting surveillance database an audit log
```

---

# 542. Final Principle

Accountability should prove important security and authority changes without making ordinary private behavior permanently observable.

The correct model is:

```text
audit narrowly
+
scope identities
+
sign exact actions
+
show users meaningful security history
+
publish only public trust artifacts
+
detect tampering
+
retain only as long as necessary
+
never turn audit into surveillance
```

This architecture gives SIAR a privacy-preserving accountability foundation for user security history, administrative actions, tenant governance, key operations, releases, federation, transparency, and dispute verification while preserving the anonymity, local-first, least-authority, and anti-surveillance guarantees established across Parts 34–93.
