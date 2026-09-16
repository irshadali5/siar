# Core System Architecture Part 83 — Anonymous Network Account Lifecycle, Registration, Enrollment, Suspension, Deactivation, Deletion & Identity-State Governance Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 83  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 2, 28, 33, 45–47, 53, 56–58, 61, 67, 69, 81–82  

**Primary purpose:** define SIAR's account lifecycle and identity-state governance architecture, including registration, invitations, enrollment, activation, suspension, security lock, deactivation, deletion, managed offboarding, retention, recovery windows, legal-hold constraints, anti-resurrection rules, session/key revocation, and lifecycle-specific authorization.

---

# 1. Purpose

Accounts move through states over time.

If account lifecycle is not explicit, systems often create inconsistent behavior such as:

```text
deleted accounts still having active sessions
suspended users continuing background sync
deactivated enterprise members retaining group keys
recovered accounts reviving old device identities
old backups resurrecting deleted records
```

The governing principle is:

> **Account lifecycle must be a first-class state machine with explicit transition authority, durable invariants, revocation effects, retention rules, and anti-resurrection guarantees.**

---

# 2. Architectural Position

```text
Registration / Invitation
          │
          ▼
      Enrollment
          │
          ▼
        Active
     ┌────┼────┐
     │    │    │
Suspended │ Deactivated
     │    │    │
     └────┼────┘
          ▼
      Deletion Flow
          │
          ▼
       Deleted
```

---

# 3. Core Separation

Keep distinct:

```text
account state
authentication state
authorization state
device state
tenant membership
federation relationship
data retention state
```

---

# 4. Non-Goals

Part 83 does not:

```text
make all account types use identical lifecycle rules
equate suspension with deletion
equate deactivation with credential compromise
allow deletion to silently erase legally retained records
restore historical device/session secrets during recovery
```

---

# 5. Account Types

```rust
pub enum AccountType {
    Personal,
    ManagedEnterprise,
    Operator,
    ServiceAccount,
}
```

---

# 6. Personal Account

User-owned.

---

# 7. Managed Enterprise

Organization-scoped identity.

---

# 8. Operator

Infrastructure administration identity.

---

# 9. Service Account

Headless service/user-like account where truly needed.

---

# 10. Hard Rule

Lifecycle rules are type-specific.

---

# 11. Account Lifecycle State

```rust
pub enum AccountState {
    PendingRegistration,
    PendingEnrollment,
    Active,
    Suspended,
    SecurityLocked,
    Deactivated,
    PendingDeletion,
    Deleted,
}
```

---

# 12. State Machine

All account lifecycle changes use validated transitions.

---

# 13. No Direct State Mutation

Hard rule.

---

# 14. Lifecycle Transition

```rust
pub struct AccountTransition {
    pub account: AccountId,
    pub from: AccountState,
    pub to: AccountState,
    pub reason: AccountTransitionReason,
    pub authority: TransitionAuthority,
}
```

---

# 15. Transition Authority

```rust
pub enum TransitionAuthority {
    User,
    OrganizationAdmin,
    SecurityEngine,
    RecoveryAuthority,
    Operator,
    LegalHoldAuthority,
}
```

---

# 16. No Universal Operator Authority

Hard rule.

---

# 17. Personal Account Lifecycle

User is primary authority.

---

# 18. Managed Account Lifecycle

Organization controls managed membership.

---

# 19. Operator Account Lifecycle

Security/governance controls.

---

# 20. Registration

Creates initial account authority.

---

# 21. Registration Modes

```rust
pub enum RegistrationMode {
    Open,
    InviteOnly,
    ManagedProvisioning,
    LocalOnly,
}
```

---

# 22. Open

Public self-registration.

---

# 23. InviteOnly

Capability/invite required.

---

# 24. ManagedProvisioning

Enterprise/admin creates membership.

---

# 25. LocalOnly

No server account.

---

# 26. Hard Rule

Core local P2P mode may exist without registration.

---

# 27. Registration Request

```rust
pub struct RegistrationRequest {
    pub mode: RegistrationMode,
    pub login_handle: Option<LoginHandle>,
    pub invite: Option<RegistrationInvite>,
}
```

---

# 28. Minimal Data

Collect only what mode requires.

---

# 29. No Mandatory Phone Number

Hard rule.

---

# 30. No Mandatory Social Login

Hard rule.

---

# 31. Invite Token

Opaque.

---

# 32. Single-use where appropriate.

---

# 33. Invite Token State

```rust
pub enum InviteState {
    Issued,
    Redeemed,
    Revoked,
    Expired,
}
```

---

