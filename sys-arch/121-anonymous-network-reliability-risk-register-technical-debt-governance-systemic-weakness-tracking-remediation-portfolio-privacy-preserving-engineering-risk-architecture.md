# Core System Architecture Part 121 — Anonymous Network Reliability Risk Register, Technical Debt Governance, Systemic Weakness Tracking, Remediation Portfolio & Privacy-Preserving Engineering Risk Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 121  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 65, 94–120

**Primary purpose:** define SIAR's engineering-risk architecture for reliability risk registers, technical debt governance, systemic weakness tracking, remediation portfolios, risk ownership, acceptance/exception policy, evidence, dependency-aware aggregation, recurrence linkage, release/change/SLO integration, and privacy-preserving engineering risk management.

---

# 1. Purpose

Not every weakness is an active incident, but unresolved weaknesses accumulate into future incidents.

A mature engineering platform must answer:

```text
Which known weaknesses exist?
Which services depend on them?
What evidence supports the risk?
What happens if we defer remediation?
Which risks are accepted temporarily?
Which technical debts are becoming systemic?
Which remediation portfolio gives the largest reliability improvement?
```

The governing principle is:

> **SIAR engineering risk should be represented as explicit, evidence-backed technical risk with scoped ownership, remediation, verification, and expiry—without using opaque people scores, user behavior, or organizational blame as risk signals.**

---

# 2. Architectural Position

```text
       KNOWN WEAKNESS / TECHNICAL DEBT
                     │
                     ▼
                RISK REGISTER
                     │
          ┌──────────┼──────────┐
          │          │          │
       EVIDENCE    IMPACT    EXPOSURE
          │          │          │
          └──────────┼──────────┘
                     ▼
              REMEDIATION PORTFOLIO
                     │
          ┌──────────┼──────────┐
          │          │          │
        FIX       ACCEPT      RETIRE
          │          │          │
          └──────────┼──────────┘
                     ▼
                 VERIFICATION
                     │
                     ▼
               RISK REDUCTION
```

---

# 3. Core Separation

Keep distinct:

```text
risk
incident
technical debt
known limitation
vulnerability
defect
control gap
accepted exception
remediation action
```

---

# 4. Non-Goals

Part 121 does not create:

```text
employee risk scores
developer rankings
user-risk profiles
one opaque global risk number
automatic blame assignment
```

---

# 5. Engineering Risk Identity

```rust
pub struct EngineeringRiskId(pub [u8; 16]);
```

Opaque, canonical, never reused.

---

# 6. Risk Scope

```rust
pub enum EngineeringRiskScope {
    Service(ServiceId),
    Component(ComponentId),
    Dependency(DependencyId),
    Region(RegionId),
    Site(PhysicalSiteId),
    Fleet(FleetGroupId),
    TenantAggregate(TenantId),
    FederationPeer(FederationDomainId),
    Platform,
}
```

---

# 7. No User Scope

Hard rule.

---

# 8. Risk Class

```rust
pub enum EngineeringRiskClass {
    Reliability,
    Availability,
    Performance,
    Capacity,
    Security,
    Privacy,
    DataIntegrity,
    Recoverability,
    Maintainability,
    Operability,
    Dependency,
    Facility,
    Hardware,
    SupplyChain,
}
```

---

# 9. Technical Debt Class

```rust
pub enum TechnicalDebtClass {
    Architectural,
    Code,
    Configuration,
    TestCoverage,
    Observability,
    Documentation,
    Automation,
    Dependency,
    Infrastructure,
    OperationalProcess,
}
```

---

# 10. Risk vs Debt

Technical debt may create risk.

Not all risk is technical debt.

---

# 11. Hard rule.

---

# 12. Risk Record

```rust
pub struct EngineeringRisk {
    pub id: EngineeringRiskId,
    pub scope: EngineeringRiskScope,
    pub class: EngineeringRiskClass,
    pub statement: RiskStatement,
    pub state: EngineeringRiskState,
    pub owner: RiskOwnerRef,
}
```

---

# 13. Risk State

```rust
pub enum EngineeringRiskState {
    Identified,
    Analyzing,
    Open,
    Mitigating,
    AcceptedTemporarily,
    Verifying,
    Resolved,
    Retired,
}
```

---

# 14. No Identified→Resolved Direct

Hard rule.

---

# 15. Risk Statement

A good risk statement expresses:

```text
condition
+
failure mode
+
technical consequence
```

Example:

```text
If the single HSM cluster fails,
release signing becomes unavailable,
because no independently attested secondary cluster exists.
```

---

# 16. Hard Rule

Avoid person-centered statements.

---

# 17. Risk Evidence

