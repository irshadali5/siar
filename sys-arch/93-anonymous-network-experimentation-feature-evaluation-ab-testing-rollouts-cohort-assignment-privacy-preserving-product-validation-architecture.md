# Core System Architecture Part 93 — Anonymous Network Experimentation, Feature Evaluation, A/B Testing, Rollouts, Cohort Assignment & Privacy-Preserving Product Validation Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 93  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 36, 42, 51, 52, 56, 61, 69–70, 74–75, 81, 92  

**Primary purpose:** define SIAR's experimentation, feature evaluation, A/B testing, staged rollout, cohort assignment, holdout, feature-flag, product validation, privacy-preserving outcome measurement, rollback, and experiment-governance architecture without creating durable cross-feature identifiers, hidden behavioral profiles, or surveillance-oriented experimentation systems.

---

# 1. Purpose

Experimentation can help answer questions such as:

```text
Does this onboarding flow reduce failure?
Does this retry policy improve sync completion?
Does this UI layout reduce confusion?
Does this protocol change reduce latency?
```

But conventional experimentation often depends on:

```text
durable user IDs
cross-feature assignment IDs
event-level analytics
behavioral tracking
long-lived cohort membership
```

The governing principle is:

> **SIAR experiments should evaluate narrowly defined product hypotheses using local or scoped cohort assignment, minimum necessary measurements, privacy-preserving aggregation, and bounded lifetimes—without creating durable experimentation identities or user-level behavior histories.**

---

# 2. Architectural Position

```text
Experiment Definition
       │
       ▼
Eligibility Evaluation
       │
       ▼
Scoped Cohort Assignment
       │
       ├── Control
       ├── Variant A
       └── Variant B
       │
       ▼
Feature Evaluation
       │
       ▼
Aggregate Outcome Collection
       │
       ▼
Product Decision
```

---

# 3. Core Separation

Keep distinct:

```text
feature flag
experiment
rollout
holdout
cohort assignment
product metric
security kill switch
```

---

# 4. Non-Goals

Part 93 does not create:

```text
a global experimentation ID
persistent user-level experiment history
cross-product behavioral profiling
experiments that override privacy/security guarantees
experiments on sensitive identity traits
```

---

# 5. Experiment Types

```rust
pub enum ExperimentType {
    ProductBehavior,
    UiVariation,
    Performance,
    ProtocolCompatibility,
    OperationalPolicy,
}
```

---

# 6. Product Behavior

Examples:

```text
onboarding flow
settings layout
notification grouping
```

---

# 7. UI Variation

Presentation-level variation.

---

# 8. Performance

Examples:

```text
batch size
cache policy
prefetch behavior
```

---

# 9. Protocol Compatibility

Version rollout/mixed-version validation.

---

# 10. Operational Policy

Low-risk runtime tuning.

---

# 11. Forbidden Experiment Classes

Examples:

```text
weaken encryption
weaken anonymity
disable privacy floor
bypass authorization
track sensitive identity traits
```

---

# 12. Hard Rule

Security/privacy invariants are not experimental variables.

---

# 13. Experiment ID

```rust
pub struct ExperimentId(pub [u8; 16]);
```

---

# 14. Experiment ID Is Not User Identity

Hard rule.

---

# 15. Experiment Definition

```rust
pub struct ExperimentDefinition {
    pub id: ExperimentId,
    pub hypothesis: HypothesisId,
    pub scope: ExperimentScope,
    pub variants: Vec<ExperimentVariant>,
    pub assignment: AssignmentPolicy,
    pub measurement: MeasurementPlan,
    pub lifetime: ExperimentLifetime,
}
```

---

# 16. Hypothesis

Explicit.

---

# 17. No "Collect Data And See"

Hard rule.

---

# 18. Every Experiment Must Answer A Defined Question

---

# 19. Experiment Scope

```rust
pub enum ExperimentScope {
    DeviceLocal,
    AccountLocal,
    TenantManaged,
    ServiceOperational,
}
```

---

# 20. DeviceLocal

Preferred for UI/product testing.

---

# 21. AccountLocal

Only when multi-device consistency needed.

---

# 22. TenantManaged

Managed profile only.

---

# 23. ServiceOperational

Infrastructure/backend behavior.

---

# 24. Hard Rule

Experiments are never global identity scopes by default.

---

# 25. Eligibility

Experiments must declare who is eligible.

---

# 26. Eligibility Policy

```rust
pub struct EligibilityPolicy {
    pub platform: Option<PlatformClass>,
    pub app_version: Option<VersionRange>,
    pub tenant_scope: Option<TenantId>,
    pub required_features: BTreeSet<FeatureCapability>,
}
```

---

# 27. No Sensitive Traits

Hard rule.

---

# 28. Forbidden Eligibility Inputs

```text
race
religion
sexual orientation
political belief
health condition
message content
contact graph
```

---

# 29. Hard Rule

Sensitive personal attributes are not experiment segmentation variables.

---

# 30. Cohort Assignment

Preferred local assignment.

---

# 31. Assignment Modes

```rust
pub enum AssignmentPolicy {
    LocalRandom,
    DeterministicLocal,
    ManagedExplicit,
    ServiceShard,
}
```