# 34. No Reusable Admin Invite Token

Hard rule.

---

# 35. Enrollment

Transforms account into usable state.

---

# 36. Enrollment Includes

```text
credential setup
recovery setup
device enrollment
policy acceptance
```

---

# 37. Enrollment State

```rust
pub struct EnrollmentProgress {
    pub credential_ready: bool,
    pub recovery_ready: bool,
    pub device_ready: bool,
    pub policy_ready: bool,
}
```

---

# 38. Active Transition

Only when required enrollment conditions satisfied.

---

# 39. No Partial Active State

Hard rule.

---

# 40. Enrollment Expiry

Pending account expires after policy-defined period.

---

# 41. Abandoned Registration

Cleanup.

---

# 42. No Infinite Pending Account Retention

Hard rule.

---

# 43. Activation

Explicit transition to Active.

---

# 44. Activation Side Effects

```text
enable session issuance
enable authorized service access
enable optional cloud/mailbox features
```

---

# 45. Active Does Not Mean Full Authority

Authorization still Part 81.

---

# 46. Hard Rule

AccountState::Active is not a super-permission.

---

# 47. Suspension

Temporary access restriction.

---

# 48. Suspension Reasons

```rust
pub enum SuspensionReason {
    AbuseInvestigation,
    BillingPolicy,
    ManagedPolicy,
    SecurityConcern,
    Administrative,
}
```

---

# 49. Suspension Effects

Can include:

```text
block new sessions
restrict send/write
preserve read/export
preserve recovery
```

---

# 50. Suspension Profile

```rust
pub struct SuspensionProfile {
    pub allow_login: bool,
    pub allow_read: bool,
    pub allow_write: bool,
    pub allow_export: bool,
}
```

---

# 51. No One Suspension Behavior For All Account Types

Hard rule.

---

# 52. Security Lock

Different from suspension.

---

# 53. SecurityLock

Triggered by suspected compromise.

---

# 54. Security Lock Effects

```text
revoke sessions
require recovery/step-up
block high-risk changes
```

---

# 55. Hard Rule

Security lock is security state, not moderation punishment.

---

# 56. Deactivation

Longer-term inactive state.

---

# 57. Personal Deactivation

User choice.

---

# 58. Managed Deactivation

Org offboarding.

---

# 59. Deactivation Effects

```text
no normal login
no new sessions
background service paused
data retained per policy
```

---

# 60. Deactivation Is Reversible

Usually.

---

# 61. Deleted Is Not

Usually.

---

# 62. Hard Rule

Deactivation cannot be implemented as deletion shortcut.

---

# 63. Managed Offboarding

Critical.

---

# 64. Offboarding Flow

```text
remove org roles
revoke org capabilities
revoke org sessions
rotate group keys
disable managed integrations
detach managed storage access
```

---

# 65. Personal Identity

Unaffected where separate.

---

# 66. Hard Rule

Org offboarding must not delete personal account state.

---

# 67. Membership State

Separate from AccountState.

---

# 68. Organization Membership

```rust
pub enum MembershipState {
    Invited,
    Active,
    Suspended,
    Offboarded,
}
```

---

# 69. Account May Remain Active

Even if membership offboarded.

---

# 70. No Account/Membership Conflation

Hard rule.

---

# 71. Tenant Membership

Part 69.

---

# 72. Tenant Offboarding

Tenant-scoped.

---

# 73. No Cross-Tenant Cascade

Hard rule.

---

# 74. Federation Relationship

Separate.

---

# 75. Federation Unlink

Revokes bilateral capability.

---

# 76. Does not delete local account.

---

# 77. Hard rule.

---

# 78. Account Suspension Authority

Must be scoped.

---

# 79. Personal Account

Platform may suspend only service-specific rights where justified.

---

# 80. Managed Account

Org may suspend managed membership.

---

# 81. Operator

Security authority may suspend admin access.

---

# 82. No Org Authority Over Personal Identity

Hard rule.

---

# 83. Suspension Audit

High-risk administrative event.

---

# 84. Minimal.

---

# 85. No message-content evidence stored in lifecycle record.

---

# 86. Lifecycle Reason

Structured.

---

# 87. Free-form notes restricted.

---

# 88. Account State Query

Authorization-gated.

---

# 89. No Public Account-State Enumeration

Hard rule.

---

# 90. Deletion

Multi-stage.

---

# 91. Why

Need:

```text
revoke access
stop new writes
delete active data
process backups
handle legal retention
```

---

# 92. Deletion State Machine

