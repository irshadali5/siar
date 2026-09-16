# Core System Architecture Part 56 — Anonymous Network Privacy Policy, User Consent, Transparency, Data Rights & Privacy-Control Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 56  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 34–55  

**Primary purpose:** define the complete user-facing privacy-control architecture for SIAR, including privacy policy versioning, consent, privacy modes, local/remote processing controls, transparency notices, data rights, consent withdrawal, managed-policy interaction, privacy dashboards, machine-readable privacy receipts, and enforceable Rust policy boundaries.

---

# 1. Purpose

Privacy policy is not merely legal text.

A privacy-preserving system must make policy enforceable in product behavior.

Users need to understand and control:

```text
what data stays local
what leaves the device
what services are contacted
what is retained
what is optional
what is required
what is anonymous
what is only pseudonymous
```

The governing principle is:

> **Privacy choices should compile into enforceable runtime policy, not remain as promises in documentation.**

---

# 2. Architectural Position

```text
Human Privacy Policy
        │
        ▼
Machine-Readable Privacy Policy
        │
        ▼
Consent / Preference State
        │
        ▼
Effective Privacy Controls
        │
        ▼
Runtime Enforcement
```

---

# 3. Core Separation

Keep distinct:

```text
legal policy
privacy policy
security invariant
user preference
managed organization policy
consent record
runtime enforcement
```

---

# 4. Non-Goals

Part 56 does not create:

```text
click-through consent for mandatory security
dark-pattern consent
one global "agree to everything"
consent as excuse for unnecessary surveillance
```

---

# 5. Privacy Policy Domains

```rust
pub enum PrivacyPolicyDomain {
    Telemetry,
    Diagnostics,
    Presence,
    Discovery,
    ContactSync,
    Backup,
    CloudStorage,
    ExternalAI,
    LinkPreview,
    CrashReporting,
    Marketing,
}
```

---

# 6. Mandatory vs Optional Processing

Not every processing activity should ask for consent.

---

# 7. Required Processing

Examples:

```text
cryptographic protocol state
local message storage
service quota enforcement
security updates
```

---

# 8. Optional Processing

Examples:

```text
remote diagnostics
external AI processing
public handle discovery
cloud backup
```

---

# 9. Consent Basis

```rust
pub enum ProcessingBasis {
    RequiredForService,
    SecurityEssential,
    UserConsent,
    ManagedPolicy,
    LegalRequirement,
}
```

---

# 10. Consent Scope

Consent must be specific.

---

# 11. Consent Record

```rust
pub struct PrivacyConsentRecord {
    pub consent_id: ConsentId,
    pub policy_version: PrivacyPolicyVersion,
    pub domain: PrivacyPolicyDomain,
    pub basis: ProcessingBasis,
    pub granted_at: Timestamp,
    pub withdrawn_at: Option<Timestamp>,
}
```

---

# 12. Privacy Policy Version

```rust
pub struct PrivacyPolicyVersion(pub u64);
```

Monotonic.

---

# 13. Human Policy

Readable text.

---

# 14. Machine Policy

Canonical structured rules.

---

# 15. Policy Bundle

```rust
pub struct PrivacyPolicyBundle {
    pub version: PrivacyPolicyVersion,
    pub effective_at: Timestamp,
    pub controls: Vec<PrivacyControlRule>,
    pub notice_digest: [u8; 32],
    pub signatures: PolicySignatureBundle,
}
```

---

# 16. Privacy Control Rule

```rust
pub struct PrivacyControlRule {
    pub domain: PrivacyPolicyDomain,
    pub default: PrivacyControlState,
    pub user_configurable: bool,
    pub requires_consent: bool,
}
```

---

# 17. Control State

```rust
pub enum PrivacyControlState {
    Disabled,
    LocalOnly,
    Enabled,
    Required,
}
```

---

# 18. Policy Signature

Signed through governance.

---

# 19. User Preference Can Strengthen Privacy

Hard rule.

---

# 20. User Preference Cannot Disable Security Essential Processing

Hard rule.

---

# 21. Privacy Mode

Existing system-wide privacy profiles.

---

# 22. Privacy Modes

```rust
pub enum UserPrivacyMode {
    Standard,
    Private,
    Anonymous,
    MaximumAnonymity,
}
```

