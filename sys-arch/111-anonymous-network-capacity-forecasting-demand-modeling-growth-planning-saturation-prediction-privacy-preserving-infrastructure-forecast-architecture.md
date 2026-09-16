# Core System Architecture Part 111 — Anonymous Network Capacity Forecasting, Demand Modeling, Growth Planning, Saturation Prediction & Privacy-Preserving Infrastructure Forecast Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 111  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 49, 63, 92–93, 100–110  

**Primary purpose:** define SIAR's infrastructure-forecast architecture for aggregate demand modeling, seasonal workload forecasting, saturation prediction, growth planning, confidence intervals, forecast error governance, capacity acquisition lead time, headroom/reserve planning, regional and dependency-aware demand, scenario forecasts, cost/reliability/residency constraints, and privacy-preserving prediction without building user-level behavioral profiles.

---

# 1. Purpose

Capacity planning cannot rely only on present utilization.

Infrastructure must answer:

```text
When will this service saturate?
How much capacity will next quarter require?
What happens during a region failure?
How much reserve is needed for a release?
Which resource becomes the bottleneck first?
How uncertain is the forecast?
```

The governing principle is:

> **SIAR capacity forecasting should predict infrastructure resource demand from aggregate technical workloads, explicit scenarios, and measured trends—without predicting individual users or constructing behavioral demand profiles.**

---

# 2. Architectural Position

```text
Aggregate Operational Signals
             │
             ▼
       Demand Normalization
             │
             ▼
          Time Series
             │
      ┌──────┼──────┐
      │      │      │
  Trend   Season   Events
      │      │      │
      └──────┼──────┘
             ▼
        Forecast Models
             │
             ▼
       Capacity Scenarios
             │
             ▼
       Saturation Horizon
             │
             ▼
        Growth / Reserve Plan
```

---

# 3. Core Separation

Keep distinct:

```text
observed demand
forecast demand
capacity
headroom
reserve
quota
budget
business projection
```

---

# 4. Non-Goals

Part 111 does not create:

```text
per-user usage forecasts
behavioral prediction
engagement forecasting
ad-targeting segments
exact future-demand claims
```

---

# 5. Forecast Scope

```rust
pub enum ForecastScope {
    Service(ServiceId),
    Region(RegionId),
    ResourcePool(ResourcePoolId),
    TenantAggregate(TenantId),
    FederationPeer(FederationDomainId),
    WorkClass(WorkClass),
}
```

---

# 6. No Global User Scope

Hard rule.

---

# 7. Tenant Aggregate

Only where contractual/capacity isolation requires.

---

# 8. No individual-user forecasting.

---

# 9. Hard rule.

---

# 10. Resource Forecast Dimension

Reuse Part 101 dimensions.

```rust
pub enum ForecastResource {
    Cpu,
    Memory,
    DiskIo,
    Storage,
    NetworkIngress,
    NetworkEgress,
    Connections,
    QueueDepth,
    CryptoOperations,
    DatabaseConnections,
    RelayBandwidth,
    MediaSessions,
}
```

---

# 11. Demand Unit

Forecast demand in technical units.

```rust
pub struct DemandUnit {
    pub resource: ForecastResource,
    pub value: FixedPoint,
}
```

---

# 12. No User Engagement Unit

Hard rule.

---

# 13. Demand Observation

```rust
pub struct DemandObservation {
    pub scope: ForecastScope,
    pub resource: ForecastResource,
    pub value: FixedPoint,
    pub interval: ForecastInterval,
}
```

---

# 14. Forecast Interval

```rust
pub enum ForecastInterval {
    Minute,
    FiveMinutes,
    Hour,
    Day,
    Week,
}
```

---

# 15. Use Coarsest Useful Interval

Hard rule.

---

# 16. Observation Inputs

Allowed:

```text
CPU utilization
queue throughput
connections
storage growth
network bytes
DB transactions
media sessions
```

---

# 17. Forbidden Baseline Inputs

```text
message content
contact graph
search queries
user engagement
stable user behavior histories
```

---

# 18. Hard rule.

---

# 19. Demand Normalization

Raw provider metrics differ.

---

# 20. Normalize to stable internal units.

---

# 21. Example

```text
provider vCPU utilization
→ normalized CPU demand units
```

---

# 22. Hard rule.

---

# 23. Capacity Denominator

Forecast utilization requires actual usable capacity.

```rust
pub struct CapacityDenominator {
    pub scope: ForecastScope,
    pub resource: ForecastResource,
    pub usable_capacity: FixedPoint,
}
```

---

# 24. Usable Capacity ≠ Nominal Capacity

Hard rule.

---

# 25. Reasons

```text
reserved headroom
failover reserve
OS overhead
security reserve
```

---

# 26. Hard rule.

---

# 27. Baseline Demand

Estimate current steady-state demand.

---

# 28. Baseline Model

```rust
pub struct BaselineDemand {
    pub resource: ForecastResource,
    pub median: FixedPoint,
    pub p95: FixedPoint,
    pub peak: FixedPoint,
}
```

