# Core System Architecture Part 95 — Anonymous Network Policy Compliance Engine, Continuous Control Evaluation, Security Posture, Evidence Collection & Privacy-Preserving Assurance Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 95  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 42, 51, 53, 55–56, 61–63, 69–73, 78–83, 92–94

**Primary purpose:** define SIAR's policy-compliance, continuous-control, security-posture, evidence-collection, assurance, exception, remediation, attestation, tenant/operator/federation assurance, audit integration, and privacy-preserving control architecture so technical and organizational controls can be continuously evaluated without creating broad user-level surveillance.

---

# 1. Purpose

A production anonymous network needs more than policies and audit logs.

It needs a way to answer:

```text
Are required controls actually active?
Are hosts running approved artifacts?
Are privacy floors enforced?
Are secrets rotated on time?
Are tenant boundaries intact?
Are audit chains healthy?
Are backups restorable?
Are risky exceptions still open?
```

The governing principle is:

> **SIAR compliance must evaluate technical controls against verifiable evidence while collecting only the minimum infrastructure and administrative data required to prove those controls.**

---

# 2. Architectural Position

```text
Policy / Standard / Internal Requirement
                  │
                  ▼
             Control Catalog
                  │
                  ▼
          Control Evaluators
                  │
      ┌───────────┼───────────┐
      │           │           │
  Runtime      Evidence     Attestation
  State         Stores        Inputs
      │           │           │
      └───────────┼───────────┘
                  ▼
            Posture Engine
                  │
        ┌─────────┼─────────┐
        │         │         │
     Pass      Exception   Fail
        │         │         │
        └─────────┼─────────┘
                  ▼
          Remediation / Assurance
```

---

# 3. Core Separation

Keep distinct:

```text
policy
control
evidence
evaluation result
exception
remediation
posture
attestation
audit
```

---

# 4. Non-Goals

Part 95 does not create:

```text
a universal compliance super-database
continuous inspection of private messages
user-behavior compliance scoring
employee surveillance
policy-by-spreadsheet with no executable semantics
```

---

# 5. Policy vs Control

Policy states:

```text
what must be true
```

Control states:

```text
how that requirement is enforced or verified
```

---

# 6. Example

Policy:

```text
Production workloads must use approved signed artifacts.
```

Control:

```text
Verify workload measurement matches an artifact in the approved release manifest.
```

---

# 7. Hard Rule

Policy text and executable control logic remain linked but distinct.

---

# 8. Control ID

```rust
pub struct ControlId(pub [u8; 16]);
```

---

# 9. Policy Requirement ID

```rust
pub struct RequirementId(pub [u8; 16]);
```

---

# 10. Control Mapping

```rust
pub struct ControlMapping {
    pub control: ControlId,
    pub requirements: Vec<RequirementId>,
}
```

---

# 11. No Free-Form Mapping Only

Preferred hard rule.

---

# 12. Control Domains

```rust
pub enum ControlDomain {
    Identity,
    Authorization,
    Cryptography,
    Secrets,
    HostIntegrity,
    Deployment,
    Network,
    DataLifecycle,
    Privacy,
    TenantIsolation,
    Federation,
    Audit,
    BackupRecovery,
    SupplyChain,
    Operations,
}
```

---

# 13. Control Catalog

Versioned and signed.

---

# 14. Control Definition

```rust
pub struct ControlDefinition {
    pub id: ControlId,
    pub domain: ControlDomain,
    pub title: ControlTitle,
    pub severity: ControlSeverity,
    pub evaluator: ControlEvaluatorRef,
    pub evidence_requirements: Vec<EvidenceRequirement>,
    pub remediation: RemediationRef,
}
```

---

# 15. Control Severity

```rust
pub enum ControlSeverity {
    Informational,
    Low,
    Medium,
    High,
    Critical,
}
```

---

# 16. Critical Controls

Examples:

```text
privacy floor enabled
approved crypto suite
production artifact verified
tenant isolation enforced
root key protected
```

---

# 17. Hard Rule

Critical control failure cannot be hidden by aggregate scoring.

---

# 18. Control State

```rust
pub enum ControlState {
    Pass,
    Fail,
    Unknown,
    NotApplicable,
    ExceptionApproved,
}
```

---

# 19. Unknown

Not enough valid evidence.

---

# 20. Hard Rule

Unknown is never treated as Pass.

---

# 21. NotApplicable

Requires explicit applicability rule.

---

# 22. No Manual "N/A" Without Reason

Hard rule.

---

# 23. ExceptionApproved

Temporary accepted deviation.

---

# 24. Never equivalent to Pass.

---

# 25. Hard rule.

---

# 26. Control Evaluation Result

```rust
pub struct ControlEvaluation {
    pub control: ControlId,
    pub state: ControlState,
    pub evidence: Vec<EvidenceRef>,
    pub evaluated_at: CoarseTimestamp,
    pub evaluator_version: EvaluatorVersion,
}
```

---

# 27. Evidence References

Minimal/scoped.

---

# 28. No raw private content.

---

# 29. Hard rule.

---

# 30. Evidence Types

```rust
pub enum EvidenceType {
    Configuration,
    ArtifactMeasurement,
    Attestation,
    AuditReceipt,
    KeyMetadata,
    PolicySnapshot,
    RuntimeHealth,
    BackupVerification,
    TestResult,
    DeploymentManifest,
}
```

---

# 31. Configuration Evidence

Signed config/policy digest.

---

# 32. Artifact Measurement

Binary/content digest.

---

# 33. Attestation

Host/workload evidence.

---

# 34. Audit Receipt

Part 94.

---

# 35. Key Metadata

Key class/rotation state, never secret.

---

# 36. Policy Snapshot

Signed policy version.

---

# 37. Runtime Health

Bounded infrastructure state.

---

# 38. Backup Verification

Restore-test evidence.

---

# 39. Test Result

Release/control validation.

---

# 40. Deployment Manifest

Artifact/topology record.

---

# 41. Hard Rule

Compliance evidence references proof, not secret payloads.

---

# 42. Evidence Object

