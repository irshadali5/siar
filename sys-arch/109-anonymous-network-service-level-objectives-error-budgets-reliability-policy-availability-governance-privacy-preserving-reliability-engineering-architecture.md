# Core System Architecture Part 109 — Anonymous Network Service Level Objectives, Error Budgets, Reliability Policy, Availability Governance & Privacy-Preserving Reliability Engineering Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 109  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 50–52, 63–70, 92–108

**Primary purpose:** define SIAR's reliability-engineering architecture for service-level indicators, service-level objectives, error budgets, burn-rate governance, dependency SLOs, service criticality, availability policy, maintenance accounting, overload and DR integration, launch/change gates, alerting, multi-region/federation reliability, and privacy-preserving reliability telemetry.

---

# 1. Purpose

A reliable system needs objective answers to:

```text
What does "healthy" mean?
Which failures matter?
How much unreliability is acceptable?
When should changes stop?
When should reliability work take precedence over feature work?
Which dependency is violating our reliability envelope?
```

The governing principle is:

> **SIAR reliability should be governed by explicit service-level indicators, objective targets, bounded error budgets, and typed policy—without using user-level behavioral telemetry as the basis for reliability decisions.**

---

# 2. Architectural Position

```text
Service / Dependency Signals
            │
            ▼
      Service-Level Indicators
            │
            ▼
     Service-Level Objectives
            │
            ▼
       Error Budget Ledger
            │
      ┌─────┼──────────┐
      │     │          │
   Healthy Warning   Exhausted
      │     │          │
      └─────┼──────────┘
            ▼
     Reliability Governance
            │
   ┌────────┼────────┐
   │        │        │
 Alert   Restrict   Recover
```

---

# 3. Core Separation

Keep distinct:

```text
SLI
SLO
SLA
error budget
availability target
maintenance policy
incident severity
```

---

# 4. Non-Goals

Part 109 does not create:

```text
per-user uptime scoring
engagement-based reliability
opaque "health scores"
SLA policy baked into monitoring agents
availability achieved by privacy downgrade
```

---

# 5. Service-Level Indicator

An SLI is a measurable technical signal.

```rust
pub struct ServiceLevelIndicator {
    pub id: SliId,
    pub service: ServiceId,
    pub kind: SliKind,
    pub scope: SliScope,
}
```

---

# 6. SLI Kinds

```rust
pub enum SliKind {
    Availability,
    SuccessRate,
    Latency,
    Freshness,
    Durability,
    Correctness,
    QueueDelay,
    RecoveryTime,
}
```

---

# 7. Availability

Can a valid request be served?

---

# 8. Success Rate

Did valid operations succeed?

---

# 9. Latency

Did they complete within target?

---

# 10. Freshness

Is data/state current enough?

---

# 11. Durability

Was committed data retained?

---

# 12. Correctness

Did system preserve intended semantics?

---

# 13. Queue Delay

How long did deferred work wait?

---

# 14. Recovery Time

How quickly did service recover?

---

# 15. Hard Rule

SLIs are technical service measures, not behavioral or business-engagement metrics.

---

# 16. SLI Scope

```rust
pub enum SliScope {
    Service(ServiceId),
    Region(RegionId),
    TenantAggregate(TenantId),
    FederationPeer(FederationDomainId),
    WorkClass(WorkClass),
}
```

---

# 17. No Global User Scope

Hard rule.

---

# 18. Tenant Aggregate

Allowed only where managed tenant reporting is required.

---

# 19. No personal-user SLO.

---

# 20. Hard rule.

---

# 21. Measurement Window

```rust
pub enum MeasurementWindow {
    FiveMinutes,
    OneHour,
    OneDay,
    SevenDays,
    TwentyEightDays,
    NinetyDays,
}
```

---

# 22. SLO Window

Explicit.

---

# 23. Hard rule.

---

# 24. Service-Level Objective

```rust
pub struct ServiceLevelObjective {
    pub id: SloId,
    pub sli: SliId,
    pub target: SloTarget,
    pub window: MeasurementWindow,
    pub policy: ReliabilityPolicyRef,
}
```

---

# 25. SLO Target

```rust
pub enum SloTarget {
    AvailabilityPercent(FixedPoint),
    SuccessPercent(FixedPoint),
    LatencyPercentile {
        percentile: FixedPoint,
        threshold: Duration,
    },
    Freshness(Duration),
    DurabilityPercent(FixedPoint),
    RecoveryWithin(Duration),
}
```

---

# 26. No Floating-Point Policy

Use fixed point.

