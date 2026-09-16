# Core System Architecture Part 63 — Anonymous Network Performance Engineering, Benchmarking, Capacity Planning, Resource Isolation & Production Sizing Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 63  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 34–62  

**Primary purpose:** define how SIAR measures, models, budgets, isolates, sizes, scales, and continuously validates performance across mix nodes, gateways, mailboxes, relays, bulk providers, bridges, control-plane services, federation links, and client-facing infrastructure without weakening anonymity or creating per-user observability.

---

# 1. Purpose

Performance is part of correctness.

If an anonymous network becomes overloaded:

```text
queues grow
latency spikes
timeouts increase
cover traffic collapses
mailboxes lag
relays reject calls
operators disable privacy features
```

The last outcome is unacceptable.

The governing principle is:

> **SIAR must achieve predictable performance through explicit capacity engineering, backpressure, isolation, and headroom—not by silently weakening privacy under load.**

---

# 2. Architectural Position

```text
Workload Model
      │
      ▼
Benchmark Suite
      │
      ▼
Capacity Envelope
      │
      ▼
Sizing / Placement
      │
      ▼
Runtime Isolation
      │
      ▼
SLO / Regression Gates
```

---

# 3. Core Separation

Keep distinct:

```text
microbenchmark
component benchmark
end-to-end benchmark
capacity test
stress test
soak test
chaos test
production telemetry
```

---

# 4. Non-Goals

Part 63 does not allow:

```text
benchmarking on raw user payloads
user-specific performance profiles
routing based on individual wealth
turning off padding/cover silently
unbounded queues for higher throughput
```

---

# 5. Performance Domains

```rust
pub enum PerformanceDomain {
    MixNode,
    Gateway,
    Mailbox,
    RealtimeRelay,
    BulkProvider,
    Bridge,
    Directory,
    ControlPlane,
    Federation,
    ClientSync,
}
```

---

# 6. Core Metrics

```text
throughput
latency
tail latency
CPU
memory
storage IOPS
network throughput
queue depth
error rate
saturation
```

---

# 7. Tail Latency

Track:

```text
p50
p90
p95
p99
p99.9
```

where meaningful.

---

# 8. Why Tail Latency Matters

Anonymous routing includes:

```text
mix delays
multi-hop paths
remote mailbox writes
cross-region links
```

Average latency can hide failure.

---

# 9. Service Budgets

Every service defines:

```text
latency budget
throughput budget
memory budget
queue budget
error budget
```

---

# 10. Performance Budget Type

```rust
pub struct PerformanceBudget {
    pub max_p95_latency: Duration,
    pub max_p99_latency: Duration,
    pub min_throughput: u64,
    pub max_memory_bytes: u64,
    pub max_queue_depth: u64,
}
```

---

# 11. Workload Model

Do not benchmark meaningless synthetic loops.

---

# 12. Workload Dimensions

```text
message size
attachment size
concurrency
session duration
queue depth
provider count
region count
network loss
latency
```

---

# 13. Workload Profile

```rust
pub struct WorkloadProfile {
    pub message_rate: u64,
    pub average_message_bytes: u64,
    pub concurrent_sessions: u64,
    pub attachment_rate: u64,
    pub network_profile: NetworkProfile,
}
```

---

# 14. Network Profile

```rust
pub struct NetworkProfile {
    pub rtt: Duration,
    pub packet_loss_ppm: u32,
    pub bandwidth_bps: u64,
    pub jitter: Duration,
}
```

---

# 15. Synthetic Workloads

Required.

---

# 16. No Production User Replay

Hard rule.

---

# 17. Privacy-Preserving Replay

If production-derived distributions are used:

```text
aggregate
coarsen
anonymize
```

---

# 18. Benchmark Classes

```rust
pub enum BenchmarkClass {
    Micro,
    Component,
    Integration,
    EndToEnd,
    Capacity,
    Stress,
    Soak,
}
```

---

# 19. Microbenchmark

Examples:

```text
Postcard encode/decode
AEAD
queue push/pop
replay lookup
```

---

# 20. Component Benchmark

Examples:

```text
mailbox write path
mix packet forwarding
relay packet fanout
```

---

# 21. Integration Benchmark

Multiple services.

---

# 22. End-to-End Benchmark

Synthetic user path.

---

# 23. Capacity Benchmark

Find sustainable load.

---

# 24. Stress Test

Find failure behavior beyond capacity.

---

# 25. Soak Test

Long-duration resource stability.

---

# 26. Criterion

Recommended for Rust microbenchmarks.

---

# 27. Flamegraph

Useful locally/staging.

---

# 28. `perf`

Useful on Linux.

---

# 29. eBPF