```rust
pub struct EvidenceObject {
    pub evidence_id: EvidenceId,
    pub evidence_type: EvidenceType,
    pub source: EvidenceSource,
    pub digest: Digest,
    pub collected_at: CoarseTimestamp,
    pub expires_at: Option<Timestamp>,
}
```

---

# 43. Evidence ID

Opaque.

---

# 44. Source

Scoped workload/service/authority.

---

# 45. No Global User Identity

Hard rule.

---

# 46. Evidence Freshness

Critical.

---

# 47. Evidence Requirement

```rust
pub struct EvidenceRequirement {
    pub evidence_type: EvidenceType,
    pub max_age: Duration,
    pub trust_level: EvidenceTrustLevel,
}
```

---

# 48. Stale Evidence

Cannot satisfy current control.

---

# 49. Hard rule.

---

# 50. Evidence Trust Level

```rust
pub enum EvidenceTrustLevel {
    SelfReported,
    SignedService,
    AttestedWorkload,
    HardwareBacked,
    MultiPartyVerified,
}
```

---

# 51. Control Defines Minimum

Example:

```text
root key control requires HardwareBacked/MultiPartyVerified
```

---

# 52. Self-Reported Evidence

Useful for low-risk controls only.

---

# 53. Hard rule.

---

# 54. Continuous Evaluation

Controls evaluated periodically or on relevant change.

---

# 55. Evaluation Trigger

```rust
pub enum EvaluationTrigger {
    Periodic,
    PolicyChanged,
    DeploymentChanged,
    KeyChanged,
    HostChanged,
    EvidenceUpdated,
    Incident,
}
```

---

# 56. Event-Driven Preferred

For high-risk state changes.

---

# 57. Periodic Safety Net

Still required.

---

# 58. Hard Rule

Critical controls cannot depend only on once-a-year manual review.

---

# 59. Evaluation Schedule

```rust
pub struct ControlSchedule {
    pub trigger: EvaluationTrigger,
    pub max_interval: Duration,
}
```

---

# 60. Security Posture

Aggregate view of control states.

---

# 61. Posture Is Not One Score

Hard rule.

---

# 62. Why

A single numeric score can hide critical failures.

---

# 63. Posture Model

```rust
pub struct SecurityPosture {
    pub critical_failures: Vec<ControlId>,
    pub high_failures: Vec<ControlId>,
    pub exceptions: Vec<ControlId>,
    pub unknown: Vec<ControlId>,
}
```

---

# 64. Optional Summary

Can show percentages by domain.

---

# 65. But

Critical failures always explicit.

---

# 66. Hard rule.

---

# 67. Posture Domains

```text
identity
crypto
secrets
host
deployment
privacy
tenant isolation
audit
backup
supply chain
```

---

# 68. User Privacy Posture

Separate from infrastructure compliance.

---

# 69. Examples:

```text
max anonymity configured
telemetry disabled
external fetch disabled
backup recovery configured
```

---

# 70. User-Visible Privacy Posture

Local only.

---

# 71. Never central product scoring.

---

# 72. Hard rule.

---

# 73. Tenant Posture

Managed organization controls only.

---

# 74. Organization Can See

```text
managed device compliance
managed policy enforcement
tenant key posture
managed backup state
```

---

# 75. Organization Cannot See

```text
personal contacts
personal messages
personal social memberships
personal feed/search behavior
```

---

# 76. Hard rule.

---

# 77. Operator Posture

Infrastructure-only.

---

# 78. Federation Posture

Peer trust/control state.

---

# 79. No remote user posture exchange.

---

# 80. Hard rule.

---

# 81. Applicability Engine

Determines which controls apply.

---

# 82. Applicability Context

```rust
pub struct ApplicabilityContext {
    pub service_class: ServiceClass,
    pub environment: EnvironmentClass,
    pub tenant_mode: TenantMode,
    pub federation_enabled: bool,
}
```

---

# 83. No Sensitive User Attribute

Hard rule.

---

# 84. Applicability Rule

Static/typed.

---

# 85. No arbitrary scripts.

---

# 86. Hard rule.

---

# 87. Control Evaluator

```rust
pub trait ControlEvaluator {
    fn evaluate(
        &self,
        control: &ControlDefinition,
        context: &EvaluationContext,
    ) -> Result<ControlEvaluation, ComplianceError>;
}
```

---

# 88. Deterministic where possible.

---

# 89. No external arbitrary shell command in production evaluator.

---

# 90. Hard rule.

---

# 91. Evaluator Version

Versioned.

---

# 92. Control Re-Evaluated After Evaluator Upgrade

Required for impacted controls.

---

# 93. No Silent Semantic Change

Hard rule.

---

# 94. Policy Compiler

Transforms signed policy to machine-evaluable constraints.

---

# 95. Inputs:

```text
governance policy
managed policy
security baseline
release policy
```

---

# 96. Output:

```text
compiled control parameters
```

---

# 97. Hard Rule

Compiled policy cannot weaken hardcoded security/privacy invariants.

---

# 98. Precedence

Recommended:

```text
compiled hard invariants
> governance/security policy
> legal/compliance constraints
> tenant managed restrictions
> service defaults
```

---

# 99. No operational config override above privacy floor.

---

# 100. Hard rule.

---

# 101. Security Baseline

Versioned.

---

# 102. Example:

```rust
pub struct SecurityBaselineVersion(pub u64);
```

---

# 103. Baseline Maps To Controls

---

# 104. Older baseline can be deprecated.

---

# 105. No automatic rollback to weaker baseline.

---

# 106. Hard rule.

---

# 107. Control Evidence Pipeline

```text
source produces evidence
→ evidence validator verifies
→ evidence store records metadata
→ control evaluator consumes
→ posture engine updates
```

---

# 108. Evidence Validator

```rust
pub trait EvidenceValidator {
    fn validate(
        &self,
        evidence: &EvidenceObject,
    ) -> Result<VerifiedEvidence, ComplianceError>;
}
```

---

# 109. Signature/attestation checks.

---

# 110. Unknown/invalid signature

evidence rejected.

---

# 111. Hard rule.

---

# 112. Evidence Provenance

Must answer:

```text
who produced it
what artifact/policy it refers to
which verification method
when it expires
```

---

