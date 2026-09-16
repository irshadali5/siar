# Core System Architecture Part 112 — Anonymous Network Performance Budgeting, Latency Decomposition, Throughput Modeling, Tail-Latency Control & Privacy-Preserving Performance Engineering Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 112  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 8, 12–13, 17, 29, 37, 40, 50–52, 63, 75, 101, 107–111  

**Primary purpose:** define SIAR's performance-engineering architecture for end-to-end performance budgets, latency decomposition, throughput modeling, queueing behavior, tail-latency control, concurrency, backpressure, benchmarking, performance-regression gates, anonymous-route overhead, client/server/platform budgets, and privacy-preserving optimization.

---

# 1. Purpose

Performance must be engineered as a budget, not discovered after deployment.

The system should be able to answer:

```text
Where is latency spent?
Which stage dominates p99?
What is the safe throughput ceiling?
What queueing delay appears near saturation?
Which dependency controls end-to-end performance?
How much anonymity overhead is expected?
Which optimizations preserve security and privacy?
```

The governing principle is:

> **SIAR performance should be managed through explicit end-to-end budgets, typed stage decomposition, bounded concurrency, measured throughput ceilings, and tail-aware control—without using private user behavior as an optimization signal or weakening security/privacy guarantees.**

---

# 2. Architectural Position

```text
User/Service Operation
        │
        ▼
 End-to-End Budget
        │
        ▼
 Stage Decomposition
        │
 ┌──────┼────────┐
 │      │        │
CPU   Queue   Network
 │      │        │
 └──────┼────────┘
        ▼
 Critical Path Model
        │
        ▼
 Throughput / Saturation
        │
        ▼
 Tail-Latency Control
        │
        ▼
 Regression / Release Gates
```

---

# 3. Core Separation

Keep distinct:

```text
latency
service time
queue time
throughput
concurrency
utilization
capacity
tail latency
user-perceived delay
```

---

# 4. Non-Goals

Part 112 does not create:

```text
behavioral performance profiling
per-user priority prediction
latency optimization by privacy downgrade
one opaque "performance score"
unbounded concurrency to improve average throughput
```

---

# 5. Performance Scope

```rust
pub enum PerformanceScope {
    Service(ServiceId),
    Capability(CapabilityId),
    Region(RegionId),
    WorkClass(WorkClass),
    Transport(TransportClass),
    Platform(PlatformClass),
    FederationPeer(FederationDomainId),
}
```

---

# 6. No Global User Scope

Hard rule.

---

# 7. Performance Class

```rust
pub enum PerformanceClass {
    RealtimeCritical,
    Interactive,
    DurableMessaging,
    Background,
    Bulk,
}
```

---

# 8. RealtimeCritical

Examples:

```text
call media
call signaling
emergency interaction
```

---

# 9. Interactive

Examples:

```text
conversation open
search
settings
```

---

# 10. DurableMessaging

Message send/receive.

---

# 11. Background

Sync/index/maintenance.

---

# 12. Bulk

Attachments/backups.

---

# 13. Hard Rule

Performance class comes from operation semantics, not user importance.

---

# 14. End-to-End Performance Budget

```rust
pub struct EndToEndPerformanceBudget {
    pub scope: PerformanceScope,
    pub class: PerformanceClass,
    pub latency_target: LatencyTarget,
    pub throughput_target: Option<ThroughputTarget>,
}
```

---

# 15. Latency Target

```rust
pub struct LatencyTarget {
    pub p50: Option<Duration>,
    pub p95: Option<Duration>,
    pub p99: Option<Duration>,
    pub hard_deadline: Option<Duration>,
}
```

---

# 16. No Average-Only Target

Hard rule.

---

# 17. Tail Percentiles

p95/p99 first-class.

---

# 18. Hard rule.

---

# 19. Hard Deadline

For realtime/protocol timeout classes.

---

# 20. Not every operation needs one.

---

# 21. Hard rule.

---

# 22. Throughput Target

```rust
pub struct ThroughputTarget {
    pub sustained_per_second: FixedPoint,
    pub burst_per_second: FixedPoint,
}
```

---

# 23. Sustained vs Burst

Separate.

---

# 24. Hard rule.

---

# 25. Latency Decomposition

End-to-end latency is composed of typed stages.

---

# 26. Performance Stage

```rust
pub enum PerformanceStage {
    UiDispatch,
    Serialization,
    LocalDatabase,
    SchedulerQueue,
    Crypto,
    TransportSetup,
    NetworkTransit,
    RelayProcessing,
    MixDelay,
    ServerQueue,
    ApplicationProcessing,
    Database,
    ObjectStorage,
    FederationTransit,
    DeliveryCommit,
    UiRender,
}
```

---

# 27. Stage Budget

```rust
pub struct StageBudget {
    pub stage: PerformanceStage,
    pub target: LatencyTarget,
}
```

---

# 28. Budget Sum Is Not Always Simple Addition

Some stages overlap.

---

# 29. Hard rule.

---

# 30. Sequential vs Parallel Stages

```rust
pub enum StageComposition {
    Sequential,
    ParallelMax,
    Conditional,
}
```

