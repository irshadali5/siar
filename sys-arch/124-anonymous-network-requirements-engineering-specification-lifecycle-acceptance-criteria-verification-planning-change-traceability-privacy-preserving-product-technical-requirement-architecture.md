# Core System Architecture Part 124 — Anonymous Network Requirements Engineering, Specification Lifecycle, Acceptance Criteria, Verification Planning, Change Traceability & Privacy-Preserving Product/Technical Requirement Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 124  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 65, 94–108, 120–123

**Primary purpose:** define SIAR's requirements-engineering architecture for requirement identity, product and technical specifications, lifecycle, acceptance criteria, verification planning, baselines, change control, traceability, ambiguity/conflict handling, decomposition, release gating, and privacy-preserving requirement governance.

---

# 1. Purpose

Requirements become dangerous when they are:

```text
ambiguous
unversioned
untestable
unowned
contradictory
detached from implementation
```

A mature platform must answer:

```text
What exactly must the system do?
Why does this requirement exist?
How will it be verified?
Which design implements it?
Which tests prove it?
What changes when the requirement changes?
Which release first satisfies it?
```

The governing principle is:

> **SIAR requirements should be explicit, versioned, testable, traceable, change-controlled, and linked to verification evidence—without creating behavioral profiling of users or performance scoring of engineers.**

---

# 2. Architectural Position

```text
                    NEED / CONSTRAINT
                          │
                          ▼
                     REQUIREMENT
                          │
              ┌───────────┼───────────┐
              │           │           │
          PRODUCT      TECHNICAL    QUALITY
              │           │           │
              └───────────┼───────────┘
                          ▼
                   ACCEPTANCE CRITERIA
                          │
                          ▼
                  VERIFICATION PLAN
                          │
                          ▼
                 DESIGN / IMPLEMENTATION
                          │
                          ▼
                    TEST / RELEASE
```

---

# 3. Core Separation

Keep distinct:

```text
need
requirement
constraint
design decision
acceptance criterion
verification method
test case
implementation
release evidence
```

---

# 4. Non-Goals

Part 124 does not create:

```text
feature-voting surveillance
user behavioral segmentation
developer requirement-completion rankings
vague prose treated as acceptance proof
```

---

# 5. Requirement Identity

```rust
pub struct RequirementId(pub [u8; 16]);
```

Stable, opaque, never reused.

---

# 6. Requirement Class

```rust
pub enum RequirementClass {
    Product,
    Functional,
    Security,
    Privacy,
    Reliability,
    Performance,
    Capacity,
    Operational,
    Compatibility,
    DataGovernance,
    Maintainability,
    ComplianceTechnical,
}
```

---

# 7. Product vs Technical Requirement

Product:

```text
what capability/value must exist
```

Technical:

```text
what system property/constraint must hold
```

---

# 8. Hard rule.

---

# 9. Requirement Strength

```rust
pub enum RequirementStrength {
    Must,
    Should,
    May,
}
```

---

# 10. Hard Invariant

Some `Must` requirements are non-waivable.

---

# 11. Hard rule.

---

# 12. Requirement Record

```rust
pub struct Requirement {
    pub id: RequirementId,
    pub class: RequirementClass,
    pub title: String,
    pub statement: RequirementStatement,
    pub strength: RequirementStrength,
    pub state: RequirementState,
}
```

---

# 13. Requirement State

```rust
pub enum RequirementState {
    Draft,
    Proposed,
    Approved,
    Implementing,
    Verifying,
    Satisfied,
    Deprecated,
    Superseded,
    Rejected,
}
```

---

# 14. No Draft→Satisfied Direct

Hard rule.

---

# 15. Requirement Statement Pattern

Prefer:

```text
The system SHALL <observable behavior> under <conditions> within <constraint>.
```

---

# 16. Avoid

```text
fast
secure enough
user friendly
scalable
robust
```

without measurable criteria.

---

# 17. Hard rule.

---

# 18. Requirement Origin