Infrastructure profiling option.

---

# 30. eBPF Privacy Rule

Do not instrument user payloads/identifiers.

---

# 31. Production Profiling

Infrastructure only, sampled.

---

# 32. Mix Node Performance

Critical metrics:

```text
cells/s
crypto cost/cell
delay-queue operations
memory/cell
network utilization
```

---

# 33. Mix Throughput

```rust
pub struct MixNodeCapacity {
    pub cells_per_second: u64,
    pub max_delay_queue: u64,
    pub sustained_network_bps: u64,
}
```

---

# 34. Delay Queue

Must remain bounded.

---

# 35. Queue Saturation

Should trigger:

```text
admission control
load shedding of cover/background
```

not real-message corruption.

---

# 36. Cover Traffic

Performance budget must include it.

---

# 37. Hard Rule

Do not size infrastructure only for real traffic and treat cover as optional.

---

# 38. Gateway Performance

Metrics:

```text
handshakes/s
active clients
packet ingress
descriptor lookup
```

---

# 39. Mailbox Performance

Metrics:

```text
deposits/s
fetches/s
storage write latency
object count
expiry cleanup
```

---

# 40. Mailbox Storage Model

Need:

```text
write amplification
index amplification
tombstone overhead
replication overhead
```

---

# 41. Mailbox Capacity Type

```rust
pub struct MailboxCapacity {
    pub deposits_per_second: u64,
    pub fetches_per_second: u64,
    pub objects: u64,
    pub bytes: u64,
}
```

---

# 42. Relay Performance

Metrics:

```text
concurrent calls
packets/s
bitrate
jitter
CPU/codec overhead
```

---

# 43. Realtime Relay Capacity

```rust
pub struct RelayCapacity {
    pub concurrent_sessions: u64,
    pub aggregate_bitrate_bps: u64,
}
```

---

# 44. Bulk Provider Performance

Metrics:

```text
upload throughput
download throughput
chunk verification
resume overhead
storage IOPS
```

---

# 45. Bridge Performance

Metrics:

```text
new connections/s
concurrent sessions
obfuscation overhead
```

---

# 46. Directory Performance

Metrics:

```text
snapshot generation
signature verification
catalog lookup
mirror distribution
```

---

# 47. Control Plane Performance

Not latency-critical per packet.

---

# 48. Hard Rule

Control-plane DB must never be required for each data-plane packet.

---

# 49. Federation Performance

Metrics:

```text
inter-domain throughput
queue depth
gateway latency
settlement batching
```

---

# 50. Client Sync Performance

Metrics:

```text
sync batch size
apply latency
local DB write
background CPU
battery
```

---

# 51. Client Privacy

No remote detailed client performance trace by default.

---

# 52. Capacity Envelope

For each service define sustainable envelope.

---

# 53. Capacity Envelope Type

```rust
pub struct CapacityEnvelope {
    pub normal: WorkloadLimit,
    pub peak: WorkloadLimit,
    pub emergency: WorkloadLimit,
}
```

---

# 54. Normal

Expected sustained load.

---

# 55. Peak

Short burst.

---

# 56. Emergency

Disaster/failover headroom.

---

# 57. Headroom

Critical.

---

# 58. Headroom Policy

```rust
pub struct CapacityHeadroomPolicy {
    pub cpu_target_pct: u8,
    pub memory_target_pct: u8,
    pub network_target_pct: u8,
    pub storage_target_pct: u8,
}
```

---

# 59. Recommended Principle

Do not run privacy-critical network at:

```text
95–100% sustained utilization
```

---

# 60. Why

Need room for:

```text
region failover
traffic burst
cover traffic
rotation
recovery
```

---

# 61. N+1 / N+2 Capacity

Part 50 integration.

---

# 62. Regional Failover Sizing

Remaining regions must absorb failed-region load.

---

# 63. Sizing Formula

Basic:

```text
required_capacity =
expected_peak
× redundancy_factor
× privacy_overhead
× safety_margin
```

---

# 64. Privacy Overhead

Includes:

```text
padding
cover traffic
multi-hop routing
replication
```

---

# 65. Safety Margin

Includes uncertainty.

---

# 66. Capacity Estimate Type

```rust
pub struct CapacityEstimate {
    pub workload: WorkloadProfile,
    pub redundancy_factor: FixedRatio,
    pub privacy_overhead: FixedRatio,
    pub safety_margin: FixedRatio,
}
```

---

# 67. Fixed Point

Prefer fixed-point over floating point for deterministic policy.

---

# 68. Queueing Theory

Useful.

---

# 69. Little's Law

Can estimate:

```text
L = λW
```

for queue/system sizing.

---

# 70. But