```rust
pub enum DeletionState {
    Requested,
    CoolingOff,
    AccessRevoked,
    ActiveDataDeleting,
    BackupPending,
    LegallyRetainedSubset,
    Completed,
    Failed,
}
```

---

# 93. PendingDeletion

Account lifecycle state while deletion workflow runs.

---

# 94. New Sessions

Forbidden.

---

# 95. New Writes

Forbidden except deletion workflow.

---

# 96. Hard Rule

PendingDeletion cannot silently return to Active without explicit cancellation/recovery policy.

---

# 97. Deletion Request

High-risk action.

---

# 98. Requires

```text
step-up authentication
explicit confirmation
```

---

# 99. Managed Account Deletion

Depends on data ownership.

---

# 100. Organization-Owned Data

May remain with org.

---

# 101. Personal Data

User deletion scope applies.

---

# 102. Shared Data

Ownership/reference rules.

---

# 103. Hard Rule

Deletion semantics are based on ownership class, not account row ownership alone.

---

# 104. Data Ownership

```rust
pub enum DataOwnership {
    Personal,
    OrganizationOwned,
    Shared,
    Infrastructure,
}
```

---

# 105. Personal

Delete per user request/retention.

---

# 106. OrganizationOwned

Org controls retention.

---

# 107. Shared

Remove user authority/reference, preserve shared record if still needed.

---

# 108. Infrastructure

Retention independent.

---

# 109. Deletion Planner

```rust
pub trait AccountDeletionPlanner {
    fn plan(
        &self,
        account: AccountId,
    ) -> Result<DeletionPlan, LifecycleError>;
}
```

---

# 110. Deletion Plan

Lists scoped data classes.

---

# 111. No Generic `DELETE FROM everything WHERE account_id`

Hard rule.

---

# 112. Why

Data ownership and denormalization differ.

---

# 113. Active Data Delete

Part 67 lifecycle engine.

---

# 114. Tombstones

Prevent resurrection.

---

# 115. Deletion Barrier

Required.

---

# 116. Deletion Barrier

```rust
pub struct AccountDeletionBarrier {
    pub account: AccountId,
    pub deletion_epoch: u64,
}
```

---

# 117. Any stale restore/sync older than deletion epoch rejected.

---

# 118. Hard Rule

Deleted account state cannot reappear from backup/offline replica.

---

# 119. Deletion Epoch

Monotonic.

---

# 120. Restore must check.

---

# 121. Backup Processing

Part 33/67.

---

# 122. Backup May Retain Encrypted data until expiry.

---

# 123. But

Active restore blocked by deletion barrier.

---

# 124. Hard rule.

---

# 125. Cryptographic Erasure

Useful.

---

# 126. Per-account/tenant DEK destruction where architecture supports.

---

# 127. No Global DEK.

---

# 128. Hard rule.

---

# 129. Session Revocation On Deletion

Immediate.

---

# 130. Refresh families revoked.

---

# 131. Recovery methods revoked.

---

# 132. Device account bindings removed.

---

# 133. Messaging Device Identity

Separate.

---

# 134. If account deletion includes communication identity deletion

Explicit.

---

# 135. Local-only messaging data

May remain on user devices until local deletion.

---

# 136. No remote magic wipe claim.

---

# 137. Hard truth.

---

# 138. Contact Data On Other Users

Cannot always be deleted remotely.

---

# 139. Example

Messages already received by peers.

---

# 140. Deletion UX/Policy Must Be Truthful

Hard rule.

---

# 141. Group Membership

Remove account/device authority.

---

# 142. Rotate group keys where required.

---

# 143. Historical messages

Depend on protocol/ownership.

---

# 144. No retroactive plaintext deletion promise.

---

# 145. Mailbox

Revoke mailbox capability.

---

# 146. Delete queued ciphertext per policy.

---

# 147. Directory

Remove discoverability entries.

---

# 148. Naming

Part 59 handles handle retirement.

---

# 149. Handle Reuse

Privacy-sensitive.

---

# 150. Prefer cooldown or permanent non-reuse depending namespace.

---

# 151. No Immediate Reassignment Of Sensitive Public Handle

Hard rule.

---

# 152. Account Identifier Reuse

Never.

---

# 153. Hard rule.

---

# 154. Login Handle Reuse

Policy-specific.

---

# 155. Email

May be reused eventually.

---

# 156. Internal AccountId

Never reused.

---

# 157. Deactivation vs Deletion

Deactivation preserves account identity.

---

# 158. Deletion permanently retires account identity.

---

# 159. Reactivation

Allowed from Deactivated.

---

# 160. Reactivation Requirements

```text
strong authentication/recovery
policy check
```

---

# 161. No Reactivation From Deleted