---

# 29. Median Alone Insufficient

Hard rule.

---

# 30. Peak vs Sustained

Separate.

---

# 31. Hard rule.

---

# 32. Time-Series Decomposition

Demand may include:

```text
trend
daily seasonality
weekly seasonality
event spikes
noise
```

---

# 33. Decomposition State

```rust
pub struct DemandComponents {
    pub trend: ForecastSeries,
    pub seasonality: Vec<SeasonalComponent>,
    pub residual: ForecastSeries,
}
```

---

# 34. Seasonal Component

```rust
pub enum SeasonalComponent {
    Daily,
    Weekly,
    Monthly,
    CustomKnownPeriod(Duration),
}
```

---

# 35. No User Cohort Seasonality

Hard rule.

---

# 36. Growth Model

```rust
pub enum GrowthModel {
    Flat,
    Linear,
    ExponentialBounded,
    PiecewiseLinear,
    ExternalScenario,
}
```

---

# 37. Avoid Unbounded Exponential By Default

Hard rule.

---

# 38. Piecewise Linear

Often practical.

---

# 39. Hard rule.

---

# 40. External Scenario

Use explicit product/business forecast as scenario, not truth.

---

# 41. Hard rule.

---

# 42. Forecast Horizon

```rust
pub enum ForecastHorizon {
    Hours(u16),
    Days(u16),
    Weeks(u16),
    Months(u16),
}
```

---

# 43. Longer Horizon = More Uncertainty

Hard rule.

---

# 44. Forecast Point

```rust
pub struct ForecastPoint {
    pub time: CoarseTimestamp,
    pub expected: FixedPoint,
    pub lower: FixedPoint,
    pub upper: FixedPoint,
}
```

---

# 45. Confidence Bounds Mandatory For Long-Term Planning

Hard rule.

---

# 46. No Single-Line Certainty

Hard rule.

---

# 47. Forecast Confidence

```rust
pub enum ForecastConfidence {
    Low,
    Medium,
    High,
}
```

---

# 48. Confidence Based On

```text
history length
stability
seasonality
model error
known future events
```

---

# 49. Hard rule.

---

# 50. Forecast Model Identity

```rust
pub struct ForecastModelVersion(pub u64);
```

---

# 51. Historical forecasts reference exact model.

---

# 52. Hard rule.

---

# 53. Forecast Result

```rust
pub struct CapacityForecast {
    pub scope: ForecastScope,
    pub resource: ForecastResource,
    pub horizon: ForecastHorizon,
    pub model: ForecastModelVersion,
    pub confidence: ForecastConfidence,
    pub points: Vec<ForecastPoint>,
}
```

---

# 54. No User Predictions Embedded

Hard rule.

---

# 55. Forecast Error

Must be measured.

---

# 56. Error Metrics

```rust
pub struct ForecastError {
    pub mae: FixedPoint,
    pub relative_error: FixedPoint,
    pub bias: FixedPoint,
}
```

---

# 57. Avoid MAPE Near Zero

Hard rule.

---

# 58. Bias

Persistent under/overforecast.

---

# 59. Hard rule.

---

# 60. Backtesting

Every model should be backtested.

---

# 61. Backtest Window

```rust
pub struct BacktestWindow {
    pub train_end: CoarseTimestamp,
    pub test_end: CoarseTimestamp,
}
```

---

# 62. No In-Sample Accuracy Claims

Hard rule.

---

# 63. Model Selection

Prefer simplest adequate model.

---

# 64. Hard rule.

---

# 65. Model Candidates

Examples:

```text
moving average
seasonal naive
Holt-Winters-like
piecewise trend
robust regression
```

---

# 66. No Need For Opaque Deep ML Baseline

Hard rule.

---

# 67. Why

Infrastructure forecasts need explainability and stability.

---

# 68. Good.

---

# 69. Robustness

Outliers should not dominate long-term forecast.

---

# 70. But real spikes should not be erased.

---

# 71. Hard rule.

---

# 72. Event Classification

Distinguish:

```text
one-off incident
release spike
scheduled event
normal seasonality
structural growth
```

---

# 73. Event Marker

```rust
pub struct ForecastEvent {
    pub kind: ForecastEventKind,
    pub starts_at: CoarseTimestamp,
    pub ends_at: Option<CoarseTimestamp>,
}
```

---

# 74. Event Kind

```rust
pub enum ForecastEventKind {
    Release,
    Migration,
    Maintenance,
    Incident,
    RegionalFailover,
    PlannedCampaign,
    KnownCalendarEvent,
}
```

---

# 75. Planned Campaign

Business input only at aggregate system level.

---

# 76. Hard rule.

---

# 77. Incident Data

Incident spikes should not automatically become growth trend.

---

# 78. Hard rule.

---

# 79. Release Demand Shift

New release may alter:

```text
CPU
memory
network
storage
```

---

# 80. Model separately.

---

