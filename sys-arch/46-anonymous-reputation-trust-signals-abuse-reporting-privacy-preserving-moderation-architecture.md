# Core System Architecture Part 46 — Anonymous Reputation, Trust Signals, Abuse Reporting & Privacy-Preserving Moderation Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 46  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 34–45  

**Primary purpose:** define the complete privacy-preserving accountability architecture for SIAR, including scoped reputation, trust signals, abuse reporting, evidence packaging, moderation capabilities, sanctions, false-report resistance, Sybil-aware reputation, appeals, transparency, and Rust service boundaries.

---

# 1. Purpose

Privacy systems face a tension:

```text
strong anonymity
vs
accountability
```

A system that exposes identity everywhere weakens privacy.

A system with no accountability at all may become vulnerable to:

```text
spam
harassment
fraud
malware
coordinated abuse
resource exhaustion
```

The governing principle is:

> **SIAR should support abuse resistance and moderation through scoped capabilities, local trust, pseudonymous evidence, and privacy-preserving sanctions rather than requiring a globally linkable identity.**

---

# 2. Architectural Position

```text
Messages / Calls / Groups / Discovery
                 │
                 ▼
        Local Trust / Reputation
                 │
                 ▼
          Abuse Reporting
                 │
                 ▼
       Moderation Capability
                 │
                 ▼
      Scoped Sanction / Revocation
```

---

# 3. Core Separation

Keep separate:

```text
Identity Trust
Behavior Reputation
Relationship Trust
Moderation Authority
Global Identity
```

---

# 4. Non-Goals

Part 46 does not require:

```text
public social credit
global user score
real-name identity
centralized behavior profiling
public blacklist of users
```

---

# 5. Trust Types

```rust
pub enum TrustSignalType {
    IdentityVerified,
    DeviceVerified,
    KnownContact,
    GroupMember,
    SuccessfulHistory,
    ModeratorVerified,
}
```

---

# 6. Identity Trust

Answers:

```text
is this the same identity I verified?
```

---

# 7. Behavioral Trust

Answers:

```text
has this pseudonymous participant behaved acceptably in this scope?
```

---

# 8. Relationship Trust

Local user decision.

---

# 9. Reputation Scope

Reputation MUST be scoped.

Possible scopes:

```text
relationship
group
community
service
operator
```

---

# 10. Reputation Scope Type

```rust
pub enum ReputationScope {
    Relationship(RelationshipId),
    Group(GroupId),
    Community(CommunityId),
    Service(ServiceId),
}
```

---

# 11. No Global User Reputation by Default

Hard rule.

---

# 12. Why

Global reputation creates:

```text
cross-context tracking
behavioral profiling
identity correlation
social graph reconstruction
```

---

# 13. Pseudonymous Reputation

Use scope-local subject IDs.

```rust
pub struct ReputationSubjectId(pub [u8; 32]);
```

---

# 14. Subject Rotation

Can rotate on:

```text
epoch
community migration
privacy policy
```

---

# 15. Rotation Tradeoff

Too frequent:

```text
destroys accountability
```

Too stable:

```text
increases linkability
```

---

# 16. Initial Recommendation

Per-community or per-group stable pseudonym.

---

# 17. Trust Signal

```rust
pub struct TrustSignal {
    pub signal_type: TrustSignalType,
    pub scope: ReputationScope,
    pub subject: ReputationSubjectId,
    pub issued_at: Timestamp,
    pub expires_at: Option<Timestamp>,
    pub issuer: TrustIssuer,
}
```

---

# 18. Trust Issuer

Could be:

```text
local user
group authority
service authority
verified contact
```

---

# 19. Local Trust First

Prefer:

```text
local trust graph
```

over server-owned global score.

---

# 20. Reputation Model

Avoid opaque single-number score.

---

# 21. Better Model

Structured dimensions:

```rust
pub struct ReputationSummary {
    pub reliability: ReputationLevel,
    pub abuse_history: AbuseRiskLevel,
    pub verification: VerificationLevel,
    pub account_age_bucket: Option<AccountAgeBucket>,
}
```

---

# 22. Reputation Level

