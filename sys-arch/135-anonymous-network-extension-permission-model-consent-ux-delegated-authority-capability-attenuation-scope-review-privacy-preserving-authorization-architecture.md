# Core System Architecture Part 135 — Anonymous Network Extension Permission Model, Consent UX, Delegated Authority, Capability Attenuation, Scope Review & Privacy-Preserving Authorization Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 135  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 45, 56, 68, 69, 81–85, 130–134

**Primary purpose:** define SIAR's extension authorization architecture for permission classes, capability requests, user/admin consent UX, delegated authority, capability attenuation, scope review, one-shot grants, time-bounded permissions, update re-consent, revocation propagation, tenant policy overlays, anonymous-mode restrictions, and privacy-preserving permission governance.

---

# 1. Purpose

Installing an extension should never mean:

```text
"grant everything this package might ever want"
```

Extension authorization must answer:

```text
What does this extension want?
Why does it want it?
Who is allowed to approve it?
For how long?
For which conversations, files, tenants, devices, or APIs?
Can authority be narrowed?
What happens after an update adds new scopes?
Can authority be revoked immediately?
Can managed policy reduce grants further?
```

The governing principle is:

> **Extension authority must be explicit, scoped, reviewable, attenuable, revocable, and visible to the user or administrator, with no ambient privilege and no silent scope expansion.**

---

# 2. Architectural Position

```text
               EXTENSION MANIFEST
                      │
                      ▼
               REQUESTED SCOPES
                      │
            ┌─────────┼─────────┐
            │         │         │
         USER       ADMIN      POLICY
            │         │         │
            └─────────┼─────────┘
                      ▼
                 GRANT DECISION
                      │
                      ▼
            ATTENUATED CAPABILITY
                      │
                      ▼
                RUNTIME BROKER
                      │
                      ▼
              USE / REVIEW / REVOKE
```

---

# 3. Core Separation

Keep distinct:

```text
requested permission
approved permission
effective permission
delegated permission
runtime capability token
platform OS permission
user consent
tenant policy
```

---

# 4. Non-Goals

Part 135 does not create:

```text
all-or-nothing install consent
permanent hidden permissions
ambient admin authority
user-consent screens that obscure scope
extension-controlled permission text
```

---

# 5. Permission Identity

```rust
pub struct ExtensionPermissionId(pub [u8; 16]);
```

---

# 6. Permission Class

```rust
pub enum ExtensionPermissionClass {
    ConversationRead,
    ConversationWrite,
    MessageContentRead,
    MessageSend,
    AttachmentRead,
    AttachmentWrite,
    PresenceRead,
    ContactLookup,
    FileRead,
    FileWrite,
    NetworkExternal,
    NetworkLocal,
    NotificationPost,
    ClipboardRead,
    ClipboardWrite,
    CameraUse,
    MicrophoneUse,
    DeviceManagement,
    DiagnosticsRead,
    TenantAdministration,
}
```

---

# 7. Fine-Grained By Default

Hard rule.

---

# 8. No `AllAccess`

Hard rule.

---

# 9. Requested Permission

```rust
pub struct RequestedExtensionPermission {
    pub permission: ExtensionPermissionClass,
    pub requested_scope: ExtensionScopeRequest,
    pub rationale: PermissionRationale,
}
```

---

# 10. Rationale

Human-readable purpose text.

---

# 11. Publisher Cannot Hide Rationale

Hard rule.

---

# 12. Rationale Is Not Authority

Hard rule.

---

# 13. Permission Scope

```rust
pub enum ExtensionScopeRequest {
    AnyAuthorizedConversation,
    Conversation(SdkConversationId),
    CurrentConversation,
    SelectedFiles,
    ExtensionStorageOnly,
    SpecificOrigins(BTreeSet<NetworkOrigin>),
    Tenant(TenantId),
    LocalDevice,
}
```

---

# 14. Scope Must Match Permission Type

Hard rule.

---

# 15. Effective Grant

```rust
pub struct ExtensionPermissionGrant {
    pub grant_id: ExtensionPermissionGrantId,
    pub extension: ExtensionId,
    pub permission: ExtensionPermissionClass,
    pub scope: EffectiveExtensionScope,
    pub lifecycle: GrantLifecycle,
    pub authority: GrantAuthority,
}
```

---

# 16. Effective Scope

```rust
pub enum EffectiveExtensionScope {
    Conversation(SdkConversationId),
    ConversationSet(BTreeSet<SdkConversationId>),
    FileSet(BTreeSet<BrokeredFileHandle>),
    NetworkOrigins(BTreeSet<NetworkOrigin>),
    Tenant(TenantId),
    LocalDevice,
    ExtensionStorage,
}
```

---

# 17. No Broader Scope Than Requested

Hard rule.

---

# 18. Attenuation

Approved scope may be narrower than requested scope.

---

# 19. Hard rule.

---

# 20. Grant Lifecycle

```rust
pub enum GrantLifecycle {
    OneShot,
    Session,
    UntilRestart,
    Until(Timestamp),
    Persistent,
}
```

---

# 21. Sensitive Permissions Prefer Shorter Lifetimes

Hard rule.

---

# 22. One-Shot Permission

Examples:

```text
select one file
send one attachment
post one notification
```

---

# 23. Hard rule.

---

# 24. Persistent Permission

Requires explicit user/admin approval.

---

# 25. Hard rule.

---

# 26. Permission Sensitivity