Hard rule.

---

# 162. Deleted Account

New registration gets new AccountId.

---

# 163. Security Lock Recovery

May return to Active.

---

# 164. Suspension Appeal

May return to Active.

---

# 165. Transition Validation

```rust
pub trait AccountLifecycleTransitionValidator {
    fn validate(
        &self,
        current: AccountState,
        requested: AccountState,
        authority: &TransitionAuthority,
    ) -> Result<(), LifecycleError>;
}
```

---

# 166. Invalid Transition

Denied.

---

# 167. Example

```text
Deleted → Active
```

invalid.

---

# 168. Example

```text
Active → PendingDeletion
```

valid with user authority/step-up.

---

# 169. Example

```text
Suspended → Deleted
```

may require deletion workflow.

---

# 170. No Direct Jump Bypassing Required Side Effects

Hard rule.

---

# 171. Lifecycle Command

```rust
pub enum AccountLifecycleCommand {
    Register,
    CompleteEnrollment,
    Suspend,
    SecurityLock,
    Deactivate,
    Reactivate,
    RequestDeletion,
    CancelDeletion,
    FinalizeDeletion,
}
```

---

# 172. Commands Are Idempotent

Required.

---

# 173. Event Model

```rust
pub enum AccountLifecycleEvent {
    Registered,
    EnrollmentCompleted,
    Activated,
    Suspended,
    SecurityLocked,
    Deactivated,
    Reactivated,
    DeletionRequested,
    DeletionCompleted,
}
```

---

# 174. Event Retention

Security/admin events bounded per policy.

---

# 175. No Lifetime Surveillance Log

Hard rule.

---

# 176. Account Lifecycle Repository

```rust
pub trait AccountLifecycleRepository {
    fn state(
        &self,
        account: AccountId,
    ) -> Result<AccountState, LifecycleError>;

    fn transition(
        &self,
        command: AccountLifecycleCommand,
    ) -> Result<AccountLifecycleEvent, LifecycleError>;
}
```

---

# 177. Transactional Transition

State + outbox event atomically.

---

# 178. Hard Rule

No lifecycle event without committed state.

---

# 179. Outbox Integration

Part 75.

---

# 180. Side Effects

Async workflows.

---

# 181. Example Suspension Side Effects

```text
revoke session
disable send
notify user
```

---

# 182. Workflow Journal

Durable.

---

# 183. No one giant transaction across systems.

---

# 184. Use saga.

---

# 185. Deletion Workflow

Long-running saga.

---

# 186. Example

```text
freeze account
→ revoke sessions
→ revoke capabilities
→ delete active data
→ process indexes
→ process blobs
→ process backups
→ finalize
```

---

# 187. Workflow Resume After Crash

Required.

---

# 188. Idempotent Deletion Steps

Mandatory.

---

# 189. No Double-Delete Corruption.

---

# 190. Deletion Failure

Account remains PendingDeletion.

---

# 191. No automatic reactivation.

---

# 192. Retry.

---

# 193. Manual intervention only if needed.

---

# 194. Security Lock Workflow

Faster.

---

# 195. Can revoke sessions immediately.

---

# 196. Suspend vs SecurityLock

Different state semantics.

---

# 197. Hard rule.

---

# 198. Billing Suspension

Possible for managed/paid service.

---

# 199. Must not delete data immediately.

---

# 200. Read-only grace

Product policy.

---

# 201. No Billing State = Security Compromise

Hard rule.

---

# 202. Anonymous Network Service Account

If service account used.

---

# 203. Separate lifecycle.

---

# 204. No password.

---

# 205. Dynamic credential/workload identity preferred.

---

# 206. Service account deletion

Revokes service authority.

---

# 207. No human-login semantics.

---

# 208. Operator Account Lifecycle

High-risk.

---

# 209. Creation

Multi-party approval.

---

# 210. Activation

Phishing-resistant MFA.

---

# 211. Suspension

Immediate on role departure/security concern.

---

# 212. Deletion

After audit/retention.

---

# 213. No Shared Operator Account.

---

# 214. Managed Enterprise Provisioning

OIDC/SCIM integration.

---

# 215. SCIM Create

Creates membership, not personal identity merge.

---

# 216. SCIM Suspend

Suspends managed membership.

---

# 217. SCIM Delete

Offboards membership.

---

# 218. Hard Rule

External IdP deprovision must not erase unrelated personal account.

---

# 219. Identity Mapping

Issuer/org scoped.

---

# 220. No email-based global merge.

---

# 221. Registration Abuse

Rate limiting.

---

# 222. Invite abuse.

---