Do not assume M/M/1 where workload is bursty/multi-stage.

---

# 71. Use Simulation

For complex paths.

---

# 72. Queue Architecture

Every queue:

```text
bounded
prioritized
observable
backpressured
```

---

# 73. No Unbounded Tokio Channel

Hard rule in production hot paths.

---

# 74. Bounded Channel

```rust
pub struct QueueBudget {
    pub max_items: usize,
    pub max_bytes: usize,
}
```

---

# 75. Queue Priority

Use existing classes:

```text
Emergency
SecurityControl
Realtime
InteractiveMessaging
Attachment
Background
Cover
```

---

# 76. Backpressure

Propagate upstream.

---

# 77. Load Shedding

Order:

```text
cover excess
background
bulk
attachments
```

before interactive/security.

---

# 78. Privacy Caveat

If Maximum Anonymity requires minimum cover level:

```text
cannot silently shed below floor
```

---

# 79. Degraded Privacy State

Must be explicit.

---

# 80. Queue Fairness

Avoid one source/domain monopolizing queue.

---

# 81. Fair Queueing

Potential:

```text
weighted fair queue
deficit round robin
```

---

# 82. Identity Privacy

Fairness key should be scoped service capability, not global user ID.

---

# 83. Resource Isolation

Critical for multi-service nodes.

---

# 84. Isolation Dimensions

```text
CPU
memory
disk
network
file descriptors
threads/tasks
```

---

# 85. cgroups

Recommended on Linux.

---

# 86. CPU Isolation

Examples:

```text
quota
shares
cpuset
```

---

# 87. Memory Isolation

Use:

```text
memory.max
memory.high
```

---

# 88. OOM Behavior

Service-specific.

---

# 89. No Global Host OOM Cascade

Hard rule.

---

# 90. Network Isolation

Traffic control.

---

# 91. Linux `tc`

Potential.

---

# 92. Bandwidth Classes

Separate:

```text
realtime
control
interactive
bulk
background
```

---

# 93. Disk Isolation

IO scheduler/cgroup IO.

---

# 94. Stateful Mailbox

Needs protected IOPS.

---

# 95. Noisy Neighbor

Must not starve privacy-critical service.

---

# 96. Resource Reservation

```rust
pub struct ServiceResourceReservation {
    pub cpu_millis: u32,
    pub memory_bytes: u64,
    pub disk_iops: u64,
    pub network_bps: u64,
}
```

---

# 97. Guaranteed Minimum

Useful for critical services.

---

# 98. Burst Limit

Higher.

---

# 99. Resource Class

```rust
pub enum ResourceClass {
    Guaranteed,
    Burstable,
    BestEffort,
}
```

---

# 100. Security/Directory

Guaranteed.

---

# 101. Relay

Guaranteed/Burstable.

---

# 102. Bulk

Burstable.

---

# 103. Background

BestEffort.

---

# 104. Threading Model

Rust/Tokio.

---

# 105. Async For

```text
network
timers
IO
```

---

# 106. Blocking Pools

For:

```text
CPU heavy crypto
compression
DB operations if blocking
```

---

# 107. No Blocking On Tokio Core

Hard rule.

---

# 108. Actor Model

Useful for ownership/isolation.

---

# 109. Actor Mailbox

Bounded.

---

# 110. Per-Connection Task Explosion

Avoid.

---

# 111. Task Budget

```rust
pub struct TaskBudget {
    pub max_active_tasks: usize,
}
```

---

# 112. Memory Budget

Do not allocate proportional to entire history.

---

# 113. Streaming

Use for:

```text
attachments
backup
federation bulk
```

---

# 114. No Full File Buffer

Hard rule.

---

# 115. Zero-Copy

Use where safe.

---

# 116. Rust Tools

Possible:

```text
bytes
io_uring
sendfile
mmap
```

---

# 117. Caution

Zero-copy can complicate lifetime/security.

---

# 118. Prefer Clarity First

Hard rule.

---

# 119. Serialization Performance

Postcard preferred.

---

# 120. Bounded Decode

Mandatory.

---

# 121. Serialization Benchmark

Per protocol version.

---

# 122. Crypto Benchmark

Measure:

```text
AEAD
signatures
KDF
Sphinx operations
```

---

# 123. Hardware Acceleration

Can use:

```text
AES-NI
AVX2
ARM crypto
```

through reviewed libraries.

---

# 124. Generic Artifact

Runtime CPU feature detection.

---

# 125. No Protocol Difference Based On CPU

Hard rule.

---

# 126. Database Performance

Different stores have different roles.

---

# 127. PostgreSQL

Control plane.

---

# 128. Embedded DB

Client/local.

---

