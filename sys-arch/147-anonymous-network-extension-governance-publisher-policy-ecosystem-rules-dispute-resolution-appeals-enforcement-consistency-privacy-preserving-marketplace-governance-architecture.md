# Core System Architecture Part 147 — Anonymous Network Extension Governance, Publisher Policy, Ecosystem Rules, Dispute Resolution, Appeals, Enforcement Consistency & Privacy-Preserving Marketplace Governance Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 147  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 53, 55–56, 69, 94–99, 104, 108, 119–126, 133–146

**Primary purpose:** define SIAR's extension ecosystem-governance architecture for publisher policy, marketplace rules, enforcement classes, appeals, dispute resolution, policy consistency, conflict-of-interest controls, sanctions, reinstatement, policy evolution, enterprise/private-marketplace overlays, and privacy-preserving governance.

---

# 1. Purpose

An extension ecosystem eventually needs more than technical controls.

It needs rules for:

```text
publisher conduct
marketplace eligibility
acceptable extension behavior
security/privacy expectations
distribution restrictions
enforcement
appeals
reinstatement
```

Without explicit governance, the ecosystem risks:

```text
arbitrary enforcement
publisher favoritism
opaque delisting
inconsistent sanctions
conflicted decision-makers
policy drift
surveillance-driven moderation
```

The governing principle is:

> **SIAR marketplace governance must be rule-based, scoped, evidence-backed, reviewable, appealable, and privacy-preserving, with consistent enforcement and no hidden behavioral ranking of publishers or users.**

---

# 2. Architectural Position

```text
                  ECOSYSTEM POLICY
                         │
                         ▼
                  PUBLISHER RULES
                         │
             ┌───────────┼───────────┐
             │           │           │
          SECURITY     PRIVACY     CONDUCT
             │           │           │
             └───────────┼───────────┘
                         ▼
                 ENFORCEMENT ENGINE
                         │
             ┌───────────┼───────────┐
             │           │           │
         WARNING       RESTRICT     SUSPEND
             │           │           │
             └───────────┼───────────┘
                         ▼
                 APPEAL / REVIEW
                         │
                         ▼
                FINAL / REINSTATED
```

---

# 3. Core Separation

Keep distinct:

```text
policy
rule
violation
incident
enforcement action
appeal
dispute
exception
reinstatement
```

---

# 4. Non-Goals

Part 147 does not create:

```text
arbitrary publisher scoring
political/content ideology ranking
user-behavior surveillance
secret enforcement rules
one-person marketplace dictatorship
```

---

# 5. Governance Policy Identity

```rust
pub struct ExtensionGovernancePolicyId(pub [u8; 16]);
```

---

# 6. Policy Version

```rust
pub struct ExtensionGovernancePolicyVersion(pub u64);
```

---

# 7. Policy Scope

```rust
pub enum ExtensionGovernanceScope {
    GlobalMarketplace,
    EnterpriseMarketplace(TenantId),
    PrivateRegistry(DependencyRegistryId),
}
```

---

# 8. Hard Rule

Scope explicit.

---

# 9. Policy Class

```rust
pub enum ExtensionGovernancePolicyClass {
    PublisherEligibility,
    Security,
    Privacy,
    DataGovernance,
    MarketplaceConduct,
    Distribution,
    Support,
    Compatibility,
    Lifecycle,
}
```

---

# 10. Hard Rule

Policy class does not imply enforcement severity automatically.

---

# 11. Rule Identity

```rust
pub struct ExtensionGovernanceRuleId(pub [u8; 16]);
```

---

# 12. Governance Rule

```rust
pub struct ExtensionGovernanceRule {
    pub id: ExtensionGovernanceRuleId,
    pub class: ExtensionGovernancePolicyClass,
    pub scope: ExtensionGovernanceScope,
    pub strength: GovernanceRuleStrength,
}
```

---

# 13. Rule Strength

```rust
pub enum GovernanceRuleStrength {
    Mandatory,
    RequiredUnlessException,
    Recommended,
    Informational,
}
```

---

# 14. Hard Rule

Mandatory privacy/security invariants cannot be converted to recommendation by lower policy.

---

# 15. Policy Precedence

```text
hard platform invariants
> signed ecosystem security/privacy policy
> marketplace governance policy
> enterprise/private marketplace policy
> publisher declaration
```

---

# 16. Hard Rule

Lower level cannot weaken higher-level invariant.

---

# 17. Publisher Identity

Publisher identity from Part 133.

---

# 18. Publisher Status

```rust
pub enum PublisherGovernanceState {
    Eligible,
    Restricted,
    Suspended,
    Revoked,
    UnderReview,
}
```

---

# 19. Hard Rule

Publisher state distinct from package state.

---

# 20. Package State