# 223. Fake accounts.

---

# 224. Do not require invasive identity verification by default.

---

# 225. Anti-Sybil

Service-specific.

---

# 226. No universal real-world identity requirement.

---

# 227. Hard rule.

---

# 228. Account Naming

Part 59.

---

# 229. Public handle optional.

---

# 230. Registration does not require public handle.

---

# 231. Account can remain private.

---

# 232. No automatic discoverability.

---

# 233. Hard rule.

---

# 234. Account Visibility

```rust
pub enum AccountVisibility {
    Private,
    ExactHandle,
    Public,
}
```

---

# 235. Default

Private.

---

# 236. Visibility Change

Explicit.

---

# 237. Deactivation

Removes public discovery.

---

# 238. Suspension

Policy-specific.

---

# 239. Deletion

Removes public identity records.

---

# 240. Account Recovery

Part 57/82.

---

# 241. Recovery is not lifecycle re-creation.

---

# 242. Same AccountId may continue if account not deleted.

---

# 243. New device identity.

---

# 244. Sessions revoked.

---

# 245. Hard Rule

Recovery never reactivates Deleted account.

---

# 246. Recovery During Suspension

May restore credentials but suspension remains.

---

# 247. Authentication state and account state separate.

---

# 248. Hard rule.

---

# 249. Session Issuance Gate

```rust
pub trait SessionEligibilityPolicy {
    fn may_issue_session(
        &self,
        state: AccountState,
        account_type: AccountType,
    ) -> bool;
}
```

---

# 250. Active

Yes.

---

# 251. Suspended

Maybe limited/support/recovery only.

---

# 252. SecurityLocked

Recovery only.

---

# 253. Deactivated

Reactivation flow only.

---

# 254. PendingDeletion

Deletion management only.

---

# 255. Deleted

No.

---

# 256. Hard Rule

Session type must match state.

---

# 257. Restricted Session

Possible.

---

# 258. Example

```rust
pub enum SessionPurpose {
    Normal,
    Recovery,
    Reactivation,
    DeletionManagement,
}
```

---

# 259. No Normal Session From Restricted State.

---

# 260. Authorization Integration

Part 81.

---

# 261. Account state is ABAC input.

---

# 262. Example

Suspended → SendMessage denied.

---

# 263. But

Export own data may be allowed.

---

# 264. Lifecycle-aware policy.

---

# 265. No authorization policy can override Deleted.

---

# 266. Hard rule.

---

# 267. Device State Integration

Account deactivation may not physically wipe device.

---

# 268. Device credential revoked.

---

# 269. Local data may remain encrypted until user deletes.

---

# 270. Managed Device

Org policy may wipe managed profile.

---

# 271. Personal profile unaffected.

---

# 272. Hard rule.

---

# 273. Key Lifecycle Integration

Part 66/67.

---

# 274. Account deletion may destroy account-level wrapping keys.

---

# 275. Device/session keys already separate.

---

# 276. Group keys rotated.

---

# 277. Backup keys handled per ownership.

---

# 278. No universal key deletion.

---

# 279. Mailbox Integration

Delete/revoke mailbox locators/capabilities.

---

# 280. Anonymous presence

Stop.

---

# 281. Discovery

Remove.

---

# 282. Federation

Revoke relationship capabilities.

---

# 283. Payments/Credits

Part 47.

---

# 284. Outstanding balance handling

Separate product/legal policy.

---

# 285. No deletion blocked forever solely by zero-value internal accounting record.

---

# 286. But

Tax/accounting records may require retention.

---

# 287. Legal Hold

Part 55/67.

---

# 288. Legal Hold Does Not Keep Account Active.

---

# 289. Hard rule.

---

# 290. Held records

Scoped.

---

# 291. Account can still be deleted operationally.

---

# 292. Retained legal subset marked.

---

# 293. No hidden broad legal hold.

---

# 294. Legal Hold State

```rust
pub struct LegalRetentionConstraint {
    pub scope: DataClassScope,
    pub expires_at: Option<Timestamp>,
}
```

---

# 295. Deletion Result

Truthful.

---

# 296. Example

```text
active account deleted
some legally required records retained
```

---

# 297. No claim "everything permanently erased" if not true.

---

# 298. Hard rule.

---

# 299. Cooling-Off Period

Optional.

---

# 300. For accidental deletion protection.

---

# 301. During CoolingOff

Account inaccessible or limited.

---

# 302. Cancellation

Requires strong reauthentication.

---

# 303. No indefinite cooling-off.

---

# 304. Hard rule.

---

# 305. Sensitive Account

May have no cooling-off.

---

