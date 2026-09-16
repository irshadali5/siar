# Core System Architecture Part 92 — Anonymous Network Analytics, Metrics, Product Insights, Privacy-Preserving Aggregation, Differential Privacy & Anti-Surveillance Data Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 92  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 42, 46–47, 51, 55–56, 61, 67–69, 73–75, 78–91

**Primary purpose:** define SIAR's analytics, metrics, product-insight, privacy-preserving aggregation, differential-privacy, local analytics, telemetry governance, retention, deletion, privacy-budget, tenant/operator isolation, and anti-surveillance data architecture so observability and product learning can exist without creating user-level behavioral profiles, social graphs, or hidden tracking systems.

---

# 1. Purpose

Analytics is useful for answering questions such as:

```text
Is the app crashing?
Are notifications being delivered?
Which feature is failing?
Is a release causing regressions?
Are users able to complete onboarding?
```

But analytics can easily become surveillance by collecting:

```text
every click
every message open
every contact edge
every search
every location
every session
every feature path
```

The governing principle is:

> **SIAR analytics must be designed to answer narrowly defined product and operational questions using the least data necessary, favoring local and aggregate computation over user-level event histories.**

---

# 2. Architectural Position

```text
Local / Service Events
        │
        ▼
Telemetry Classification
        │
        ├── LocalOnly
        ├── SafeAggregate
        ├── SensitiveAggregate
        └── Forbidden
        │
        ▼
Aggregation / Threshold / DP
        │
        ▼
Metrics / Product Insight
```

---

# 3. Core Separation

Keep distinct:

```text
operational telemetry
product analytics
security telemetry
privacy telemetry
business/accounting metrics
user content
```

---

# 4. Non-Goals

Part 92 does not create:

```text
a global user event warehouse
session replay analytics
cross-product user profiles
ad-targeting profiles
social graph analytics
per-user behavioral scoring
```

---

# 5. Telemetry Privacy Classes

```rust
pub enum TelemetryPrivacyClass {
    LocalOnly,
    SafeAggregate,
    SensitiveAggregate,
    Forbidden,
}
```

---

# 6. LocalOnly

Never leaves device/service boundary.

Examples:

```text
local debug timing
UI performance trace
local search history metrics
local recommendation state
```

---

# 7. SafeAggregate

Can be reported after aggregation.

Examples:

```text
crash count
request latency bucket
feature success count
```

---

# 8. SensitiveAggregate

Requires stronger controls.

Examples:

```text
small-cohort feature adoption
security-event trends
regional capacity
```

---

# 9. Forbidden

Never collected remotely.

Examples:

```text
message plaintext
contact graph
search terms
exact interaction graph
precise location history
private feed sequence
```

---

# 10. Hard Rule

Every telemetry field has an explicit privacy class.

---

# 11. Telemetry Descriptor

```rust
pub struct TelemetryDescriptor {
    pub metric: MetricId,
    pub privacy: TelemetryPrivacyClass,
    pub purpose: TelemetryPurpose,
    pub retention: RetentionClass,
}
```

---

# 12. Telemetry Purpose

```rust
pub enum TelemetryPurpose {
    Reliability,
    Performance,
    Security,
    ProductQuality,
    Capacity,
    BillingAggregate,
}
```

---

# 13. Purpose Binding

Data collected for one purpose is not silently reused for another.

---

# 14. Hard Rule

Purpose expansion requires explicit policy/schema change.

---

# 15. Data Minimization

Prefer:

```text
counter
histogram
bucket
```

over:

```text
raw event stream
```

---

# 16. Example

Prefer:

```text
sync_success_count += 1
```

not:

```text
store full sync event with user/device/object IDs
```

---

# 17. Hard Rule

If aggregate answers the product question, raw events are forbidden.

---

# 18. Metric Types

```rust
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
    Distribution,
    BooleanRate,
}
```

---

# 19. No Free-Form Event Blob

Hard rule.

---

# 20. Metric Schema

Typed.

---

# 21. Example

```rust
pub struct MetricSample {
    pub metric: MetricId,
    pub value: MetricValue,
    pub dimensions: SmallVec<[MetricDimension; 4]>,
}
```

---

# 22. Dimensions

Strictly allowlisted.

---

# 23. No Arbitrary Tags

Hard rule.

---

# 24. Why

Arbitrary dimensions cause accidental user identifiers.

---

# 25. Allowed Dimensions

Examples:

```text
app_version
platform
service_class
coarse_region
error_class
```

---

# 26. Forbidden Dimensions

Examples:

```text
user_id
account_id
relationship_id
message_id
email
phone
exact_ip
exact_location
```

---

# 27. Hard Rule

Stable user identifiers are forbidden telemetry dimensions by default.

---

# 28. Device Identifier

No stable global device ID.

---

# 29. Hard Rule

Telemetry must not create one.

---

# 30. Anonymous Installation Cohort

Optional short-lived random identifier.

---

# 31. Default

Avoid.

---

# 32. If needed

Short TTL and local rotation.

---

# 33. No cross-product reuse.

---

# 34. Hard rule.

---

# 35. Session Analytics

Avoid persistent session tracking.

---

# 36. Local session timing

Can stay local.

---

# 37. Aggregate session count

Can be derived locally.

---

# 38. No server-side detailed session path by default.

---

# 39. Hard rule.

---

# 40. Product Insights

Examples:

```text
onboarding completion rate
feature error rate
crash-free sessions
sync success
notification delivery
```

---

# 41. Good Question

"How often does backup fail?"

---

# 42. Bad Question

"Which exact users opened backup settings after reading which message?"

---

# 43. Hard Rule

