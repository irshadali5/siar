# Core System Architecture Part 101 — Anonymous Network Capacity Management, Load Shedding, Admission Control, Autoscaling, Resource Fairness & Privacy-Preserving Availability Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 101  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 8, 12–13, 17, 49–53, 63, 69–70, 74–80, 92, 95–100

**Primary purpose:** define SIAR's capacity-management architecture for service/resource budgets, admission control, overload detection, backpressure, queue isolation, autoscaling, regional capacity, load shedding, brownout modes, tenant/anonymous-user fairness, protected critical traffic, cost-aware scaling, privacy-safe telemetry, and resilient availability without relying on stable user identity or behavioral profiling.

---

# 1. Purpose

Capacity management decides what happens when demand approaches or exceeds available resources.

Without explicit architecture, overloaded systems often fail by:

```text
unbounded queues
memory exhaustion
cascading retries
latency collapse
priority inversion
tenant starvation
privacy downgrade
```

The governing principle is:

> **SIAR should preserve the most important security, control, and communication capabilities under load by bounding work, isolating resources, shedding low-priority demand first, and scaling from privacy-safe operational signals rather than user profiling.**

---

# 2. Architectural Position

```text
Incoming Work
     │
     ▼
Admission Control
     │
     ▼
Priority / Fairness Classification
     │
     ▼
Bounded Queue / Resource Budget
     │
     ├── Execute
     ├── Backpressure
     ├── Defer
     └── Shed
     │
     ▼
Capacity / Autoscaling Feedback
```

---

# 3. Core Separation

Keep distinct:

```text
capacity
admission
priority
fairness
quota
backpressure
load shedding
autoscaling
billing
```

---

# 4. Non-Goals

Part 101 does not create:

```text
per-user behavioral priority
pay-to-bypass security controls
unbounded burst handling
autoscaling based on private content
availability via privacy downgrade
```

---

# 5. Resource Dimensions

```rust
pub enum ResourceDimension {
    Cpu,
    Memory,
    DiskIo,
    NetworkIngress,
    NetworkEgress,
    Connections,
    QueueDepth,
    Storage,
    FileDescriptors,
    CryptoOperations,
}
```

---

# 6. Hard Rule

Capacity decisions must identify the actual constrained resource.

---

# 7. Service Capacity Model

```rust
pub struct ServiceCapacityModel {
    pub service: ServiceId,
    pub budgets: Vec<ResourceBudget>,
    pub overload_policy: OverloadPolicy,
}
```

---

# 8. Resource Budget

```rust
pub struct ResourceBudget {
    pub dimension: ResourceDimension,
    pub soft_limit: u64,
    pub hard_limit: u64,
}
```

---

# 9. Soft Limit

Triggers pressure response.

---

# 10. Hard Limit

Must not be exceeded.

---

# 11. Hard Rule

Hard resource limits are enforced, not advisory.

---

# 12. Capacity State

```rust
pub enum CapacityState {
    Normal,
    Elevated,
    Saturated,
    Critical,
    Recovering,
}
```

---

# 13. Normal

Healthy headroom.

---

# 14. Elevated

Pressure building.

---

# 15. Saturated

Near resource ceiling.

---

# 16. Critical

Risk of service collapse.

---

# 17. Recovering

Pressure reduced but system not yet stable.

---

# 18. No Critical→Normal Jump

Hard rule.

---

# 19. Hysteresis

Required to avoid oscillation.

---

# 20. Capacity Thresholds

```rust
pub struct CapacityThresholds {
    pub elevated: FixedPoint,
    pub saturated: FixedPoint,
    pub critical: FixedPoint,
    pub recovery: FixedPoint,
}
```

---

# 21. Recovery threshold lower than entry threshold.

---

# 22. Hard rule.

---

# 23. Work Classes

```rust
pub enum WorkClass {
    CriticalSecurity,
    EmergencyRealtime,
    ControlPlane,
    DirectMessaging,
    Interactive,
    Background,
    Bulk,
    Optional,
}
```

---

# 24. CriticalSecurity

Examples:

```text
credential revocation
key rotation
security update
incident containment
```

---

# 25. EmergencyRealtime

Emergency call/message.

---

# 26. ControlPlane

Identity, config, service discovery, quorum.

---

# 27. DirectMessaging

Normal user messaging.

---

# 28. Interactive

Search/settings/feed interactions.

---

# 29. Background

Sync/maintenance.

---

# 30. Bulk

Attachments/backups.

---

# 31. Optional

Analytics/prefetch/nonessential work.

---

# 32. Hard Rule

Work class is semantic, not user-status based.

---

# 33. Priority Order

Recommended:

```text
CriticalSecurity
> EmergencyRealtime
> ControlPlane
> DirectMessaging
> Interactive
> Background
> Bulk
> Optional
```

---

# 34. Privacy Floor

Priority never allows privacy downgrade.

---

# 35. Hard rule.

---

# 36. Protected Capacity

Reserve headroom for high-priority work.

---

# 37. Protected Capacity Policy

```rust
pub struct ProtectedCapacity {
    pub class: WorkClass,
    pub minimum_share: FixedPoint,
}
```

---

# 38. Example

Reserve CPU/queue slots for:

```text
security revocation
identity
emergency
```

---

# 39. Hard Rule

Low-priority floods cannot consume all protected capacity.

---

# 40. Resource Pools

Separate where practical.

---

# 41. Example:

```text
realtime pool
control pool
bulk pool
background pool
```

---

# 42. Why

Prevents head-of-line blocking.

---

# 43. Hard rule.

---

# 44. Queue Architecture

Every queue bounded.

---

# 45. Queue Budget

```rust
pub struct QueueBudget {
    pub max_items: usize,
    pub max_bytes: usize,
    pub max_age: Duration,
}
```

---

# 46. No Unbounded Channel

Hard rule.

---

# 47. Tokio Channels

Bounded.

---

# 48. Blocking Work