```rust
pub enum ReputationLevel {
    Unknown,
    Limited,
    Established,
    Trusted,
}
```

---

# 23. Abuse Risk Level

```rust
pub enum AbuseRiskLevel {
    Unknown,
    Low,
    Elevated,
    Restricted,
}
```

---

# 24. No Absolute Score

Hard rule.

---

# 25. Evidence-Based Signals

Trust signals should correspond to evidence.

Examples:

```text
identity verified in person
member has existed for 90+ days
no valid abuse sanctions
moderator attestation
```

---

# 26. Privacy Boundary

Do not publish raw evidence globally.

---

# 27. Abuse Categories

```rust
pub enum AbuseCategory {
    Spam,
    Harassment,
    Threat,
    Fraud,
    Malware,
    Impersonation,
    UnauthorizedContent,
    ResourceAbuse,
    ModerationEvasion,
}
```

---

# 28. Report Scope

A report should be tied to:

```text
specific event
specific pseudonym
specific group/community
```

not necessarily global account identity.

---

# 29. Abuse Report

```rust
pub struct AbuseReport {
    pub report_id: AbuseReportId,
    pub scope: ReputationScope,
    pub subject: ReputationSubjectId,
    pub category: AbuseCategory,
    pub evidence: AbuseEvidenceBundle,
    pub reporter: ReporterPrivacyDescriptor,
    pub created_at: Timestamp,
}
```

---

# 30. Report ID

Random.

---

# 31. Reporter Privacy

Possible:

```text
identified to moderator
pseudonymous
anonymous
```

---

# 32. Reporter Privacy Type

```rust
pub enum ReporterPrivacyMode {
    Identified,
    Pseudonymous,
    Anonymous,
}
```

---

# 33. Anonymous Reporting

Useful for sensitive communities.

---

# 34. Risk

Anonymous reports can be abused.

---

# 35. Therefore

Need:

```text
rate limits
proof of membership
evidence requirements
duplicate suppression
```

---

# 36. Abuse Evidence

Potential:

```text
message
attachment digest
call event
group control event
metadata necessary for verification
```

---

# 37. Evidence Minimization

Only include what is needed.

---

# 38. Evidence Bundle

```rust
pub struct AbuseEvidenceBundle {
    pub version: EvidenceVersion,
    pub event_proofs: Vec<EventEvidence>,
    pub context_digest: Option<[u8; 32]>,
    pub reporter_statement: Option<EncryptedText>,
}
```

---

# 39. Event Evidence

Could contain:

```text
signed message envelope
sender pseudonym
epoch
timestamp bucket
```

---

# 40. Avoid Full Conversation Export

Hard rule unless reporter explicitly chooses.

---

# 41. Message Evidence Authenticity

Use existing message authentication.

---

# 42. Attachment Evidence

Prefer:

```text
content digest
manifest proof
```

plus optional file upload if required.

---

# 43. Sensitive Evidence

Encrypted to moderation authority.

---

# 44. Evidence Storage

Separate secure store.

---

# 45. Evidence Retention

Short and policy-defined.

---

# 46. Evidence Access

Role-based and audited.

---

# 47. Moderator Capability

```rust
pub struct ModeratorCapability(SecretBytes);
```

---

# 48. Capability Scope

Bound to:

```text
community/group
actions
expiry
```

---

# 49. Moderator Actions

```rust
pub enum ModerationAction {
    Warn,
    RateLimit,
    Mute,
    RemoveContent,
    SuspendPosting,
    RemoveMember,
    RevokeCapability,
}
```

---

# 50. Moderation Does Not Require Real Identity

Hard rule.

---

# 51. Scope-Limited Sanction

A group moderator can sanction:

```text
that group pseudonym
```

without learning global account identity.

---

# 52. Global Safety Authority

If SIAR ever supports cross-service sanctions, it must be a separate explicit subsystem.

---

# 53. Default

No global cross-context moderation identity.

---

# 54. Sanction Object