```rust
pub enum MarketplacePackageGovernanceState {
    Allowed,
    Restricted,
    Hidden,
    BlockedNewInstalls,
    Suspended,
    Revoked,
}
```

---

# 21. Hard Rule

Publisher sanction does not automatically imply all historical packages malicious.

---

# 22. Publisher Eligibility

Possible requirements:

```text
verified signing identity
contact channel
accepted marketplace terms
supply-chain compliance
security contact
support commitment
```

---

# 23. Hard Rule

Eligibility requirements are factual and explicit.

---

# 24. No Popularity Requirement

Hard rule.

---

# 25. No Revenue-Based Trust

Hard rule.

---

# 26. No Pay-to-Policy Advantage

Hard rule.

---

# 27. Publisher Profile

```rust
pub struct PublisherGovernanceProfile {
    pub publisher: PublisherId,
    pub status: PublisherGovernanceState,
    pub eligibility: Vec<PublisherEligibilityRecord>,
}
```

---

# 28. Hard Rule

Profile stores governance facts, not behavior score.

---

# 29. Marketplace Conduct Rules

May cover:

```text
misleading descriptions
permission misrepresentation
malicious updates
notification abuse
support abandonment
security disclosure obligations
```

---

# 30. Hard Rule

Rules target extension/publisher behavior, not user opinions.

---

# 31. Extension Behavior Rule

```rust
pub struct ExtensionBehaviorRule {
    pub rule_id: ExtensionGovernanceRuleId,
    pub prohibited_behavior: ExtensionBehaviorClass,
}
```

---

# 32. Behavior Class

```rust
pub enum ExtensionBehaviorClass {
    HiddenPrivilegeEscalation,
    UndeclaredDataExfiltration,
    MisleadingPermissionUx,
    CoreUiImpersonation,
    NotificationSpam,
    SandboxEscapeAttempt,
    RevocationBypass,
    SupplyChainViolation,
}
```

---

# 33. Hard Rule

Behavior classes map to concrete technical evidence.

---

# 34. No Vague “Bad Behavior” Rule

Hard rule.

---

# 35. Violation Identity

```rust
pub struct ExtensionGovernanceViolationId(pub [u8; 16]);
```

---

# 36. Violation Record

```rust
pub struct ExtensionGovernanceViolation {
    pub id: ExtensionGovernanceViolationId,
    pub rule: ExtensionGovernanceRuleId,
    pub subject: GovernanceSubject,
    pub evidence: Vec<EvidenceRef>,
    pub confidence: EvidenceConfidence,
}
```

---

# 37. Governance Subject

```rust
pub enum GovernanceSubject {
    Publisher(PublisherId),
    Extension(ExtensionId),
    Package(ExtensionPackageId),
}
```

---

# 38. Hard Rule

Violation target precise.

---

# 39. Evidence Standard

Possible levels:

```rust
pub enum GovernanceEvidenceStandard {
    TechnicalConfirmation,
    StrongCorroboration,
    Preponderance,
}
```

---

# 40. Hard Rule

Evidence standard chosen by enforcement class.

---

# 41. Severe Actions

Require stronger evidence.

---

# 42. Hard Rule

No permanent publisher revocation on weak single-source allegation absent immediate hard security necessity.

---

# 43. Emergency Exception

Part 146 may temporarily contain before full governance review.

---

# 44. Hard Rule

Emergency containment remains temporary/scoped until governance disposition.

---

# 45. Enforcement Identity

```rust
pub struct ExtensionEnforcementActionId(pub [u8; 16]);
```

---

# 46. Enforcement Class

```rust
pub enum ExtensionEnforcementClass {
    Notice,
    Warning,
    RemediationRequired,
    CapabilityRestriction,
    DistributionRestriction,
    PackageSuspension,
    PublisherRestriction,
    PublisherSuspension,
    Revocation,
}
```

---

# 47. Hard Rule

Enforcement is graduated where safe.

---

# 48. No “One Strike Everything Deleted”

Hard rule.

---

# 49. Enforcement Action

```rust
pub struct ExtensionEnforcementAction {
    pub id: ExtensionEnforcementActionId,
    pub subject: GovernanceSubject,
    pub class: ExtensionEnforcementClass,
    pub scope: ExtensionEnforcementScope,
    pub effective_at: Timestamp,
    pub expires_at: Option<Timestamp>,
}
```

---

# 50. Enforcement Scope

```rust
pub enum ExtensionEnforcementScope {
    PackageVersion(ExtensionPackageId),
    ExtensionAllVersions(ExtensionId),
    PublisherSelectedExtensions(BTreeSet<ExtensionId>),
    PublisherAllPackages(PublisherId),
}
```

---

# 51. Hard Rule