Dedicated bounded blocking pools.

---

# 49. Hard rule.

---

# 50. Queue State

```rust
pub enum QueueState {
    Healthy,
    Pressured,
    Full,
    Draining,
}
```

---

# 51. Backpressure

Preferred before failure.

---

# 52. Backpressure Mechanisms

```text
await capacity
reduce producer rate
pause optional fetch
defer bulk transfer
return retry-after
```

---

# 53. Hard Rule

Backpressure propagates toward producer rather than accumulating hidden memory.

---

# 54. Admission Control

Decides whether new work enters.

---

# 55. Admission Decision

```rust
pub enum AdmissionDecision {
    Admit,
    AdmitDegraded,
    Queue,
    RetryLater,
    Reject,
}
```

---

# 56. Admit

Normal execution.

---

# 57. AdmitDegraded

Reduced optional features.

---

# 58. Queue

Durable bounded defer.

---

# 59. RetryLater

Caller should retry with policy.

---

# 60. Reject

Cannot accept safely.

---

# 61. Hard Rule

Admission decision is explicit and observable.

---

# 62. Admission Context

```rust
pub struct AdmissionContext {
    pub class: WorkClass,
    pub resource_cost: EstimatedResourceCost,
    pub scope: AdmissionScope,
}
```

---

# 63. Admission Scope

```rust
pub enum AdmissionScope {
    Anonymous,
    Relationship(RelationshipId),
    Tenant(TenantId),
    Service(ServiceId),
    FederationPeer(FederationDomainId),
}
```

---

# 64. No Global User Identity Required

Hard rule.

---

# 65. Estimated Resource Cost

```rust
pub struct EstimatedResourceCost {
    pub cpu_units: u32,
    pub memory_bytes: u64,
    pub io_bytes: u64,
    pub network_bytes: u64,
}
```

---

# 66. Estimate Can Be Coarse

---

# 67. No need to inspect private payload semantics.

---

# 68. Hard rule.

---

# 69. Admission Controller

```rust
pub trait AdmissionController {
    fn decide(
        &self,
        ctx: &AdmissionContext,
        state: &CapacitySnapshot,
    ) -> AdmissionDecision;
}
```

---

# 70. Deterministic/typed.

---

# 71. No remote ML black box baseline.

---

# 72. Hard rule.

---

# 73. Anonymous Fairness

Anonymous users cannot be allocated by stable identity.

---

# 74. Use scoped short-lived mechanisms.

---

# 75. Examples:

```text
capability token
anonymous quota token
connection bucket
proof-of-work/cost token if needed
```

---

# 76. No permanent tracking identifier.

---

# 77. Hard rule.

---

# 78. Fairness Model

```rust
pub enum FairnessScope {
    AnonymousCapability,
    Tenant,
    FederationPeer,
    ServiceClass,
}
```

---

# 79. No `GlobalUser`.

Hard rule.

---

# 80. Fair Share

```rust
pub struct FairShare {
    pub scope: FairnessScope,
    pub weight: u32,
    pub burst: u32,
}
```

---

# 81. Weighted Fair Queuing

Good fit.

---

# 82. Deficit Round Robin

Also suitable.

---

# 83. No strict FIFO across all traffic.

---

# 84. Hard rule.

---

# 85. Tenant Fairness

Each tenant gets bounded share.

---

# 86. Prevent noisy neighbor.

---

# 87. Enterprise tiers may have contracted capacity.

---

# 88. But

security/control capacity remains protected.

---

# 89. Hard rule.

---

# 90. Anonymous Network Fairness

Capacity assignment should not require deanonymization.

---

# 91. Good primitives:

```text
scoped bearer quota
blind/anonymous token
per-connection fairness
```

---

# 92. No stable account identity.

---

# 93. Hard rule.

---

# 94. Federation Fairness

Per peer/domain quotas.

---

# 95. No remote user identity.

---

# 96. Hard rule.

---

# 97. Abuse vs Capacity

Separate concepts.

---

# 98. Capacity handles resource pressure.

---

# 99. Abuse controls malicious behavior.

---

# 100. Do not infer maliciousness merely from heavy legitimate usage.

---

# 101. Hard rule.

---

# 102. Rate Limiting Integration

Part 44.

---

# 103. Rate limit by:

```text
capability
tenant
service endpoint
federation peer
```

---

# 104. Not global user unless authenticated managed scope explicitly needs.

---

# 105. Hard rule.

---

# 106. Token Bucket

```rust
pub struct TokenBucketPolicy {
    pub refill_per_second: u64,
    pub burst: u64,
}
```

---

# 107. Separate bucket per work class.

---

# 108. Critical traffic cannot share low-priority bucket.

---

# 109. Hard rule.

---

# 110. Concurrency Limits

Essential.

---

# 111. Example:

```text
max concurrent uploads
max concurrent DB queries
max active crypto jobs
max relay streams
```

---

# 112. Concurrency Semaphore

Bounded.

---

# 113. No queue-behind-unbounded semaphore.

---

# 114. Hard rule.

---

# 115. Load Shedding

Drop/defer low-priority work when overloaded.

---

# 116. Shedding Order

Recommended:

```text
Optional
→ Bulk
→ Background
→ Interactive extras
```

---

# 117. Do Not Shed

```text
critical revocation
security policy
emergency communication
control-plane quorum
```

unless physically impossible.

---

# 118. Hard rule.

---

# 119. Load Shedding Decision

```rust
pub enum SheddingAction {
    DisableOptionalFeature,
    DropCacheWarmup,
    PauseBulkTransfer,
    DeferBackgroundSync,
    ReduceResultSize,
    RejectNewWork,
}
```

---

# 120. No Privacy Shedding

Never:

```text
disable encryption
disable anonymity
skip authorization
```

---

# 121. Hard rule.

---

# 122. Brownout Mode

Reduce nonessential features before outage.

---

# 123. Brownout Features