```rust
pub struct RiskEvidence {
    pub risk: EngineeringRiskId,
    pub references: Vec<EvidenceRef>,
    pub confidence: RiskConfidence,
}
```

---

# 18. Risk Confidence

```rust
pub enum RiskConfidence {
    Low,
    Medium,
    High,
    Verified,
}
```

---

# 19. Unknown ≠ Low

Represent missing evidence separately.

---

# 20. Hard rule.

---

# 21. Evidence Sources

Examples:

```text
incident review
benchmark
SLO violation
capacity forecast
resilience simulation
vulnerability finding
inventory drift
audit/control gap
facility assessment
```

---

# 22. No Employee Opinion As Sole Evidence For High-Severity Risk

Hard rule.

---

# 23. Risk Dimensions

Avoid one opaque score.

Use explicit dimensions:

```rust
pub struct RiskDimensions {
    pub impact: RiskImpact,
    pub likelihood: RiskLikelihood,
    pub exposure: RiskExposure,
    pub detectability: RiskDetectability,
    pub recovery_difficulty: RecoveryDifficulty,
}
```

---

# 24. Risk Impact

```rust
pub enum RiskImpact {
    Minor,
    Moderate,
    Major,
    Severe,
    Catastrophic,
}
```

---

# 25. Risk Likelihood

```rust
pub enum RiskLikelihood {
    Rare,
    Unlikely,
    Possible,
    Likely,
    Frequent,
    Unknown,
}
```

---

# 26. Risk Exposure

```rust
pub enum RiskExposure {
    Localized,
    ServiceWide,
    Regional,
    MultiRegion,
    PlatformWide,
}
```

---

# 27. Risk Detectability

```rust
pub enum RiskDetectability {
    Easy,
    Moderate,
    Difficult,
    Silent,
    Unknown,
}
```

---

# 28. Recovery Difficulty

```rust
pub enum RecoveryDifficulty {
    Easy,
    Moderate,
    Difficult,
    Complex,
    Unknown,
}
```

---

# 29. No Single Composite Score By Default

Hard rule.

---

# 30. Why

A single number hides:

```text
catastrophic-but-rare
frequent-but-minor
silent-but-recoverable
```

---

# 31. Risk Matrix

Use dimensions explicitly.

---

# 32. Hard rule.

---

# 33. Risk Priority

Allowed as categorical governance decision.

```rust
pub enum RiskPriority {
    Critical,
    High,
    Medium,
    Low,
}
```

---

# 34. Priority Must Reference Dimensions/Evidence

Hard rule.

---

# 35. Risk Priority Is Not Employee Priority

Hard rule.

---

# 36. Systemic Weakness

A systemic weakness appears across multiple services/components or can recur broadly.

---

# 37. Weakness Record

```rust
pub struct SystemicWeakness {
    pub weakness_id: SystemicWeaknessId,
    pub class: SystemicWeaknessClass,
    pub affected_scopes: BTreeSet<EngineeringRiskScope>,
    pub evidence: Vec<EvidenceRef>,
}
```

---

# 38. Weakness Class

```rust
pub enum SystemicWeaknessClass {
    SharedDependency,
    CommonConfiguration,
    RepeatedIncidentPattern,
    MissingGuardrail,
    WeakRecovery,
    WeakObservability,
    SharedCapacityConstraint,
    CommonSoftwareFault,
    OperationalComplexity,
}
```

---

# 39. Hard Rule

Systemic weakness tracking is technical, not a "problem people" list.

---

# 40. Link Risk To Weakness

```rust
pub struct RiskWeaknessLink {
    pub risk: EngineeringRiskId,
    pub weakness: SystemicWeaknessId,
}
```

---

# 41. Repeated Incident Linkage

Part 120.

---

# 42. Hard rule.

---

# 43. Technical Debt Record

```rust
pub struct TechnicalDebtItem {
    pub debt_id: TechnicalDebtId,
    pub class: TechnicalDebtClass,
    pub scope: EngineeringRiskScope,
    pub statement: DebtStatement,
    pub linked_risks: BTreeSet<EngineeringRiskId>,
}
```

---

# 44. Debt Lifecycle

```rust
pub enum TechnicalDebtState {
    Documented,
    Scheduled,
    Reducing,
    Accepted,
    Resolved,
}
```

---

# 45. Accepted Debt Is Explicit

Hard rule.

---

# 46. Debt Interest

Conceptually:

```text
maintenance burden
change risk
incident likelihood
performance cost
operational complexity
```

---

# 47. Avoid Monetary-Like Fake Precision

Hard rule.

---

# 48. Debt Interest Dimensions

```rust
pub struct DebtInterest {
    pub change_friction: DebtBurden,
    pub operational_burden: DebtBurden,
    pub reliability_burden: DebtBurden,
    pub security_burden: DebtBurden,
}
```

