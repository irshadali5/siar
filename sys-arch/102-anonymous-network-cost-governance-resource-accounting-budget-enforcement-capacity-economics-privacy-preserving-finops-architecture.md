# Core System Architecture Part 102 — Anonymous Network Cost Governance, Resource Accounting, Budget Enforcement, Capacity Economics & Privacy-Preserving FinOps Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 102  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 44, 47, 49, 54, 63, 69, 74, 92, 95, 98–101

**Primary purpose:** define SIAR's cost-governance architecture for resource accounting, budget hierarchies, cost attribution, spend limits, capacity economics, cost-aware scheduling, forecasting, commitments, provider abstraction, showback/chargeback, anonymous-service accounting, tenant budgets, FinOps controls, and privacy-preserving financial observability without creating a user-level surveillance or behavioral billing system.

---

# 1. Purpose

Cloud and infrastructure cost is a systems concern, not merely an accounting concern.

Poor cost architecture causes:

```text
runaway autoscaling
unexpected egress bills
inefficient storage
hidden idle capacity
tenant subsidy leakage
unbounded background workloads
```

The governing principle is:

> **SIAR cost governance should measure resource consumption at infrastructure and contractual scopes, enforce explicit budgets, and influence scheduling without turning every user action into a billable behavioral event.**

---

# 2. Architectural Position

```text
Infrastructure / Service Usage
           │
           ▼
     Resource Accounting
           │
           ▼
      Cost Normalization
           │
           ▼
        Cost Ledger
           │
      ┌────┼─────┐
      │    │     │
   Budget Forecast Allocation
      │    │     │
      └────┼─────┘
           ▼
       Governance
           │
    ┌──────┼───────┐
    │      │       │
  Alert  Restrict  Optimize
```

---

# 3. Core Separation

Keep distinct:

```text
resource usage
technical quota
financial budget
billing charge
capacity reservation
provider invoice
```

---

# 4. Non-Goals

Part 102 does not create:

```text
per-message surveillance billing
per-contact cost profiles
behavioral monetization
ad-tech style user value scoring
privacy downgrade to reduce cost
```

---

# 5. Cost Domains

```rust
pub enum CostDomain {
    Compute,
    Memory,
    Storage,
    NetworkIngress,
    NetworkEgress,
    Database,
    ObjectStorage,
    Relay,
    Media,
    Backup,
    Observability,
    Security,
    ThirdPartyService,
}
```

---

# 6. Compute

CPU/GPU/instance/runtime costs.

---

# 7. Memory

Reserved/consumed RAM.

---

# 8. Storage

Persistent block/database/object capacity.

---

# 9. Network

Ingress/egress/transfer.

---

# 10. Database

Compute/storage/I/O/connection cost.

---

# 11. Relay

Relay fleet cost.

---

# 12. Media

TURN/media relay/transcoding cost.

---

# 13. Backup

Snapshot/archive/restore cost.

---

# 14. Security

HSM, KMS, scanning, security services.

---

# 15. Hard Rule

Cost domains are infrastructure categories, not user behavior categories.

---

# 16. Accounting Scope

```rust
pub enum CostScope {
    Service(ServiceId),
    Environment(EnvironmentClass),
    Region(RegionId),
    Tenant(TenantId),
    FederationPeer(FederationDomainId),
    Project(ProjectId),
    SharedPlatform,
}
```

---

# 17. No Global User Cost Scope

Hard rule.

---

# 18. Tenant Scope

Only where tenant contract requires attribution.

---

# 19. Federation Scope

Peer/service accounting only.

---

# 20. Resource Unit

```rust
pub enum ResourceUnit {
    CpuSecond,
    GiBSecond,
    GiBMonth,
    Request,
    ConnectionMinute,
    GiBIngress,
    GiBEgress,
    DbIoUnit,
    RelayGiB,
    MediaMinute,
}
```

---

# 21. No Message Content Needed

Hard rule.

---

# 22. Usage Record

```rust
pub struct ResourceUsage {
    pub scope: CostScope,
    pub domain: CostDomain,
    pub unit: ResourceUnit,
    pub quantity: FixedPoint,
    pub epoch: CoarseEpoch,
}
```

---

# 23. Coarse Time

Hourly/daily where possible.

---

# 24. No exact user-action timestamp.

---

# 25. Hard rule.

---

# 26. Resource Accounting

Should derive from:

```text
provider meters
service counters
storage inventory
database stats
network totals
```

---

# 27. No Product Event Pipeline

Hard rule.

---

# 28. Cost Normalization

Different providers expose different units.

---

# 29. Normalize into internal units.

---

# 30. Pricing Model