---

# 27. Hard rule.

---

# 28. SLO Example

```text
99.95% successful message submission over 28 days
```

---

# 29. Reliability Class

Not all services need same SLO.

```rust
pub enum ReliabilityClass {
    BestEffort,
    Standard,
    High,
    Critical,
}
```

---

# 30. BestEffort

Noncritical background capability.

---

# 31. Standard

Normal product service.

---

# 32. High

Important durable service.

---

# 33. Critical

Security/control/core communication service.

---

# 34. Hard Rule

Criticality must be declared, not inferred from traffic volume alone.

---

# 35. Reliability Profile

```rust
pub struct ReliabilityProfile {
    pub class: ReliabilityClass,
    pub objectives: Vec<SloId>,
    pub recovery_target: Option<Rto>,
    pub durability_target: Option<Rpo>,
}
```

---

# 36. Service Profile

Each production service has one.

---

# 37. No Ownerless / SLO-less Critical Service

Hard rule.

---

# 38. SLI Numerator / Denominator

For ratio SLIs:

```rust
pub struct RatioSliSample {
    pub good: u64,
    pub total: u64,
}
```

---

# 39. Valid Events Only

Malformed/unauthorized traffic may not belong in denominator depending policy.

---

# 40. Hard rule.

---

# 41. Denominator Policy

Explicit.

```rust
pub enum DenominatorPolicy {
    AllEligibleRequests,
    AuthenticatedEligibleRequests,
    AdmittedRequests,
    CompletedRequests,
}
```

---

# 42. Avoid Gaming

Cannot redefine denominator during outage.

---

# 43. Hard rule.

---

# 44. SLI Data Source

Preferred:

```text
service counters
queue state
health checks
durability confirmations
recovery events
```

---

# 45. Not:

```text
user clickstream
private message content
user engagement
```

---

# 46. Hard rule.

---

# 47. Privacy-Safe SLI Telemetry

Aggregate by:

```text
service
region
work class
tenant aggregate where needed
```

---

# 48. No stable user ID.

---

# 49. Hard rule.

---

# 50. Error Budget

Error budget is allowed unreliability.

---

# 51. Error Budget Definition

```rust
pub struct ErrorBudget {
    pub slo: SloId,
    pub allowed_bad: FixedPoint,
    pub consumed_bad: FixedPoint,
    pub remaining: FixedPoint,
}
```

---

# 52. Example

99.9% SLO permits 0.1% bad events.

---

# 53. Hard rule.

---

# 54. Error Budget State

```rust
pub enum ErrorBudgetState {
    Healthy,
    Warning,
    Critical,
    Exhausted,
}
```

---

# 55. Healthy

Plenty remains.

---

# 56. Warning

Elevated consumption.

---

# 57. Critical

Near exhaustion.

---

# 58. Exhausted

Target breached or budget depleted.

---

# 59. Hard Rule

Exhausted is explicit; no masking behind aggregate status.

---

# 60. Burn Rate

How quickly budget is being consumed.

```rust
pub struct BurnRate {
    pub short_window: FixedPoint,
    pub long_window: FixedPoint,
}
```

---

# 61. Multi-Window Burn Rate

Preferred.

---

# 62. Why

Avoid noisy single-window alerts.

---

# 63. Hard rule.

---

# 64. Burn Thresholds

```rust
pub struct BurnRateThresholds {
    pub fast: FixedPoint,
    pub slow: FixedPoint,
}
```

---

# 65. Alert Policy

Example:

```text
fast burn + slow burn => page
slow burn only => ticket
```

---

# 66. Hard rule.

---

# 67. Error Budget Ledger

Tracks objective consumption.

---

# 68. Ledger Entry

```rust
pub struct ErrorBudgetEntry {
    pub slo: SloId,
    pub period: BudgetPeriodId,
    pub bad_units: FixedPoint,
    pub source: SliSourceRef,
}
```

---

# 69. No User Identity

Hard rule.

---

# 70. Budget Reset

By measurement window.

---

# 71. No manual reset to hide outage.

---

# 72. Hard rule.

---

# 73. Reliability Policy

```rust
pub struct ReliabilityPolicy {
    pub version: ReliabilityPolicyVersion,
    pub class: ReliabilityClass,
    pub budget_actions: Vec<BudgetActionRule>,
}
```

---

# 74. Versioned/signed.

---

# 75. Anti-rollback.

---

# 76. Hard rule.

---

# 77. Budget Action Rule

```rust
pub struct BudgetActionRule {
    pub state: ErrorBudgetState,
    pub actions: Vec<ReliabilityGovernanceAction>,
}
```

