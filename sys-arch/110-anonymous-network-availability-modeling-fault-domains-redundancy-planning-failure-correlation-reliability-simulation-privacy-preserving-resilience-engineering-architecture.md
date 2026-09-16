# Core System Architecture Part 110 — Anonymous Network Availability Modeling, Fault Domains, Redundancy Planning, Failure Correlation, Reliability Simulation & Privacy-Preserving Resilience Engineering Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 110  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 50, 63–76, 92, 95–109  

**Primary purpose:** define SIAR's resilience-engineering architecture for availability modeling, fault domains, redundancy planning, correlated failure analysis, quorum survivability, regional/provider diversity, dependency reliability, N+1/N+2 capacity, failure simulation, fault injection, resilience evidence, and privacy-preserving operational modeling.

---

# 1. Purpose

High availability does not come from duplicating infrastructure blindly.

Reliable systems must understand:

```text
which components can fail together
which replicas are actually independent
which dependencies dominate service availability
which failure combinations are survivable
which redundancy is real vs cosmetic
```

The governing principle is:

> **SIAR resilience should be modeled around explicit failure domains, dependency semantics, redundancy structure, and correlated-failure assumptions rather than simplistic replica counts or optimistic independence assumptions.**

---

# 2. Architectural Position

```text
Operational Topology
        │
        ▼
  Fault-Domain Model
        │
        ▼
 Redundancy Topology
        │
        ▼
 Availability Model
        │
        ▼
 Failure Simulation
        │
   ┌────┼─────┐
   │    │     │
 PASS  RISK  FAIL
   │    │     │
   └────┼─────┘
        ▼
 Resilience Planning
```

---

# 3. Core Separation

Keep distinct:

```text
availability
durability
capacity
fault tolerance
redundancy
recovery
correlation
```

---

# 4. Non-Goals

Part 110 does not create:

```text
one opaque reliability score
per-user failure models
fake mathematical precision
assumption that replicas are independent
availability achieved by privacy downgrade
```

---

# 5. Fault Domain

A fault domain is a scope in which one underlying failure can affect multiple assets.

```rust
pub enum FaultDomainType {
    Process,
    Host,
    Rack,
    AvailabilityZone,
    Region,
    Provider,
    NetworkCarrier,
    PowerDomain,
    DatabaseCluster,
    ObjectStore,
    CertificateAuthority,
    SecretBroker,
    ControlPlane,
    SoftwareVersion,
    ConfigurationVersion,
}
```

---

# 6. Hard Rule

Fault domains are technical/infrastructure concepts, not user groups.

---

# 7. Fault Domain ID

```rust
pub struct FaultDomainId(pub [u8; 16]);
```

---

# 8. Fault Domain Record

```rust
pub struct FaultDomain {
    pub id: FaultDomainId,
    pub domain_type: FaultDomainType,
    pub scope: FaultScope,
    pub parent: Option<FaultDomainId>,
}
```

---

# 9. Fault Scope

```rust
pub enum FaultScope {
    Asset(AssetId),
    Service(ServiceId),
    Region(RegionId),
    Provider(ProviderId),
    Environment(EnvironmentClass),
}
```

---

# 10. No User Scope

Hard rule.

---

# 11. Hierarchical Fault Domains

Example:

```text
process
→ host
→ rack
→ zone
→ region
→ provider
```

---

# 12. Hard Rule

Placement must not assume sibling assets are independent if they share a parent failure domain.

---

# 13. Shared Software Fault Domain

Often ignored.

Two replicas running:

```text
same binary
same config
same dependency version
```

may fail simultaneously.

---

# 14. Software Fault Domain

```rust
pub struct SoftwareFaultDomain {
    pub artifact: ArtifactDigest,
    pub config: DesiredStateId,
    pub protocol: ProtocolVersion,
}
```

---

# 15. Hard Rule

Hardware redundancy does not eliminate software-correlated failure.

---

# 16. Shared Credential Fault Domain

If all replicas depend on one CA/HSM/secret broker, that dependency is common-mode.

---

# 17. Hard rule.

---

# 18. Fault Domain Membership

```rust
pub struct FaultDomainMembership {
    pub asset: AssetId,
    pub domains: BTreeSet<FaultDomainId>,
}
```

---

# 19. Multi-Domain Membership

Expected.

---

# 20. Hard rule.

---

# 21. Failure Class

```rust
pub enum FailureClass {
    Crash,
    Omission,
    Latency,
    Partition,
    Corruption,
    Byzantine,
    CapacityExhaustion,
    DependencyUnavailable,
    ConfigurationFailure,
    CredentialFailure,
}
```

---

# 22. Failure Semantics Matter

Crash ≠ corruption.

---

# 23. Hard rule.

---

# 24. Failure Event Model

```rust
pub struct FailureEvent {
    pub class: FailureClass,
    pub domain: FaultDomainId,
    pub duration: FailureDuration,
}
```

---

# 25. Failure Duration

```rust
pub enum FailureDuration {
    Transient(Duration),
    Sustained,
    PermanentUntilRepair,
}
```

---

# 26. Failure Correlation

Failures may be correlated.

---

# 27. Correlation Classes