Analytics questions should be framed at aggregate/system level.

---

# 44. Product Insight Definition

```rust
pub struct ProductInsightSpec {
    pub question: InsightQuestionId,
    pub source_metrics: Vec<MetricId>,
    pub minimum_cohort: u32,
    pub privacy_budget: Option<PrivacyBudget>,
}
```

---

# 45. Minimum Cohort

Required for sensitive aggregates.

---

# 46. Small Cohort

Suppress result.

---

# 47. Hard Rule

No low-cardinality slicing that can deanonymize individuals.

---

# 48. Cohort Examples

Allowed:

```text
platform=Android
version=1.4.x
region=South Asia
```

---

# 49. Avoid combinations like:

```text
platform
+
rare version
+
small region
+
specific hour
```

---

# 50. Cohort Explosion

Dangerous.

---

# 51. Query Planner

Should enforce minimum result size.

---

# 52. No Analyst Override Without Governance

Hard rule.

---

# 53. Differential Privacy

Optional but valuable for some aggregate metrics.

---

# 54. Differential Privacy Goal

Reduce inference about individual contribution.

---

# 55. DP Is Not Magic

It requires:

```text
bounded contribution
privacy budget
noise mechanism
cohort definition
composition accounting
```

---

# 56. Hard Rule

Never claim "differentially private" without explicit epsilon/delta accounting and mechanism definition.

---

# 57. Privacy Budget

```rust
pub struct PrivacyBudget {
    pub epsilon_micros: u64,
    pub delta_nanos: u64,
}
```

---

# 58. Fixed-Point Representation

Avoid float policy ambiguity.

---

# 59. Budget Scope

```rust
pub enum PrivacyBudgetScope {
    Metric,
    Cohort,
    Release,
    TimeWindow,
}
```

---

# 60. Budget Consumption

Tracked.

---

# 61. No Infinite DP Queries

Hard rule.

---

# 62. DP Mechanisms

Reviewed standard implementations only.

---

# 63. Possible:

```text
Laplace
Gaussian
randomized response
```

---

# 64. No Custom Privacy Mechanism

Hard rule.

---

# 65. Bounded Contribution

Critical.

---

# 66. Example

One device can contribute at most:

```text
1 onboarding completion per day
```

---

# 67. Hard Rule

Unbounded event contribution invalidates privacy assumptions.

---

# 68. Contribution Bound

```rust
pub struct ContributionBound {
    pub max_per_window: u32,
    pub window: Duration,
}
```

---

# 69. Local Clipping

Preferred.

---

# 70. Server clipping

Defense in depth.

---

# 71. Secure Aggregation

Optional.

---

# 72. Goal

Server receives aggregate, not individual contribution.

---

# 73. Useful for:

```text
feature usage counts
crash counts
coarse device capability distributions
```

---

# 74. Not required for all metrics.

---

# 75. No Custom Secure Aggregation Protocol

Hard rule.

---

# 76. Initial Baseline

Local aggregation + thresholded reporting.

---

# 77. Future

Reviewed secure aggregation.

---

# 78. Local Analytics

Strong default.

---

# 79. Client computes:

```text
feature counters
error buckets
performance histograms
```

---

# 80. Upload only aggregate.

---

# 81. No raw clickstream.

---

# 82. Hard rule.

---

# 83. Local Metric Window

```rust
pub struct LocalMetricWindow {
    pub start_epoch: CoarseEpoch,
    pub counters: BTreeMap<MetricId, MetricAccumulator>,
}
```

---

# 84. Coarse Epoch

No precise event timestamp.

---

# 85. Hard Rule

Precise user-level timestamps are avoided unless operationally necessary.

---

# 86. Aggregation Epoch

Examples:

```text
hour
day
week
```

---

# 87. Product analytics

Daily often sufficient.

---

# 88. Security telemetry

May need finer but still bounded.

---

# 89. Remote Analytics Envelope

```rust
pub struct AggregateTelemetryEnvelope {
    pub schema_version: TelemetrySchemaVersion,
    pub coarse_epoch: CoarseEpoch,
    pub samples: Vec<MetricSample>,
}
```

---

# 90. No user identifier.

---

# 91. No raw event IDs.

---

# 92. Hard rule.

---

# 93. Telemetry Transport

Can use normal authenticated service path.

---

# 94. Max-Anonymity Mode

Remote product analytics off by default.

---

# 95. Operational telemetry may be locally aggregated or disabled per policy.

---

# 96. Hard Rule

Privacy mode cannot silently re-enable analytics.

---

# 97. User Control

Analytics preference explicit.

---

# 98. Possible modes:

```rust
pub enum AnalyticsConsent {
    Disabled,
    EssentialOnly,
    AggregateProductInsights,
}
```

---

# 99. Disabled

No remote analytics except strictly necessary service operations if declared.

---

# 100. EssentialOnly

Reliability/security minimum.

---

# 101. AggregateProductInsights

Allows approved aggregate metrics.

---

# 102. No Hidden "Improve Product" Catch-All

Hard rule.

---

# 103. Transparency

User can inspect:

```text
what categories are collected
why
retention
```

---

# 104. No hidden telemetry schema.

---

# 105. Hard rule.

---

# 106. Local Inspection

Optional diagnostics view.

---

# 107. Can show current metric categories, not raw sensitive state.

---

# 108. Telemetry Schema Registry

Machine-readable.

---

# 109. Every metric includes:

```text
purpose
privacy class
retention
dimensions
contribution bound
```

---

# 110. No Unregistered Metric Emission

Hard rule.

---

# 111. Rust Registry Example