---

# 23. Mode Is Not Consent

Hard rule.

Privacy mode controls system behavior.

Consent controls optional processing.

---

# 24. Default Privacy Mode

Should be conservative.

---

# 25. No "Public By Default"

Hard rule for sensitive features.

---

# 26. Presence Default

Disabled or contacts-only.

---

# 27. Discovery Default

Exact-handle or invite-based.

---

# 28. Telemetry Default

Local-only in strict modes.

---

# 29. External AI Default

Off.

---

# 30. Cloud Backup Default

Explicit setup required.

---

# 31. Consent UX

Must be:

```text
specific
clear
reversible
non-coercive
```

---

# 32. No Bundled Consent

Avoid:

```text
"Agree to telemetry + marketing + AI + backup"
```

---

# 33. Per-Domain Consent

Preferred.

---

# 34. Consent Revocation

Must be possible if basis is consent.

---

# 35. Revocation Result

Future processing stops.

---

# 36. Existing Data

If already retained remotely:

```text
delete if policy allows
retain only if required
```

---

# 37. Consent State Machine

```rust
pub enum ConsentState {
    NotRequested,
    Granted,
    Withdrawn,
    Required,
    Unavailable,
}
```

---

# 38. Consent Service

```rust
pub trait PrivacyConsentService {
    fn state(
        &self,
        domain: PrivacyPolicyDomain,
    ) -> ConsentState;

    fn grant(
        &self,
        domain: PrivacyPolicyDomain,
        policy_version: PrivacyPolicyVersion,
    ) -> Result<(), PrivacyControlError>;

    fn withdraw(
        &self,
        domain: PrivacyPolicyDomain,
    ) -> Result<(), PrivacyControlError>;
}
```

---

# 39. Consent Receipt

Machine-readable.

---

# 40. Privacy Receipt

```rust
pub struct PrivacyReceipt {
    pub receipt_id: PrivacyReceiptId,
    pub policy_version: PrivacyPolicyVersion,
    pub domain: PrivacyPolicyDomain,
    pub state: ConsentState,
    pub timestamp: Timestamp,
}
```

---

# 41. Receipt Storage

Local-first.

---

# 42. Remote Consent Record

Only if needed for remote processing.

---

# 43. No Global Consent Identifier

Hard rule.

---

# 44. Service-Scoped Consent Proof

If remote service needs proof.

---

# 45. Consent Proof

```rust
pub struct ScopedConsentProof {
    pub domain: PrivacyPolicyDomain,
    pub policy_version: PrivacyPolicyVersion,
    pub expires_at: Timestamp,
    pub proof: ConsentProofBytes,
}
```

---

# 46. No Cross-Service Reuse

Hard rule.

---

# 47. Privacy Dashboard

User-facing control center.

---

# 48. Dashboard Sections

Recommended:

```text
Privacy Mode
Presence
Discovery
Diagnostics
Backup
External Services
Data Rights
Connected Providers
Policy History
```

---

# 49. Privacy Dashboard Principle

Show:

```text
what is happening now
```

not just settings.

---

# 50. Runtime Status

Examples:

```text
Remote telemetry: Off
Presence: Contacts only
Anonymous routing: Required
Cloud backup: Disabled
```

---

# 51. Data Flow Transparency

User should see broad categories.

---

# 52. Example

```text
Messages: device + recipient devices
Mailbox: encrypted ciphertext only
Relay: encrypted media packets
```

---

# 53. No False Precision

Do not imply provider learns nothing if it sees timing/volume.

---

# 54. Transparency Labels

Use honest language.

---

# 55. Example Privacy Claim

Good:

```text
The relay cannot read call content, but it may observe connection timing and traffic volume.
```

---

# 56. Bad Claim

```text
Completely invisible.
```

---

# 57. Privacy Notice Trigger

Only when material.

---

# 58. Material Changes

Examples:

```text
new remote processing
new data retention
new external provider
new optional analytics
```

---

# 59. Minor Changes

No repeated nuisance modal.

---

# 60. Change Classification

```rust
pub enum PrivacyPolicyChangeClass {
    Cosmetic,
    Clarification,
    Material,
    Critical,
}
```

---

# 61. Material Change

Requires notice.

---

# 62. Critical Change