```rust
pub struct UnitPrice {
    pub domain: CostDomain,
    pub unit: ResourceUnit,
    pub price: Money,
    pub currency: Currency,
    pub effective_from: Timestamp,
}
```

---

# 31. Version Prices

Provider price changes.

---

# 32. Never retroactively reinterpret historical cost without explicit recalculation.

---

# 33. Hard rule.

---

# 34. Cost Calculation

```rust
pub struct CostLine {
    pub usage: ResourceUsage,
    pub unit_price: UnitPrice,
    pub estimated_cost: Money,
}
```

---

# 35. Estimated vs Invoiced

Separate.

---

# 36. Hard rule.

---

# 37. Cost State

```rust
pub enum CostState {
    Estimated,
    Settled,
    Adjusted,
}
```

---

# 38. Estimated

Operational estimate.

---

# 39. Settled

Provider invoice/contract finalized.

---

# 40. Adjusted

Credits/discount/reconciliation.

---

# 41. No Silent Mutation

Hard rule.

---

# 42. Cost Ledger

Append/reconcile.

---

# 43. Not a full accounting ERP.

---

# 44. Stores infrastructure cost facts.

---

# 45. Cost Ledger Entry

```rust
pub struct CostLedgerEntry {
    pub entry_id: CostEntryId,
    pub scope: CostScope,
    pub domain: CostDomain,
    pub amount: Money,
    pub state: CostState,
    pub epoch: CoarseEpoch,
}
```

---

# 46. No user identity.

---

# 47. Hard rule.

---

# 48. Budget Hierarchy

```text
organization
→ environment
→ service
→ region
→ tenant/project
```

---

# 49. Budget Node

```rust
pub struct BudgetNode {
    pub budget_id: BudgetId,
    pub scope: CostScope,
    pub period: BudgetPeriod,
    pub limit: Money,
}
```

---

# 50. Budget Period

```rust
pub enum BudgetPeriod {
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    Annual,
}
```

---

# 51. Budget ≠ Quota

Hard rule.

---

# 52. Budget

Financial limit.

---

# 53. Quota

Technical resource entitlement.

---

# 54. Good separation.

---

# 55. Budget Status

```rust
pub enum BudgetStatus {
    Healthy,
    Warning,
    Critical,
    Exhausted,
}
```

---

# 56. Thresholds

Example:

```text
70%
90%
100%
```

---

# 57. Configurable.

---

# 58. Hard rule.

---

# 59. Budget Evaluation

Forecast-aware.

---

# 60. Current spend alone insufficient.

---

# 61. Burn Rate

```rust
pub struct CostBurnRate {
    pub actual: Money,
    pub forecast_period_end: Money,
}
```

---

# 62. Burn Rate Uses Aggregate Time Series

---

# 63. No user-level prediction.

---

# 64. Hard rule.

---

# 65. Spend Guard

```rust
pub enum SpendGuardAction {
    Alert,
    RequireApproval,
    RestrictOptionalWork,
    PauseBackgroundWork,
    CapAutoscaling,
}
```

---

# 66. No `DisableSecurity`.

Hard rule.

---

# 67. No `DisablePrivacy`.

Hard rule.

---

# 68. Budget Enforcement

Applies first to:

```text
optional
bulk
background
analytics
noncritical replicas
```

---

# 69. Never to:

```text
security revocation
control plane
emergency
minimum anonymity/privacy floor
```

---

# 70. Hard rule.

---

# 71. Cost Governance Precedence

Recommended:

```text
security/privacy floor
> continuity minimum
> contractual critical service
> budget policy
> optimization
```

---

# 72. Hard rule.

---

# 73. Cost-Aware Scheduling

Can choose among equivalent resources.

---

# 74. Example:

```text
region/provider/instance class
```

---

# 75. Only if security/privacy/residency constraints equal.

---

# 76. Hard rule.

---

# 77. Placement Decision

```rust
pub struct PlacementCandidate {
    pub region: RegionId,
    pub provider: ProviderId,
    pub estimated_cost: Money,
    pub security_ok: bool,
    pub privacy_ok: bool,
    pub residency_ok: bool,
}
```

---

# 78. Filter Security/Privacy First

Then optimize cost.

---

# 79. Hard rule.

---

# 80. Cost-Aware Scheduler

```rust
pub trait CostAwareScheduler {
    fn select(
        &self,
        candidates: &[PlacementCandidate],
    ) -> Result<PlacementCandidate, FinOpsError>;
}
```

---

# 81. No cost-only optimization.

---

# 82. Hard rule.

---

# 83. Autoscaling Integration

Part 101.

---

# 84. Autoscaling policy can consider cost ceiling.