```rust
pub enum RequirementOrigin {
    ProductNeed,
    ArchitecturePrinciple,
    SecurityThreatModel,
    PrivacyConstraint,
    ReliabilityObjective,
    IncidentLearning,
    EngineeringRisk,
    CompatibilityNeed,
    TechnicalComplianceControl,
}
```

---

# 19. Origin Trace

Every approved requirement should have origin.

---

# 20. Hard rule.

---

# 21. Requirement Owner

Prefer product/service/team owner.

```rust
pub enum RequirementOwnerRef {
    ProductArea(ProductAreaId),
    Service(ServiceId),
    PlatformTeam(TeamId),
    SecurityTeam(TeamId),
    PrivacyTeam(TeamId),
    GovernanceBody(GovernanceBodyId),
}
```

---

# 22. No Individual-Blame Ownership

Hard rule.

---

# 23. Requirement Scope

```rust
pub enum RequirementScope {
    Platform,
    Service(ServiceId),
    Protocol(ProtocolId),
    ClientPlatform(PlatformClass),
    Region(RegionId),
    TenantAggregate(TenantId),
}
```

---

# 24. No Individual User Scope

Hard rule.

---

# 25. Requirement Version

```rust
pub struct RequirementVersion(pub u64);
```

---

# 26. Versioned Requirement

```rust
pub struct VersionedRequirement {
    pub id: RequirementId,
    pub version: RequirementVersion,
    pub content_digest: Digest,
}
```

---

# 27. Historical Versions Immutable

Hard rule.

---

# 28. Requirement Baseline

Snapshot for release/program increment.

```rust
pub struct RequirementBaseline {
    pub baseline_id: RequirementBaselineId,
    pub requirements: BTreeMap<RequirementId, RequirementVersion>,
}
```

---

# 29. Baseline Immutable

Hard rule.

---

# 30. Product Requirement

```rust
pub struct ProductRequirement {
    pub requirement: RequirementId,
    pub capability: ProductCapabilityId,
    pub acceptance: Vec<AcceptanceCriterionId>,
}
```

---

# 31. Technical Requirement

```rust
pub struct TechnicalRequirement {
    pub requirement: RequirementId,
    pub invariant: Option<InvariantId>,
    pub verification_plan: VerificationPlanId,
}
```

---

# 32. Non-Functional Requirement

Examples:

```text
p99 latency
availability
recovery time
privacy
memory bound
```

---

# 33. Hard rule.

---

# 34. Requirement Decomposition

Parent → child.

```rust
pub struct RequirementDecomposition {
    pub parent: RequirementId,
    pub children: Vec<RequirementId>,
}
```

---

# 35. Child Requirements Must Preserve Parent Intent

Hard rule.

---

# 36. No Circular Decomposition

Hard rule.

---

# 37. Requirement Dependency

```rust
pub struct RequirementDependency {
    pub from: RequirementId,
    pub to: RequirementId,
    pub relation: RequirementDependencyKind,
}
```

---

# 38. Dependency Kind

```rust
pub enum RequirementDependencyKind {
    Requires,
    ConflictsWith,
    Refines,
    Supersedes,
}
```

---

# 39. Hard rule.

---

# 40. Requirement Conflict

Conflicts must be explicit.

---

# 41. Conflict Record

```rust
pub struct RequirementConflict {
    pub a: RequirementId,
    pub b: RequirementId,
    pub state: ConflictState,
}
```

---

# 42. Conflict State

```rust
pub enum ConflictState {
    Detected,
    UnderReview,
    Resolved,
    AcceptedTradeoff,
}
```

---

# 43. AcceptedTradeoff Requires Decision Record

Hard rule.

---

# 44. Ambiguity Detection

Requirement may be marked ambiguous.

```rust
pub enum RequirementQualityState {
    Clear,
    Ambiguous,
    Untestable,
    Conflicting,
    Incomplete,
}
```

---

# 45. Approved Requires Clear/Testable

Hard rule.

---

# 46. Acceptance Criterion