```rust
pub enum PermissionSensitivity {
    Low,
    Moderate,
    High,
    Critical,
}
```

---

# 27. Sensitivity Policy

Examples:

```text
NotificationPost → Low
ConversationRead → Moderate
MessageContentRead → High
MicrophoneUse → High
TenantAdministration → Critical
```

---

# 28. Hard rule.

---

# 29. Permission Approval Authority

```rust
pub enum GrantAuthority {
    User,
    TenantAdmin,
    PlatformPolicy,
    DevicePolicy,
}
```

---

# 30. No Extension Self-Approval

Hard rule.

---

# 31. User Consent

Applies to user-owned resources/capabilities.

---

# 32. Tenant Admin Consent

Applies to managed/organizational resources.

---

# 33. Platform Policy

Can restrict, never broaden.

---

# 34. Device Policy

Can further restrict device access.

---

# 35. Hard rule.

---

# 36. Effective Authorization

```text
requested
∩ user/admin approval
∩ tenant policy
∩ platform policy
∩ privacy mode
∩ runtime state
```

---

# 37. Hard rule.

---

# 38. Deny Dominates

If any applicable authority denies, final result denies.

---

# 39. Hard rule.

---

# 40. Permission Decision

```rust
pub enum ExtensionPermissionDecision {
    Allow,
    AllowAttenuated,
    Deny,
    AskUser,
    AskAdmin,
}
```

---

# 41. No Implicit Allow On Unknown

Hard rule.

---

# 42. Consent UX

The consent screen must answer:

```text
what data/resource
what action
what scope
why
for how long
which privacy modes affected
```

---

# 43. Hard rule.

---

# 44. Human-Readable Permission Description

Generated by platform, not package alone.

---

# 45. Hard rule.

---

# 46. Example

Bad:

```text
"Needs account access"
```

Good:

```text
"Read message content in the conversations you select"
```

---

# 47. Hard rule.

---

# 48. Permission Grouping

Can group related low-risk permissions.

---

# 49. High-risk permissions always individually visible.

---

# 50. Hard rule.

---

# 51. No Dark Pattern Consent

Prohibited:

```text
preselected high-risk scopes
misleading colors
buried deny action
false urgency
```

---

# 52. Hard rule.

---

# 53. Consent Receipt

```rust
pub struct ExtensionConsentReceipt {
    pub extension: ExtensionId,
    pub grants: Vec<ExtensionPermissionGrantId>,
    pub approved_at: Timestamp,
    pub context_digest: Digest,
}
```

---

# 54. Hard rule.

---

# 55. Consent Context

Should include:

```text
package version
permission set
privacy mode
policy version
```

---

# 56. Hard rule.

---

# 57. Re-Consent Trigger

Needed when:

```text
new permission class
scope broadens
lifecycle becomes longer
data class expands
network origins expand
privacy-mode behavior changes
```

---

# 58. Hard rule.

---

# 59. No Re-Consent For Pure Scope Reduction

Hard rule.

---

# 60. Permission Diff

```rust
pub struct PermissionDiff {
    pub added: Vec<RequestedExtensionPermission>,
    pub removed: Vec<RequestedExtensionPermission>,
    pub broadened: Vec<PermissionScopeChange>,
    pub narrowed: Vec<PermissionScopeChange>,
}
```

---

# 61. Hard rule.

---

# 62. Update Review

Before activating extension update, platform computes permission diff.

---

# 63. Hard rule.

---

# 64. Scope Expansion Update

Cannot auto-activate.

---

# 65. Hard rule.

---

# 66. Capability Attenuation

A broad approved capability can be attenuated to narrower downstream capability.

---

# 67. Example

```text
Read selected conversations
→ Read conversation A only
→ Read message metadata only
```

---

# 68. Hard rule.

---

# 69. Delegated Capability

```rust
pub struct DelegatedExtensionCapability {
    pub parent_grant: ExtensionPermissionGrantId,
    pub child_scope: EffectiveExtensionScope,
    pub child_lifecycle: GrantLifecycle,
    pub audience: CapabilityAudience,
}
```

---

# 70. Hard rule.

---

# 71. Delegation Invariants

Child capability must be:

```text
same or narrower permission
same or narrower scope
same or shorter lifetime
same or narrower audience
```

---

# 72. Hard rule.

---

# 73. Capability Audience

```rust
pub enum CapabilityAudience {
    Runtime(ExtensionRuntimeId),
    BackgroundJob(ExtensionBackgroundJobId),
    Broker(BrokerId),
}
```

---

# 74. Hard rule.

---

# 75. No Delegation To Another Extension By Default

Hard rule.

---

# 76. Cross-Extension Delegation

If ever allowed, requires explicit brokered contract.

---

# 77. Hard rule.

---

# 78. Delegation Depth

Bounded.

---

# 79. Hard rule.

---

# 80. Capability Chain

```rust
pub struct CapabilityDelegationChain {
    pub root: ExtensionPermissionGrantId,
    pub links: Vec<DelegatedExtensionCapability>,
}
```

---

# 81. Hard rule.

---

# 82. Revocation Propagation

Revoking parent invalidates all descendants.

---

# 83. Hard rule.

---

# 84. Grant Epoch Integration

Part 134.

---

# 85. Hard rule.

---

# 86. Consent vs Authorization

Consent does not replace authorization.

---

# 87. Authorization does not imply consent where consent required.

---