---

# 85. But cost ceiling cannot starve protected capacity.

---

# 86. Hard rule.

---

# 87. Cost-Constrained Autoscaling

```rust
pub struct CostConstrainedScalingPolicy {
    pub max_hourly_cost: Money,
    pub emergency_override: bool,
    pub protected_min_instances: u16,
}
```

---

# 88. Protected Min

Never scale below.

---

# 89. Hard rule.

---

# 90. Cost Anomaly Detection

Useful.

---

# 91. Inputs:

```text
aggregate daily spend
usage spike
unexpected egress
idle resource cost
```

---

# 92. No user behavior.

---

# 93. Hard rule.

---

# 94. Cost Anomaly

```rust
pub struct CostAnomaly {
    pub scope: CostScope,
    pub domain: CostDomain,
    pub expected: Money,
    pub observed: Money,
    pub confidence: AnomalyConfidence,
}
```

---

# 95. Explainable.

---

# 96. No black-box user anomaly score.

---

# 97. Hard rule.

---

# 98. Anomaly Classes

```rust
pub enum CostAnomalyClass {
    SpendSpike,
    EgressSpike,
    IdleWaste,
    ScalingLoop,
    StorageGrowth,
    ProviderPriceChange,
}
```

---

# 99. Spend Spike

May be traffic growth or attack.

---

# 100. Do not assume abuse.

---

# 101. Hard rule.

---

# 102. Cost Alerting

Notification to operators/finance.

---

# 103. No user-facing spam.

---

# 104. Hard rule.

---

# 105. Provider Abstraction

Avoid hard-coding one cloud.

---

# 106. Provider ID

```rust
pub struct ProviderId(pub String);
```

---

# 107. Provider Rate Card Adapter

```rust
pub trait ProviderRateCard {
    fn unit_prices(
        &self,
        epoch: CoarseEpoch,
    ) -> Result<Vec<UnitPrice>, FinOpsError>;
}
```

---

# 108. Provider Usage Adapter

```rust
pub trait ProviderUsageAdapter {
    fn usage(
        &self,
        period: UsagePeriod,
    ) -> Result<Vec<ResourceUsage>, FinOpsError>;
}
```

---

# 109. Normalize Into Internal Cost Model

Hard rule.

---

# 110. Provider Billing APIs Are Not Security Authority

Hard rule.

---

# 111. Invoice Reconciliation

Compare estimated vs provider invoice.

---

# 112. Reconciliation Entry

```rust
pub struct CostReconciliation {
    pub scope: CostScope,
    pub estimated: Money,
    pub invoiced: Money,
    pub adjustment: Money,
}
```

---

# 113. Differences visible.

---

# 114. No silent overwrite.

---

# 115. Hard rule.

---

# 116. Currency

Support explicit currency.

---

# 117. Internal budget currency chosen.

---

# 118. FX conversion separate.

---

# 119. No floating-point money.

---

# 120. Hard rule.

---

# 121. Money Type

```rust
pub struct Money {
    pub minor_units: i128,
    pub currency: Currency,
}
```

---

# 122. Fixed precision.

---

# 123. Good Rust type safety.

---

# 124. Showback

Display cost to service/tenant owner.

---

# 125. Chargeback

Actually bill/allocate.

---

# 126. Separate.

---

# 127. Hard rule.

---

# 128. Showback Record

```rust
pub struct CostAllocation {
    pub scope: CostScope,
    pub period: BudgetPeriod,
    pub direct_cost: Money,
    pub shared_cost: Money,
}
```

---

# 129. Shared Cost

Infrastructure shared across scopes.

---

# 130. Allocation Policy

Explicit.

---

# 131. No hidden arbitrary allocation.

---

# 132. Hard rule.

---

# 133. Shared Cost Allocation Methods

Possible:

```text
equal
resource share
contract weight
fixed reservation
```

---

# 134. No user behavior weight.

---

# 135. Hard rule.

---

# 136. Tenant Billing

Only if product model requires.

---

# 137. Tenant cost attribution can use:

```text
storage bytes
service units
managed seats
bandwidth bucket
```

---

# 138. Not:

```text
message content
contact graph
private usage timeline
```

---

# 139. Hard rule.

---

# 140. Anonymous Service Accounting

Need no identity.

---

# 141. Can use:

```text
prepaid scoped tokens
resource credits
capability budgets
```

---

# 142. Part 47 integration.

---

# 143. No persistent anonymous-user account required.

---

# 144. Hard rule.

---

# 145. Resource Credit

```rust
pub struct ResourceCredit {
    pub credit_id: CreditId,
    pub scope: CreditScope,
    pub units: ResourceUnits,
    pub expires_at: Timestamp,
}
```