# 113. No Unsourced Evidence

Hard rule.

---

# 114. Evidence Digest

Canonical.

---

# 115. No mutable evidence object without new digest.

---

# 116. Hard rule.

---

# 117. Evidence Store

Metadata/proof-oriented.

---

# 118. Do Not Store

```text
private message bodies
search terms
contact lists
full memory dumps
```

---

# 119. Hard rule.

---

# 120. Evidence Minimization

Store:

```text
digest
state
scope
signature
timestamp
```

instead of raw source where possible.

---

# 121. Raw evidence retention

Short/bounded.

---

# 122. Hard rule.

---

# 123. Evidence Retention Class

```rust
pub enum EvidenceRetentionClass {
    Short,
    Standard,
    LongCritical,
    LegalScoped,
}
```

---

# 124. Critical governance evidence

Longer.

---

# 125. Runtime posture evidence

Shorter.

---

# 126. No forever default.

---

# 127. Hard rule.

---

# 128. Control Exceptions

Temporary deviation.

---

# 129. Exception Record

```rust
pub struct ControlException {
    pub exception_id: ExceptionId,
    pub control: ControlId,
    pub scope: ExceptionScope,
    pub justification_code: ExceptionReasonCode,
    pub approved_by: ApprovalBundle,
    pub expires_at: Timestamp,
    pub compensating_controls: Vec<ControlId>,
}
```

---

# 130. Expiration Required

Hard rule.

---

# 131. No Permanent Exception

Default hard rule.

---

# 132. Exception Scope

Narrow.

---

# 133. Examples:

```text
one service
one host class
one tenant
one environment
```

---

# 134. No "all production" unless governance explicitly permits.

---

# 135. Hard rule.

---

# 136. Justification

Structured reason code + bounded note if needed.

---

# 137. No free-form sensitive data.

---

# 138. Hard rule.

---

# 139. Compensating Controls

Optional.

---

# 140. Example

If one control temporarily unavailable:

```text
extra monitoring
network isolation
manual approval
```

---

# 141. Exception Does Not Change Original Control Definition

Hard rule.

---

# 142. Exception Approval

High-severity controls may require multi-party approval.

---

# 143. Part 94 evidence.

---

# 144. Exception Usage Is Audited

---

# 145. No hidden exception.

---

# 146. Hard rule.

---

# 147. Exception Expiry

Automatic.

---

# 148. On expiry

control becomes Fail/Unknown unless fixed.

---

# 149. No silent renewal.

---

# 150. Hard rule.

---

# 151. Remediation

Every actionable control can define remediation.

---

# 152. Remediation Types

```rust
pub enum RemediationType {
    Manual,
    Guided,
    AutomatedSafe,
    AutomatedRestricted,
}
```

---

# 153. AutomatedSafe

Examples:

```text
restart failed collector
refresh signed policy
rotate low-risk credential
```

---

# 154. AutomatedRestricted

Requires approval.

---

# 155. Hard Rule

Automation cannot perform high-risk destructive remediation without explicit authorization.

---

# 156. Remediation Plan

```rust
pub struct RemediationPlan {
    pub control: ControlId,
    pub steps: Vec<RemediationStep>,
    pub rollback: Option<RollbackPlanRef>,
}
```

---

# 157. No Raw Shell Script In Domain Model

Hard rule.

---

# 158. Typed Remediation Commands

Preferred.

---

# 159. Remediation State

```rust
pub enum RemediationState {
    Proposed,
    Approved,
    Running,
    Succeeded,
    Failed,
    RolledBack,
}
```

---

# 160. Evidence After Remediation

Must re-evaluate control.

---

# 161. Success message alone insufficient.

---

# 162. Hard rule.

---

# 163. Control Drift

Previously compliant control becomes noncompliant.

---

# 164. Drift Sources:

```text
config change
artifact drift
certificate expiry
key age
policy update
host change
dependency vulnerability
```

---

# 165. Drift Detector

Event-driven.

---

# 166. Hard rule.

---

# 167. Drift Event

```rust
pub struct ControlDriftEvent {
    pub control: ControlId,
    pub previous: ControlState,
    pub current: ControlState,
    pub detected_at: CoarseTimestamp,
}
```

---

# 168. Critical Drift

High-priority alert.

---

# 169. No alert flood for repeated same state.

---

# 170. Deduplicate.

---

# 171. Hard rule.

---

# 172. Security Posture History

Keep aggregate state changes.

---

# 173. Not raw user activity.

---

# 174. Hard rule.

---

# 175. Host Attestation Integration

Part 71.

---

# 176. Controls can require:

```text
approved boot state
approved artifact measurement
approved config digest
non-quarantined host
```

---

# 177. Attestation Evidence Freshness

Short.

---

# 178. Hard rule.

---

# 179. HSM Integration

Part 72.

---

# 180. Controls:

```text
root key hardware-bound
release signing dual control
no plaintext key export
```

---

# 181. Evidence

HSM operation metadata/attestation.

---

# 182. Never private key.

---

# 183. Hard rule.

---

# 184. Supply-Chain Integration

Part 73.

---

# 185. Controls:

```text
lockfile pinned
SBOM generated
artifact reproducible
signature valid
dependency policy passed
```

---

# 186. Release Promotion Gate

Critical.

---

# 187. Hard Rule

Artifact failing mandatory supply-chain controls cannot be promoted.

---

# 188. Deployment Integration

Parts 62/73.

---

# 189. Controls:

```text
approved release only
signed config
immutable host
secret source valid
```

---

# 190. No deployment exception without explicit record.

---

# 191. Hard rule.

---

# 192. Secrets Integration

Part 80.

---

# 193. Controls:

```text
credential TTL
rotation age
secretless runtime
forbidden env secrets
```

---

# 194. Evidence metadata only.

---

# 195. No secret values.

---

# 196. Hard rule.

---

# 197. Identity/Auth Integration

Parts 81–83.

---

# 198. Controls:

```text
phishing-resistant admin auth
session rotation
no global admin
deleted accounts disabled
```

---

# 199. Compliance engine does not inspect passwords/credentials.

---

# 200. Hard rule.

---

# 201. Privacy Controls

Examples:

```text
max-anonymity no silent direct fallback
remote product analytics disabled
private search local-first
public profile opt-in
```

---

# 202. Privacy Controls Are First-Class

Not secondary.

---

# 203. Hard rule.

---

# 204. Privacy Evidence

Could be:

```text
compiled policy digest
runtime privacy mode
route-policy assertion
telemetry policy state
```

---

# 205. Do Not Collect User Communication Content To Prove Privacy

Hard rule.

---

# 206. Tenant Isolation Controls

Examples:

```text
RLS enabled
tenant keys separated
tenant context explicit
cross-tenant queries denied
```

---

# 207. Evidence

Config/tests/runtime probes.

---

# 208. No tenant user behavior dataset.

---

# 209. Hard rule.

---

# 210. Federation Controls

Examples:

```text
peer trust valid
agreement current
no transitive trust
remote admin authority denied
```

---

# 211. Evidence scoped per federation peer.

---

# 212. No remote user activity evidence.

---

# 213. Hard rule.

---

# 214. Audit Controls

Part 94.

---

# 215. Examples:

```text
audit chain intact
critical actions signed
retention job current
transparency root consistent
```

---

# 216. Evidence from audit subsystem.

---

# 217. No audit data imported wholesale.

---

# 218. Hard rule.

---

# 219. Backup/DR Controls

Examples:

```text
backup freshness
restore drill success
deletion barriers preserved
recovery key custody valid
```

---

# 220. Restore Drill Evidence

Important.

---

# 221. No "backup exists" control without restore test.

---

# 222. Hard rule.

---

# 223. Reliability Controls

Examples:

```text
RTO/RPO tested
quorum health
capacity headroom
```

---

# 224. Operational evidence only.

---

# 225. No user content.

---

# 226. Hard rule.

---

# 227. Vulnerability Controls

Need dependency/host vulnerability input.

---

# 228. Source:

```text
SBOM scanner
OS/package advisory feed
RustSec
```

---

# 229. Compliance state based on severity/policy.

---

# 230. No arbitrary internet execution by evaluator.

---

# 231. Hard rule.

---

# 232. Vulnerability Evidence

Package/artifact metadata.

---

# 233. No source-code secrets.

---

# 234. Hard rule.

---

# 235. Risk Acceptance

Separate from control exception.

---

# 236. Risk Record

```rust
pub struct RiskAcceptance {
    pub risk_id: RiskId,
    pub related_controls: Vec<ControlId>,
    pub severity: RiskSeverity,
    pub expires_at: Timestamp,
    pub approvals: ApprovalBundle,
}
```

---

# 237. No Permanent Risk Acceptance

Default hard rule.

---

# 238. Compliance Framework Mapping

Optional.

---

# 239. Internal controls can map to:

```text
SOC 2
ISO 27001
NIST
organization-specific requirements
```

---

# 240. Framework Mapping Is Metadata

Not control logic.

---

# 241. Hard rule.

---

# 242. One Control Can Satisfy Multiple Requirements

Good.

---

# 243. Avoid duplicate technical controls for paperwork.

---

# 244. Evidence Reuse

Allowed across requirements if scope valid.

---

# 245. But

Evidence access follows strict least privilege.

---

# 246. Hard rule.

---

# 247. Assurance Reports

Generated from control/evidence state.

---

# 248. Types:

```rust
pub enum AssuranceReportType {
    InternalSecurity,
    TenantAssurance,
    OperatorAssurance,
    FederationAssurance,
    AuditPreparation,
}
```

---

# 249. Reports Are Scoped

Hard rule.

---

# 250. Tenant Assurance Report

Can show:

```text
managed controls
status
exceptions
evidence references
```

---

# 251. Must Not Show

```text
other tenants
personal user activity
private message data
```

---

# 252. Hard rule.

---

# 253. Operator Assurance Report

Infrastructure-only.

---

# 254. Federation Assurance

Peer control posture needed for trust agreement.

---

# 255. No private member data.

---

# 256. Hard rule.

---

# 257. Report Signature

Signed manifest.

---

# 258. Report generated from immutable snapshot.

---

# 259. No mutable live-page as evidence.

---

# 260. Hard rule.

---

# 261. Assurance Snapshot

```rust
pub struct AssuranceSnapshot {
    pub snapshot_id: AssuranceSnapshotId,
    pub baseline: SecurityBaselineVersion,
    pub control_results: Vec<ControlEvaluation>,
    pub evidence_root: Digest,
    pub generated_at: CoarseTimestamp,
}
```

---

# 262. Snapshot Does Not Contain Secrets

Hard rule.

---

# 263. Evidence Merkle Root

Optional.

---

# 264. Supports integrity/proof.

---

# 265. No custom crypto.

---

# 266. Hard rule.

---

# 267. Continuous Assurance

Not every control needs real-time evaluation.

---

# 268. Control Frequency Based On Risk.

---

# 269. Critical runtime controls

minutes/hours/event-driven.

---

# 270. Governance/backup controls

daily/weekly/drill cadence.

---

# 271. Hard rule.

---

# 272. Staleness Budget

```rust
pub struct ControlFreshnessBudget {
    pub max_age: Duration,
}
```

---

# 273. If stale

state becomes Unknown.

---

# 274. No stale-pass.

---

# 275. Hard rule.

---

# 276. Control Dependency Graph

Some controls depend on others.

---

# 277. Example:

```text
release-approved
depends on
SBOM-valid
signature-valid
tests-passed
```

---

# 278. Typed dependency graph.

---

# 279. No cycles.

---

# 280. Hard rule.

---

# 281. Control Graph

```rust
pub struct ControlDependency {
    pub control: ControlId,
    pub depends_on: Vec<ControlId>,
}
```

---

# 282. Failure Propagation

Critical dependency failure can fail dependent control.

---

# 283. Hard rule.

---

# 284. Derived Controls

Can compute from subordinate control states.

---

# 285. No manual override without exception.

---

# 286. Hard rule.

---

# 287. Compliance Query API

