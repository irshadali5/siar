# Core System Architecture Part 120 — Anonymous Network Post-Incident Review, Root-Cause Analysis, Corrective Actions, Organizational Learning, Recurrence Prevention & Privacy-Preserving Reliability Improvement Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 120  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 65, 94–100, 104–119

**Primary purpose:** define SIAR's post-incident learning architecture for evidence-preserving review, root-cause and causal-factor analysis, contributing-condition modeling, corrective/preventive actions, recurrence prevention, action ownership, verification, systemic learning, reliability improvement, and privacy-preserving organizational learning without employee or user surveillance.

---

# 1. Purpose

Incident response restores service.

Post-incident review improves the system.

A mature platform must answer:

```text
What happened?
Why did it happen?
Why did safeguards not prevent it?
What made detection or recovery slower?
Which conditions allowed recurrence?
Which corrective actions are actually verified?
What should change in architecture, tests, runbooks, policy, or training?
```

The governing principle is:

> **SIAR post-incident review should explain system behavior, identify causal and contributing conditions, produce verifiable corrective actions, and reduce recurrence without turning incident analysis into employee blame, productivity scoring, or user-behavior surveillance.**

---

# 2. Architectural Position

```text
                    INCIDENT CLOSED
                          │
                          ▼
                    EVIDENCE FREEZE
                          │
                          ▼
                    REVIEW ASSEMBLY
                          │
             ┌────────────┼────────────┐
             │            │            │
          TIMELINE      CAUSES      CONTROLS
             │            │            │
             └────────────┼────────────┘
                          ▼
                 CORRECTIVE ACTIONS
                          │
                          ▼
                  VERIFICATION LOOP
                          │
                          ▼
               RELIABILITY IMPROVEMENT
```

---

# 3. Core Separation

Keep distinct:

```text
root cause
contributing factor
trigger
impact
detection gap
response gap
control failure
corrective action
preventive action
lesson learned
```

---

# 4. Non-Goals

Part 120 does not create:

```text
employee blame scoring
operator ranking
user-behavior mining
performance-review inputs
single-person blame narratives
```

---

# 5. Review Eligibility

Not every event needs full review.

---

# 6. Review Trigger

```rust
pub enum PostIncidentReviewTrigger {
    Sev1OrHigher,
    SecurityIncident,
    PrivacyIncident,
    DataIntegrityIncident,
    MajorFacilityIncident,
    RepeatedFailure,
    ErrorBudgetExhaustion,
    GovernanceRequest,
}
```

---

# 7. Review Requirement

```rust
pub struct ReviewRequirement {
    pub incident: IncidentId,
    pub trigger: PostIncidentReviewTrigger,
    pub required: bool,
}
```

---

# 8. Hard Rule

Mandatory review cannot be skipped merely because service has recovered.

---

# 9. Review State

```rust
pub enum PostIncidentReviewState {
    Pending,
    CollectingEvidence,
    Analyzing,
    Drafting,
    Reviewing,
    Accepted,
    ActionsOpen,
    VerifiedClosed,
}
```

---

# 10. No Pending→VerifiedClosed

Hard rule.

---

# 11. Review Identity

```rust
pub struct ReviewId(pub [u8; 16]);
```

---

# 12. Review Record

```rust
pub struct PostIncidentReview {
    pub review_id: ReviewId,
    pub incident: IncidentId,
    pub state: PostIncidentReviewState,
    pub owner: ReviewOwnerRef,
}
```

---

# 13. Review Owner

Usually incident/service owner.

---

# 14. Owner ≠ sole author

Hard rule.

---

# 15. Independence

High-severity or security/privacy incidents may require independent reviewer.

---

# 16. Hard rule.

---

# 17. Evidence Preservation

Use Part 94/96/119 evidence.

---

# 18. Evidence Types

```rust
pub enum ReviewEvidenceType {
    IncidentTimeline,
    ChangeReceipt,
    AuditRecord,
    Alert,
    SloEvaluation,
    CapacitySnapshot,
    TopologySnapshot,
    FacilityState,
    HardwareAttestation,
    RecoveryEvidence,
}
```

---

# 19. Evidence Reference

```rust
pub struct ReviewEvidenceRef {
    pub evidence_type: ReviewEvidenceType,
    pub reference: EvidenceRef,
}
```

---

# 20. No Raw Data Copy By Default

Hard rule.

---

# 21. Evidence Freeze