```rust
pub enum FailureCorrelation {
    Independent,
    SharedDomain,
    CommonDependency,
    CommonSoftware,
    CommonConfiguration,
    ExternalEvent,
    Unknown,
}
```

---

# 28. Unknown ≠ Independent

Hard rule.

---

# 29. Correlation Record

```rust
pub struct CorrelationRelation {
    pub a: FaultDomainId,
    pub b: FaultDomainId,
    pub correlation: FailureCorrelation,
    pub confidence: CorrelationConfidence,
}
```

---

# 30. Confidence

```rust
pub enum CorrelationConfidence {
    Assumed,
    Observed,
    Verified,
}
```

---

# 31. No Precision Theater

Do not fabricate numerical correlation coefficients unless measured.

---

# 32. Hard rule.

---

# 33. Availability Model

Availability model expresses service survival under failure assumptions.

---

# 34. Availability Requirement

```rust
pub struct AvailabilityRequirement {
    pub service: ServiceId,
    pub reliability_class: ReliabilityClass,
    pub minimum_survivable_failures: Vec<FailureScenarioClass>,
}
```

---

# 35. Failure Scenario Class

```rust
pub enum FailureScenarioClass {
    SingleHost,
    SingleZone,
    SingleRegion,
    SingleProvider,
    DependencyLoss,
    QuorumMemberLoss(u8),
}
```

---

# 36. Explicit.

---

# 37. Hard rule.

---

# 38. Redundancy Model

```rust
pub enum RedundancyModel {
    None,
    NPlusOne,
    NPlusTwo,
    ActivePassive,
    ActiveActive,
    NOfM { required: u8, total: u8 },
    Quorum { voters: u8, required: u8 },
}
```

---

# 39. N+1

System survives one capacity/unit failure.

---

# 40. N+2

Survives two independent capacity/unit failures.

---

# 41. Hard Rule

N+1 means nothing unless the spare is in an independent relevant fault domain.

---

# 42. Redundancy Group

```rust
pub struct RedundancyGroup {
    pub group_id: RedundancyGroupId,
    pub members: Vec<AssetId>,
    pub model: RedundancyModel,
    pub required_independence: BTreeSet<FaultDomainType>,
}
```

---

# 43. Example

Two DB replicas in same zone:

```text
not zone-redundant
```

---

# 44. Hard rule.

---

# 45. Redundancy Validation

```rust
pub trait RedundancyValidator {
    fn validate(
        &self,
        group: &RedundancyGroup,
        topology: &TopologySnapshot,
    ) -> Result<RedundancyAssessment, ResilienceError>;
}
```

---

# 46. Assessment

```rust
pub enum RedundancyAssessment {
    Sufficient,
    Insufficient(Vec<RedundancyDefect>),
    Unknown,
}
```

---

# 47. Unknown ≠ Sufficient

Hard rule.

---

# 48. Redundancy Defect

```rust
pub enum RedundancyDefect {
    SharedHost,
    SharedZone,
    SharedRegion,
    SharedProvider,
    SharedCredentialDependency,
    SharedSoftwareFault,
    CapacityInsufficient,
}
```

---

# 49. Useful.

---

# 50. Availability Composition

Dependencies determine service availability.

---

# 51. Series Dependency

If all required components must work:

```text
service availability constrained by weakest/common path
```

---

# 52. Parallel Dependency

If any one redundant path suffices:

```text
availability improves
```

---

# 53. Hard Rule

Do not blindly multiply probabilities when failures are correlated.

---

# 54. Dependency Availability Model

```rust
pub enum DependencyAvailability {
    Required,
    Optional,
    AnyOf(Vec<AssetId>),
    NOfM { required: u8, members: Vec<AssetId> },
}
```

---

# 55. Service Resilience Model

```rust
pub struct ServiceResilienceModel {
    pub service: ServiceId,
    pub dependencies: Vec<ResilienceDependency>,
    pub redundancy_groups: Vec<RedundancyGroupId>,
}
```

---

# 56. Resilience Dependency

```rust
pub struct ResilienceDependency {
    pub dependency: AssetId,
    pub mode: DependencyAvailability,
    pub failure_policy: DependencyFailurePolicy,
}
```

---

# 57. Hard rule.

---

# 58. Critical Path

Part 106 integration.

---

# 59. Critical Path Reliability

Identify paths where:

```text
one unprotected dependency can violate service SLO
```

---

# 60. Single Point of Failure

Explicit.

---

# 61. Hard rule.

---

# 62. Quorum Systems

Need special handling.

---

# 63. Quorum Survival

```rust
pub struct QuorumModel {
    pub voters: u8,
    pub quorum: u8,
    pub fault_domains: Vec<FaultDomainId>,
}
```

---

# 64. Majority Example

```text
3 voters → quorum 2 → tolerate 1 loss
5 voters → quorum 3 → tolerate 2 losses
```

---

# 65. Hard Rule

Voter placement must consider fault domains, not count only.

---

# 66. 3 Voters In One Region

Not region tolerant.

---

# 67. Hard rule.

---

# 68. Consensus Correlated Failure

Shared network/provider/config can remove multiple voters.

---