```rust
pub trait ComplianceQueryService {
    fn posture(
        &self,
        scope: ComplianceScope,
    ) -> Result<SecurityPosture, ComplianceError>;

    fn control(
        &self,
        id: ControlId,
        scope: ComplianceScope,
    ) -> Result<ControlEvaluation, ComplianceError>;
}
```

---

# 288. No `list_all_everything()` Super API

Hard rule.

---

# 289. Compliance Scope

```rust
pub enum ComplianceScope {
    Service(ServiceId),
    Environment(EnvironmentClass),
    Tenant(TenantId),
    FederationPeer(FederationDomainId),
    GovernanceDomain(GovernanceDomainId),
}
```

---

# 290. No Personal User Behavior Scope

Hard rule.

---

# 291. Evidence Access API

Separate capability.

---

# 292. Posture reader need not see raw evidence.

---

# 293. Hard rule.

---

# 294. Evidence Access Roles

```text
SecurityReviewer
Auditor
TenantAssuranceReviewer
OperatorAssuranceReviewer
```

---

# 295. Scoped.

---

# 296. No universal compliance admin.

---

# 297. Hard rule.

---

# 298. Compliance Access Audited

High-risk export/evidence read.

---

# 299. Do not log every dashboard view.

---

# 300. Hard rule.

---

# 301. Policy Change Evaluation

On signed policy update:

```text
recompile relevant controls
invalidate stale results
reevaluate affected scopes
```

---

# 302. No stale posture after policy change.

---

# 303. Hard rule.

---

# 304. Release Gate Integration

Before production promotion:

```text
artifact controls
test controls
supply-chain controls
security controls
privacy controls
```

must pass.

---

# 305. Gate Result

```rust
pub enum ReleaseComplianceDecision {
    Approve,
    Block,
    ExceptionRequired,
}
```

---

# 306. Critical Fail

Block.

---

# 307. No positive metric offset.

---

# 308. Hard rule.

---

# 309. Runtime Admission Gate

Can require control pass before workload Active.

---

# 310. Example:

```text
attestation valid
approved artifact
valid workload cert
config policy current
```

---

# 311. Admission Failure

Quarantine.

---

# 312. No permissive fallback.

---

# 313. Hard rule.

---

# 314. Network Admission

High-value service endpoints can enforce posture capability.

---

# 315. But

Avoid user device attestation as global identity.

---

# 316. Hard rule.

---

# 317. Client Assurance

Optional local checks.

---

# 318. Examples:

```text
app binary integrity
OS security state
secure storage availability
```

---

# 319. Results local by default.

---

# 320. No central device fingerprint.

---

# 321. Hard rule.

---

# 322. Managed Device Posture

Enterprise can require managed-profile/device controls.

---

# 323. Must be disclosed to user.

---

# 324. Personal profile remains isolated.

---

# 325. Hard rule.

---

# 326. Managed Device Evidence

Examples:

```text
screen lock enabled
app version supported
device integrity
managed profile active
```

---

# 327. Avoid:

```text
personal app list
personal message data
location
```

---

# 328. Hard rule.

---

# 329. Assurance vs Surveillance

Important distinction.

---

# 330. Assurance asks:

```text
Is the control in place?
```

Surveillance asks:

```text
What is the user doing?
```

---

# 331. Hard Rule

Compliance engine only asks the former.

---

# 332. Privacy-Preserving Evidence Pattern

Prefer:

```text
boolean proof
digest
signed assertion
attestation claim
aggregate state
```

over:

```text
full raw dataset
```

---

# 333. Hard rule.

---

# 334. Example

To prove telemetry disabled:

store:

```text
signed policy says Disabled
runtime reports Disabled
```

not:

```text
copy entire user activity history
```

---

# 335. Good pattern.

---

# 336. Evidence Redaction

Before export/report.

---

# 337. Remove:

```text
internal hostnames if unnecessary
personal identifiers
secret refs beyond class
```

---

# 338. Hard rule.

---

# 339. Evidence Export

Signed bundle.

---

# 340. Includes:

```text
control IDs
results
evidence hashes
verification metadata
```

---

# 341. Raw evidence only if explicitly required.

---

# 342. Hard rule.

---

# 343. Compliance Snapshot Verification

Offline tool can verify signatures/digests.

---

# 344. Good for auditors/customers.

---

# 345. No production access required.

---

# 346. Compliance Transparency

Possible public high-level report:

```text
supported crypto baseline
release signing posture
audit transparency state
privacy baseline
```

---

# 347. No tenant/user data.

---

# 348. Hard rule.

---

# 349. Privacy Assurance Claims

Must be testable.

---

# 350. Example:

```text
No product analytics in MaximumAnonymity mode.
```

Control can validate config/runtime.

---

# 351. Avoid unverifiable marketing claims.

---

# 352. Hard rule.

---

# 353. Continuous Control Event Bus

Part 75.

---

# 354. Subscribe to:

```text
policy updates
deployment changes
key rotations
host attestation changes
audit failures
backup drill results
```

---

# 355. No user content events.

---

# 356. Hard rule.

---

# 357. Compliance Event

```rust
pub enum ComplianceEvent {
    EvidenceUpdated,
    ControlStateChanged,
    ExceptionCreated,
    ExceptionExpired,
    RemediationStarted,
    RemediationCompleted,
    PostureDegraded,
}
```

---

# 358. Event Payload Minimal

---

# 359. No secrets/private data.

---

# 360. Hard rule.

---

# 361. Notification Integration

Critical posture degradation can notify operators/admins.

---

# 362. Not ordinary users unless their security affected.

---

# 363. No notification spam.

---

# 364. Hard rule.

---

# 365. Incident Integration

Control failure may open incident.

---

# 366. Incident scope explicit.

---

# 367. Compliance engine not incident-management replacement.

---

# 368. Hard rule.

---

# 369. Analytics Integration

Part 92.

---

# 370. Compliance metrics aggregate:

```text
controls passing
controls failing
evaluation latency
evidence freshness
```

---

# 371. No user behavior.

---

# 372. Hard rule.

---

# 373. Experimentation Integration

Part 93.

---

# 374. Experiments cannot bypass controls.

---

# 375. Experiment rollback if control guardrail fails.

---

# 376. Hard rule.

---

# 377. Audit Integration

Part 94.

---