# 88. Hard rule.

---

# 89. OS Permission Integration

Camera/mic/file/clipboard may also require OS/platform permission.

---

# 90. Triple Gate

```text
extension grant
∩ platform permission
∩ user/admin approval
```

---

# 91. Hard rule.

---

# 92. File Picker Permission

Prefer user-selected brokered handles.

---

# 93. Hard rule.

---

# 94. Folder Access

Separate broader permission.

---

# 95. Hard rule.

---

# 96. Clipboard Read

High sensitivity.

---

# 97. Hard rule.

---

# 98. Clipboard Write

Separate permission.

---

# 99. Hard rule.

---

# 100. Camera/Microphone

Session-scoped by default unless policy explicitly allows persistence.

---

# 101. Hard rule.

---

# 102. Network Permission

External networking requires origin-scoped approval.

---

# 103. Hard rule.

---

# 104. Wildcard Origin

High sensitivity.

---

# 105. Hard rule.

---

# 106. Local Network Access

Separate permission due discovery/privacy impact.

---

# 107. Hard rule.

---

# 108. Anonymous Network Access

Separate capability.

---

# 109. Hard rule.

---

# 110. Extension Cannot Choose Less Private Route

Hard rule.

---

# 111. Privacy Mode Overlay

```rust
pub enum ExtensionPrivacyModePolicy {
    Standard,
    Private,
    Anonymous,
    MaximumAnonymity,
}
```

---

# 112. Each mode may reduce effective permissions.

---

# 113. Hard rule.

---

# 114. Maximum Anonymity Restrictions

Potentially deny:

```text
PresenceRead
ContactLookup
NetworkLocal
DiagnosticsRead
detailed device metadata
```

---

# 115. Hard rule.

---

# 116. Privacy Mode Change

If user raises privacy level, effective permission set may shrink immediately.

---

# 117. Hard rule.

---

# 118. Lowering Privacy Mode

Does not automatically restore previously withheld high-risk permission.

---

# 119. Hard rule.

---

# 120. Managed Tenant Policy

Tenant admin may define:

```text
allowed extension classes
allowed publishers
maximum permission sensitivity
disallowed data classes
network restrictions
```

---

# 121. Hard rule.

---

# 122. Tenant Policy Overlay

```rust
pub struct TenantExtensionPermissionPolicy {
    pub tenant: TenantId,
    pub denied_permissions: BTreeSet<ExtensionPermissionClass>,
    pub restricted_origins: BTreeSet<NetworkOrigin>,
    pub max_persistent_sensitivity: PermissionSensitivity,
}
```

---

# 123. Hard rule.

---

# 124. Tenant Policy Cannot Broaden User Capability

Hard rule.

---

# 125. Personal vs Managed Context

Keep separate.

---

# 126. Hard rule.

---

# 127. Device Policy

Example:

```text
camera disabled
clipboard disabled
local network disabled
```

---

# 128. Hard rule.

---

# 129. Device Policy Cannot Be Overridden By Extension Consent

Hard rule.

---

# 130. Permission Request Timing

Ask just in time where practical.

---

# 131. Hard rule.

---

# 132. Install-Time vs Runtime Consent

Install-time:

```text
broad architecture-level permissions
```

Runtime:

```text
specific file
specific conversation
specific mic session
```

---

# 133. Hard rule.

---

# 134. Avoid Permission Fatigue

Do not ask repeatedly for identical denied scopes without meaningful reason/state change.

---

# 135. Hard rule.

---

# 136. Remembered Denial

Scoped locally.

---

# 137. Hard rule.

---

# 138. No Behavioral Pressure

Do not nag based on inferred likelihood to accept.

---

# 139. Hard rule.

---

# 140. Permission Review Center

User/admin can inspect:

```text
extensions
active grants
scope
lifetime
last-used coarse state
privacy-mode restrictions
```

---

# 141. Hard rule.

---

# 142. Last Used

If shown, coarse and local where possible.

---

# 143. Hard rule.

---

# 144. No Detailed Permission Usage Timeline By Default

Hard rule.

---

# 145. Permission Revocation

Can revoke:

```text
one scope
one permission class
all runtime grants
entire extension
```

---

# 146. Hard rule.

---

# 147. Revocation State

```rust
pub enum ExtensionGrantState {
    Active,
    Suspended,
    Revoked,
    Expired,
}
```

---

# 148. Hard rule.

---

# 149. Immediate Runtime Effect

Part 134 grant epoch update.

---

# 150. Hard rule.

---

# 151. In-Flight Requests

Must revalidate before side-effect commit.

---

# 152. Hard rule.

---

# 153. Permission Expiry

Time-bound grant automatically expires.

---

# 154. Hard rule.

---

# 155. Expiry Notification

May notify extension/user if useful.

---

# 156. Hard rule.

---

# 157. No Auto-Renewal Of High-Risk Permission

Hard rule.

---

# 158. Renewal

Requires explicit policy/consent.

---

# 159. Hard rule.

---

# 160. Background Job Permissions

Background jobs inherit attenuated subset only.

---

# 161. Hard rule.

---

# 162. Example

```text
interactive extension: ConversationRead + MessageSend
background job: ConversationRead(metadata only)
```

---

# 163. Hard rule.

---

# 164. Job Audience Binding

Capability token tied to job ID.

---

# 165. Hard rule.

---

# 166. Offline Permissions

Local grants may remain usable offline for local resources.