# 69. Hard rule.

---

# 70. Quorum Placement Policy

```rust
pub struct QuorumPlacementPolicy {
    pub minimum_regions: u8,
    pub minimum_zones: u8,
    pub max_voters_per_fault_domain: u8,
}
```

---

# 71. No Quorum Reduction During Failure

Part 76.

---

# 72. Hard rule.

---

# 73. Database Replication

Separate:

```text
availability
durability
consistency
```

---

# 74. Sync vs Async Replication

Availability tradeoff.

---

# 75. Hard rule.

---

# 76. Replica Lag

Affects RPO.

---

# 77. Part 100.

---

# 78. Hard rule.

---

# 79. Object Storage

Provider replication may hide shared fault domain.

---

# 80. Verify provider durability/region semantics.

---

# 81. No assumption multi-AZ == multi-region.

---

# 82. Hard rule.

---

# 83. Backup Redundancy

Backup copies should span independent failure domains.

---

# 84. But respect residency.

---

# 85. Hard rule.

---

# 86. Relay Fleet Resilience

Relay fleet should tolerate:

```text
host loss
zone loss
regional pressure
```

---

# 87. No user affinity needed.

---

# 88. Hard rule.

---

# 89. Mailbox Provider Resilience

Distinct provider identities/regions.

---

# 90. Avoid all replicas sharing same operator/provider.

---

# 91. Hard rule.

---

# 92. Mixnet Resilience

Need route diversity.

---

# 93. Mix nodes should span:

```text
operators
regions
providers
network domains
```

---

# 94. Hard rule.

---

# 95. Diversity vs Latency

Explicit tradeoff.

---

# 96. Hard rule.

---

# 97. Privacy/Resilience Interaction

Fallback must preserve privacy floor.

---

# 98. No direct path fallback just because anonymous route capacity is low.

---

# 99. Hard rule.

---

# 100. Provider Diversity

Multi-provider can reduce correlated risk.

---

# 101. But increases:

```text
complexity
cost
operational burden
```

---

# 102. Hard rule.

---

# 103. Provider Diversity Requirement

```rust
pub struct ProviderDiversityPolicy {
    pub minimum_providers: u8,
    pub applies_to: BTreeSet<ReliabilityClass>,
}
```

---

# 104. Not required for every service.

---

# 105. Hard rule.

---

# 106. Region Diversity

Critical service may require multi-region.

---

# 107. Residency can constrain choices.

---

# 108. If only one compliant region exists:

```text
availability guarantee must reflect that reality
```

---

# 109. Hard rule.

---

# 110. Power/Network Diversity

Different AZ names may still share:

```text
carrier
power source
provider backbone
```

---

# 111. Hard truth.

---

# 112. No false independence claim.

---

# 113. Capacity Redundancy

Part 101.

---

# 114. N+1 compute capacity.

---

# 115. Need spare after failure.

---

# 116. Hard rule.

---

# 117. Capacity Failure Model

```rust
pub struct CapacitySurvivalRequirement {
    pub service: ServiceId,
    pub failed_units: u16,
    pub required_remaining_capacity: FixedPoint,
}
```

---

# 118. Test.

---

# 119. Hard rule.

---

# 120. Failover Reserve

Reserve enough for:

```text
regional failover
rolling deployment
maintenance
```

---

# 121. Hard rule.

---

# 122. Headroom

Availability model includes load after failure.

---

# 123. A system that survives structurally but saturates is not truly available.

---

# 124. Hard rule.

---

# 125. Latent Failure

Redundant component may already be unhealthy.

---

# 126. Need health validation.

---

# 127. Hard rule.

---

# 128. Dormant Standby Risk

Passive standby can silently rot.

---

# 129. Require periodic drills.

---

# 130. Hard rule.

---

# 131. Redundancy Health

```rust
pub enum RedundancyHealth {
    Healthy,
    Degraded,
    Unsafe,
    Unknown,
}
```

---

# 132. Unknown standby health blocks strong HA claims.

---

# 133. Hard rule.

---

# 134. Failure Detection

Need detect actual failure.

---

# 135. False positives cause unnecessary failover.

---

# 136. False negatives delay recovery.

---

# 137. Hard rule.

---

# 138. Failure Detector

```rust
pub trait FailureDetector {
    fn state(
        &self,
        asset: AssetId,
    ) -> Result<FailureDetectorState, ResilienceError>;
}
```

---

# 139. State

```rust
pub enum FailureDetectorState {
    Healthy,
    Suspected,
    Failed,
    Unknown,
}
```

---

# 140. Suspected ≠ Failed

Hard rule.

---

# 141. Split-Brain Risk

False failure detection + promotion.

---

# 142. Use fencing.

---

# 143. Part 100.

---

# 144. Hard rule.

---

# 145. Failure Detection Signals

Allowed:

```text
health probes
heartbeat
quorum state
replication state
resource saturation
```

---

# 146. No user behavior.

---

# 147. Hard rule.

---

# 148. Health Probe Diversity

Avoid same path for probe and service.

---

# 149. Hard rule.

---

# 150. Dependency Health

Service health includes critical dependency health.

---

# 151. No green service if required dependency unavailable.