```rust
pub struct AcceptanceCriterion {
    pub id: AcceptanceCriterionId,
    pub requirement: RequirementId,
    pub condition: AcceptanceCondition,
    pub expected: AcceptanceOutcome,
}
```

---

# 47. Criterion Must Be Observable

Hard rule.

---

# 48. Acceptance Types

```rust
pub enum AcceptanceCriterionType {
    Behavioral,
    SecurityInvariant,
    PerformanceThreshold,
    ReliabilityThreshold,
    Compatibility,
    Operational,
    DataLifecycle,
}
```

---

# 49. Example

```text
Given a persisted outgoing message,
when network connectivity is unavailable,
then the message remains durable locally
and transitions to retryable state without data loss.
```

---

# 50. Good.

---

# 51. Acceptance Criterion Strength

```rust
pub enum AcceptanceStrength {
    Required,
    Advisory,
}
```

---

# 52. Hard rule.

---

# 53. Verification Method

```rust
pub enum VerificationMethod {
    UnitTest,
    PropertyTest,
    FuzzTest,
    IntegrationTest,
    EndToEndTest,
    Benchmark,
    Simulation,
    FormalVerification,
    Inspection,
    OperationalExercise,
}
```

---

# 54. Verification Plan

```rust
pub struct VerificationPlan {
    pub id: VerificationPlanId,
    pub requirement: RequirementId,
    pub methods: Vec<VerificationMethod>,
    pub evidence_required: Vec<VerificationEvidenceKind>,
}
```

---

# 55. Hard rule.

---

# 56. Verification Evidence Kind

```rust
pub enum VerificationEvidenceKind {
    TestResult,
    BenchmarkResult,
    SimulationResult,
    FormalProofResult,
    ReleaseReceipt,
    DrillReceipt,
    ArchitectureComplianceReport,
}
```

---

# 57. Satisfied ≠ Implemented

Hard rule.

---

# 58. Requirement Satisfaction

```rust
pub enum RequirementSatisfactionState {
    NotImplemented,
    PartiallyImplemented,
    ImplementedUnverified,
    Verified,
    FailedVerification,
}
```

---

# 59. Verified Is Evidence-Backed

Hard rule.

---

# 60. Requirement-to-Code Trace

Part 123.

```text
Requirement
→ ADR/Standard
→ Crate/Module/Symbol
→ Test
→ Release
```

---

# 61. Hard rule.

---

# 62. Verification Coverage

```rust
pub struct RequirementVerificationCoverage {
    pub requirement: RequirementId,
    pub required_methods: usize,
    pub verified_methods: usize,
    pub state: TraceHealth,
}
```

---

# 63. No Misleading Percent Without Scope

Hard rule.

---

# 64. Security Requirement

Must link:

```text
threat/control
implementation
verification
```

---

# 65. Hard rule.

---

# 66. Privacy Requirement

Must link:

```text
data-flow constraint
implementation
retention/deletion/access tests
```

---

# 67. Hard rule.

---

# 68. Reliability Requirement

Must link:

```text
SLO/resilience model
implementation
simulation/drill/test
```

---

# 69. Hard rule.

---

# 70. Performance Requirement

Must specify:

```text
metric
percentile
load model
environment
```

---

# 71. Hard rule.

---

# 72. Capacity Requirement

Must specify:

```text
resource
headroom
failure scenario
forecast horizon
```

---

# 73. Hard rule.

---

# 74. Operational Requirement

Must specify:

```text
operator action
conditions
expected outcome
recovery path
```

---

# 75. Hard rule.

---

# 76. Compatibility Requirement

```rust
pub struct CompatibilityRequirement {
    pub protocol_version: VersionRange,
    pub client_version: VersionRange,
    pub migration_window: Duration,
}
```

---

# 77. Hard rule.

---

# 78. Requirement Change

Requirement changes are governed.

---

# 79. Change Request

```rust
pub struct RequirementChangeRequest {
    pub id: RequirementChangeId,
    pub requirement: RequirementId,
    pub from_version: RequirementVersion,
    pub proposed_statement: RequirementStatement,
    pub rationale: ChangeRationale,
}
```