Scope must be no broader than evidence supports unless emergency hard-risk policy requires.

---

# 52. Distribution Restriction

Examples:

```text
block new installs
hide from discovery
require manual install
require admin approval
```

---

# 53. Hard Rule

Existing local data preserved.

---

# 54. Capability Restriction

May remove:

```text
network access
background execution
notification posting
high-risk data access
```

---

# 55. Hard Rule

Restrictions integrated with Part 135/146 grant epochs.

---

# 56. Package Suspension

Stops execution/distribution temporarily.

---

# 57. Hard Rule

Suspension distinct from revocation.

---

# 58. Revocation

Permanent distrust of specific package/certification key/state.

---

# 59. Hard Rule

Revocation cannot be silently reversed in-place.

---

# 60. Reinstatement

Creates new reviewed state.

---

# 61. Enforcement Decision State

```rust
pub enum GovernanceDecisionState {
    Draft,
    Reviewing,
    Approved,
    Active,
    Appealed,
    Stayed,
    Superseded,
    Closed,
}
```

---

# 62. Hard Rule

Draft decisions have no enforcement effect.

---

# 63. Decision Authority

```rust
pub enum GovernanceAuthorityRole {
    MarketplaceReviewer,
    SecurityReviewer,
    PrivacyReviewer,
    LegalReviewer,
    AppealsReviewer,
}
```

---

# 64. Hard Rule

Authority role scoped.

---

# 65. No Universal Marketplace Root

Hard rule.

---

# 66. Separation of Duties

For severe enforcement:

```text
investigator
decision-maker
appeals reviewer
```

should be distinct where feasible.

---

# 67. Hard Rule

Appeal not reviewed solely by original decision-maker for major sanction.

---

# 68. Conflict Of Interest

```rust
pub struct GovernanceConflictDeclaration {
    pub reviewer: ReviewerId,
    pub subject: GovernanceSubject,
    pub conflict_class: ConflictClass,
}
```

---

# 69. Conflict Class

```rust
pub enum ConflictClass {
    Financial,
    Employment,
    DirectCompetition,
    PriorMaterialInvolvement,
    OtherMaterialConflict,
}
```

---

# 70. Hard Rule

Conflicted reviewer recuses from final decision.

---

# 71. No Competitor-Driven Enforcement

Hard rule.

---

# 72. Policy Transparency

Marketplace publishes:

```text
current rule set
rule versions
enforcement classes
appeal process
```

subject to security-sensitive redactions.

---

# 73. Hard Rule

Secret unpublished rules cannot be basis for ordinary enforcement.

---

# 74. Security-Sensitive Detail

Detection implementation can remain restricted.

---

# 75. Hard Rule

Underlying prohibited behavior/rule still public enough to be understood.

---

# 76. Policy Change

```rust
pub struct ExtensionGovernancePolicyChange {
    pub from: ExtensionGovernancePolicyVersion,
    pub to: ExtensionGovernancePolicyVersion,
    pub changed_rules: Vec<ExtensionGovernanceRuleId>,
}
```

---

# 77. Hard Rule

Policy changes versioned and traceable.

---

# 78. Retroactivity

New rules normally apply prospectively.

---

# 79. Hard Rule

No retroactive punishment for previously permitted behavior unless continuing behavior currently violates a critical security/privacy invariant.

---

# 80. Transition Window

Non-critical policy changes may allow migration period.

---

# 81. Hard Rule

Transition period explicit.

---

# 82. Hard Security Rule

May take immediate effect.

---

# 83. Hard Rule

Immediate policy still needs signed version/communication.

---

# 84. Publisher Notification

When policy materially changes.

---

# 85. Hard Rule

Notification does not require tracking publisher behavior.

---

# 86. Policy Exception

```rust
pub struct GovernancePolicyException {
    pub exception_id: GovernanceExceptionId,
    pub rule: ExtensionGovernanceRuleId,
    pub subject: GovernanceSubject,
    pub expires_at: Timestamp,
    pub rationale: String,
}
```

---

# 87. Hard Rule

Exception only where rule permits.

---

# 88. Non-Waivable Rule

Examples:

```text
malicious package signature
sandbox bypass
hidden secret exfiltration
revoked artifact execution
```

---

# 89. Hard Rule

No exception for non-waivable rule.

---

# 90. Exception Expiry

Automatic.

---

# 91. Hard Rule

Expired exception cannot continue silently.

---

# 92. Enforcement Consistency

Need case-comparison tooling.

---

# 93. Case Similarity

Based on:

```text
violated rule
evidence class
scope
impact
remediation history
```

---

# 94. Hard Rule

Do not compare publisher popularity/revenue.

---

# 95. Governance Precedent Record