Examples:

```text
disable recommendations
disable previews
reduce feed page size
pause analytics
delay attachment transfers
```

---

# 124. Keep core messaging/security.

---

# 125. Hard rule.

---

# 126. Brownout State

```rust
pub enum BrownoutLevel {
    None,
    Level1,
    Level2,
    Level3,
}
```

---

# 127. Level1

Optional features off.

---

# 128. Level2

Bulk/background constrained.

---

# 129. Level3

Core services only.

---

# 130. Explicit UI/ops state.

---

# 131. No hidden behavior.

---

# 132. Hard rule.

---

# 133. Retry Architecture

Retries can amplify overload.

---

# 134. Use:

```text
exponential backoff
jitter
retry budget
server retry-after
```

---

# 135. Retry Budget

```rust
pub struct RetryBudget {
    pub max_attempts: u8,
    pub max_elapsed: Duration,
}
```

---

# 136. No infinite retries.

---

# 137. Hard rule.

---

# 138. Retry Storm Protection

Server can return coarse backoff class.

---

# 139. Clients honor.

---

# 140. No synchronized fixed retry interval.

---

# 141. Hard rule.

---

# 142. Circuit Breakers

Useful for failing dependencies.

---

# 143. Circuit State

```rust
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}
```

---

# 144. Scope per downstream dependency.

---

# 145. No global circuit for unrelated services.

---

# 146. Hard rule.

---

# 147. Bulkhead Isolation

Separate resources across dependency/service classes.

---

# 148. Failure in media service should not exhaust identity service.

---

# 149. Hard rule.

---

# 150. Autoscaling

Scale based on resource/service pressure.

---

# 151. Allowed Signals

```text
CPU
memory
queue depth
request latency
active connections
throughput
```

---

# 152. Forbidden Baseline Signals

```text
user identity
message content
search interests
feed behavior
contact graph
```

---

# 153. Hard rule.

---

# 154. Autoscaling Metric

```rust
pub struct ScalingMetric {
    pub dimension: ResourceDimension,
    pub observed: u64,
    pub target: u64,
}
```

---

# 155. Autoscaling Decision

```rust
pub enum ScalingDecision {
    NoChange,
    ScaleOut(u16),
    ScaleIn(u16),
}
```

---

# 156. Scale-Out

Add capacity.

---

# 157. Scale-In

Careful/drained.

---

# 158. No instant scale-in during recovery.

---

# 159. Hard rule.

---

# 160. Autoscaling Controller

```rust
pub trait AutoscalingController {
    fn decide(
        &self,
        state: &CapacitySnapshot,
        policy: &AutoscalingPolicy,
    ) -> ScalingDecision;
}
```

---

# 161. Autoscaling Policy

```rust
pub struct AutoscalingPolicy {
    pub min_instances: u16,
    pub max_instances: u16,
    pub cooldown: Duration,
    pub scale_out_threshold: FixedPoint,
    pub scale_in_threshold: FixedPoint,
}
```

---

# 162. Hysteresis + Cooldown

Required.

---

# 163. Hard rule.

---

# 164. Privacy-Safe Autoscaling

Metrics aggregate by service/region.

---

# 165. No per-user series.

---

# 166. Hard rule.

---

# 167. Anonymous Traffic Scaling

Scale by total queue/throughput.

---

# 168. No need to identify users.

---

# 169. Hard rule.

---

# 170. Regional Autoscaling

Per region.

---

# 171. Respect residency.

---

# 172. No moving sensitive load to forbidden region.

---

# 173. Hard rule.

---

# 174. Capacity Headroom

Reserve for spikes.

---

# 175. Headroom Policy

```rust
pub struct CapacityHeadroom {
    pub minimum_percent: FixedPoint,
    pub critical_reserve_percent: FixedPoint,
}
```

---

# 176. Critical reserve not consumed by optional work.

---

# 177. Hard rule.

---

# 178. Capacity Forecasting

Can use aggregate time series.

---

# 179. Examples:

```text
daily peak
release event
school-hour usage
regional trend
```

---

# 180. No user-level prediction.

---

# 181. Hard rule.

---

# 182. Forecasting Inputs

Operational aggregates only.

---

# 183. Long-term planning.

---

# 184. No real-time decision based solely on forecast.

---

# 185. Hard rule.

---

# 186. Capacity Planning

Part 63 integration.

---

# 187. Need:

```text
steady state
peak
failure mode
growth
recovery reserve
```

---

# 188. Test under fault.

---

# 189. Hard rule.

---

# 190. Cost-Aware Scaling

Possible.

---

# 191. Cost must not override security/privacy minimum.

---

# 192. Hard rule.

---

# 193. Cost Policy

```rust
pub struct CapacityCostPolicy {
    pub max_hourly_cost: Money,
    pub emergency_override: bool,
}
```

---

# 194. Emergency override allowed for critical service continuity.

---

# 195. Audited at policy level.

---

# 196. Hard rule.

---

# 197. Cost Shedding

If budget constrained:

```text
shed optional/bulk/background
```

---

# 198. Not encryption/anonymity/control plane.

---

# 199. Hard rule.

---

# 200. Storage Capacity

Need quotas + GC.

---

# 201. Storage Classes

```rust
pub enum StorageClass {
    CriticalMetadata,
    Messages,
    Attachments,
    Cache,
    Logs,
    Backups,
    Analytics,
}
```

---

# 202. Deletion Order Under Pressure

Prefer:

```text
expired cache
expired logs
expired analytics
rebuildable state
```

before authoritative data.

---

# 203. Hard rule.

---

# 204. Storage Emergency

If disk nearly full:

```text
stop bulk ingest
pause attachments
protect DB WAL/headroom
```

---

# 205. Do not corrupt authoritative DB.

---

# 206. Hard rule.

---

# 207. Memory Pressure

Actions:

```text
trim caches
reduce concurrency
shed optional work
```

---