---

# 146. Credit Scope

```rust
pub enum CreditScope {
    Relay,
    Mailbox,
    Storage,
    Compute,
    Media,
}
```

---

# 147. No social identity.

---

# 148. Hard rule.

---

# 149. Resource Credit Spend

Exact one-time or decrementable secure balance.

---

# 150. Privacy architecture may use anonymous token mechanisms.

---

# 151. No custom crypto baseline.

---

# 152. Hard rule.

---

# 153. Financial Accounting Boundary

FinOps ledger is not general accounting ledger.

---

# 154. It tracks infrastructure/resource cost.

---

# 155. Payment settlement handled separately.

---

# 156. Hard rule.

---

# 157. Capacity Economics

Need relationship between:

```text
reserved capacity
on-demand capacity
spot/preemptible
egress
storage
```

---

# 158. Capacity Purchase Class

```rust
pub enum CapacityPurchaseClass {
    OnDemand,
    Reserved,
    CommittedUse,
    Spot,
    Owned,
}
```

---

# 159. Critical control-plane baseline

prefer predictable capacity.

---

# 160. Background workloads

spot/preemptible acceptable.

---

# 161. Hard rule.

---

# 162. Commitment Planning

Use aggregate forecast.

---

# 163. Avoid overcommit.

---

# 164. Commitment Decision

```rust
pub struct CapacityCommitmentPlan {
    pub resource_class: ResourceClass,
    pub committed_units: u64,
    pub term: CommitmentTerm,
}
```

---

# 165. Term

```rust
pub enum CommitmentTerm {
    Monthly,
    Annual,
    MultiYear,
}
```

---

# 166. No irreversible commitment without forecast/confidence.

---

# 167. Hard rule.

---

# 168. Reserved Capacity

Useful for base load.

---

# 169. Keep burst on-demand.

---

# 170. Hard rule.

---

# 171. Spot Capacity

Only for interruptible jobs.

---

# 172. Examples:

```text
analytics
reindex
batch conversion
noncritical CI
```

---

# 173. Not:

```text
identity
critical relay
quorum voter
HSM gateway
```

---

# 174. Hard rule.

---

# 175. Network Egress Economics

Often expensive.

---

# 176. Optimize through:

```text
regional locality
compression
caching
P2P distribution
delta updates
```

---

# 177. Never through:

```text
privacy downgrade
cross-user content dedup if unsafe
```

---

# 178. Hard rule.

---

# 179. Storage Economics

Classify:

```text
hot
warm
cold
archive
rebuildable
```

---

# 180. Storage Tier

```rust
pub enum StorageTier {
    Hot,
    Warm,
    Cold,
    Archive,
}
```

---

# 181. Move only if latency/retention/security allow.

---

# 182. Hard rule.

---

# 183. Rebuildable Data

Can use cheaper storage or no backup.

---

# 184. Authoritative data

stronger durability.

---

# 185. Hard rule.

---

# 186. Backup Economics

Retention costs.

---

# 187. Optimize:

```text
incremental
compression
retention rotation
tiering
```

---

# 188. Never drop deletion ledger/security recovery state.

---

# 189. Hard rule.

---

# 190. Observability Cost

Metrics/logs can be expensive.

---

# 191. Control through:

```text
sampling
rollups
short retention
cardinality limits
```

---

# 192. Part 92.

---

# 193. No user-level telemetry to justify cost.

---

# 194. Hard rule.

---

# 195. Security Cost

Security controls may cost more.

---

# 196. Examples:

```text
HSM
attestation
redundant signing
secure logging
```

---

# 197. Cost must not remove security control below minimum.

---

# 198. Hard rule.

---

# 199. Privacy Cost

Anonymity/mixnet/cover traffic has real cost.

---

# 200. Model explicitly.

---

# 201. Privacy Cost Domain

```rust
pub enum PrivacyCostClass {
    CoverTraffic,
    MixRelay,
    PrivateMailbox,
    AnonymousRouting,
}
```

---

# 202. Cost Governance Must Preserve Minimum Privacy Budget

Hard rule.

---

# 203. Privacy Reserve

```rust
pub struct PrivacyCapacityReserve {
    pub class: PrivacyCostClass,
    pub minimum_budget: Money,
}
```

---

# 204. Cannot be cut to zero for savings.

---

# 205. Hard rule.

---

# 206. Emergency Cost Override

During incident/DR.

---

# 207. Security/continuity may temporarily exceed budget.

---

# 208. Audited/visible.

---

# 209. No hidden uncontrolled override.

---

# 210. Hard rule.

---