Preserve relevant immutable references.

---

# 22. Hard rule.

---

# 23. Timeline Reconstruction

Build a factual sequence.

---

# 24. Timeline Entry

```rust
pub struct ReviewTimelineEntry {
    pub at: Timestamp,
    pub event: TimelineEventKind,
    pub evidence: Vec<EvidenceRef>,
    pub confidence: EvidenceConfidence,
}
```

---

# 25. Evidence Confidence

```rust
pub enum EvidenceConfidence {
    Low,
    Medium,
    High,
    Verified,
}
```

---

# 26. Unknown ≠ Fact

Hard rule.

---

# 27. Timeline Sources

Examples:

```text
monitoring
audit
change records
incident notes
facility/hardware events
```

---

# 28. No private user content baseline.

---

# 29. Hard rule.

---

# 30. Root-Cause Model

Avoid simplistic single-cause narratives.

---

# 31. Cause Categories

```rust
pub enum CausalFactorClass {
    DesignDefect,
    ImplementationDefect,
    ConfigurationError,
    CapacityShortfall,
    DependencyFailure,
    ProcessGap,
    DetectionGap,
    RecoveryGap,
    DocumentationGap,
    SecurityControlFailure,
    PhysicalInfrastructureFailure,
    Unknown,
}
```

---

# 32. Causal Factor

```rust
pub struct CausalFactor {
    pub id: CausalFactorId,
    pub class: CausalFactorClass,
    pub statement: CausalStatement,
    pub evidence: Vec<EvidenceRef>,
    pub confidence: EvidenceConfidence,
}
```

---

# 33. Hard Rule

A causal factor must be evidence-backed.

---

# 34. Trigger vs Root Cause

Trigger:

```text
the immediate event
```

Root cause:

```text
the deeper condition that made the event possible or damaging
```

---

# 35. Hard rule.

---

# 36. Contributing Conditions

```rust
pub struct ContributingCondition {
    pub id: ContributingConditionId,
    pub class: ContributingConditionClass,
    pub statement: CausalStatement,
}
```

---

# 37. Classes

```rust
pub enum ContributingConditionClass {
    LatentDefect,
    IncompleteAutomation,
    WeakAlerting,
    InadequateCapacity,
    DocumentationGap,
    OwnershipGap,
    TrainingGap,
    ProcessComplexity,
    ToolingGap,
    DependencyOpacity,
}
```

---

# 38. No Human Character Judgments

Hard rule.

---

# 39. Avoid:

```text
careless
incompetent
lazy
```

---

# 40. Prefer:

```text
runbook omitted rollback validation
approval policy allowed ambiguous ownership
tooling made unsafe action easy
```

---

# 41. Hard rule.

---

# 42. Five-Whys

Optional technique.

---

# 43. Not authoritative by itself.

---

# 44. Hard rule.

---

# 45. Fault Tree

Useful for complex incidents.

---

# 46. Cause Graph

```rust
pub struct CauseGraph {
    pub factors: Vec<CausalFactor>,
    pub edges: Vec<CausalEdge>,
}
```

---

# 47. Causal Edge

```rust
pub struct CausalEdge {
    pub from: CausalFactorId,
    pub to: CausalFactorId,
    pub relation: CausalRelation,
}
```

---

# 48. Relation

```rust
pub enum CausalRelation {
    Enabled,
    Amplified,
    DelayedDetection,
    DelayedRecovery,
    PreventedContainment,
}
```

---

# 49. Hard Rule

Causal graph is explanatory, not a blame graph.

---

# 50. Barrier Analysis

Ask:

```text
Which control should have prevented this?
Which control should have detected this?
Which control should have reduced impact?
```

---

# 51. Barrier Type

```rust
pub enum BarrierType {
    Prevention,
    Detection,
    Containment,
    Recovery,
}
```

---

# 52. Barrier Evaluation

```rust
pub struct BarrierEvaluation {
    pub control: ControlId,
    pub barrier_type: BarrierType,
    pub state: BarrierState,
    pub evidence: Vec<EvidenceRef>,
}
```

---

# 53. Barrier State

```rust
pub enum BarrierState {
    Worked,
    Failed,
    Missing,
    BypassedByDesign,
    Unknown,
}
```

---

# 54. Unknown ≠ Worked

Hard rule.

---

# 55. Detection Analysis

Measure:

```text
time to first signal
time to recognition
time to escalation
```