---

# 49. Debt Burden

```rust
pub enum DebtBurden {
    Low,
    Medium,
    High,
    Critical,
    Unknown,
}
```

---

# 50. Hard rule.

---

# 51. Known Limitation

Distinct from debt.

---

# 52. Known Limitation Record

```rust
pub struct KnownLimitation {
    pub limitation_id: LimitationId,
    pub scope: EngineeringRiskScope,
    pub behavior: LimitationStatement,
    pub linked_risks: BTreeSet<EngineeringRiskId>,
}
```

---

# 53. Hard rule.

---

# 54. Risk Discovery

Sources:

```text
PIR
security review
architecture review
capacity forecast
resilience simulation
drift detection
benchmark
support trend
dependency advisory
facility audit
```

---

# 55. No Behavior-Mining Discovery

Hard rule.

---

# 56. Risk Intake

```rust
pub struct RiskIntake {
    pub source: RiskSource,
    pub scope: EngineeringRiskScope,
    pub statement: RiskStatement,
    pub evidence: Vec<EvidenceRef>,
}
```

---

# 57. Risk Source

```rust
pub enum RiskSource {
    IncidentReview,
    ArchitectureReview,
    SecurityReview,
    Audit,
    Forecast,
    Simulation,
    Monitoring,
    DependencyAdvisory,
    ManualTechnicalObservation,
}
```

---

# 58. Intake Validation

Check:

```text
scope
evidence
duplicate
ownership
```

---

# 59. Hard rule.

---

# 60. Duplicate Risk Detection

Technical similarity only.

---

# 61. Do not merge risks merely because same team owns them.

---

# 62. Hard rule.

---

# 63. Risk Ownership

Owner is accountable for risk treatment.

---

# 64. Owner Scope

```rust
pub enum RiskOwnerRef {
    ServiceOwner(ServiceId),
    PlatformTeam(TeamId),
    SecurityTeam(TeamId),
    FacilityTeam(TeamId),
    GovernanceBody(GovernanceBodyId),
}
```

---

# 65. No Person-Blame Ownership

Prefer team/service ownership.

---

# 66. Hard rule.

---

# 67. Risk Treatment

```rust
pub enum RiskTreatment {
    Mitigate,
    Avoid,
    TransferOperationally,
    AcceptTemporarily,
    RetireDependency,
}
```

---

# 68. "Transfer" Does Not Mean Security Responsibility Vanishes

Hard rule.

---

# 69. Risk Acceptance

Time-bounded.

---

# 70. Acceptance Record

```rust
pub struct RiskAcceptance {
    pub risk: EngineeringRiskId,
    pub rationale: RiskAcceptanceReason,
    pub compensating_controls: Vec<ControlId>,
    pub accepted_by: RiskAcceptanceAuthorityRef,
    pub expires_at: Timestamp,
}
```

---

# 71. No Permanent Acceptance By Default

Hard rule.

---

# 72. Cannot Accept Hard Invariant Violation

Examples:

```text
tenant isolation broken
invalid signatures allowed
encryption disabled
```

---

# 73. Hard rule.

---

# 74. Acceptance Authority

Depends risk class/priority.

---

# 75. Hard rule.

---

# 76. Expired Acceptance

Risk returns Open.

---

# 77. Hard rule.

---

# 78. Compensating Controls

May reduce exposure temporarily.

---

# 79. Example:

```text
disable vulnerable feature
reduce blast radius
increase monitoring
```

---

# 80. Hard rule.

---

# 81. Remediation Action

```rust
pub struct RiskRemediationAction {
    pub action_id: RiskRemediationActionId,
    pub risk: EngineeringRiskId,
    pub category: RemediationCategory,
    pub owner: ActionOwnerRef,
    pub state: RemediationState,
}
```

---

# 82. Remediation Category

```rust
pub enum RemediationCategory {
    Architecture,
    Code,
    Configuration,
    Capacity,
    Dependency,
    SecurityControl,
    Recovery,
    Test,
    Monitoring,
    Documentation,
    Facility,
    Hardware,
}
```

---

# 83. Remediation State

```rust
pub enum RemediationState {
    Proposed,
    Approved,
    Scheduled,
    InProgress,
    Implemented,
    Verifying,
    Verified,
    Rejected,
}
```

---

# 84. Implemented ≠ Verified

Hard rule.

---

# 85. Verification Evidence

```rust
pub enum RiskVerificationEvidence {
    TestPass,
    SimulationPass,
    SloImprovement,
    CapacityEvidence,
    ResilienceEvidence,
    ReleaseReceipt,
    ControlPass,
    DrillPass,
}
```

---

# 86. Hard rule.

---

# 87. Remediation Portfolio