---

# 152. Hard rule.

---

# 153. Resilience Simulation

Model failure scenarios before production.

---

# 154. Simulation Types

```rust
pub enum ResilienceSimulationType {
    SingleAssetFailure,
    FaultDomainFailure,
    RegionFailure,
    ProviderFailure,
    NetworkPartition,
    DependencyFailure,
    CapacityLoss,
    QuorumLoss,
    CredentialInfrastructureFailure,
    SoftwareCommonModeFailure,
}
```

---

# 155. No User-Behavior Simulation Required

Hard rule.

---

# 156. Simulation Scenario

```rust
pub struct ResilienceScenario {
    pub scenario_id: ScenarioId,
    pub failure: ResilienceSimulationType,
    pub affected_domains: Vec<FaultDomainId>,
    pub duration: FailureDuration,
}
```

---

# 157. Simulation Output

```rust
pub struct ResilienceSimulationResult {
    pub scenario: ScenarioId,
    pub service_results: Vec<ServiceResilienceResult>,
    pub state: SimulationState,
}
```

---

# 158. Service Result

```rust
pub struct ServiceResilienceResult {
    pub service: ServiceId,
    pub available: bool,
    pub degraded: bool,
    pub lost_capabilities: Vec<CapabilityId>,
    pub estimated_capacity_remaining: FixedPoint,
}
```

---

# 159. No User Impact Identity

Hard rule.

---

# 160. Simulation Confidence

Based on topology/model freshness.

---

# 161. Stale inventory => lower confidence.

---

# 162. Hard rule.

---

# 163. Deterministic Topology Simulation

Preferred baseline.

---

# 164. Monte Carlo

Optional for planning.

---

# 165. If used:

```text
assumptions explicit
correlation model explicit
```

---

# 166. No invented precision.

---

# 167. Hard rule.

---

# 168. Monte Carlo Model

```rust
pub struct ProbabilisticFailureModel {
    pub domain_rates: Vec<FailureRateAssumption>,
    pub correlations: Vec<CorrelationRelation>,
}
```

---

# 169. Assumption Tagged

```rust
pub enum AssumptionSource {
    Measured,
    ProviderPublished,
    Historical,
    ConservativeEstimate,
}
```

---

# 170. Hard rule.

---

# 171. Failure Rate

Only where meaningful.

---

# 172. Do not infer from tiny sample sizes.

---

# 173. Hard rule.

---

# 174. Reliability Simulation

Can compare architectures.

Examples:

```text
3-zone vs 2-zone
single provider vs dual provider
active-passive vs active-active
```

---

# 175. Good.

---

# 176. Chaos Engineering

Simulation is not enough.

---

# 177. Controlled fault injection validates reality.

---

# 178. Hard rule.

---

# 179. Fault Injection Types

```rust
pub enum FaultInjection {
    KillProcess,
    StopHost,
    DropNetwork,
    AddLatency,
    RejectDependency,
    FillDisk,
    ExhaustConnectionPool,
    RevokeCredential,
}
```

---

# 180. Production Fault Injection

High risk.

---

# 181. Requires scoped authorization.

---

# 182. Hard rule.

---

# 183. No Destructive Arbitrary Fault

Fault types bounded.

---

# 184. Hard rule.

---

# 185. Fault Injection Scope

```rust
pub struct FaultInjectionPlan {
    pub change_id: ChangeId,
    pub targets: Vec<AssetId>,
    pub fault: FaultInjection,
    pub duration: Duration,
    pub abort_conditions: Vec<AbortCondition>,
}
```

---

# 186. Part 107 change framework.

---

# 187. Hard rule.

---

# 188. Game Days

Periodic resilience exercises.

---

# 189. Examples:

```text
region outage
DB primary loss
CA outage
mailbox provider failure
mixnet layer loss
```

---

# 190. Hard rule.

---

# 191. Exercise Must Measure

```text
detection
failover
capacity
RTO
operator readiness
privacy preservation
```

---

# 192. Good.

---

# 193. Failure Injection Privacy

No user targeting.

---

# 194. Use infrastructure scopes.

---

# 195. Hard rule.

---

# 196. Reliability Simulation Inputs

Use:

```text
asset graph
fault domains
dependency graph
capacity model
SLOs
DR topology
```

---

# 197. No private message/content.

---

# 198. Hard rule.

---

# 199. Correlated Failure Catalog

Store known common-mode risks.

Examples:

```text
same provider IAM
same root CA
same release artifact
same DNS provider
same dependency vendor
```

---

# 200. Hard rule.

---

# 201. Common-Mode Failure

Often more important than independent hardware failure.

---

# 202. Hard rule.

---

# 203. Software Diversity

Multiple implementations can reduce software common-mode risk.

---

# 204. But increases complexity.

---

# 205. Use selectively.

---

# 206. Hard rule.

---

# 207. Version Diversity

During rollout, mixed versions can improve or worsen resilience.

---

# 208. Must be compatibility-safe.

---

# 209. Hard rule.

---

# 210. Configuration Diversity

Usually undesirable.

---

# 211. Intentional diversity must be explicit.

---

# 212. Hard rule.