---

# 32. LocalRandom

CSPRNG locally.

---

# 33. DeterministicLocal

Stable within experiment using local secret.

---

# 34. ManagedExplicit

Org-controlled managed profile.

---

# 35. ServiceShard

Infrastructure service instance/shard rollout.

---

# 36. No Global User Hash Assignment

Hard rule.

---

# 37. Why

A global user hash becomes cross-experiment identity.

---

# 38. Local Assignment Secret

Device/account-local.

---

# 39. Not uploaded.

---

# 40. Deterministic Local Assignment

```text
HMAC(local_experiment_secret, experiment_id)
```

---

# 41. Produces Stable Assignment Per Experiment

---

# 42. Cannot Correlate Across Experiments Without Local Secret

---

# 43. Hard Rule

Experiment assignments are unlinkable across experiments remotely.

---

# 44. Cohort Assignment Record

```rust
pub struct ExperimentAssignment {
    pub experiment: ExperimentId,
    pub variant: VariantId,
    pub assigned_at: CoarseEpoch,
    pub expires_at: Timestamp,
}
```

---

# 45. Assignment Lifetime

Bounded.

---

# 46. No Permanent Assignment

Hard rule.

---

# 47. Variant ID

```rust
pub struct VariantId(pub u16);
```

---

# 48. Variant Count

Small/bounded.

---

# 49. No High-Cardinality Variants

Hard rule.

---

# 50. Holdout

A control population intentionally not exposed.

---

# 51. Holdout Scope

Experiment-specific.

---

# 52. No Universal Permanent Holdout ID

Hard rule.

---

# 53. Holdout Policy

```rust
pub struct HoldoutPolicy {
    pub percentage_basis_points: u16,
}
```

---

# 54. Basis Points

Fixed-point.

---

# 55. Feature Flags

Different from experiments.

---

# 56. Feature Flag

Turns behavior on/off.

---

# 57. Experiment

Measures impact.

---

# 58. Hard Rule

A feature flag is not automatically an experiment.

---

# 59. Feature Flag Definition

```rust
pub struct FeatureFlag {
    pub feature: FeatureId,
    pub state: FeatureFlagState,
    pub scope: FeatureFlagScope,
}
```

---

# 60. Feature Flag State

```rust
pub enum FeatureFlagState {
    Disabled,
    Enabled,
    Gradual,
    EmergencyDisabled,
}
```

---

# 61. EmergencyDisabled

Security kill switch.

---

# 62. Hard Rule

Emergency disable can remove functionality but cannot weaken security/privacy.

---

# 63. Gradual Rollout

Percentage-based.

---

# 64. Rollout Policy

```rust
pub struct GradualRollout {
    pub percentage_basis_points: u16,
    pub eligibility: EligibilityPolicy,
}
```

---

# 65. Local Assignment Preferred

---

# 66. No Per-User Server Targeting

Hard rule.

---

# 67. Rollout Phases

```rust
pub enum RolloutPhase {
    Internal,
    Canary,
    Small,
    Medium,
    Broad,
    GeneralAvailability,
}
```

---

# 68. Internal

Developers/test environment.

---

# 69. Canary

Small population.

---

# 70. Small/Medium/Broad

Increasing exposure.

---

# 71. GeneralAvailability

Experiment ends.

---

# 72. Rollout Gate

Advance only if:

```text
reliability good
privacy invariants good
security good
product metric acceptable
```

---

# 73. Hard Rule

Positive engagement metric cannot override security/privacy regression.

---

# 74. Exposure Logging

Dangerous.

---

# 75. Preferred

Local exposure counter.

---

# 76. No raw per-user exposure stream.

---

# 77. Hard rule.

---

# 78. Exposure Confirmation

Sometimes necessary to know whether variant actually rendered.

---

# 79. Local Aggregate

Count:

```text
variant_shown += 1
```

---

# 80. Upload only aggregate.

---

# 81. No per-user exposure ID.

---

# 82. Hard rule.

---

# 83. Measurement Plan

Explicit.

---

# 84. Measurement Plan

```rust
pub struct MeasurementPlan {
    pub primary_metric: MetricId,
    pub secondary_metrics: Vec<MetricId>,
    pub guardrail_metrics: Vec<MetricId>,
    pub minimum_cohort: u32,
}
```

---

# 85. Primary Metric

One main hypothesis result.

---

# 86. Secondary Metrics

Supporting.

---

# 87. Guardrails

Examples:

```text
crash rate
privacy error
security failure
latency regression
```

---

# 88. Hard Rule

Experiment cannot declare only success metrics and omit guardrails.

---

# 89. Metric Registry

Uses Part 92 approved metrics.

---

# 90. No Experiment-Specific Raw Event Bypass

Hard rule.

---

# 91. Product Outcome

Aggregate.

---

# 92. Example

```text
onboarding_completed / onboarding_started
```

---

# 93. No per-user journey upload.

---

# 94. Hard rule.

---

# 95. Conversion Metrics

Local aggregate per variant.

---

# 96. Upload:

```text
variant
started_count
completed_count
```

