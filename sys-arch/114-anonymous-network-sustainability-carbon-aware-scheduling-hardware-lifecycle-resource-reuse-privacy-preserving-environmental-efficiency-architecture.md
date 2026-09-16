# Core System Architecture Part 114 — Anonymous Network Sustainability, Carbon-Aware Scheduling, Hardware Lifecycle, Resource Reuse & Privacy-Preserving Environmental Efficiency Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 114  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 62–63, 67, 73, 92, 100–113

**Primary purpose:** define SIAR's sustainability architecture for carbon-aware scheduling, environmental efficiency, hardware lifecycle, embodied-carbon accounting, resource reuse, refurbishment, e-waste reduction, sustainable procurement, renewable-energy windows, environmental capacity planning, and privacy-preserving ecological governance.

---

# 1. Purpose

Infrastructure consumes:

```text
electricity
hardware
storage media
network capacity
cooling
manufacturing resources
```

Sustainability engineering asks:

```text
Can this work run when electricity is cleaner?
Can hardware lifetime be extended safely?
Can idle resources be reused before new capacity is purchased?
Can e-waste be reduced without sacrificing reliability?
Can environmental optimization respect data residency and privacy?
```

The governing principle is:

> **SIAR may optimize infrastructure for environmental efficiency only after correctness, security, privacy, reliability, residency, and service-level requirements are satisfied.**

---

# 2. Architectural Position

```text
Operational Work / Capacity Need
             │
             ▼
      Sustainability Policy
             │
    ┌────────┼────────┐
    │        │        │
 Energy   Hardware   Carbon
    │        │        │
    └────────┼────────┘
             ▼
 Environmental Decision Engine
             │
    ┌────────┼────────┐
    │        │        │
 Schedule  Reuse   Procure
             │
             ▼
 Verification / Evidence
```

---

# 3. Core Separation

Keep distinct:

```text
energy efficiency
carbon intensity
embodied carbon
cost
capacity
performance
hardware lifetime
resource reuse
```

---

# 4. Non-Goals

Part 114 does not create:

```text
user-level carbon scores
behavioral nudging based on private activity
carbon optimization that weakens privacy
greenwashing metrics without provenance
mandatory workload migration across forbidden jurisdictions
```

---

# 5. Sustainability Scope

```rust
pub enum SustainabilityScope {
    Service(ServiceId),
    Region(RegionId),
    ResourcePool(ResourcePoolId),
    Asset(AssetId),
    HardwareClass(HardwareClass),
    WorkClass(WorkClass),
}
```

---

# 6. No User Scope

Hard rule.

---

# 7. Sustainability Dimension

```rust
pub enum SustainabilityDimension {
    OperationalEnergy,
    CarbonIntensity,
    EmbodiedCarbon,
    HardwareLifetime,
    ResourceReuse,
    CoolingEfficiency,
    EWaste,
}
```

---

# 8. Environmental Budget

```rust
pub struct EnvironmentalBudget {
    pub scope: SustainabilityScope,
    pub max_operational_energy: Option<FixedPoint>,
    pub max_carbon_intensity: Option<FixedPoint>,
    pub minimum_hardware_lifetime: Option<Duration>,
}
```

---

# 9. Budget Is Advisory Unless Policy Says Otherwise

Security/reliability always higher priority.

---

# 10. Policy Precedence

```text
correctness
> security/privacy
> residency
> reliability
> performance SLO
> sustainability
> cost
```

---

# 11. Hard rule.

---

# 12. Operational Energy

Measure infrastructure energy usage where practical.

```rust
pub struct OperationalEnergySample {
    pub scope: SustainabilityScope,
    pub watt_hours: FixedPoint,
    pub interval: TimeRange,
}
```

---

# 13. Aggregate Only

No user/device behavioral profile.

---

# 14. Hard rule.

---

# 15. Energy Source Metadata

```rust
pub enum EnergySourceClass {
    GridMixed,
    RenewableHigh,
    RenewableLow,
    Unknown,
}
```

---

# 16. Avoid False Precision

If source mix uncertain, mark Unknown.