---

# 78. Governance Actions

```rust
pub enum ReliabilityGovernanceAction {
    Alert,
    HoldRiskyChanges,
    RequireExtraApproval,
    RestrictFeatureLaunch,
    PrioritizeReliabilityWork,
    StartIncidentReview,
}
```

---

# 79. No "disable security/privacy".

Hard rule.

---

# 80. Error Budget Governance

Recommended:

```text
Healthy:
  normal changes

Warning:
  higher review for risky changes

Critical:
  hold nonessential risky launches

Exhausted:
  reliability work first
```

---

# 81. Hard rule.

---

# 82. Release Integration

Part 108.

---

# 83. Launch can be held if relevant service error budget is Critical/Exhausted.

---

# 84. Hard rule.

---

# 85. Change Integration

Part 107.

---

# 86. Risky change may require extra approval when budget low.

---

# 87. No blanket freeze for unrelated service.

---

# 88. Hard rule.

---

# 89. Service Dependencies

SLOs depend on downstream reliability.

---

# 90. Dependency SLO

```rust
pub struct DependencySlo {
    pub consumer: ServiceId,
    pub provider: ServiceId,
    pub required: SloTarget,
    pub criticality: DependencyCriticality,
}
```

---

# 91. Consumer Must Budget For Dependency Failure

Hard rule.

---

# 92. No Assuming 100% Dependency Availability

Hard rule.

---

# 93. Composite Reliability

Can be modeled.

---

# 94. But avoid false precision.

---

# 95. Hard rule.

---

# 96. Critical Path Reliability

Use Part 106 topology.

---

# 97. Query:

```text
which required dependencies dominate service availability?
```

---

# 98. Good.

---

# 99. Redundancy

Active-active dependency can improve service-level reliability.

---

# 100. But correlated failures matter.

---

# 101. Hard rule.

---

# 102. Correlated Failure Domains

Examples:

```text
same region
same provider
same CA
same database cluster
```

---

# 103. Must be modeled.

---

# 104. Hard rule.

---

# 105. Regional SLOs

Track region-specific health.

---

# 106. Global SLO

Can aggregate across regions.

---

# 107. But regional failures should remain visible.

---

# 108. Hard rule.

---

# 109. Multi-Region Availability

```rust
pub struct RegionalSlo {
    pub region: RegionId,
    pub objective: SloTarget,
}
```

---

# 110. No "global green" while one mandated region is failing.

---

# 111. Hard rule.

---

# 112. Residency-Aware Reliability

Part 103.

---

# 113. If compliant region unavailable:

```text
availability may degrade
```

rather than use forbidden region.

---

# 114. Hard rule.

---

# 115. DR Objectives

Part 100.

---

# 116. RTO/RPO integrated into reliability profile.

---

# 117. Recovery SLO

```rust
pub struct RecoverySlo {
    pub service: ServiceId,
    pub rto: Duration,
    pub rpo: Duration,
}
```

---

# 118. Measured in drills/incidents.

---

# 119. No theoretical-only claim.

---

# 120. Hard rule.

---

# 121. Durability SLO

Important for:

```text
messages
account state
audit
financial/tenant state
```

---

# 122. Example

```text
committed message persistence loss <= target
```

---

# 123. Hard rule.

---

# 124. Correctness SLO

Useful where:

```text
wrong result = failure even if request succeeded
```

---

# 125. Example:

```text
authorization decision correctness
message dedup correctness
```

---

# 126. Hard rule.

---

# 127. Security Reliability

Security controls need reliability too.

Examples:

```text
credential revocation propagation
policy distribution
key rotation
```

---

# 128. Hard rule.

---

# 129. Security SLO

```rust
pub enum SecuritySliKind {
    RevocationPropagation,
    PolicyPropagation,
    KeyRotationCompletion,
    AlertDelivery,
}
```

---

# 130. Security reliability cannot be deprioritized by product SLO.

---

# 131. Hard rule.

---

# 132. Privacy Reliability

Privacy guarantees can have operational objectives.

Examples:

```text
no direct fallback
cover traffic floor maintained
telemetry disabled in max anonymity
```

---

# 133. These are hard invariants, not ordinary error budgets where violation is unacceptable.

---

# 134. Hard rule.

---

# 135. Hard Invariant vs SLO

Different.

---

# 136. Hard invariant:

```text
must never happen
```

SLO:

```text
target reliability over window
```

---

# 137. Hard rule.

---

# 138. No Error Budget For