---

# 97. No account IDs.

---

# 98. Hard rule.

---

# 99. Minimum Cohort

Mandatory for product experiments.

---

# 100. Small sample

Do not interpret/publish.

---

# 101. Hard rule.

---

# 102. Differential Privacy

Optional.

---

# 103. Can protect experiment aggregate releases.

---

# 104. Uses Part 92 privacy budget.

---

# 105. No experiment-local homemade DP.

---

# 106. Hard rule.

---

# 107. Experiment Privacy Budget

```rust
pub struct ExperimentPrivacyBudget {
    pub experiment: ExperimentId,
    pub budget: PrivacyBudget,
}
```

---

# 108. Budget bounded.

---

# 109. Multiple queries consume budget.

---

# 110. No unlimited slicing.

---

# 111. Hard rule.

---

# 112. Variant Assignment Privacy

Server should not need assignment for every UI experiment.

---

# 113. DeviceLocal experiment:

assignment local.

---

# 114. Server only sees aggregate metric by variant.

---

# 115. No user mapping.

---

# 116. Hard rule.

---

# 117. AccountLocal Experiment

May need stable multi-device assignment.

---

# 118. Store encrypted account sync state.

---

# 119. Server need not inspect.

---

# 120. Hard rule.

---

# 121. Managed Experiment

Org may test managed workflow.

---

# 122. Assignment scoped to managed profile.

---

# 123. Personal profile unaffected.

---

# 124. Hard rule.

---

# 125. Tenant Isolation

Tenant A experiments do not affect B.

---

# 126. No shared assignment ID.

---

# 127. Hard rule.

---

# 128. Infrastructure Experiment

ServiceShard assignment.

---

# 129. Example:

```text
new retry policy on 5% of relay nodes
```

---

# 130. Use service instance/shard identity.

---

# 131. No user cohort needed.

---

# 132. Hard rule.

---

# 133. Protocol Experiment

High risk.

---

# 134. Only for compatibility/performance where security invariant unchanged.

---

# 135. Version negotiation explicit.

---

# 136. No experimental crypto in production unless approved Experimental suite under Part 66.

---

# 137. Hard rule.

---

# 138. Privacy Experiment

Cannot weaken privacy floor.

---

# 139. Could compare:

```text
different privacy explanation UI
different default presentation
```

---

# 140. Cannot compare:

```text
privacy on vs off
```

if off violates network floor.

---

# 141. Hard rule.

---

# 142. Security Experiment

No A/B testing weaker security.

---

# 143. Hard rule.

---

# 144. Experiment Governance

Every experiment requires:

```text
owner
hypothesis
scope
lifetime
metrics
guardrails
rollback
privacy review
```

---

# 145. Experiment Lifecycle

```rust
pub enum ExperimentState {
    Draft,
    Approved,
    Running,
    Paused,
    Completed,
    RolledBack,
    Archived,
}
```

---

# 146. Draft

Not active.

---

# 147. Approved

Ready.

---

# 148. Running

Assignments active.

---

# 149. Paused

No new assignment/exposure.

---

# 150. Completed

No new measurement.

---

# 151. RolledBack

Variant disabled.

---

# 152. Archived

Metadata retained.

---

# 153. No "Running Forever"

Hard rule.

---

# 154. Experiment Lifetime

```rust
pub struct ExperimentLifetime {
    pub starts_at: Timestamp,
    pub ends_at: Timestamp,
}
```

---

# 155. End Date Required

Hard rule.

---

# 156. Extension

Requires review.

---

# 157. No Silent Infinite Extension

Hard rule.

---

# 158. Experiment Enrollment

Local/managed.

---

# 159. No server creates durable global experiment membership table.

---

# 160. Hard rule.

---

# 161. Experiment Enrollment State

```rust
pub enum EnrollmentState {
    Eligible,
    Assigned,
    Exposed,
    Completed,
    Expired,
}
```

---

# 162. Stored locally where possible.

---

# 163. Not synchronized unless scope needs.

---

# 164. Hard rule.

---

# 165. Experiment Assignment Determinism

Must remain stable during experiment.

---

# 166. Reinstall/device migration

Can get new assignment for DeviceLocal.

---

# 167. Acceptable.

---

# 168. AccountLocal

Can preserve encrypted assignment.

---

# 169. No server-side identity join.

---

# 170. Hard rule.

---

# 171. Cross-Experiment Correlation

Forbidden.

---

# 172. Assignment secrets domain-separated by experiment.

---

# 173. No common assignment key.

---

# 174. Hard rule.

---

# 175. Experiment Bucketing

```rust
pub struct Bucket(pub u16);
```

---

# 176. 0..9999 basis points.

---

# 177. Assignment algorithm versioned.

---

# 178. No silent bucket algorithm change mid-experiment.

---

# 179. Hard rule.

---

# 180. Assignment Algorithm

```rust
pub struct AssignmentAlgorithmVersion(pub u16);
```

---

# 181. If changed

new experiment or explicit migration.

---

# 182. No variant churn.

---

# 183. Hard rule.

---

# 184. Sticky Assignment

Only experiment lifetime.