A portfolio groups actions across risks.

---

# 88. Portfolio

```rust
pub struct RemediationPortfolio {
    pub portfolio_id: RemediationPortfolioId,
    pub actions: Vec<RiskRemediationActionId>,
    pub objective: PortfolioObjective,
}
```

---

# 89. Portfolio Objective

```rust
pub enum PortfolioObjective {
    ReduceCriticalRisk,
    RemoveSystemicWeakness,
    ImproveRecoverability,
    ImproveCapacitySafety,
    ReduceDependencyConcentration,
    ReduceOperationalComplexity,
}
```

---

# 90. Hard rule.

---

# 91. Portfolio Dependencies

Actions may depend on each other.

---

# 92. Model DAG.

---

# 93. Hard rule.

---

# 94. Portfolio Constraints

```text
budget
engineering capacity
maintenance windows
release freeze
residency
security
```

---

# 95. Constraints do not erase risk.

---

# 96. Hard rule.

---

# 97. Portfolio Prioritization

Use dimensions:

```text
criticality
blast radius
recurrence
remediation effectiveness
effort
dependency unlock
```

---

# 98. No employee capacity score.

---

# 99. Hard rule.

---

# 100. Remediation Effectiveness

```rust
pub enum RemediationEffectiveness {
    Partial,
    Substantial,
    EliminatesFailureMode,
    Unknown,
}
```

---

# 101. Unknown ≠ Eliminates

Hard rule.

---

# 102. Residual Risk

After remediation:

```rust
pub struct ResidualEngineeringRisk {
    pub original: EngineeringRiskId,
    pub remaining_dimensions: RiskDimensions,
    pub evidence: Vec<EvidenceRef>,
}
```

---

# 103. Hard rule.

---

# 104. Risk Closure

Risk closes only if:

```text
remediation verified
or
risk no longer applicable
```

---

# 105. Hard rule.

---

# 106. Dependency Risk

Track external/internal dependency weaknesses.

---

# 107. Dependency Concentration

```rust
pub struct DependencyConcentrationRisk {
    pub dependency: DependencyId,
    pub affected_services: BTreeSet<ServiceId>,
    pub redundancy: RedundancyAssessment,
}
```

---

# 108. Part 110 integration.

---

# 109. Hard rule.

---

# 110. Single Provider Risk

Explicit.

---

# 111. Single CA/HSM/DNS/DB dependency.

---

# 112. Hard rule.

---

# 113. Change Risk Integration

Part 107.

Proposed changes should query open risks.

---

# 114. Example:

```text
change touches component with Critical open risk
```

---

# 115. Hard rule.

---

# 116. Release Risk Integration

Part 108.

Critical open risks may:

```text
block launch
require exception
require constrained rollout
```

---

# 117. Hard rule.

---

# 118. SLO Risk Integration

Part 109.

Repeated SLO exhaustion can create reliability risk.

---

# 119. Hard rule.

---

# 120. Resilience Risk Integration

Part 110.

Unsurvivable fault-domain scenario creates risk.

---

# 121. Hard rule.

---

# 122. Forecast Risk Integration

Part 111.

Saturation horizon inside lead time creates capacity risk.

---

# 123. Hard rule.

---

# 124. Performance Risk Integration

Part 112.

Persistent p99 regression can create performance risk.

---

# 125. Hard rule.

---

# 126. Efficiency/Sustainability Risk

Parts 113–114.

Examples:

```text
memory amplification
unsupported hardware
energy-heavy legacy service
```

---

# 127. Hard rule.

---

# 128. Hardware/Facility Risk

Parts 115–118.

Examples:

```text
unsupported firmware
single UPS path
unverified salvage hardware
```

---

# 129. Hard rule.

---

# 130. Crisis/Post-Incident Integration

Parts 119–120.

PIR corrective actions may create/close risks.

---

# 131. Hard rule.

---

# 132. Risk Aging

Older risk may become more urgent.

---

# 133. Risk Age

```rust
pub struct RiskAge {
    pub identified_at: Timestamp,
    pub last_reviewed_at: Timestamp,
}
```

---

# 134. Age Alone Does Not Determine Severity

Hard rule.

---

# 135. Review Cadence

```rust
pub enum RiskReviewCadence {
    Weekly,
    Monthly,
    Quarterly,
    EventDriven,
}
```

---

# 136. Based on priority/class.

---

# 137. Hard rule.

---

# 138. Stale Risk

If not reviewed:

```rust
pub enum RiskFreshness {
    Fresh,
    DueSoon,
    Stale,
}
```

---

# 139. Stale ≠ Closed

Hard rule.

---

# 140. Risk Trend

```rust
pub enum RiskTrend {
    Improving,
    Stable,
    Worsening,
    Unknown,
}
```