# 129. Mailbox Store

May use SQL/KV/object hybrid.

---

# 130. DB Benchmarks

Measure:

```text
write latency
read latency
compaction
index cost
recovery
```

---

# 131. Connection Pool

Bounded.

---

# 132. Pool Exhaustion

Backpressure.

---

# 133. No Unlimited DB Connections

Hard rule.

---

# 134. Storage Sizing

Formula:

```text
required_storage =
retained_objects
× avg_object_size
× replication_factor
× metadata_overhead
× safety_margin
```

---

# 135. Mailbox Retention

Part 35.

---

# 136. Bulk Storage

Part 40.

---

# 137. Tombstone Overhead

Include.

---

# 138. Backup Overhead

Separate.

---

# 139. Network Sizing

Formula:

```text
network_required =
real_traffic
+ cover_traffic
+ replication
+ federation
+ failover_margin
```

---

# 140. Egress Cost

Economic Part 54 input.

---

# 141. Relay Sizing

Use:

```text
concurrent sessions
× avg bitrate
× safety factor
```

---

# 142. Multi-Relay

Double/more bandwidth.

---

# 143. Mixnet Sizing

Use:

```text
cell rate
× cell size
× hops
× cover ratio
```

---

# 144. Cover Ratio

Privacy policy input.

---

# 145. No Fake Capacity Claim

Hard rule.

---

# 146. Capacity Publication

Coarse classes.

---

# 147. Capacity Class

```rust
pub enum CapacityClass {
    Tiny,
    Small,
    Medium,
    Large,
    ExtraLarge,
}
```

---

# 148. Provider Catalog

Publishes class, not exact host utilization.

---

# 149. Runtime Utilization

Coarse bucket.

---

# 150. Utilization Class

```rust
pub enum UtilizationClass {
    Low,
    Moderate,
    High,
    Critical,
}
```

---

# 151. Autoscaling

Possible for:

```text
relay
bulk
mailbox frontends
directory mirrors
```

---

# 152. Mix Nodes

Autoscaling more complex due topology epochs.

---

# 153. Scaling Policy

```rust
pub struct AutoscalingPolicy {
    pub min_instances: u16,
    pub max_instances: u16,
    pub scale_up_threshold: UtilizationClass,
    pub scale_down_delay: Duration,
}
```

---

# 154. Scale-Up

Can be responsive.

---

# 155. Scale-Down

Slow.

---

# 156. Why

Avoid oscillation and correlation.

---

# 157. Jitter Scale Actions

Recommended.

---

# 158. No Per-User Autoscaling Trigger

Hard rule.

---

# 159. Trigger Inputs

Aggregate:

```text
queue
CPU
network
session count
```

---

# 160. Privacy-Aware Autoscaling

Do not scale based on:

```text
one conversation
one mailbox
one user cohort
```

---

# 161. Cold Start

Important.

---

# 162. Warm Pool

Useful for relay/emergency capacity.

---

# 163. Capacity Reservation

Part 49.

---

# 164. Reserved Headroom

Part 50.

---

# 165. Scaling Lag

Model explicitly.

---

# 166. Capacity Planner

```rust
pub trait CapacityPlanner {
    fn estimate(
        &self,
        workload: &WorkloadProfile,
        service: PerformanceDomain,
        policy: &CapacityPolicy,
    ) -> Result<CapacityPlan, CapacityError>;
}
```

---

# 167. Capacity Plan

```rust
pub struct CapacityPlan {
    pub instances: u32,
    pub resources_per_instance: ServiceResourceReservation,
    pub headroom: FixedRatio,
}
```

---

# 168. Capacity Policy

```rust
pub struct CapacityPolicy {
    pub redundancy: FixedRatio,
    pub privacy_overhead: FixedRatio,
    pub safety_margin: FixedRatio,
}
```

---

# 169. Production Sizing Profiles

Provide defaults.

---

# 170. Example Classes

```rust
pub enum ProductionSize {
    Dev,
    Small,
    Medium,
    Large,
    Regional,
}
```

---

# 171. Dev

Single host.

---

# 172. Small

Community/self-host.

---

# 173. Medium

Regional organization.

---

# 174. Large

Public provider.

---

# 175. Regional

Multi-service regional cluster.

---

# 176. Do Not Hardcode User Counts

Hard rule.

---

# 177. Why

Traffic pattern matters more than account count.

---

# 178. Sizing Inputs

Use:

```text
active concurrency
messages/sec
bytes/sec
retention
call sessions
```

---

# 179. Capacity Report

```rust
pub struct CapacityReport {
    pub workload: WorkloadProfile,
    pub plan: CapacityPlan,
    pub bottlenecks: Vec<ResourceBottleneck>,
}
```