# 211. Emergency Spend Policy

```rust
pub struct EmergencySpendPolicy {
    pub max_multiplier: FixedPoint,
    pub max_duration: Duration,
    pub approval_required: bool,
}
```

---

# 212. Time-bound.

---

# 213. Hard rule.

---

# 214. Cost Governance State

```rust
pub enum CostGovernanceState {
    Normal,
    Warning,
    Restricted,
    EmergencyOverride,
}
```

---

# 215. Restricted

Optional/background cost controls active.

---

# 216. EmergencyOverride

Security/continuity spend cap temporarily raised.

---

# 217. No Permanent Emergency State

Hard rule.

---

# 218. Budget Enforcement Controller

```rust
pub trait BudgetEnforcementController {
    fn evaluate(
        &self,
        budget: &BudgetNode,
        spend: &CostBurnRate,
    ) -> Vec<SpendGuardAction>;
}
```

---

# 219. Cost Forecasting

Use:

```text
historical aggregate
known growth
commitment state
planned releases
```

---

# 220. No user behavior.

---

# 221. Hard rule.

---

# 222. Forecast Window

Daily/monthly/quarterly.

---

# 223. Confidence interval.

---

# 224. No false precision.

---

# 225. Hard rule.

---

# 226. Cost Forecast

```rust
pub struct CostForecast {
    pub scope: CostScope,
    pub period: BudgetPeriod,
    pub expected: Money,
    pub lower: Money,
    pub upper: Money,
}
```

---

# 227. Provider Price Shock

Need detection.

---

# 228. Update rate card.

---

# 229. Reforecast.

---

# 230. No silent overrun.

---

# 231. Hard rule.

---

# 232. Cost Allocation Policy

Versioned.

---

# 233. Allocation Policy ID

```rust
pub struct AllocationPolicyVersion(pub u64);
```

---

# 234. Historical report references exact policy.

---

# 235. Hard rule.

---

# 236. Cost Policy Anti-Rollback

Old permissive budgets should not reappear.

---

# 237. Hard rule.

---

# 238. Financial Governance Roles

```rust
pub enum FinOpsRole {
    FinOpsViewer,
    BudgetOwner,
    ServiceOwner,
    FinanceApprover,
    PlatformOperator,
}
```

---

# 239. No Universal Finance+Security Superuser

Hard rule.

---

# 240. Budget Change Authorization

Scoped.

---

# 241. High-value commitment requires approval.

---

# 242. Hard rule.

---

# 243. FinOps Capability

```rust
pub struct FinOpsCapability {
    pub role: FinOpsRole,
    pub scope: CostScope,
    pub actions: BTreeSet<FinOpsAction>,
    pub expires_at: Option<Timestamp>,
}
```

---

# 244. Actions

```rust
pub enum FinOpsAction {
    ViewCost,
    SetBudget,
    ApproveCommitment,
    ApproveEmergencySpend,
    ExportReport,
}
```

---

# 245. No production shell access implied.

---

# 246. Hard rule.

---

# 247. Cost Report

```rust
pub struct CostReport {
    pub period: BudgetPeriod,
    pub scope: CostScope,
    pub direct: Money,
    pub shared: Money,
    pub forecast: Option<CostForecast>,
}
```

---

# 248. No user-level drilldown.

---

# 249. Hard rule.

---

# 250. FinOps Dashboard

Shows:

```text
spend
budget
forecast
cost by domain
commitment utilization
idle waste
```

---

# 251. Not:

```text
which users cost most
who sends most messages
```

---

# 252. Hard rule.

---

# 253. Cost Attribution Precision

Do not overfit.

---

# 254. Shared anonymous infra

use aggregate allocation.

---

# 255. Hard rule.

---

# 256. Unit Economics

Useful at product/service level.

---

# 257. Examples:

```text
cost per managed tenant
cost per GiB relayed
cost per media minute
cost per active service instance
```

---

# 258. Avoid per-user lifetime value in infrastructure core.

---

# 259. Hard rule.

---

# 260. Unit Cost

```rust
pub struct UnitCost {
    pub domain: CostDomain,
    pub unit: ResourceUnit,
    pub cost: Money,
}
```

---

# 261. Trend Over Time

Aggregate.

---

# 262. Good.

---

# 263. Pricing Strategy Boundary

Product pricing separate from infrastructure cost.

---

# 264. No automatic customer price changes from raw cloud cost.

---

# 265. Hard rule.

---

# 266. Cost Optimization Candidates

```text
idle instance
overprovisioned DB
cold storage
excess log retention
underused reserved capacity
```

---

# 267. Optimization Recommendation