# 208. No OOM-as-policy.

---

# 209. Hard rule.

---

# 210. CPU Pressure

Reduce:

```text
background encoding
index rebuild
bulk crypto
analytics
```

---

# 211. Preserve critical crypto/auth.

---

# 212. Hard rule.

---

# 213. Network Egress Pressure

Prioritize:

```text
control
direct messages
emergency
```

over:

```text
attachments
backups
analytics
```

---

# 214. Hard rule.

---

# 215. Connection Pressure

Limit:

```text
idle connections
bulk transfer concurrency
```

---

# 216. Reuse connections.

---

# 217. No mass disconnect of control/security channels.

---

# 218. Hard rule.

---

# 219. Database Capacity

Protect connection pool.

---

# 220. Query Classes

```text
critical writes
interactive reads
background jobs
analytics
```

---

# 221. Separate pool/priority where possible.

---

# 222. Hard rule.

---

# 223. DB Admission

Reject/defer expensive low-priority query under pressure.

---

# 224. Never allow analytics to exhaust production connection pool.

---

# 225. Hard rule.

---

# 226. Queue Durability

Only some queues durable.

---

# 227. Durable:

```text
message outbox
critical jobs
billing/accounting tasks
```

---

# 228. Ephemeral:

```text
cache refresh
prefetch
analytics flush
```

---

# 229. Hard rule.

---

# 230. Durable Queue Pressure

Need spill/partition/retention.

---

# 231. Cannot simply drop authoritative work.

---

# 232. May stop admission.

---

# 233. Hard rule.

---

# 234. Anonymous Mailbox Capacity

Per mailbox capability quota.

---

# 235. No global identity.

---

# 236. Under abuse:

```text
quota
proof
token
expiration
```

---

# 237. No deanonymization.

---

# 238. Hard rule.

---

# 239. Mixnet Capacity

Special.

---

# 240. Real/cover traffic interaction.

---

# 241. Under pressure:

```text
shed optional cover first within privacy floor
reduce bulk anonymous transfers
preserve minimum cover schedule
```

---

# 242. Hard Rule

Never reduce cover below configured privacy floor just to increase throughput.

---

# 243. Mix Node Admission

Bound packets/queues.

---

# 244. No unbounded packet buffering.

---

# 245. Hard rule.

---

# 246. Cover Traffic Budget

```rust
pub struct CoverTrafficBudget {
    pub minimum_rate: u64,
    pub target_rate: u64,
    pub maximum_rate: u64,
}
```

---

# 247. Minimum is privacy floor.

---

# 248. Hard rule.

---

# 249. Anonymous Attachment Transfer Capacity

Defer/throttle bulk.

---

# 250. No direct path fallback.

---

# 251. Hard rule.

---

# 252. Realtime Calls

Need bounded admission.

---

# 253. Call Capacity

```rust
pub struct RealtimeCapacity {
    pub max_sessions: u32,
    pub reserved_emergency_sessions: u32,
}
```

---

# 254. When full:

```text
reject new normal call
preserve emergency reserve
```

---

# 255. Hard rule.

---

# 256. Media Degradation

Can lower bitrate/quality.

---

# 257. But no unencrypted fallback.

---

# 258. Hard rule.

---

# 259. Search Capacity

Reduce result count.

---

# 260. Defer federated fanout.

---

# 261. Local search unaffected.

---

# 262. No private query centralization.

---

# 263. Hard rule.

---

# 264. Notification Capacity

Aggregate low-priority notifications.

---

# 265. Preserve security alerts.

---

# 266. Hard rule.

---

# 267. Feed Capacity

Fallback to chronological/local cache.

---

# 268. Disable recommendation extras first.

---

# 269. Hard rule.

---

# 270. Analytics Capacity

Pause entirely if needed.

---

# 271. Analytics loss acceptable.

---

# 272. Hard rule.

---

# 273. Backup Capacity

Throttle background backup.

---

# 274. Do not starve minimum RPO forever.

---

# 275. Hard rule.

---

# 276. Update Capacity

Security update distribution high priority.

---

# 277. Routine update lower.

---

# 278. Hard rule.

---

# 279. DR Capacity Reserve

Keep reserve for failover/recovery.

---

# 280. Normal load should not consume all standby capacity.

---

# 281. Hard rule.

---

# 282. Regional Failure Capacity

Remaining regions must absorb failover.

---

# 283. Capacity plan includes N-1 failure.

---

# 284. Critical systems maybe N-2.

---

# 285. Hard rule.

---

# 286. Failover Admission

During regional loss:

```text
preserve existing sessions
limit bulk/new optional work
```

---

# 287. Avoid secondary collapse.

---

# 288. Hard rule.

---

# 289. Autoscaling Failure

Need static safe floor.

---

# 290. If autoscaler unavailable:

```text
fixed minimum fleet
manual control
load shedding
```

---

# 291. No uncontrolled scale-to-zero.

---

# 292. Hard rule.

---

# 293. Capacity Control Plane

Must not depend on overloaded data plane.

---

# 294. Separate resources.

---

# 295. Hard rule.

---

# 296. Scheduler Architecture

Resource-aware scheduling.

---

# 297. Scheduler Input

```rust
pub struct SchedulingRequest {
    pub class: WorkClass,
    pub cost: EstimatedResourceCost,
    pub scope: FairnessScope,
    pub deadline: Option<Duration>,
}
```

---

# 298. Scheduler Decision

```rust
pub enum SchedulingDecision {
    RunNow,
    Queue,
    Defer,
    Reject,
}
```

---

# 299. No behavioral priority.

---

# 300. Hard rule.

---

# 301. Deadline Scheduling

Useful for realtime/control.

---

# 302. But deadline cannot bypass hard resource limits.

---

# 303. Hard rule.

---

# 304. Fair Scheduler

Recommended:

```text
weighted fair queueing
+
priority ceilings
+
protected reserves
```

---