---

# 180. Bottleneck Types

```rust
pub enum ResourceBottleneck {
    Cpu,
    Memory,
    Disk,
    Network,
    Database,
    Queue,
}
```

---

# 181. Benchmark Reproducibility

Benchmark environment must be recorded.

---

# 182. Benchmark Manifest

```rust
pub struct BenchmarkManifest {
    pub commit: GitCommitHash,
    pub artifact_hash: BuildHash,
    pub host_profile: BenchmarkHostProfile,
    pub workload: WorkloadProfile,
}
```

---

# 183. Host Profile

```text
CPU
RAM
storage
kernel
runtime
```

---

# 184. No Hidden Turbo/Power State

Record where relevant.

---

# 185. Cross-Machine Comparison

Normalize carefully.

---

# 186. Benchmark Noise

Run repetitions.

---

# 187. Confidence Intervals

Recommended.

---

# 188. Regression Gate

Compare to baseline.

---

# 189. Performance Baseline

```rust
pub struct PerformanceBaseline {
    pub metric: BenchmarkMetric,
    pub expected: BenchmarkRange,
}
```

---

# 190. Gate Policy

```rust
pub struct PerformanceRegressionPolicy {
    pub max_regression_pct: u8,
}
```

---

# 191. Different Thresholds

Security/privacy hot path stricter.

---

# 192. No Blind "Faster Is Better"

Hard rule.

---

# 193. Example

Removing padding improves throughput.

Not allowed if privacy worsens.

---

# 194. Performance Change Review

Must include:

```text
security impact
privacy impact
resource impact
```

---

# 195. Privacy Cost Function

Useful concept.

---

# 196. Example

Optimization:

```text
batch size ↑
```

may:

```text
improve throughput
increase latency
improve timing privacy
```

---

# 197. Multi-Objective Optimization

Need tradeoffs.

---

# 198. Performance Objective

```rust
pub struct PerformanceObjective {
    pub latency_weight: u8,
    pub throughput_weight: u8,
    pub resource_weight: u8,
    pub privacy_constraint: PrivacyConstraintSet,
}
```

---

# 199. Privacy Is Constraint

Not optional weight.

---

# 200. Hard Rule

Privacy invariants cannot be traded away for benchmark score.

---

# 201. Network Emulation

Required.

---

# 202. Tools

Possible:

```text
tc netem
Toxiproxy
custom QUIC simulator
```

---

# 203. Simulate

```text
loss
latency
reorder
bandwidth limit
partition
```

---

# 204. Mobile Network Profile

Include:

```text
high RTT
loss
bandwidth variation
sleep/wake
```

---

# 205. Censorship Profile

Part 41.

---

# 206. Federation Profile

Cross-region latency.

---

# 207. Disaster Load Profile

Part 50.

---

# 208. Failover Capacity Test

Remove region and verify remaining capacity.

---

# 209. Soak Test

Run hours/days.

---

# 210. Watch

```text
memory leak
FD leak
task leak
queue drift
storage growth
```

---

# 211. Rust Memory Safety

Does not prevent logical leaks.

---

# 212. Leak Detection

Measure steady state.

---

# 213. Fragmentation

Track allocator behavior.

---

# 214. Allocator

Default initially.

---

# 215. Alternative Allocator

Only if measured.

---

# 216. No Premature Allocator Swap

Hard rule.

---

# 217. Heap Profiling

Staging/local.

---

# 218. Production

Coarse/controlled.

---

# 219. Network Buffering

Kernel/socket buffers matter.

---

# 220. Tune Carefully

Too large can hide congestion.

---

# 221. BBR/CUBIC

Transport-specific.

---

# 222. QUIC Congestion Control

Benchmark.

---

# 223. Different Networks

Need:

```text
Wi-Fi
mobile
high latency
lossy links
```

---

# 224. QUIC Connection Reuse

Important.

---

# 225. Handshake Cost

Benchmark.

---

# 226. Connection Pool

Bounded.

---

# 227. Idle Timeout

Privacy/performance tradeoff.

---

# 228. Connection Persistence

Can reveal relationship.

---

# 229. Strict Mode

May prefer shorter-lived/multiplexed anonymized provider connections.

---

# 230. Benchmark Both

Hard rule.

---

# 231. CPU Crypto Budget

Crypto is required.

---

# 232. Do Not Benchmark "crypto off"

Production invalid.

---

# 233. Compression

May help bandwidth.

---

# 234. Compression Risk

Side channels.

---

# 235. User Secret + Attacker-Controlled Data

Avoid unsafe compression context.

---

# 236. Compression Policy

Per protocol.

---