---

# 167. Hard rule.

---

# 168. Remote Policy Staleness

Never broadens access.

---

# 169. Hard rule.

---

# 170. Offline Revocation

Locally known revocation applies immediately.

---

# 171. Hard rule.

---

# 172. Policy Reconciliation

On reconnect:

```text
fetch signed policy
compare epochs
apply restrictive changes
invalidate stale grants
```

---

# 173. Hard rule.

---

# 174. Permission Registry

```rust
pub struct ExtensionPermissionRegistry {
    pub version: ExtensionPermissionRegistryVersion,
    pub permissions: BTreeMap<ExtensionPermissionClass, PermissionDefinition>,
}
```

---

# 175. Hard rule.

---

# 176. Permission Definition

```rust
pub struct PermissionDefinition {
    pub sensitivity: PermissionSensitivity,
    pub allowed_scopes: BTreeSet<PermissionScopeKind>,
    pub default_lifecycle: GrantLifecycle,
    pub consent_required: bool,
}
```

---

# 177. Hard rule.

---

# 178. Signed / Versioned

Hard rule.

---

# 179. Anti-Rollback

Permission policy epoch monotonic.

---

# 180. Hard rule.

---

# 181. Permission Policy Epoch

```rust
pub struct ExtensionPermissionPolicyEpoch(pub u64);
```

---

# 182. Hard rule.

---

# 183. Policy Rollback

Cannot re-enable removed permissions.

---

# 184. Hard rule.

---

# 185. Permission Description Registry

Platform-owned canonical user-facing descriptions.

---

# 186. Hard rule.

---

# 187. Localization

Descriptions translatable.

---

# 188. Hard rule.

---

# 189. Accessibility

Consent UX accessible.

---

# 190. Hard rule.

---

# 191. Permission Explanation

Can include:

```text
requested
effective
denied
reason
policy source
```

---

# 192. Hard rule.

---

# 193. Explainability Service

```rust
pub trait ExtensionPermissionExplainService {
    fn explain(
        &self,
        extension: ExtensionId,
        permission: ExtensionPermissionClass,
    ) -> Result<PermissionExplanation, ExtensionAuthorizationError>;
}
```

---

# 194. Hard rule.

---

# 195. Permission Decision Service

```rust
pub trait ExtensionAuthorizationService {
    fn evaluate(
        &self,
        request: ExtensionAuthorizationRequest,
    ) -> Result<ExtensionAuthorizationDecision, ExtensionAuthorizationError>;
}
```

---

# 196. Request

```rust
pub struct ExtensionAuthorizationRequest {
    pub extension: ExtensionId,
    pub runtime: ExtensionRuntimeId,
    pub permission: ExtensionPermissionClass,
    pub requested_scope: ExtensionScopeRequest,
    pub privacy_mode: ExtensionPrivacyModePolicy,
}
```

---

# 197. Hard rule.

---

# 198. Grant Service

```rust
pub trait ExtensionGrantService {
    fn create(
        &self,
        grant: ExtensionPermissionGrant,
    ) -> Result<ExtensionPermissionGrantId, ExtensionAuthorizationError>;

    fn revoke(
        &self,
        grant: ExtensionPermissionGrantId,
    ) -> Result<(), ExtensionAuthorizationError>;
}
```

---

# 199. Hard rule.

---

# 200. Delegation Service

```rust
pub trait ExtensionDelegationService {
    fn attenuate(
        &self,
        parent: ExtensionPermissionGrantId,
        scope: EffectiveExtensionScope,
        lifecycle: GrantLifecycle,
        audience: CapabilityAudience,
    ) -> Result<DelegatedExtensionCapability, ExtensionAuthorizationError>;
}
```

---

# 201. Hard rule.

---

# 202. Consent Service

```rust
pub trait ExtensionConsentService {
    fn request_consent(
        &self,
        request: ConsentRequest,
    ) -> Result<ConsentDecision, ExtensionAuthorizationError>;
}
```

---

# 203. No Headless Auto-Consent For Sensitive User Permissions

Hard rule.

---

# 204. Managed Headless Context

Admin policy may pre-authorize only within explicitly managed scopes.

---

# 205. Hard rule.

---

# 206. Consent Request

```rust
pub struct ConsentRequest {
    pub extension: ExtensionId,
    pub requested: Vec<RequestedExtensionPermission>,
    pub package: ExtensionPackageId,
    pub privacy_mode: ExtensionPrivacyModePolicy,
}
```

---

# 207. Hard rule.

---

# 208. Consent Decision

```rust
pub enum ConsentDecision {
    Approved(Vec<ExtensionPermissionGrantId>),
    ApprovedAttenuated(Vec<ExtensionPermissionGrantId>),
    Denied,
}
```

---

# 209. Hard rule.

---

# 210. Permission Scope Review

Scope changes require diff-aware review.

---

# 211. Hard rule.

---

# 212. Scope Review Result

```rust
pub enum ScopeReviewResult {
    NoChange,
    Narrowed,
    ExpandedRequiresConsent,
    ExpandedForbiddenByPolicy,
}
```

---

# 213. Hard rule.

---

# 214. Install-Time Review

Shows full requested permission set.

---

# 215. Hard rule.

---

# 216. Update-Time Review

Shows only meaningful diffs + resulting full effective set.

---

# 217. Hard rule.

---

# 218. Runtime Just-In-Time Prompt

Possible for:

```text
file picker
camera session
microphone session
conversation access
```

---

# 219. Hard rule.

---

# 220. Consent Prompt Rate Limit

Avoid abuse/nagging.

---

# 221. Hard rule.

---

# 222. Extension Prompt Abuse

Repeated spam can trigger temporary deny/quarantine.

---

# 223. Hard rule.

---

# 224. Privilege Escalation Attempts

Examples:

```text
requesting wildcard after repeated denials
probing unauthorized scope IDs
using multiple permissions to reconstruct restricted data
```

---

# 225. Hard rule.

---

# 226. Authorization Abuse Detection

Based on technical authorization violations, not user content.

---

# 227. Hard rule.

---

# 228. Capability Composition

Two individually allowed permissions may combine into higher privacy risk.

---

# 229. Hard rule.

---

# 230. Sensitive Combination Policy

Examples:

```text
ContactLookup + NetworkExternal
MessageContentRead + NetworkExternal
ClipboardRead + NetworkExternal
```

may require stronger consent/review.

---

# 231. Hard rule.

---

# 232. Permission Combination Rule

```rust
pub struct PermissionCombinationRule {
    pub permissions: BTreeSet<ExtensionPermissionClass>,
    pub minimum_review: PermissionCombinationReview,
}
```

---

# 233. Review Level

```rust
pub enum PermissionCombinationReview {
    Normal,
    EnhancedConsent,
    AdminReview,
    CertificationRequired,
}
```

---

# 234. Hard rule.

---

# 235. Exfiltration Boundary

High-risk source + external network sink may require explicit declaration.

---

# 236. Hard rule.

---

# 237. Data Flow Grant

```rust
pub struct ExtensionDataFlowGrant {
    pub source_permission: ExtensionPermissionClass,
    pub sink_permission: ExtensionPermissionClass,
    pub approved: bool,
}
```

---

# 238. Hard rule.

---

# 239. Information Flow Mediation

Broker may tag data class.

---

# 240. Hard rule.

---

# 241. Data Classification

Examples:

```text
Public
Metadata
PrivateContent
Secret
```

---

# 242. Hard rule.

---

# 243. Secret Data

Never exportable through extension data flow without specialized broker.

---

# 244. Hard rule.

---

# 245. Clipboard As Source/Sink

Treat explicitly.

---

# 246. Hard rule.

---

# 247. File Content As Source

High-risk if network sink exists.

---

# 248. Hard rule.

---

# 249. Network Response As Input

Untrusted.

---

# 250. Hard rule.

---

# 251. Audit Receipt

Record high-value permission changes.

---

# 252. Hard rule.

---

# 253. Audit Events

Examples:

```text
high-risk grant created
admin scope approved
scope expanded
grant revoked
policy blocked grant
```

---

# 254. Hard rule.

---

# 255. No Audit Of Every Benign Broker Call

Hard rule.

---

# 256. Privacy-Preserving Audit

No private content.

---

# 257. Hard rule.

---

# 258. Permission Telemetry

Safe aggregate:

```text
permission class
allow/deny
extension class
policy source
```

---

# 259. No user identity.

---

# 260. Hard rule.

---

# 261. Consent Analytics

Do not optimize consent UX to manipulate acceptance rates.

---

# 262. Hard rule.

---

# 263. No "Conversion Rate" For High-Risk Permissions

Hard rule.

---

# 264. Product Experimentation

Part 93.

Never experiment with weaker disclosure for permission consent.

---

# 265. Hard rule.

---

# 266. User Review Center Data

Stored local-first where possible.

---

# 267. Hard rule.

---

# 268. Multi-Device Grants

Default: device-local grant unless explicitly account/tenant scoped.

---

# 269. Hard rule.

---

# 270. Cross-Device Permission Sync

If supported, sync signed grant intent, not raw capability token.

---

# 271. Hard rule.

---

# 272. New Device

Does not blindly inherit device-sensitive grants.

---

# 273. Hard rule.

---

# 274. Device-Sensitive Grants

Examples:

```text
camera
microphone
local files
clipboard
local network
```

must be re-approved per device where appropriate.

---

# 275. Hard rule.

---

# 276. Account Recovery

Part 57.

Recovered account does not restore live extension capability tokens.

---

# 277. Hard rule.

---

# 278. Fresh Device Security State

Required.

---

# 279. Hard rule.

---

# 280. Backup/Restore

Permission metadata may be restored as history, but active device-sensitive grants should revalidate.

---

# 281. Hard rule.

---

# 282. Extension Uninstall

Revokes all active grants.

---

# 283. Hard rule.

---

# 284. Extension Reinstall

Does not automatically restore high-risk permissions.

---

# 285. Hard rule.

---

# 286. Package Re-Signing

Does not imply permission continuity unless same trusted extension identity and policy allows.

---

# 287. Hard rule.

---

# 288. Publisher Transfer

Requires re-review of sensitive persistent grants.

---

# 289. Hard rule.

---

# 290. Extension Identity Transfer

High risk.

---

# 291. Hard rule.

---

# 292. Ownership Change

Potentially trigger user/admin notification.

---

# 293. Hard rule.

---

# 294. Permission Compatibility

Part 129.

Permission registry version independent from SDK version.

---

# 295. Hard rule.

---

# 296. Unknown Permission Code

Deny.

---

# 297. Hard rule.

---

# 298. Deprecated Permission

May map to narrower replacements.