---

# 141. Based on technical evidence.

---

# 142. No person-based trend.

---

# 143. Hard rule.

---

# 144. Risk Aggregation

Aggregate by:

```text
service
region
risk class
systemic weakness
```

---

# 145. Do Not Average Into One Global Score

Hard rule.

---

# 146. Risk Heatmap

Allowed categorical matrix.

---

# 147. No ranking of teams/people.

---

# 148. Hard rule.

---

# 149. Cross-Risk Dependency

Some risks amplify others.

---

# 150. Risk Dependency Edge

```rust
pub struct RiskDependency {
    pub from: EngineeringRiskId,
    pub to: EngineeringRiskId,
    pub relation: RiskRelation,
}
```

---

# 151. Risk Relation

```rust
pub enum RiskRelation {
    Amplifies,
    Enables,
    SharesDependency,
    SharesFaultDomain,
    BlocksRemediation,
}
```

---

# 152. Hard rule.

---

# 153. Risk Graph

Useful for systemic analysis.

---

# 154. Not an employee graph.

---

# 155. Hard rule.

---

# 156. Control Mapping

Risk may map to controls.

---

# 157. Risk Control Link

```rust
pub struct RiskControlLink {
    pub risk: EngineeringRiskId,
    pub control: ControlId,
    pub relationship: ControlRelationship,
}
```

---

# 158. Relationship

```rust
pub enum ControlRelationship {
    Prevents,
    Detects,
    Contains,
    Recovers,
}
```

---

# 159. Hard rule.

---

# 160. Control Failure

If control fails repeatedly, systemic weakness.

---

# 161. Hard rule.

---

# 162. Architecture Decision Integration

High-risk remediation may require ADR.

---

# 163. Hard rule.

---

# 164. Technical Debt Governance

Debt must be visible.

---

# 165. No hidden TODO-only debt.

---

# 166. Hard rule.

---

# 167. Debt Registration Trigger

Examples:

```text
temporary workaround
duplicated subsystem
manual production step
unsupported dependency
missing tests
```

---

# 168. Hard rule.

---

# 169. Debt Expiry

Temporary debt should have review/expiry.

---

# 170. Hard rule.

---

# 171. "Temporary" Means Time-Bounded

Hard rule.

---

# 172. Debt Conversion

Debt can become explicit risk when evidence shows exposure.

---

# 173. Hard rule.

---

# 174. Risk Acceptance vs Debt Acceptance

Separate records.

---

# 175. Hard rule.

---

# 176. Exception Registry Integration

Accepted risk links to exception/approval evidence.

---

# 177. Hard rule.

---

# 178. Governance Policy

```rust
pub struct EngineeringRiskPolicy {
    pub version: EngineeringRiskPolicyVersion,
    pub review_rules: Vec<RiskReviewRule>,
    pub acceptance_rules: Vec<RiskAcceptanceRule>,
    pub release_gates: Vec<RiskReleaseGateRule>,
}
```

---

# 179. Signed/versioned.

---

# 180. Anti-rollback.

---

# 181. Hard rule.

---

# 182. Risk Policy Precedence

```text
hard invariants
> security/privacy/residency
> risk policy
> team preference
```

---

# 183. Hard rule.

---

# 184. Risk Release Gate

```rust
pub struct RiskReleaseGate {
    pub release: ReleaseId,
    pub blocking_risks: Vec<EngineeringRiskId>,
    pub required_exceptions: Vec<RiskAcceptanceId>,
}
```

---

# 185. Hard rule.

---

# 186. Change Gate

```rust
pub struct RiskChangeGate {
    pub change: ChangeId,
    pub affected_risks: Vec<EngineeringRiskId>,
    pub decision: GateResult,
}
```

---

# 187. Hard rule.

---

# 188. Remediation Verification

Use evidence.

---

# 189. Example:

```text
risk: no region failover capacity
action: add region capacity
verification: N-1 resilience simulation passes
```

---

# 190. Hard rule.

---

# 191. No Closure From Ticket Status Alone

Hard rule.

---

# 192. Risk Acceptance Renewal

Requires re-review.

---

# 193. No automatic perpetual renewal.

---

# 194. Hard rule.

---

# 195. Audit Integration

Part 94.

Audit:

```text
risk accepted
critical risk closed
exception renewed
```

---

# 196. Not every edit/comment.

---

# 197. Hard rule.

---

# 198. Compliance Integration

Part 95.

Can map:

```text
open control risk
accepted exception
remediation evidence
```

---

# 199. Hard rule.

---

# 200. Security Risk Boundary

Part 98/97.

Vulnerability management remains specialized.

---

# 201. Engineering risk may reference vulnerability exposure.

---

# 202. Hard rule.