# 306. Operator account

Immediate disable then scheduled destruction.

---

# 307. Deletion Confirmation

Could require multiple channels for high-risk org deletion.

---

# 308. Personal account

one strong authenticated confirmation.

---

# 309. Tenant deletion

multi-party approval.

---

# 310. Account deletion vs Tenant deletion

Separate workflows.

---

# 311. Hard rule.

---

# 312. Account Merge

Avoid.

---

# 313. Why

Identity correlation/privacy risk.

---

# 314. No Automatic Merge By Email/Phone

Hard rule.

---

# 315. If merge ever supported

Explicit user-confirmed transfer with new authority graph.

---

# 316. Initial scope

Do not support.

---

# 317. Account Split

Also complex.

---

# 318. Avoid initial scope.

---

# 319. Account Migration

Portability.

---

# 320. Could transfer account authority to new provider/domain.

---

# 321. Part 57/58.

---

# 322. Migration Is Not Deletion.

---

# 323. Old provider cleanup after transfer.

---

# 324. No dual active authority indefinitely.

---

# 325. Hard rule.

---

# 326. Account State Governance

Lifecycle policy signed/versioned.

---

# 327. Lifecycle Policy

```rust
pub struct AccountLifecyclePolicy {
    pub version: PolicyVersion,
    pub account_type: AccountType,
    pub allowed_transitions: BTreeSet<StateTransition>,
    pub retention: RetentionPolicyId,
}
```

---

# 328. Anti-Rollback

Required.

---

# 329. No local operator ad-hoc state graph.

---

# 330. Hard rule.

---

# 331. Transition Authorization

Uses Part 81.

---

# 332. Example

User can deactivate own personal account.

---

# 333. Org admin can offboard managed membership.

---

# 334. Security engine can security-lock.

---

# 335. Operator cannot impersonate user to reactivate.

---

# 336. Hard rule.

---

# 337. Lifecycle Decision Point

```rust
pub trait LifecycleDecisionPoint {
    fn authorize_transition(
        &self,
        request: &LifecycleTransitionRequest,
    ) -> Result<LifecycleTransitionDecision, LifecycleError>;
}
```

---

# 338. Lifecycle Enforcement Point

Account service.

---

# 339. No direct DB update.

---

# 340. Hard rule.

---

# 341. State Version

Monotonic.

---

# 342. Account State Record

```rust
pub struct AccountStateRecord {
    pub account: AccountId,
    pub state: AccountState,
    pub version: u64,
    pub deletion_epoch: Option<u64>,
}
```

---

# 343. Optimistic Concurrency

Use version.

---

# 344. Prevent conflicting transitions.

---

# 345. Example

suspend vs delete race.

---

# 346. Transactional compare-and-set.

---

# 347. Hard rule.

---

# 348. Distributed Event Integration

Part 75.

---

# 349. State transition commits first.

---

# 350. Then outbox events.

---

# 351. Consumers idempotent.

---

# 352. No duplicate revoke/wipe side effects.

---

# 353. Lifecycle Event Consumers

```text
auth service
device service
mailbox
group service
tenant service
backup service
```

---

# 354. Minimal event payload.

---

# 355. No message content.

---

# 356. Event Example

```rust
pub struct AccountStateChanged {
    pub account_ref: OpaqueAccountRef,
    pub new_state: AccountState,
    pub state_version: u64,
}
```

---

# 357. Opaque ref.

---

# 358. Tenant-scoped where relevant.

---

# 359. No global analytics event.

---

# 360. Hard rule.

---

# 361. Account Lifecycle Observability

Safe metrics:

```text
registrations
activations
suspensions
deletions
workflow failures
```

---

# 362. Aggregate.

---

# 363. No user-level timeline.

---

# 364. Security/admin audit

Only high-risk transitions.

---

# 365. Lifecycle Audit Record

```rust
pub struct AccountLifecycleAudit {
    pub account_class: AccountType,
    pub transition: LifecycleTransitionKind,
    pub authority_class: TransitionAuthorityClass,
    pub outcome: AuditOutcome,
}
```

---

# 366. No user message data.

---

# 367. No precise login history.

---

# 368. Lifecycle SLOs

Examples:

```text
session revocation after suspension
deletion workflow completion
offboarding completion
anti-resurrection validation
```

---

# 369. Security SLO

```text
0 deleted account reactivation
0 suspended account normal session issuance
```

---

# 370. Privacy SLO

```text
0 deleted account discoverability after deletion finalization
```

---

# 371. Failure Modes

```text
partial offboarding
deletion workflow crash
backup restoration
stale sync client
legal-hold mismatch
```