---

# 299. Hard rule.

---

# 300. Removed Permission

Cannot be requested by new package.

---

# 301. Hard rule.

---

# 302. Permission Version

```rust
pub struct ExtensionPermissionSchemaVersion(pub u32);
```

---

# 303. Hard rule.

---

# 304. Policy Compilation

Human-readable RON → typed runtime policy.

---

# 305. Hard rule.

---

# 306. Example RON Policy

```ron
(
    permission: "MessageContentRead",
    sensitivity: "High",
    allowed_scopes: ["Conversation", "ConversationSet"],
    default_lifecycle: "Session",
    consent_required: true,
)
```

---

# 307. Hard rule.

---

# 308. Policy Precedence

```text
hard platform invariant
> privacy mode
> device policy
> tenant policy
> user/admin grant
> extension request
```

---

# 309. Hard rule.

---

# 310. No Lower Layer Can Broaden Higher Restriction

Hard rule.

---

# 311. Permission Decision Trace

Useful for debugging:

```text
requested
allowed by manifest
allowed by certification
allowed by tenant
allowed by privacy mode
approved by user/admin
effective scope
```

---

# 312. Hard rule.

---

# 313. No Secret Policy Detail Leakage To Extension

Extension sees coarse denial reason where necessary.

---

# 314. Hard rule.

---

# 315. User/Admin Explanation Can Be Richer

Hard rule.

---

# 316. Extension-Facing Error

```rust
pub enum ExtensionAuthorizationDenial {
    PermissionNotDeclared,
    ScopeNotGranted,
    ConsentRequired,
    PolicyDenied,
    PrivacyModeDenied,
    GrantExpired,
    GrantRevoked,
}
```

---

# 317. Hard rule.

---

# 318. No Enumeration Through Error Detail

Hard rule.

---

# 319. Authorization Hot Path

Must be fast.

---

# 320. Use:

```text
compiled policy
grant epoch
local cache
typed lookup
```

---

# 321. Hard rule.

---

# 322. No Remote Authorization Lookup In Every Broker Call

Hard rule.

---

# 323. Cached Grant State

Epoch-bound.

---

# 324. Hard rule.

---

# 325. Revocation Push

Update local cache immediately.

---

# 326. Hard rule.

---

# 327. Authorization Store

Separate:

```text
grant metadata
consent receipts
delegation chains
policy epochs
scope review records
revocation records
```

---

# 328. Hard rule.

---

# 329. Secrets Not Stored Here

Hard rule.

---

# 330. Partitioning

By:

```text
extension
tenant
device
permission class
```

---

# 331. No user-behavior analytics partition.

---

# 332. Hard rule.

---

# 333. Permission Review UX States

```rust
pub enum PermissionReviewUiState {
    Summary,
    Details,
    ScopeSelection,
    DurationSelection,
    Confirm,
    Completed,
}
```

---

# 334. Hard rule.

---

# 335. Scope Selection UX

Examples:

```text
this conversation
selected conversations
all conversations allowed by tenant policy
selected files
one session
```

---

# 336. Hard rule.

---

# 337. Duration Selection UX

Examples:

```text
once
this session
until restart
24 hours
always
```

---

# 338. Persistent option hidden/disabled if policy forbids.

---

# 339. Hard rule.

---

# 340. High-Risk Confirmation

Require explicit confirmation after scope/duration review.

---

# 341. Hard rule.

---

# 342. Admin Bulk Approval

Allowed only for managed contexts.

---

# 343. Must remain scope-bounded.

---

# 344. Hard rule.

---

# 345. Admin Cannot Approve Personal Context Access

Hard rule.

---

# 346. Tenant Boundary

No grant crosses tenant boundary.

---

# 347. Hard rule.

---

# 348. Federation Boundary

Extension grant in one federation domain does not confer authority in another.

---

# 349. Hard rule.

---

# 350. External Integration Boundary

SDK integration grants and local extension grants are separate records.

---

# 351. Hard rule.

---

# 352. Permission Migration

If permission model changes:

```text
map old grants to equal/narrower new grants
or
require re-consent
```

---

# 353. Hard rule.

---

# 354. Never Widen During Migration

Hard rule.

---

# 355. Legacy Grant

```rust
pub struct LegacyPermissionGrant {
    pub old_permission_code: u32,
    pub mapped_to: Option<ExtensionPermissionClass>,
}
```

---

# 356. Hard rule.

---

# 357. Unmappable Legacy Grant

Expires/requires review.

---

# 358. Hard rule.

---

# 359. Permission Revocation List

Could include extension/package-wide emergency deny.

---

# 360. Hard rule.

---

# 361. Emergency Policy

Can only restrict.

---

# 362. Hard rule.

---

# 363. Crisis Mode

Part 119.

Incident command may restrict extension permissions temporarily.

---

# 364. Cannot broaden.

---

# 365. Hard rule.

---

# 366. Support/Diagnostics

Support can explain permission state but cannot approve on behalf of user/admin unless explicitly authorized managed process.

---

# 367. Hard rule.

---

# 368. Marketplace Integration

Part 133.

Install/update screens derive permission data from canonical registry + manifest.

---

# 369. Hard rule.

---

# 370. Runtime Integration

Part 134.

Effective grants compile into runtime broker policy.

---

# 371. Hard rule.

---

# 372. SDK Integration

Part 130.

SDK capability requests map to extension permission registry.

---