# 81. Hard rule.

---

# 82. Forecast Scenario

```rust
pub enum ForecastScenario {
    Baseline,
    HighGrowth,
    LowGrowth,
    ReleaseImpact,
    RegionFailure,
    ProviderFailure,
    DisasterRecovery,
    SecurityEvent,
}
```

---

# 83. Scenario Forecasting

Planning needs multiple futures.

---

# 84. No single forecast treated as certainty.

---

# 85. Hard rule.

---

# 86. Scenario Set

```rust
pub struct ScenarioForecastSet {
    pub baseline: CapacityForecast,
    pub scenarios: Vec<ScenarioCapacityForecast>,
}
```

---

# 87. Scenario Capacity Forecast

```rust
pub struct ScenarioCapacityForecast {
    pub scenario: ForecastScenario,
    pub forecast: CapacityForecast,
}
```

---

# 88. Good.

---

# 89. Saturation

Resource saturation occurs when forecast demand approaches usable capacity.

---

# 90. Saturation Threshold

```rust
pub struct SaturationThreshold {
    pub resource: ForecastResource,
    pub warning: FixedPoint,
    pub critical: FixedPoint,
}
```

---

# 91. Example

```text
warning 70%
critical 85%
```

---

# 92. Depends on resource.

---

# 93. Hard rule.

---

# 94. Saturation Horizon

```rust
pub struct SaturationHorizon {
    pub scope: ForecastScope,
    pub resource: ForecastResource,
    pub warning_at: Option<CoarseTimestamp>,
    pub critical_at: Option<CoarseTimestamp>,
    pub confidence: ForecastConfidence,
}
```

---

# 95. This Is A Forecast

Not guaranteed date.

---

# 96. Hard rule.

---

# 97. Earliest Bound Planning

For high-risk resource, plan using conservative upper forecast.

---

# 98. Hard rule.

---

# 99. Bottleneck Prediction

First resource to saturate.

---

# 100. Bottleneck Candidate

```rust
pub struct BottleneckCandidate {
    pub resource: ForecastResource,
    pub horizon: SaturationHorizon,
}
```

---

# 101. No One Overall "Capacity Score"

Hard rule.

---

# 102. Multiple Bottlenecks

Possible.

---

# 103. Hard rule.

---

# 104. Capacity Lead Time

Provisioning capacity takes time.

---

# 105. Lead Time

```rust
pub struct CapacityLeadTime {
    pub resource: ResourceClass,
    pub minimum: Duration,
    pub typical: Duration,
    pub worst_case: Duration,
}
```

---

# 106. Examples:

```text
cloud VM: minutes
database migration: days
hardware procurement: months
new region: months
```

---

# 107. Hard rule.

---

# 108. Planning Trigger

Order/provision before saturation horizon minus lead time.

---

# 109. Capacity Action Deadline

```rust
pub struct CapacityActionDeadline {
    pub resource: ForecastResource,
    pub act_by: CoarseTimestamp,
    pub reason: CapacityPlanningReason,
}
```

---

# 110. Hard rule.

---

# 111. Capacity Planning Reason

```rust
pub enum CapacityPlanningReason {
    Growth,
    SeasonalPeak,
    Release,
    FailoverReserve,
    ReliabilityRequirement,
    ResidencyConstraint,
}
```

---

# 112. Growth Plan

```rust
pub struct GrowthPlan {
    pub scope: ForecastScope,
    pub actions: Vec<CapacityPlanningAction>,
}
```

---

# 113. Planning Actions

```rust
pub enum CapacityPlanningAction {
    AddInstances,
    IncreaseDatabaseCapacity,
    AddStorage,
    IncreaseRelayCapacity,
    AddRegionCapacity,
    PurchaseHardware,
    PurchaseCommitment,
    OptimizeWorkload,
}
```

---

# 114. Proposal Only

Execution via Part 107.

---

# 115. Hard rule.

---

# 116. Headroom Planning

Part 101.

---

# 117. Forecast must preserve:

```text
normal headroom
critical reserve
failover reserve
```

---

# 118. Hard rule.

---

# 119. Headroom Requirement

```rust
pub struct ForecastHeadroomPolicy {
    pub normal_minimum: FixedPoint,
    pub failover_minimum: FixedPoint,
    pub uncertainty_reserve: FixedPoint,
}
```

---

# 120. Uncertainty Reserve

Extra capacity for forecast error.

---

# 121. Hard rule.

---

# 122. Forecast Uncertainty & Headroom

Lower-confidence forecast => larger reserve.

---

# 123. Hard rule.

---

# 124. Failure-Aware Forecast

Part 110.

---

# 125. Capacity forecast must include post-failure scenario.

---

# 126. Example:

```text
Region A fails
→ Region B+C absorb demand
```

---

# 127. Hard rule.

---

# 128. N-1 Forecast

```rust
pub struct FailureAdjustedForecast {
    pub failure_scenario: FailureScenarioClass,
    pub resulting_forecast: CapacityForecast,
}
```