---

# 80. Change State

```rust
pub enum RequirementChangeState {
    Draft,
    ImpactAnalysis,
    Review,
    Approved,
    Rejected,
    Applied,
}
```

---

# 81. No Applied Without Impact Analysis

Hard rule.

---

# 82. Impact Analysis

Part 123.

Requirement change identifies:

```text
ADRs
standards
code
tests
releases
deployments
risks
```

---

# 83. Hard rule.

---

# 84. Impact Report

```rust
pub struct RequirementChangeImpact {
    pub requirement: RequirementId,
    pub affected_nodes: Vec<KnowledgeNodeId>,
    pub risks: Vec<EngineeringRiskId>,
}
```

---

# 85. Hard rule.

---

# 86. Requirement Change Approval

Authority depends on class/scope.

---

# 87. Security/privacy hard requirement changes require specialized review.

---

# 88. Hard rule.

---

# 89. Requirement Supersession

New version may supersede old.

---

# 90. Historical release keeps old baseline.

---

# 91. Hard rule.

---

# 92. Requirement Deprecation

Deprecated requirement remains historical.

---

# 93. Replacement recommended.

---

# 94. Hard rule.

---

# 95. Acceptance Criteria Change

Must trigger verification-plan review.

---

# 96. Hard rule.

---

# 97. Verification Plan Change

May not silently weaken mandatory method.

---

# 98. Hard rule.

---

# 99. Requirement Review

Triggers:

```text
incident
risk
architecture change
protocol change
platform change
regulatory technical requirement change
```

---

# 100. Hard rule.

---

# 101. Review Cadence

```rust
pub enum RequirementReviewCadence {
    Quarterly,
    SemiAnnual,
    Annual,
    EventDriven,
}
```

---

# 102. High-risk requirements more frequent.

---

# 103. Hard rule.

---

# 104. Requirement Freshness

```rust
pub enum RequirementFreshness {
    Fresh,
    DueForReview,
    Stale,
}
```

---

# 105. Stale ≠ Invalid

Hard rule.

---

# 106. Requirement Assumption

```rust
pub struct RequirementAssumption {
    pub requirement: RequirementId,
    pub statement: AssumptionStatement,
    pub expires_at: Option<Timestamp>,
}
```

---

# 107. Assumption Changes Trigger Review

Hard rule.

---

# 108. Requirement Constraint

```rust
pub struct RequirementConstraint {
    pub requirement: RequirementId,
    pub constraint: ConstraintKind,
}
```

---

# 109. Constraint Kind

```rust
pub enum ConstraintKind {
    Security,
    Privacy,
    Residency,
    Compatibility,
    Resource,
    Platform,
    Dependency,
}
```

---

# 110. Hard rule.

---

# 111. Product Requirement Discovery

Sources:

```text
explicit product goals
support pain points
operational needs
technical constraints
```

---

# 112. No behavioral surveillance baseline.

---

# 113. Hard rule.

---

# 114. User Research Boundary

Product requirements may use voluntary research or explicit feedback.

---

# 115. Do not infer hidden personal traits/behavior profiles into requirements.

---

# 116. Hard rule.

---

# 117. Requirement Prioritization

Use:

```text
criticality
security/privacy
reliability
dependency
delivery sequencing
```

---

# 118. No user-value scoring by sensitive personal category.

---

# 119. Hard rule.

---

# 120. Requirement Priority

```rust
pub enum RequirementPriority {
    Critical,
    High,
    Medium,
    Low,
}
```

---

# 121. Priority Must Reference Rationale

Hard rule.

---

# 122. MVP Requirement

Can mark delivery phase.

```rust
pub enum DeliveryPhase {
    Foundation,
    V1,
    PostV1,
    Optional,
}
```

---

# 123. Hard rule.

---

# 124. Requirement Set

```rust
pub struct RequirementSet {
    pub id: RequirementSetId,
    pub requirements: BTreeSet<RequirementId>,
    pub baseline: RequirementBaselineId,
}
```

---

# 125. Use For