---

# 17. Hard rule.

---

# 18. Carbon Intensity

Represent external/provider estimate.

```rust
pub struct CarbonIntensity {
    pub grams_co2e_per_kwh: FixedPoint,
    pub source: CarbonDataSource,
    pub observed_at: CoarseTimestamp,
}
```

---

# 19. Carbon Data Source

```rust
pub enum CarbonDataSource {
    ProviderPublished,
    GridOperator,
    IndependentDataset,
    ConservativeEstimate,
}
```

---

# 20. Hard Rule

Carbon data is evidence with provenance, not an unquestioned truth.

---

# 21. Carbon-Aware Scheduling

Only deferrable workloads can shift in time.

Examples:

```text
backup compaction
index rebuild
analytics rollup
nonurgent update prefetch
large background sync
```

---

# 22. Not Deferrable

```text
security revocation
critical control traffic
direct messaging
emergency traffic
incident recovery
```

---

# 23. Hard rule.

---

# 24. Deferrability

```rust
pub enum SustainabilityDeferrability {
    Immediate,
    ShortWindow(Duration),
    FlexibleWindow(Duration),
    BatchWindow(Duration),
}
```

---

# 25. Carbon-Aware Job

```rust
pub struct CarbonAwareJob {
    pub work: WorkId,
    pub class: WorkClass,
    pub deferrability: SustainabilityDeferrability,
    pub deadline: Option<Timestamp>,
}
```

---

# 26. No Missed Deadline For Carbon Optimization

Hard rule.

---

# 27. Renewable Window

```rust
pub struct RenewableWindow {
    pub region: RegionId,
    pub starts_at: Timestamp,
    pub ends_at: Timestamp,
    pub intensity: CarbonIntensity,
}
```

---

# 28. Scheduling Decision

```rust
pub enum SustainabilitySchedulingDecision {
    RunNow,
    DeferUntil(Timestamp),
    RunInRegion(RegionId),
    NoSustainableAlternative,
}
```

---

# 29. Region Shift

Only if:

```text
security
privacy
residency
latency
reliability
```

allow it.

---

# 30. Hard rule.

---

# 31. No Silent Cross-Border Carbon Routing

Hard rule.

---

# 32. Carbon-Aware Region Selection

Selection order:

```text
eligible by security/privacy/residency
→ sufficient reliability/capacity
→ acceptable latency
→ environmental preference
→ cost
```

---

# 33. Hard rule.

---

# 34. Carbon Budget

```rust
pub struct CarbonBudget {
    pub scope: SustainabilityScope,
    pub period: BudgetPeriodId,
    pub target_grams_co2e: FixedPoint,
    pub consumed_grams_co2e: FixedPoint,
}
```

---

# 35. No Per-User Carbon Budget

Hard rule.

---

# 36. Carbon Budget State

```rust
pub enum CarbonBudgetState {
    Healthy,
    Warning,
    Exceeded,
    Unknown,
}
```

---

# 37. Exceeded Does Not Override Reliability

Hard rule.

---

# 38. Embodied Carbon

Hardware manufacturing impact.

---

# 39. Hardware Embodied Carbon Record

```rust
pub struct EmbodiedCarbonRecord {
    pub hardware: HardwareAssetId,
    pub estimated_grams_co2e: FixedPoint,
    pub source: EmbodiedCarbonSource,
}
```

---

# 40. Source

```rust
pub enum EmbodiedCarbonSource {
    ManufacturerPublished,
    LifecycleAssessment,
    ConservativeEstimate,
}
```

---

# 41. Hard Rule

Embodied carbon is modeled separately from electricity use.

---

# 42. Hardware Lifecycle

```rust
pub enum HardwareLifecycleState {
    Planned,
    InService,
    Degraded,
    RepairCandidate,
    Refurbished,
    SecondaryUse,
    Decommissioning,
    Recycled,
    Disposed,
}
```

---

# 43. No Automatic Disposal On First Fault

Hard rule.

---

# 44. Hardware Lifecycle Record