---

# 185. Not permanent.

---

# 186. Experiment Storage

Client:

```text
definitions
assignments
local counters
```

---

# 187. Server:

```text
signed experiment catalog
aggregate outcomes
approval metadata
```

---

# 188. No raw behavior stream.

---

# 189. Hard rule.

---

# 190. Experiment Catalog

Signed.

---

# 191. Catalog contains:

```text
experiment ID
scope
variants
eligibility
lifetime
measurement plan
```

---

# 192. No user targeting list.

---

# 193. Hard rule.

---

# 194. Catalog Distribution

Part 61 signed config flow.

---

# 195. Push hint then pull signed bundle.

---

# 196. Anti-rollback.

---

# 197. No unsigned experiment activation.

---

# 198. Hard rule.

---

# 199. Catalog Validation

Client checks:

```text
signature
time window
supported version
scope
privacy policy
```

---

# 200. Invalid catalog

ignore/last valid.

---

# 201. No permissive fallback.

---

# 202. Hard rule.

---

# 203. Feature Evaluation API

```rust
pub trait FeatureEvaluator {
    fn evaluate(
        &self,
        feature: FeatureId,
        context: &FeatureContext,
    ) -> FeatureDecision;
}
```

---

# 204. Feature Decision

```rust
pub enum FeatureDecision {
    Disabled,
    Enabled,
    Variant(VariantId),
}
```

---

# 205. No Caller-Supplied Variant

Hard rule.

---

# 206. Experiment Evaluator

```rust
pub trait ExperimentEvaluator {
    fn assignment(
        &self,
        experiment: ExperimentId,
        context: &ExperimentContext,
    ) -> Result<Option<ExperimentAssignment>, ExperimentError>;
}
```

---

# 207. Evaluation Local

Preferred.

---

# 208. No network request per feature check.

---

# 209. Hard rule.

---

# 210. Variant Exposure

Separate from assignment.

---

# 211. Why

Assignment does not guarantee feature rendered.

---

# 212. Exposure Recording

Local one-shot per relevant window.

---

# 213. No repeated raw exposure events.

---

# 214. Hard rule.

---

# 215. Exposure Window

Coarse.

---

# 216. Example

Per experiment/day.

---

# 217. No exact timestamps.

---

# 218. Hard rule.

---

# 219. Outcome Measurement

Bounded contribution.

---

# 220. Example

One onboarding completion per experiment/device.

---

# 221. No repeated outcome inflation.

---

# 222. Hard rule.

---

# 223. Contribution Ledger

Local.

---

# 224. Prevent duplicate contribution.

---

# 225. No user identifier.

---

# 226. Hard rule.

---

# 227. Experiment Metrics Envelope

```rust
pub struct ExperimentAggregate {
    pub experiment: ExperimentId,
    pub variant: VariantId,
    pub coarse_epoch: CoarseEpoch,
    pub metrics: Vec<MetricAggregate>,
}
```

---

# 228. No account ID.

---

# 229. No raw action sequence.

---

# 230. Hard rule.

---

# 231. Upload Scheduling

Low priority.

---

# 232. Batched.

---

# 233. No wake for experiment upload.

---

# 234. Hard rule.

---

# 235. Max-Anonymity Mode

Product experiments disabled by default.

---

# 236. Operational service rollouts may continue if user data unaffected.

---

# 237. Hard Rule

Privacy mode cannot be bypassed to enroll user in product experiment.

---

# 238. User Consent

Experiment participation can follow analytics consent.

---

# 239. If analytics disabled

product experiment measurement off.

---

# 240. Feature rollout may still occur without measurement if product feature is normal release rollout.

---

# 241. Important distinction.

---

# 242. Hard rule.

---

# 243. Experimentation vs Rollout

Experimentation asks:

```text
Which variation is better?
```

Rollout asks:

```text
Can we safely deploy this?
```

---

# 244. Rollout Does Not Require Behavioral Analytics

Hard rule.

---

# 245. Release Rollout

Part 52/70.

---

# 246. Feature flag can gradually enable feature.

---

# 247. Monitor operational metrics.

---

# 248. No user-level experiment needed.

---

# 249. Hard rule.

---

# 250. Canary Rollout

Prefer fault-domain/service/device class, not user identity.

---

# 251. App rollout may use local random bucket.

---

# 252. No server persistent user targeting list.

---

# 253. Hard rule.

---

# 254. Kill Switch

Signed high-priority policy.

---

# 255. Can disable experiment/feature immediately.

---

# 256. Cannot enable forbidden weak mode.

---

# 257. Hard rule.

---

# 258. Rollback

Variant state restored to stable behavior.

---

# 259. Migration safety needed.

---

# 260. If experiment changes data format

must support backward/forward compatibility.

---

# 261. Hard rule.

---

# 262. No Irreversible Schema Experiment Without Migration Plan

Hard rule.

---

# 263. Experiment Data Migration

Rare.

---

# 264. Prefer experiment only presentation/behavior first.

---

# 265. Data-model experiments need dedicated schema version.

---

# 266. No same key interpreted differently by cohorts without version tag.

---