```rust
pub struct ModerationSanction {
    pub sanction_id: SanctionId,
    pub scope: ReputationScope,
    pub subject: ReputationSubjectId,
    pub action: ModerationAction,
    pub starts_at: Timestamp,
    pub expires_at: Option<Timestamp>,
    pub authority_proof: ModerationAuthorityProof,
}
```

---

# 55. Sanction Enforcement

Possible:

```text
deny post capability
deny call capability
deny group join
rate-limit requests
```

---

# 56. Capability Revocation

Preferred enforcement primitive.

---

# 57. Why

Works without:

```text
global identity blacklist
```

---

# 58. Temporary Sanction

Can expire automatically.

---

# 59. Permanent Group Removal

Revoke membership capability and rekey group.

---

# 60. Moderation Privacy

Ordinary members need not know moderator identity.

---

# 61. Moderator Accountability

Administrative audit can still know which authority capability performed action.

---

# 62. Anonymous Moderator Action

Public event may say:

```text
Authorized moderator action
```

---

# 63. Moderation Audit Record

```rust
pub struct ModerationAuditRecord {
    pub action_id: ModerationActionId,
    pub scope: ReputationScope,
    pub authority: ModerationAuthorityId,
    pub action: ModerationAction,
    pub timestamp: Timestamp,
}
```

---

# 64. Audit Privacy

No reporter identity unless required.

---

# 65. False Reporting

Threat:

```text
coordinated malicious reports
```

---

# 66. Defenses

```text
rate limits
reporter reputation
evidence authenticity
duplicate detection
moderator review
```

---

# 67. Reporter Reputation

Scoped.

---

# 68. No Retaliatory Public Score

Hard rule.

---

# 69. Duplicate Reports

Can be aggregated.

---

# 70. Report Fingerprint

Use privacy-safe digest.

---

# 71. Coordinated Abuse Detection

May aggregate:

```text
same evidence
same subject
same scope
```

without cross-scope identity linking.

---

# 72. Sybil Reporting Attack

Attacker creates many pseudonyms and reports same target.

---

# 73. Mitigations

```text
membership age
capability age
resource cost
rate limits
proof of participation
```

---

# 74. Proof of Participation

Reporter proves:

```text
was valid member during event
```

without revealing global identity.

---

# 75. Anonymous Credential

Future option.

---

# 76. Initial Production

Use:

```text
scope-local membership capability proof
```

---

# 77. Trust Aggregation

Avoid naive:

```text
100 reports = guilty
```

---

# 78. Better

Weight by:

```text
evidence quality
independent reporters
scope history
moderator verification
```

---

# 79. Independence

Reports from same operator/device cluster should not count as fully independent where detectable.

---

# 80. Privacy Constraint

Sybil detection must not become universal device tracking.

---

# 81. Device Fingerprint

Avoid persistent fingerprint.

---

# 82. Safer Sybil Inputs

```text
membership age
capability issuance history
rate-limit token
scope-local credential
```

---

# 83. Trust Attestation

```rust
pub struct TrustAttestation {
    pub scope: ReputationScope,
    pub subject: ReputationSubjectId,
    pub claim: TrustClaim,
    pub issuer: TrustIssuer,
    pub expires_at: Option<Timestamp>,
}
```

---

# 84. Trust Claim

Examples:

```text
VerifiedMember
LongStandingMember
ModeratorApproved
```

---

# 85. Public Trust Badges

Can leak social structure.

---

# 86. Recommendation

Expose only minimal user-facing trust indicators.

---

# 87. User-Facing Trust

Examples:

```text
Verified
Known contact
New member
Restricted
```

---

# 88. No "87/100 Trust Score"

Hard rule.

---

# 89. Local Reputation Cache

Store:

```text
scope-local trust
sanctions
report outcomes
```

---

# 90. Server Reputation State

If server-assisted:

```text
opaque scoped records
```

not global graph.

---

# 91. Community Reputation Authority

Could maintain:

```text
community-local pseudonym state
```

---

# 92. Federation

Different communities may have independent moderation policy.

---

# 93. Cross-Community Import

Default:

```text
no automatic reputation import
```

---

# 94. Optional Portable Attestations

User may carry signed claims.

---

# 95. Portable Attestation Risk