# 305. Prevent tenant domination.

---

# 306. Prevent priority starvation.

---

# 307. Hard rule.

---

# 308. Priority Inversion

Need prevention.

---

# 309. Example:

Bulk lock holds resource needed by security task.

---

# 310. Use:

```text
small critical sections
separate pools
priority inheritance where appropriate
```

---

# 311. Hard rule.

---

# 312. Starvation Prevention

Low-priority work needs bounded progress when healthy.

---

# 313. During sustained crisis

may remain deferred.

---

# 314. Explicit.

---

# 315. Hard rule.

---

# 316. Capacity Leases

Possible for expensive jobs.

---

# 317. Lease expires.

---

# 318. Prevent orphaned resource reservation.

---

# 319. Hard rule.

---

# 320. Admission Tokens

Could represent short-lived resource entitlement.

---

# 321. Anonymous-capable.

---

# 322. No long-lived identity.

---

# 323. Hard rule.

---

# 324. Anonymous Resource Token

```rust
pub struct AnonymousResourceToken {
    pub class: WorkClass,
    pub units: u32,
    pub expires_at: Timestamp,
}
```

---

# 325. Signed/blind if privacy architecture supports.

---

# 326. No custom crypto baseline.

---

# 327. Hard rule.

---

# 328. Tenant Quota

Separate from billing.

---

# 329. Quota says resource entitlement.

---

# 330. Billing says payment accounting.

---

# 331. Hard rule.

---

# 332. Quota Policy

```rust
pub struct TenantQuota {
    pub tenant: TenantId,
    pub resource: ResourceDimension,
    pub sustained: u64,
    pub burst: u64,
}
```

---

# 333. Tenant A quota cannot affect B except shared physical pressure.

---

# 334. Hard rule.

---

# 335. Shared Pressure Fairness

When shared resource saturated:

```text
minimum fair share
weighted excess
```

---

# 336. No zeroing small tenants.

---

# 337. Hard rule.

---

# 338. Federation Quota

Peer-scoped.

---

# 339. Prevent one domain saturating ingress.

---

# 340. No user-level remote quota.

---

# 341. Hard rule.

---

# 342. Service-to-Service Quota

Internal services get bounded concurrency.

---

# 343. Avoid cascade.

---

# 344. Hard rule.

---

# 345. Autoscaling And Quotas

Scaling may increase total capacity.

---

# 346. Quotas still prevent burst dominance.

---

# 347. Hard rule.

---

# 348. Admission During Scale-Out

Do not assume new capacity instantly.

---

# 349. Continue load shedding/backpressure until healthy.

---

# 350. Hard rule.

---

# 351. Scale-In Draining

Before removing instance:

```text
stop new work
drain
handoff
terminate
```

---

# 352. No kill active critical work.

---

# 353. Hard rule.

---

# 354. Spot/Preemptible Capacity

Optional for:

```text
batch
background
analytics
```

---

# 355. Not critical control/security baseline.

---

# 356. Hard rule.

---

# 357. Capacity By Service Tier

Examples:

```text
control-plane nodes
relay nodes
mailbox nodes
search nodes
media relays
```

---

# 358. Independent scaling.

---

# 359. No monolithic fleet scale.

---

# 360. Hard rule.

---

# 361. Relay Capacity

Scale by:

```text
connections
bandwidth
CPU
queue
```

---

# 362. Not by identity.

---

# 363. Hard rule.

---

# 364. Mailbox Capacity

Scale by:

```text
stored encrypted objects
delivery queue
I/O
```

---

# 365. No plaintext inspection.

---

# 366. Hard rule.

---

# 367. Search Capacity

Public search service only.

---

# 368. Private search local.

---

# 369. Good.

---

# 370. Media Relay Capacity

Scale by:

```text
sessions
bandwidth
packet rate
```

---

# 371. No content inspection.

---

# 372. Hard rule.

---

# 373. Capacity Telemetry

Safe operational metrics:

```text
queue depth
latency buckets
utilization
throughput
rejection rate
```

---

# 374. Forbidden:

```text
stable user usage history
social graph
private content
```

---

# 375. Hard rule.

---

# 376. Capacity Telemetry Dimensions

Allowed:

```text
service
region
work class
resource dimension
tenant aggregate if managed
```

---

# 377. No arbitrary user tags.

---

# 378. Hard rule.

---

# 379. Capacity Dashboard

Shows:

```text
headroom
saturation
queue pressure
shed rate
autoscaling state
```

---

# 380. No user leaderboard.

---

# 381. Hard rule.

---

# 382. SLO Integration

Capacity should protect SLOs.

---

# 383. Examples:

```text
message latency
security control latency
call setup latency
```

---

# 384. Under pressure

prioritize according to SLO/criticality.

---

# 385. Hard rule.

---

# 386. Availability Budget

Not same as error budget.

---

# 387. Capacity planning ensures headroom before SLO breach.

---

# 388. Good.

---

# 389. Admission Error Responses

Typed.

---

# 390. Example:

```rust
pub enum CapacityError {
    RetryAfter(DurationClass),
    QuotaExceeded,
    ServiceDegraded,
    CapacityUnavailable,
}
```

---

# 391. No revealing global fleet internals.

---

# 392. Hard rule.

---

# 393. Retry-After Privacy

Coarse duration.

---

# 394. Avoid unique timing fingerprint.

---

# 395. Hard rule.

---

# 396. Capacity Incident

Persistent critical saturation can open incident.

---

# 397. Not every peak.

---

# 398. Hard rule.

---

# 399. Capacity Abuse

If source exceeds fair share

rate-limit.

---

# 400. Does not imply malicious attribution.

---

# 401. Hard rule.

---

# 402. Compliance Integration

Part 95.

---

# 403. Controls:

```text
bounded queues
protected security capacity
autoscaler min floor
headroom
regional reserve
```

---

# 404. No compliance through user tracking.

---

# 405. Hard rule.