# 267. Hard rule.

---

# 268. Experiment Flag Precedence

Recommended:

```text
hard security/privacy invariant
> emergency disable
> signed release rollout
> tenant managed restriction
> experiment assignment
> local user setting
```

---

# 269. User Setting

Can often disable optional experiment feature.

---

# 270. Experiment cannot override user privacy/security setting.

---

# 271. Hard rule.

---

# 272. Managed Policy

Can restrict managed profile.

---

# 273. Cannot expand personal experiment visibility.

---

# 274. Hard rule.

---

# 275. Account Lifecycle

Deleted/deactivated account loses account-scoped experiment state.

---

# 276. Local device assignment may remain meaningless/local until cleanup.

---

# 277. No experiment history carried into new AccountId.

---

# 278. Hard rule.

---

# 279. New Account

New assignments.

---

# 280. No identity continuity through experiment buckets.

---

# 281. Hard rule.

---

# 282. Device Replacement

DeviceLocal assignments reset.

---

# 283. No attempt to fingerprint and restore assignment.

---

# 284. Hard rule.

---

# 285. Experiment Telemetry Separation

Product experiments use Part 92 aggregate store.

---

# 286. No separate hidden experiment event pipeline.

---

# 287. Hard rule.

---

# 288. Experiment Analyst Access

Aggregate only.

---

# 289. No per-user variant table.

---

# 290. No session replay.

---

# 291. Hard rule.

---

# 292. Statistical Analysis

Need predeclared method.

---

# 293. Avoid p-hacking.

---

# 294. Experiment Plan Can Include:

```text
sample size target
primary metric
minimum effect
stopping rule
```

---

# 295. No repeated arbitrary peeking without correction.

---

# 296. Hard rule.

---

# 297. Sequential Testing

Possible if predeclared.

---

# 298. Bayesian/Frequentist

Either acceptable.

---

# 299. Architecture does not mandate one.

---

# 300. But

method must be declared.

---

# 301. No "whichever gives significance."

---

# 302. Hard rule.

---

# 303. Experiment Result

```rust
pub struct ExperimentResult {
    pub experiment: ExperimentId,
    pub conclusion: ExperimentConclusion,
    pub confidence: InsightConfidence,
}
```

---

# 304. Experiment Conclusion

```rust
pub enum ExperimentConclusion {
    Inconclusive,
    ControlPreferred,
    VariantPreferred(VariantId),
    RolledBackForGuardrail,
}
```

---

# 305. No Automatic Product Rollout Based Solely On Metric Winner

Hard rule.

---

# 306. Human/Policy Review

Required for significant product changes.

---

# 307. Guardrail Failure

Immediate pause/rollback.

---

# 308. Examples:

```text
crash spike
privacy regression
security failure
latency explosion
```

---

# 309. Hard rule.

---

# 310. Automated Rollout Gate

```rust
pub struct RolloutGate {
    pub max_error_rate: FixedPoint,
    pub max_latency_regression: FixedPoint,
    pub forbidden_privacy_events: u64,
    pub forbidden_security_events: u64,
}
```

---

# 311. Privacy/Security Forbidden Events

Zero tolerance where invariant.

---

# 312. Hard rule.

---

# 313. Operational Experiments

Can compare service policies.

---

# 314. Example:

```text
queue batch size
retry jitter
cache TTL
```

---

# 315. Assignment By Service Instance

---

# 316. No user cohort.

---

# 317. Metrics service-level.

---

# 318. Hard rule.

---

# 319. Network Experiment

Example route-selection heuristic.

---

# 320. Must not reduce privacy floor.

---

# 321. Compare among privacy-equivalent options.

---

# 322. Hard rule.

---

# 323. Anonymity Experiments

Highly restricted.

---

# 324. Do not test weaker anonymity against stronger anonymity on real users.

---

# 325. Use simulation/lab.

---

# 326. Hard rule.

---

# 327. Cryptographic Experiments

Lab/test only unless reviewed algorithm suite marked experimental and users knowingly opt into test environment.

---

# 328. No production random crypto experimentation.

---

# 329. Hard rule.

---

# 330. UI Experiments

Safe candidate.

---

# 331. Still avoid manipulative dark-pattern testing.

---

# 332. Forbidden UI Experiments:

```text
making privacy harder to find
increasing compulsive notifications
hiding chronological feed
obscuring unsubscribe
```

---

# 333. Hard rule.

---

# 334. Ethical Product Validation

Optimize:

```text
task success
clarity
reliability
latency
error reduction
```

---

# 335. Do Not Optimize Primarily:

```text
time-on-app
return frequency
notification opens
compulsive engagement
```

---

# 336. Hard rule.

---

# 337. Experiment User Experience

No need to expose every harmless UI experiment.

---

# 338. But analytics/privacy policy should disclose experimentation category.

---

# 339. Managed tenant may require explicit admin visibility.

---

# 340. No secret sensitive-trait experiment.

---

# 341. Hard rule.

---

# 342. Opt-Out

User can disable product experiments when analytics disabled/privacy mode.

---

# 343. Stable release behavior remains available.

---

# 344. Hard rule.

---