Cross-context linkability.

---

# 96. Strict Mode

Disable portable reputation by default.

---

# 97. Reputation Decay

Old behavior should not define user forever.

---

# 98. Expiry

Positive and negative signals can expire.

---

# 99. Severe Sanctions

May persist longer.

---

# 100. Policy-Driven

```rust
pub struct ReputationRetentionPolicy {
    pub positive_ttl: Duration,
    pub warning_ttl: Duration,
    pub severe_ttl: Duration,
}
```

---

# 101. Appeal

Moderated systems need appeal.

---

# 102. Appeal Privacy

Appeal can use:

```text
scope-local pseudonym
sanction ID
```

without global identity.

---

# 103. Appeal Object

```rust
pub struct ModerationAppeal {
    pub appeal_id: AppealId,
    pub sanction_id: SanctionId,
    pub subject: ReputationSubjectId,
    pub statement: EncryptedText,
}
```

---

# 104. Appeal Authority

May differ from original moderator.

---

# 105. Two-Level Review

Useful for serious sanctions.

---

# 106. Emergency Actions

Immediate suspension may happen before full review.

---

# 107. Audit Requirement

Emergency action must be reviewed later.

---

# 108. Transparency

Community may publish aggregate moderation statistics.

---

# 109. Safe Aggregate Stats

```text
reports received
reports upheld
sanctions issued
appeals overturned
```

---

# 110. Differential Privacy

Potential for public aggregate metrics.

---

# 111. No Small-Group Stats

Avoid deanonymization.

---

# 112. Moderation Transparency Log

Append-only local/community log.

---

# 113. Public vs Private Entries

Public:

```text
policy version
aggregate action
```

Private:

```text
evidence
reporter
subject mapping
```

---

# 114. Policy Version

```rust
pub struct ModerationPolicyVersion(pub u64);
```

---

# 115. Moderation Policy

```rust
pub struct ModerationPolicy {
    pub version: ModerationPolicyVersion,
    pub report_rules: ReportRules,
    pub sanction_rules: SanctionRules,
    pub appeal_rules: AppealRules,
}
```

---

# 116. Signed Policy

Group/community participants can verify policy version.

---

# 117. Policy Changes

Should not silently change retroactively.

---

# 118. Retroactive Enforcement

Only for severe safety cases if explicitly defined.

---

# 119. Abuse Rate Limits

Unknown users:

```text
strict limits
```

Trusted scoped users:

```text
higher limits
```

---

# 120. Rate-Limit Credential

```rust
pub struct ScopedRateLimitCredential(SecretBytes);
```

---

# 121. Privacy-Preserving Rate Limits

Goal:

```text
limit actions
without global identity
```

---

# 122. Token Bucket

Scoped by pseudonymous credential.

---

# 123. Anonymous E-Cash-Style Tokens

Future option for unlinkable rate limits.

---

# 124. Initial Scope

Use pseudonymous scoped tokens.

---

# 125. Resource Abuse

Examples:

```text
large uploads
call floods
group spam
bridge abuse
```

---

# 126. Resource Policy

Trust level may affect:

```text
upload quota
request quota
call invitation rate
```

---

# 127. Do Not Reduce E2EE Security

Rate limiting operates outside content encryption.

---

# 128. Malware Reports

Can include:

```text
file digest
manifest digest
```

---

# 129. Known-Bad Digest Lists

Potential.

---

# 130. Privacy Risk

File hash lookup can reveal what users possess.

---

# 131. Recommendation

Use local known-bad checks where possible.

---

# 132. Remote Reputation Lookup

If used, query privacy-preservingly.

---

# 133. No Raw Attachment Hash Telemetry

Hard rule.

---

# 134. Contact-Level Trust

User can locally mark:

```text
trusted
untrusted
blocked
```

---

# 135. Local Trust Does Not Propagate Automatically

Hard rule.

---

# 136. Trusted Introductions

Can produce signed attestation.

---

# 137. No Web-of-Trust Auto-Expansion

Avoid:

```text
friend of friend is trusted
```

---

# 138. Group Moderation Integration

Part 43 capabilities:

```text
post
read
moderate
```

can be revoked.

---

# 139. Anonymous Group Ban

Revoke:

```text
membership capability
post capability
future epoch keys
```

---

# 140. Group Report

Reporter may remain pseudonymous to moderators if policy allows.

---

# 141. Group Message Proof

Report includes signed/authenticated message envelope.

---

# 142. Private Calls Abuse

Call spam can be reported by:

```text
invite proof
call capability
session metadata
```

---

# 143. Do Not Upload Media by Default

Call report does not include audio/video unless user explicitly supplies evidence.

---

# 144. Discovery Abuse

Report:

```text
spam contact request
impersonation
enumeration behavior
```

---

# 145. Presence Abuse

Presence is capability-gated, so abuse mostly means capability misuse.

---

# 146. Bridge/Mix Abuse

Infrastructure moderation is separate operator governance.

---

# 147. Reputation Across Infrastructure

Node/operator reputation from Part 39 is distinct from user/community reputation.

---

# 148. Hard Separation

Do not mix:

```text
user behavior reputation
mix-node operator reputation
```

---

# 149. User Safety Preferences

Local filters can act before moderation.

---

# 150. Examples

```text
block unknown callers
hide unknown attachments
mute new group members
```

---

# 151. Local Filtering

Privacy-preserving because no server decision needed.

---

# 152. Content Classification

If optional on-device AI is used:

```text
local only by default
```

---

# 153. External Moderation AI

Requires explicit consent and privacy review.

---

# 154. Maximum Anonymity

No external content moderation API by default.

---

# 155. Report Submission Path

Use anonymous transport where possible.

---

# 156. Moderator Endpoint

Report can be delivered via:

```text
anonymous mailbox
mixnet
community moderation mailbox
```

---

# 157. Reporter IP Hiding

Strict mode requires anonymous report submission.

---

# 158. Evidence Confidentiality

Encrypt to moderation authority.

---

# 159. Evidence Integrity

Signed/authenticated.

---

# 160. Evidence Chain

```rust
pub struct EvidenceChain {
    pub items: Vec<EvidenceItem>,
    pub root_digest: [u8; 32],
}
```

---

# 161. Evidence Mutation

Detected.

---

# 162. Moderator View

Shows:

```text
report category
evidence
scope
subject pseudonym
policy
```

---

# 163. Moderator Must Not See Unnecessary Global Identity

Hard rule.

---

# 164. Escalation

Some cases may require explicit identity/legal escalation outside ordinary anonymous moderation.

---

# 165. Separate Boundary

Must be policy- and jurisdiction-specific.

---

# 166. No Automatic Deanonymization

Hard rule.

---

# 167. Identity Reveal Capability

Do not build universal deanonymization key into SIAR.

---

# 168. Why

Such a key becomes:

```text
single catastrophic privacy backdoor
```

---

# 169. Safe Architecture

Moderation enforces capability-level sanctions without identity reveal.

---

# 170. Appeals and Due Process

For community moderation:

```text
notice
reason
duration
appeal path
```

where appropriate.

---

# 171. Anonymous Notice

Can be delivered via relationship/group mailbox.

---

# 172. Sanction Reason

Use structured category.

---

# 173. Avoid Excess Evidence Exposure

Do not show reporter identity.

---

# 174. Moderator Misuse

Threat:

```text
malicious moderator
```

---

# 175. Mitigations

```text
capability scope
audit log
threshold approval
appeals
policy transparency
```

---

# 176. Threshold Moderation

For severe actions:

```rust
pub struct ModerationQuorum {
    pub required: u16,
    pub total: u16,
}
```

---

# 177. Actions Requiring Quorum

Potential:

```text
permanent ban
mass deletion
owner removal
```

---

# 178. Ordinary Actions

Single moderator may handle:

```text
warning
temporary mute
```

---

# 179. Key Separation

Moderator signing keys separate from:

```text
user identity keys
group content keys
```

---

# 180. Moderator Key Rotation

Supported.

---

# 181. Compromised Moderator

Revoke capability.

---

# 182. Moderation State Machine