---

# 406. Incident Integration

Part 96.

---

# 407. Capacity collapse can trigger incident.

---

# 408. Incident containment should not consume entire control pool.

---

# 409. Hard rule.

---

# 410. SOC Integration

Part 97.

---

# 411. Security floods/DoS can inform capacity response.

---

# 412. But SOC does not set user-specific resource priority.

---

# 413. Hard rule.

---

# 414. Vulnerability Integration

Part 98.

---

# 415. Emergency patch rollout may temporarily reserve update capacity.

---

# 416. Hard rule.

---

# 417. Update Integration

Part 99.

---

# 418. Security update traffic prioritized over routine downloads.

---

# 419. No blocking control plane.

---

# 420. Hard rule.

---

# 421. DR Integration

Part 100.

---

# 422. Maintain failover reserve.

---

# 423. During DR

shed optional work before critical restore/control traffic.

---

# 424. Hard rule.

---

# 425. Database Integration

Part 74.

---

# 426. DB pools bounded.

---

# 427. Background reconciliation cannot starve critical writes.

---

# 428. Hard rule.

---

# 429. Queue/Event Integration

Part 75.

---

# 430. Consumer lag drives backpressure/scale.

---

# 431. No unbounded replay storm.

---

# 432. Hard rule.

---

# 433. Consensus Integration

Part 76.

---

# 434. Quorum traffic protected.

---

# 435. No autoscaling that accidentally changes voter set.

---

# 436. Hard rule.

---

# 437. Discovery Integration

Part 77.

---

# 438. Newly scaled instances published only after health/readiness.

---

# 439. Hard rule.

---

# 440. Edge Integration

Part 78.

---

# 441. Edge admission/rate limiting.

---

# 442. Backend revalidates.

---

# 443. Hard rule.

---

# 444. East-West Integration

Part 79.

---

# 445. Per-service concurrency/rate budgets.

---

# 446. No trust in upstream admission alone.

---

# 447. Hard rule.

---

# 448. Secrets Integration

Part 80.

---

# 449. Autoscaled workload credentials issued dynamically.

---

# 450. No image-baked secret.

---

# 451. Hard rule.

---

# 452. Tenant Integration

Part 69.

---

# 453. Quotas/fairness scoped.

---

# 454. Personal anonymous usage independent.

---

# 455. Hard rule.

---

# 456. Federation Integration

Part 58.

---

# 457. Peer quotas and backpressure.

---

# 458. No transitive resource entitlement.

---

# 459. Hard rule.

---

# 460. Deployment Integration

Part 62.

---

# 461. Autoscaler only launches approved artifacts/config.

---

# 462. No dynamic arbitrary image.

---

# 463. Hard rule.

---

# 464. Host Integrity Integration

Part 71.

---

# 465. Scaled instances attested before sensitive traffic.

---

# 466. Hard rule.

---

# 467. Analytics Integration

Part 92.

---

# 468. Capacity metrics aggregate.

---

# 469. Product analytics never drives priority.

---

# 470. Hard rule.

---

# 471. Experimentation Integration

Part 93.

---

# 472. Capacity policy experiments only among safe equivalent policies.

---

# 473. No A/B weaker fairness/security.

---

# 474. Hard rule.

---

# 475. Audit Integration

Part 94.

---

# 476. Audit high-level changes:

```text
quota policy changed
protected reserve changed
emergency brownout activated
```

---

# 477. Not every admission decision.

---

# 478. Hard rule.

---

# 479. Capacity Policy

Signed/versioned.

---

# 480. No dynamic arbitrary remote rule.

---

# 481. Hard rule.

---

# 482. Capacity Policy Example

```rust
pub struct CapacityPolicy {
    pub version: CapacityPolicyVersion,
    pub resource_budgets: Vec<ResourceBudget>,
    pub protected_capacity: Vec<ProtectedCapacity>,
    pub shedding: SheddingPolicy,
    pub autoscaling: AutoscalingPolicy,
}
```

---

# 483. Policy Anti-Rollback

Important if old policy lacks protections.

---

# 484. Hard rule.

---

# 485. Shedding Policy

```rust
pub struct SheddingPolicy {
    pub order: Vec<WorkClass>,
    pub brownout_thresholds: Vec<BrownoutThreshold>,
}
```

---

# 486. Critical classes cannot appear in ordinary shed list.

---

# 487. Type/runtime validation.

---

# 488. Hard rule.

---

# 489. Capacity Snapshot

```rust
pub struct CapacitySnapshot {
    pub state: CapacityState,
    pub resources: Vec<ResourceObservation>,
    pub queues: Vec<QueueObservation>,
    pub timestamp: CoarseTimestamp,
}
```

---

# 490. No user identity.

---

# 491. Hard rule.

---

# 492. Resource Observation

```rust
pub struct ResourceObservation {
    pub dimension: ResourceDimension,
    pub used: u64,
    pub available: u64,
}
```

---

# 493. Queue Observation

```rust
pub struct QueueObservation {
    pub class: WorkClass,
    pub depth: u64,
    pub age: Duration,
}
```

---

# 494. Privacy-safe.

---

# 495. Capacity State Evaluator

```rust
pub trait CapacityStateEvaluator {
    fn evaluate(
        &self,
        snapshot: &CapacitySnapshot,
        thresholds: &CapacityThresholds,
    ) -> CapacityState;
}
```

---

# 496. Load Shedding Controller

```rust
pub trait LoadSheddingController {
    fn actions(
        &self,
        state: CapacityState,
        policy: &SheddingPolicy,
    ) -> Vec<SheddingAction>;
}
```

---

# 497. Fair Scheduler

```rust
pub trait FairScheduler {
    fn schedule(
        &self,
        request: SchedulingRequest,
    ) -> SchedulingDecision;
}
```

---

# 498. Quota Service