---

# 31. Sequential

Times add.

---

# 32. ParallelMax

Critical path dominated by slowest branch.

---

# 33. Conditional

Only some requests use stage.

---

# 34. Hard rule.

---

# 35. Performance Path

```rust
pub struct PerformancePath {
    pub path_id: PerformancePathId,
    pub stages: Vec<PerformancePathNode>,
}
```

---

# 36. Performance Path Node

```rust
pub struct PerformancePathNode {
    pub stage: PerformanceStage,
    pub composition: StageComposition,
    pub budget: StageBudget,
}
```

---

# 37. Critical Path

The stages controlling completion.

---

# 38. Hard rule.

---

# 39. Example: Message Send

```text
composer action
→ local persist/outbox
→ encrypt
→ transport queue
→ network
→ server/mailbox commit
→ delivery ACK
```

---

# 40. Persist-Before-Send Requirement

Performance optimization cannot bypass durability semantics.

---

# 41. Hard rule.

---

# 42. Example: Local Message Open

```text
SQLite query
→ decrypt
→ model conversion
→ Dioxus/Compose render
```

---

# 43. Local-first path should remain fast without network.

---

# 44. Hard rule.

---

# 45. Example: Anonymous Send

```text
local persist
→ E2EE
→ framing
→ mix queue
→ Sphinx processing
→ mix delays
→ mailbox/provider
```

---

# 46. Mix Delay Is Intentional

Hard rule.

---

# 47. Do Not Optimize Away Anonymity Delay

Hard rule.

---

# 48. User-Visible Performance Modes

May expose:

```text
Standard
Private
Anonymous
MaximumAnonymity
```

---

# 49. Different latency expectations.

---

# 50. Hard rule.

---

# 51. Privacy Routing Performance Budget

```rust
pub struct PrivacyModePerformanceBudget {
    pub mode: PrivacyRoutingMode,
    pub expected_latency_class: LatencyClass,
    pub throughput_class: ThroughputClass,
}
```

---

# 52. No silent route downgrade for speed.

---

# 53. Hard rule.

---

# 54. Service Time

Actual processing time excluding queue delay.

```rust
pub struct ServiceTimeSample {
    pub stage: PerformanceStage,
    pub duration: Duration,
}
```

---

# 55. Queue Time

Waiting before execution.

```rust
pub struct QueueTimeSample {
    pub queue: QueueClass,
    pub duration: Duration,
}
```

---

# 56. Separate Service Time From Queue Time

Hard rule.

---

# 57. Why

Optimization differs:

```text
high service time → optimize work
high queue time → capacity/admission/scheduling
```

---

# 58. Good.

---

# 59. Queueing Model

Near saturation, queue latency rises nonlinearly.

---

# 60. Hard rule.

---

# 61. Queue Metrics

Track:

```text
arrival rate
service rate
depth
age
```

---

# 62. Queue Observation

```rust
pub struct PerformanceQueueObservation {
    pub arrival_rate: FixedPoint,
    pub service_rate: FixedPoint,
    pub depth: usize,
    pub oldest_age: Duration,
}
```

---

# 63. No User Queue Dimension

Hard rule.

---

# 64. Queue Utilization

```text
arrival/service relationship
```

---

# 65. If arrival >= service for sustained period:

```text
backlog grows
```

---

# 66. Hard rule.

---

# 67. Queue Age SLO

Often more useful than depth alone.

---

# 68. Good.

---

# 69. Throughput Modeling

Throughput is limited by bottleneck resource.

---

# 70. Throughput Resource

```rust
pub enum ThroughputResource {
    Cpu,
    MemoryBandwidth,
    DiskIo,
    Network,
    DatabaseConnections,
    QueueWorkers,
    CryptoWorkers,
    HsmOperations,
    RelayBandwidth,
}
```

---

# 71. Throughput Ceiling

```rust
pub struct ThroughputCeiling {
    pub resource: ThroughputResource,
    pub sustained: FixedPoint,
    pub burst: FixedPoint,
    pub confidence: BenchmarkConfidence,
}
```

---

# 72. Benchmark Confidence

```rust
pub enum BenchmarkConfidence {
    Synthetic,
    Representative,
    ProductionValidated,
}
```

---

# 73. Synthetic ≠ ProductionValidated

Hard rule.

---

# 74. Bottleneck Identification

Compare resource ceilings.

---

# 75. First limiting resource defines current ceiling.

---

# 76. Hard rule.

---

# 77. Throughput Under Failure

Part 110.

---

# 78. Need post-failure throughput ceiling.

---

# 79. Hard rule.

---

# 80. Concurrency

More concurrency can increase throughput until contention dominates.

---

# 81. Concurrency Budget

```rust
pub struct ConcurrencyBudget {
    pub stage: PerformanceStage,
    pub minimum: u32,
    pub target: u32,
    pub maximum: u32,
}
```

---

# 82. Hard Maximum

Hard rule.

---

# 83. No Unbounded Tokio Spawn

Hard rule.

---

# 84. Worker Pools

Bounded per class.

---

# 85. Separate pools for:

```text
realtime
control
message
bulk
background
```