```rust
pub struct HardwareLifecycleRecord {
    pub asset: HardwareAssetId,
    pub state: HardwareLifecycleState,
    pub commissioned_at: Timestamp,
    pub last_assessment: Timestamp,
}
```

---

# 45. Expected Lifetime

```rust
pub struct HardwareLifetimePolicy {
    pub hardware_class: HardwareClass,
    pub minimum_expected_lifetime: Duration,
    pub repair_preference: bool,
}
```

---

# 46. Reliability Still Governs

A failing critical host is not retained merely to meet sustainability targets.

---

# 47. Hard rule.

---

# 48. Repair vs Replace

Decision inputs:

```text
failure risk
repairability
energy efficiency
embodied carbon
capacity requirement
security support
firmware support
```

---

# 49. Hardware Decision

```rust
pub enum HardwareDisposition {
    ContinueUse,
    Repair,
    Refurbish,
    SecondaryUse,
    Recycle,
    Replace,
}
```

---

# 50. Security Support Sunset

Unsupported firmware/hardware can force retirement.

---

# 51. Hard rule.

---

# 52. Secondary Use

Older hardware may serve:

```text
development
test
batch jobs
cold storage
offline tools
```

---

# 53. Not:

```text
critical trust root
security-sensitive production
```

unless policy permits.

---

# 54. Hard rule.

---

# 55. Resource Reuse

Prefer reusing available approved capacity before procurement when safe.

---

# 56. Reuse Candidate

```rust
pub struct ReuseCandidate {
    pub asset: AssetId,
    pub capacity: CapacitySnapshotRef,
    pub security_posture: SecurityPostureLevel,
    pub lifecycle: AssetLifecycleState,
}
```

---

# 57. Reuse Decision

```rust
pub enum ReuseDecision {
    Reuse,
    ReuseWithUpgrade,
    Reject,
}
```

---

# 58. No Reuse If Security Posture Inadequate

Hard rule.

---

# 59. Idle Capacity Reclamation

Can reduce new hardware procurement.

---

# 60. But do not reclaim protected failover/security reserves.

---

# 61. Hard rule.

---

# 62. Consolidation

Consolidate lightly utilized workloads if:

```text
fault-domain diversity preserved
capacity reserve preserved
tenant isolation preserved
```

---

# 63. Part 110 interaction.

---

# 64. Hard rule.

---

# 65. E-Waste

Track disposition.

```rust
pub enum EWasteDisposition {
    Reused,
    Refurbished,
    RecycledCertified,
    DonatedApproved,
    Disposed,
}
```

---

# 66. Data Erasure Before Reuse/Recycle

Part 67.

---

# 67. Hard rule.

---

# 68. Storage Device Retirement

Requires:

```text
cryptographic erasure
secure erase
physical destruction where necessary
```

---

# 69. Hard rule.

---

# 70. Reuse After Sanitization

Asset may re-enter lower-risk role after verified sanitization.

---

# 71. Hard rule.

---

# 72. Sustainable Procurement

Procurement inputs:

```text
performance-per-watt
repairability
expected lifetime
vendor support
firmware longevity
recycled content
energy efficiency
```

---

# 73. Security/compatibility remain mandatory.

---

# 74. Hard rule.

---

# 75. Procurement Policy

```rust
pub struct SustainableProcurementPolicy {
    pub hardware_class: HardwareClass,
    pub minimum_support_lifetime: Duration,
    pub repairability_required: bool,
    pub minimum_efficiency_score: Option<FixedPoint>,
}
```

---

# 76. No Single Environmental Score

Prefer matrix.

---

# 77. Hard rule.

---

# 78. Hardware Efficiency

Useful metric:

```text
work units / watt-hour
```

---

# 79. But workload-specific.

---

# 80. Hard rule.

---

# 81. Performance per Watt

```rust
pub struct PerformancePerWatt {
    pub work_units: FixedPoint,
    pub watt_hours: FixedPoint,
}
```

---

# 82. No User Value Metric

Hard rule.

---

# 83. Server Power States

Use:

```text
active
low-power
sleep
off
```

only where reliability class permits.

---