---

# 129. Critical systems may need N-2.

---

# 130. Hard rule.

---

# 131. Regional Demand

Forecast per region.

---

# 132. Global aggregate can hide regional saturation.

---

# 133. Hard rule.

---

# 134. Residency-Aware Demand

Part 103.

---

# 135. Demand cannot automatically spill across forbidden jurisdictions.

---

# 136. Forecast must model constrained region capacity.

---

# 137. Hard rule.

---

# 138. Tenant Aggregate Forecast

Managed tenant capacity.

---

# 139. Use aggregated tenant technical usage.

---

# 140. No individual user profile.

---

# 141. Hard rule.

---

# 142. Federation Demand

Peer-level aggregate.

---

# 143. Forecast:

```text
ingress
egress
queue
storage
```

---

# 144. No remote user demand.

---

# 145. Hard rule.

---

# 146. Relay Demand

Forecast:

```text
connections
bandwidth
packet rate
```

---

# 147. No user-flow tracking.

---

# 148. Hard rule.

---

# 149. Mixnet Demand

Forecast:

```text
packet cells
cover traffic
mix processing
directory load
```

---

# 150. Important:

cover traffic is policy demand, not organic user demand.

---

# 151. Hard rule.

---

# 152. Cover Traffic Capacity

Forecast includes minimum privacy floor.

---

# 153. Cannot optimize it away.

---

# 154. Hard rule.

---

# 155. Anonymous Mailbox Demand

Forecast:

```text
ciphertext storage
fetch volume
delivery queue
```

---

# 156. No mailbox-owner identity.

---

# 157. Hard rule.

---

# 158. Realtime Media Demand

Forecast:

```text
concurrent sessions
relay bandwidth
codec CPU
```

---

# 159. Aggregate only.

---

# 160. Hard rule.

---

# 161. Database Demand

Forecast dimensions:

```text
connections
transactions
IOPS
CPU
storage
WAL
```

---

# 162. Need multiple resource model.

---

# 163. Hard rule.

---

# 164. Storage Growth

Forecast bytes over time.

---

# 165. Separate:

```text
authoritative data
attachments
backups
logs
cache
```

---

# 166. Different retention.

---

# 167. Hard rule.

---

# 168. Storage Saturation

Critical because cleanup may take time.

---

# 169. Early warning.

---

# 170. Hard rule.

---

# 171. Object Storage Demand

Forecast:

```text
stored bytes
PUT rate
GET rate
egress
```

---

# 172. Good.

---

# 173. Queue Demand

Forecast:

```text
arrival rate
service rate
backlog
age
```

---

# 174. Queueing Dynamics

Important.

---

# 175. Hard rule.

---

# 176. Arrival vs Service Rate

If:

```text
arrival > service
```

backlog grows.

---

# 177. Forecast saturation by queue age/depth.

---

# 178. Hard rule.

---

# 179. Queue Forecast

```rust
pub struct QueueDemandForecast {
    pub arrival_rate: FixedPoint,
    pub service_rate: FixedPoint,
    pub projected_depth: Vec<ForecastPoint>,
}
```

---

# 180. Good.

---

# 181. Connection Demand

Forecast concurrent connections, not just request rate.

---

# 182. Important for relays/DBs.

---

# 183. Hard rule.

---

# 184. Crypto Demand

Forecast expensive operations:

```text
handshakes
signature verify
key generation
```

---

# 185. Security reserves maintained.

---

# 186. Hard rule.

---

# 187. Demand Elasticity

Some demand can be delayed.

---

# 188. Elasticity Class

```rust
pub enum DemandElasticity {
    Immediate,
    ShortDeferrable,
    BackgroundDeferrable,
    Batch,
}
```

---

# 189. Useful for capacity plan.

---

# 190. Hard rule.

---

# 191. Critical Demand

Security/emergency/control-plane demand should have reserve independent of forecast average.

---

# 192. Hard rule.

---

# 193. Demand Shock

Unexpected spike.

---

# 194. Forecast architecture does not replace autoscaling.

---

# 195. Part 101 handles real-time.

---

# 196. Hard rule.

---

# 197. Forecast vs Autoscaling

```text
forecast = plan future capacity
autoscaling = react to current demand
```

---

# 198. Hard separation.

---

# 199. Anomaly Detection

Unexpected deviation from forecast.

---

# 200. Forecast Residual

```rust
pub struct ForecastResidual {
    pub expected: FixedPoint,
    pub observed: FixedPoint,
    pub deviation: FixedPoint,
}
```

---

# 201. Deviation Is Not Automatically Abuse

Hard rule.

---

# 202. Anomaly Classes

```rust
pub enum DemandAnomalyClass {
    UnexpectedGrowth,
    SuddenSpike,
    SuddenDrop,
    StructuralShift,
    MeasurementFault,
}
```

---

# 203. No user anomaly classification.

---

# 204. Hard rule.

---

# 205. Structural Break

Demand behavior changes permanently.