# 378. Record:

```text
control baseline change
exception approval
remediation approval
assurance report export
```

---

# 379. Not every routine evaluation.

---

# 380. Hard rule.

---

# 381. Search Integration

Compliance evidence search is scoped/structured.

---

# 382. No full-text raw evidence lake.

---

# 383. Hard rule.

---

# 384. Database Integration

Stores:

```text
control catalog
control results
evidence metadata
exceptions
remediation state
assurance snapshots
```

---

# 385. Separate from user/product DB.

---

# 386. Hard rule.

---

# 387. Consistency

Critical control result updates should be strongly consistent per scope.

---

# 388. Aggregate dashboard can be eventually consistent.

---

# 389. Hard rule.

---

# 390. Control State Version

```rust
pub struct ControlEvaluationVersion(pub u64);
```

---

# 391. Monotonic per scope/control.

---

# 392. No stale result overwrite.

---

# 393. Hard rule.

---

# 394. Evidence Version

Content-addressed.

---

# 395. Immutable.

---

# 396. No in-place mutation.

---

# 397. Hard rule.

---

# 398. Deletion

Expired evidence deleted per retention.

---

# 399. Control result can retain digest/reference history.

---

# 400. No raw evidence resurrection.

---

# 401. Hard rule.

---

# 402. Backup

Compliance control catalog/results/exceptions can be backed up.

---

# 403. Evidence raw payload optional/minimized.

---

# 404. Restore must preserve:

```text
exception expiry
baseline version
evidence expiry
control versions
```

---

# 405. No stale exception resurrection.

---

# 406. Hard rule.

---

# 407. Multi-Region

Control evaluation can run regionally.

---

# 408. Global posture merges scoped results.

---

# 409. No global user identity.

---

# 410. Hard rule.

---

# 411. Regional Evidence

Region-bound where legal/data-residency requires.

---

# 412. Assurance report can aggregate without moving raw evidence.

---

# 413. Hard rule.

---

# 414. Federation Assurance Exchange

Peers may share signed assurance summary.

---

# 415. Example:

```text
approved release baseline
trust root state
audit integrity
federation policy version
```

---

# 416. No internal host inventory unless required.

---

# 417. No user data.

---

# 418. Hard rule.

---

# 419. Federation Assurance Token

```rust
pub struct FederationAssuranceStatement {
    pub domain: FederationDomainId,
    pub baseline: SecurityBaselineVersion,
    pub posture_digest: Digest,
    pub expires_at: Timestamp,
    pub signature: SignatureBytes,
}
```

---

# 420. Short-lived.

---

# 421. No permanent attestation identity.

---

# 422. Hard rule.

---

# 423. External Auditor Access

Prefer exported evidence bundle/offline verification.

---

# 424. No direct broad production DB access.

---

# 425. Hard rule.

---

# 426. Time-Bounded Auditor Capability

If live access necessary.

---

# 427. Read-only.

---

# 428. Scoped.

---

# 429. Audited.

---

# 430. Auto-expire.

---

# 431. Hard rule.

---

# 432. Evidence Collection Agents

Minimal.

---

# 433. Agent Permissions

Read only what control requires.

---

# 434. No root-wide filesystem scraping baseline.

---

# 435. Hard rule.

---

# 436. Agent Sandboxing

Recommended.

---

# 437. Separate collectors per evidence class.

---

# 438. No monolithic privileged collector.

---

# 439. Hard rule.

---

# 440. Collector Identity

Workload identity Part 79.

---

# 441. Short-lived cert.

---

# 442. Evidence signed.

---

# 443. Hard rule.

---

# 444. Collector Compromise

Evidence trust downgrade/revocation.

---

# 445. Re-evaluate affected controls.

---

# 446. Hard rule.

---

# 447. Evidence Replay

Prevent via:

```text
freshness
nonce/challenge
sequence
attestation epoch
```

---

# 448. No stale signed evidence reuse.

---

# 449. Hard rule.

---

# 450. Challenge-Response Evidence

For high-value controls.

---

# 451. Example host attestation.

---

# 452. Nonce-scoped.

---

# 453. Replay rejected.

---

# 454. Hard rule.

---

# 455. Policy Exception UX

Show:

```text
control
reason
scope
approver class
expiry
compensating controls
```

---

# 456. No hidden exception.

---

# 457. Hard rule.

---

# 458. Posture UX

Do not use gamified single security score.

---

# 459. Show:

```text
critical failures
high failures
unknowns
exceptions
stale evidence
```

---

# 460. Hard rule.

---

# 461. User Security Center

Can show local assurance:

```text
backup configured
devices verified
privacy mode
security history integrity
```

---

# 462. No shame/scoring.

---

# 463. Good.

---

# 464. Control Documentation

Every control includes:

```text
intent
threat addressed
evidence
failure impact
remediation
```

---

# 465. No opaque checkbox.

---

# 466. Hard rule.

---

# 467. Machine-Readable Control Spec

RON source.

---

# 468. Compiled to typed/Postcard internal.

---

# 469. JSON only external interop.

---

# 470. No dynamic arbitrary evaluator code in RON.

---

# 471. Hard rule.

---

# 472. Control Spec Example

```rust
pub struct ControlSpec {
    pub id: ControlId,
    pub domain: ControlDomain,
    pub applicability: ApplicabilityRule,
    pub evidence: Vec<EvidenceRequirement>,
    pub evaluation: EvaluationRule,
}
```

---

# 473. Evaluation Rule

Typed enum/DSL with bounded semantics.

---

# 474. No Turing-complete scripts.

---

# 475. Hard rule.

---

# 476. Evaluation DSL

Possible operations:

```text
Equals
Contains
LessThan
GreaterThan
All
Any
Not
Fresh
SignedBy
```

---

# 477. Deterministic.

---

# 478. No network I/O from rule.

---

# 479. Hard rule.

---

# 480. CI Integration

Controls can be pre-evaluated in CI.

---

# 481. Examples:

```text
license policy
dependency policy
SBOM
tests
reproducibility
```

---

# 482. Release compliance manifest generated.

---

# 483. Hard rule.

---

# 484. Runtime Compliance

Separate from build-time controls.