```rust
pub enum ReportState {
    Submitted,
    Triaged,
    Investigating,
    Resolved,
    Rejected,
    Appealed,
    Closed,
}
```

---

# 183. Sanction State

```rust
pub enum SanctionState {
    Proposed,
    Active,
    Expired,
    Revoked,
    Overturned,
}
```

---

# 184. Transactionality

When sanction activates:

```text
persist sanction
revoke capabilities
record audit
```

atomically/transactionally coordinated.

---

# 185. Crash Safety

No active sanction without corresponding durable record.

---

# 186. Idempotency

Repeated moderation action must not duplicate side effects.

---

# 187. Report Dedup

Same report resubmission safe.

---

# 188. Moderation Store

Potential tables:

```text
reputation_subjects
trust_attestations
abuse_reports
evidence_items
moderation_sanctions
moderation_appeals
moderation_audit
rate_limit_credentials
```

---

# 189. Secret References

Store capabilities by secure-store ref.

---

# 190. Data Retention

Moderation evidence has explicit TTL.

---

# 191. Legal Hold

If product supports it, separate explicit governance feature.

---

# 192. Default

No indefinite retention.

---

# 193. Deletion

Resolved benign report can be deleted earlier.

---

# 194. Negative Reputation Retention

Policy-defined.

---

# 195. Reputation Reset

Subject may obtain fresh pseudonym after leaving/rejoining depending policy.

---

# 196. Evasion Risk

Complete reset enables ban evasion.

---

# 197. Privacy vs Evasion

Needs scope-specific balance.

---

# 198. Rejoin Policy

```rust
pub enum RejoinPolicy {
    AllowedFresh,
    Cooldown,
    ModeratorApproval,
    Denied,
}
```

---

# 199. Ban-Evasion Token

Could maintain scope-local revoked credential digest.

---

# 200. No Global Device Ban by Default

Hard rule.

---

# 201. High-Risk Communities

May require stronger admission credentials.

---

# 202. Credential Types

Potential:

```text
invite
verified membership
organization credential
resource bond
```

---

# 203. Community Rules

Signed and versioned.

---

# 204. Rule Acknowledgement

User may accept before joining.

---

# 205. Rule Version

Stored locally.

---

# 206. Rule Change Notification

Privacy-safe.

---

# 207. Reputation Service Trait

```rust
pub trait ScopedReputationService {
    fn summary(
        &self,
        scope: ReputationScope,
        subject: ReputationSubjectId,
    ) -> Result<ReputationSummary, ReputationError>;

    fn add_attestation(
        &self,
        attestation: TrustAttestation,
    ) -> Result<(), ReputationError>;
}
```

---

# 208. Reporting Service

```rust
#[async_trait]
pub trait AbuseReportingService: Send + Sync {
    async fn submit(
        &self,
        report: AbuseReport,
    ) -> Result<AbuseReportReceipt, ReportError>;

    async fn status(
        &self,
        report_id: AbuseReportId,
    ) -> Result<ReportState, ReportError>;
}
```

---

# 209. Moderation Service

```rust
#[async_trait]
pub trait ModerationService: Send + Sync {
    async fn review(
        &self,
        report_id: AbuseReportId,
    ) -> Result<ModerationCase, ModerationError>;

    async fn apply(
        &self,
        sanction: ModerationSanction,
    ) -> Result<(), ModerationError>;

    async fn appeal(
        &self,
        appeal: ModerationAppeal,
    ) -> Result<(), ModerationError>;
}
```

---

# 210. Capability Revocation Service

```rust
pub trait ScopedCapabilityRevoker {
    fn revoke_subject(
        &self,
        scope: ReputationScope,
        subject: ReputationSubjectId,
    ) -> Result<(), CapabilityError>;
}
```

---

# 211. Observability

Privacy-safe metrics:

```text
reports submitted
reports upheld
sanctions active
appeals
abuse category buckets
```

---

# 212. Forbidden Metrics

No:

```text
AccountId
DeviceId
RelationshipId
message plaintext
reporter identity
```

---

# 213. Moderator Metrics

Aggregate by:

```text
community
policy version
```

only where safe.

---