# 84. Hard rule.

---

# 85. Scale-Down & Sustainability

Scale-down idle capacity where:

```text
reliability floor preserved
failover reserve preserved
cold-start latency acceptable
```

---

# 86. Hard rule.

---

# 87. Scale-to-Zero

Only for best-effort/noncritical workloads.

---

# 88. Hard rule.

---

# 89. Carbon-Aware Autoscaling

Part 101 extension.

Can prefer low-carbon eligible capacity when scaling out.

---

# 90. Cannot delay critical scale-out waiting for cleaner energy.

---

# 91. Hard rule.

---

# 92. Carbon-Aware Capacity Forecast

Part 111 extension.

Forecast:

```text
future capacity
energy intensity
hardware acquisition
```

---

# 93. Good.

---

# 94. Renewable Capacity Planning

If region has variable renewable availability:

```text
deferrable jobs shift
critical services remain provisioned
```

---

# 95. Hard rule.

---

# 96. Storage Sustainability

Cold data may move to lower-energy storage if:

```text
latency
durability
residency
recovery
```

allow it.

---

# 97. Hard rule.

---

# 98. Data Minimization

Part 67 also reduces environmental load.

Delete data that no longer needs retention.

---

# 99. Hard rule.

---

# 100. Cache Sustainability

Unnecessary caches waste RAM and energy.

Evaluate saved work vs retained memory/energy.

---

# 101. Good.

---

# 102. Compression Tradeoff

Compression can reduce network/storage energy but increase CPU.

---

# 103. Evaluate end-to-end.

---

# 104. Hard rule.

---

# 105. Network Sustainability

Reduce duplicate transfers, reconnect storms, excessive keepalives, and redundant sync.

---

# 106. Security/privacy traffic protected.

---

# 107. Hard rule.

---

# 108. CDN/Edge

Use for public artifacts where efficient.

---

# 109. Private content only under explicit policy.

---

# 110. Hard rule.

---

# 111. Mixnet Sustainability

Cover traffic/delays consume energy.

---

# 112. They remain protected privacy costs.

---

# 113. Optimize implementation, not privacy floor.

---

# 114. Hard rule.

---

# 115. Relay Sustainability

Prefer efficient servers/network paths within allowed regions and diversity policy.

---

# 116. No user-flow location optimization.

---

# 117. Hard rule.

---

# 118. Media Sustainability

Adaptive bitrate and hardware codec use can lower energy.

---

# 119. Preserve call security/privacy.

---

# 120. Hard rule.

---

# 121. Client-Side Energy Sustainability

Part 113.

Idle efficiency reduces battery usage and device wear.

---

# 122. No battery optimization by weakening message durability/security.

---

# 123. Hard rule.

---

# 124. Update Sustainability

Delta updates reduce network/storage.

---

# 125. Must verify final artifact digest.

---

# 126. Hard rule.

---

# 127. Reproducible Builds

Part 73.

Avoid unnecessary duplicate build pipelines/artifacts.

---

# 128. Build cache can reduce energy if supply-chain safety maintained.

---

# 129. Hard rule.

---

# 130. CI Sustainability

Schedule low-priority CI to available capacity/windows where practical.

---

# 131. Security-critical CI not delayed.

---

# 132. Hard rule.

---

# 133. Build Work Consolidation

Batch similar builds/tests carefully.

---

# 134. Preserve isolation/reproducibility.

---

# 135. Hard rule.

---

# 136. Hardware Inventory Integration

Part 106.

Track:

```text
hardware class
age
lifecycle
energy profile
repair state
```

---

# 137. No employee/user association.

---

# 138. Hard rule.

---

# 139. Desired-State Integration

Part 105.

Sustainability scheduling/power policies become signed desired state where relevant.

---

# 140. Hard rule.

---

# 141. Change Integration

Part 107.

Hardware consolidation, region migration, power-state policy, or carbon-aware placement are governed changes.

---

# 142. Hard rule.

---

# 143. Release Integration

Part 108.

Release readiness may include:

```text
energy regression
hardware requirement change
background-work increase
```