---

# 86. Part 101 integration.

---

# 87. Hard rule.

---

# 88. Async vs Blocking

Blocking workloads use bounded blocking pool.

---

# 89. Hard rule.

---

# 90. Lock Contention

Measure.

---

# 91. Rust design preference:

```text
ownership
message passing
small critical sections
sharded locks
```

---

# 92. Hard rule.

---

# 93. Lock Wait Time

```rust
pub struct LockWaitSample {
    pub lock_class: LockClass,
    pub wait: Duration,
}
```

---

# 94. No per-user lock key telemetry.

---

# 95. Hard rule.

---

# 96. Database Performance Budget

Stages:

```text
pool wait
query
commit
fsync
```

---

# 97. Separate.

---

# 98. Hard rule.

---

# 99. Database Pool Wait

Often tail latency source.

---

# 100. Track p95/p99.

---

# 101. Hard rule.

---

# 102. Query Budget

```rust
pub struct QueryPerformanceBudget {
    pub class: QueryClass,
    pub latency: LatencyTarget,
    pub max_rows: Option<u64>,
}
```

---

# 103. No arbitrary unbounded query in request path.

---

# 104. Hard rule.

---

# 105. Database Transaction Duration

Bound.

---

# 106. Avoid long-lived transactions.

---

# 107. Hard rule.

---

# 108. Local SQLite Performance

Client authoritative local metadata/history.

---

# 109. Use:

```text
prepared statements
WAL
indexes
bounded transactions
pagination
```

---

# 110. Hard rule.

---

# 111. Search Performance

Local search:

```text
FTS index
bounded result set
virtualized UI
```

---

# 112. Remote private query fanout not baseline.

---

# 113. Hard rule.

---

# 114. Object Storage Performance

Stages:

```text
encrypt
chunk
upload
store
download
verify
decrypt
```

---

# 115. Chunking Enables Parallelism

Bounded.

---

# 116. Hard rule.

---

# 117. Attachment Throughput

Low priority relative messages.

---

# 118. No attachment transfer saturating realtime/control.

---

# 119. Hard rule.

---

# 120. Crypto Performance

Measure by primitive class.

```rust
pub enum CryptoOperationClass {
    Hash,
    SignatureVerify,
    SignatureCreate,
    Kdf,
    AeadEncrypt,
    AeadDecrypt,
    KeyExchange,
}
```

---

# 121. Crypto Must Not Be Disabled For Speed

Hard rule.

---

# 122. Crypto Pool

Bounded.

---

# 123. Security reserve.

---

# 124. Hard rule.

---

# 125. HSM Performance

Measure queue + operation separately.

---

# 126. No software-key fallback under HSM latency.

---

# 127. Hard rule.

---

# 128. Network Latency Decomposition

Separate:

```text
DNS/service resolution
connection setup
TLS/QUIC
relay traversal
network RTT
application ACK
```

---

# 129. Hard rule.

---

# 130. QUIC Connection Reuse

Preferred.

---

# 131. Avoid repeated handshake overhead.

---

# 132. Hard rule.

---

# 133. Multipath Performance

Part 12.

---

# 134. Measure per path class, not per user.

---

# 135. Hard rule.

---

# 136. Relay Performance

Metrics:

```text
connection setup
queue
packet processing
bandwidth
```

---

# 137. No flow history retention.

---

# 138. Hard rule.

---

# 139. Mixnet Performance

Latency includes intentional stochastic delay.

---

# 140. Measure distributions, not source-destination paths.

---

# 141. Hard rule.

---

# 142. Mixnet Latency Metric

```rust
pub struct MixLatencyAggregate {
    pub layer_count: u8,
    pub latency_histogram: BoundedHistogram,
}
```

---

# 143. No end-to-end correlated trace ID.

---

# 144. Hard rule.

---

# 145. Cover Traffic Performance

Consumes capacity.

---

# 146. It is privacy overhead, not waste.

---

# 147. Hard rule.

---

# 148. Anonymous Mailbox Performance

Metrics:

```text
store latency
fetch latency
queue age
```

---

# 149. No stable mailbox-owner label.

---

# 150. Hard rule.

---

# 151. Realtime Call Performance

Budgets:

```text
signaling
call setup
jitter
packet loss
audio pipeline
video pipeline
```

---

# 152. Hard rule.

---

# 153. Call Setup Budget

```rust
pub struct CallSetupBudget {
    pub signaling: Duration,
    pub relay_setup: Duration,
    pub media_start: Duration,
}
```

---

# 154. No direct-route fallback if privacy policy forbids.

---

# 155. Hard rule.

---

# 156. Audio Pipeline Budget

Stages:

```text
capture
AEC/NS/AGC
encode
transport
jitter buffer
decode
render
```

---

# 157. Tail jitter matters.

---

# 158. Hard rule.

---

# 159. Video Pipeline Budget

Stages:

```text
capture
encode
network
decode
render
```

---

# 160. Hardware acceleration where available.

---

# 161. No privacy compromise.

---

# 162. Hard rule.

---

# 163. UI Performance

Desktop/mobile.

---