May require re-consent for optional processing.

---

# 63. Re-Consent

Only relevant domains.

---

# 64. No Full-App Lockout For Unrelated Optional Consent

Hard rule.

---

# 65. Decline Behavior

Optional feature remains off.

Core secure service continues.

---

# 66. Dark Pattern Prohibition

Hard rule.

---

# 67. No Consent Nag Loop

If user declines optional processing, respect choice.

---

# 68. No Deceptive Defaults

Hard rule.

---

# 69. No Hidden Prechecked Optional Consent

Hard rule.

---

# 70. Consent Expiry

Usually not necessary unless policy/legal basis demands.

---

# 71. Scoped Time-Limited Consent

Possible for:

```text
one-time support upload
temporary external AI request
```

---

# 72. One-Time Consent

```rust
pub struct OneTimeConsent {
    pub domain: PrivacyPolicyDomain,
    pub scope: ConsentScope,
    pub expires_at: Timestamp,
}
```

---

# 73. Consent Scope

```rust
pub enum ConsentScope {
    SingleOperation,
    Session,
    Persistent,
}
```

---

# 74. Single Operation

Best for sensitive external processing.

---

# 75. External AI

Prefer:

```text
per-request
or
explicit persistent
```

---

# 76. Diagnostics Upload

Per-upload explicit.

---

# 77. Link Preview

Could be:

```text
local fetch
privacy proxy
disabled
```

---

# 78. External Link Fetch

Strict mode default:

```text
disabled
```

---

# 79. Contact Discovery

Private modes should not upload raw address book.

---

# 80. Address Book Consent

If external discovery ever uses contacts:

```text
clear explanation
data-minimized method
```

---

# 81. Maximum Anonymity

Disable remote address-book discovery.

---

# 82. Backup Consent

Cloud backup requires explicit configuration.

---

# 83. Backup Destination

User sees provider.

---

# 84. Backup Data Classes

User sees broad categories.

---

# 85. Backup Key Ownership

Explain:

```text
who can decrypt
```

---

# 86. No Hidden Recovery Key Escrow

Hard rule.

---

# 87. Crash Reporting

Consent can be separate.

---

# 88. Support Upload

Not same as crash reporting.

---

# 89. Telemetry

Separate from support.

---

# 90. Marketing

Entirely separate domain.

---

# 91. No Marketing Piggyback

Hard rule.

---

# 92. Data Rights

Architecture should support:

```text
access
export
deletion
correction
restriction
```

where applicable.

---

# 93. Local Data Access

User can inspect local data directly.

---

# 94. Operator-Held Data Access

Part 55 export.

---

# 95. Unified Data Rights View

Can coordinate:

```text
local data
operator-held data
backup data
```

---

# 96. Data Rights Request

```rust
pub struct DataRightsRequest {
    pub request_id: DataRightsRequestId,
    pub right: DataRight,
    pub scope: DataRightsScope,
    pub requested_at: Timestamp,
}
```

---

# 97. Data Right

```rust
pub enum DataRight {
    Access,
    Export,
    Delete,
    Correct,
    Restrict,
}
```

---

# 98. Data Rights Scope

```rust
pub enum DataRightsScope {
    LocalDevice,
    AccountService,
    BackupProvider,
    AllKnownRemoteServices,
}
```

---

# 99. No Claim Of Universal Reach

Hard rule.

Remote providers outside SIAR control may have separate policy.

---

# 100. Access Export

Can include:

```text
settings
policy records
consent records
account metadata
```

---

# 101. Message Export

Part 33.

---

# 102. Deletion

Must distinguish:

```text
local deletion
remote account deletion
remote ciphertext expiry
other recipients' copies
```

---

# 103. Deletion UX

Show consequences.

---

# 104. No "Delete Everywhere" Unless Technically True

Hard rule.

---

# 105. Data Deletion State

```rust
pub enum DeletionOutcome {
    Deleted,
    Scheduled,
    NotHeld,
    RetainedRequired,
    UnreachableProvider,
}
```

---

# 106. Correction

Useful for account/profile data.

---

# 107. Message History Correction

Not meaningful after send.

---

# 108. Restriction

Can disable optional remote processing.

---

# 109. Remote Provider List

User can inspect connected remote services.

---

# 110. Connected Provider Record