```text
release
feature
protocol
migration
```

---

# 126. Hard rule.

---

# 127. Release Requirement Gate

Part 108.

Release checks:

```text
required requirements approved
implementation linked
verification passed
exceptions valid
```

---

# 128. Hard rule.

---

# 129. Release Requirement Report

```rust
pub struct ReleaseRequirementReport {
    pub release: ReleaseId,
    pub baseline: RequirementBaselineId,
    pub unsatisfied: Vec<RequirementId>,
    pub unverified: Vec<RequirementId>,
}
```

---

# 130. Hard rule.

---

# 131. No Release Satisfaction By Ticket Status

Hard rule.

---

# 132. Change Integration

Part 107.

Change references requirement IDs.

---

# 133. Hard rule.

---

# 134. Architecture Governance Integration

Part 122.

Requirement influences ADR/standard.

---

# 135. Hard rule.

---

# 136. Knowledge Graph Integration

Part 123.

All requirement traces queryable.

---

# 137. Hard rule.

---

# 138. Risk Integration

Part 121.

Risk can create requirement.

---

# 139. Requirement can mitigate risk.

---

# 140. Hard rule.

---

# 141. PIR Integration

Part 120.

Incident lesson can create/strengthen requirement.

---

# 142. Hard rule.

---

# 143. SLO Integration

Part 109.

Reliability requirement may map to SLO.

---

# 144. Hard rule.

---

# 145. Resilience Integration

Part 110.

Failure-survival requirements map to fault-domain simulations.

---

# 146. Hard rule.

---

# 147. Capacity Integration

Part 111.

Capacity requirement maps to forecast/headroom evidence.

---

# 148. Hard rule.

---

# 149. Performance Integration

Part 112.

Latency/throughput requirements map to benchmark evidence.

---

# 150. Hard rule.

---

# 151. Efficiency/Sustainability Integration

Parts 113–114.

Resource/energy requirements remain subordinate to hard security/privacy/reliability constraints.

---

# 152. Hard rule.

---

# 153. Hardware/Facility Integration

Parts 115–118.

Physical requirements map to site/fleet controls/tests.

---

# 154. Hard rule.

---

# 155. Crisis Command Integration

Part 119.

Emergency requirements specify authority and fallback behavior.

---

# 156. Hard rule.

---

# 157. Compliance Technical Requirements

Represent technical implementation obligations only.

---

# 158. Legal interpretation stays outside core.

---

# 159. Hard rule.

---

# 160. Requirement Exception

Some non-hard requirements can have exception.

---

# 161. Exception Record

```rust
pub struct RequirementException {
    pub requirement: RequirementId,
    pub rationale: ExceptionRationale,
    pub expires_at: Timestamp,
    pub compensating_controls: Vec<ControlId>,
}
```

---

# 162. Hard Requirements Non-Waivable

Hard rule.

---

# 163. Exception Expiry

Automatic.

---

# 164. Hard rule.

---

# 165. Verification Waiver

Never allowed for hard invariants.

---

# 166. Hard rule.

---

# 167. Requirement Conflict Resolution

Possible outcomes:

```text
change one requirement
scope requirements differently
accept explicit tradeoff
reject one requirement
```

---

# 168. Hard rule.

---

# 169. Tradeoff Decision

Requires ADR/design review.

---

# 170. Hard rule.

---

# 171. Requirement Quality Gate

Before approval:

```text
clear
bounded
testable
owned
origin known
acceptance criteria present
verification plan present
```

---

# 172. Hard rule.

---

# 173. Requirement Linter

```rust
pub trait RequirementLintService {
    fn evaluate(
        &self,
        requirement: &Requirement,
    ) -> Result<Vec<RequirementLintFinding>, RequirementsError>;
}
```

---

# 174. Lint Finding

```rust
pub enum RequirementLintFinding {
    AmbiguousLanguage,
    MissingAcceptanceCriteria,
    MissingVerificationPlan,
    MissingOwner,
    MissingOrigin,
    NonMeasurableThreshold,
    ConflictingRequirement,
}
```