---

# 144. Hard rule.

---

# 145. Reliability Integration

Part 109.

Sustainability cannot consume reliability budget silently.

---

# 146. Hard rule.

---

# 147. Resilience Integration

Part 110.

Consolidation/reuse must preserve fault-domain diversity.

---

# 148. Hard rule.

---

# 149. Capacity Forecast Integration

Part 111.

Environmental planning uses future capacity demand.

---

# 150. Hard rule.

---

# 151. Performance Integration

Part 112.

Energy-saving changes cannot violate latency/throughput budget.

---

# 152. Hard rule.

---

# 153. Efficiency Integration

Part 113.

Efficiency metrics feed sustainability model.

---

# 154. Hard rule.

---

# 155. FinOps Integration

Part 102.

Cost and carbon are different dimensions.

---

# 156. Cheap is not necessarily low-carbon.

---

# 157. Hard rule.

---

# 158. Geographic Governance Integration

Part 103.

Carbon-aware placement cannot cross prohibited jurisdiction/region policy.

---

# 159. Hard rule.

---

# 160. Organizational Governance Integration

Part 104.

Sustainability policy has accountable owner.

---

# 161. No employee environmental scoring.

---

# 162. Hard rule.

---

# 163. Sustainability Data Provenance

Every external carbon/energy factor stores source/version/time.

---

# 164. Hard rule.

---

# 165. Environmental Evidence

```rust
pub struct SustainabilityEvidence {
    pub scope: SustainabilityScope,
    pub period: TimeRange,
    pub energy: Option<FixedPoint>,
    pub carbon: Option<FixedPoint>,
    pub data_source_digest: Digest,
}
```

---

# 166. No Marketing-Only Number

Hard rule.

---

# 167. Sustainability Snapshot

```rust
pub struct SustainabilitySnapshot {
    pub snapshot_id: SustainabilitySnapshotId,
    pub generated_at: CoarseTimestamp,
    pub evidence_digest: Digest,
}
```

---

# 168. Signed for assurance if needed.

---

# 169. Hard rule.

---

# 170. Environmental Reporting

Report:

```text
energy by infrastructure class
estimated carbon by region/provider
hardware lifecycle
reuse/refurbishment
e-waste disposition
```

---

# 171. No user-level report.

---

# 172. Hard rule.

---

# 173. Uncertainty

Environmental data often approximate.

---

# 174. Include confidence.

```rust
pub enum SustainabilityConfidence {
    Low,
    Medium,
    High,
}
```

---

# 175. No false precision.

---

# 176. Hard rule.

---

# 177. Carbon Accounting Boundary

Define included sources.

---

# 178. Example:

```text
operational compute
storage
network estimate
embodied hardware
```

---

# 179. Keep scope explicit.

---

# 180. Hard rule.

---

# 181. Lifecycle Amortization

Embodied carbon may be amortized over expected useful lifetime.

---

# 182. Estimate only.

---

# 183. Hard rule.

---

# 184. Hardware Extension Tradeoff

Old hardware:

```text
higher energy
lower new embodied carbon
```

---

# 185. Evaluate both.

---

# 186. Hard rule.

---

# 187. Environmental Scenario

```rust
pub enum SustainabilityScenario {
    Baseline,
    RenewableWindow,
    HardwareReuse,
    RegionShift,
    Consolidation,
    HardwareRefresh,
}
```

---

# 188. Scenario Analysis

Compare:

```text
energy
carbon
cost
capacity
reliability
performance
```

---

# 189. Multi-dimensional.

---

# 190. Hard rule.

---

# 191. No Single Sustainability Score

Use matrix.

---

# 192. Hard rule.

---

# 193. Environmental Decision

```rust
pub struct SustainabilityDecision {
    pub scenario: SustainabilityScenario,
    pub energy_delta: Option<FixedPoint>,
    pub carbon_delta: Option<FixedPoint>,
    pub constraints_satisfied: bool,
}
```

---

# 194. If constraints false → reject.

---

# 195. Hard rule.

---

# 196. Carbon-Aware Scheduler Interface