---

# 372. Partial Offboarding

Workflow resumes.

---

# 373. Account remains restricted.

---

# 374. Deletion Crash

Resume from journal.

---

# 375. Backup Restore

Deletion barrier prevents resurrection.

---

# 376. Stale Client

Sync rejects pre-deletion authority.

---

# 377. Legal Hold Mismatch

Deletion planner fails safely.

---

# 378. No Silent Over-Deletion Or Under-Deletion

Hard rule.

---

# 379. Disaster Recovery

Part 70.

---

# 380. Restore account state + deletion epochs.

---

# 381. Deletion epoch is critical durable state.

---

# 382. Hard rule.

---

# 383. Backup Without Deletion Ledger

Insufficient.

---

# 384. Account service DR test

Required.

---

# 385. Migration

Schema evolution Part 74.

---

# 386. State enum expansion

Backward-compatible.

---

# 387. Old binary encountering unknown state

Fail closed/read-only.

---

# 388. Hard rule.

---

# 389. Event Schema

Versioned.

---

# 390. No semantic reuse of state.

---

# 391. Testing

Need lifecycle testkit.

---

# 392. Test Scenarios

```text
registration
abandoned enrollment
suspension
security lock
deactivation
deletion
```

---

# 393. Registration Test

No active session before enrollment complete.

---

# 394. Invite Replay Test

Single-use invite rejected second time.

---

# 395. Suspension Test

Normal session/write denied.

---

# 396. Security Lock Test

Existing sessions revoked.

---

# 397. Deactivation Test

Reactivation requires strong authentication.

---

# 398. Deletion Test

No reactivation after Completed.

---

# 399. Deletion Barrier Test

Old backup restore cannot resurrect.

---

# 400. Stale Sync Test

Pre-deletion offline write rejected.

---

# 401. Managed Offboarding Test

Org authority removed, personal identity preserved.

---

# 402. Federation Unlink Test

Remote relationship revoked only.

---

# 403. Legal Hold Test

Held subset retained while account deleted.

---

# 404. Recovery Test

Suspension persists through credential recovery.

---

# 405. Tenant Test

Tenant A offboarding does not affect B.

---

# 406. Race Test

Suspend/delete concurrent transition resolves safely.

---

# 407. Idempotency Test

Repeated deletion command does not duplicate side effects.

---

# 408. Privacy Test

Deleted account removed from discoverability.

---

# 409. Fuzzing

Fuzz:

```text
lifecycle command
state transition
deletion plan
membership offboarding event
```

---

# 410. Property Tests

Properties:

```text
Deleted is terminal
PendingDeletion cannot issue normal session
deletion_epoch never decreases
org offboarding cannot modify personal account state
```

---

# 411. Formal Verification Targets

Strong candidates:

```text
account state machine
deletion saga
membership/account separation
anti-resurrection
```

---

# 412. Kani Candidate

transition legality/state terminality.

---

# 413. TLA+ Candidate

deletion workflow + backup restore + stale client.

---

# 414. Loom Candidate

concurrent suspend/deactivate/delete.

---

# 415. Performance

Lifecycle is control-plane, not hot path.

---

# 416. State lookup

Fast local/cache.

---

# 417. Session issuance checks state.

---

# 418. Authorization can cache state version.

---

# 419. Revocation/state change invalidates.

---

# 420. No central lifecycle lookup per message.

---

# 421. Account State Cache

Short-lived.

---

# 422. Versioned.

---

# 423. Deleted/SecurityLocked states pushed rapidly.

---

# 424. No stale allow beyond policy.

---

# 425. Crate Layout

Recommended:

```text
crates/
├── siar-account-lifecycle-core/
├── siar-registration/
├── siar-enrollment/
├── siar-account-state/
├── siar-membership-lifecycle/
├── siar-account-suspension/
├── siar-account-deletion/
├── siar-deletion-barrier/
├── siar-lifecycle-policy/
├── siar-lifecycle-observability/
└── siar-lifecycle-testkit/
```

---

# 426. `siar-account-lifecycle-core`

Owns:

```text
account states
transition types
errors
```

---

# 427. `siar-registration`

Registration/invite state.

---

# 428. `siar-enrollment`

Credential/recovery/device/policy enrollment.

---

# 429. `siar-account-state`

Durable lifecycle repository.

---

# 430. `siar-membership-lifecycle`

Tenant/org offboarding.

---

# 431. `siar-account-suspension`

Suspension/security-lock profiles.

---

# 432. `siar-account-deletion`

Deletion workflow/planner.

---

# 433. `siar-deletion-barrier`