```rust
pub struct ConnectedPrivacyService {
    pub service_type: AnonymousServiceType,
    pub provider_id: AnonymousProviderId,
    pub policy_class: ProviderPrivacyClass,
}
```

---

# 111. Provider ID Display

Advanced view only.

---

# 112. Normal View

Human-readable provider category.

---

# 113. Policy Link

Show provider privacy declaration/audit class.

---

# 114. No Tracking Click

Provider policy docs can be bundled/cached.

---

# 115. Privacy Event Log

Local-only.

---

# 116. Events

```text
consent granted
consent withdrawn
privacy mode changed
backup enabled
external service enabled
```

---

# 117. Privacy Event

```rust
pub struct PrivacyEvent {
    pub event_type: PrivacyEventType,
    pub timestamp: Timestamp,
    pub policy_version: PrivacyPolicyVersion,
}
```

---

# 118. No Remote Upload Needed

Hard rule.

---

# 119. Privacy Mode Change

Can have consequences.

---

# 120. Example

Moving:

```text
MaximumAnonymity → Standard
```

may enable:

```text
presence
direct transport
external services
```

---

# 121. Confirmation

Required for meaningful privacy reduction.

---

# 122. Privacy Upgrade

Moving stricter can apply immediately.

---

# 123. Privacy Downgrade Guard

```rust
pub trait PrivacyDowngradeGuard {
    fn approve(
        &self,
        from: UserPrivacyMode,
        to: UserPrivacyMode,
    ) -> Result<(), PrivacyDowngradeError>;
}
```

---

# 124. No Background Privacy Downgrade

Hard rule.

---

# 125. Managed Policy

Organization may lock certain controls.

---

# 126. UI Must Explain

```text
Managed by organization
```

---

# 127. Managed Policy Cannot Pretend User Chose It

Hard rule.

---

# 128. User Preference vs Managed Policy

Effective state should show both.

---

# 129. Effective Control

```rust
pub struct EffectivePrivacyControl {
    pub domain: PrivacyPolicyDomain,
    pub state: PrivacyControlState,
    pub source: PrivacyControlSource,
}
```

---

# 130. Control Source

```rust
pub enum PrivacyControlSource {
    User,
    Governance,
    ManagedPolicy,
    SecurityInvariant,
    LegalRequirement,
}
```

---

# 131. Transparency Benefit

User sees why setting cannot change.

---

# 132. Security Invariant

Examples:

```text
no direct fallback
no unauthenticated downgrade
```

---

# 133. Not Exposed As User Toggle

Hard rule.

---

# 134. Privacy Policy Compiler

Human policy -> runtime rules.

---

# 135. Compiler

```rust
pub trait PrivacyPolicyCompiler {
    fn compile(
        &self,
        bundle: &PrivacyPolicyBundle,
    ) -> Result<CompiledPrivacyPolicy, PrivacyControlError>;
}
```

---

# 136. Runtime Gate

```rust
pub trait PrivacyControlGate {
    fn authorize(
        &self,
        operation: PrivacySensitiveOperation,
        policy: &EffectivePrivacyPolicy,
    ) -> Result<PrivacyAuthorization, PrivacyControlError>;
}
```

---

# 137. Sensitive Operations

```rust
pub enum PrivacySensitiveOperation {
    SendTelemetry,
    UploadDiagnostics,
    EnablePresence,
    QueryDiscovery,
    UseExternalAI,
    FetchExternalPreview,
    UploadBackup,
    SyncContactGraph,
}
```

---

# 138. Default Deny

If policy missing or ambiguous:

```text
deny remote optional operation
```

---

# 139. Fail Closed

Hard rule.

---

# 140. Privacy Authorization

```rust
pub struct PrivacyAuthorization {
    pub domain: PrivacyPolicyDomain,
    pub policy_version: PrivacyPolicyVersion,
    pub scope: ConsentScope,
}
```

---

# 141. Authorization Lifetime

Short for one-time/sensitive operations.

---

# 142. Policy Cache

Local signed.

---

# 143. Stale Policy

Core security continues.

New optional remote processing pauses if needed.

---

# 144. Privacy Policy Rollback

Reject lower version.

---

# 145. Anti-Rollback

Required.

---

# 146. Privacy Mode Sync