---

# 203. Privacy Risk Boundary

Privacy risks remain technical policy/control issues.

---

# 204. No behavioral user profiling.

---

# 205. Hard rule.

---

# 206. Risk Review Meeting

Optional.

---

# 207. Artifact is canonical.

---

# 208. Hard rule.

---

# 209. Risk Register Views

Allowed:

```text
critical open risks
risks by service
risks by class
accepted temporary risks
systemic weaknesses
```

---

# 210. Forbidden:

```text
risks by employee
engineer risk ranking
```

---

# 211. Hard rule.

---

# 212. Risk Dashboard

Show:

```text
open risk dimensions
owner scope
acceptance expiry
remediation state
verification state
```

---

# 213. Hard rule.

---

# 214. Systemic Weakness Dashboard

Show:

```text
affected services
common dependencies
linked incidents
remediation portfolio
```

---

# 215. Hard rule.

---

# 216. Portfolio Planning

Plan across multiple risks.

---

# 217. Example:

```text
replace shared legacy auth library
```

may close many risks.

---

# 218. Good.

---

# 219. Hard rule.

---

# 220. Portfolio Capacity

Engineering capacity is finite.

---

# 221. But capacity limits do not erase risk.

---

# 222. Hard rule.

---

# 223. Risk Escalation

Escalate if:

```text
priority rises
acceptance expires
incident recurs
remediation fails
```

---

# 224. Hard rule.

---

# 225. Risk De-Escalation

Requires evidence.

---

# 226. No cosmetic downgrade.

---

# 227. Hard rule.

---

# 228. Risk Retirement

Only if:

```text
component/service no longer exists
```

or risk no longer applies.

---

# 229. Hard rule.

---

# 230. Risk Transfer On Ownership Change

Ownership change preserves history.

---

# 231. Hard rule.

---

# 232. Risk Import/Export

For federation/tenant contexts, share scoped summaries.

---

# 233. No internal secret dump.

---

# 234. Hard rule.

---

# 235. Federation Risk

Peer-domain reliability/security risk tracked at peer scope.

---

# 236. No remote user risk.

---

# 237. Hard rule.

---

# 238. Tenant Aggregate Risk

Managed tenant-specific infrastructure risk allowed.

---

# 239. No individual user risk.

---

# 240. Hard rule.

---

# 241. Risk Evidence Retention

Retain while risk open + policy period.

---

# 242. Minimize duplicated evidence.

---

# 243. Hard rule.

---

# 244. Risk Register API

```rust
pub trait EngineeringRiskRegister {
    fn create(
        &self,
        intake: RiskIntake,
    ) -> Result<EngineeringRiskId, EngineeringRiskError>;

    fn get(
        &self,
        id: EngineeringRiskId,
    ) -> Result<EngineeringRisk, EngineeringRiskError>;
}
```

---

# 245. Risk Treatment Service

```rust
pub trait RiskTreatmentService {
    fn accept_temporarily(
        &self,
        acceptance: RiskAcceptance,
    ) -> Result<RiskAcceptanceReceipt, EngineeringRiskError>;

    fn propose_remediation(
        &self,
        action: RiskRemediationAction,
    ) -> Result<RiskRemediationActionId, EngineeringRiskError>;
}
```

---

# 246. Verification Service

```rust
pub trait RiskVerificationService {
    fn verify(
        &self,
        action: RiskRemediationActionId,
        evidence: Vec<EvidenceRef>,
    ) -> Result<VerificationState, EngineeringRiskError>;
}
```

---

# 247. Portfolio Service

```rust
pub trait RemediationPortfolioService {
    fn build(
        &self,
        risks: &[EngineeringRiskId],
    ) -> Result<RemediationPortfolio, EngineeringRiskError>;
}
```

---

# 248. No People-Ranking API

Hard rule.

---

# 249. Error Taxonomy

```rust
pub enum EngineeringRiskError {
    RiskUnknown,
    EvidenceMissing,
    OwnerMissing,
    InvalidStateTransition,
    AcceptanceExpired,
    HardInvariantNotAcceptable,
    VerificationFailed,
    DependencyCycle,
    PolicyViolation,
    ScopeViolation,
    Unauthorized,
    Internal,
}
```

---

# 250. Observability

Safe metrics:

```text
open risks by class
accepted risks expiring
critical remediation actions
systemic weakness count
verified closures
```

---

# 251. Forbidden:

```text
risks per employee
who created most debt
user-risk history
```

---

# 252. Hard rule.

---

# 253. Risk Governance SLOs

Examples:

```text
critical risk owner assigned within target
acceptances reviewed before expiry
critical remediation verified within deadline
stale risk review rate below target
```

---

# 254. Security SLO