```rust
pub struct MetricDescriptor {
    pub id: MetricId,
    pub privacy: TelemetryPrivacyClass,
    pub purpose: TelemetryPurpose,
    pub allowed_dimensions: &'static [MetricDimensionKind],
    pub retention: RetentionClass,
}
```

---

# 112. Metric Emission Trait

```rust
pub trait TelemetrySink {
    fn record(
        &self,
        descriptor: &'static MetricDescriptor,
        sample: MetricSample,
    ) -> Result<(), TelemetryError>;
}
```

---

# 113. Compile-Time/Static Registration

Preferred.

---

# 114. No Dynamic Ad-Hoc Metrics In Production

Hard rule.

---

# 115. Dimension Type

```rust
pub enum MetricDimension {
    AppVersion(AppVersionClass),
    Platform(PlatformClass),
    Service(ServiceClass),
    Region(CoarseRegion),
    Error(ErrorClass),
}
```

---

# 116. No String Key/Value Pairs

Preferred hard rule.

---

# 117. Cardinality Guard

Metric backend rejects high-cardinality dimension sets.

---

# 118. Hard rule.

---

# 119. Data Retention

Telemetry retention should be short.

---

# 120. Suggested classes:

```rust
pub enum TelemetryRetention {
    Hours24,
    Days7,
    Days30,
    Days90,
}
```

---

# 121. Longer Retention

Requires explicit justification.

---

# 122. No "Keep Forever"

Hard rule.

---

# 123. Aggregated Historical Trends

May retain aggregate series longer.

---

# 124. Raw/pre-aggregation data

Short-lived.

---

# 125. Hard rule.

---

# 126. Retention Transformation

Example:

```text
raw aggregate window
→ weekly aggregate
→ monthly aggregate
→ delete source rows
```

---

# 127. No ability to drill back to user.

---

# 128. Hard rule.

---

# 129. Deletion

Account deletion should remove account-linked analytics if any exist.

---

# 130. But preferred architecture has no durable account-link.

---

# 131. Hard rule.

---

# 132. Telemetry Anti-Resurrection

Deleted/expired telemetry should not return from backups.

---

# 133. Backup retention aligned.

---

# 134. No analytics backup with infinite retention.

---

# 135. Hard rule.

---

# 136. Operational Metrics

Examples:

```text
CPU
memory
queue depth
request latency
error rate
```

---

# 137. These are infrastructure metrics.

---

# 138. They must not contain user payload labels.

---

# 139. Hard rule.

---

# 140. Request Metrics

Use route template.

---

# 141. Not full URL/query.

---

# 142. Example:

```text
POST /v1/messages
```

not:

```text
POST /v1/messages?contact=...
```

---

# 143. Hard rule.

---

# 144. Error Metrics

Use error class.

---

# 145. Not raw error message if it may contain sensitive data.

---

# 146. Hard rule.

---

# 147. Structured Logs

Separate from metrics.

---

# 148. Logs retention short.

---

# 149. Sensitive fields redacted.

---

# 150. No private payload.

---

# 151. Hard rule.

---

# 152. Distributed Tracing

Part 51.

---

# 153. Analytics must not piggyback on trace IDs to create user history.

---

# 154. Trace IDs short-lived/infrastructure-scoped.

---

# 155. No user-level persistent trace join.

---

# 156. Hard rule.

---

# 157. Product Funnel Analytics

Potentially invasive.

---

# 158. Preferred model

Local funnel aggregation.

---

# 159. Example:

```text
started onboarding = 100
completed onboarding = 82
```

---

# 160. Upload counts only.

---

# 161. No per-user step sequence.

---

# 162. Hard rule.

---

# 163. Funnel Window

Coarse.

---

# 164. No precise event timing.

---

# 165. Funnel State

Local ephemeral.

---

# 166. Delete after aggregate.

---

# 167. Hard rule.

---

# 168. Feature Adoption

Count aggregate enabled/used.

---

# 169. Avoid per-user long-lived feature map.

---

# 170. Hard rule.

---

# 171. A/B Testing

High privacy risk.

---

# 172. Not baseline.

---

# 173. If introduced

Use:

```text
local assignment
short-lived experiment ID
aggregate outcomes
minimum cohort
```

---

# 174. No hidden permanent experimentation ID.

---

# 175. Hard rule.

---

# 176. Experiment Identity

```rust
pub struct ExperimentAssignment {
    pub experiment: ExperimentId,
    pub variant: ExperimentVariant,
    pub expires_at: Timestamp,
}
```

---

# 177. Stored locally.

---

# 178. Upload only aggregate outcome.

---

# 179. No cross-experiment linkage.

---

# 180. Hard rule.

---

# 181. Personalization Analytics

Not baseline.

---

# 182. Local personalization Part 84/87.

---

# 183. No behavioral model upload.

---

# 184. Hard rule.

---

# 185. Search Analytics

Part 91.

---

# 186. No raw queries.

---

# 187. Allowed:

```text
query latency
zero-result rate aggregate
```

---

# 188. Zero-result metric

Without terms.

---

# 189. Hard rule.

---

# 190. Feed Analytics

Part 87.

---

# 191. No dwell time profile.

---

# 192. Allowed:

```text
timeline load latency
candidate fetch error rate
```

---

# 193. Hard rule.

---

# 194. Notification Analytics

Part 90.

---

# 195. No open-propensity model.

---

# 196. Allowed:

```text
push invalid token rate
delivery failure
digest latency
```

---

# 197. Hard rule.

---

# 198. Contact/Social Analytics

Parts 85–86.

---

# 199. Forbidden:

```text
contact degree
group membership graph
mutual contacts
```

---