# 164. Budgets:

```text
input-to-feedback
list scroll frame time
conversation open
search results
```

---

# 165. No network dependency for local data path.

---

# 166. Hard rule.

---

# 167. Frame Budget

Platform-specific.

```rust
pub struct FrameBudget {
    pub target_frame_time: Duration,
    pub max_main_thread_block: Duration,
}
```

---

# 168. Avoid giant UI state updates.

---

# 169. Hard rule.

---

# 170. Virtualization

Large histories/files must virtualize.

---

# 171. Part UI/UX 26 integration conceptually.

---

# 172. Hard rule.

---

# 173. JNI/FFI Performance

Android path:

```text
Compose
→ ViewModel
→ Rust bridge
```

---

# 174. Avoid giant copied byte arrays.

---

# 175. Use handles/FDs/chunks.

---

# 176. Hard rule.

---

# 177. Desktop Dioxus Performance

Rust-native presenter/core path.

---

# 178. Avoid serializing large in-memory models unnecessarily.

---

# 179. Hard rule.

---

# 180. Serialization Budget

Postcard preferred internal binary.

---

# 181. RON human config.

---

# 182. JSON external only.

---

# 183. Hard rule.

---

# 184. Serialization Metric

```rust
pub struct SerializationBudget {
    pub max_payload: usize,
    pub p99_encode: Duration,
    pub p99_decode: Duration,
}
```

---

# 185. Bounded payloads.

---

# 186. Hard rule.

---

# 187. Compression

Can improve network throughput.

---

# 188. But adds CPU/latency.

---

# 189. Use by payload class.

---

# 190. Hard rule.

---

# 191. Compression Bomb Protection

Bound decompression ratio/output size.

---

# 192. Part security integration.

---

# 193. Hard rule.

---

# 194. Tail Latency

Tail often dominates user experience and SLOs.

---

# 195. Tail Sources

```text
queueing
lock contention
GC-like allocator pressure
slow dependency
retry
cold cache
connection setup
disk fsync
```

---

# 196. Rust avoids GC but not allocator/contention stalls.

---

# 197. Hard rule.

---

# 198. Tail-Latency Policy

```rust
pub struct TailLatencyPolicy {
    pub p95_limit: Duration,
    pub p99_limit: Duration,
    pub max_outlier_rate: FixedPoint,
}
```

---

# 199. Outlier Rate

Explicit.

---

# 200. Hard rule.

---

# 201. Tail Control Techniques

Use:

```text
bounded queues
admission control
hedging only where safe
timeouts
cancellation
bulkheads
connection reuse
cache
```

---

# 202. No indiscriminate retries.

---

# 203. Hard rule.

---

# 204. Hedged Requests

Potential tail reduction.

---

# 205. Only for:

```text
idempotent safe reads
```

---

# 206. Hard rule.

---

# 207. No Hedging For

```text
non-idempotent writes
financial mutations
message sends without idempotency
```

---

# 208. Hard rule.

---

# 209. Hedging Budget

```rust
pub struct HedgingPolicy {
    pub delay: Duration,
    pub max_extra_requests: u8,
}
```

---

# 210. Bounded.

---

# 211. Hard rule.

---

# 212. Timeout Architecture

Timeouts typed by stage.

---

# 213. Deadline Propagation

Overall deadline passed downstream.

---

# 214. Hard rule.

---

# 215. Deadline Budget

```rust
pub struct DeadlineBudget {
    pub total: Duration,
    pub stages: BTreeMap<PerformanceStage, Duration>,
}
```

---

# 216. No child timeout > remaining parent budget.

---

# 217. Hard rule.

---

# 218. Cancellation

Expensive work should be cancellable.

---

# 219. Cancel releases capacity.

---

# 220. Hard rule.

---

# 221. Retry Budget

Part 101.

---

# 222. Retry adds latency and load.

---

# 223. Hard rule.

---

# 224. Retry-Aware Latency

Performance model includes retries.

---

# 225. No "single-attempt" benchmark only.

---

# 226. Hard rule.

---

# 227. Backpressure

Prefer controlled latency/defer over collapse.

---

# 228. Hard rule.

---

# 229. Brownout Performance

Optional features disabled to protect core latency.

---

# 230. Hard rule.

---

# 231. Load Shedding

Under overload:

```text
preserve security/control/messages
defer bulk/background
```

---

# 232. Hard rule.

---

# 233. Cache Performance

Cache is optimization, not authority.

---

# 234. Cache hit/miss separated.

---

# 235. Hard rule.

---

# 236. Cache Latency Budget

```rust
pub struct CacheBudget {
    pub p99_hit: Duration,
    pub miss_fallback: Duration,
}
```

---

# 237. Stale cache semantics explicit.

---

# 238. Hard rule.

---

# 239. Cache Stampede

Use request coalescing/bounded refresh.

---

# 240. Hard rule.

---

# 241. Cold Start

Measure:

```text
process startup
JIT? none
DB open
index warmup
connection establish
```

---

# 242. Rust binaries no runtime JIT baseline.

---

# 243. Good.

---

# 244. Startup Budget