---

# 206. Model must adapt.

---

# 207. Hard rule.

---

# 208. Forecast Drift

Model accuracy deteriorates.

---

# 209. Forecast Drift State

```rust
pub enum ForecastDriftState {
    Stable,
    Warning,
    RetrainRequired,
    Invalid,
}
```

---

# 210. No stale model indefinitely.

---

# 211. Hard rule.

---

# 212. Retraining

Use aggregate technical data.

---

# 213. Model version increments.

---

# 214. Hard rule.

---

# 215. Forecast Governance

Models need ownership.

---

# 216. Forecast Owner

Usually capacity/service owner.

---

# 217. Hard rule.

---

# 218. Forecast Policy

```rust
pub struct ForecastPolicy {
    pub scope: ForecastScope,
    pub minimum_history: Duration,
    pub review_cadence: Duration,
    pub confidence_requirement: ForecastConfidence,
}
```

---

# 219. No one model for everything.

---

# 220. Hard rule.

---

# 221. Planning Cadence

Examples:

```text
hourly operational short horizon
weekly capacity review
monthly/quarterly procurement
```

---

# 222. Hard rule.

---

# 223. Demand Model Registry

Versioned.

---

# 224. Model Registry Entry

```rust
pub struct DemandModelRecord {
    pub model: ForecastModelVersion,
    pub scope: ForecastScope,
    pub resource: ForecastResource,
    pub training_window: TimeRange,
    pub backtest: ForecastError,
}
```

---

# 225. No user feature metadata.

---

# 226. Hard rule.

---

# 227. Model Promotion

Candidate model must outperform baseline sufficiently.

---

# 228. No model complexity for its own sake.

---

# 229. Hard rule.

---

# 230. Baseline Model

Always keep simple baseline.

---

# 231. Example seasonal naive.

---

# 232. Hard rule.

---

# 233. Forecast Comparison

```rust
pub struct ModelComparison {
    pub baseline: ForecastError,
    pub candidate: ForecastError,
}
```

---

# 234. Good.

---

# 235. Forecast Confidence Calibration

Predicted intervals should match reality.

---

# 236. Measure coverage.

---

# 237. Hard rule.

---

# 238. Scenario Assumptions

Explicit.

---

# 239. Scenario Assumption

```rust
pub struct ScenarioAssumption {
    pub key: ScenarioAssumptionKey,
    pub value: FixedPoint,
    pub source: AssumptionSource,
}
```

---

# 240. No hidden multiplier.

---

# 241. Hard rule.

---

# 242. Business Growth Input

Allowed at aggregate level.

---

# 243. Example:

```text
expected school/customer onboarding
new region launch
```

---

# 244. Convert into infrastructure scenario.

---

# 245. Do not forecast specific users.

---

# 246. Hard rule.

---

# 247. Release Forecast

Part 108.

---

# 248. Release may increase demand.

---

# 249. Use benchmark/load-test evidence.

---

# 250. Hard rule.

---

# 251. Change Integration

Part 107.

---

# 252. Large capacity plan becomes governed change.

---

# 253. Hard rule.

---

# 254. Resilience Integration

Part 110.

---

# 255. Forecast must include failure scenarios.

---

# 256. Hard rule.

---

# 257. SLO Integration

Part 109.

---

# 258. Plan capacity to meet latency/availability SLO.

---

# 259. Not just avoid 100% utilization.

---

# 260. Hard rule.

---

# 261. Inventory Integration

Part 106.

---

# 262. Forecast scope uses actual asset/resource pools.

---

# 263. Hard rule.

---

# 264. Desired-State Integration

Part 105.

---

# 265. Planned capacity changes become desired state.

---

# 266. Hard rule.

---

# 267. Organizational Governance Integration

Part 104.

---

# 268. Capacity plan has owner/approval.

---

# 269. Hard rule.

---

# 270. Geographic Governance Integration

Part 103.

---

# 271. Forecast respects regional placement constraints.

---

# 272. Hard rule.

---

# 273. FinOps Integration

Part 102.

---

# 274. Capacity forecast informs:

```text
budget
commitment
provider spend
```

---

# 275. Cost does not override resilience/privacy.

---

# 276. Hard rule.

---

# 277. DR Integration

Part 100.

---

# 278. Forecast includes recovery/failover load.

---

# 279. Hard rule.

---

# 280. Analytics Boundary

Part 92.

---

# 281. Infrastructure forecasting is operational analytics.

---

# 282. Keep separate from product/user analytics.

---

# 283. Hard rule.

---

# 284. Experimentation Boundary

Part 93.

---

# 285. A/B user experimentation not needed.

---

# 286. Forecast model evaluation uses historical backtests.

---

# 287. Hard rule.

---

# 288. Demand Forecast Service

```rust
pub trait DemandForecastService {
    fn forecast(
        &self,
        scope: ForecastScope,
        resource: ForecastResource,
        horizon: ForecastHorizon,
    ) -> Result<CapacityForecast, ForecastErrorType>;
}
```