# 200. Hard rule.

---

# 201. Interaction Analytics

Part 89.

---

# 202. Public aggregate comment/reaction counts can exist.

---

# 203. No actor graph.

---

# 204. Hard rule.

---

# 205. Publishing Analytics

Part 88.

---

# 206. Creator insights thresholded.

---

# 207. No viewer list.

---

# 208. Hard rule.

---

# 209. Security Analytics

Needed carefully.

---

# 210. Examples:

```text
failed authentication count
refresh replay count
revoked credential attempts
```

---

# 211. These can be sensitive.

---

# 212. Use service/account security systems, not product warehouse.

---

# 213. Hard rule.

---

# 214. Security Analytics Store

Separate access.

---

# 215. Short retention.

---

# 216. Incident-specific expansion requires incident authorization.

---

# 217. No blanket indefinite logging.

---

# 218. Hard rule.

---

# 219. Privacy Telemetry

Measure privacy system health.

---

# 220. Examples:

```text
privacy-mode adoption aggregate
direct-fallback prevented count
telemetry suppression count
```

---

# 221. Avoid revealing which user.

---

# 222. Hard rule.

---

# 223. Differential Privacy Use Cases

Suitable:

```text
feature adoption
coarse platform distribution
coarse crash rate
coarse public trend stats
```

---

# 224. Less suitable:

```text
high-cardinality debugging
security forensics
real-time operational SLOs
```

---

# 225. Hard rule.

---

# 226. DP Release Pipeline

```text
aggregate raw metric
→ clip contribution
→ enforce cohort threshold
→ consume privacy budget
→ add calibrated noise
→ publish aggregate
```

---

# 227. No Noise Before Bounding Contribution

Hard rule.

---

# 228. DP Release Record

```rust
pub struct DifferentialPrivacyRelease {
    pub metric: MetricId,
    pub window: CoarseEpoch,
    pub epsilon: PrivacyBudget,
    pub cohort_size_lower_bound: u32,
}
```

---

# 229. Audit DP Release

Metadata only.

---

# 230. No source records.

---

# 231. Privacy Budget Ledger

Durable.

---

# 232. Prevent repeated-query privacy leakage.

---

# 233. Hard rule.

---

# 234. Analyst Query Layer

Very constrained.

---

# 235. Prefer predeclared metrics.

---

# 236. No arbitrary SQL over raw behavioral events.

---

# 237. Hard rule.

---

# 238. Analytics Query API

```rust
pub trait AnalyticsQueryService {
    fn query(
        &self,
        spec: ApprovedInsightQuery,
    ) -> Result<InsightResult, AnalyticsError>;
}
```

---

# 239. Approved Insight Query

Pre-registered.

---

# 240. No analyst-supplied arbitrary dimension joins.

---

# 241. Hard rule.

---

# 242. Product Dashboard

Displays approved aggregates.

---

# 243. No drill-down to user.

---

# 244. Hard rule.

---

# 245. Tenant Analytics

Part 69.

---

# 246. Tenant can see aggregate organization-owned metrics.

---

# 247. Cannot see personal usage outside managed profile.

---

# 248. Hard rule.

---

# 249. Tenant Metrics

Examples:

```text
active managed devices bucket
managed message-delivery health
storage usage
license seats
```

---

# 250. Avoid:

```text
which employee messaged whom
private personal app usage
```

---

# 251. Hard rule.

---

# 252. Tenant Cohort Threshold

Still applies.

---

# 253. Very small tenant

may get only operational totals that are already inherently known.

---

# 254. No pretending anonymity in group of one.

---

# 255. Hard truth.

---

# 256. Federation Analytics

Part 58.

---

# 257. Exchange only aggregate service-level metrics.

---

# 258. No cross-domain user analytics.

---

# 259. Hard rule.

---

# 260. Federation Metrics

Examples:

```text
delivery latency
failure rate
volume bucket
```

---

# 261. No member/user IDs.

---

# 262. No per-peer behavioral histories.

---

# 263. Hard rule.

---

# 264. Operator Analytics

Infrastructure only.

---

# 265. Operator should not gain product-level user insights by default.

---

# 266. Hard rule.

---

# 267. Data Lake

Avoid as baseline.

---

# 268. If introduced for aggregates

Store only approved aggregate tables.

---

# 269. No raw user event lake.

---

# 270. Hard rule.

---

# 271. Warehouse Schema

Aggregate-oriented.

---

# 272. Example table:

```text
metric_id
coarse_epoch
platform
version
count
```

---

# 273. Not:

```text
user_id
event_name
timestamp
metadata_json
```

---

# 274. Hard rule.

---

# 275. Metric Rollups

Hourly → daily → weekly.

---

# 276. Delete lower-level after rollup.

---

# 277. No indefinite high-resolution history.

---

# 278. Hard rule.

---

# 279. Coarse Region

Optional.

---

# 280. Region granularity must meet privacy threshold.

---

# 281. Never precise GPS.

---

# 282. Hard rule.

---

# 283. IP Address

May exist transiently at network edge.

---

# 284. Not copied into analytics.

---

# 285. Hard rule.

---

# 286. User Agent

Avoid raw full string.

---

# 287. Parse into coarse platform/app version locally/service edge.

---

# 288. Discard raw.

---

# 289. Hard rule.

---

# 290. Fingerprinting Resistance

Telemetry dimensions cannot create unique fingerprint.

---

# 291. Cardinality estimator checks combination uniqueness.

---

# 292. High uniqueness

reject/suppress.

---

# 293. Hard rule.

---

# 294. Metric Cardinality Budget