```rust
pub struct StartupPerformanceBudget {
    pub daemon_ready: Duration,
    pub ui_interactive: Duration,
}
```

---

# 245. Staged startup.

---

# 246. Hard rule.

---

# 247. Idle Performance

Mobile/desktop idle CPU near zero.

---

# 248. Hard rule.

---

# 249. Idle Budget

```rust
pub struct IdleBudget {
    pub average_cpu_percent: FixedPoint,
    pub wakeups_per_minute: u32,
}
```

---

# 250. Important for battery.

---

# 251. Hard rule.

---

# 252. Battery-Aware Performance

Part 13.

---

# 253. Background work slower/deferred on battery constraints.

---

# 254. No correctness/security downgrade.

---

# 255. Hard rule.

---

# 256. Memory Performance

Bound memory by active working set, not history size.

---

# 257. Hard rule.

---

# 258. Memory Budget

```rust
pub struct MemoryBudget {
    pub steady_state_bytes: u64,
    pub peak_bytes: u64,
}
```

---

# 259. Per platform/service class.

---

# 260. Hard rule.

---

# 261. Allocation Pressure

Measure large allocation rate.

---

# 262. Avoid cloning giant payloads.

---

# 263. Hard rule.

---

# 264. Zero-Copy

Use where it materially helps and preserves safety.

---

# 265. No unsafe complexity solely for microbenchmark.

---

# 266. Hard rule.

---

# 267. I/O Performance

Batch where safe.

---

# 268. Avoid giant batch causing tail spikes.

---

# 269. Hard rule.

---

# 270. Fsync Semantics

Durability-critical path may require fsync.

---

# 271. Do not remove durability for latency benchmark.

---

# 272. Hard rule.

---

# 273. Benchmark Architecture

Benchmark at multiple levels:

```text
micro
component
service
end-to-end
failure-mode
```

---

# 274. Hard rule.

---

# 275. Benchmark Class

```rust
pub enum BenchmarkClass {
    Micro,
    Component,
    Service,
    EndToEnd,
    FailureMode,
}
```

---

# 276. Microbenchmark

Useful for primitives.

---

# 277. Cannot prove system performance.

---

# 278. Hard rule.

---

# 279. Representative Workload

Define workload mix.

---

# 280. Example:

```text
80% text messages
15% small media metadata
5% attachments
```

---

# 281. Must be labeled scenario, not user profiling.

---

# 282. Hard rule.

---

# 283. Workload Model

```rust
pub struct PerformanceWorkloadModel {
    pub classes: Vec<WorkloadClassWeight>,
    pub concurrency: u32,
    pub duration: Duration,
}
```

---

# 284. Synthetic/aggregate.

---

# 285. Hard rule.

---

# 286. Benchmark Environment

Record:

```text
hardware
OS
kernel
artifact
config
database version
network class
```

---

# 287. Hard rule.

---

# 288. Benchmark Reproducibility

Use exact artifact/config.

---

# 289. Part 73/99.

---

# 290. Hard rule.

---

# 291. Warm vs Cold

Measure both where relevant.

---

# 292. Hard rule.

---

# 293. Benchmark Noise

Repeat runs.

---

# 294. Report variance/confidence.

---

# 295. Hard rule.

---

# 296. Performance Regression

Compare candidate vs baseline.

---

# 297. Regression Rule

```rust
pub struct PerformanceRegressionRule {
    pub metric: PerformanceMetricId,
    pub max_regression: FixedPoint,
    pub severity: RegressionSeverity,
}
```

---

# 298. Regression Severity

```rust
pub enum RegressionSeverity {
    Warning,
    Blocking,
}
```

---

# 299. Release Gate

Part 108.

---

# 300. Blocking regression prevents certification unless approved exception.

---

# 301. Hard rule.

---

# 302. No Single Number Regression

Evaluate:

```text
p50
p95
p99
throughput
memory
CPU
```

---

# 303. Hard rule.

---

# 304. Performance Baseline

Versioned.

---

# 305. Historical baseline tied to release.

---

# 306. Hard rule.

---

# 307. Performance Budget Drift

Budget itself changes only by governed policy.

---

# 308. No loosen target after regression silently.

---

# 309. Hard rule.

---

# 310. Performance Change Impact

Part 107.

---

# 311. Large regression included in change impact.

---

# 312. Hard rule.

---

# 313. Capacity Forecast Integration

Part 111.

---

# 314. Per-operation resource cost informs capacity model.

---

# 315. Hard rule.

---

# 316. SLO Integration

Part 109.

---

# 317. Performance budgets should support latency SLOs.

---

# 318. Hard rule.

---

# 319. Resilience Integration

Part 110.

---

# 320. Benchmark performance under degraded/failure state.

---

# 321. Hard rule.

---

# 322. DR Performance

Recovery capacity must handle target load.

---

# 323. Hard rule.

---

# 324. Regional Performance

Measure by infrastructure region.

---

# 325. No precise user geolocation.

---

# 326. Hard rule.

---

# 327. Residency Constraints

Cannot move workload to faster forbidden region.

---

# 328. Hard rule.

---

# 329. Federation Performance