# 373. Hard rule.

---

# 374. Portal/Toolchain Integration

Parts 131–132.

Developer tooling can inspect requested permissions but cannot self-approve production grants.

---

# 375. Hard rule.

---

# 376. Assurance Integration

Part 125.

Permission behavior requires contract/security/privacy tests.

---

# 377. Hard rule.

---

# 378. Archive Integration

Part 126.

High-value policy/consent/certification evidence archived.

---

# 379. Hard rule.

---

# 380. Knowledge Graph Integration

Part 123.

Trace:

```text
permission
→ requirement
→ policy
→ broker
→ tests
```

---

# 381. Hard rule.

---

# 382. Risk Integration

Part 121.

Overbroad permission is explicit risk.

---

# 383. Hard rule.

---

# 384. PIR Integration

Part 120.

Permission incident creates tighter policy/test.

---

# 385. Hard rule.

---

# 386. Testing

Need extension-authorization testkit.

---

# 387. Test Scenarios

```text
single-conversation read grant
microphone session grant
tenant admin extension
scope-expanding update
privacy-mode escalation
```

---

# 388. No Ambient Access Test

Installed extension with zero grants cannot access brokered resources.

---

# 389. Scope Attenuation Test

Request all conversations, approve one conversation → only one accessible.

---

# 390. Lifetime Test

Session grant expires at session end.

---

# 391. One-Shot Test

File read grant consumed once.

---

# 392. Delegation Test

Child scope cannot exceed parent.

---

# 393. Delegation Lifetime Test

Child cannot outlive parent.

---

# 394. Revocation Test

Parent revoke invalidates descendants immediately.

---

# 395. Update Diff Test

New network origin requires renewed consent.

---

# 396. Privacy Mode Test

Maximum anonymity removes forbidden permissions.

---

# 397. Tenant Policy Test

Tenant deny overrides user approval.

---

# 398. Device Policy Test

OS/device deny overrides extension grant.

---

# 399. Data Flow Test

MessageContentRead + NetworkExternal triggers enhanced review.

---

# 400. Migration Test

Legacy grant maps only equal/narrower.

---

# 401. Reinstall Test

High-risk persistent permission not automatically restored.

---

# 402. Recovery Test

Recovered account/device receives fresh runtime grants.

---

# 403. Privacy Test

No detailed grant-use timeline required.

---

# 404. Fuzzing

Fuzz:

```text
permission manifests
scope expressions
delegation chains
consent receipts
policy overlays
```

---

# 405. Property Tests

Properties:

```text
effective grant can never exceed requested scope
delegated capability can never exceed parent permission/scope/lifetime
denied platform/tenant/privacy policy can never be overridden by consent
scope-expanding update can never activate without required review
```

---

# 406. Formal Verification Targets

Strong candidates:

```text
policy precedence
capability attenuation
delegation/revocation
grant lifecycle
```

---

# 407. Kani Candidate

scope/lifetime/precedence invariants.

---

# 408. TLA+ Candidate

request → consent → grant → delegate → use → revoke/expire.

---

# 409. Loom Candidate

concurrent broker call + parent grant revocation + policy refresh.

---

# 410. Performance

Authorization is hot-path work.

---

# 411. Target:

```text
constant-time or indexed grant lookup
epoch check
compiled scope test
```

---

# 412. Hard rule.

---

# 413. No Unbounded Scope Evaluation

Hard rule.

---

# 414. Scope Set Bounds

Limit selected conversation/file/origin sets.

---

# 415. Hard rule.

---

# 416. Large Scope

Use policy-backed aggregate scope rather than giant raw ID list.

---

# 417. Hard rule.

---

# 418. Cache

Cache effective grant snapshot by runtime/grant epoch.

---

# 419. Hard rule.

---

# 420. Cache Invalidated On:

```text
revocation
privacy-mode change
tenant-policy change
device-policy change
extension update
```

---

# 421. Hard rule.

---

# 422. Crate Layout

Recommended:

```text
crates/
├── siar-extension-authz-core/
├── siar-extension-permission-registry/
├── siar-extension-consent/
├── siar-extension-scope/
├── siar-capability-attenuation/
├── siar-extension-delegation/
├── siar-extension-policy-overlay/
├── siar-extension-permission-review/
├── siar-extension-authz-observability/
└── siar-extension-authz-testkit/
```

---

# 423. `siar-extension-authz-core`

Owns:

```text
ExtensionPermissionClass
ExtensionPermissionGrant
GrantLifecycle
ExtensionAuthorizationError
```

---

# 424. `siar-extension-permission-registry`

Canonical permission definitions/schema/versioning.

---

# 425. `siar-extension-consent`

Consent receipts/prompt decisions/re-consent.

---

# 426. `siar-extension-scope`

Scope modeling/intersection/validation.

---

# 427. `siar-capability-attenuation`

Parent→child narrowing rules.

---

# 428. `siar-extension-delegation`

Audience-bound delegated capabilities.

---

# 429. `siar-extension-policy-overlay`

Privacy/device/tenant/platform precedence.

---

# 430. `siar-extension-permission-review`

Install/update/review-center UX models.

---

# 431. `siar-extension-authz-observability`

Aggregate permission policy health only.

---

# 432. `siar-extension-authz-testkit`

scope/delegation/revocation/privacy/formal tests.

---