# 237. Benchmark With Real Security Settings

Hard rule.

---

# 238. Padding

Benchmark enabled.

---

# 239. Cover Traffic

Benchmark enabled for target privacy mode.

---

# 240. Replication

Benchmark enabled.

---

# 241. Storage Encryption

Benchmark enabled.

---

# 242. Realistic Production Benchmark

Hard rule.

---

# 243. Performance Test Environments

```text
local
CI
nightly
staging
pre-release
```

---

# 244. CI

Micro/component.

---

# 245. Nightly

Integration/capacity.

---

# 246. Pre-Release

Full scale selected scenarios.

---

# 247. Staging

Soak/chaos.

---

# 248. Production

Only safe aggregate monitoring.

---

# 249. No Production Load Bomb

Hard rule.

---

# 250. Synthetic Probe Load

Tiny.

---

# 251. Benchmark Dataset

Synthetic.

---

# 252. Message Sizes

Use distribution:

```text
small text
medium structured
large encrypted payload
```

---

# 253. Attachment Sizes

Buckets.

---

# 254. Call Profiles

```text
audio
low video
HD video
```

---

# 255. Codec Cost

Part 29/26.

---

# 256. AV1

CPU intensive.

---

# 257. Hardware Codec Availability

Client side mostly.

---

# 258. Relay

Should avoid transcoding.

---

# 259. Hard Rule

Relay must not transcode E2EE media.

---

# 260. Bulk Object Chunk Size

Benchmark.

---

# 261. Chunk Tradeoff

Small:

```text
more metadata
better resume
```

Large:

```text
higher throughput
worse retry granularity
```

---

# 262. Adaptive Chunking

Possible.

---

# 263. Privacy

Avoid unique size fingerprints where possible.

---

# 264. Capacity Planning Inputs

```rust
pub struct CapacityInputs {
    pub workload: WorkloadProfile,
    pub failure_model: FailureModel,
    pub privacy_mode: PrivacyRoutingMode,
    pub retention: RetentionProfile,
}
```

---

# 265. Failure Model

```rust
pub enum FailureModel {
    Normal,
    OneNodeLost,
    OneZoneLost,
    OneRegionLost,
    OneOperatorLost,
}
```

---

# 266. Sizing Must Pass Failure Model

Hard rule.

---

# 267. Example

Production regional cluster sized for:

```text
OneZoneLost
```

---

# 268. Multi-Region

May size for:

```text
OneRegionLost
```

---

# 269. Capacity Shortage

Behavior:

```text
defer bulk
reduce video quality
queue messages
```

---

# 270. Not:

```text
direct-route privacy bypass
```

---

# 271. Autoscaling Limits

Min/max bounded.

---

# 272. Scale Failure

Part 50/49 fallback.

---

# 273. Capacity Alert

Before saturation.

---

# 274. Leading Indicators

```text
queue growth
memory high
network high
storage high
```

---

# 275. Saturation Alert

Critical.

---

# 276. Capacity SLO

Examples:

```text
<70% normal CPU
<75% network
queue p99 below threshold
```

Exact values measured per service.

---

# 277. No Universal Percentage

Hard rule.

---

# 278. Cost-Performance

Part 54.

---

# 279. Cost Efficiency Metric

```text
cost per sustained service unit
```

---

# 280. Privacy Cost Included

Need compare same privacy profile.

---

# 281. No Benchmarking Cheap Insecure Mode Against Secure Mode

Hard rule.

---

# 282. Efficiency Index

Optional.

---

# 283. Capacity Classes Public

Coarse.

---

# 284. Exact Infrastructure Details

Operator-private.

---

# 285. Security Risk

Exact capacity may aid attackers.

---

# 286. Public Metrics

Coarse.

---

# 287. Benchmark Publishing

Can publish:

```text
software version
hardware class
workload
results
```

---

# 288. No Sensitive Topology

Hard rule.

---

# 289. Performance Observability

Part 51.

---

# 290. Safe Metrics

```text
aggregate latency
queue depth bucket
resource saturation
throughput class
```

---

# 291. Forbidden Metrics

No:

```text
per-user latency
per-contact throughput
conversation size profile
```

---

# 292. Resource Isolation Crate

Recommended.

---

# 293. Runtime Resource Controller

```rust
pub trait ResourceController {
    fn apply(
        &self,
        service: ServiceInstanceId,
        reservation: ServiceResourceReservation,
    ) -> Result<(), ResourceControlError>;
}
```

---

# 294. Benchmark Runner

```rust
pub trait BenchmarkRunner {
    fn run(
        &self,
        manifest: BenchmarkManifest,
    ) -> Result<BenchmarkReport, BenchmarkError>;
}
```