Peer-level aggregate latency/throughput.

---

# 330. No remote user labels.

---

# 331. Hard rule.

---

# 332. FinOps Integration

Part 102.

---

# 333. Cost/performance tradeoff allowed within hard constraints.

---

# 334. Hard rule.

---

# 335. Performance Efficiency

Useful ratios:

```text
requests per CPU-second
bytes per CPU-second
messages per DB transaction
```

---

# 336. Aggregate.

---

# 337. Hard rule.

---

# 338. Efficiency Metric

```rust
pub struct EfficiencyMetric {
    pub numerator: PerformanceQuantity,
    pub denominator: ResourceQuantity,
}
```

---

# 339. No user value metric.

---

# 340. Hard rule.

---

# 341. Privacy Performance Cost

Model overhead of:

```text
E2EE
mixnet
cover traffic
relay-only media
```

---

# 342. Do not classify as removable waste.

---

# 343. Hard rule.

---

# 344. Security Performance Cost

Model:

```text
signature verification
attestation
authorization
HSM
```

---

# 345. Security operations remain mandatory.

---

# 346. Hard rule.

---

# 347. Privacy/Security Optimization

Optimize implementation, not guarantee.

---

# 348. Hard rule.

---

# 349. Performance Telemetry

Safe dimensions:

```text
service
region
stage
work class
transport class
platform class
```

---

# 350. Forbidden:

```text
stable user ID
contact
message content
social graph
```

---

# 351. Hard rule.

---

# 352. Performance Span

Distributed tracing can be privacy-sensitive.

---

# 353. Use stage-local correlation.

---

# 354. Avoid universal end-to-end trace ID across anonymity boundaries.

---

# 355. Hard rule.

---

# 356. Trace Boundary

```rust
pub enum TraceBoundary {
    LocalService,
    TrustedServiceDomain,
    AnonymousBoundary,
    FederationBoundary,
}
```

---

# 357. Crossing AnonymousBoundary breaks trace continuity.

---

# 358. Hard rule.

---

# 359. Histograms

Bounded latency histograms.

---

# 360. Avoid raw request log retention.

---

# 361. Hard rule.

---

# 362. Metric Cardinality

Strict limits.

---

# 363. No path/user-generated labels.

---

# 364. Hard rule.

---

# 365. Sampling

Performance traces may be sampled.

---

# 366. Security/privacy events separate.

---

# 367. Hard rule.

---

# 368. Client Performance Telemetry

Prefer local diagnostics.

---

# 369. Optional aggregate upload.

---

# 370. No device fingerprint.

---

# 371. Hard rule.

---

# 372. Performance Decision Service

```rust
pub trait PerformanceBudgetService {
    fn budget(
        &self,
        scope: PerformanceScope,
    ) -> Result<EndToEndPerformanceBudget, PerformanceError>;

    fn evaluate(
        &self,
        scope: PerformanceScope,
    ) -> Result<PerformanceEvaluation, PerformanceError>;
}
```

---

# 373. Performance Evaluation

```rust
pub struct PerformanceEvaluation {
    pub scope: PerformanceScope,
    pub latency: LatencyEvaluation,
    pub throughput: ThroughputEvaluation,
    pub regressions: Vec<PerformanceRegression>,
}
```

---

# 374. Stage Analyzer

```rust
pub trait LatencyDecompositionService {
    fn decompose(
        &self,
        path: PerformancePathId,
    ) -> Result<Vec<StageLatencySummary>, PerformanceError>;
}
```

---

# 375. Throughput Model Service

```rust
pub trait ThroughputModelService {
    fn ceiling(
        &self,
        scope: PerformanceScope,
    ) -> Result<Vec<ThroughputCeiling>, PerformanceError>;
}
```

---

# 376. Regression Gate

```rust
pub trait PerformanceRegressionGate {
    fn evaluate_release(
        &self,
        release: ReleaseId,
    ) -> Result<GateResult, PerformanceError>;
}
```

---

# 377. No User Behavior Input

Hard rule.

---

# 378. Performance Error Taxonomy

```rust
pub enum PerformanceError {
    BudgetMissing,
    TelemetryMissing,
    BenchmarkInvalid,
    PathUnknown,
    ThroughputUnknown,
    RegressionDetected,
    ScopeViolation,
    Unauthorized,
    Internal,
}
```

---

# 379. Observability

Safe metrics:

```text
p50/p95/p99 latency
queue delay
service time
throughput
concurrency
CPU/memory efficiency
```

---

# 380. No user-level history.

---

# 381. Hard rule.

---

# 382. Performance SLOs

Examples:

```text
message local persist p99
message submission p99
call setup p95
search local p95
```

---

# 383. Reliability integration.

---

# 384. Hard rule.

---

# 385. Performance Governance SLO

Meta:

```text
benchmark freshness
regression-gate availability
latency histogram freshness
```

---

# 386. Hard rule.

---

# 387. Failure Modes

```text
benchmark environment drift
telemetry loss
load-generator mismatch
hidden bottleneck
performance cliff
```

---

# 388. Environment Drift

Benchmark incomparable.

---

# 389. Mark invalid.