```rust
pub struct CostOptimizationRecommendation {
    pub scope: CostScope,
    pub action: OptimizationAction,
    pub estimated_savings: Money,
    pub risk_class: OptimizationRisk,
}
```

---

# 268. Optimization Action

```rust
pub enum OptimizationAction {
    Rightsize,
    ScaleDown,
    MoveStorageTier,
    ReduceRetention,
    PurchaseCommitment,
    ReleaseCommitment,
}
```

---

# 269. Security/Privacy Review

Required if recommendation affects critical system.

---

# 270. Hard rule.

---

# 271. No Auto-Apply High-Risk Optimization

Hard rule.

---

# 272. Low-Risk Auto Optimization

Possible:

```text
delete expired cache
remove idle test environment
```

---

# 273. Must be reversible or noncritical.

---

# 274. Hard rule.

---

# 275. Rightsizing

Based on aggregate utilization.

---

# 276. Keep headroom from Part 101.

---

# 277. No shrink below failure reserve.

---

# 278. Hard rule.

---

# 279. Idle Detection

Resource idle, not "user inactive."

---

# 280. Hard rule.

---

# 281. Dev/Test Environments

Auto-stop possible.

---

# 282. Production never auto-stop solely for cost.

---

# 283. Hard rule.

---

# 284. CI/CD Cost

Forge-like workloads may be expensive.

---

# 285. Separate budget scope.

---

# 286. Use:

```text
job quotas
spot workers
artifact cache
```

---

# 287. No effect on critical production.

---

# 288. Hard rule.

---

# 289. AI/Compute Cost

If optional AI exists.

---

# 290. Device-local preferred for privacy/cost where feasible.

---

# 291. Remote AI cost separately budgeted.

---

# 292. No private content upload merely to reduce local compute cost.

---

# 293. Hard rule.

---

# 294. Provider Lock-In Economics

Track exit cost.

---

# 295. Egress/managed-service dependency.

---

# 296. Architecture should expose provider-neutral cost model.

---

# 297. Hard rule.

---

# 298. Multi-Cloud Cost

Not automatically cheaper.

---

# 299. Use only for resilience/legal/performance where justified.

---

# 300. Hard rule.

---

# 301. Federation Economics

Peer settlement.

---

# 302. Aggregate service units.

---

# 303. No remote user details.

---

# 304. Hard rule.

---

# 305. Federation Cost Statement

```rust
pub struct FederationCostStatement {
    pub peer: FederationDomainId,
    pub period: BudgetPeriod,
    pub units: Vec<ResourceUsage>,
    pub amount: Money,
}
```

---

# 306. Signed.

---

# 307. No member/user identity.

---

# 308. Hard rule.

---

# 309. Cost Privacy

Provider billing data may expose topology.

---

# 310. Access restricted.

---

# 311. Do not expose detailed internal cost data publicly.

---

# 312. Hard rule.

---

# 313. Tenant Cost Privacy

Tenant A cannot see B spend/usage.

---

# 314. Shared-cost allocation doesn't reveal tenant names.

---

# 315. Hard rule.

---

# 316. Anonymous User Cost Privacy

No persistent "cost profile."

---

# 317. Hard rule.

---

# 318. Cost Telemetry

Safe:

```text
service cost
region cost
domain cost
budget status
```

---

# 319. Forbidden:

```text
user-level spend
contact-linked cost
message-linked cost
```

---

# 320. Hard rule.

---

# 321. Cost SLOs

Examples:

```text
budget evaluation freshness
provider invoice reconciliation
critical budget alert latency
```

---

# 322. Privacy SLO

```text
0 stable user IDs in FinOps data
0 private content in cost attribution
0 cost policy causing privacy-floor downgrade
```

---

# 323. Security SLO

```text
0 critical security capacity disabled by budget enforcement
0 cost optimizer auto-removes required resilience
```

---

# 324. Failure Modes

```text
provider billing API outage
wrong price sheet
cost spike
budget config error
forecast error
```

---

# 325. Billing API Outage

Continue local estimate.

---

# 326. Mark unsettled.

---

# 327. No freeze of production.

---

# 328. Hard rule.

---

# 329. Wrong Price Sheet

Recalculate estimates.

---

# 330. Preserve original reports/version.

---

# 331. Hard rule.

---

# 332. Cost Spike

Alert.

---

# 333. Restrict optional work.

---

# 334. Do not violate critical availability/privacy.

---

# 335. Hard rule.

---

# 336. Budget Config Error

Validation + approval.

---

# 337. Protect critical minimums.

---

# 338. Hard rule.

---

# 339. Forecast Error

Adjust forecast.

---

# 340. Do not treat forecast as actual spend.