```text
plaintext secret leakage
tenant isolation breach
silent deanonymization
signature bypass
```

---

# 139. Hard rule.

---

# 140. Maintenance Windows

Planned maintenance can affect SLO accounting.

---

# 141. Must be explicit.

---

# 142. Maintenance Policy

```rust
pub enum MaintenanceAccountingPolicy {
    Included,
    ExcludedIfApproved,
    SeparateObjective,
}
```

---

# 143. No Retroactive Exclusion

Hard rule.

---

# 144. Approved Maintenance

Defined before event.

---

# 145. Hard rule.

---

# 146. Emergency Maintenance

Usually included unless policy says otherwise.

---

# 147. No gaming.

---

# 148. Hard rule.

---

# 149. Partial Availability

Need semantic treatment.

---

# 150. Example:

```text
text works
attachments unavailable
```

---

# 151. Use separate SLIs by capability.

---

# 152. Do not force one binary service-health metric.

---

# 153. Hard rule.

---

# 154. Capability SLO

```rust
pub struct CapabilitySlo {
    pub service: ServiceId,
    pub capability: CapabilityId,
    pub objective: SloTarget,
}
```

---

# 155. Good.

---

# 156. Work-Class SLOs

Part 101.

---

# 157. EmergencyRealtime may have stricter objective than Bulk.

---

# 158. Hard rule.

---

# 159. Queue SLO

Example:

```text
99% of direct messages leave queue within 5 seconds
```

---

# 160. Good.

---

# 161. Latency SLO

Prefer percentiles.

---

# 162. Average latency insufficient.

---

# 163. Hard rule.

---

# 164. Tail Latency

P95/P99 important.

---

# 165. But no false precision beyond sample quality.

---

# 166. Hard rule.

---

# 167. Error Classification

Not all failures equal.

---

# 168. Error Class

```rust
pub enum ReliabilityErrorClass {
    InternalFailure,
    DependencyFailure,
    CapacityRejected,
    ClientInvalid,
    AuthorizationDenied,
    Timeout,
    DataIntegrityFailure,
}
```

---

# 169. ClientInvalid / Auth Denied May Not Count As Reliability Failure

Policy-defined.

---

# 170. Hard rule.

---

# 171. Eligible Request

Typed.

---

# 172. Avoid denominator inflation/deflation.

---

# 173. Hard rule.

---

# 174. SLI Sampling

Aggregate counters preferred.

---

# 175. Sampling okay only if mathematically valid for metric.

---

# 176. Hard rule.

---

# 177. Missing Telemetry

Missing SLI data is not automatically healthy.

---

# 178. SLI Data State

```rust
pub enum SliDataState {
    Complete,
    Partial,
    Missing,
}
```

---

# 179. Missing Mandatory Telemetry

SLO status Unknown.

---

# 180. Hard rule.

---

# 181. SLO Evaluation State

```rust
pub enum SloEvaluationState {
    Meeting,
    AtRisk,
    Violated,
    Unknown,
}
```

---

# 182. Unknown ≠ Meeting

Hard rule.

---

# 183. Multi-Window Evaluation

Short and long window.

---

# 184. Good for burn alerts.

---

# 185. Hard rule.

---

# 186. Reliability Alerting

Alert on budget burn, not every single failure.

---

# 187. Hard rule.

---

# 188. Alert Types

```rust
pub enum ReliabilityAlert {
    FastBurn,
    SlowBurn,
    BudgetCritical,
    BudgetExhausted,
    TelemetryMissing,
}
```

---

# 189. Fast Burn

Page.

---

# 190. Slow Burn

Ticket/notification.

---

# 191. Hard rule.

---

# 192. Alert Privacy

Alert contains:

```text
service
SLO
burn rate
region
```

---

# 193. No user IDs.

---

# 194. Hard rule.

---

# 195. Reliability Incident

Part 96.

---

# 196. Error budget exhaustion may trigger incident/review.

---

# 197. Not every SLO miss is security incident.

---

# 198. Hard rule.

---

# 199. Reliability Review

After significant violation.

---

# 200. Inputs:

```text
timeline
dependency state
capacity
change history
incident evidence
```

---

# 201. No employee blame scoring.

---

# 202. Hard rule.

---

# 203. Postmortem Integration

Part 65.

---

# 204. Reliability policy changes feed:

```text
tests
capacity
runbooks
SLO targets
```

---

# 205. Hard rule.

---

# 206. Error Budget Policy & Product Velocity

Error budget may govern change rate.

---

# 207. But not arbitrary feature prioritization.