Multi-device.

---

# 147. Sync Policy

Share:

```text
preferences
consent state
```

E2EE.

---

# 148. Device-Specific Controls

Some controls remain device-local.

Examples:

```text
crash reporting
local notifications
```

---

# 149. Account-Level Controls

Examples:

```text
public discovery
cloud backup
```

---

# 150. Control Scope

```rust
pub enum PrivacyControlScope {
    Device,
    Account,
    Conversation,
    Group,
    Operation,
}
```

---

# 151. Conversation-Level Privacy

Possible:

```text
receipts
typing
presence sharing
```

---

# 152. Group-Level Privacy

Part 43 integration.

---

# 153. Operation-Level Privacy

External AI/share/export.

---

# 154. Scope Hierarchy

Most restrictive applicable rule wins.

---

# 155. Example

Global receipts on.

Conversation receipts off.

Result:

```text
off for that conversation
```

---

# 156. Privacy Policy Merge

```rust
pub trait PrivacyPolicyMerger {
    fn effective(
        &self,
        inputs: PrivacyPolicyInputs,
    ) -> Result<EffectivePrivacyPolicy, PrivacyControlError>;
}
```

---

# 157. Inputs

```text
security invariants
governance policy
legal restrictions
managed policy
user mode
user preference
conversation/group override
```

---

# 158. Restrictive Merge

Preferred.

---

# 159. No Lower Layer Weakening Higher Floor

Hard rule.

---

# 160. Transparency Surface

Need user-readable explanation.

---

# 161. Explain Why

Examples:

```text
Disabled by Maximum Anonymity
Required by organization
Unavailable in this jurisdiction
```

---

# 162. Privacy Nutrition Label

Optional summary.

---

# 163. Good Fields

```text
content encryption
metadata exposure
retention
external processing
```

---

# 164. No Misleading Score

Hard rule.

---

# 165. No Single "Privacy 95/100"

Hard rule.

---

# 166. Better

Structured claims.

---

# 167. Claim Assurance

```rust
pub enum PrivacyClaimAssurance {
    Architectural,
    Verified,
    Audited,
    SelfDeclared,
}
```

---

# 168. Example

```text
Message content encryption: Architectural
Provider logging policy: SelfDeclared/Audited
```

---

# 169. Transparency Report

Network-wide.

---

# 170. Contents

```text
policy changes
privacy incidents
lawful request aggregates
audit summaries
```

---

# 171. No User-Level Data

Hard rule.

---

# 172. Policy History

User can inspect historical policy versions.

---

# 173. Diff View

Show what changed.

---

# 174. Policy Diff

```rust
pub struct PrivacyPolicyDiff {
    pub from: PrivacyPolicyVersion,
    pub to: PrivacyPolicyVersion,
    pub changes: Vec<PrivacyPolicyChange>,
}
```

---

# 175. Material Change Highlight

Recommended.

---

# 176. Consent Reconciliation

If policy changes:

```text
existing consent may remain
or
re-consent may be required
```

---

# 177. Re-Consent Rule

```rust
pub enum ReconsentRequirement {
    None,
    NoticeOnly,
    Required,
}
```

---

# 178. Re-Consent Trigger

Only if purpose/scope materially expands.

---

# 179. No Re-Consent Theater

Hard rule.

---

# 180. User Data Export

Machine-readable.

---

# 181. Export Formats

Use:

```text
RON
JSON for interoperability if needed
Markdown summary
```

---

# 182. Export Security

Encrypted archive option.

---

# 183. Export Warning

File may contain sensitive data.

---

# 184. Remote Deletion Workflow

Track status.

---

# 185. Deletion Request State

```rust
pub enum RemoteDeletionState {
    Pending,
    Sent,
    Confirmed,
    PartiallyCompleted,
    Failed,
}
```

---

# 186. No Infinite Retry

Bounded.

---

# 187. User Feedback

Show unresolved provider.

---

# 188. Provider Deletion Capability

Descriptor can advertise.

---

# 189. Data Rights Capability

```rust
pub struct ProviderDataRightsCapabilities {
    pub access: bool,
    pub deletion: bool,
    pub correction: bool,
    pub export: bool,
}
```

---

# 190. Local Privacy Audit

Client can compute:

```text
which optional remote operations are enabled
```