---

# 56. Detection Gap

```rust
pub struct DetectionGap {
    pub signal_expected: Option<SignalId>,
    pub observed_delay: Duration,
    pub cause: DetectionGapCause,
}
```

---

# 57. Cause

```rust
pub enum DetectionGapCause {
    MissingSignal,
    ThresholdPoor,
    AlertSuppressed,
    OwnershipUnclear,
    TelemetryUnavailable,
    Unknown,
}
```

---

# 58. Hard rule.

---

# 59. Response Analysis

Measure:

```text
time to containment
time to failover
time to recovery
```

---

# 60. Response Gap

```rust
pub struct ResponseGap {
    pub phase: ResponsePhase,
    pub delay: Duration,
    pub cause: ResponseGapCause,
}
```

---

# 61. Response Phase

```rust
pub enum ResponsePhase {
    Triage,
    Containment,
    Mitigation,
    Recovery,
    Requalification,
}
```

---

# 62. Hard rule.

---

# 63. Impact Analysis

Impact should be factual.

---

# 64. Impact Dimensions

```rust
pub enum IncidentImpactDimension {
    Availability,
    Latency,
    DataIntegrity,
    Security,
    Privacy,
    Recovery,
    Facility,
}
```

---

# 65. No User Emotion/Behavior Inference

Hard rule.

---

# 66. Impact Summary

```rust
pub struct IncidentImpactSummary {
    pub dimension: IncidentImpactDimension,
    pub scope: CrisisScope,
    pub duration: Option<Duration>,
    pub evidence: Vec<EvidenceRef>,
}
```

---

# 67. Error Budget Integration

Part 109.

---

# 68. Impact can include error-budget consumption.

---

# 69. Hard rule.

---

# 70. Change Correlation

Part 107.

Ask:

```text
Did a recent change contribute?
Was rollout policy followed?
Did canary/gates work?
```

---

# 71. Hard rule.

---

# 72. Release Correlation

Part 108.

---

# 73. No assumption new release caused incident without evidence.

---

# 74. Hard rule.

---

# 75. Capacity Correlation

Part 101/111.

---

# 76. Check:

```text
headroom
forecast accuracy
load shedding
autoscaling
```

---

# 77. Hard rule.

---

# 78. Resilience Correlation

Part 110.

---

# 79. Compare modeled vs actual failure behavior.

---

# 80. Hard rule.

---

# 81. Facility/Hardware Correlation

Parts 115–118.

---

# 82. Include:

```text
firmware
tamper
power/cooling
salvage
site recovery
```

---

# 83. Hard rule.

---

# 84. Corrective Action

Action that fixes discovered cause/control gap.

---

# 85. Corrective Action Record

```rust
pub struct CorrectiveAction {
    pub id: CorrectiveActionId,
    pub review: ReviewId,
    pub category: CorrectiveActionCategory,
    pub owner: ActionOwnerRef,
    pub due_at: Option<Timestamp>,
    pub state: CorrectiveActionState,
}
```

---

# 86. Categories

```rust
pub enum CorrectiveActionCategory {
    Code,
    Architecture,
    Configuration,
    Test,
    Monitoring,
    Capacity,
    Runbook,
    Process,
    SecurityControl,
    Training,
    Facility,
    Hardware,
}
```

---

# 87. Corrective Action State

```rust
pub enum CorrectiveActionState {
    Proposed,
    Approved,
    InProgress,
    Implemented,
    Verifying,
    Verified,
    Rejected,
}
```

---

# 88. Implemented ≠ Verified

Hard rule.

---

# 89. Verification Requirement

```rust
pub struct ActionVerificationRequirement {
    pub action: CorrectiveActionId,
    pub evidence_required: Vec<VerificationEvidenceType>,
}
```

---

# 90. Verification Evidence Type

```rust
pub enum VerificationEvidenceType {
    TestPass,
    SimulationPass,
    ReleaseReceipt,
    ConfigConvergence,
    DrillPass,
    SloImprovement,
    ControlEvaluation,
}
```

---

# 91. Hard rule.

---

# 92. Preventive Action

Broader recurrence reduction.

---

# 93. Example:

```text
make unsafe config impossible by type
add release gate
add invariant test
improve architecture
```

---

# 94. Hard rule.

---

# 95. Recurrence Prevention

Action should reduce probability or impact.

---

# 96. Recurrence Control