```rust
pub struct GovernancePrecedent {
    pub precedent_id: GovernancePrecedentId,
    pub rule: ExtensionGovernanceRuleId,
    pub enforcement_class: ExtensionEnforcementClass,
    pub rationale_digest: Digest,
}
```

---

# 96. Hard Rule

Precedent guides consistency, not automatic sentencing.

---

# 97. Consistency Check

Before severe action, compare analogous cases.

---

# 98. Hard Rule

Different outcome requires documented factual/policy distinction.

---

# 99. No Automated Sanction Scoring

Hard rule.

---

# 100. AI Assistance

May summarize:

```text
case history
policy text
evidence links
```

---

# 101. Hard Rule

AI cannot issue final sanction, approve appeal, or infer intent without human-governed evidence.

---

# 102. Dispute Identity

```rust
pub struct ExtensionGovernanceDisputeId(pub [u8; 16]);
```

---

# 103. Dispute Class

```rust
pub enum GovernanceDisputeClass {
    PublisherPolicyDispute,
    PackageOwnershipDispute,
    TrademarkNamingDispute,
    DependencyDispute,
    CertificationDispute,
    EnforcementDispute,
}
```

---

# 104. Hard Rule

Dispute class explicit.

---

# 105. Ownership Dispute

Package namespace ownership can be disputed.

---

# 106. Hard Rule

Marketplace freezes transfer-sensitive actions while unresolved if necessary.

---

# 107. No Destructive Transfer During Dispute

Hard rule.

---

# 108. Publisher Key / Identity Dispute

Handled separately from package functional review.

---

# 109. Hard Rule

Identity dispute does not automatically delete package data/install history.

---

# 110. Certification Dispute

Publisher may contest:

```text
test failure
compatibility result
evidence freshness
certification suspension
```

---

# 111. Hard Rule

Appeal must provide evidence, not merely assertion.

---

# 112. Appeal Identity

```rust
pub struct ExtensionAppealId(pub [u8; 16]);
```

---

# 113. Appeal State

```rust
pub enum ExtensionAppealState {
    Submitted,
    AdmissibilityReview,
    EvidenceReview,
    DecisionPending,
    Granted,
    PartiallyGranted,
    Denied,
    Withdrawn,
}
```

---

# 114. Hard Rule

Appeal lifecycle immutable/history-preserving.

---

# 115. Appeal Grounds

```rust
pub enum ExtensionAppealGround {
    FactualError,
    NewEvidence,
    PolicyMisapplication,
    ScopeDisproportionate,
    ProceduralError,
}
```

---

# 116. Hard Rule

Ground explicit.

---

# 117. Appeal Submission

```rust
pub struct ExtensionAppeal {
    pub appeal_id: ExtensionAppealId,
    pub enforcement: ExtensionEnforcementActionId,
    pub grounds: BTreeSet<ExtensionAppealGround>,
    pub evidence: Vec<EvidenceRef>,
}
```

---

# 118. Hard Rule

Appeal cannot mutate original decision record.

---

# 119. Appeal Stay

Some enforcement may be temporarily stayed.

---

# 120. Hard Rule

Severe active security/privacy containment generally remains until risk safely addressed.

---

# 121. Stay Decision

```rust
pub enum AppealStayDecision {
    NoStay,
    PartialStay,
    FullStay,
}
```

---

# 122. Hard Rule

Stay based on risk, not commercial pressure.

---

# 123. Appeals Reviewer

Independent role for major enforcement.

---

# 124. Hard Rule

No direct business/revenue incentive in appeal decision.

---

# 125. Appeal Decision

```rust
pub enum ExtensionAppealDecision {
    Uphold,
    NarrowScope,
    ReduceSanction,
    Vacate,
    RemandForReReview,
}
```

---

# 126. Hard Rule

Decision rationale references rules/evidence.

---

# 127. No Secret Rationale

Hard rule, except security-sensitive evidence details may be redacted.

---

# 128. Publisher Due Process

For non-emergency actions, publisher should receive:

```text
rule cited
evidence category
proposed action
response window
```

---

# 129. Hard Rule

Emergency containment can precede response opportunity when necessary.

---

# 130. Post-Emergency Review

Required.

---

# 131. Hard Rule

Emergency action does not bypass later governance review.

---

# 132. Response Window

Time-bounded.

---

# 133. Hard Rule

Critical security risk does not require waiting before containment.

---

# 134. Remediation Plan

Publisher can propose:

```text
fixed package
permission reduction
dependency replacement
data cleanup
support change
```

---

# 135. Hard Rule

Remediation verified before sanction reduction.

---

# 136. Publisher Reinstatement

```rust
pub enum PublisherReinstatementState {
    NotEligible,
    RemediationRequired,
    UnderReview,
    ReinstatedRestricted,
    Reinstated,
}
```