Deletion epoch/anti-resurrection.

---

# 434. `siar-lifecycle-policy`

Signed lifecycle transition policy.

---

# 435. `siar-lifecycle-observability`

Privacy-safe metrics/audit.

---

# 436. `siar-lifecycle-testkit`

Race/recovery/backup/offboarding simulation.

---

# 437. Error Taxonomy

```rust
pub enum LifecycleError {
    InvalidTransition,
    UnauthorizedTransition,
    EnrollmentIncomplete,
    AccountSuspended,
    AccountSecurityLocked,
    AccountDeactivated,
    AccountPendingDeletion,
    AccountDeleted,
    LegalRetentionConflict,
    DeletionBarrierViolation,
    ConcurrentTransition,
    Internal,
}
```

---

# 438. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. Account lifecycle is a typed state machine; direct arbitrary state mutation is forbidden.
2. Account, authentication, authorization, device, membership, federation, and retention states remain separate.
3. Active account state never implies universal authorization.
4. Suspension, security lock, deactivation, and deletion have distinct semantics and side effects.
5. Managed organization offboarding revokes only managed authority and cannot erase or control unrelated personal identity/state.
6. Deleted is terminal for the AccountId; a later registration always receives a new AccountId.
7. PendingDeletion and Deleted accounts cannot receive normal sessions or new user-data writes.
8. Deletion epochs/tombstones prevent backups, stale replicas, or offline clients from resurrecting deleted identity/state.
9. Account deletion revokes sessions, capabilities, discovery entries, and relevant mailbox/device authority before finalization.
10. Legal retention constrains only scoped records and never keeps the deleted account operationally active.
11. Lifecycle events and audits contain minimal state metadata and never become a user-behavior or message-content timeline.
12. Recovery may restore authority for non-deleted accounts but never restores old device/session/transport secrets or resurrects a Deleted account.
```

---

# 439. Initial Production Scope

Implement first:

```text
typed AccountState state machine
open/invite/managed/local-only registration modes
single-use invites
enrollment completion gate
suspension profiles
security-lock state
deactivation/reactivation
organization/tenant membership lifecycle separation
durable deletion saga
deletion epoch/barrier
session/capability/device revocation hooks
mailbox/discovery cleanup
backup anti-resurrection integration
legal-retention scoped constraints
privacy-safe lifecycle metrics
lifecycle testkit
```

Then add:

```text
advanced portability/migration lifecycle
managed SCIM lifecycle adapters
formal deletion-saga verification
provider-to-provider account authority transfer
policy explainability for lifecycle decisions
```

---

# 440. Definition of Done

Part 83 is complete when:

- account lifecycle is a strict typed state machine
- registration/enrollment are distinct
- invites and pending accounts expire safely
- suspension/security-lock/deactivation semantics differ
- managed membership lifecycle is separate from personal account state
- deletion is long-running, journaled, idempotent, and anti-resurrection protected
- deleted AccountId is terminal/non-reusable
- sessions/capabilities/devices/mailboxes/discovery are revoked appropriately
- legal holds retain only scoped records
- recovery cannot resurrect deleted account or old transport/session secrets
- stale backups/sync clients cannot reintroduce deleted state
- lifecycle audit/metrics remain privacy-safe
- race/offboarding/deletion/restore/fuzz/formal tests are specified

---

# 441. Final Architecture

```text
                    REGISTRATION / INVITE
                            │
                            ▼
                       ENROLLMENT
                            │
                            ▼
                          ACTIVE
                  ┌─────────┼─────────┐
                  │         │         │
              SUSPENDED  LOCKED   DEACTIVATED
                  │         │         │
                  └─────────┼─────────┘
                            ▼
                    PENDING DELETION
                            │
                            ▼
                         DELETED
```

Lifecycle safety model:

```text
typed states
+
authorized transitions
+
durable side-effect workflows
+
session/capability revocation
+
deletion barriers
+
ownership-aware deletion
+
anti-resurrection
```

not:

```text
toggle an "active" boolean and hope every subsystem behaves correctly
```

---

# 442. Final Principle

Identity lifecycle should be governed as durable authority state, not as incidental account metadata.

The correct model is:

```text
register minimally
+
enroll explicitly
+
activate intentionally
+
restrict with precise states
+
offboard by scope
+
delete through durable workflow
+
block resurrection forever
```

This architecture gives SIAR a rigorous account and identity-state lifecycle for personal users, managed organizations, operators, tenants, federation relationships, devices, sessions, backups, and deletion workflows while preserving the local-first, anonymous, privacy-preserving, and least-authority guarantees established across Parts 34–82.