```rust
pub trait CarbonAwareScheduler {
    fn decide(
        &self,
        job: &CarbonAwareJob,
        regions: &[RegionDescriptor],
        windows: &[RenewableWindow],
    ) -> Result<SustainabilitySchedulingDecision, SustainabilityError>;
}
```

---

# 197. Hardware Lifecycle Service

```rust
pub trait HardwareLifecycleService {
    fn assess(
        &self,
        asset: HardwareAssetId,
    ) -> Result<HardwareDisposition, SustainabilityError>;
}
```

---

# 198. Sustainability Planning Service

```rust
pub trait SustainabilityPlanningService {
    fn compare(
        &self,
        scenarios: &[SustainabilityScenario],
        scope: SustainabilityScope,
    ) -> Result<Vec<SustainabilityDecision>, SustainabilityError>;
}
```

---

# 199. No Direct Mutation

Execution through Part 107.

---

# 200. Hard rule.

---

# 201. Error Taxonomy

```rust
pub enum SustainabilityError {
    EnergyDataMissing,
    CarbonDataMissing,
    ConfidenceTooLow,
    ResidencyConflict,
    ReliabilityConflict,
    PerformanceConflict,
    SecurityConflict,
    HardwareUnsupported,
    ScopeViolation,
    Unauthorized,
    Internal,
}
```

---

# 202. Observability

Safe metrics:

```text
energy by service/region
carbon intensity by region
hardware age/lifecycle
reuse rate
recycling rate
```

---

# 203. Forbidden:

```text
per-user carbon
private activity
employee environmental ranking
```

---

# 204. Hard rule.

---

# 205. Sustainability SLOs

Examples:

```text
100% retired storage sanitized
hardware lifecycle inventory freshness
deferrable jobs shifted only within deadline
carbon data freshness
```

---

# 206. Hard rule.

---

# 207. Security SLO

```text
0 sustainability optimization bypassing security/residency/reliability gates
```

---

# 208. Privacy SLO

```text
0 user-level sustainability profiling
0 carbon-aware routing based on private user behavior
```

---

# 209. Failure Modes

```text
stale carbon data
provider greenwashing metadata
energy optimization causes latency regression
hardware kept beyond safe support
consolidation creates SPOF
```

---

# 210. Stale Carbon Data

Mark Unknown.

---

# 211. Do not defer critical workload based on stale signal.

---

# 212. Hard rule.

---

# 213. Greenwashing Risk

Prefer provenance and independent validation.

---

# 214. No marketing label as technical truth.

---

# 215. Hard rule.

---

# 216. Latency Regression

Rollback sustainability change.

---

# 217. Hard rule.

---

# 218. Unsupported Hardware

Retire despite environmental preference.

---

# 219. Hard rule.

---

# 220. Consolidation Risk

Resilience model must approve.

---

# 221. Hard rule.

---

# 222. Testing

Need sustainability testkit.

---

# 223. Test Scenarios

```text
renewable-window batch scheduling
region carbon shift
hardware repair vs replace
storage reuse
server consolidation
```

---

# 224. Deadline Test

Deferred job still completes before deadline.

---

# 225. Residency Test

Carbon-aware scheduler rejects forbidden region.

---

# 226. Reliability Test

Scheduler rejects region without required redundancy/capacity.

---

# 227. Security Test

Critical security job never deferred for carbon reason.

---

# 228. Carbon Data Test

Stale/unknown intensity cannot produce strong claim.

---

# 229. Hardware Test

Unsupported firmware forces retirement.

---

# 230. Reuse Test

Sanitization/security posture required before reuse.

---

# 231. E-Waste Test

Retired storage requires disposition evidence.

---

# 232. Consolidation Test

No required redundancy loss.

---

# 233. Privacy Test

No user identity accepted as sustainability dimension.

---

# 234. Release Test

Energy regression visible in readiness.

---

# 235. Fuzzing

Fuzz:

```text
carbon windows
hardware lifecycle transitions
environmental budgets
procurement policies
sustainability scenarios
```

---

# 236. Property Tests

Properties:

```text
carbon-aware scheduler can never select region outside allowed geographic policy
critical work can never be deferred beyond its deadline
hardware reuse can never bypass sanitization/security checks
sustainability policy can never weaken privacy/security/reliability invariants
```

---

# 237. Formal Verification Targets

Strong candidates:

```text
scheduler constraint precedence
hardware lifecycle state machine
deadline-safe deferral
reuse eligibility
```

---

# 238. Kani Candidate

policy precedence and lifecycle invariants.

---

# 239. TLA+ Candidate

job arrives → defer/run → deadline → completion under changing carbon windows.

---

# 240. Loom Candidate

concurrent schedule update + cancellation + job execution.

---

# 241. Performance

Sustainability decisions are off hot path for most work.

---

# 242. Critical realtime path must not wait on external carbon API.

---

# 243. Hard rule.

---

# 244. Carbon Data Cache

Versioned/freshness-bound.

---

# 245. Unknown on expiry.

---

# 246. Hard rule.

---

# 247. Storage

Separate:

```text
energy aggregates
carbon-intensity records
hardware lifecycle
embodied-carbon estimates
reuse/recycle evidence
sustainability policy
```

---

# 248. No user event warehouse.

---

# 249. Hard rule.

---

# 250. Partitioning

By:

```text
service
region
resource pool
hardware class
asset
work class
```

---

# 251. No user partition.

---

# 252. Hard rule.

---

# 253. Crate Layout

Recommended:

```text
crates/
├── siar-sustainability-core/
├── siar-carbon-intensity/
├── siar-carbon-aware-scheduler/
├── siar-operational-energy/
├── siar-hardware-lifecycle/
├── siar-embodied-carbon/
├── siar-resource-reuse/
├── siar-sustainable-procurement/
├── siar-ewaste/
├── siar-sustainability-observability/
└── siar-sustainability-testkit/
```

---

# 254. `siar-sustainability-core`

Owns:

```text
SustainabilityScope
SustainabilityDimension
SustainabilityScenario
errors
```

---

# 255. `siar-carbon-intensity`

External carbon-data normalization/provenance.

---

# 256. `siar-carbon-aware-scheduler`

Deferrable-work timing/eligible-region decisions.

---

# 257. `siar-operational-energy`

Aggregate service/region energy accounting.

---

# 258. `siar-hardware-lifecycle`

Repair/refurbish/reuse/retire transitions.

---

# 259. `siar-embodied-carbon`

Hardware lifecycle estimates.

---

# 260. `siar-resource-reuse`

Idle-capacity/secondary-use eligibility.

---

# 261. `siar-sustainable-procurement`

Hardware selection constraints/evidence.

---

# 262. `siar-ewaste`

Sanitization/disposition evidence.

---

# 263. `siar-sustainability-observability`

Aggregate environmental health only.

---

# 264. `siar-sustainability-testkit`

scheduling/hardware/privacy tests.

---