---

# 213. Credential Diversity

Critical systems may use independent signing/issuance paths.

---

# 214. No single CA/HSM dependency if threat model requires higher resilience.

---

# 215. Hard rule.

---

# 216. DNS Resilience

Multiple authoritative servers/providers where needed.

---

# 217. Signed bootstrap records.

---

# 218. Hard rule.

---

# 219. Service Discovery Resilience

Part 77.

---

# 220. Local caches + signed records.

---

# 221. Control plane outage should not instantly destroy data plane if safe.

---

# 222. Hard rule.

---

# 223. Control Plane vs Data Plane

Separate fault domains.

---

# 224. A control-plane outage may freeze changes but allow existing sessions.

---

# 225. Hard rule.

---

# 226. Secrets Service Outage

Existing short-lived credentials may continue until expiry.

---

# 227. No static fallback secret.

---

# 228. Hard rule.

---

# 229. HSM Outage

Design signing/verification paths.

---

# 230. No software key fallback.

---

# 231. Hard rule.

---

# 232. Identity Provider Outage

Existing session semantics from Part 82.

---

# 233. Messaging/local operations may continue where safe.

---

# 234. Hard rule.

---

# 235. Database Outage

Define:

```text
read-only
queued writes
local-only
```

---

# 236. Part 100 continuity modes.

---

# 237. Hard rule.

---

# 238. Object Storage Outage

Messages may continue if attachment subsystem degraded.

---

# 239. Capability SLOs separate.

---

# 240. Hard rule.

---

# 241. Search Outage

Core messaging continues.

---

# 242. Good degradation boundary.

---

# 243. Hard rule.

---

# 244. Analytics Outage

No product outage.

---

# 245. Analytics should be lowest resilience priority.

---

# 246. Hard rule.

---

# 247. Reliability Tiering

Map components to resilience tier.

```rust
pub enum ResilienceTier {
    Tier0Critical,
    Tier1High,
    Tier2Standard,
    Tier3BestEffort,
}
```

---

# 248. Tier0

Security/control/core communication.

---

# 249. Tier1

Durable supporting infrastructure.

---

# 250. Tier2

Normal feature services.

---

# 251. Tier3

Optional/background.

---

# 252. Hard Rule

Tier affects redundancy investment but never security policy weakening.

---

# 253. Redundancy Policy

```rust
pub struct ResilienceTierPolicy {
    pub tier: ResilienceTier,
    pub minimum_redundancy: RedundancyModel,
    pub required_fault_domain_diversity: BTreeSet<FaultDomainType>,
}
```

---

# 254. Explicit.

---

# 255. Hard rule.

---

# 256. Availability Budget

Distinct from SLO error budget.

---

# 257. Availability budget = planned architecture capacity for failure.

---

# 258. Hard rule.

---

# 259. Failure Budget

Could describe number of simultaneous failures architecture should tolerate.

---

# 260. Typed:

```rust
pub struct FailureToleranceBudget {
    pub scenario: FailureScenarioClass,
    pub count: u8,
}
```

---

# 261. Good.

---

# 262. No ambiguity.

---

# 263. Resilience Gap

Difference between required and modeled survivability.

---

# 264. Resilience Gap Record

```rust
pub struct ResilienceGap {
    pub service: ServiceId,
    pub requirement: AvailabilityRequirement,
    pub observed: RedundancyAssessment,
    pub severity: ResilienceGapSeverity,
}
```

---

# 265. Severity

```rust
pub enum ResilienceGapSeverity {
    Low,
    Medium,
    High,
    Critical,
}
```

---

# 266. Hard rule.

---

# 267. Gap Remediation

Examples:

```text
move replica
add provider
add zone
remove common dependency
increase capacity reserve
```

---

# 268. No automatic high-risk topology mutation.

---

# 269. Hard rule.

---

# 270. Resilience Planning

Use architecture graph.

---

# 271. Planning Inputs:

```text
SLO targets
fault domains
dependency graph
capacity
cost
residency
```

---

# 272. Security/privacy constraints first.

---

# 273. Hard rule.

---

# 274. Cost vs Resilience

Part 102.

---

# 275. Cost optimization cannot remove redundancy below policy.

---

# 276. Hard rule.

---

# 277. Residency vs Resilience

Part 103.

---

# 278. Fewer compliant regions may reduce availability.

---

# 279. State truthfully.

---

# 280. Hard rule.

---

# 281. Capacity vs Resilience

Part 101.

---

# 282. Redundancy without spare capacity is weak.

---

# 283. Hard rule.

---

# 284. Release vs Resilience

Part 108.

---

# 285. Launch may require resilience certification.

---

# 286. Hard rule.

---

# 287. Change Impact vs Resilience

Part 107.

---

# 288. Proposed change simulation includes fault survivability delta.

---

# 289. Hard rule.

---

# 290. Desired-State Integration

Part 105.

---

# 291. Fault-domain placement policy encoded in desired state.

---

# 292. Drift can break redundancy.

---

# 293. Hard rule.

---

# 294. Inventory Integration

Part 106.

---

# 295. Fault-domain topology derives from canonical inventory.

---