---

# 137. Reinstatement Criteria

May include:

```text
compromised key rotated
fixed package certified
open incident resolved
required support/security contact restored
```

---

# 138. Hard Rule

Reinstatement never automatically restores revoked package versions.

---

# 139. Package Reinstatement

New package/version may be accepted.

---

# 140. Hard Rule

Historical revoked artifact remains revoked.

---

# 141. Publisher Transfer

Ownership transfer is high-risk governance event.

---

# 142. Hard Rule

Transfer cannot silently carry all trust state.

---

# 143. Transfer Review

Check:

```text
new publisher identity
signing keys
support contact
policy acceptance
permission sensitivity
```

---

# 144. Hard Rule

Sensitive extensions may require user/admin notice after publisher change.

---

# 145. Marketplace Discovery Governance

Search/ranking rules should be explicit.

---

# 146. Hard Rule

No pay-to-rank hidden prioritization.

---

# 147. Ranking Inputs

Allowed:

```text
query relevance
platform compatibility
explicit category
user-selected filters
```

---

# 148. Hard Rule

Do not use invasive behavioral profiling.

---

# 149. Trust Signals

Factual signals only:

```text
verified publisher
certification status
permission class
update freshness
SBOM/provenance availability
```

---

# 150. Hard Rule

No one opaque trust score.

---

# 151. User Reviews

If supported, separate from governance.

---

# 152. Hard Rule

User-review popularity cannot override security/privacy enforcement.

---

# 153. Review Abuse

Moderated by separate anti-abuse process.

---

# 154. Hard Rule

No review-bombing-driven automatic delisting.

---

# 155. Marketplace Categories

Functional categories only.

---

# 156. Hard Rule

No sensitive-user-category targeting.

---

# 157. Private Marketplace Governance

Enterprise can define additional rules.

---

# 158. Hard Rule

Enterprise rules cannot weaken global security/privacy floor.

---

# 159. Enterprise Allowlist

May restrict available extensions.

---

# 160. Hard Rule

Allowlist does not bypass package verification/certification.

---

# 161. Private Publisher

Tenant-internal publisher may distribute privately.

---

# 162. Hard Rule

Private distribution still follows technical security guarantees.

---

# 163. Enterprise Appeals

Tenant may have separate internal process.

---

# 164. Hard Rule

Platform-level revocation still dominates tenant allowance.

---

# 165. Federation / Multiple Marketplaces

Future support possible.

---

# 166. Hard Rule

No transitive trust between marketplaces automatically.

---

# 167. Marketplace Trust Domain

```rust
pub struct MarketplaceTrustDomainId(pub [u8; 16]);
```

---

# 168. Hard Rule

Publisher status scoped to trust domain where appropriate.

---

# 169. Cross-Marketplace Package

Requires independent verification.

---

# 170. Hard Rule

Certification portability explicit.

---

# 171. Governance Audit

High-value events:

```text
policy change
enforcement approval
appeal decision
exception grant
publisher reinstatement
```

---

# 172. Hard Rule

Audit excludes private content.

---

# 173. Decision Record

```rust
pub struct GovernanceDecisionRecord {
    pub decision_id: GovernanceDecisionId,
    pub subject: GovernanceSubject,
    pub rules: Vec<ExtensionGovernanceRuleId>,
    pub evidence: Vec<EvidenceRef>,
    pub decision: ExtensionEnforcementClass,
    pub rationale_digest: Digest,
}
```

---

# 174. Hard Rule

Decision record immutable; corrections via supersession.

---

# 175. Public Transparency Record

Can disclose:

```text
rule category
action type
affected package/version
status
```

where appropriate.

---

# 176. Hard Rule

No user/tenant identities.

---

# 177. Transparency Reporting

Aggregate periodic reporting may show:

```text
number of suspensions
number of appeals
appeal outcomes
policy changes
```

---

# 178. Hard Rule

Aggregate only.

---

# 179. No Publisher Scoreboard

Hard rule.

---

# 180. Governance Metrics

Safe metrics:

```text
decision latency
appeal latency
policy exception count
reinstatement duration
```

---

# 181. Hard Rule

Process-quality metrics, not publisher/developer performance scoring.

---

# 182. Enforcement Latency SLO

Severe security actions may have response target.

---

# 183. Hard Rule

Faster is not always better for non-emergency due process.

---

# 184. Appeal SLO

Timely review.

---

# 185. Hard Rule

No automatic grant/deny due to timer expiry unless policy explicitly says so.

---

# 186. Policy Consistency SLO

Track unresolved analogous-case divergence.

---

# 187. Hard Rule

No numeric “fairness score”.

---

# 188. Governance Review Board

Optional multi-role body.

---