---

# 175. Hard rule.

---

# 176. Machine-Enforceable Validation

Where possible:

```text
schema
enum
threshold
trace requirement
```

---

# 177. Hard rule.

---

# 178. Human Review

For semantics/tradeoffs.

---

# 179. Hard rule.

---

# 180. Requirement Authoring Format

Preferred:

```text
Markdown prose
RON metadata
Postcard compiled runtime policy where relevant
```

---

# 181. Hard rule.

---

# 182. Requirement Metadata RON Example

```ron
(
    id: "REQ-REL-042",
    class: "Reliability",
    strength: "Must",
    owner: "service:messaging",
    acceptance: ["AC-042-1", "AC-042-2"],
)
```

---

# 183. Human-reviewable.

---

# 184. Hard rule.

---

# 185. Requirement Registry Service

```rust
pub trait RequirementRegistry {
    fn get(
        &self,
        id: RequirementId,
    ) -> Result<Requirement, RequirementsError>;

    fn baseline(
        &self,
        id: RequirementBaselineId,
    ) -> Result<RequirementBaseline, RequirementsError>;
}
```

---

# 186. Requirement Change Service

```rust
pub trait RequirementChangeService {
    fn propose(
        &self,
        change: RequirementChangeRequest,
    ) -> Result<RequirementChangeId, RequirementsError>;

    fn apply(
        &self,
        change: RequirementChangeId,
    ) -> Result<RequirementVersion, RequirementsError>;
}
```

---

# 187. Verification Service

```rust
pub trait RequirementVerificationService {
    fn evaluate(
        &self,
        requirement: RequirementId,
        release: ReleaseId,
    ) -> Result<RequirementSatisfactionState, RequirementsError>;
}
```

---

# 188. Trace Service

```rust
pub trait RequirementImpactService {
    fn impact(
        &self,
        requirement: RequirementId,
    ) -> Result<RequirementChangeImpact, RequirementsError>;
}
```

---

# 189. No People-Analytics API

Hard rule.

---

# 190. Error Taxonomy

```rust
pub enum RequirementsError {
    RequirementUnknown,
    InvalidStateTransition,
    AmbiguousRequirement,
    MissingAcceptanceCriteria,
    MissingVerificationPlan,
    VerificationFailed,
    ConflictUnresolved,
    ImpactAnalysisRequired,
    ExceptionNotAllowed,
    ExceptionExpired,
    BaselineMismatch,
    ScopeViolation,
    Unauthorized,
    Internal,
}
```

---

# 191. Observability

Safe metrics:

```text
requirements by state
stale requirements
unverified critical requirements
broken traces
open conflicts
```

---

# 192. Forbidden:

```text
requirements completed per developer
rejections per author
user-behavior requirement analytics
```

---

# 193. Hard rule.

---

# 194. Requirements SLOs

Examples:

```text
critical approved requirement has acceptance + verification plan
release has no unverified hard requirement
requirement change impact analysis completed before approval
```

---

# 195. Security SLO

```text
0 hard security requirement waived
0 hard requirement marked satisfied without verification evidence
```

---

# 196. Privacy SLO

```text
0 user behavioral profiling used as hidden requirement input
0 engineer productivity scoring from requirement metadata
```

---

# 197. Failure Modes

```text
ambiguous requirement approved
test does not verify intended behavior
requirement changes without trace propagation
release satisfies old requirement accidentally
```

---

# 198. Ambiguous Requirement

Blocks approval.

---

# 199. Hard rule.

---

# 200. Weak Test Mapping

Verification plan reviewed.

---

# 201. Hard rule.

---

# 202. Unpropagated Change

Impact-analysis gate blocks application.

---

# 203. Hard rule.

---

# 204. Baseline Drift

Release pins explicit requirement baseline.

---

# 205. Hard rule.

---

# 206. Testing

Need requirements-engineering testkit.

---

# 207. Test Scenarios

```text
new privacy requirement
performance threshold change
requirement supersession
release verification
conflicting requirements
```

---