# 296. Stale inventory lowers model confidence.

---

# 297. Hard rule.

---

# 298. SLO Integration

Part 109.

---

# 299. Resilience model must support required SLO.

---

# 300. Hard rule.

---

# 301. DR Integration

Part 100.

---

# 302. Availability model includes failover/recovery paths.

---

# 303. Hard rule.

---

# 304. Incident Integration

Part 96.

---

# 305. Real incidents validate or falsify model assumptions.

---

# 306. Feed model updates.

---

# 307. No employee blame.

---

# 308. Hard rule.

---

# 309. SOC Integration

Part 97.

---

# 310. Security control common-mode failures included.

---

# 311. No user telemetry.

---

# 312. Hard rule.

---

# 313. Compliance Integration

Part 95.

---

# 314. Controls may require:

```text
multi-zone
backup independence
tested failover
```

---

# 315. Evidence from model + drills.

---

# 316. Hard rule.

---

# 317. Resilience Evidence

Need signed summaries.

---

# 318. Resilience Assessment

```rust
pub struct ResilienceAssessment {
    pub service: ServiceId,
    pub model_version: ResilienceModelVersion,
    pub requirements: Vec<AvailabilityRequirement>,
    pub gaps: Vec<ResilienceGap>,
    pub evidence_digest: Digest,
}
```

---

# 319. Model Versioned

Hard rule.

---

# 320. Resilience Model Version

```rust
pub struct ResilienceModelVersion(pub u64);
```

---

# 321. Historical assessments reference exact model.

---

# 322. Hard rule.

---

# 323. Assumption Registry

```rust
pub struct ResilienceAssumption {
    pub id: AssumptionId,
    pub source: AssumptionSource,
    pub statement: AssumptionStatement,
    pub expires_at: Option<Timestamp>,
}
```

---

# 324. Important.

---

# 325. No hidden assumptions.

---

# 326. Hard rule.

---

# 327. Assumption Expiry

Old provider/failure assumptions revalidated.

---

# 328. Hard rule.

---

# 329. Model Confidence

```rust
pub enum ModelConfidence {
    Low,
    Medium,
    High,
    VerifiedByExercise,
}
```

---

# 330. No "verified" without actual drill/test.

---

# 331. Hard rule.

---

# 332. Resilience Certification

Optional for Tier0/Tier1.

---

# 333. Requires:

```text
topology current
simulation pass
drill evidence
no critical gaps
```

---

# 334. Hard rule.

---

# 335. Resilience Certification State

```rust
pub enum ResilienceCertificationState {
    Pending,
    Certified,
    Conditional,
    Blocked,
    Expired,
}
```

---

# 336. Conditional ≠ Certified.

---

# 337. Hard rule.

---

# 338. Simulation Service

```rust
pub trait ResilienceSimulationService {
    fn simulate(
        &self,
        scenario: &ResilienceScenario,
        model: &ServiceResilienceModel,
    ) -> Result<ResilienceSimulationResult, ResilienceError>;
}
```

---

# 339. Fault Domain Service

```rust
pub trait FaultDomainService {
    fn domains_for_asset(
        &self,
        asset: AssetId,
    ) -> Result<Vec<FaultDomainId>, ResilienceError>;
}
```

---

# 340. Resilience Planner

```rust
pub trait ResiliencePlanner {
    fn assess(
        &self,
        service: ServiceId,
    ) -> Result<ResilienceAssessment, ResilienceError>;

    fn propose_remediations(
        &self,
        gap: &ResilienceGap,
    ) -> Result<Vec<ResilienceRemediation>, ResilienceError>;
}
```

---

# 341. Proposed Remediation Is Advisory

Hard rule.

---

# 342. Execution goes through Part 107.

---

# 343. Hard rule.

---

# 344. Failure Injection Service

```rust
pub trait FaultInjectionService {
    fn plan(
        &self,
        scenario: ResilienceScenario,
    ) -> Result<FaultInjectionPlan, ResilienceError>;
}
```

---

# 345. No direct execution without change authority.

---

# 346. Hard rule.

---

# 347. Resilience Error Taxonomy

```rust
pub enum ResilienceError {
    ModelMissing,
    FaultDomainUnknown,
    TopologyStale,
    RedundancyInsufficient,
    CorrelationUnknown,
    SimulationFailed,
    SimulationInconclusive,
    FaultInjectionDenied,
    ScopeViolation,
    Unauthorized,
    Internal,
}
```

---

# 348. Observability

Safe metrics:

```text
resilience gaps
redundancy health
simulation coverage
exercise pass rate
fault-domain diversity
```

---

# 349. Forbidden:

```text
user behavior
private communications
employee ranking
```

---

# 350. Hard rule.

---

# 351. Resilience SLOs

Examples:

```text
Tier0 services assessed monthly
regional failover exercised quarterly
critical redundancy health current
```

---

# 352. Security SLO

```text
0 Tier0 single-point failure accepted unknowingly
0 quorum placement violating minimum domain diversity
```

---

# 353. Privacy SLO

```text
0 resilience decision requiring user-level telemetry
0 failover path reducing anonymity/privacy floor
```

---

# 354. Failure Modes