---

# 191. Privacy Audit Result

```rust
pub struct LocalPrivacyAudit {
    pub remote_processing_enabled: Vec<PrivacyPolicyDomain>,
    pub strict_mode_conflicts: Vec<PrivacyConflict>,
}
```

---

# 192. Privacy Conflict

Example:

```text
MaximumAnonymity + external AI enabled
```

should not happen.

---

# 193. Runtime Invariant

Prevent invalid combination.

---

# 194. Privacy Testkit

Need policy/consent simulator.

---

# 195. Scenarios

```text
grant consent
withdraw consent
policy upgrade
policy rollback
managed lock
jurisdiction restriction
privacy mode change
multi-device sync
```

---

# 196. Consent Withdrawal Test

Remote optional operation stops.

---

# 197. Stale Policy Test

Optional remote processing pauses safely.

---

# 198. Managed Policy Test

User sees locked state and source.

---

# 199. Re-Consent Test

Only affected domain asks again.

---

# 200. Privacy Downgrade Test

Background service cannot lower privacy mode.

---

# 201. Dark Pattern UX Test

Optional consent decline remains easy.

---

# 202. Data Export Test

Only requested scope exported.

---

# 203. Remote Deletion Test

Accurate outcome state.

---

# 204. Provider Capability Test

Unsupported deletion right reported honestly.

---

# 205. Multi-Device Test

Account-level privacy choices sync E2EE.

---

# 206. Device Scope Test

Device-only choice does not leak to account.

---

# 207. Fuzzing

Fuzz:

```text
privacy policy bundle
consent receipt
policy diff
data-rights request
```

---

# 208. Property Tests

Properties:

```text
optional remote processing requires valid authorization
withdrawn consent cannot authorize future operation
policy version never rolls back
most restrictive applicable control wins
```

---

# 209. Formal Verification Targets

Strong candidates:

```text
policy precedence
consent state machine
privacy-mode downgrade guard
data-rights request state
```

---

# 210. TLA+ Candidate

Consent withdrawal racing with remote operation.

---

# 211. Kani Candidate

Effective-policy merge.

---

# 212. Loom Candidate

Concurrent privacy setting update + operation authorization.

---

# 213. Performance Tests

Measure:

```text
policy merge
authorization gate
consent lookup
privacy dashboard load
```

---

# 214. No Per-Packet Privacy Policy Evaluation

Hard rule.

---

# 215. Cache Effective Policy

Local.

---

# 216. Invalidate On

```text
policy update
consent change
managed policy change
privacy mode change
```

---

# 217. Observability

Safe metrics:

```text
policy version adoption
privacy control failures
consent-domain aggregate counts
```

only if privacy-safe.

---

# 218. Maximum Anonymity

No remote consent telemetry by default.

---

# 219. Forbidden Telemetry

No:

```text
user identity + consent choice
contact + privacy mode
```

---

# 220. Privacy Dashboard Analytics

Local only.

---

# 221. Support Bundle

Can include:

```text
effective privacy mode
policy version
```

but not detailed consent history unless user explicitly chooses.

---

# 222. Security & Privacy Invariants

Mandatory:

```text
1. Optional remote processing requires explicit policy authorization and, where required, valid consent.
2. Consent is scoped, reversible, and not bundled across unrelated domains.
3. User privacy preferences may strengthen but not weaken mandatory security/privacy floors.
4. Managed/legal policy can restrict availability but cannot enable weaker privacy paths.
5. Maximum Anonymity disables optional remote telemetry/external processing by default.
6. Privacy mode changes that weaken protection require explicit user confirmation.
7. No background component can silently reduce privacy mode or enable optional remote processing.
8. Machine-readable policy is anti-rollback protected and signed.
9. Data-rights workflows accurately distinguish deleted, retained, not-held, and unreachable data.
10. User-facing privacy claims state limitations honestly and avoid absolute guarantees.
11. Privacy dashboard explains the source of effective controls.
12. Privacy settings and consent records do not become a remote behavioral tracking profile.
```

---

# 223. Recommended Crate Layout