---

# 208. Hard rule.

---

# 209. Example Policy

```text
budget healthy:
  normal launch cadence

budget warning:
  additional release review

budget critical:
  only low-risk/reliability/security changes

budget exhausted:
  reliability/security recovery work
```

---

# 210. Good.

---

# 211. Security Patch Exception

Even exhausted reliability budget must not block urgent security patch.

---

# 212. Instead require safer rollout.

---

# 213. Hard rule.

---

# 214. DR / Incident Exception

Recovery changes allowed.

---

# 215. Hard rule.

---

# 216. Cost Pressure

Part 102.

---

# 217. Cost optimization cannot violate SLO floor without explicit approved product/service policy change.

---

# 218. Hard rule.

---

# 219. Capacity Pressure

Part 101.

---

# 220. Load shedding may intentionally sacrifice low-priority SLOs to preserve critical ones.

---

# 221. This should be explicit in policy.

---

# 222. Hard rule.

---

# 223. Brownout SLO

During brownout:

```text
core SLOs remain
optional capability SLOs may be suspended/degraded
```

---

# 224. No hidden SLO redefinition.

---

# 225. Hard rule.

---

# 226. Anonymous Network Reliability

Special considerations:

```text
mix latency
mailbox availability
cover traffic
relay availability
```

---

# 227. Do not optimize reliability by deanonymizing users.

---

# 228. Hard rule.

---

# 229. Mixnet SLI

Possible:

```text
packet delivery success
latency distribution
directory freshness
```

---

# 230. Aggregate.

---

# 231. No source-destination correlation.

---

# 232. Hard rule.

---

# 233. Mailbox SLI

Examples:

```text
store success
fetch success
delivery freshness
```

---

# 234. No mailbox-owner identity in central telemetry.

---

# 235. Hard rule.

---

# 236. Relay SLI

Examples:

```text
connection success
bandwidth availability
latency
```

---

# 237. No user-linkable flow history.

---

# 238. Hard rule.

---

# 239. Federation SLO

Peer-level.

---

# 240. Examples:

```text
handshake success
message exchange success
directory refresh
```

---

# 241. No remote user metrics.

---

# 242. Hard rule.

---

# 243. Federation Error Budget

Per peer/domain.

---

# 244. Local operator controls own objective.

---

# 245. No transitive SLA.

---

# 246. Hard rule.

---

# 247. Tenant Reliability

Managed tenants may have contractual SLOs.

---

# 248. Tenant aggregate only.

---

# 249. No user breakdown.

---

# 250. Hard rule.

---

# 251. SLA Boundary

SLA is contractual/business layer.

---

# 252. SLO is engineering objective.

---

# 253. Keep separate.

---

# 254. Hard rule.

---

# 255. SLA Adapter

```rust
pub struct SlaMapping {
    pub external_sla_id: ExternalSlaId,
    pub internal_slos: Vec<SloId>,
}
```

---

# 256. One SLA may map to multiple SLOs.

---

# 257. Good.

---

# 258. SLA Breach Handling

Financial/legal/business process separate.

---

# 259. Reliability engine emits verified facts.

---

# 260. Hard rule.

---

# 261. Reliability Ownership

Every SLO has owner.

---

# 262. Usually service owner.

---

# 263. Hard rule.

---

# 264. SLO Ownership Record

```rust
pub struct SloOwnership {
    pub slo: SloId,
    pub owner: ServiceOwnerRef,
}
```

---

# 265. No employee performance usage.

---

# 266. Hard rule.

---

# 267. SLO Review Cadence

Periodic.

---

# 268. Review:

```text
target relevance
measurement quality
dependency changes
service criticality
```

---

# 269. No arbitrary lowering after misses.

---

# 270. Hard rule.

---

# 271. SLO Change Governance

Changing target is a policy change.

---

# 272. Requires review.

---

# 273. Cannot retroactively improve historical compliance.

---

# 274. Hard rule.

---

# 275. SLO Versioning

```rust
pub struct SloVersion(pub u64);
```

---

# 276. Historical evaluations reference exact version.

---

# 277. Hard rule.

---

# 278. Target Tightening

Allowed after evidence.

---

# 279. Target Loosening

Requires rationale.

---

# 280. Hard rule.

---

# 281. Reliability Objective Registry

Signed/versioned.

---

# 282. No runtime mutable target.

---

# 283. Hard rule.

---

# 284. SLI Definition Registry

```rust
pub struct SliDefinition {
    pub id: SliId,
    pub expression: SliExpression,
    pub source: SliSourceRef,
    pub denominator: DenominatorPolicy,
}
```