```text
stale topology
incorrect fault-domain mapping
unknown correlation
simulation/model mismatch
drill failure
```

---

# 355. Stale Topology

Assessment confidence reduced.

---

# 356. High-risk certification may block.

---

# 357. Hard rule.

---

# 358. Incorrect Fault Domain Mapping

Reconcile inventory/provider metadata.

---

# 359. No silent assumption.

---

# 360. Hard rule.

---

# 361. Unknown Correlation

Use conservative assumption.

---

# 362. Hard rule.

---

# 363. Model Mismatch

Incident/game-day shows unexpected failure.

---

# 364. Update model.

---

# 365. Hard rule.

---

# 366. Drill Failure

Open resilience gap.

---

# 367. Do not keep "certified" status unchanged.

---

# 368. Hard rule.

---

# 369. Testing

Need resilience-engineering testkit.

---

# 370. Test Scenarios

```text
host loss
zone loss
region loss
provider loss
software common-mode failure
```

---

# 371. Fault Domain Test

Assets sharing region are correlated at region failure.

---

# 372. Redundancy Test

N+1 fails validation if spare shares same critical fault domain.

---

# 373. Quorum Test

3 voters in 1 region not region-tolerant.

---

# 374. Provider Test

Dual-region same provider != provider diversity.

---

# 375. Software Test

Multiple hosts on same bad artifact share common-mode risk.

---

# 376. Capacity Test

After one-region loss remaining capacity meets requirement.

---

# 377. DR Test

Failover path respects residency/security.

---

# 378. Privacy Test

No direct/anonymity downgrade under failure.

---

# 379. Dependency Test

Single critical dependency identified as SPOF.

---

# 380. Redundant Dependency Test

AnyOf path survives one member failure.

---

# 381. Network Partition Test

Quorum behaves correctly.

---

# 382. Secret Broker Test

Shared secret broker modeled as common dependency.

---

# 383. Simulation Confidence Test

Stale topology lowers confidence.

---

# 384. Exercise Test

Successful drill can elevate confidence to VerifiedByExercise.

---

# 385. Failure Injection Test

Fault cannot execute outside approved scope.

---

# 386. Cost Test

Optimizer cannot remove required redundancy.

---

# 387. Residency Test

Resilience planner cannot propose forbidden region.

---

# 388. Fuzzing

Fuzz:

```text
fault-domain hierarchy
redundancy group
failure scenario
correlation relation
resilience policy
```

---

# 389. Property Tests

Properties:

```text
redundancy group cannot be declared sufficient when required fault-domain diversity is unsatisfied
unknown correlation can never be treated as proven independence
quorum survivability can never exceed what voter/domain placement allows
privacy/security hard floor can never be removed by failover simulation or remediation
```

---

# 390. Formal Verification Targets

Strong candidates:

```text
quorum survivability
fault-domain hierarchy
redundancy validation
failover/capacity survival
```

---

# 391. Kani Candidate

N-of-M/quorum/fault-domain invariants.

---

# 392. TLA+ Candidate

normal → domain failure → failover → degraded → recovery.

---

# 393. Loom Candidate

concurrent failure detection + promotion + fencing update.

---

# 394. Performance

Simulation can be expensive.

---

# 395. Use cached topology snapshots.

---

# 396. No full live provider scan per query.

---

# 397. Hard rule.

---

# 398. Graph Traversal

Bounded.

---

# 399. Reuse Part 106 query budgets.

---

# 400. Hard rule.

---

# 401. Probabilistic Simulation

Run offline/background.

---

# 402. Never block request path.

---

# 403. Hard rule.

---

# 404. Storage

Separate:

```text
fault domains
fault-domain memberships
redundancy groups
correlation assumptions
resilience requirements
simulation results
exercise evidence
```

---

# 405. No user telemetry warehouse.

---

# 406. Hard rule.

---

# 407. Partitioning

By:

```text
service
region
provider
environment
fault domain
```

---

# 408. No user partition.

---

# 409. Hard rule.

---

# 410. Crate Layout

Recommended:

```text
crates/
├── siar-resilience-core/
├── siar-fault-domain/
├── siar-redundancy/
├── siar-correlation-model/
├── siar-availability-model/
├── siar-quorum-resilience/
├── siar-resilience-simulation/
├── siar-fault-injection/
├── siar-resilience-certification/
├── siar-resilience-observability/
└── siar-resilience-testkit/
```

---

# 411. `siar-resilience-core`

Owns:

```text
FaultDomainId
FailureClass
ResilienceTier
errors
```

---

# 412. `siar-fault-domain`

Hierarchy/membership/provider mappings.

---

# 413. `siar-redundancy`

N+1/N+2/N-of-M/active-active validation.

---

# 414. `siar-correlation-model`

Common-mode/shared-domain assumptions.

---

# 415. `siar-availability-model`

Service/dependency survivability.

---

# 416. `siar-quorum-resilience`

Consensus placement/survival analysis.

---

# 417. `siar-resilience-simulation`

Deterministic/probabilistic scenario analysis.

---

# 418. `siar-fault-injection`

Typed game-day/chaos plans.

---

# 419. `siar-resilience-certification`