---

# 390. Hard rule.

---

# 391. Telemetry Loss

Evaluation Unknown.

---

# 392. No green default.

---

# 393. Hard rule.

---

# 394. Workload Mismatch

Benchmark not representative.

---

# 395. Confidence reduced.

---

# 396. Hard rule.

---

# 397. Hidden Bottleneck

Stage decomposition/instrumentation update.

---

# 398. Hard rule.

---

# 399. Performance Cliff

Nonlinear collapse near saturation.

---

# 400. Capacity policy should stay away from cliff.

---

# 401. Hard rule.

---

# 402. Testing

Need performance-engineering testkit.

---

# 403. Test Scenarios

```text
low load
target load
burst
near saturation
failure mode
```

---

# 404. Latency Decomposition Test

Stage totals/path semantics correct.

---

# 405. Queue Test

Queue delay separated from service time.

---

# 406. Tail Test

p99 regression detected even if average improves.

---

# 407. Throughput Test

Bottleneck resource identified.

---

# 408. Concurrency Test

Beyond maximum cannot create unbounded work.

---

# 409. Deadline Test

Child timeout never exceeds parent remaining budget.

---

# 410. Retry Test

Retries included in end-to-end latency.

---

# 411. Hedging Test

Non-idempotent writes never hedge.

---

# 412. Mixnet Test

No trace continuity across anonymous boundary.

---

# 413. Privacy Test

No user ID in performance labels.

---

# 414. Region Test

Faster forbidden region never selected.

---

# 415. Security Test

Crypto/HSM not bypassed for performance.

---

# 416. Release Test

Blocking regression stops certification.

---

# 417. Failure Test

Performance measured after N-1 capacity loss.

---

# 418. UI Test

Large history remains virtualized.

---

# 419. Memory Test

History size does not linearly grow active UI memory.

---

# 420. Serialization Test

Payload bounds enforced.

---

# 421. Fuzzing

Fuzz:

```text
performance budget
deadline budget
regression rules
throughput model
trace boundary
```

---

# 422. Property Tests

Properties:

```text
anonymous-boundary traces can never preserve a universal end-to-end trace identifier
non-idempotent operation can never use hedged execution
performance optimization can never disable mandatory crypto/privacy controls
bounded-concurrency policies can never admit more than configured hard maximum
```

---

# 423. Formal Verification Targets

Strong candidates:

```text
deadline propagation
bounded concurrency
queue admission
regression gate semantics
```

---

# 424. Kani Candidate

deadline/concurrency/budget invariants.

---

# 425. TLA+ Candidate

arrival → queue → execute → backpressure → recover under saturation.

---

# 426. Loom Candidate

concurrent cancellation + worker completion + capacity lease release.

---

# 427. Performance

Performance subsystem itself must be cheap.

---

# 428. Prefer histograms/counters.

---

# 429. Avoid per-request persistence.

---

# 430. Hard rule.

---

# 431. Benchmark Pipeline

Runs outside request path.

---

# 432. Hard rule.

---

# 433. Stage Instrumentation

Compile-time feature / lightweight runtime.

---

# 434. No high-cardinality labels.

---

# 435. Hard rule.

---

# 436. Storage

Separate:

```text
performance budgets
benchmark results
regression baselines
aggregate latency histograms
throughput ceilings
```

---

# 437. No raw user request archive.

---

# 438. Hard rule.

---

# 439. Partitioning

By:

```text
service
capability
region
work class
transport
platform
federation peer
```

---

# 440. No user partition.

---

# 441. Hard rule.

---

# 442. Crate Layout

Recommended:

```text
crates/
├── siar-performance-core/
├── siar-performance-budget/
├── siar-latency-decomposition/
├── siar-queue-performance/
├── siar-throughput-model/
├── siar-tail-latency/
├── siar-performance-benchmark/
├── siar-performance-regression/
├── siar-performance-telemetry/
├── siar-performance-governance/
└── siar-performance-testkit/
```

---

# 443. `siar-performance-core`

Owns:

```text
PerformanceScope
PerformanceClass
PerformanceStage
errors
```

---

# 444. `siar-performance-budget`

End-to-end/stage budgets/deadlines.

---

# 445. `siar-latency-decomposition`

Critical path/stage summaries.

---

# 446. `siar-queue-performance`

arrival/service/depth/age modeling.

---

# 447. `siar-throughput-model`

resource bottleneck/ceilings.

---

# 448. `siar-tail-latency`

tail policies/timeouts/cancellation/hedging.

---

# 449. `siar-performance-benchmark`

micro/component/service/E2E/failure benchmarks.

---

# 450. `siar-performance-regression`

release baseline/regression gates.

---

# 451. `siar-performance-telemetry`

bounded aggregate histograms/counters.

---

# 452. `siar-performance-governance`

budget versioning/policy/change integration.

---

# 453. `siar-performance-testkit`

latency/throughput/privacy/concurrency tests.

---