---

# 289. Scenario Forecast Service

```rust
pub trait ScenarioForecastService {
    fn forecast_scenario(
        &self,
        scope: ForecastScope,
        resource: ForecastResource,
        scenario: ForecastScenario,
        horizon: ForecastHorizon,
    ) -> Result<CapacityForecast, ForecastErrorType>;
}
```

---

# 290. Saturation Service

```rust
pub trait SaturationPredictionService {
    fn horizon(
        &self,
        forecast: &CapacityForecast,
        capacity: &CapacityDenominator,
        threshold: &SaturationThreshold,
    ) -> Result<SaturationHorizon, ForecastErrorType>;
}
```

---

# 291. Growth Planner

```rust
pub trait GrowthPlanningService {
    fn plan(
        &self,
        forecasts: &[CapacityForecast],
        lead_times: &[CapacityLeadTime],
        policy: &ForecastHeadroomPolicy,
    ) -> Result<GrowthPlan, ForecastErrorType>;
}
```

---

# 292. No Direct Execution

Hard rule.

---

# 293. Forecast Error Taxonomy

```rust
pub enum ForecastErrorType {
    InsufficientHistory,
    TelemetryMissing,
    ModelInvalid,
    ModelDrift,
    CapacityUnknown,
    ConfidenceTooLow,
    ScenarioInvalid,
    ScopeViolation,
    Unauthorized,
    Internal,
}
```

---

# 294. Missing Data

No fabricated forecast.

---

# 295. Hard rule.

---

# 296. Cold Start

New service lacks history.

---

# 297. Use:

```text
benchmark model
analogous technical workload
conservative scenario
```

---

# 298. Label Low confidence.

---

# 299. Hard rule.

---

# 300. No Cross-User Analogies

Hard rule.

---

# 301. Capacity Forecast Evidence

```rust
pub struct ForecastEvidence {
    pub model: ForecastModelVersion,
    pub input_window: TimeRange,
    pub backtest_error: ForecastError,
    pub assumptions: Vec<ScenarioAssumption>,
}
```

---

# 302. Planning decision stores evidence digest.

---

# 303. Hard rule.

---

# 304. Forecast Explainability

Output should explain:

```text
trend
seasonality
event assumptions
confidence
saturation resource
```

---

# 305. No black-box number only.

---

# 306. Hard rule.

---

# 307. Forecast Visualization

UI can show:

```text
expected line
confidence band
capacity line
warning/critical line
```

---

# 308. No user cohort chart.

---

# 309. Hard rule.

---

# 310. Alerting

Forecast alerts are planning alerts, not incident alerts.

---

# 311. Forecast Alert

```rust
pub enum ForecastAlert {
    SaturationApproaching,
    LeadTimeRisk,
    ConfidenceLow,
    ModelDrift,
    ReserveInsufficient,
}
```

---

# 312. Example

```text
storage critical horizon < procurement lead time
```

---

# 313. Important.

---

# 314. Hard rule.

---

# 315. Alert Routing

Service/capacity owner.

---

# 316. No user notification.

---

# 317. Hard rule.

---

# 318. Planning State

```rust
pub enum CapacityPlanningState {
    Healthy,
    ReviewNeeded,
    ActionRequired,
    LeadTimeCritical,
}
```

---

# 319. LeadTimeCritical

Capacity action should already be underway.

---

# 320. Hard rule.

---

# 321. Forecast SLOs

Meta SLO examples:

```text
daily forecasts refreshed
backtest error under threshold
capacity denominators fresh
```

---

# 322. No recursive complexity.

---

# 323. Hard rule.

---

# 324. Privacy SLO

```text
0 stable user IDs in forecast inputs
0 private-content features in demand models
0 behavioral prediction exported into capacity planning
```

---

# 325. Security SLO

```text
0 forecast-based plan allowed to remove protected security capacity
0 failure reserve ignored in critical-service forecast
```

---

# 326. Failure Modes

```text
telemetry outage
model overfit
structural break
capacity inventory stale
unexpected demand shock
```

---

# 327. Telemetry Outage

Forecast confidence drops or becomes unavailable.

---

# 328. Do not extrapolate indefinitely.

---

# 329. Hard rule.

---

# 330. Model Overfit

Backtest/holdout detection.

---

# 331. Fall back simpler model.

---

# 332. Hard rule.

---

# 333. Structural Break

Retrain/review.

---

# 334. Do not force old seasonality.

---

# 335. Hard rule.

---

# 336. Stale Capacity Inventory

Saturation horizon unreliable.

---

# 337. Mark Unknown/low confidence.

---

# 338. Hard rule.

---

# 339. Demand Shock

Autoscaling/load shedding handles immediate event.

---

# 340. Forecast model updated after classification.

---

# 341. Hard rule.

---

# 342. Testing

Need forecast testkit.

---

# 343. Test Scenarios

```text
steady growth
daily seasonality
weekly seasonality
release step change
regional failover
```