Tier0/Tier1 resilience assessments/certificates.

---

# 420. `siar-resilience-observability`

Aggregate resilience health only.

---

# 421. `siar-resilience-testkit`

fault-domain/redundancy/privacy tests.

---

# 422. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. Availability models use explicit infrastructure fault domains, dependency semantics, redundancy structures, capacity, and recovery paths—not user identities, private content, social graphs, or behavioral telemetry.
2. Replica count alone never proves redundancy; required fault-domain diversity must be satisfied and validated for the failure classes the service claims to tolerate.
3. Unknown or shared failure correlation can never be silently modeled as independence, and numerical reliability estimates must identify their assumptions and evidence source.
4. Hardware, region, and provider diversity do not eliminate common-mode software, configuration, credential, CA, secret-broker, DNS, or control-plane failures; these are first-class fault domains.
5. Quorum survivability is derived from actual voter placement and correlated failure domains; quorum is never reduced merely to maintain availability during failure.
6. Redundancy plans include post-failure capacity and headroom; a topology that remains structurally connected but saturates under failover is not considered fully resilient.
7. Disaster recovery, failover, redundancy, and provider diversity can never violate security, anonymity, authorization, tenant isolation, residency, or cryptographic trust floors.
8. Simulation and fault injection operate on infrastructure/service scopes and never target or identify users; production fault injection requires scoped change authority, abort conditions, and bounded typed faults.
9. Resilience certification depends on current topology, explicit assumptions, simulation, and exercise evidence; stale topology, unknown correlations, or failed drills reduce confidence or revoke certification.
10. Cost optimization, rightsizing, and scaling cannot reduce redundancy below declared resilience policy without an explicit governed policy change.
11. Resilience evidence and observability expose technical topology/fault-domain status only and cannot become user-tracking, message-flow, or employee-performance datasets.
12. Resilience engineering integrates with SLOs, DR, capacity, FinOps, geographic governance, change impact, release readiness, desired state, inventory, incident response, compliance, and SOC without creating a bypass around hard security/privacy constraints.
```

---

# 423. Initial Production Scope

Implement first:

```text
typed FaultDomainId/FaultDomainType
fault-domain hierarchy
asset memberships
failure classes
correlation classes
redundancy models
N+1/N+2/N-of-M validation
quorum survivability
dependency availability semantics
regional/provider diversity checks
software/config common-mode fault domains
post-failure capacity validation
deterministic resilience simulation
resilience gap detection
game-day/fault-injection plans
model confidence/assumption registry
resilience certification for Tier0/Tier1
audit/compliance integration
privacy-safe resilience metrics
resilience testkit
```

Then add:

```text
probabilistic Monte Carlo planning
advanced correlated-failure models
provider/network-carrier dependency intelligence
automated resilience remediation proposals
formal quorum/redundancy verification
cross-federation resilience attestations
```

---

# 424. Definition of Done

Part 110 is complete when:

- fault domains are typed and hierarchical
- redundancy groups declare required diversity
- replica count cannot falsely imply independence
- common-mode software/config/credential faults are modeled
- quorum survival reflects real placement
- post-failure capacity is checked
- regional/provider/residency constraints are integrated
- resilience simulations expose assumptions/confidence
- fault injection is typed and governed
- resilience gaps are explicit
- Tier0/Tier1 certification can use drill evidence
- privacy/anonymity floors survive failure paths
- resilience telemetry contains no user behavioral dimensions
- redundancy/simulation/privacy/fuzz/formal tests are specified

---

# 425. Final Architecture

```text
                 OPERATIONAL TOPOLOGY
                         │
                         ▼
                   FAULT DOMAINS
                         │
                         ▼
                  REDUNDANCY MODEL
                         │
          ┌──────────────┼──────────────┐
          │              │              │
       QUORUM         CAPACITY       DEPENDENCIES
          │              │              │
          └──────────────┼──────────────┘
                         ▼
                 FAILURE CORRELATION
                         │
                         ▼
                 RESILIENCE SIMULATION
                         │
              ┌──────────┼──────────┐
              │          │          │
           SURVIVES    DEGRADES    FAILS
              │
              ▼
                 RESILIENCE PLANNING
```

Resilience-engineering safety model:

```text
explicit fault domains
+
validated redundancy
+
correlation awareness
+
quorum survivability
+
post-failure capacity
+
simulation and drills
+
security/privacy-preserving failover
+
evidence-backed certification
```

not:

```text
run three replicas, assume they are independent, and call the service highly available
```

---

# 426. Final Principle

Resilience comes from independent failure tolerance, not from replica count alone.

The correct model is:

```text
map fault domains
+
identify correlated risks
+
place redundancy deliberately
+
validate quorum and capacity
+
simulate realistic failures
+
test recovery in practice
+
preserve privacy/security under failover
+
state uncertainty honestly
```

This architecture gives SIAR a privacy-preserving resilience-engineering foundation for fault-domain modeling, redundancy planning, quorum survivability, failure correlation, capacity survival, simulation, chaos/game-day testing, and resilience certification while preserving the anonymity, local-first, least-authority, and anti-surveillance guarantees established across Parts 34–109.