```rust
pub struct CardinalityBudget {
    pub max_distinct_dimension_combinations: u32,
}
```

---

# 295. Per metric.

---

# 296. No unbounded labels.

---

# 297. Telemetry Sampling

Can reduce volume.

---

# 298. But sampling must not create stable cohort identity.

---

# 299. Random local sampling.

---

# 300. No persistent user bucket.

---

# 301. Hard rule.

---

# 302. Sampling Rate

Signed config.

---

# 303. Users in max privacy can disable product telemetry.

---

# 304. No sample override.

---

# 305. Hard rule.

---

# 306. Error Reporting

Crash reports can leak.

---

# 307. Default crash report:

```text
stack symbols
version
platform
error class
```

---

# 308. Exclude:

```text
memory dumps
message content
paths with user names
environment secrets
```

---

# 309. Hard rule.

---

# 310. Minidump

Only with explicit safer pipeline/consent.

---

# 311. Redaction.

---

# 312. No automatic upload of full memory.

---

# 313. Hard rule.

---

# 314. Panic Messages

Sanitize.

---

# 315. No secret values.

---

# 316. Part 80 secret wrappers help.

---

# 317. Analytics Governance

Metric addition requires review.

---

# 318. Review questions:

```text
What question does it answer?
Can aggregate suffice?
What privacy class?
What retention?
What dimensions?
```

---

# 319. No metric without owner/purpose.

---

# 320. Hard rule.

---

# 321. Metric Lifecycle

```rust
pub enum MetricLifecycleState {
    Proposed,
    Approved,
    Active,
    Deprecated,
    Removed,
}
```

---

# 322. Deprecated metrics stop emitting.

---

# 323. Removed metrics deleted from pipelines.

---

# 324. No zombie telemetry.

---

# 325. Hard rule.

---

# 326. Schema Versioning

Telemetry schema versioned.

---

# 327. Old clients can emit older approved schema.

---

# 328. Server validates.

---

# 329. Unknown metric rejected.

---

# 330. Hard rule.

---

# 331. Remote Config

Cannot introduce arbitrary new metric names.

---

# 332. Only enable/disable precompiled descriptors.

---

# 333. Hard rule.

---

# 334. Why

Prevents remote analytics expansion without release review.

---

# 335. Audit

High-level analytics configuration changes.

---

# 336. No audit of individual contributions.

---

# 337. Hard rule.

---

# 338. Privacy Review Manifest

Each release can include:

```text
added metrics
removed metrics
changed dimensions
changed retention
changed DP budget
```

---

# 339. Useful CI gate.

---

# 340. No silent telemetry expansion between releases.

---

# 341. Hard rule.

---

# 342. Supply-Chain Integration

Analytics SDKs are risk.

---

# 343. Avoid third-party analytics SDK baseline.

---

# 344. Hard rule.

---

# 345. Why

Third-party SDK may collect undocumented metadata.

---

# 346. Prefer in-house minimal Rust telemetry crate.

---

# 347. External analytics service

Only if it accepts already-aggregated data.

---

# 348. No raw device SDK.

---

# 349. Hard rule.

---

# 350. Mobile Analytics

Android app should not embed tracking SDK by default.

---

# 351. Desktop same.

---

# 352. Web if any

No third-party tracker baseline.

---

# 353. Hard rule.

---

# 354. Cookies/Tracking IDs

Analytics does not require.

---

# 355. Hard rule.

---

# 356. Product Insight Storage

Approved aggregate store.

---

# 357. Access Control

Part 81.

---

# 358. Roles:

```text
ProductAnalyst
ReliabilityEngineer
SecurityAnalyst
PrivacyReviewer
```

---

# 359. Separate scopes.

---

# 360. Product analyst cannot query security store.

---

# 361. Security analyst cannot access private message data.

---

# 362. Hard rule.

---

# 363. Analytics Admin

No universal raw-data superuser.

---

# 364. Hard rule.

---

# 365. Operator Break-Glass

Not for product analytics.

---

# 366. No emergency "download all telemetry" path.

---

# 367. Hard rule.

---

# 368. Analytics Service Identity

Part 79 workload identity.

---

# 369. Ingestion endpoints mTLS internally.

---

# 370. External client upload uses authenticated app endpoint if needed.

---

# 371. No secret API key in app.

---

# 372. Hard rule.

---

# 373. Ingestion Validation

Check:

```text
known metric
allowed dimensions
value bounds
contribution bounds
schema version
```

---

# 374. Reject invalid/high-cardinality data.

---

# 375. Hard rule.

---

# 376. Server Trust

Client metrics are untrusted.

---

# 377. They can be manipulated.

---

# 378. Do not use product analytics as security authority.

---

# 379. Hard rule.

---

# 380. Fraud/Abuse Analytics

Separate from product analytics.

---

# 381. Do not combine stores.

---

# 382. Hard rule.

---

# 383. Billing Analytics

Part 47/54.

---

# 384. Billing requires accurate scoped accounting.

---

# 385. Not differential privacy.

---

# 386. Keep billing identity separate from product analytics.

---

# 387. Hard rule.

---

# 388. Usage Billing Metrics

Only service units needed.

---

# 389. No message content.

---

# 390. No social graph.

---

# 391. Legal/Compliance

Part 55.

---

# 392. Data minimization documented.

---

# 393. User rights

analytics records should usually not be individually identifiable.

---

# 394. If identifiable data exists for security/legal reason

separate lifecycle and disclosure.

---

# 395. Hard rule.

---

# 396. Consent Integration

Part 56.

---

# 397. Product analytics consent explicit.

---

# 398. Consent changes apply prospectively.

---