---

# 344. Baseline Test

Seasonal naive baseline works.

---

# 345. Trend Test

Linear growth horizon correct.

---

# 346. Confidence Test

Longer horizon widens uncertainty.

---

# 347. Backtest Test

Candidate measured on holdout.

---

# 348. Bias Test

Persistent underforecast detected.

---

# 349. Saturation Test

Warning/critical horizon correct.

---

# 350. Lead-Time Test

Action deadline accounts for provisioning time.

---

# 351. Failure Scenario Test

N-1 demand modeled against remaining capacity.

---

# 352. Residency Test

Demand cannot spill to forbidden region.

---

# 353. Privacy Test

User ID/content fields rejected from model input.

---

# 354. Tenant Test

Tenant A forecast isolated.

---

# 355. Federation Test

Peer forecast has no remote user dimensions.

---

# 356. Mixnet Test

Cover traffic included as policy demand.

---

# 357. Queue Test

Arrival > service predicts backlog growth.

---

# 358. Storage Test

Retention policy affects growth forecast.

---

# 359. FinOps Test

Commitment plan uses forecast confidence.

---

# 360. Forecast Drift Test

Accuracy deterioration marks RetrainRequired.

---

# 361. Cold Start Test

Low-confidence conservative estimate used.

---

# 362. Fuzzing

Fuzz:

```text
forecast policy
scenario assumptions
capacity denominators
saturation thresholds
model registry records
```

---

# 363. Property Tests

Properties:

```text
forecast inputs can never contain stable global user identity
critical capacity plan can never ignore configured failure reserve
low-confidence/unknown capacity can never produce falsely certain saturation date
cost optimization can never move planned capacity into a forbidden residency region
```

---

# 364. Formal Verification Targets

Strong candidates:

```text
saturation-horizon calculation
lead-time action deadline
headroom/reserve policy
scenario constraint intersection
```

---

# 365. Kani Candidate

capacity/headroom/saturation arithmetic invariants.

---

# 366. TLA+ Candidate

forecast → action deadline → provision → capacity available before saturation.

---

# 367. Loom Candidate

concurrent telemetry update + model refresh + planning decision.

---

# 368. Performance

Forecasting runs off request path.

---

# 369. Batch aggregate time series.

---

# 370. No raw event processing needed.

---

# 371. Hard rule.

---

# 372. Incremental Updates

Update model state with new aggregate intervals.

---

# 373. Avoid full-history recomputation.

---

# 374. Hard rule.

---

# 375. Retention

High-resolution recent.

---

# 376. Older data rolled up.

---

# 377. Hard rule.

---

# 378. Example Retention

```text
5-minute: 30 days
hourly: 1 year
daily: multi-year if needed
```

---

# 379. Aggregate only.

---

# 380. Hard rule.

---

# 381. Storage

Separate:

```text
aggregate demand series
capacity denominators
forecast models
backtest results
scenario assumptions
growth plans
```

---

# 382. No user-event warehouse.

---

# 383. Hard rule.

---

# 384. Partitioning

By:

```text
service
region
resource pool
resource dimension
tenant aggregate
federation peer
work class
```

---

# 385. No user partition.

---

# 386. Hard rule.

---

# 387. Crate Layout

Recommended:

```text
crates/
├── siar-forecast-core/
├── siar-demand-series/
├── siar-demand-normalization/
├── siar-forecast-model/
├── siar-forecast-backtest/
├── siar-scenario-forecast/
├── siar-saturation-prediction/
├── siar-growth-planning/
├── siar-forecast-governance/
├── siar-forecast-observability/
└── siar-forecast-testkit/
```

---

# 388. `siar-forecast-core`

Owns:

```text
ForecastScope
ForecastResource
ForecastHorizon
errors
```

---

# 389. `siar-demand-series`

Aggregate time-series storage/rollup.

---

# 390. `siar-demand-normalization`

Provider/service metrics → internal units.

---

# 391. `siar-forecast-model`

Baseline/trend/seasonal forecast implementations.

---

# 392. `siar-forecast-backtest`

holdout evaluation/error/bias/calibration.

---

# 393. `siar-scenario-forecast`

release/failure/high-growth scenario planning.

---

# 394. `siar-saturation-prediction`

warning/critical saturation horizons.

---

# 395. `siar-growth-planning`

lead-time/headroom/reserve action planning.

---

# 396. `siar-forecast-governance`

model lifecycle/ownership/confidence/policy.

---

# 397. `siar-forecast-observability`

forecast-pipeline health only.

---

# 398. `siar-forecast-testkit`

trend/seasonality/saturation/privacy tests.

---