# 189. Hard Rule

Board authority defined by policy.

---

# 190. No Hidden Ad-Hoc Board Powers

Hard rule.

---

# 191. Quorum

For high-impact governance changes.

---

# 192. Governance Quorum

```rust
pub struct GovernanceQuorumPolicy {
    pub required_roles: BTreeSet<GovernanceAuthorityRole>,
    pub minimum_approvals: u8,
}
```

---

# 193. Hard Rule

Quorum cannot be lowered during dispute for convenience.

---

# 194. Emergency Quorum

May be narrower for temporary containment.

---

# 195. Hard Rule

Permanent sanction still requires ordinary review path afterward.

---

# 196. Policy Signing

Governance policies signed.

---

# 197. Hard Rule

Clients reject unsigned governance policy.

---

# 198. Policy Anti-Rollback

Monotonic epoch/version.

---

# 199. Hard Rule

Old permissive policy cannot replace newer restrictive rule set silently.

---

# 200. Policy Distribution

Part 61/99 patterns.

---

# 201. Hard Rule

Offline clients retain last valid restrictive policy.

---

# 202. Policy Staleness

For new installs, stale policy may block high-risk operation.

---

# 203. Hard Rule

No permissive fail-open.

---

# 204. Governance API

```rust
pub trait ExtensionGovernanceService {
    fn effective_policy(
        &self,
        scope: ExtensionGovernanceScope,
    ) -> Result<ExtensionGovernancePolicySnapshot, ExtensionGovernanceError>;
}
```

---

# 205. Enforcement Service

```rust
pub trait ExtensionEnforcementService {
    fn propose(
        &self,
        violation: ExtensionGovernanceViolation,
    ) -> Result<ExtensionEnforcementActionId, ExtensionGovernanceError>;

    fn activate(
        &self,
        action: ExtensionEnforcementActionId,
    ) -> Result<(), ExtensionGovernanceError>;
}
```

---

# 206. Appeal Service

```rust
pub trait ExtensionAppealService {
    fn submit(
        &self,
        appeal: ExtensionAppeal,
    ) -> Result<ExtensionAppealId, ExtensionGovernanceError>;

    fn decide(
        &self,
        appeal: ExtensionAppealId,
        decision: ExtensionAppealDecision,
    ) -> Result<(), ExtensionGovernanceError>;
}
```

---

# 207. Reinstatement Service

```rust
pub trait ExtensionReinstatementService {
    fn evaluate_publisher(
        &self,
        publisher: PublisherId,
    ) -> Result<PublisherReinstatementState, ExtensionGovernanceError>;
}
```

---

# 208. Consistency Service

```rust
pub trait GovernanceConsistencyService {
    fn analogous_cases(
        &self,
        violation: ExtensionGovernanceViolationId,
    ) -> Result<Vec<GovernancePrecedent>, ExtensionGovernanceError>;
}
```

---

# 209. Error Taxonomy

```rust
pub enum ExtensionGovernanceError {
    PolicyUnknown,
    RuleUnknown,
    SubjectUnknown,
    EvidenceInsufficient,
    ConflictOfInterest,
    UnauthorizedDecision,
    QuorumNotMet,
    ExceptionNotAllowed,
    AppealNotAdmissible,
    ReinstatementNotReady,
    PolicyStale,
    RevokedSubject,
    Internal,
}
```

---

# 210. Policy Storage

Separate:

```text
policy documents
rule registry
precedent records
decision records
appeals
exceptions
reinstatement records
```

---

# 211. Hard Rule

No user behavior data in governance store.

---

# 212. Evidence References

Governance store references incident/assurance evidence; does not duplicate private blobs unnecessarily.

---

# 213. Hard Rule

Evidence access remains source-policy controlled.

---

# 214. Retention

Governance decisions retained long-term for precedent/accountability.

---

# 215. Hard Rule

Sensitive raw evidence may have shorter retention than decision metadata.

---

# 216. Public vs Internal Record

Separate.

---

# 217. Hard Rule

Public transparency does not leak restricted evidence.

---

# 218. Testing

Need extension-governance testkit.

Required scenarios:

```text
warning
package suspension
publisher suspension
appeal
reinstatement
conflict-of-interest recusal
```

---

# 219. Evidence Test

Severe sanction rejected if evidence standard unmet.

---

# 220. Scope Test

Package-level evidence cannot automatically sanction unrelated publisher packages.

---

# 221. Exception Test

Non-waivable rule rejects exception.

---

# 222. Appeal Independence Test

Original severe-action approver cannot be sole appeals reviewer.

---

# 223. Quorum Test

High-impact decision fails if quorum not met.

---

# 224. Consistency Test

Analogous case divergence requires documented rationale.

---

# 225. Retroactivity Test