```rust
pub struct RecurrenceControl {
    pub action: CorrectiveActionId,
    pub target_failure_mode: FailureModeRef,
    pub prevention_layer: PreventionLayer,
}
```

---

# 97. Prevention Layer

```rust
pub enum PreventionLayer {
    CompilerTypeSystem,
    StaticValidation,
    UnitTest,
    PropertyTest,
    IntegrationTest,
    ReleaseGate,
    RuntimeGuard,
    Monitoring,
    Runbook,
    OrganizationalPolicy,
}
```

---

# 98. Strong Preference

Move prevention left when possible.

---

# 99. Hard rule.

---

# 100. Type-System Remediation

For Rust, prefer:

```text
invalid states unrepresentable
typed capability
bounded enum
newtype IDs
```

---

# 101. Good.

---

# 102. Hard rule.

---

# 103. Test Remediation

Convert incident into deterministic regression test where possible.

---

# 104. Hard rule.

---

# 105. Property Test Remediation

If class of bug:

```text
property-based invariant
```

---

# 106. Hard rule.

---

# 107. Fuzz Regression

Crash/security parser incident → fuzz corpus.

---

# 108. Hard rule.

---

# 109. Formal Verification Remediation

High-value state-machine failure may justify:

```text
TLA+
Kani
Loom
```

---

# 110. Good.

---

# 111. Runbook Remediation

Update steps only if human procedure was genuinely a factor.

---

# 112. Do not substitute docs for architectural fix where automation/type enforcement is possible.

---

# 113. Hard rule.

---

# 114. Monitoring Remediation

Add signal if detection gap.

---

# 115. No high-cardinality/user tracking.

---

# 116. Hard rule.

---

# 117. Capacity Remediation

If capacity failure:

```text
headroom
forecast
admission
load shedding
```

---

# 118. Hard rule.

---

# 119. Security Remediation

If control failed:

```text
policy
authz
credential
key rotation
segmentation
```

---

# 120. Hard rule.

---

# 121. Facility Remediation

Examples:

```text
generator
UPS
cooling
sensor
physical access
```

---

# 122. Hard rule.

---

# 123. Action Prioritization

Use severity/risk.

---

# 124. No Person-Based Priority

Hard rule.

---

# 125. Priority

```rust
pub enum CorrectivePriority {
    Critical,
    High,
    Medium,
    Low,
}
```

---

# 126. Critical Action

Blocks risky operation/release if necessary.

---

# 127. Hard rule.

---

# 128. Action Ownership

Assign team/service owner.

---

# 129. Not employee blame owner.

---

# 130. Hard rule.

---

# 131. Due Dates

Risk-based.

---

# 132. Hard rule.

---

# 133. Overdue Action

Escalate organizationally.

---

# 134. No automatic punitive scoring.

---

# 135. Hard rule.

---

# 136. Action Dependency

```rust
pub struct CorrectiveActionDependency {
    pub action: CorrectiveActionId,
    pub depends_on: CorrectiveActionId,
}
```

---

# 137. No cycles.

---

# 138. Hard rule.

---

# 139. Verification

Corrective action closes only when evidence shows intended control works.

---

# 140. Hard rule.

---

# 141. Verification State

```rust
pub enum VerificationState {
    Pending,
    Passed,
    Failed,
    Inconclusive,
}
```

---

# 142. Inconclusive ≠ Passed

Hard rule.

---

# 143. Recurrence Test

Where possible, reproduce failure before fix and prove prevention after fix.

---

# 144. Hard rule.

---

# 145. Review Acceptance

Review is accepted when:

```text
timeline supported
causal model supported
unknowns explicit
actions assigned
```

---

# 146. Hard rule.

---

# 147. Review Closure

Review remains open until critical actions verified.

---

# 148. Hard rule.

---

# 149. Residual Risk

Some actions may be deferred.

---

# 150. Residual Risk Record

```rust
pub struct ResidualRisk {
    pub review: ReviewId,
    pub statement: RiskStatement,
    pub owner: RiskOwnerRef,
    pub expires_at: Option<Timestamp>,
}
```

---

# 151. Explicit.

---

# 152. Hard rule.

---

# 153. Risk Acceptance

Requires appropriate authority.

---

# 154. Cannot accept hard security/privacy invariant violation.

---

# 155. Hard rule.

---

# 156. Organizational Learning

Incidents should feed reusable improvements.

---