```rust
pub trait QuotaService {
    fn consume(
        &self,
        scope: FairnessScope,
        resource: ResourceDimension,
        units: u64,
    ) -> Result<(), CapacityError>;
}
```

---

# 499. No billing side effect.

---

# 500. Hard rule.

---

# 501. Capacity Error Taxonomy

```rust
pub enum CapacityError {
    CapacityUnavailable,
    QueueFull,
    RetryLater,
    QuotaExceeded,
    BrownoutActive,
    AutoscalingLimitReached,
    ProtectedCapacityReserved,
    ScopeInvalid,
    Internal,
}
```

---

# 502. Capacity Observability

Safe metrics:

```text
resource utilization
queue depth
shed rate
retry-later rate
scale events
headroom
```

---

# 503. Forbidden:

```text
per-user lifetime consumption
behavioral priority profile
```

---

# 504. Hard rule.

---

# 505. Capacity SLOs

Examples:

```text
critical-security admission latency
direct-message p95 latency
queue saturation duration
autoscaling reaction time
```

---

# 506. Privacy SLO

```text
0 stable user IDs in autoscaling/admission telemetry
0 privacy-floor reduction under overload
```

---

# 507. Security SLO

```text
0 critical-security starvation by optional work
0 unbounded queue in production
0 autoscaled untrusted artifact admitted
```

---

# 508. Failure Modes

```text
traffic spike
dependency slowdown
autoscaler failure
database saturation
regional loss
```

---

# 509. Traffic Spike

Backpressure + scale + shed.

---

# 510. No uncontrolled queue growth.

---

# 511. Hard rule.

---

# 512. Dependency Slowdown

Circuit breaker/bulkhead.

---

# 513. Do not amplify retries.

---

# 514. Hard rule.

---

# 515. Autoscaler Failure

Static floor + brownout.

---

# 516. No scale-to-zero.

---

# 517. Hard rule.

---

# 518. Database Saturation

Protect critical pool.

---

# 519. Defer analytics/background.

---

# 520. Hard rule.

---

# 521. Regional Loss

Use DR reserve.

---

# 522. Shed noncritical to avoid secondary collapse.

---

# 523. Hard rule.

---

# 524. Capacity Testing

Need dedicated load/capacity testkit.

---

# 525. Test Scenarios

```text
flash crowd
bulk upload storm
critical security event during saturation
regional failover load
autoscaler outage
```

---

# 526. Queue Bound Test

Queue cannot exceed item/byte limit.

---

# 527. Protected Capacity Test

Security task admitted during bulk saturation.

---

# 528. Fairness Test

One tenant cannot starve another.

---

# 529. Anonymous Fairness Test

No stable user identity required.

---

# 530. Brownout Test

Optional features shed first.

---

# 531. Privacy Test

No direct/anonymity downgrade under overload.

---

# 532. Retry Storm Test

Backoff/jitter prevents synchronization.

---

# 533. Circuit Breaker Test

Failing dependency isolated.

---

# 534. Autoscaling Hysteresis Test

No oscillating scale in/out.

---

# 535. Scale-In Drain Test

Active critical work not killed.

---

# 536. DB Saturation Test

Analytics cannot exhaust critical pool.

---

# 537. Mixnet Test

Cover traffic minimum preserved.

---

# 538. Realtime Test

Emergency call reserve preserved.

---

# 539. DR Test

Failover reserve survives normal peak.

---

# 540. Policy Rollback Test

Old weak capacity policy rejected.

---

# 541. Tenant Quota Test

A cannot consume B entitlement.

---

# 542. Federation Quota Test

One peer cannot flood all ingress.

---

# 543. Fuzzing

Fuzz:

```text
capacity policy
admission request
quota token
brownout policy
autoscaling decision input
```

---

# 544. Property Tests

Properties:

```text
critical-security work can never be shed by ordinary shedding policy
queue depth can never exceed configured hard bound
privacy floor can never be disabled by overload state
quota scope cannot authorize consumption outside its resource domain
```

---

# 545. Formal Verification Targets

Strong candidates:

```text
capacity-state machine
protected-capacity invariant
fair scheduler starvation properties
autoscaling hysteresis
```

---

# 546. Kani Candidate

admission/shedding priority invariants.

---

# 547. TLA+ Candidate

load spike → backpressure → scale-out → brownout → recovery.

---

# 548. Loom Candidate

concurrent quota consume + queue admission + scale-state update.

---

# 549. Performance

Capacity control itself must be cheap.

---

# 550. Admission decision O(1) or near.

---

# 551. Metrics preaggregated.

---

# 552. Scheduler bounded.

---

# 553. No synchronous central coordinator for every request.

---

# 554. Hard rule.

---

# 555. Local Decisions

Prefer per-service/local node decisions within signed policy.

---

# 556. Global control only for:

```text
fleet scaling
regional capacity
quota policy
```

---

# 557. Hard rule.

---

# 558. Cache Capacity State

Short TTL.

---

# 559. Stale state tolerated only conservatively.

---

# 560. No stale "capacity available" beyond hard limit.

---

# 561. Hard rule.

---

# 562. Storage

Separate:

```text
capacity policy
quota config
autoscaling state
aggregate utilization
```

---

# 563. No per-user historical warehouse.

---

# 564. Hard rule.

---

# 565. Partitioning

By:

```text
service
region
tenant aggregate
federation peer
work class
```

---

# 566. No user partition.

---

# 567. Hard rule.

---

# 568. Crate Layout

Recommended:

```text
crates/
├── siar-capacity-core/
├── siar-resource-budget/
├── siar-admission-control/
├── siar-backpressure/
├── siar-fair-scheduler/
├── siar-quota/
├── siar-load-shedding/
├── siar-brownout/
├── siar-autoscaling/
├── siar-capacity-planning/
├── siar-capacity-observability/
└── siar-capacity-testkit/
```

---

# 569. `siar-capacity-core`

Owns:

```text
CapacityState
WorkClass
ResourceDimension
errors
```