---

# 285. SLI Expression

Typed DSL.

---

# 286. No arbitrary SQL/script.

---

# 287. Hard rule.

---

# 288. Reliability DSL

Bounded/non-Turing.

---

# 289. Example:

```text
good = status in SUCCESS
total = admitted_requests
```

---

# 290. Hard rule.

---

# 291. SLI Source Trust

Telemetry source must be authenticated.

---

# 292. No client-submitted arbitrary SLO status.

---

# 293. Hard rule.

---

# 294. Client Reliability Telemetry

Only aggregate/minimized.

---

# 295. Example:

```text
startup success
sync success
crash-free sessions
```

---

# 296. No stable user/device identity.

---

# 297. Hard rule.

---

# 298. Local-Only Reliability

Some client metrics can stay local.

---

# 299. User diagnostics export on demand.

---

# 300. Good.

---

# 301. Availability Governance

Defines minimum operating modes.

---

# 302. Availability Policy

```rust
pub struct AvailabilityPolicy {
    pub service: ServiceId,
    pub normal_mode: ContinuityMode,
    pub degraded_modes: Vec<ContinuityMode>,
    pub forbidden_downgrades: BTreeSet<ForbiddenAvailabilityDowngrade>,
}
```

---

# 303. Forbidden Availability Downgrade

```rust
pub enum ForbiddenAvailabilityDowngrade {
    DisableEncryption,
    DisableAuthorization,
    DisableAnonymity,
    CrossResidencyBoundary,
}
```

---

# 304. Hard rule.

---

# 305. Reliability Priority

During pressure:

```text
security
control plane
emergency
core messaging
interactive
background
bulk
```

---

# 306. Part 101.

---

# 307. Hard rule.

---

# 308. Reliability Decision Service

```rust
pub trait ReliabilityGovernanceService {
    fn evaluate_slo(
        &self,
        slo: SloId,
    ) -> Result<SloEvaluation, ReliabilityError>;

    fn error_budget(
        &self,
        slo: SloId,
    ) -> Result<ErrorBudget, ReliabilityError>;

    fn governance_actions(
        &self,
        service: ServiceId,
    ) -> Result<Vec<ReliabilityGovernanceAction>, ReliabilityError>;
}
```

---

# 309. SLO Evaluation

```rust
pub struct SloEvaluation {
    pub slo: SloId,
    pub state: SloEvaluationState,
    pub budget: ErrorBudget,
    pub burn: BurnRate,
}
```

---

# 310. Release Gate API

```rust
pub trait ReliabilityReleaseGate {
    fn evaluate_release(
        &self,
        service: ServiceId,
        risk: ChangeRiskClass,
    ) -> Result<GateResult, ReliabilityError>;
}
```

---

# 311. No Behavioral Input

Hard rule.

---

# 312. Error Taxonomy

```rust
pub enum ReliabilityError {
    SliNotFound,
    SloNotFound,
    TelemetryMissing,
    InvalidDefinition,
    BudgetUnavailable,
    PolicyViolation,
    ScopeViolation,
    Unauthorized,
    Internal,
}
```

---

# 313. Observability

Safe metrics:

```text
SLO state
burn rate
budget remaining
SLO violations
recovery time
```

---

# 314. Forbidden:

```text
user-level availability history
social graph
private content
```

---

# 315. Hard rule.

---

# 316. Reliability SLOs For Reliability System

Meta reliability.

Examples:

```text
SLI evaluation freshness
alert delivery latency
error-budget ledger availability
```

---

# 317. Avoid infinite recursion.

---

# 318. Hard rule.

---

# 319. Failure Modes

```text
telemetry outage
misconfigured SLI
dependency hidden
error-budget corruption
SLO target mismatch
```

---

# 320. Telemetry Outage

Evaluation Unknown.

---

# 321. No green default.

---

# 322. Hard rule.

---

# 323. Misconfigured SLI

Disable affected evaluation.

---

# 324. Alert governance owner.

---

# 325. No silently wrong SLO.

---

# 326. Hard rule.

---

# 327. Hidden Dependency

Inventory/drift process.

---

# 328. Revisit reliability model.

---

# 329. Hard rule.

---

# 330. Error-Budget Corruption

Recalculate from authoritative SLI aggregates.

---

# 331. Ledger derived/recoverable where possible.

---

# 332. Hard rule.

---

# 333. Target Mismatch

Version/policy issue.

---

# 334. Historical records preserve old target.

---