# 157. Learning Artifact

```rust
pub enum LearningArtifact {
    Adr,
    RunbookUpdate,
    TestCase,
    TrainingScenario,
    PolicyUpdate,
    ArchitectureChange,
    SimulationScenario,
}
```

---

# 158. Hard rule.

---

# 159. Knowledge Base

Part 65.

---

# 160. Post-incident learning goes into structured docs/ADRs/runbooks.

---

# 161. No chat-only institutional memory.

---

# 162. Hard rule.

---

# 163. Incident Pattern Catalog

Detect recurring technical patterns.

---

# 164. Pattern Scope

```rust
pub enum IncidentPatternClass {
    Deployment,
    Database,
    Capacity,
    Dependency,
    Security,
    Facility,
    Hardware,
    Recovery,
}
```

---

# 165. No Employee Pattern Catalog

Hard rule.

---

# 166. Pattern Record

```rust
pub struct IncidentPattern {
    pub class: IncidentPatternClass,
    pub failure_mode: FailureModeRef,
    pub occurrence_count: u32,
}
```

---

# 167. Aggregate technical recurrence only.

---

# 168. Hard rule.

---

# 169. Trend Analysis

Allowed:

```text
incident class frequency
detection latency
recovery latency
action completion
```

---

# 170. Forbidden:

```text
which employee caused most incidents
who responds slowest
```

---

# 171. Hard rule.

---

# 172. Review Quality

Assess completeness, not author.

---

# 173. Good metrics:

```text
evidence coverage
unknowns resolved
critical actions verified
```

---

# 174. Hard rule.

---

# 175. Blamelessness

Blameless means:

```text
focus on system conditions
```

not:

```text
ignore accountability or intentional misuse
```

---

# 176. Hard rule.

---

# 177. Intentional Misconduct Boundary

If credible evidence of intentional malicious action exists:

```text
security/legal process
```

separate from reliability review.

---

# 178. Hard rule.

---

# 179. Human Error

Treat as system signal.

Ask:

```text
Why was unsafe action possible?
Why was guardrail absent?
Why was interface ambiguous?
```

---

# 180. Hard rule.

---

# 181. Automation Opportunity

Prefer removing repetitive failure-prone manual steps.

---

# 182. Hard rule.

---

# 183. Review Meeting

Optional.

---

# 184. Artifact is canonical.

---

# 185. Hard rule.

---

# 186. Review Distribution

Need-to-know based on sensitivity.

---

# 187. Public/internal/security-sensitive versions may differ.

---

# 188. Hard rule.

---

# 189. Redaction

Remove:

```text
private user content
secrets
unnecessary employee details
facility-sensitive detail
```

---

# 190. Hard rule.

---

# 191. Security Incident Review

May contain restricted details.

---

# 192. Share sanitized lessons broadly.

---

# 193. Hard rule.

---

# 194. Privacy Incident Review

Must include:

```text
data scope
control gap
containment
deletion/retention effect
```

---

# 195. No unnecessary identity expansion.

---

# 196. Hard rule.

---

# 197. Facility Incident Review

Parts 116–118.

Include:

```text
power/cooling
sensor behavior
evacuation
requalification
```

---

# 198. Hard rule.

---

# 199. Crisis Command Review

Part 119.

Review:

```text
activation
role handoff
decision quality
authority expiry
communications
```

---

# 200. Not commander personality.

---

# 201. Hard rule.

---

# 202. SLO Integration

Part 109.

---

# 203. Review may adjust SLO only through governed process.

---

# 204. Never lower SLO retroactively.

---

# 205. Hard rule.

---

# 206. Resilience Model Update

Part 110.

---

# 207. Incident may invalidate assumptions.

---

# 208. Hard rule.

---

# 209. Forecast Update

Part 111.

---

# 210. Capacity incident may recalibrate forecast.

---

# 211. Hard rule.

---

# 212. Performance Update

Part 112.

---

# 213. Tail-latency/performance incident becomes benchmark regression scenario.

---

# 214. Hard rule.

---

# 215. Efficiency/Sustainability Update

Parts 113–114.

---

# 216. Avoid learning that saves cost/energy at expense of reliability/privacy.

---

# 217. Hard rule.

---

# 218. Hardware/Facility Update

Parts 115–118.

---

# 219. Update maintenance, firmware, facility, disaster procedures as evidence indicates.

---

# 220. Hard rule.