# 208. State Test

Invalid lifecycle transition rejected.

---

# 209. Quality Test

Ambiguous text blocks approval.

---

# 210. Acceptance Test

Missing acceptance criterion blocks approval.

---

# 211. Verification Test

Implemented but unverified remains unsatisfied.

---

# 212. Conflict Test

Unresolved conflict blocks affected baseline.

---

# 213. Baseline Test

Historical release retains historical requirement version.

---

# 214. Change Test

Requirement change propagates impact analysis.

---

# 215. Exception Test

Hard invariant cannot be waived.

---

# 216. Trace Test

Deleted implementation breaks requirement trace.

---

# 217. Privacy Test

No user/employee dimension accepted.

---

# 218. Fuzzing

Fuzz:

```text
requirement lifecycle
baseline manifests
acceptance criteria
change requests
exception records
```

---

# 219. Property Tests

Properties:

```text
hard requirement can never reach Satisfied without verification evidence
approved requirement can never lack acceptance criteria and verification plan
historical requirement baseline can never mutate after release
individual user or employee identity can never become a requirement analytics dimension
```

---

# 220. Formal Verification Targets

Strong candidates:

```text
requirement lifecycle
baseline immutability
change impact gating
exception expiry
```

---

# 221. Kani Candidate

state/waiver/verification invariants.

---

# 222. TLA+ Candidate

draft → approve → implement → verify → baseline → change → supersede.

---

# 223. Loom Candidate

concurrent requirement change + release baseline creation + verification update.

---

# 224. Performance

Requirements governance is control-plane/offline work.

---

# 225. Release gate queries indexed state.

---

# 226. Hard rule.

---

# 227. Storage

Separate:

```text
requirements
versions
baselines
acceptance criteria
verification plans
change requests
exceptions
conflicts
```

---

# 228. No user/employee analytics warehouse.

---

# 229. Hard rule.

---

# 230. Partitioning

By:

```text
requirement class
service
protocol
release
baseline
```

---

# 231. No person/user partition.

---

# 232. Hard rule.

---

# 233. Crate Layout

Recommended:

```text
crates/
├── siar-requirements-core/
├── siar-requirement-registry/
├── siar-requirement-quality/
├── siar-acceptance-criteria/
├── siar-verification-planning/
├── siar-requirement-change/
├── siar-requirement-baseline/
├── siar-requirement-trace/
├── siar-requirement-observability/
└── siar-requirement-testkit/
```

---

# 234. `siar-requirements-core`

Owns:

```text
RequirementId
RequirementClass
RequirementState
RequirementsError
```

---

# 235. `siar-requirement-registry`

Requirement/version/origin/ownership storage.

---

# 236. `siar-requirement-quality`

Ambiguity/conflict/testability checks.

---

# 237. `siar-acceptance-criteria`

Observable outcome definitions.

---

# 238. `siar-verification-planning`

Verification method/evidence policy.

---

# 239. `siar-requirement-change`

Change request/impact/approval.

---

# 240. `siar-requirement-baseline`

Immutable release/program requirement snapshots.

---

# 241. `siar-requirement-trace`

Integration with Part 123 knowledge graph.

---

# 242. `siar-requirement-observability`

Aggregate requirement health only.

---

# 243. `siar-requirement-testkit`

lifecycle/baseline/trace/privacy tests.

---