# 335. Hard rule.

---

# 336. Testing

Need reliability testkit.

---

# 337. Test Scenarios

```text
normal service
fast burn
slow burn
regional outage
brownout
```

---

# 338. SLI Definition Test

Good/total semantics correct.

---

# 339. Missing Telemetry Test

Unknown, not Meeting.

---

# 340. Error Budget Test

Correct remaining budget.

---

# 341. Burn Rate Test

Fast/slow windows correct.

---

# 342. Release Gate Test

Critical budget can Hold risky change.

---

# 343. Security Patch Test

Budget exhaustion does not block urgent security change.

---

# 344. Maintenance Test

Only preapproved exclusion applied.

---

# 345. Regional Test

One region's failure not hidden by global aggregate.

---

# 346. Tenant Test

Tenant A cannot query B SLO data.

---

# 347. Privacy Test

No stable user ID in SLI.

---

# 348. Mixnet Test

No source-destination correlation needed.

---

# 349. DR Test

RTO/RPO objective measured in drill.

---

# 350. Brownout Test

Optional capability SLO degrades without redefining core SLO.

---

# 351. SLO Change Test

Historical evaluation preserves old target.

---

# 352. Hard-Invariant Test

Privacy/security invariant cannot be converted into error budget.

---

# 353. Fuzzing

Fuzz:

```text
SLI definition
SLO target
error budget
burn-rate policy
maintenance policy
```

---

# 354. Property Tests

Properties:

```text
missing mandatory telemetry can never produce Meeting
hard privacy/security invariant can never be represented as consumable error budget
error-budget exhaustion can never silently disable urgent security remediation
tenant-scoped reliability query can never return another tenant's data
```

---

# 355. Formal Verification Targets

Strong candidates:

```text
error-budget state machine
burn-rate alert semantics
release gate policy
maintenance accounting
```

---

# 356. Kani Candidate

target/budget arithmetic and policy invariants.

---

# 357. TLA+ Candidate

healthy → burn → warning → critical → exhausted → recovery.

---

# 358. Loom Candidate

concurrent SLI ingestion + budget evaluation + release gate decision.

---

# 359. Performance

SLI processing must be efficient.

---

# 360. Use counters/histograms.

---

# 361. No raw event retention required.

---

# 362. Hard rule.

---

# 363. Aggregation

Preaggregate by:

```text
service
region
work class
tenant aggregate
```

---

# 364. No user dimension.

---

# 365. Hard rule.

---

# 366. Latency Histograms

Bounded buckets.

---

# 367. Avoid high-cardinality labels.

---

# 368. Hard rule.

---

# 369. Storage

Separate:

```text
SLI definitions
SLO definitions
aggregate SLI samples
error-budget ledger
reliability policies
```

---

# 370. No per-request/user archive.

---

# 371. Hard rule.

---

# 372. Retention

Long enough for objective windows/trends.

---

# 373. Roll up old metrics.

---

# 374. Hard rule.

---

# 375. Crate Layout

Recommended:

```text
crates/
├── siar-reliability-core/
├── siar-sli/
├── siar-slo/
├── siar-error-budget/
├── siar-burn-rate/
├── siar-reliability-policy/
├── siar-dependency-slo/
├── siar-availability-governance/
├── siar-reliability-alerting/
├── siar-reliability-observability/
└── siar-reliability-testkit/
```

---

# 376. `siar-reliability-core`

Owns:

```text
SliId
SloId
ReliabilityClass
errors
```

---

# 377. `siar-sli`

Typed indicator definitions/aggregation.

---

# 378. `siar-slo`

Objective registry/evaluation.

---

# 379. `siar-error-budget`

Budget arithmetic/ledger.

---

# 380. `siar-burn-rate`

Multi-window burn computation.

---

# 381. `siar-reliability-policy`

Governance actions/change gates.

---

# 382. `siar-dependency-slo`

Dependency/critical-path reliability.

---

# 383. `siar-availability-governance`

Continuity/degradation rules.

---

# 384. `siar-reliability-alerting`

Burn-rate alerts/escalation.

---

# 385. `siar-reliability-observability`

Aggregate reliability-system health.

---

# 386. `siar-reliability-testkit`

SLO/budget/privacy/DR tests.

---