# 265. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. Sustainability decisions are subordinate to correctness, security, privacy, residency, reliability, and required performance; environmental optimization cannot weaken a hard invariant.
2. Sustainability telemetry is scoped to services, regions, resource pools, hardware classes, assets, and work classes and never uses stable user identity, private content, social graphs, or behavioral activity.
3. Carbon-aware scheduling may defer only explicitly deferrable work and can never delay critical security, emergency, recovery, control-plane, or correctness-preserving operations beyond policy.
4. Carbon-aware placement may select only regions already allowed by security, privacy, residency, capacity, and reliability policy; no "green" region creates an exception.
5. Carbon intensity and embodied-carbon values always carry provenance, timestamp/version, confidence, and accounting scope; stale or uncertain environmental data is never represented as exact fact.
6. Hardware lifecycle policy prefers safe repair, refurbishment, secondary use, and reuse where possible but never keeps unsupported, insecure, unreliable, or privacy-incompatible hardware in critical production.
7. Hardware/storage reuse requires verified sanitization, cryptographic erasure where applicable, and security-posture validation before reassignment.
8. Consolidation and reuse may not consume protected failover/security capacity, violate tenant isolation, or create unreviewed common-mode failure or single points of failure.
9. Mixnet cover traffic, anonymity delay, E2EE, HSM use, authorization, security monitoring, and other required privacy/security energy costs are protected and cannot be removed merely to reduce environmental impact.
10. Sustainability reports and decisions use aggregate infrastructure evidence and cannot become per-user carbon scoring, user nudging, employee ranking, or behavior-surveillance systems.
11. Cost and carbon are separate dimensions: cheaper infrastructure is not automatically greener, and lower-carbon infrastructure is not automatically acceptable if it violates hard operational constraints.
12. Sustainability engineering integrates with resource efficiency, performance, capacity forecasting, SLOs, resilience, DR, FinOps, residency, desired state, inventory, change/release governance, and organizational ownership without creating an alternate policy bypass.
```

---

# 266. Initial Production Scope

Implement first:

```text
typed SustainabilityScope/SustainabilityDimension
operational energy aggregates
carbon-intensity source/provenance model
carbon-aware scheduling for deferrable background work
renewable-window model
security/privacy/residency-first region filtering
hardware lifecycle state machine
repair/refurbish/reuse/retire policy
storage-device sanitization/reuse evidence
embodied-carbon metadata
resource reuse eligibility
sustainable procurement policy
e-waste disposition tracking
sustainability scenario comparison
release/change integration
privacy-safe environmental dashboards
sustainability testkit
```

Then add:

```text
advanced provider carbon-data adapters
hardware telemetry integration
environmental capacity forecasting
carbon-aware CI/build scheduling
multi-provider sustainability optimization
lifecycle carbon amortization models
formal deadline/placement verification
```

---

# 267. Definition of Done

Part 114 is complete when:

- sustainability scopes/dimensions are typed
- operational energy is measured in aggregate
- carbon-intensity data has provenance/confidence/freshness
- only deferrable work can be shifted for carbon reasons
- geographic/security/reliability filters precede carbon optimization
- hardware lifecycle supports repair/refurbish/reuse/retirement
- reuse requires sanitization/security validation
- protected redundancy cannot be reclaimed
- sustainability reporting contains no user profiling
- carbon and cost remain separate
- environmental changes are governed through Part 107
- sustainability/privacy/fuzz/formal tests are specified

---

# 268. Final Architecture

```text
               AGGREGATE INFRASTRUCTURE WORK
                           │
                           ▼
                   SUSTAINABILITY POLICY
                           │
            ┌──────────────┼──────────────┐
            │              │              │
         ENERGY         HARDWARE        CARBON
            │              │              │
            └──────────────┼──────────────┘
                           ▼
                ENVIRONMENTAL DECISION
                           │
             ┌─────────────┼─────────────┐
             │             │             │
          SCHEDULE       REUSE        PROCURE
             │             │             │
             └─────────────┼─────────────┘
                           ▼
                 GOVERNED EXECUTION
```

Sustainability safety model:

```text
aggregate energy evidence
+
carbon-data provenance
+
deferrable-work scheduling
+
hardware lifetime extension
+
safe reuse/refurbishment
+
e-waste governance
+
security/privacy/residency-first placement
+
multi-dimensional validation
```

not:

```text
move private workloads anywhere electricity looks greener, keep insecure hardware forever, or assign carbon scores to users
```

---

# 269. Final Principle

Sustainability is the reduction of unnecessary environmental impact without reducing the guarantees the system owes its users.

The correct model is:

```text
measure energy honestly
+
use carbon data with provenance
+
shift only deferrable work
+
reuse hardware safely
+
extend lifetime when support permits
+
recycle/sanitize responsibly
+
preserve redundancy and residency
+
never trade privacy/security for environmental metrics
```

This architecture gives SIAR a privacy-preserving sustainability foundation for carbon-aware scheduling, operational energy efficiency, hardware lifecycle extension, refurbishment, safe reuse, sustainable procurement, and e-waste governance while preserving the anonymity, local-first, least-authority, reliability, performance, and anti-surveillance guarantees established across Parts 34–113.