# 345. Experiment Reset

When experiment ends

local assignment deleted/expired.

---

# 346. No permanent history.

---

# 347. Hard rule.

---

# 348. Experiment Archive

Server retains:

```text
definition
aggregate result
approval/decision
```

---

# 349. Does not retain:

```text
user assignment map
raw exposure events
behavioral event history
```

---

# 350. Hard rule.

---

# 351. Experiment Retention

Aggregate result can retain long-term.

---

# 352. Fine-grained rollout metrics expire.

---

# 353. Assignment local state deleted after expiry.

---

# 354. Hard rule.

---

# 355. Backup

Local experiment assignment need not be backed up for DeviceLocal scope.

---

# 356. AccountLocal assignment may be encrypted sync during active experiment.

---

# 357. Expired experiments excluded.

---

# 358. No old assignment resurrection.

---

# 359. Hard rule.

---

# 360. Restore

Active experiment can recompute assignment if scope permits.

---

# 361. No user fingerprint recovery.

---

# 362. Hard rule.

---

# 363. Experiment Catalog State

```rust
pub struct ExperimentCatalog {
    pub version: ExperimentCatalogVersion,
    pub experiments: Vec<ExperimentDefinition>,
    pub signatures: PolicySignatureBundle,
}
```

---

# 364. Catalog Size Bounded.

---

# 365. No unbounded flag list.

---

# 366. Hard rule.

---

# 367. Feature Flag Registry

Static/typed.

---

# 368. Example:

```rust
pub enum FeatureId {
    NewConversationLayout,
    SyncBatchingV2,
    NotificationAggregationV2,
}
```

---

# 369. No arbitrary dynamic string flag in domain core.

---

# 370. Hard rule.

---

# 371. Why

Typed feature IDs prevent remote behavior injection.

---

# 372. Variant Payloads

Prefer enum/config small.

---

# 373. No remote arbitrary code.

---

# 374. Hard rule.

---

# 375. Variant Parameters

```rust
pub enum VariantConfig {
    Boolean(bool),
    Integer(i64),
    Enum(u16),
    FixedPoint(FixedPoint),
}
```

---

# 376. Bounded.

---

# 377. No scripting.

---

# 378. Hard rule.

---

# 379. Feature Evaluation Cache

Local.

---

# 380. Invalidated on catalog update.

---

# 381. No server call per render.

---

# 382. Hard rule.

---

# 383. Experiment Result Store

Aggregate only.

---

# 384. Partition by experiment/variant/coarse epoch.

---

# 385. Not user.

---

# 386. Hard rule.

---

# 387. Experiment Dashboard

Shows:

```text
aggregate counts
confidence
guardrails
rollout state
```

---

# 388. No drill-down to user/device.

---

# 389. Hard rule.

---

# 390. Tenant Experiment Dashboard

Tenant-scoped aggregate only.

---

# 391. No employee behavioral timeline.

---

# 392. Hard rule.

---

# 393. Federation

Experiment assignments never shared across domains.

---

# 394. Each domain controls own rollouts.

---

# 395. Cross-domain protocol compatibility tested through version negotiation, not shared user cohorts.

---

# 396. Hard rule.

---

# 397. Federation Metrics

Aggregate service compatibility only.

---

# 398. No user cross-domain assignment linkage.

---

# 399. Hard rule.

---

# 400. Analytics Integration

Part 92.

---

# 401. Experiment metrics must be registered metrics.

---

# 402. No ad hoc event collection.

---

# 403. Hard rule.

---

# 404. Authorization Integration

Part 81.

---

# 405. Experiment management requires scoped admin capability.

---

# 406. No product analyst implicit production config authority.

---

# 407. Hard rule.

---

# 408. Authentication Integration

Operator/admin step-up for high-risk experiment activation.

---

# 409. Security-sensitive rollout requires phishing-resistant auth.

---

# 410. Hard rule.

---

# 411. Secrets Integration

Experiment service uses scoped workload creds.

---

# 412. No static supertoken.

---

# 413. Hard rule.

---

# 414. Event Bus Integration

Only aggregate experiment result events.

---

# 415. No raw user exposure bus.

---

# 416. Hard rule.

---

# 417. Database Integration

Experiment catalog/config store separate from analytics result store.

---

# 418. No user assignment table baseline.

---

# 419. Hard rule.

---

# 420. Deployment Integration

Feature rollout tied to release artifact compatibility.

---

# 421. No experiment activates unsupported code path.

---

# 422. Hard rule.

---

# 423. Supply-Chain Integration

Experiment catalog signature verified.

---

# 424. No unsigned remote flag control.

---

# 425. Hard rule.

---

# 426. Observability

Safe:

```text
catalog fetch success
flag evaluation failures
variant aggregate counts
rollout guardrail health
```

---

# 427. Forbidden:

```text
user assignment lookup
per-user exposure timeline
cross-experiment identity map
```

---

# 428. Hard rule.

---

# 429. Experiment Service SLOs

Examples:

```text
catalog availability
evaluation determinism
guardrail detection latency
rollback propagation
```

---

# 430. Privacy SLOs