---

# 221. Change Integration

Part 107.

---

# 222. Corrective implementation is governed change.

---

# 223. Hard rule.

---

# 224. Release Integration

Part 108.

---

# 225. Critical unresolved corrective actions may block launch.

---

# 226. Hard rule.

---

# 227. Organizational Governance

Part 104.

---

# 228. Ownership and escalation explicit.

---

# 229. No employee ranking.

---

# 230. Hard rule.

---

# 231. Audit Integration

Part 94.

Audit:

```text
review accepted
critical action closed
risk accepted
```

---

# 232. Not every comment/edit.

---

# 233. Hard rule.

---

# 234. Compliance Integration

Part 95.

Controls may require:

```text
PIR completion
root-cause evidence
action verification
recurrence prevention
```

---

# 235. Hard rule.

---

# 236. Incident Response Integration

Part 96.

---

# 237. Incident timeline/evidence imported by reference.

---

# 238. Hard rule.

---

# 239. SOC Integration

Part 97.

---

# 240. Detection gaps feed rule engineering.

---

# 241. Hard rule.

---

# 242. Vulnerability Integration

Part 98.

---

# 243. Vulnerability-caused incident updates remediation policy.

---

# 244. Hard rule.

---

# 245. Update Integration

Part 99.

---

# 246. Update failures feed rollout/rollback safety.

---

# 247. Hard rule.

---

# 248. DR Integration

Part 100.

---

# 249. Recovery failures feed drill/runbook/topology improvements.

---

# 250. Hard rule.

---

# 251. Review Service

```rust
pub trait PostIncidentReviewService {
    fn create(
        &self,
        incident: IncidentId,
    ) -> Result<PostIncidentReview, PostIncidentError>;

    fn accept(
        &self,
        review: ReviewId,
    ) -> Result<ReviewReceipt, PostIncidentError>;
}
```

---

# 252. Causal Analysis Service

```rust
pub trait CausalAnalysisService {
    fn add_factor(
        &self,
        review: ReviewId,
        factor: CausalFactor,
    ) -> Result<(), PostIncidentError>;
}
```

---

# 253. Corrective Action Service

```rust
pub trait CorrectiveActionService {
    fn propose(
        &self,
        action: CorrectiveAction,
    ) -> Result<CorrectiveActionId, PostIncidentError>;

    fn verify(
        &self,
        action: CorrectiveActionId,
        evidence: Vec<EvidenceRef>,
    ) -> Result<VerificationState, PostIncidentError>;
}
```

---

# 254. Pattern Service

```rust
pub trait IncidentPatternService {
    fn patterns(
        &self,
        scope: IncidentPatternClass,
    ) -> Result<Vec<IncidentPattern>, PostIncidentError>;
}
```

---

# 255. No Employee Pattern API

Hard rule.

---

# 256. Error Taxonomy

```rust
pub enum PostIncidentError {
    IncidentUnknown,
    ReviewAlreadyExists,
    EvidenceMissing,
    CausalClaimUnsupported,
    ActionOwnerMissing,
    VerificationFailed,
    ResidualRiskUnapproved,
    ReviewNotReady,
    ScopeViolation,
    Unauthorized,
    Internal,
}
```

---

# 257. Observability

Safe metrics:

```text
reviews pending
critical actions open
verification success
recurrence by technical class
time to verified closure
```

---

# 258. Forbidden:

```text
employee incident count
operator error leaderboard
user behavior
private content
```

---

# 259. Hard rule.

---

# 260. Post-Incident SLOs

Examples:

```text
Sev1 review started within target
critical actions assigned within target
verification completed by due date
```

---

# 261. Security SLO

```text
0 critical security corrective action closed without evidence
0 unsupported causal conclusion marked Verified
```

---

# 262. Privacy SLO

```text
0 PIR workforce-performance scoring
0 user-level behavioral analysis in recurrence modeling
```

---

# 263. Failure Modes

```text
review never completed
actions implemented but unverified
blame narrative replaces systems analysis
repeat incident not linked to prior pattern
```

---

# 264. Review Stall

Escalate ownership.

---

# 265. No punitive automation.

---

# 266. Hard rule.

---

# 267. Unverified Action

State remains Verifying/Pending.

---

# 268. Hard rule.

---

# 269. Blame Drift

Review quality gate rejects unsupported personal characterization.

---

# 270. Hard rule.

---

# 271. Repeat Incident