# 214. Small Community Privacy

Avoid publishing public stats if low counts.

---

# 215. Support Bundle

Redact:

```text
report IDs if linkable
evidence
subject pseudonyms
moderator IDs
```

---

# 216. UI — Trust Signals

Display simple:

```text
Verified contact
New member
Established member
Restricted
```

---

# 217. UI — Report

Flow:

```text
choose category
select evidence
review privacy impact
submit
```

---

# 218. Evidence Preview

User sees what will be shared.

---

# 219. Reporter Privacy Choice

If policy allows:

```text
identified
pseudonymous
anonymous
```

---

# 220. UI — Moderation

Moderator sees only necessary context.

---

# 221. UI — Sanction

Show:

```text
action
duration
reason
policy reference
```

---

# 222. UI — Appeal

Structured appeal.

---

# 223. No Public Reputation Leaderboard

Hard rule.

---

# 224. No Shame Badges

Avoid permanent public negative labels.

---

# 225. Privacy Lab Integration

Part 42 should test whether moderation data can be used to correlate identities across scopes.

---

# 226. Cross-Scope Correlation Test

Same user in two communities should not be linkable via:

```text
subject ID
sanction ID
report metadata
```

---

# 227. Reporter Anonymity Test

Moderator cannot identify anonymous reporter from payload.

---

# 228. False Report Test

Many low-trust reports do not automatically trigger severe sanction.

---

# 229. Sybil Report Test

Thousands of fresh pseudonyms report one target.

---

# 230. Evidence Forgery Test

Tampered evidence rejected.

---

# 231. Capability Revocation Test

Banned subject cannot post after sanction.

---

# 232. Appeal Test

Overturned sanction restores allowed capabilities with fresh secrets if needed.

---

# 233. Moderator Abuse Test

Unauthorized moderator action rejected.

---

# 234. Threshold Test

Permanent sanction requires quorum where policy says so.

---

# 235. Expiry Test

Temporary sanction expires predictably.

---

# 236. Replay Test

Old sanction command cannot be replayed after revocation/epoch change.

---

# 237. Fuzzing

Fuzz:

```text
abuse report
evidence bundle
sanction
appeal
trust attestation
```

---

# 238. Property Tests

Properties:

```text
moderator cannot act outside scope
blocked subject cannot retain valid post capability
anonymous report contains no global account identifier
overturned sanction cannot remain active
```

---

# 239. Formal Verification Targets

Good candidates:

```text
report state machine
sanction lifecycle
moderator capability scope
appeal transition
```

---

# 240. TLA+ Candidate

Report + sanction + appeal workflow.

---

# 241. Kani Candidate

Capability scope validation.

---

# 242. Loom Candidate

Concurrent sanction activation and capability use.

---

# 243. Performance Tests

Measure:

```text
report submission
evidence verification
capability revocation
reputation summary lookup
```

---

# 244. Scale Tests

Synthetic communities:

```text
100
10,000
1,000,000
```

participants for metadata/service scaling, without real identities.

---

# 245. Abuse Load Tests

Large coordinated spam/report storms.

---

# 246. Privacy-Safe Rate Limiting Tests

Ensure anti-abuse controls do not require global identity.

---

# 247. Security Invariants

Mandatory:

```text
1. Reputation is scoped; no universal user score is required.
2. ReputationSubjectId is distinct across unrelated scopes.
3. Moderation does not require revealing global account identity.
4. Reports include minimum evidence necessary.
5. Anonymous reporters are not deanonymized by ordinary moderation flow.
6. Moderator actions are capability-scoped and auditable.
7. Severe sanctions can require quorum.
8. Sanctions enforce through scoped capability revocation.
9. Cross-scope reputation import is opt-in and privacy-reviewed.
10. False-report volume alone cannot automatically trigger severe sanctions.
11. Moderation evidence and capabilities never appear in telemetry/support bundles.
12. SIAR contains no universal deanonymization key for moderation.
```

---

# 248. Recommended Crate Layout