# 399. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. Capacity forecasting uses aggregate technical resource and workload signals by service, region, resource pool, tenant aggregate, federation peer, or work class; stable user identity and private behavioral histories are not valid forecast dimensions.
2. Forecast demand, observed demand, usable capacity, headroom, failover reserve, technical quota, and financial budget are distinct quantities and cannot be substituted for one another.
3. Every material forecast exposes horizon, model version, confidence/uncertainty, assumptions, and backtest error; a single predicted value is never presented as certain future demand.
4. Forecast models are evaluated out-of-sample against simple baselines, monitored for bias/drift, and replaced or downgraded when accuracy deteriorates.
5. Saturation prediction uses usable post-reserve capacity rather than nominal installed capacity and plans against conservative forecast bounds for critical resources.
6. Growth planning includes provisioning/procurement lead time, uncertainty reserve, normal headroom, security reserve, and failure/failover capacity; a plan that arrives after predicted saturation is not considered adequate.
7. Regional, tenant, and federation demand remain scope-isolated; a global aggregate cannot hide a constrained region or authorize capacity movement across prohibited residency boundaries.
8. Reliability/failure scenarios such as N-1/N-2, regional failover, and provider loss are first-class forecasts for critical services rather than afterthoughts to normal-demand planning.
9. Mixnet cover traffic, security/control-plane demand, and other policy-required load are modeled as protected demand and cannot be optimized away merely because organic traffic is lower.
10. Forecast anomalies represent deviations in aggregate infrastructure demand and cannot be interpreted automatically as malicious user behavior or converted into user-level profiles.
11. Forecasting, model registries, dashboards, evidence, and reports contain no private message content, contact graph, precise user activity history, or employee-performance telemetry.
12. Capacity forecasting integrates with autoscaling, resilience, SLOs, DR, FinOps, residency, change/release governance, desired state, inventory, and organizational ownership without creating an alternate path around security/privacy/reliability constraints.
```

---

# 400. Initial Production Scope

Implement first:

```text
typed ForecastScope/ForecastResource
aggregate resource-demand series
provider/service metric normalization
usable capacity denominators
baseline median/p95/peak model
seasonal-naive baseline
linear/piecewise trend model
daily/weekly seasonality
forecast confidence intervals
backtesting/error/bias
forecast model versioning
baseline/high-growth/release/failure scenarios
saturation warning/critical horizon
capacity lead-time model
headroom/uncertainty/failover reserve planning
regional forecasts
queue/storage/DB/relay forecast adapters
forecast drift detection
privacy-safe forecast dashboards
forecast testkit
```

Then add:

```text
robust Holt-Winters-like forecasting
probabilistic scenario simulation
capacity commitment optimization
multi-provider supply/lead-time modeling
formal saturation/lead-time verification
automated forecast-model recommendation constrained by explainability
```

---

# 401. Definition of Done

Part 111 is complete when:

- forecasts use aggregate technical demand only
- observed demand and usable capacity are typed separately
- seasonal/trend models and simple baselines exist
- forecast confidence/error/backtesting are explicit
- saturation horizons use usable capacity
- action deadlines include procurement lead times
- uncertainty/failover/security reserves are preserved
- regional and residency constraints are modeled
- N-1/N-2 scenarios exist for critical services
- cover/security/control demand cannot be optimized away
- forecast drift triggers review/retraining
- no user behavioral prediction exists
- forecast/saturation/privacy/fuzz/formal tests are specified

---

# 402. Final Architecture

```text
                AGGREGATE RESOURCE SIGNALS
                           │
                           ▼
                    DEMAND SERIES
                           │
              ┌────────────┼────────────┐
              │            │            │
            TREND       SEASONALITY    EVENTS
              │            │            │
              └────────────┼────────────┘
                           ▼
                      FORECASTS
                           │
             ┌─────────────┼─────────────┐
             │             │             │
          BASELINE      FAILURE       GROWTH
             │             │             │
             └─────────────┼─────────────┘
                           ▼
                  SATURATION PREDICTION
                           │
                           ▼
                LEAD-TIME / RESERVE PLAN
                           │
                           ▼
                  GOVERNED CAPACITY CHANGE
```

Forecast safety model:

```text
aggregate demand
+
simple explainable baselines
+
trend/seasonal models
+
confidence bounds
+
backtesting
+
failure scenarios
+
lead-time-aware planning
+
privacy/security/reliability reserves
```

not:

```text
track every user's behavior, fit an opaque model, output one exact growth number, and buy capacity only when current utilization reaches 100%
```

---

# 403. Final Principle

Capacity forecasting should tell infrastructure operators when and where resources are likely to become constrained while remaining honest about uncertainty and ignorant of individual user behavior.

The correct model is:

```text
measure aggregate technical demand
+
separate trend from seasonality and incidents
+
forecast with uncertainty
+
backtest continuously
+
predict saturation against usable capacity
+
include lead time and failure reserve
+
plan regional growth within policy
+
never turn infrastructure forecasting into behavioral profiling
```

This architecture gives SIAR a privacy-preserving infrastructure-forecast foundation for capacity growth, saturation prediction, failure-aware planning, regional demand, procurement timing, forecast governance, and long-term resource planning while preserving the anonymity, local-first, least-authority, and anti-surveillance guarantees established across Parts 34–110.