---

# 485. Never assume CI pass means runtime pass forever.

---

# 486. Hard rule.

---

# 487. Compliance Manifest

```rust
pub struct ComplianceManifest {
    pub release: ReleaseId,
    pub baseline: SecurityBaselineVersion,
    pub controls: Vec<ControlEvaluation>,
    pub manifest_digest: Digest,
}
```

---

# 488. Signed.

---

# 489. Part of promotion evidence.

---

# 490. No user data.

---

# 491. Hard rule.

---

# 492. Control Failure Categories

```rust
pub enum ControlFailureClass {
    Configuration,
    MissingEvidence,
    ExpiredEvidence,
    ExplicitViolation,
    CollectorFailure,
    PolicyMismatch,
}
```

---

# 493. Helps remediation.

---

# 494. No ambiguous generic fail.

---

# 495. Hard rule.

---

# 496. Compliance Error Taxonomy

```rust
pub enum ComplianceError {
    UnknownControl,
    ControlNotApplicable,
    EvidenceMissing,
    EvidenceExpired,
    EvidenceInvalid,
    EvaluatorFailed,
    PolicyMismatch,
    ExceptionInvalid,
    ExceptionExpired,
    RemediationDenied,
    ScopeMismatch,
    UnauthorizedAccess,
    Internal,
}
```

---

# 497. Observability

Safe metrics:

```text
evaluation latency
controls by state
evidence freshness
collector error rate
exception expiry count
```

---

# 498. Forbidden:

```text
user behavior
private content
cross-tenant identity map
```

---

# 499. Hard rule.

---

# 500. Compliance SLOs

Examples:

```text
critical control freshness
evaluation success
posture update latency
exception expiry enforcement
```

---

# 501. Security SLO

```text
0 unknown critical control treated as pass
0 expired exception treated as valid
0 unsigned assurance report accepted
```

---

# 502. Privacy SLO

```text
0 private content ingested as control evidence
0 personal-user activity used for managed compliance
```

---

# 503. Failure Modes

```text
collector outage
policy compiler bug
evidence stale
posture store outage
exception service outage
```

---

# 504. Collector Outage

Affected controls become Unknown after freshness budget.

---

# 505. No stale-pass.

---

# 506. Hard rule.

---

# 507. Policy Compiler Bug

Freeze affected policy activation.

---

# 508. Last valid signed compiled policy within validity.

---

# 509. No permissive fallback.

---

# 510. Hard rule.

---

# 511. Evidence Stale

Unknown/Fail per control semantics.

---

# 512. Notify owner.

---

# 513. No silent extension.

---

# 514. Hard rule.

---

# 515. Posture Store Outage

Critical admission gates can fail closed.

---

# 516. Noncritical dashboards may be stale.

---

# 517. Explicit.

---

# 518. Hard rule.

---

# 519. Exception Service Outage

No new exception.

---

# 520. Existing signed exception remains valid until expiry.

---

# 521. No automatic extension.

---

# 522. Hard rule.

---

# 523. Testing

Need compliance/assurance testkit.

---

# 524. Test Scenarios

```text
critical pass/fail
stale evidence
exception expiry
remediation
tenant isolation
```

---

# 525. Unknown Test

Missing evidence => Unknown, never Pass.

---

# 526. Freshness Test

Expired evidence no longer satisfies control.

---

# 527. Critical Gate Test

Critical failure blocks release/admission.

---

# 528. Exception Test

Approved exception yields ExceptionApproved, not Pass.

---

# 529. Expiry Test

Expired exception automatically invalid.

---

# 530. Scope Test

Exception cannot apply outside target scope.

---

# 531. Remediation Test

Successful action triggers re-evaluation.

---

# 532. Evidence Privacy Test

Private content rejected as evidence class.

---

# 533. Tenant Test

Tenant A cannot inspect B posture/evidence.

---

# 534. Personal Privacy Test

Managed compliance cannot inspect personal content.

---

# 535. Federation Test

Assurance statement contains no user activity.

---

# 536. Attestation Replay Test

Stale challenge rejected.

---

# 537. Backup Test

Expired exception/evidence not resurrected.

---

# 538. Policy Update Test

Affected controls re-evaluated.

---

# 539. Collector Compromise Test

Revoked collector evidence no longer trusted.

---

# 540. Release Gate Test

Supply-chain critical failure blocks promotion.

---

# 541. Audit Integration Test

Exception approval/remediation approval audited.

---

# 542. Fuzzing

Fuzz:

```text
control spec
evidence envelope
assurance snapshot
exception record
federation assurance statement
```

---

# 543. Property Tests

Properties:

```text
unknown critical control can never become compliant
expired exception can never authorize a failing control
evidence outside scope can never satisfy control
privacy-sensitive user data can never be accepted by infrastructure evidence collectors
```

---

# 544. Formal Verification Targets

Strong candidates:

```text
control-state lattice
exception expiry
release gate
admission gate
evidence freshness
```

---

# 545. Kani Candidate

state/exception/applicability invariants.

---

# 546. TLA+ Candidate

policy update → evidence expiry → control fail → remediation → reevaluation.

---

# 547. Loom Candidate

concurrent evidence update + exception expiry + posture query.

---

# 548. Performance

Evaluation should be incremental.

---

# 549. Re-evaluate affected controls only.

---

# 550. No scan-everything loop.

---

# 551. Hard rule.

---

# 552. Evidence Cache

Bounded.

---

# 553. Immutable digest keyed.

---

# 554. Expired evidence evicted.

---

# 555. Control Results

Small.

---

# 556. Posture queries fast.

---

# 557. Background evaluation priority lower than critical serving path, except admission gates.

---

# 558. Hard rule.

---

# 559. Storage

Separate stores:

```text
control catalog
compiled policy
evidence metadata
control results
exceptions
remediation
assurance snapshots
```

---

# 560. No user data warehouse.

---

# 561. Hard rule.

---

# 562. Partitioning

By:

```text
environment
service
tenant
federation peer
governance domain
```

---

# 563. Not by global user.

---

# 564. Hard rule.

---

# 565. Crate Layout

Recommended:

```text
crates/
├── siar-compliance-core/
├── siar-control-catalog/
├── siar-policy-compiler/
├── siar-evidence-core/
├── siar-evidence-collector/
├── siar-control-evaluator/
├── siar-posture-engine/
├── siar-compliance-exception/
├── siar-remediation/
├── siar-assurance-report/
├── siar-compliance-observability/
└── siar-compliance-testkit/
```

---

# 566. `siar-compliance-core`

Owns:

```text
ControlId
ControlState
severity
scope
errors
```

---

# 567. `siar-control-catalog`

Signed machine-readable control definitions.

---

# 568. `siar-policy-compiler`

Signed policy → bounded typed evaluation rules.

---

# 569. `siar-evidence-core`

Evidence objects/provenance/freshness/trust.

---

# 570. `siar-evidence-collector`

Minimal scoped evidence agents.

---

# 571. `siar-control-evaluator`

Deterministic control evaluation.

---

# 572. `siar-posture-engine`

Per-scope security posture aggregation.

---

# 573. `siar-compliance-exception`

Temporary scoped exceptions/risk acceptance.

---

# 574. `siar-remediation`

Typed remediation workflows/rollback.

---

# 575. `siar-assurance-report`

Signed assurance snapshots/exports.

---

# 576. `siar-compliance-observability`

Pipeline health only.

---

# 577. `siar-compliance-testkit`

privacy/evidence/exception/release-gate tests.

---

# 578. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. Compliance evaluates technical and administrative controls; it does not monitor ordinary user behavior or private communication content.
2. Control state is explicit: Pass, Fail, Unknown, NotApplicable, or ExceptionApproved; Unknown and ExceptionApproved are never silently treated as Pass.
3. Evidence is scoped, provenance-bound, freshness-bounded, and cryptographically verifiable where required; stale or untrusted evidence cannot satisfy a control.
4. Critical security/privacy controls cannot be offset by aggregate posture scores, product metrics, or unrelated passing controls.
5. Exceptions are narrow, signed/approved, time-bounded, auditable, and cannot mutate the underlying control definition or become permanent hidden bypasses.
6. Managed tenant compliance can inspect managed posture only and cannot consume personal messages, contacts, searches, social memberships, or other unrelated personal activity.
7. Assurance evidence prefers digests, signed assertions, attestation claims, and aggregate state over raw datasets; secret values and private content are never baseline evidence.
8. Policy compilation and remote configuration cannot weaken hardcoded privacy, anonymity, cryptographic, authorization, or tenant-isolation invariants.
9. Release/runtime admission gates fail safely on critical control failures, invalid evidence, expired exceptions, or untrusted posture.
10. Evidence, exceptions, and control results are versioned and anti-rollback; stale backups or replicas cannot resurrect expired exceptions or superseded evidence.
11. Assurance reports and federation posture statements are signed, scoped, privacy-minimized, and never expose cross-tenant or user-level activity.
12. Compliance outputs are used for assurance and remediation—not analytics, advertising, employee surveillance, or social/behavioral profiling.
```

---

# 579. Initial Production Scope

Implement first:

```text
typed control catalog
signed baseline/control definitions
bounded applicability rules
evidence model with provenance/freshness/trust
configuration/artifact/attestation/audit evidence
incremental control evaluator
explicit Pass/Fail/Unknown/N/A/Exception state
security posture engine
temporary scoped exceptions
exception expiry enforcement
release compliance gate
runtime admission gate
typed remediation workflow
tenant/operator/federation scope isolation
signed assurance snapshots
audit integration
privacy-safe compliance metrics
compliance testkit
```

Then add:

```text
Merkle-rooted evidence bundles
independent assurance witnesses
advanced automated remediation
external framework mappings
privacy-preserving managed-device posture proofs
formal control/evidence verification
```

---

# 580. Definition of Done

Part 95 is complete when:

- policies map to typed machine-evaluable controls
- controls have explicit severity/evidence/applicability
- Unknown never means Pass
- evidence has provenance/trust/freshness
- private user content is rejected as baseline compliance evidence
- critical control failures remain explicit
- exceptions are scoped, approved, expiring, and auditable
- remediation triggers re-evaluation
- release/runtime admission gates use posture safely
- tenant/operator/federation assurance scopes stay isolated
- signed assurance snapshots can be verified offline
- backups cannot resurrect stale evidence/exceptions
- audit/analytics boundaries remain strict
- compliance/exception/attestation/fuzz/formal tests are specified

---

# 581. Final Architecture

```text
                SIGNED POLICY / BASELINE
                          │
                          ▼
                    CONTROL CATALOG
                          │
                          ▼
                 EVIDENCE REQUIREMENTS
                          │
             ┌────────────┼────────────┐
             │            │            │
        CONFIG/STATE   ATTESTATION   AUDIT/PROOFS
             │            │            │
             └────────────┼────────────┘
                          ▼
                  CONTROL EVALUATION
                          │
             ┌────────────┼────────────┐
             │            │            │
           PASS       EXCEPTION       FAIL
             │            │            │
             └────────────┼────────────┘
                          ▼
                  SECURITY POSTURE
                          │
             ┌────────────┼────────────┐
             │            │            │
         ASSURANCE     REMEDIATION   GATING
```

Assurance safety model:

```text
typed controls
+
minimal evidence
+
freshness and provenance
+
explicit unknown state
+
bounded exceptions
+
signed assurance
+
privacy-preserving scopes
+
release/runtime gates
```

not:

```text
collect everything users and employees do, call it evidence, and centralize it forever in a compliance warehouse
```

---

# 582. Final Principle

Compliance should prove that required controls are operating—not observe people in order to infer whether the system might be safe.

The correct model is:

```text
define controls precisely
+
collect minimal evidence
+
verify provenance and freshness
+
fail visibly
+
exception narrowly
+
remediate safely
+
sign assurance
+
never turn compliance into surveillance
```

This architecture gives SIAR a privacy-preserving assurance foundation for continuous control evaluation, security posture, release/runtime gating, tenant and federation assurance, evidence collection, exceptions, remediation, and external verification while preserving the anonymity, local-first, least-authority, and anti-surveillance guarantees established across Parts 34–94.