Pattern service links technical failure mode.

---

# 272. Hard rule.

---

# 273. Testing

Need post-incident testkit.

---

# 274. Test Scenarios

```text
deployment incident
capacity exhaustion
security incident
facility outage
recovery failure
```

---

# 275. Evidence Test

Causal factor without evidence cannot be Verified.

---

# 276. Unknown Test

Unknown remains explicit.

---

# 277. Timeline Test

Event confidence preserved.

---

# 278. Cause Graph Test

Cycles allowed only if semantics permit? Prefer DAG for explanation.

---

# 279. Hard rule.

---

# 280. Action Test

Implemented != Verified.

---

# 281. Verification Test

Action closes only with required evidence.

---

# 282. Residual Risk Test

Unapproved critical residual risk blocks closure.

---

# 283. Pattern Test

Technical recurrence links correctly.

---

# 284. Privacy Test

No employee leaderboard API.

---

# 285. User Privacy Test

No message content needed for reliability review.

---

# 286. Change Test

Corrective implementation gets governed change ID.

---

# 287. Release Test

Critical open actions can block launch.

---

# 288. SLO Test

Historical SLO not retroactively changed.

---

# 289. Fuzzing

Fuzz:

```text
review states
causal graphs
action dependencies
verification requirements
residual-risk records
```

---

# 290. Property Tests

Properties:

```text
unsupported causal claim can never become Verified
corrective action can never reach Verified without required evidence
review closure can never leave mandatory critical actions unowned
person identity can never become a valid recurrence-analysis dimension
```

---

# 291. Formal Verification Targets

Strong candidates:

```text
review lifecycle
corrective-action verification
residual-risk acceptance
action dependency graph
```

---

# 292. Kani Candidate

state-transition and verification invariants.

---

# 293. TLA+ Candidate

incident closed → review → actions → verify → closure.

---

# 294. Loom Candidate

concurrent action verification + review closure + risk acceptance.

---

# 295. Performance

Post-incident review is offline/control-plane work.

---

# 296. No production request-path dependency.

---

# 297. Hard rule.

---

# 298. Storage

Separate:

```text
review records
evidence references
timeline entries
causal factors
barrier evaluations
corrective actions
verification evidence
residual risks
pattern catalog
```

---

# 299. No employee-performance warehouse.

---

# 300. Hard rule.

---

# 301. Partitioning

By:

```text
incident
service
region
site
technical failure class
```

---

# 302. No user/person partition.

---

# 303. Hard rule.

---

# 304. Crate Layout

Recommended:

```text
crates/
├── siar-pir-core/
├── siar-pir-evidence/
├── siar-causal-analysis/
├── siar-barrier-analysis/
├── siar-corrective-actions/
├── siar-action-verification/
├── siar-incident-patterns/
├── siar-organizational-learning/
├── siar-pir-observability/
└── siar-pir-testkit/
```

---

# 305. `siar-pir-core`

Owns:

```text
ReviewId
PostIncidentReviewState
PostIncidentError
```

---

# 306. `siar-pir-evidence`

Evidence refs/timeline/confidence.

---

# 307. `siar-causal-analysis`

Causal factors/contributing conditions/cause graph.

---

# 308. `siar-barrier-analysis`

Prevention/detection/containment/recovery controls.

---

# 309. `siar-corrective-actions`

Action lifecycle/ownership/dependencies.

---

# 310. `siar-action-verification`

Evidence-backed verification.

---

# 311. `siar-incident-patterns`

Technical recurrence classes.

---

# 312. `siar-organizational-learning`

ADRs/runbooks/tests/training/policy feedback.

---

# 313. `siar-pir-observability`

Aggregate review/process health only.

---

# 314. `siar-pir-testkit`

causal/action/privacy/formal tests.

---