# 387. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. Reliability engineering is based on typed service, capability, region, tenant-aggregate, work-class, and dependency signals—not stable user identity, private content, social graphs, or engagement behavior.
2. SLI, SLO, SLA, error budget, hard invariant, incident severity, and maintenance policy are distinct concepts and cannot be collapsed into a single generic "health" score.
3. Missing or partial mandatory telemetry yields Unknown SLO state, never Meeting/Healthy by default.
4. Security/privacy hard invariants such as tenant isolation, cryptographic integrity, authorization, and anonymity floors can never be converted into consumable error budgets.
5. Error-budget state can restrict risky product/change velocity but can never block urgent security remediation, incident containment, or disaster recovery; such changes instead require safer rollout controls.
6. Regional and tenant-specific reliability remains visible and cannot be hidden by a global aggregate that appears healthy.
7. Reliability governance uses dependency criticality, redundancy, correlated failure domains, and recovery objectives rather than assuming every connected dependency fails independently.
8. Maintenance exclusions are defined prospectively, versioned, and policy-controlled; outages cannot be retroactively removed to improve SLO compliance.
9. Reliability telemetry is aggregate, bounded-cardinality, privacy-minimized, and never repurposed into user-level availability history or workforce-performance surveillance.
10. Error-budget policy changes, SLO target changes, and service criticality changes are versioned and preserve historical interpretation; targets cannot be silently loosened after failures.
11. Availability degradation, brownout, load shedding, cost pressure, or region failure can never justify disabling encryption, authorization, anonymity, residency, or other hard security/privacy floors.
12. Reliability engineering integrates with capacity, DR, changes, release governance, inventory, incident response, SOC, compliance, FinOps, residency, and organizational governance without creating an alternate path around security/privacy policy.
```

---

# 388. Initial Production Scope

Implement first:

```text
typed SliId/SloId
SLI kinds and denominator policies
fixed-point SLO targets
service reliability classes
SLO registry/versioning
aggregate SLI counters/histograms
error-budget arithmetic
Healthy/Warning/Critical/Exhausted states
multi-window burn rate
burn-rate alerts
change/release budget gates
dependency SLOs
regional SLOs
RTO/RPO reliability objectives
maintenance accounting
hard-invariant separation
privacy-safe reliability dashboards
reliability testkit
```

Then add:

```text
advanced dependency reliability modeling
correlated-failure simulation
federation reliability attestations
automatic SLO recommendation from infrastructure evidence
formal error-budget/release-gate verification
tenant contractual SLA adapters
```

---

# 389. Definition of Done

Part 109 is complete when:

- SLIs/SLOs/error budgets are typed and distinct
- SLO definitions are versioned
- missing telemetry yields Unknown
- hard security/privacy guarantees are not budgeted away
- burn-rate policies exist
- error-budget state can govern risky changes
- urgent security/DR changes remain possible
- regional/tenant reliability cannot be hidden
- maintenance exclusions are prospective
- dependency SLOs understand redundancy/correlation
- brownout/load shedding preserve critical objectives
- telemetry contains no user behavioral dimensions
- SLO/budget/privacy/fuzz/formal tests are specified

---

# 390. Final Architecture

```text
                  SERVICE SIGNALS
                        │
                        ▼
                  SLI AGGREGATION
                        │
                        ▼
                  SLO EVALUATION
                        │
                        ▼
                 ERROR BUDGET
                        │
             ┌──────────┼──────────┐
             │          │          │
          HEALTHY     WARNING    EXHAUSTED
             │          │          │
             └──────────┼──────────┘
                        ▼
              RELIABILITY GOVERNANCE
                        │
          ┌─────────────┼─────────────┐
          │             │             │
       ALERT        CHANGE GATE     RECOVERY
```

Reliability safety model:

```text
typed SLIs
+
explicit SLOs
+
bounded error budgets
+
multi-window burn rates
+
dependency-aware objectives
+
prospective maintenance policy
+
privacy-safe aggregate telemetry
+
hard security/privacy floors
```

not:

```text
compute one green score, hide regional failures, track users to measure availability, and loosen targets whenever reliability gets difficult
```

---

# 391. Final Principle

Reliability governance should tell the system when it is consuming too much failure budget without ever treating privacy or security guarantees as expendable reliability currency.

The correct model is:

```text
measure technical outcomes
+
set explicit objectives
+
track error budgets
+
alert on burn
+
govern risky changes
+
preserve critical service classes
+
measure recovery honestly
+
never trade away security or privacy for uptime
```

This architecture gives SIAR a privacy-preserving reliability-engineering foundation for SLI/SLO governance, error budgets, burn-rate alerting, change/release gates, dependency reliability, regional objectives, maintenance accounting, and availability policy while preserving the anonymity, local-first, least-authority, and anti-surveillance guarantees established across Parts 34–108.