---

# 295. Capacity Planner Trait

```rust
pub trait ProductionSizer {
    fn size(
        &self,
        inputs: CapacityInputs,
    ) -> Result<CapacityReport, CapacityError>;
}
```

---

# 296. Regression Gate

```rust
pub trait PerformanceGate {
    fn evaluate(
        &self,
        report: &BenchmarkReport,
        baseline: &PerformanceBaseline,
    ) -> PerformanceGateDecision;
}
```

---

# 297. Gate Decision

```rust
pub enum PerformanceGateDecision {
    Pass,
    Warn,
    Fail,
}
```

---

# 298. Benchmark Report

```rust
pub struct BenchmarkReport {
    pub manifest: BenchmarkManifest,
    pub metrics: Vec<BenchmarkMetricResult>,
    pub bottlenecks: Vec<ResourceBottleneck>,
}
```

---

# 299. Benchmark Integrity

Record artifact hash.

---

# 300. No Benchmark Against Different Binary

Hard rule.

---

# 301. Capacity Database

Store historical benchmark results.

---

# 302. No User Data

Hard rule.

---

# 303. Regression Trend

Useful.

---

# 304. Release Gate

Performance regression can block release.

---

# 305. Privacy Regression

Always blocks release.

---

# 306. Capacity Regression

Blocks if SLO violated.

---

# 307. Benchmark Flakiness

Needs tolerance.

---

# 308. Use Repetitions

Hard rule.

---

# 309. Statistical Significance

Useful but not sufficient.

---

# 310. Engineering Judgment

Still needed.

---

# 311. Testkit

Need deterministic load generator.

---

# 312. Load Generator

Can generate:

```text
messages
mailbox deposits
relay sessions
bulk transfers
mix cells
federation traffic
```

---

# 313. No Real Identity

Synthetic only.

---

# 314. Synthetic Identity Pools

Disposable.

---

# 315. Scale Test

Examples:

```text
10k synthetic clients
100k mailboxes
1M queued objects
10k concurrent calls
```

as infrastructure allows.

---

# 316. Test Failure Modes

```text
CPU saturation
memory pressure
disk pressure
network saturation
DB saturation
queue saturation
```

---

# 317. Graceful Degradation Test

Verify priority order.

---

# 318. Privacy Floor Test

Under overload:

```text
privacy-required features remain enforced
```

---

# 319. Resource Starvation Test

Bulk cannot starve security control.

---

# 320. Noisy Neighbor Test

One domain/provider load isolated.

---

# 321. Queue Bomb Test

Bounded queue rejects/backpressures.

---

# 322. Connection Flood Test

Admission control.

---

# 323. Slowloris-like Test

Connection resource bounds.

---

# 324. Storage Fill Test

Mailbox rejects new deposit safely.

---

# 325. Disk Latency Spike Test

Backpressure.

---

# 326. DB Pool Exhaustion Test

No task explosion.

---

# 327. Thread Pool Saturation Test

Bounded.

---

# 328. Network Loss Test

QUIC retries without runaway CPU.

---

# 329. Federation Backlog Test

Per-domain bulkhead works.

---

# 330. Region Failover Load Test

Remaining regions maintain target.

---

# 331. Cover Traffic Test

Still meets minimum required profile.

---

# 332. Fuzzing

Fuzz:

```text
capacity policy
resource reservation
benchmark manifest
queue budget
```

---

# 333. Property Tests

Properties:

```text
queue memory remains bounded
privacy-required traffic class is never silently downgraded
autoscaling trigger uses aggregate service metrics only
sizing under failure model never returns fewer resources than normal model
```

---

# 334. Formal Verification Targets

Strong candidates:

```text
priority scheduler
queue bounds
resource reservation
autoscaling state
```

---

# 335. TLA+ Candidate

Priority/backpressure system under overload.

---

# 336. Kani Candidate

capacity arithmetic/overflow.

---

# 337. Loom Candidate

bounded queue concurrency.

---

# 338. Performance Security Invariants

Mandatory:

```text
1. Performance optimization cannot disable required encryption, padding, cover, replay protection, or anonymity routing.
2. Every hot-path queue is bounded by item count and/or bytes.
3. Resource isolation prevents bulk/background workloads from starving security-control and realtime classes.
4. Production sizing includes privacy overhead, redundancy, and failure headroom.
5. Autoscaling uses aggregate infrastructure signals, not per-user identity or conversation metrics.
6. Benchmark datasets are synthetic or privacy-safe aggregates; raw production user traffic is not replayed.
7. Tail latency and saturation are first-class metrics, not only averages.
8. Capacity publication is coarse and does not reveal detailed live topology.
9. Realistic benchmarks keep production security/privacy features enabled.
10. Region/operator failover capacity is tested before production claims are made.
11. Performance telemetry never becomes a user-behavior profile.
12. Overload degrades features/capacity before privacy/security guarantees.
```