# 244. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. Requirements are scoped to products, services, protocols, platforms, regions, tenants, and system properties—not to individual users, employee performance, or behavioral profiles.
2. Requirement needs, design decisions, acceptance criteria, verification plans, tests, implementations, and release evidence remain distinct artifacts linked explicitly through traceability.
3. Every approved requirement is versioned, owned, sourced, clear, testable, and has acceptance criteria and a verification plan; ambiguity or unresolved conflict blocks approval.
4. Historical requirement versions and release baselines are immutable; later requirement changes never rewrite what an older release was required to satisfy.
5. Hard security/privacy/reliability requirements are non-waivable, and a requirement cannot reach Satisfied merely because implementation code exists—verification evidence is mandatory.
6. Requirement changes require impact analysis across ADRs, standards, code, tests, releases, deployments, risks, and exceptions before application.
7. Acceptance criteria define observable outcomes and cannot rely on vague qualities such as "fast", "secure", or "robust" without measurable context.
8. Requirement conflicts and tradeoffs are explicit and resolved through architecture/design governance rather than hidden inside implementation choices.
9. Product requirement discovery may use explicit research and operational evidence but cannot rely on covert behavioral profiling, sensitive-personal inference, or user-surveillance datasets.
10. Requirement governance metrics track technical lifecycle, verification, conflicts, baselines, and trace health and cannot become developer productivity, author rejection, or individual performance scoring.
11. Release and change gates use pinned requirement baselines and verified satisfaction state so ticket completion or implementation presence alone cannot claim compliance.
12. Requirements engineering integrates with architecture governance, engineering knowledge, risk/PIR, SLOs, resilience, capacity, performance, hardware/facility controls, compliance, audit, change, and release governance without creating an alternate authority or surveillance path.
```

---

# 245. Initial Production Scope

Implement first:

```text
typed RequirementId/RequirementClass/RequirementState
product vs technical requirement model
requirement origin/owner/scope
versioning
immutable requirement baselines
requirement quality linting
acceptance criteria
verification plans
satisfaction states
requirement decomposition/dependencies/conflicts
change requests
impact analysis
exception rules
release baseline/gate integration
architecture/risk/PIR traceability
privacy-safe requirement dashboards
requirements testkit
```

Then add:

```text
controlled natural-language requirement linting
automatic ambiguity hints
formal requirement consistency checks
cross-repository implementation trace
AI-assisted candidate acceptance criteria
temporal baseline diff
machine-generated verification-plan suggestions with human approval
```

---

# 246. Definition of Done

Part 124 is complete when:

- requirements are typed/versioned/scoped;
- product and technical requirements are distinct;
- acceptance criteria are mandatory for approval;
- verification plans are mandatory;
- historical baselines are immutable;
- hard requirements cannot be waived;
- implemented does not imply satisfied;
- change impact propagation exists;
- conflicts/ambiguity block approval;
- releases pin requirement baselines;
- requirement traces integrate with Part 123;
- no user/developer profiling is created;
- lifecycle/baseline/trace/privacy/fuzz/formal tests are specified.

---

# 247. Final Architecture

```text
                     NEED / CONSTRAINT
                           │
                           ▼
                      REQUIREMENT
                           │
              ┌────────────┼────────────┐
              │            │            │
           PRODUCT      TECHNICAL     QUALITY
              │            │            │
              └────────────┼────────────┘
                           ▼
                  ACCEPTANCE CRITERIA
                           │
                           ▼
                  VERIFICATION PLAN
                           │
                           ▼
                 DESIGN / IMPLEMENTATION
                           │
                           ▼
                    TEST / EVIDENCE
                           │
                           ▼
                     RELEASE BASELINE
```

Requirements-engineering safety model:

```text
versioned requirements
+
clear acceptance criteria
+
verification planning
+
immutable baselines
+
change impact analysis
+
traceability
+
release gating
+
privacy/blame safeguards
```

not:

```text
write vague feature prose, treat merged code as proof, change requirements without impact analysis, and rank developers by requirement throughput
```

---

# 248. Final Principle

A requirement is only useful when everyone can tell exactly what it means, what implements it, and what evidence proves it.

The correct model is:

```text
state the need clearly
+
classify and scope it
+
define acceptance
+
plan verification
+
trace implementation
+
pin baselines
+
govern changes
+
verify before claiming satisfaction
+
never turn requirements into user or engineer surveillance
```

This architecture gives SIAR a privacy-preserving requirements-engineering foundation for product/technical specifications, acceptance criteria, verification planning, baselines, change traceability, release gating, and requirement lifecycle governance while preserving the anonymity, local-first, least-authority, architecture-governance, engineering-knowledge, and anti-surveillance guarantees established across Parts 34–123.