```text
0 global experimentation IDs
0 user-level exposure history
0 sensitive-trait segmentation
```

---

# 431. Security SLOs

```text
0 experiment override of hard privacy/security invariant
0 unsigned experiment activation
```

---

# 432. Failure Modes

```text
catalog outage
assignment bug
metric pipeline outage
guardrail alarm
rollback failure
```

---

# 433. Catalog Outage

Use last valid signed catalog within validity.

---

# 434. No new unknown experiment.

---

# 435. Hard rule.

---

# 436. Assignment Bug

Pause experiment.

---

# 437. Clear/recompute local assignment if safe.

---

# 438. Do not create central user migration map.

---

# 439. Hard rule.

---

# 440. Metric Pipeline Outage

Experiment can continue only if policy allows.

---

# 441. Otherwise pause.

---

# 442. No raw fallback event logging.

---

# 443. Hard rule.

---

# 444. Guardrail Alarm

Automatic pause/rollback.

---

# 445. Security/privacy guardrails highest priority.

---

# 446. Hard rule.

---

# 447. Rollback Failure

Emergency disable.

---

# 448. Incident response.

---

# 449. No silent continuation.

---

# 450. Hard rule.

---

# 451. Testing

Need experimentation/rollout testkit.

---

# 452. Test Scenarios

```text
local assignment
catalog rollback
privacy mode
guardrail failure
variant expiry
```

---

# 453. Determinism Test

Same local secret + experiment => stable variant.

---

# 454. Unlinkability Test

Different experiments produce unlinkable assignments.

---

# 455. Expiry Test

Expired experiment no longer evaluates.

---

# 456. Sensitive Eligibility Test

Forbidden sensitive trait cannot be registered.

---

# 457. Privacy Mode Test

Product experimentation disabled.

---

# 458. Analytics Consent Test

No product outcome upload when disabled.

---

# 459. Guardrail Test

Privacy/security failure pauses rollout.

---

# 460. Rollback Test

Stable feature state restored.

---

# 461. Catalog Signature Test

Unsigned/invalid catalog rejected.

---

# 462. Anti-Rollback Test

Older permissive catalog rejected.

---

# 463. Tenant Test

Tenant experiment does not affect personal profile/other tenants.

---

# 464. Federation Test

No cross-domain assignment exchange.

---

# 465. Backup Test

Expired assignment not restored.

---

# 466. Variant Migration Test

Assignment algorithm version change handled explicitly.

---

# 467. Telemetry Test

No per-user exposure history emitted.

---

# 468. Remote Config Test

Cannot introduce arbitrary new feature ID.

---

# 469. Fuzzing

Fuzz:

```text
experiment catalog
variant config
eligibility policy
rollout state
aggregate result envelope
```

---

# 470. Property Tests

Properties:

```text
experiment assignment cannot override privacy/security invariant
assignment identity is experiment-scoped
expired experiment cannot remain active
variant measurement cannot emit unregistered metrics
```

---

# 471. Formal Verification Targets

Strong candidates:

```text
rollout phase state machine
guardrail rollback
assignment scoping
experiment expiry
```

---

# 472. Kani Candidate

flag precedence and forbidden override invariants.

---

# 473. TLA+ Candidate

catalog update → assignment → exposure → guardrail failure → rollback.

---

# 474. Loom Candidate

concurrent catalog refresh + feature evaluation + rollback.

---

# 475. Performance

Feature evaluation is hot path.

---

# 476. Must be local/in-memory.

---

# 477. No network request.

---

# 478. Typed lookup.

---

# 479. O(1) or near.

---

# 480. Hard rule.

---

# 481. Catalog refresh

background.

---

# 482. Metrics upload

low priority.

---

# 483. No render/action waits on analytics.

---

# 484. Hard rule.

---

# 485. Memory

Small catalog.

---

# 486. No per-user experiment history cache.

---

# 487. Hard rule.

---

# 488. Storage

Client:

```text
active catalog
experiment-local assignments
local aggregate counters
```

---

# 489. Server:

```text
signed experiment definitions
aggregate outcome tables
guardrail state
```

---

# 490. No user assignment database.

---

# 491. Hard rule.

---

# 492. Crate Layout

Recommended:

```text
crates/
├── siar-experiment-core/
├── siar-feature-flags/
├── siar-experiment-catalog/
├── siar-experiment-assignment/
├── siar-rollout/
├── siar-experiment-metrics/
├── siar-experiment-guardrails/
├── siar-experiment-governance/
├── siar-experiment-observability/
└── siar-experiment-testkit/
```

---

# 493. `siar-experiment-core`

Owns:

```text
experiment IDs
variants
states
errors
```

---

# 494. `siar-feature-flags`

Typed feature IDs/evaluation.

---

# 495. `siar-experiment-catalog`

Signed definitions/lifetimes.

---

# 496. `siar-experiment-assignment`

Local scoped assignment.

---

# 497. `siar-rollout`

Canary/gradual/rollback state.

---

# 498. `siar-experiment-metrics`

Aggregate outcomes using Part 92 registry.

---

# 499. `siar-experiment-guardrails`

Privacy/security/reliability guardrails.