```text
0 hard-invariant risk accepted temporarily
0 critical risk closed without verification
```

---

# 255. Privacy SLO

```text
0 employee-risk scoring
0 user-behavior risk modeling
```

---

# 256. Failure Modes

```text
risk register becomes stale
accepted risk never expires
ticket closure masquerades as risk closure
systemic weaknesses fragmented across duplicate risks
```

---

# 257. Stale Register

Freshness policy alerts.

---

# 258. Hard rule.

---

# 259. Non-Expiring Acceptance

Rejected by schema/policy for temporary acceptance.

---

# 260. Hard rule.

---

# 261. Ticket Closure

Does not change risk state automatically.

---

# 262. Hard rule.

---

# 263. Duplicate Fragmentation

Pattern/weakness linking.

---

# 264. Hard rule.

---

# 265. Testing

Need engineering-risk testkit.

---

# 266. Test Scenarios

```text
capacity risk
dependency concentration
accepted temporary security risk
systemic recurring incident weakness
technical debt portfolio
```

---

# 267. Intake Test

Risk without owner/evidence remains Analyzing.

---

# 268. Hard-Invariant Test

Cannot AcceptTemporarily tenant isolation failure.

---

# 269. Expiry Test

Expired acceptance returns risk Open.

---

# 270. Verification Test

Implemented remediation cannot close risk.

---

# 271. Portfolio Test

Action dependency graph remains acyclic.

---

# 272. Systemic Weakness Test

Multiple linked risks produce shared weakness.

---

# 273. Change Gate Test

Affected critical open risk surfaced.

---

# 274. Release Gate Test

Blocking risk prevents certification.

---

# 275. SLO Test

Repeated error-budget exhaustion can create risk.

---

# 276. Privacy Test

No employee/user dimension accepted.

---

# 277. Recurrence Test

PIR pattern links to risk/weakness.

---

# 278. Ownership Test

Team ownership persists through personnel changes.

---

# 279. Fuzzing

Fuzz:

```text
risk state transitions
acceptance records
remediation graphs
risk dependencies
policy rules
```

---

# 280. Property Tests

Properties:

```text
hard-invariant risk can never become AcceptedTemporarily
expired acceptance can never suppress an Open risk
risk can never become Resolved without verification or non-applicability evidence
person identity can never become a valid risk aggregation dimension
```

---

# 281. Formal Verification Targets

Strong candidates:

```text
risk lifecycle
acceptance expiry
remediation verification
dependency DAG
```

---

# 282. Kani Candidate

state/expiry/verification invariants.

---

# 283. TLA+ Candidate

identify → analyze → accept/mitigate → verify → resolve.

---

# 284. Loom Candidate

concurrent acceptance expiry + remediation verification + release gate.

---

# 285. Performance

Risk governance is control-plane/offline work.

---

# 286. No request-path dependency.

---

# 287. Hard rule.

---

# 288. Storage

Separate:

```text
risk records
evidence references
technical debt
systemic weaknesses
risk acceptance
remediation actions
portfolios
verification evidence
```

---

# 289. No workforce/user risk warehouse.

---

# 290. Hard rule.

---

# 291. Partitioning

By:

```text
service
region
site
risk class
systemic weakness
```

---

# 292. No person/user partition.

---

# 293. Hard rule.

---

# 294. Crate Layout

Recommended:

```text
crates/
├── siar-risk-core/
├── siar-risk-register/
├── siar-technical-debt/
├── siar-systemic-weakness/
├── siar-risk-acceptance/
├── siar-risk-remediation/
├── siar-remediation-portfolio/
├── siar-risk-verification/
├── siar-risk-observability/
└── siar-risk-testkit/
```

---

# 295. `siar-risk-core`

Owns:

```text
EngineeringRiskId
EngineeringRiskClass
RiskDimensions
EngineeringRiskError
```

---

# 296. `siar-risk-register`

Intake/lifecycle/ownership/freshness.

---

# 297. `siar-technical-debt`

Debt classes/burden/state.

---

# 298. `siar-systemic-weakness`

Cross-service/common-mode weakness model.

---

# 299. `siar-risk-acceptance`

Time-bounded exceptions/compensating controls.

---

# 300. `siar-risk-remediation`

Remediation action lifecycle.

---

# 301. `siar-remediation-portfolio`

Cross-risk action planning/dependencies.

---

# 302. `siar-risk-verification`

Evidence-backed closure.

---

# 303. `siar-risk-observability`

Aggregate risk-governance health only.

---

# 304. `siar-risk-testkit`

lifecycle/acceptance/portfolio/privacy tests.

---