---

# 339. Recommended Crate Layout

```text
crates/
├── siar-performance-core/
├── siar-benchmark/
├── siar-workload-model/
├── siar-capacity-planner/
├── siar-resource-isolation/
├── siar-queueing/
├── siar-autoscaling/
├── siar-performance-gate/
├── siar-performance-observability/
└── siar-performance-testkit/
```

---

# 340. `siar-performance-core`

Owns:

```text
budgets
capacity types
bottlenecks
errors
```

---

# 341. `siar-benchmark`

Benchmark manifests/reports/runners.

---

# 342. `siar-workload-model`

Synthetic production-like workload definitions.

---

# 343. `siar-capacity-planner`

Sizing/headroom/failure-model calculations.

---

# 344. `siar-resource-isolation`

cgroup/runtime resource reservations.

---

# 345. `siar-queueing`

Bounded priority/backpressure queues.

---

# 346. `siar-autoscaling`

Aggregate service autoscaling policy.

---

# 347. `siar-performance-gate`

Release regression gates.

---

# 348. `siar-performance-observability`

Privacy-safe performance metrics.

---

# 349. `siar-performance-testkit`

Load/failure/noisy-neighbor simulator.

---

# 350. Error Taxonomy

```rust
pub enum CapacityError {
    InvalidWorkload,
    ResourceUnsatisfied,
    HeadroomInsufficient,
    FailureModelUnsatisfied,
    ArithmeticOverflow,
    BenchmarkMissing,
    Internal,
}

pub enum BenchmarkError {
    EnvironmentInvalid,
    ArtifactMismatch,
    WorkloadInvalid,
    ResultIncomplete,
    RegressionDetected,
    Internal,
}
```

---

# 351. Initial Production Scope

Implement first:

```text
service-specific performance budgets
synthetic workload profiles
Criterion microbenchmarks
integration/capacity test harness
bounded queues everywhere
priority/backpressure scheduler
CPU/memory/network/storage resource reservations
mailbox/relay/mix sizing formulas
privacy-overhead factors
N+1 / failover headroom
regression baselines
nightly performance tests
region-loss capacity test
privacy-floor-under-load test
```

Then add:

```text
automatic production sizing recommendations
adaptive capacity models
advanced network simulation
multi-region cost/performance optimizer
formal scheduler verification
capacity forecasting
```

---

# 352. Definition of Done

Part 63 is complete when:

- every major anonymous service has an explicit performance budget
- workloads are modeled by concurrency/rate/size/network characteristics rather than only account count
- realistic benchmark suites keep all production security/privacy features enabled
- hot-path queues are bounded and backpressured
- resource isolation exists for CPU, memory, disk, network, FDs, and task counts
- priority classes prevent background/bulk load from starving critical traffic
- capacity formulas include replication, cover/padding, redundancy, and safety margin
- production sizing is tested against explicit failure models
- autoscaling is aggregate, bounded, and privacy-safe
- headroom/failover targets are explicit
- benchmark artifacts are tied to exact release hashes
- performance regressions are release-gated
- overload behavior degrades feature quality before privacy/security
- capacity, noisy-neighbor, region-loss, queue, soak, fuzz, and formal tests are specified

---

# 353. Final Architecture

```text
                    SYNTHETIC WORKLOADS
                           │
                           ▼
                     BENCHMARK SUITE
                           │
                           ▼
                   CAPACITY ENVELOPE
                           │
          ┌────────────────┼────────────────┐
          │                │                │
        CPU             Memory           Network
          │                │                │
          └────────────────┼────────────────┘
                           ▼
                    RESOURCE ISOLATION
                           │
                           ▼
                PRODUCTION SIZING / SCALE
                           │
                           ▼
                   SLO + RELEASE GATES
```

Performance safety model:

```text
realistic benchmark
+
bounded queues
+
resource isolation
+
privacy-aware capacity planning
+
failure headroom
+
regression gates
```

not:

```text
run hot until overloaded, then disable privacy features
```

---

# 354. Final Principle

Performance engineering for an anonymous network is not about maximizing raw throughput at any cost.

The correct model is:

```text
predictable latency
+
bounded resource usage
+
explicit headroom
+
failure-aware sizing
+
privacy-preserving optimization
```

This architecture gives SIAR a measurable, reproducible path from development benchmarks to production sizing while ensuring that overload, scaling, and optimization never become reasons to weaken the anonymity guarantees established across Parts 34–62.