# 399. Local buffer deletion on disable.

---

# 400. Hard rule.

---

# 401. Revocation Of Analytics Consent

Stop uploads.

---

# 402. Delete local unsent aggregate queue.

---

# 403. Server aggregate already anonymized may remain if not attributable.

---

# 404. Truthful policy.

---

# 405. No claim to delete non-identifiable aggregate from all historical reports.

---

# 406. Hard truth.

---

# 407. Max-Anonymity Mode

Remote product analytics:

```text
Disabled
```

by default.

---

# 408. Security-critical local diagnostics still available.

---

# 409. No hidden exception.

---

# 410. Hard rule.

---

# 411. Telemetry Queue

Local bounded queue.

---

# 412. Aggregate first.

---

# 413. Queue only aggregate envelopes.

---

# 414. No raw event queue.

---

# 415. Hard rule.

---

# 416. Queue Bounds

```text
items
bytes
age
```

---

# 417. Old aggregates can drop.

---

# 418. No correctness dependency.

---

# 419. Telemetry loss acceptable.

---

# 420. Hard rule.

---

# 421. Upload Scheduling

Battery/network aware.

---

# 422. Low priority.

---

# 423. No wake device just for analytics.

---

# 424. Hard rule.

---

# 425. Strict Mode

Batch/jitter uploads.

---

# 426. Reduce timing fingerprint.

---

# 427. No immediate event-triggered upload.

---

# 428. Hard rule.

---

# 429. Analytics Endpoint

Does not return personalized content.

---

# 430. Avoid side-channel linkage.

---

# 431. Hard rule.

---

# 432. Aggregation Service

```rust
pub trait AggregateTelemetryCollector {
    fn ingest(
        &self,
        envelope: AggregateTelemetryEnvelope,
    ) -> Result<(), AnalyticsError>;
}
```

---

# 433. Metric Registry

```rust
pub trait MetricRegistry {
    fn descriptor(
        &self,
        id: MetricId,
    ) -> Option<&'static MetricDescriptor>;
}
```

---

# 434. Local Aggregator

```rust
pub trait LocalTelemetryAggregator {
    fn record(
        &self,
        metric: &'static MetricDescriptor,
        value: MetricValue,
    ) -> Result<(), AnalyticsError>;

    fn flush_window(
        &self,
    ) -> Result<AggregateTelemetryEnvelope, AnalyticsError>;
}
```

---

# 435. DP Engine

```rust
pub trait DifferentialPrivacyEngine {
    fn release(
        &self,
        request: DpReleaseRequest,
    ) -> Result<DifferentialPrivacyRelease, AnalyticsError>;
}
```

---

# 436. Cohort Guard

```rust
pub trait CohortPrivacyGuard {
    fn allow(
        &self,
        cohort_size: u64,
        spec: &ProductInsightSpec,
    ) -> bool;
}
```

---

# 437. Insight Service

```rust
pub trait ProductInsightService {
    fn query(
        &self,
        insight: ApprovedInsightQuery,
    ) -> Result<InsightResult, AnalyticsError>;
}
```

---

# 438. No Raw Event Access Trait

Hard rule.

---

# 439. Analytics Error Taxonomy

```rust
pub enum AnalyticsError {
    UnknownMetric,
    ForbiddenMetric,
    InvalidDimension,
    CardinalityExceeded,
    ContributionExceeded,
    CohortTooSmall,
    PrivacyBudgetExceeded,
    ConsentDenied,
    SchemaMismatch,
    Internal,
}
```

---

# 440. Observability Of Analytics

Analytics pipeline itself needs metrics.

---

# 441. Allowed:

```text
ingestion failures
queue depth
DP budget errors
metric schema rejection
```

---

# 442. No payload contents.

---

# 443. Hard rule.

---

# 444. Analytics Pipeline SLOs

Examples:

```text
aggregate ingestion availability
metric schema validation
rollup freshness
DP release correctness
```

---

# 445. Privacy SLOs

```text
0 stable user IDs in product analytics
0 forbidden metrics emitted
0 raw query/message/contact graph ingestion
```

---

# 446. Security SLOs

```text
0 unauthorized analytics access
0 cross-tenant aggregate leakage
```

---

# 447. Failure Modes

```text
ingestion outage
aggregation bug
DP budget bug
schema drift
high-cardinality explosion
```

---

# 448. Ingestion Outage

Local aggregate queue bounded.

---

# 449. Old data can drop.

---

# 450. No user-facing failure.

---

# 451. Hard rule.

---

# 452. Aggregation Bug

Invalidate affected insight release.

---

# 453. Recompute from retained aggregate if available.

---

# 454. No need to retain raw user events.

---

# 455. Hard rule.

---

# 456. DP Budget Bug

Stop releases.

---

# 457. Review ledger.

---

# 458. No continue with guessed noise.

---

# 459. Hard rule.

---

# 460. Schema Drift

Unknown dimensions rejected.

---

# 461. No permissive JSON acceptance.

---

# 462. Hard rule.

---

# 463. Cardinality Explosion

Automatic circuit breaker.

---

# 464. Drop/reject metric series.

---

# 465. Alert privacy/ops team.

---

# 466. Hard rule.

---

# 467. Data Quality

Aggregates may be approximate.

---

# 468. Product dashboards should show uncertainty where DP/noise applies.

---

# 469. No false precision.

---

# 470. Hard rule.

---

# 471. Confidence Metadata

```rust
pub struct InsightConfidence {
    pub noisy: bool,
    pub cohort_lower_bound: Option<u64>,
    pub error_bound: Option<FixedPoint>,
}
```

---

# 472. Analysts can see.

---