---

# 341. Hard rule.

---

# 342. Testing

Need FinOps/cost-governance testkit.

---

# 343. Test Scenarios

```text
budget overrun
egress spike
provider price change
emergency DR spend
tenant cost allocation
```

---

# 344. Money Test

No floating point drift.

---

# 345. Budget Threshold Test

Correct Warning/Critical/Exhausted.

---

# 346. Priority Test

Budget cannot disable CriticalSecurity work.

---

# 347. Privacy Test

No user ID accepted in CostScope.

---

# 348. Tenant Isolation Test

Tenant A cannot query B report.

---

# 349. Shared Cost Test

Allocation policy deterministic.

---

# 350. Provider Reconciliation Test

Estimate/invoice adjustment explicit.

---

# 351. Emergency Spend Test

Override auto-expires.

---

# 352. Autoscaling Test

Cost cap never drops below protected min instances.

---

# 353. Commitment Test

Reserved capacity plan respects forecast confidence.

---

# 354. Storage Optimization Test

Authoritative data not tiered/deleted unsafely.

---

# 355. Privacy Reserve Test

Cover/mix minimum cannot be cut by budget controller.

---

# 356. Federation Test

Peer settlement contains no user data.

---

# 357. Backup Test

Old permissive budget policy not restored active.

---

# 358. Fuzzing

Fuzz:

```text
rate card
cost line
budget policy
allocation rule
federation cost statement
```

---

# 359. Property Tests

Properties:

```text
financial budget enforcement can never disable protected security/privacy capacity
CostScope can never encode a global user identity
estimated and settled cost can never silently overwrite one another
expired emergency-spend override can never remain active
```

---

# 360. Formal Verification Targets

Strong candidates:

```text
budget state machine
emergency override expiry
protected minimum capacity
shared cost allocation
```

---

# 361. Kani Candidate

budget/security precedence invariants.

---

# 362. TLA+ Candidate

normal budget → spike → restriction → emergency override → expiry → recovery.

---

# 363. Loom Candidate

concurrent usage ingest + budget evaluation + autoscaling decision.

---

# 364. Performance

Cost accounting is background.

---

# 365. Do not block critical request path.

---

# 366. Aggregate counters.

---

# 367. Batch provider reconciliation.

---

# 368. Hard rule.

---

# 369. Storage

Separate stores:

```text
usage aggregates
rate cards
cost ledger
budgets
allocation policies
commitments
forecasts
```

---

# 370. No user event warehouse.

---

# 371. Hard rule.

---

# 372. Partitioning

By:

```text
service
environment
region
tenant
federation peer
cost domain
```

---

# 373. No global user partition.

---

# 374. Hard rule.

---

# 375. Crate Layout

Recommended:

```text
crates/
├── siar-finops-core/
├── siar-resource-accounting/
├── siar-cost-normalization/
├── siar-cost-ledger/
├── siar-budget/
├── siar-cost-allocation/
├── siar-cost-forecast/
├── siar-cost-anomaly/
├── siar-capacity-economics/
├── siar-provider-billing/
├── siar-finops-observability/
└── siar-finops-testkit/
```

---

# 376. `siar-finops-core`

Owns:

```text
Money
Currency
CostDomain
CostScope
errors
```

---

# 377. `siar-resource-accounting`

Aggregate infrastructure usage.

---

# 378. `siar-cost-normalization`

Provider units → internal units.

---

# 379. `siar-cost-ledger`

Estimated/settled/adjusted cost facts.

---

# 380. `siar-budget`

Budget tree, burn rate, spend guards.

---

# 381. `siar-cost-allocation`

Showback/chargeback/shared-cost policy.

---

# 382. `siar-cost-forecast`

Aggregate spend forecasting.

---

# 383. `siar-cost-anomaly`

Spend/egress/idle/scaling anomaly detection.

---

# 384. `siar-capacity-economics`

reserved/on-demand/spot/commitment planning.

---

# 385. `siar-provider-billing`

provider invoice/rate-card adapters.

---

# 386. `siar-finops-observability`

FinOps pipeline health only.

---

# 387. `siar-finops-testkit`

budget/allocation/privacy/economics tests.

---

# 388. Error Taxonomy

```rust
pub enum FinOpsError {
    InvalidMoney,
    CurrencyMismatch,
    RateCardUnavailable,
    UsageUnavailable,
    BudgetNotFound,
    BudgetExceeded,
    AllocationInvalid,
    ForecastUnavailable,
    CommitmentDenied,
    Unauthorized,
    ScopeViolation,
    Internal,
}
```

---