New non-critical rule does not punish already-ended permitted behavior.

---

# 226. Transfer Test

Publisher ownership transfer triggers trust re-evaluation.

---

# 227. Enterprise Test

Tenant policy cannot override global package revocation.

---

# 228. Privacy Test

Governance record has no user behavior fields.

---

# 229. Fuzzing

Fuzz:

```text
policy documents
appeal records
exception records
enforcement scopes
decision records
```

---

# 230. Property Tests

Properties:

```text
lower policy can never weaken hard platform invariant
publisher sanction can never implicitly revoke unrelated package without scoped action
appeal can never mutate original decision record
revoked package can never become allowed through lower marketplace policy
```

---

# 231. Formal Verification Targets

Strong candidates:

```text
policy precedence
enforcement lifecycle
appeal/reinstatement transitions
quorum/SoD rules
```

---

# 232. Kani Candidate

scope/precedence/exception invariants.

---

# 233. TLA+ Candidate

```text
rule → violation → enforcement → appeal → uphold/modify/vacate → reinstate
```

---

# 234. Loom Candidate

Concurrent:

```text
enforcement activation
appeal stay
emergency disable
policy refresh
```

---

# 235. Performance

Governance is control-plane work.

---

# 236. Hard Rule

No governance evaluation on message send/runtime hot path except cached enforcement state.

---

# 237. Enforcement Cache

Signed local snapshot.

---

# 238. Hard Rule

Revocation/suspension updates invalidate cache promptly.

---

# 239. Policy Compilation

Human-readable governance policy:

```text
Markdown explanation
RON typed metadata
Postcard compiled policy
```

---

# 240. Hard Rule

Compiled policy digest binds human source/version.

---

# 241. Policy Review Workflow

```text
draft
review
security/privacy check
approve
sign
publish
```

---

# 242. Hard Rule

No direct production mutation.

---

# 243. Governance Workspace

Recommended repository structure:

```text
governance/
├── policies/
├── rules/
├── precedents/
├── appeals/
├── transparency/
└── schemas/
```

---

# 244. Hard Rule

Sensitive evidence remains outside public repository.

---

# 245. Crate Layout

Recommended:

```text
crates/
├── siar-extension-governance-core/
├── siar-extension-policy-registry/
├── siar-extension-enforcement/
├── siar-extension-appeals/
├── siar-extension-disputes/
├── siar-extension-precedent/
├── siar-extension-reinstatement/
├── siar-extension-governance-policy/
├── siar-extension-governance-observability/
└── siar-extension-governance-testkit/
```

---

# 246. `siar-extension-governance-core`

Owns:

```text
ExtensionGovernancePolicyId
ExtensionGovernanceRuleId
ExtensionGovernanceViolationId
ExtensionGovernanceError
```

---

# 247. `siar-extension-policy-registry`

Versioned rules/scopes/strengths/precedence.

---

# 248. `siar-extension-enforcement`

Graduated enforcement actions/scope/activation.

---

# 249. `siar-extension-appeals`

Appeal lifecycle/stays/independent review.

---

# 250. `siar-extension-disputes`

Ownership/certification/policy dispute handling.

---

# 251. `siar-extension-precedent`

Comparable-case retrieval and consistency review.

---

# 252. `siar-extension-reinstatement`

Publisher/package return-to-good-standing flow.

---

# 253. `siar-extension-governance-policy`

Signed policy compilation/distribution/anti-rollback.

---

# 254. `siar-extension-governance-observability`

Aggregate governance-process metrics only.

---

# 255. `siar-extension-governance-testkit`

scope/appeal/quorum/privacy/formal tests.

---