# 454. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. Performance engineering uses service, capability, region, work-class, transport, platform, and federation-level technical signals—not stable user identities, private content, social graphs, or behavioral profiles.
2. End-to-end latency is decomposed into explicit stages with queue time, service time, network time, storage time, crypto time, and intentional anonymity delay kept semantically distinct.
3. Performance targets use tail distributions such as p95/p99 and cannot be represented solely by averages that hide queueing or outlier collapse.
4. Every queue, worker pool, blocking pool, crypto pool, concurrency limiter, retry path, and hedging mechanism remains explicitly bounded; throughput optimization cannot rely on unbounded parallelism.
5. Performance optimization can never disable or weaken durability, authorization, encryption, anonymity, tenant isolation, residency, HSM use, or other hard security/privacy guarantees.
6. Anonymous/mixnet performance measurements preserve anonymity boundaries and never introduce universal end-to-end trace IDs, source-destination correlation, or stable mailbox/user identifiers.
7. Throughput models identify the limiting technical resource and include post-failure capacity where required; nominal peak benchmark numbers cannot substitute for sustained safe throughput.
8. Tail-latency controls such as retries, hedging, timeouts, cancellation, caching, and load shedding are operation-aware; unsafe duplication of non-idempotent work is forbidden.
9. Performance regressions are evaluated against exact artifact/config/environment baselines and cannot be hidden by improved averages if p95/p99, memory, CPU, or correctness-relevant metrics degrade materially.
10. Performance telemetry is bounded-cardinality and aggregate, with no raw request archive or user/device behavioral history used as a performance optimization substrate.
11. Cost or latency pressure cannot cause routing into forbidden regions, direct-path privacy downgrade, software-key fallback, or bypass of security controls.
12. Performance engineering integrates with SLOs, capacity forecasting, resilience, DR, change impact, release certification, desired state, inventory, FinOps, residency, and security/privacy governance without creating an alternate path around hard invariants.
```

---

# 455. Initial Production Scope

Implement first:

```text
typed PerformanceScope/PerformanceClass/PerformanceStage
end-to-end latency budgets
p50/p95/p99 targets
stage decomposition
queue vs service-time measurement
bounded latency histograms
throughput ceilings
concurrency budgets
deadline propagation
timeouts/cancellation
retry-aware latency
database pool/query budgets
serialization budgets
local UI startup/frame/interaction budgets
mixnet latency aggregates without trace correlation
realtime call setup/audio/video budgets
micro/component/service/E2E benchmarks
exact environment recording
performance baselines/regression gates
failure-mode benchmarks
privacy-safe performance dashboards
performance testkit
```

Then add:

```text
automated critical-path extraction
advanced queueing simulation
hardware-counter profiling
cross-platform performance laboratory
adaptive concurrency controllers
formal deadline/concurrency verification
privacy-safe continuous performance regression detection
```

---

# 456. Definition of Done

Part 112 is complete when:

- end-to-end and per-stage performance budgets exist
- p95/p99 are first-class
- queue and service time are separated
- sustained/burst throughput ceilings are measured
- concurrency and worker pools are bounded
- deadlines propagate correctly
- retries/hedging are operation-safe
- mixnet/anonymous boundaries break universal tracing
- performance optimizations cannot weaken security/privacy/durability
- benchmarks use exact artifact/config/environment
- blocking regressions gate releases
- failure-mode performance is measured
- UI/local-first paths have explicit budgets
- telemetry contains no user behavioral dimensions
- latency/throughput/privacy/fuzz/formal tests are specified

---

# 457. Final Architecture

```text
                OPERATION / CAPABILITY
                         │
                         ▼
               END-TO-END PERFORMANCE BUDGET
                         │
                         ▼
                  STAGE DECOMPOSITION
                         │
          ┌──────────────┼──────────────┐
          │              │              │
        QUEUE          SERVICE       NETWORK
          │              │              │
          └──────────────┼──────────────┘
                         ▼
                  CRITICAL PATH
                         │
                         ▼
               THROUGHPUT / SATURATION
                         │
                         ▼
                 TAIL-LATENCY CONTROL
                         │
                         ▼
               BENCHMARK / REGRESSION GATE
```

Performance-engineering safety model:

```text
explicit budgets
+
stage decomposition
+
tail-aware targets
+
bounded concurrency
+
throughput ceilings
+
deadline/cancellation discipline
+
privacy-safe telemetry
+
release regression gates
```

not:

```text
optimize average latency, spawn more work, trace every user end-to-end, and remove security/privacy overhead when benchmarks look slow
```

---

# 458. Final Principle

Performance is the disciplined allocation of latency, throughput, concurrency, memory, and I/O budgets across a critical path—not the pursuit of the smallest benchmark number at any cost.

The correct model is:

```text
budget the whole path
+
measure every meaningful stage
+
separate queueing from work
+
control concurrency
+
protect the tail
+
benchmark realistically
+
gate regressions
+
preserve privacy/security even when they cost performance
```

This architecture gives SIAR a privacy-preserving performance-engineering foundation for latency budgets, throughput ceilings, queueing control, tail-latency management, benchmarking, release regression gates, local-first responsiveness, realtime media, and anonymity-aware performance while preserving the anonymity, local-first, least-authority, and anti-surveillance guarantees established across Parts 34–111.