# 473. No user-level data needed.

---

# 474. A/B Experiment Analysis

If enabled later

must consume privacy budget/thresholds.

---

# 475. No tiny experiment cohorts.

---

# 476. Hard rule.

---

# 477. Anti-Surveillance Invariants

Analytics architecture must not permit reconstruction of:

```text
social graph
search interest graph
notification attention graph
feed engagement graph
precise location history
```

---

# 478. Hard rule.

---

# 479. Joinability Risk

Even separate aggregate tables can combine.

---

# 480. Avoid shared pseudonymous IDs across datasets.

---

# 481. Hard rule.

---

# 482. Dataset Linkage

Aggregate tables use coarse dimensions only.

---

# 483. No common user key.

---

# 484. Hard rule.

---

# 485. Time Resolution

Coarse enough to prevent linkage.

---

# 486. High-anonymity mode

larger epochs.

---

# 487. Hard rule.

---

# 488. Sparse Metric Suppression

If only one/small number of events in bucket:

suppress.

---

# 489. No zero/one exact publication.

---

# 490. Hard rule.

---

# 491. Metric Publication Threshold

```rust
pub struct PublicationThreshold {
    pub minimum_count: u32,
}
```

---

# 492. Enforced before dashboard/export.

---

# 493. Product Analytics Export

Aggregate only.

---

# 494. No raw user data export.

---

# 495. Hard rule.

---

# 496. External BI Tool

If used

receives aggregate tables only.

---

# 497. No direct production DB/user data connection.

---

# 498. Hard rule.

---

# 499. Third-Party Analytics SaaS

Not baseline.

---

# 500. If ever used

only aggregate export.

---

# 501. No client SDK.

---

# 502. Hard rule.

---

# 503. Analytics And AI

AI may summarize aggregate dashboards.

---

# 504. But AI never receives raw private event history.

---

# 505. Hard rule.

---

# 506. AI Insight Input

```text
aggregate metrics
system health
DP releases
```

---

# 507. No user content.

---

# 508. No per-user timeline.

---

# 509. Hard rule.

---

# 510. Testing

Need analytics privacy testkit.

---

# 511. Test Scenarios

```text
forbidden metric
high-cardinality dimension
small cohort
privacy budget exhaustion
consent disabled
```

---

# 512. Forbidden Metric Test

Emission rejected.

---

# 513. Dimension Test

AccountId/user ID cannot compile/validate as metric dimension.

---

# 514. Cardinality Test

High cardinality rejected.

---

# 515. Cohort Test

Below threshold suppressed.

---

# 516. DP Budget Test

Budget exhaustion blocks release.

---

# 517. Contribution Test

Per-window bound enforced.

---

# 518. Consent Test

Disabled analytics prevents upload.

---

# 519. Max-Anonymity Test

Remote product analytics off.

---

# 520. Search Privacy Test

Raw terms absent.

---

# 521. Feed Privacy Test

No dwell-time upload.

---

# 522. Notification Privacy Test

No open-propensity metric.

---

# 523. Contact Privacy Test

No relationship-degree metric.

---

# 524. Tenant Test

Tenant A aggregate not visible to B.

---

# 525. Federation Test

No user-level analytics exchange.

---

# 526. Retention Test

Expired detailed rollups deleted.

---

# 527. Backup Test

Expired telemetry not resurrected.

---

# 528. Schema Drift Test

Unknown metric/dimension rejected.

---

# 529. Remote Config Test

Cannot introduce new metric dynamically.

---

# 530. Crash Report Test

No secret/private payload.

---

# 531. AI Insight Test

Only aggregate inputs allowed.

---

# 532. Fuzzing

Fuzz:

```text
telemetry envelope
metric schema
dimension set
DP release request
aggregate export format
```

---

# 533. Property Tests

Properties:

```text
forbidden telemetry can never enter remote analytics store
stable user identifiers can never appear in product-analytics dimensions
small cohorts can never produce published insight
privacy budget never increases through repeated queries
```

---

# 534. Formal Verification Targets

Strong candidates:

```text
metric privacy-class enforcement
DP budget accounting
cohort-threshold publication
retention/rollup state machine
```

---

# 535. Kani Candidate

privacy-class/dimension allowlist invariants.

---

# 536. TLA+ Candidate

aggregate window → rollup → expiry → backup restore.

---

# 537. Loom Candidate

concurrent local aggregation + consent disable + upload flush.

---

# 538. Performance

Analytics is low priority.

---

# 539. Never on critical user interaction path.

---

# 540. Local counters lock-free/low-contention where practical.

---

# 541. Flush batched.

---

# 542. No analytics network call synchronously from product action.

---

# 543. Hard rule.

---

# 544. Memory

Bounded metric registry.

---

# 545. Bounded local windows.

---

# 546. No unbounded event queue.

---

# 547. Hard rule.

---

# 548. Storage

Client:

```text
aggregate windows
consent state
metric schema
```

---

# 549. Server:

```text
aggregate telemetry
rollups
DP ledger
approved insights
```

---

# 550. No raw behavioral event table.

---

# 551. Hard rule.

---

# 552. Partitioning

By:

```text
metric
coarse time
coarse platform/service/region
```

---

# 553. Not user.

---

# 554. Hard rule.

---

# 555. Crate Layout

Recommended:

```text
crates/
├── siar-analytics-core/
├── siar-metric-registry/
├── siar-local-analytics/
├── siar-aggregate-ingest/
├── siar-analytics-rollup/
├── siar-differential-privacy/
├── siar-cohort-privacy/
├── siar-insight-service/
├── siar-analytics-governance/
├── siar-analytics-observability/
└── siar-analytics-testkit/
```