```text
crates/
├── siar-reputation-core/
├── siar-trust-signals/
├── siar-abuse-reporting/
├── siar-moderation-core/
├── siar-moderation-capability/
├── siar-moderation-evidence/
├── siar-moderation-policy/
├── siar-moderation-appeal/
├── siar-reputation-storage/
├── siar-moderation-observability/
└── siar-moderation-testkit/
```

---

# 249. `siar-reputation-core`

Owns:

```text
scope
subject IDs
reputation levels
errors
```

---

# 250. `siar-trust-signals`

Structured trust attestations.

---

# 251. `siar-abuse-reporting`

Report submission/status.

---

# 252. `siar-moderation-core`

Cases/sanctions/state machines.

---

# 253. `siar-moderation-capability`

Moderator/action capability enforcement.

---

# 254. `siar-moderation-evidence`

Evidence packaging/verification/storage.

---

# 255. `siar-moderation-policy`

Signed moderation policy.

---

# 256. `siar-moderation-appeal`

Appeal workflows.

---

# 257. `siar-reputation-storage`

Scoped persistence.

---

# 258. `siar-moderation-observability`

Privacy-safe metrics/audit.

---

# 259. `siar-moderation-testkit`

Synthetic abuse, false reports, malicious moderators.

---

# 260. Error Taxonomy

```rust
pub enum ModerationError {
    Unauthorized,
    InvalidEvidence,
    InvalidScope,
    ReportRateLimited,
    SanctionConflict,
    AppealUnavailable,
    QuorumInsufficient,
    CapabilityRevoked,
    PolicyMismatch,
    Internal,
}
```

---

# 261. Initial Production Scope

Implement first:

```text
scope-local subject IDs
structured trust signals
group/community-scoped reputation
anonymous/pseudonymous reporting
evidence bundles
moderator capabilities
warning/mute/post suspension/member removal
capability-based sanctions
report rate limits
appeal workflow
audit log
privacy-safe metrics
```

Then add:

```text
threshold moderation
portable privacy-reviewed attestations
anonymous credential proofs
privacy-preserving rate-limit tokens
differentially private transparency metrics
advanced coordinated-abuse detection
```

---

# 262. Definition of Done

Part 46 is complete when:

- trust and reputation are explicitly scoped
- no global score is required
- scope-local pseudonyms prevent unnecessary cross-context linking
- reports support identified, pseudonymous, and anonymous modes
- evidence is minimized, authenticatable, encrypted, and retention-bounded
- moderation works through scoped capabilities
- sanctions revoke capabilities instead of requiring identity deanonymization
- false-report and Sybil-report resistance are defined
- reporter privacy and moderator accountability coexist
- appeals and audit flows are explicit
- severe sanctions can use quorum
- trust attestations and portable reputation are privacy-reviewed
- cross-scope reputation sharing is opt-in
- observability excludes global identity and evidence content
- fuzz/property/formal/privacy tests are specified
- SIAR contains no universal deanonymization mechanism

---

# 263. Final Architecture

```text
                 TRUST / BEHAVIOR SIGNALS
                           │
                           ▼
                 SCOPED REPUTATION STATE
                           │
           ┌───────────────┴───────────────┐
           │                               │
      Local Trust                    Abuse Reports
                                           │
                                           ▼
                                    Evidence Review
                                           │
                                           ▼
                                  Moderator Capability
                                           │
                                           ▼
                                  Scoped Sanction
                                           │
                                           ▼
                                 Capability Revocation
```

Privacy-preserving accountability model:

```text
scope-local pseudonym
+
verifiable evidence
+
capability-based moderation
+
bounded reputation
+
appeals
+
audit
```

not:

```text
global identity
→ universal score
→ centralized blacklist
```

---

# 264. Final Principle

Privacy and moderation are not mutually exclusive if accountability is designed around **scope and capability**.

The correct model is:

```text
local/scoped reputation
+
minimal evidence
+
anonymous reporting
+
auditable moderator authority
+
capability revocation
+
appeals
```

rather than:

```text
identify everyone globally first
then moderate
```

This architecture gives SIAR a path to meaningful abuse resistance without sacrificing the identity separation, relationship privacy, and anonymity guarantees established in Parts 34–45.