# 315. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. Post-incident review focuses on technical/systemic causes, contributing conditions, control failures, and recovery behavior—not employee blame, productivity ranking, or user behavior.
2. Causal claims, timelines, barrier evaluations, and impact statements must reference evidence and confidence; unknown or disputed facts remain explicitly uncertain.
3. Trigger, root cause, contributing condition, impact, detection gap, response gap, and corrective action remain distinct concepts and cannot be collapsed into a single simplistic explanation.
4. Corrective actions are not complete when code/config is merely changed; they reach Verified only after evidence demonstrates the intended recurrence-prevention control works.
5. High-value recurrence prevention should move left where practical into Rust types, static validation, property tests, fuzzing, formal models, release gates, or runtime guards rather than relying only on human memory.
6. Critical unresolved actions and unaccepted residual risks can block risky changes/releases according to policy, while urgent security/recovery work remains possible through governed emergency paths.
7. Incident-pattern analysis is scoped to technical failure modes, services, regions, dependencies, or facilities and can never become a database of which employee "causes incidents."
8. Post-incident evidence uses references/minimized extracts and cannot justify broad collection of private message content, user social graphs, workforce activity histories, or unrelated personal data.
9. Blameless review does not suppress intentional malicious behavior; credible misconduct/security concerns move to the appropriate security/legal process instead of being disguised as reliability analysis.
10. SLOs, policies, forecasts, architecture assumptions, and resilience models may be updated prospectively from incident evidence but historical targets/results cannot be rewritten retroactively to make the incident look better.
11. Review learning feeds ADRs, runbooks, tests, simulations, controls, training, and architecture and cannot remain only in transient chat or meeting memory.
12. Post-incident review integrates with incident response, SOC, audit, compliance, changes, releases, SLOs, resilience, DR, capacity, hardware, facility, physical security, and crisis command without creating an alternate governance path or surveillance system.
```

---

# 316. Initial Production Scope

Implement first:

```text
typed ReviewId/PostIncidentReviewState
automatic PIR trigger rules
evidence-reference freeze
timeline reconstruction
fact/confidence model
causal factors/contributing conditions
barrier analysis
detection/response gap model
impact summaries
corrective-action lifecycle
owners/due dates
action dependencies
verification requirements
residual-risk tracking
technical incident-pattern catalog
learning artifacts
release/change integration
privacy-safe review dashboards
PIR testkit
```

Then add:

```text
cause-graph visualization
automated typed evidence summarization
regression-test generation helpers
formal action-closure verification
cross-incident pattern mining on technical features only
organizational-learning quality gates
```

---

# 317. Definition of Done

Part 120 is complete when:

- review triggers/states are typed;
- timelines preserve evidence confidence;
- root cause and contributing factors are distinct;
- barrier/detection/response analysis exists;
- personal blame labels are prohibited;
- corrective actions have owners and due dates;
- implemented actions require verification;
- recurrence controls can feed code/tests/policy/runbooks;
- residual risk is explicit;
- technical patterns can be tracked across incidents;
- critical unresolved actions can influence release/change gates;
- historical SLOs cannot be rewritten;
- no employee/user surveillance is created;
- causal/action/privacy/fuzz/formal tests are specified.

---

# 318. Final Architecture

```text
                     INCIDENT CLOSED
                           │
                           ▼
                     EVIDENCE FREEZE
                           │
                           ▼
                     FACTUAL TIMELINE
                           │
              ┌────────────┼────────────┐
              │            │            │
            CAUSES       BARRIERS      GAPS
              │            │            │
              └────────────┼────────────┘
                           ▼
                   CORRECTIVE ACTIONS
                           │
                           ▼
                    VERIFICATION LOOP
                           │
                           ▼
                  ORGANIZATIONAL LEARNING
                           │
                           ▼
                  RECURRENCE PREVENTION
```

Post-incident improvement model:

```text
evidence-backed facts
+
multi-factor causal analysis
+
barrier evaluation
+
verifiable corrective actions
+
systemic prevention
+
technical recurrence tracking
+
learning feedback loops
+
privacy/blamelessness safeguards
```

not:

```text
find the person who made the mistake, write a meeting note, mark action items done when code merges, and forget the incident until it happens again
```

---

# 319. Final Principle

A post-incident review is successful only when it changes the system in a way that can be verified.

The correct model is:

```text
preserve evidence
+
reconstruct facts
+
separate trigger from root cause
+
identify contributing conditions
+
evaluate failed/missing barriers
+
assign corrective actions
+
verify the fix
+
convert lessons into durable architecture/tests/policy
+
never turn reliability learning into surveillance or blame scoring
```

This architecture gives SIAR a privacy-preserving post-incident learning foundation for root-cause analysis, causal-factor modeling, barrier analysis, corrective action verification, recurrence prevention, technical trend analysis, organizational learning, and reliability improvement while preserving the anonymity, local-first, least-authority, crisis-command, facility-continuity, and anti-surveillance guarantees established across Parts 34–119.