---

# 556. `siar-analytics-core`

Owns:

```text
metric IDs
privacy classes
purposes
errors
```

---

# 557. `siar-metric-registry`

Static approved descriptor registry.

---

# 558. `siar-local-analytics`

Local aggregation/windows/contribution bounds.

---

# 559. `siar-aggregate-ingest`

Validate aggregate envelopes.

---

# 560. `siar-analytics-rollup`

Hourly/daily/weekly retention transformation.

---

# 561. `siar-differential-privacy`

Reviewed DP mechanisms/privacy-budget ledger.

---

# 562. `siar-cohort-privacy`

Minimum cohort/cardinality/sparsity checks.

---

# 563. `siar-insight-service`

Preapproved aggregate insight queries.

---

# 564. `siar-analytics-governance`

Metric lifecycle/privacy review manifests.

---

# 565. `siar-analytics-observability`

Pipeline health only.

---

# 566. `siar-analytics-testkit`

privacy/retention/DP/consent/tenant tests.

---

# 567. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. Product analytics contains no stable global user, account, relationship, contact, message, or device identifier by default.
2. Every metric is statically registered with an explicit purpose, privacy class, retention policy, allowed dimensions, and contribution bound.
3. If an aggregate metric can answer a product question, collecting a raw user event stream for that purpose is forbidden.
4. High-cardinality or user-identifying dimensions are rejected by schema/type enforcement and runtime cardinality guards.
5. Raw queries, message content, contact graphs, interaction graphs, feed histories, notification-open histories, precise locations, and behavioral timelines are forbidden remote analytics data.
6. Sensitive aggregate insights require minimum cohort thresholds, and differential-privacy claims require explicit mechanism, bounded contribution, and privacy-budget accounting.
7. User consent and privacy mode directly control remote product telemetry; disabling analytics stops uploads and cannot be silently overridden remotely.
8. Managed tenant analytics is scoped to managed organization-owned data and cannot inspect unrelated personal activity or cross-tenant behavior.
9. Analytics retention is bounded, lower-resolution rollups replace detailed windows, and expired data cannot be resurrected from backups.
10. Third-party analytics SDKs and arbitrary analyst SQL over raw user event data are not part of the production architecture.
11. Analytics telemetry, operational metrics, security telemetry, billing records, and compliance data remain separate stores and authority domains.
12. Analytics optimizes system/product quality, not compulsive engagement, advertising profiles, psychographic inference, or social surveillance.
```

---

# 568. Initial Production Scope

Implement first:

```text
typed metric registry
privacy-class/purpose/retention metadata
allowlisted low-cardinality dimensions
local counter/histogram aggregation
aggregate-only upload envelopes
analytics consent modes
max-anonymity telemetry disable
bounded local telemetry queue
daily/weekly rollups
minimum cohort suppression
product insight service with preapproved queries
tenant-scoped aggregate metrics
security/product analytics store separation
privacy review manifest in CI
privacy-safe crash/error reporting
analytics testkit
```

Then add:

```text
reviewed differential-privacy mechanisms
privacy-budget ledger
secure aggregation for selected metrics
local experiment aggregation
aggregate-only external BI export
formal telemetry-policy verification
```

---

# 569. Definition of Done

Part 92 is complete when:

- every metric is explicitly classified and purpose-bound
- no stable user identifier is used in product analytics
- local aggregation replaces raw click/event collection where possible
- high-cardinality dimensions are structurally rejected
- raw search/feed/contact/interaction/notification behavioral histories are forbidden
- consent and anonymity mode control remote telemetry
- tenant and operator analytics remain isolated
- retention/rollups prevent indefinite detailed history
- small cohorts are suppressed
- DP, where used, has explicit privacy accounting
- third-party tracking SDKs are absent from baseline
- analytics queries are preapproved aggregate queries
- expired telemetry cannot reappear from backup
- product metrics cannot create social/interest/attention graphs
- privacy/DP/retention/consent/fuzz/formal tests are specified

---

# 570. Final Architecture

```text
                  LOCAL / SERVICE EVENTS
                           │
                           ▼
                 TELEMETRY CLASSIFIER
                           │
          ┌────────────────┼────────────────┐
          │                │                │
      LOCAL ONLY      SAFE AGGREGATE   FORBIDDEN
          │                │
          │                ▼
          │        LOCAL AGGREGATION
          │                │
          │                ▼
          │       COHORT / DP / ROLLUP
          │                │
          └────────────────┼────────────────┐
                           ▼                │
                    PRODUCT INSIGHTS        │
                                            ▼
                                      DISCARD / NEVER SEND
```

Analytics safety model:

```text
explicit metric registry
+
local aggregation
+
low-cardinality dimensions
+
short retention
+
cohort thresholds
+
optional differential privacy
+
strict purpose separation
+
no stable user IDs
+
no behavioral event warehouse
```

not:

```text
instrument every action, assign every user a durable analytics ID, send raw events to a third-party SDK, and reconstruct behavior later
```

---

# 571. Final Principle

Analytics should answer narrowly defined system and product questions without creating a reusable history of individual behavior.

The correct model is:

```text
ask a narrow question
+
collect the minimum aggregate
+
bound contribution
+
suppress small cohorts
+
add privacy protection where useful
+
retain briefly
+
separate purposes
+
never build user profiles
```

This architecture gives SIAR a privacy-preserving analytics foundation for reliability, product quality, capacity planning, tenant operations, privacy monitoring, and future differential-privacy releases while preserving the anonymity, local-first, least-authority, and anti-surveillance guarantees established across Parts 34–91.