# 389. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. FinOps operates on infrastructure, service, tenant, region, project, and federation scopes; global user identity is not a cost-accounting key.
2. Resource accounting uses aggregate technical units such as CPU, memory, storage, network, database, relay, and media usage—not private content or behavioral event histories.
3. Financial budgets and technical quotas are distinct: budget enforcement may restrict optional cost but cannot silently revoke required technical entitlement or security-critical capacity.
4. Security, privacy, anonymity, continuity, residency, and tenant-isolation requirements are evaluated before cost optimization or placement decisions.
5. Cost caps, optimization recommendations, and autoscaling constraints can never disable protected security/control/emergency capacity or minimum anonymity/cover-traffic floors.
6. Provider estimates, settled invoices, credits, and adjustments remain separate immutable financial states and cannot silently overwrite historical cost facts.
7. Anonymous-service accounting uses scoped resource credits/tokens or aggregate infrastructure units rather than durable user cost profiles.
8. Tenant and federation cost data remain scope-isolated; shared-cost allocation reveals no unrelated tenant or user activity.
9. Emergency spend overrides are signed/approved where required, time-bounded, auditable, and automatically expire.
10. Cost forecasting, anomaly detection, commitments, and rightsizing use aggregate operational data only and do not create behavioral user models.
11. FinOps dashboards and exports do not provide per-user spend, per-contact cost, private-message cost, or any social/behavioral drilldown.
12. Cost governance integrates with capacity management, DR, security, privacy, federation, billing, and provider infrastructure without becoming an alternate authorization, surveillance, or privacy-downgrade system.
```

---

# 390. Initial Production Scope

Implement first:

```text
typed Money/Currency
CostDomain/CostScope
aggregate ResourceUsage
provider rate-card adapters
estimated vs settled cost ledger
budget hierarchy
burn-rate/forecast basics
budget alerts
spend guards for optional/background work
protected security/privacy minimums
tenant/service showback
shared-cost allocation
provider invoice reconciliation
cost-aware placement after security/privacy filtering
cost-constrained autoscaling with protected minimums
commitment planning
privacy-safe cost dashboards
FinOps testkit
```

Then add:

```text
anonymous resource-credit accounting
multi-provider economic optimizer
advanced commitment portfolio planning
privacy-preserving federation settlement
automated low-risk cost optimization
formal budget/protected-capacity verification
```

---

# 391. Definition of Done

Part 102 is complete when:

- resource usage is normalized into typed infrastructure units
- cost scopes exclude global user identity
- estimates and invoices are separately represented
- budgets and technical quotas are distinct
- spend guards affect optional work before critical work
- security/privacy minimums are financially protected
- showback/chargeback policies are explicit
- tenant/federation cost visibility is isolated
- anonymous services can account without durable identity
- cost-aware placement filters security/privacy first
- forecasting/anomaly detection use aggregate data
- emergency spend overrides expire
- provider reconciliation is explicit
- budget/allocation/privacy/economics/fuzz/formal tests are specified

---

# 392. Final Architecture

```text
                 RESOURCE USAGE
                       │
                       ▼
               COST NORMALIZATION
                       │
                       ▼
                  COST LEDGER
                       │
            ┌──────────┼──────────┐
            │          │          │
         BUDGET     FORECAST   ALLOCATION
            │          │          │
            └──────────┼──────────┘
                       ▼
                FINOPS GOVERNANCE
                       │
          ┌────────────┼────────────┐
          │            │            │
       ALERT        RESTRICT      OPTIMIZE
          │            │            │
          └────────────┼────────────┘
                       ▼
             CAPACITY / PLACEMENT
```

FinOps safety model:

```text
aggregate resource accounting
+
explicit budgets
+
provider-neutral cost normalization
+
protected security/privacy minimums
+
scope-isolated allocation
+
forecasting and commitments
+
privacy-safe showback
```

not:

```text
attach a cost identity to every user action, optimize purely for cheapest execution, and disable resilience or anonymity when the bill gets high
```

---

# 393. Final Principle

Cost governance should make infrastructure economically sustainable without changing who the system protects or what privacy guarantees it provides.

The correct model is:

```text
measure resources
+
normalize costs
+
budget explicitly
+
protect critical capacity
+
optimize only inside security/privacy constraints
+
attribute by infrastructure scope
+
forecast conservatively
+
never build user cost profiles
```

This architecture gives SIAR a privacy-preserving FinOps foundation for infrastructure cost control, tenant/service showback, budget enforcement, cost-aware scaling, capacity commitments, provider reconciliation, anonymous resource accounting, and long-term sustainability while preserving the anonymity, local-first, least-authority, and anti-surveillance guarantees established across Parts 34–101.