---

# 500. `siar-experiment-governance`

Approvals/review/expiry.

---

# 501. `siar-experiment-observability`

Catalog/rollout health only.

---

# 502. `siar-experiment-testkit`

assignment/privacy/rollback/tenant tests.

---

# 503. Error Taxonomy

```rust
pub enum ExperimentError {
    UnknownExperiment,
    ExperimentExpired,
    ExperimentNotApproved,
    Ineligible,
    InvalidVariant,
    CatalogInvalid,
    CatalogRollback,
    SensitiveSegmentationForbidden,
    AnalyticsConsentDenied,
    PrivacyModeDenied,
    GuardrailFailed,
    RolloutPaused,
    Internal,
}
```

---

# 504. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. Experiment assignment identifiers are scoped to one experiment and are not reusable as cross-experiment or cross-product user identifiers.
2. Sensitive personal traits, private content, social-graph position, message/search history, and other protected data are forbidden experiment segmentation inputs.
3. Hard security, privacy, anonymity, authorization, and cryptographic invariants cannot be weakened by feature flags, experiments, rollout percentages, or emergency configuration.
4. Product experiment assignments are local by default; servers receive aggregate outcomes rather than user-to-variant assignment tables.
5. Product measurements use the approved analytics metric registry and cannot create an alternate raw event pipeline.
6. Every experiment has a defined hypothesis, owner, scope, lifetime, primary metric, guardrails, rollback path, and explicit end date.
7. Experiment assignment expires with the experiment and cannot become a permanent identity or fingerprint.
8. Rollout and experimentation remain distinct: safety rollouts can proceed using operational metrics without behavioral A/B tracking.
9. Privacy/security guardrail failures override positive product metrics and trigger pause or rollback.
10. Max-anonymity mode and disabled analytics consent prevent product experimentation/measurement from silently re-enabling tracking.
11. Managed tenant experiments remain within managed scope and cannot observe or alter unrelated personal profiles or other tenants.
12. Experiment archives retain definitions and aggregate conclusions—not user-level exposure, assignment, or behavioral histories.
```

---

# 505. Initial Production Scope

Implement first:

```text
typed FeatureId/ExperimentId/VariantId
signed experiment catalog
local random/deterministic assignment
experiment-scoped local secret
explicit eligibility policies
bounded experiment lifetime
control/variant/holdout model
gradual rollout phases
feature-flag evaluation
guardrail metrics
aggregate experiment outcomes
analytics-consent integration
max-anonymity experiment disable
tenant-managed experiment scope
rollback/emergency disable
privacy-safe experiment dashboard
experimentation testkit
```

Then add:

```text
differentially private experiment result releases
secure aggregation of experiment outcomes
sequential testing support
advanced service-shard experimentation
formal rollout/rollback verification
cross-release experiment continuity where justified
```

---

# 506. Definition of Done

Part 93 is complete when:

- assignments are experiment-scoped and unlinkable across experiments
- no stable experimentation user ID exists
- sensitive traits/content cannot be used for segmentation
- feature flags cannot override hard security/privacy floors
- experiments have explicit hypothesis/lifetime/metrics/guardrails
- product outcomes are aggregate-only
- analytics consent/privacy mode controls measurement
- rollout and A/B testing are separate concepts
- guardrail failures automatically pause/rollback
- tenant experiments stay tenant-managed
- federation never shares experiment assignments
- expired assignments are deleted/not restored
- dashboards contain no user drill-down
- assignment/rollback/privacy/fuzz/formal tests are specified

---

# 507. Final Architecture

```text
                 SIGNED EXPERIMENT CATALOG
                           │
                           ▼
                  ELIGIBILITY EVALUATION
                           │
                           ▼
                  LOCAL COHORT ASSIGNMENT
                  ┌────────┼────────┐
                  │        │        │
               CONTROL   VARIANT A VARIANT B
                  │        │        │
                  └────────┼────────┘
                           ▼
                    FEATURE EXPOSURE
                           │
                           ▼
                LOCAL AGGREGATE OUTCOMES
                           │
                           ▼
               GUARDRAILS / PRIVACY CHECKS
                           │
                           ▼
                 PRODUCT / ROLLOUT DECISION
```

Experimentation safety model:

```text
local scoped assignment
+
explicit hypothesis
+
bounded lifetime
+
aggregate outcomes
+
guardrails
+
signed rollout policy
+
privacy consent
+
no cross-experiment identity
```

not:

```text
give every user a permanent experimentation ID, record every exposure and action, and join the data across every product surface forever
```

---

# 508. Final Principle

Experimentation should validate a specific hypothesis with the minimum data and authority needed, then disappear when the question is answered.

The correct model is:

```text
define narrowly
+
assign locally
+
measure aggregates
+
protect small cohorts
+
respect consent
+
guard security/privacy
+
roll back quickly
+
expire everything
```

This architecture gives SIAR a privacy-preserving foundation for feature evaluation, staged rollouts, A/B testing, holdouts, canaries, tenant experiments, and product validation while preserving the anonymity, local-first, least-authority, and anti-surveillance guarantees established across Parts 34–92.