# 433. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. Extension installation, manifest request, user/admin consent, effective authorization, delegated capability, runtime token, and OS/device permission are distinct states and cannot be conflated.
2. Effective extension authority is always the intersection of manifest request, user/admin approval, tenant/device/platform policy, privacy mode, runtime grant state, and current policy epoch; deny dominates.
3. No extension receives an "all access" permission, ambient administrator authority, unrestricted filesystem/network/device capability, or broader scope than explicitly requested and approved.
4. Delegated capabilities can only attenuate permission, scope, lifetime, and audience; child capabilities can never exceed or outlive their parent and are invalidated when parent authority is revoked.
5. Scope-expanding extension updates require diff-based review and renewed consent/admin approval where required; silent permission growth and auto-activation are prohibited.
6. Sensitive device permissions such as camera, microphone, clipboard, files, and local network require both SIAR authorization and applicable OS/user permission, with session/one-shot grants preferred where feasible.
7. Privacy-mode, tenant-policy, device-policy, and emergency-policy changes can only reduce extension authority; lowering a restriction later does not automatically restore previously withheld high-risk grants.
8. Permission combinations that create materially stronger exfiltration or surveillance risk—such as private-content read plus unrestricted network—receive explicit enhanced review/certification rather than being assessed only permission-by-permission.
9. Revocation, expiry, uninstall, account/device recovery, publisher/ownership transfer, and policy migration invalidate or re-review sensitive grants so stale authority cannot survive lifecycle transitions.
10. Consent UX uses platform-controlled, truthful, accessible permission descriptions with visible scope/lifetime and no dark patterns, hidden high-risk permissions, consent-rate optimization, or repeated manipulative prompting.
11. Permission telemetry, audit, review-center state, and consent records are minimized and technical; they cannot become detailed user-behavior timelines, developer/publisher scoring, or surveillance of how people use extensions.
12. Extension authorization integrates with marketplace manifests, SDK capabilities, runtime brokers, compatibility, tenant/device/privacy policies, assurance, certification, risk/PIR, account recovery, backup/restore, and emergency governance without creating a side channel around SIAR's security, privacy, anonymity, or tenant boundaries.
```

---

# 434. Initial Production Scope

Implement first:

```text
typed permission registry
permission sensitivity
requested vs effective scope
one-shot/session/time-bounded/persistent grants
platform-owned permission descriptions
install-time consent UX model
runtime JIT consent
scope attenuation
delegation chains
grant epochs
privacy/device/tenant policy overlays
scope-diff review on update
permission review center
grant revoke/expiry
background-job audience binding
sensitive permission combination rules
data-flow source/sink review
multi-device device-local grant semantics
uninstall/reinstall/recovery invalidation
RON permission policy compilation
authorization hot-path cache
privacy-safe permission metrics
extension-authz testkit
```

Then add:

```text
cross-device signed grant-intent sync
richer information-flow labels
formal delegation-chain proofs
policy simulation/explain tools
enterprise permission templates
offline policy bundles
automated exfiltration-risk rule generation
```

---

# 435. Definition of Done

Part 135 is complete when:

- permission classes are typed and fine-grained;
- requested and effective authority are separate;
- effective authority is an intersection of all policy layers;
- deny dominates;
- grant scopes/lifetimes are explicit;
- delegated capabilities only attenuate;
- parent revoke invalidates descendants;
- high-risk consent UX is explicit and accessible;
- scope-expanding updates require re-consent;
- privacy/tenant/device policy can immediately reduce access;
- device-sensitive grants are appropriately local/session-bound;
- reinstall/recovery does not resurrect stale high-risk permissions;
- exfiltration-risk permission combinations are reviewed;
- no permission telemetry becomes behavioral surveillance;
- scope/delegation/privacy/fuzz/formal tests are specified.

---

# 436. Final Architecture

```text
                  EXTENSION MANIFEST
                         │
                         ▼
                 REQUESTED PERMISSIONS
                         │
              ┌──────────┼──────────┐
              │          │          │
            USER       ADMIN      POLICY
              │          │          │
              └──────────┼──────────┘
                         ▼
                 EFFECTIVE GRANTS
                         │
                         ▼
                 ATTENUATED TOKENS
                         │
                         ▼
                RUNTIME CAPABILITY BROKER
                         │
                         ▼
              USE / EXPIRE / REVOKE / REVIEW
```

Authorization safety model:

```text
fine-grained permissions
+
explicit scope/lifetime
+
user/admin consent
+
policy intersection
+
capability attenuation
+
delegation revocation
+
scope-diff review
+
privacy-mode restriction
```

not:

```text
install once, grant everything forever, let updates silently add permissions, and assume user consent overrides platform or tenant security policy
```

---

# 437. Final Principle

Extension permissions are trustworthy when authority is visible, narrow, temporary where possible, and continuously revalidated.

The correct model is:

```text
declare what is requested
+
explain why and where
+
let the user/admin choose scope
+
intersect with platform/privacy/tenant policy
+
attenuate delegated authority
+
review permission diffs on update
+
expire and revoke immediately
+
never allow consent UX to become a mechanism for hidden privilege escalation
```

This architecture gives SIAR a privacy-preserving extension-authorization foundation for permission modeling, consent UX, delegated capabilities, attenuation, scope review, revocation, managed policy overlays, anonymous-mode restrictions, and secure update authorization while preserving the anonymity, local-first, least-authority, marketplace, SDK, extension-runtime, and anti-surveillance guarantees established across Parts 34–134.