---

# 570. `siar-resource-budget`

Soft/hard limits/headroom/protected reserves.

---

# 571. `siar-admission-control`

Admission decisions/resource estimates.

---

# 572. `siar-backpressure`

Retry-after/retry budgets/circuit-breakers.

---

# 573. `siar-fair-scheduler`

Weighted fair queueing/priority/deadlines.

---

# 574. `siar-quota`

Tenant/anonymous/federation scoped quotas.

---

# 575. `siar-load-shedding`

Shedding order/policy.

---

# 576. `siar-brownout`

Feature degradation levels.

---

# 577. `siar-autoscaling`

Scale-out/in decisions/hysteresis.

---

# 578. `siar-capacity-planning`

Forecast/headroom/failure-reserve modeling.

---

# 579. `siar-capacity-observability`

Aggregate operational pressure metrics only.

---

# 580. `siar-capacity-testkit`

load/fairness/privacy/autoscaling tests.

---

# 581. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. Every production queue, pool, concurrency limiter, and retry path has explicit hard bounds; no unbounded backlog is allowed to become the overload strategy.
2. Work priority is based on semantic system criticality—security, emergency, control, messaging, interactive, background, bulk—not user identity, wealth, engagement, or behavioral profile.
3. Protected capacity for critical security, emergency, and control-plane work cannot be consumed by optional, bulk, analytics, or background workloads.
4. Overload responses may defer, degrade, backpressure, shed, or reject work, but can never disable encryption, anonymity, authorization, tenant isolation, or other privacy/security floors.
5. Anonymous-user fairness uses scoped capabilities, short-lived quotas, connection/resource tokens, or other unlinkable mechanisms rather than stable global identities.
6. Tenant and federation-peer fairness is scope-isolated; one tenant or peer cannot monopolize shared resource pools or gain access to another scope's quota state.
7. Autoscaling uses aggregate operational resource signals such as CPU, memory, queue depth, throughput, latency, and connection count—not private content or user behavioral histories.
8. Brownout and load-shedding policies remove optional/background/bulk work before direct messaging, emergency, security, and control-plane functionality.
9. Retry, circuit-breaker, bulkhead, and backpressure policies prevent cascading overload and synchronized retry storms.
10. Scale-in, failover, and maintenance operations drain or hand off critical work safely and never assume capacity exists before new instances are healthy, trusted, and admitted.
11. Capacity observability contains aggregate service/region/work-class pressure only and cannot become a per-user usage, social, or behavioral analytics system.
12. Capacity management integrates with quotas, rate limiting, DR, autoscaling, deployment, tenant isolation, federation, security operations, and updates without creating an alternate authorization or privacy-downgrade path.
```

---

# 582. Initial Production Scope

Implement first:

```text
typed ResourceDimension/WorkClass/CapacityState
hard/soft resource budgets
bounded Tokio channels
concurrency semaphores
protected capacity for security/control traffic
admission controller
retry budgets/backoff/jitter
circuit breakers
bulkhead isolation
weighted fair scheduling
tenant/federation quotas
anonymous scoped resource tokens
load-shedding order
brownout levels
autoscaling with hysteresis/cooldown
regional headroom/failover reserve
DB connection-pool isolation
mixnet cover-traffic floor
realtime emergency reserve
privacy-safe capacity metrics
capacity testkit
```

Then add:

```text
advanced forecasting
multi-region global capacity optimizer
anonymous blind quota tokens
cost-aware heterogeneous scaling
formal scheduler/fairness verification
dynamic capacity reservation marketplace where compatible with privacy/economic architecture
```

---

# 583. Definition of Done

Part 101 is complete when:

- all queues and resource pools are bounded
- semantic priority classes are explicit
- critical/security/control traffic has protected capacity
- admission/backpressure/shedding state machines exist
- retries cannot amplify overload indefinitely
- anonymous fairness works without stable identity
- tenants/federation peers cannot starve each other
- brownout disables optional work before core features
- autoscaling uses aggregate privacy-safe signals
- scale-in drains safely
- regional reserve supports failover
- overload never weakens privacy/anonymity/security
- capacity telemetry contains no behavioral user profiles
- load/fairness/autoscaling/privacy/fuzz/formal tests are specified

---

# 584. Final Architecture

```text
                     INCOMING WORK
                          │
                          ▼
                  ADMISSION CONTROL
                          │
                          ▼
             PRIORITY + FAIRNESS CLASS
                          │
             ┌────────────┼────────────┐
             │            │            │
        PROTECTED      NORMAL       OPTIONAL
        CAPACITY       POOLS          POOLS
             │            │            │
             └────────────┼────────────┘
                          ▼
                  BOUNDED EXECUTION
                          │
                  ┌───────┼───────┐
                  │       │       │
               EXECUTE  DEFER   SHED
                          │
                          ▼
                 CAPACITY FEEDBACK
                          │
                  AUTOSCALE / BROWNOUT
```

Capacity safety model:

```text
hard resource bounds
+
protected critical capacity
+
fair admission
+
backpressure
+
load shedding
+
brownout
+
privacy-safe autoscaling
+
failure reserve
```

not:

```text
accept everything, queue forever, let low-value work consume every resource, then drop privacy guarantees when the system collapses
```

---

# 585. Final Principle

Availability is not achieved by serving every request at any cost; it is achieved by preserving the right work under pressure.

The correct model is:

```text
bound everything
+
reserve critical capacity
+
admit fairly
+
backpressure early
+
shed low-value work first
+
scale from aggregate signals
+
preserve privacy floors
+
recover gradually
```

This architecture gives SIAR a privacy-preserving availability foundation for admission control, resource fairness, load shedding, autoscaling, brownout, multi-tenant isolation, anonymous quotas, regional reserve, realtime capacity, mixnet capacity, and overload recovery while preserving the anonymity, local-first, least-authority, and anti-surveillance guarantees established across Parts 34–100.