# 256. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. Governance policy, rule, violation, incident, enforcement action, appeal, dispute, exception, reinstatement, and marketplace trust state are distinct typed records with explicit scope and immutable history.
2. Enforcement is evidence-backed, rule-cited, and no broader than the demonstrated risk where safe; a package-level violation cannot silently sanction unrelated publisher packages, tenants, users, or extensions without separate scoped authority.
3. Publisher popularity, revenue, advertising spend, app-store ranking, user engagement, or commercial relationship can never grant policy advantage, bypass technical/security rules, determine appeal outcome, or substitute for evidence.
4. Security/privacy hard invariants are non-waivable and dominate marketplace/enterprise/publisher policy; tenant/private marketplaces may restrict further but can never weaken platform revocation, sandbox, cryptographic, privacy, or data-isolation guarantees.
5. Emergency containment may act before ordinary process when necessary, but remains signed, scoped, auditable, and subject to later governance review; temporary emergency authority cannot silently become permanent governance power.
6. Severe enforcement uses separation of duties, conflict-of-interest recusal, and quorum where applicable; an investigator, original decision-maker, conflicted competitor, or revenue owner cannot unilaterally control major sanction and final appeal.
7. Appeals preserve original decision history, cite explicit grounds/evidence, receive independent review for major sanctions, and may narrow/vacate/remand decisions without deleting the historical record or auto-restoring revoked artifacts.
8. Publisher/package reinstatement requires verified remediation, fresh security/privacy/certification evidence, current valid signing identity, and resolved incidents; historical revoked package artifacts remain revoked even after publisher reinstatement.
9. Policy changes are signed, versioned, anti-rollback protected, and normally prospective; ordinary enforcement cannot rely on hidden unpublished rules or retroactively punish completed behavior that was permitted when performed.
10. Marketplace discovery, trust signals, policy enforcement, appeals, user reviews, and transparency reporting are separated from invasive behavioral profiling; no governance process may require user activity timelines, private content, sensitive inferred traits, or publisher/user scoring.
11. Governance metrics and transparency records measure process outcomes—decision latency, appeals, policy changes, enforcement classes—without becoming publisher scoreboards, developer productivity analytics, workforce surveillance, or user-level tracking.
12. Extension governance integrates with marketplace distribution, runtime enforcement, permissions, data governance, state sync, dependencies, updates, certification, observability, incident response, publisher identity, enterprise policy, audit, and evidence archives without creating a side channel around SIAR's security, privacy, anonymity, local-first, or tenant-isolation guarantees.
```

---

# 257. Initial Production Scope

Implement first:

```text
typed governance policy/rule registry
publisher/package governance states
explicit conduct/security/privacy rules
graduated enforcement classes
scoped enforcement actions
decision authority roles
conflict-of-interest records
quorum for severe actions
publisher notification/remediation flow
appeal lifecycle
independent appeals review
policy exceptions with expiry
non-waivable rule support
precedent/case consistency records
publisher/package reinstatement
enterprise marketplace overlays
signed governance policy distribution
anti-rollback policy epoch
transparency records
privacy-safe governance metrics
extension-governance testkit
```

Then add:

```text
cross-marketplace governance federation
privacy-preserving transparency proofs
structured public precedent summaries
automated policy consistency linting
formal quorum/enforcement proofs
publisher transparency reporting
```

---

# 258. Definition of Done

Part 147 is complete when:

- governance policy/rules are versioned and signed;
- publisher/package state is distinct;
- violations are evidence-backed and scoped;
- enforcement is graduated;
- severe sanctions require SoD/quorum where appropriate;
- appeals preserve history and receive independent review;
- conflicts of interest require recusal;
- policy exceptions are bounded/non-waivable where required;
- reinstatement requires verified remediation;
- enterprise/private policy cannot weaken platform floor;
- governance records contain no user-behavior surveillance data;
- policy/scope/appeal/quorum/privacy/fuzz/formal tests are specified.

---

# 259. Final Architecture

```text
                 GOVERNANCE POLICY
                        │
                        ▼
                 PUBLISHER / PACKAGE
                        │
                        ▼
                   RULE EVALUATION
                        │
             ┌──────────┼──────────┐
             │          │          │
          NOTICE      RESTRICT    SUSPEND
             │          │          │
             └──────────┼──────────┘
                        ▼
                   DECISION RECORD
                        │
                        ▼
                 APPEAL / DISPUTE
                        │
                        ▼
             UPHOLD / MODIFY / VACATE
                        │
                        ▼
                  REINSTATEMENT
```

Marketplace-governance safety model:

```text
published rules
+
scoped evidence
+
graduated enforcement
+
separation of duties
+
appeals
+
precedent consistency
+
anti-rollback policy
+
privacy-safe transparency
```

not:

```text
opaque rules, ad-hoc sanctions, pay-to-play exceptions, popularity-based enforcement, and user surveillance used as marketplace governance
```

---

# 260. Final Principle

Extension governance is trustworthy when publishers and users can understand **which rule applies, what evidence supports an action, who had authority to decide it, how the decision can be appealed, and what must happen for reinstatement**.

The correct model is:

```text
publish clear rules
+
apply them to concrete evidence
+
scope enforcement narrowly
+
separate investigators from appeal authority
+
handle conflicts of interest
+
preserve decision history
+
allow structured appeals
+
verify remediation before reinstatement
+
never turn marketplace governance into commercial favoritism or surveillance
```

This architecture gives SIAR a privacy-preserving marketplace-governance foundation for publisher policy, ecosystem rules, enforcement, disputes, appeals, consistency, policy exceptions, reinstatement, enterprise overlays, and transparency while preserving the anonymity, local-first, least-authority, marketplace, runtime, permission, data-governance, update, certification, observability, incident-response, and anti-surveillance guarantees established across Parts 34–146.