```text
crates/
├── siar-privacy-policy-core/
├── siar-privacy-policy-bundle/
├── siar-consent/
├── siar-privacy-control/
├── siar-privacy-mode/
├── siar-data-rights/
├── siar-privacy-receipt/
├── siar-privacy-dashboard/
├── siar-privacy-transparency/
├── siar-privacy-policy-sync/
└── siar-privacy-control-testkit/
```

---

# 224. `siar-privacy-policy-core`

Owns:

```text
domains
states
scopes
versions
errors
```

---

# 225. `siar-privacy-policy-bundle`

Signed machine-readable policy.

---

# 226. `siar-consent`

Consent records/receipts/revocation.

---

# 227. `siar-privacy-control`

Runtime authorization gate.

---

# 228. `siar-privacy-mode`

Mode transitions/downgrade guard.

---

# 229. `siar-data-rights`

Access/export/delete/correct/restrict workflows.

---

# 230. `siar-privacy-receipt`

Machine-readable user receipts.

---

# 231. `siar-privacy-dashboard`

Presentation DTOs/status.

---

# 232. `siar-privacy-transparency`

Policy history/diff/transparency.

---

# 233. `siar-privacy-policy-sync`

E2EE multi-device preference sync.

---

# 234. `siar-privacy-control-testkit`

Consent/policy/data-rights simulator.

---

# 235. Error Taxonomy

```rust
pub enum PrivacyControlError {
    ConsentRequired,
    ConsentWithdrawn,
    PolicyStale,
    PolicyRollback,
    ManagedRestriction,
    LegalRestriction,
    PrivacyDowngradeRejected,
    UnsupportedControl,
    DataRightsUnavailable,
    InvalidPolicy,
    Internal,
}
```

---

# 236. Initial Production Scope

Implement first:

```text
signed machine-readable privacy policy
privacy modes
per-domain optional processing controls
consent records
consent revocation
privacy downgrade guard
privacy dashboard
policy history/diff
data-access/export/deletion workflows
managed-policy source display
local-only privacy event log
runtime privacy authorization gate
```

Then add:

```text
machine-verifiable privacy receipts
provider data-rights capability negotiation
advanced privacy nutrition labels
formal policy proofs
cross-device privacy conflict resolution
```

---

# 237. Definition of Done

Part 56 is complete when:

- human and machine-readable privacy policy are separated but linked
- privacy policy is signed, versioned, and anti-rollback protected
- required/security processing and optional consent-based processing are explicit
- consent is specific, reversible, scoped, and non-bundled
- privacy modes and consent are separate concepts
- optional remote operations are runtime-gated
- effective policy merges security, governance, legal, managed, user, conversation, and group controls
- most-restrictive applicable policy wins
- privacy downgrade requires explicit confirmation
- privacy dashboard shows effective runtime state and source
- policy history/diffs are user-visible
- data access/export/deletion/correction/restriction workflows are explicit
- remote deletion outcomes are honest
- machine-readable privacy receipts exist
- multi-device privacy preference sync is E2EE
- privacy/consent settings cannot become remote behavioral tracking data
- consent, downgrade, policy rollback, rights, fuzz, and formal tests are specified

---

# 238. Final Architecture

```text
                    HUMAN PRIVACY POLICY
                              │
                              ▼
                 MACHINE-READABLE POLICY
                              │
                              ▼
                    CONSENT / PREFERENCES
                              │
                     ┌────────┴────────┐
                     │                 │
                User Mode        Managed/Legal
                     │                 │
                     └────────┬────────┘
                              ▼
                    EFFECTIVE POLICY
                              │
                              ▼
                    RUNTIME PRIVACY GATE
                              │
                              ▼
                         OPERATION
```

Privacy-control safety model:

```text
signed policy
+
specific consent
+
restrictive policy merge
+
runtime enforcement
+
transparent UI
+
accurate data-rights workflows
```

not:

```text
legal text says one thing
while runtime does another
```

---

# 239. Final Principle

Privacy policy is only meaningful when it is enforceable.

The correct model is:

```text
clear policy
+
specific consent
+
honest transparency
+
runtime controls
+
user-accessible data rights
```

with security and anonymity floors that cannot be silently weakened by remote policy, managed configuration, or convenience features.

This architecture gives SIAR a complete user-facing privacy-control layer above the legal/compliance boundary defined in Part 55, while preserving the anonymity and metadata-minimization guarantees established across Parts 34–55.