# 305. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. Engineering risks are scoped to services, components, dependencies, regions, sites, fleets, tenants, federation peers, or platform infrastructure and never to individual users or employee performance.
2. Risk statements, dimensions, priorities, acceptances, and closures are evidence-backed and cannot be represented solely by one opaque composite score.
3. Technical debt, defects, vulnerabilities, incidents, control gaps, known limitations, and engineering risks remain distinct entities linked explicitly rather than collapsed into a generic issue record.
4. Temporary risk acceptance is scoped, approved, compensated where possible, and expires automatically; hard security/privacy invariants cannot be accepted as ordinary engineering risk.
5. A remediation action is not considered effective merely because code/config changed; closure requires verification evidence or proof that the risk no longer applies.
6. Systemic weaknesses are modeled across shared dependencies, common software/configuration, repeated incidents, missing guardrails, recovery gaps, and capacity constraints without creating people-centered blame graphs.
7. Remediation portfolios may optimize sequencing and engineering effort but cannot hide, average away, or silently downgrade unresolved critical risks because delivery capacity is limited.
8. Open engineering risk integrates with change and release gates, SLOs, resilience, capacity, forecasts, hardware/facility state, and post-incident reviews while urgent security/recovery actions retain governed emergency paths.
9. Risk aging, trend, recurrence, and prioritization use technical evidence and cannot become employee ranking, debt-by-developer scoring, or user-risk profiling.
10. Ownership is primarily service/team/governance scoped so risk history remains stable across personnel changes and does not become an individual blame ledger.
11. Risk evidence, dashboards, exports, and federation/tenant summaries expose only the minimum technical context required and cannot become a warehouse of private content or behavioral data.
12. Engineering-risk governance integrates with audit, compliance, incident response, SOC, vulnerability management, DR, crisis command, PIR, change/release governance, and organizational ownership without creating an alternate route around hard security/privacy controls.
```

---

# 306. Initial Production Scope

Implement first:

```text
typed EngineeringRiskId/EngineeringRiskClass
risk intake/lifecycle
explicit RiskDimensions
categorical priority
team/service ownership
risk evidence/confidence
technical debt registry
systemic weakness registry
risk-dependency graph
temporary acceptance with expiry
compensating controls
remediation action lifecycle
verification-before-closure
remediation portfolios
risk freshness/review cadence
change/release gates
SLO/resilience/capacity/PIR integration
privacy-safe dashboards
risk testkit
```

Then add:

```text
automated technical duplicate detection
risk graph visualization
portfolio optimization constrained by policy
cross-incident systemic weakness inference
formal acceptance/closure verification
risk-aware architecture review automation
```

---

# 307. Definition of Done

Part 121 is complete when:

- risk classes/scopes/states are typed;
- risk evidence/confidence is explicit;
- no opaque global score is required;
- technical debt and risk are distinct;
- systemic weaknesses can link multiple risks;
- temporary acceptance expires automatically;
- hard-invariant risk cannot be accepted;
- remediation must be verified;
- portfolios can group dependent actions;
- critical open risks integrate with change/release gates;
- risk ownership is team/service scoped;
- no employee/user risk profiling exists;
- lifecycle/acceptance/portfolio/privacy/fuzz/formal tests are specified.

---

# 308. Final Architecture

```text
                  TECHNICAL WEAKNESS
                         │
                         ▼
                    RISK REGISTER
                         │
            ┌────────────┼────────────┐
            │            │            │
         EVIDENCE     DIMENSIONS    OWNERSHIP
            │            │            │
            └────────────┼────────────┘
                         ▼
               TREATMENT / ACCEPTANCE
                         │
                         ▼
                REMEDIATION PORTFOLIO
                         │
                         ▼
                    VERIFICATION
                         │
                         ▼
                 RESOLVE / REASSESS
```

Engineering-risk safety model:

```text
typed risks
+
explicit dimensions
+
evidence/confidence
+
time-bounded acceptance
+
systemic weakness linkage
+
verified remediation
+
release/change integration
+
privacy/blame safeguards
```

not:

```text
assign one risk score, rank engineers by debt, accept issues forever, and close risk when a ticket is moved to Done
```

---

# 309. Final Principle

A risk register is useful only when it turns known weaknesses into verifiable engineering change.

The correct model is:

```text
identify technical risk
+
state evidence and uncertainty
+
separate debt from risk
+
track systemic weakness
+
assign durable ownership
+
accept temporarily only with expiry
+
build remediation portfolios
+
verify reduction before closure
+
never turn engineering risk into people scoring
```

This architecture gives SIAR a privacy-preserving engineering-risk foundation for reliability risk registers, technical debt governance, systemic weakness tracking, accepted exceptions, remediation portfolios, verification, and risk-aware change/release governance while preserving the anonymity, local-first, least-authority, post-incident-learning, and anti-surveillance guarantees established across Parts 34–120.
