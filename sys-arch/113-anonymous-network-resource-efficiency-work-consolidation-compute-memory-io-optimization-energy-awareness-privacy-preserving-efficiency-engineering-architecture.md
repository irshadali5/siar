# Core System Architecture Part 113 — Anonymous Network Resource Efficiency, Work Consolidation, Compute/Memory/I/O Optimization, Energy Awareness & Privacy-Preserving Efficiency Engineering Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 113  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 8, 13, 17, 29, 31, 40, 63, 101–112  

**Primary purpose:** define SIAR's efficiency-engineering architecture for compute, memory, I/O, storage, network, batching, work consolidation, energy awareness, idle-state behavior, mobile battery constraints, background deferral, resource efficiency policy, and privacy-preserving optimization.

---

# 1. Purpose

Performance answers:

```text
How fast?
```

Efficiency answers:

```text
How much resource is spent to achieve that speed and reliability?
```

A production system must understand:

```text
How many CPU cycles are spent per operation?
How much memory is retained per active conversation?
How many wakeups does an idle client generate?
How much I/O is duplicated?
How much network traffic can safely be coalesced?
Which workloads can be deferred or consolidated?
How much privacy/security overhead must remain protected?
```

The governing principle is:

> **SIAR should minimize unnecessary compute, memory, I/O, network, storage, and energy use while preserving correctness, reliability, security, privacy, anonymity, and latency requirements.**

---

# 2. Architectural Position

```text
Operational Work
      │
      ▼
 Resource Accounting
      │
      ▼
 Efficiency Budgets
      │
  ┌───┼────┬────┐
  │   │    │    │
 CPU MEM   I/O  NET
  │   │    │    │
  └───┼────┴────┘
      ▼
 Work Consolidation
      │
      ▼
 Energy-Aware Scheduling
      │
      ▼
 Efficiency Verification
```

---

# 3. Core Separation

Keep distinct:

```text
performance
efficiency
capacity
cost
energy
latency
resource utilization
```

---

# 4. Non-Goals

Part 113 does not create:

```text
efficiency by disabling security
battery savings by suppressing critical delivery
user-level energy profiling
per-user resource ranking
unbounded batching that destroys latency
```

---

# 5. Efficiency Scope

```rust
pub enum EfficiencyScope {
    Service(ServiceId),
    Capability(CapabilityId),
    WorkClass(WorkClass),
    Platform(PlatformClass),
    Region(RegionId),
    ResourcePool(ResourcePoolId),
}
```

---

# 6. No Global User Scope

Hard rule.

---

# 7. Resource Efficiency Dimension

```rust
pub enum EfficiencyDimension {
    Cpu,
    Memory,
    DiskIo,
    Network,
    Storage,
    Wakeups,
    Energy,
}
```

---

# 8. Efficiency Unit

```rust
pub struct EfficiencyUnit {
    pub dimension: EfficiencyDimension,
    pub value: FixedPoint,
}
```

---

# 9. Operation Cost

Efficiency measured per technical unit of work.

Examples:

```text
CPU-seconds / 1000 messages
bytes allocated / message
network bytes / delivered attachment byte
disk writes / committed message
wakeups / idle minute
```

---

# 10. No User Value Metric

Hard rule.

---

# 11. Efficiency Budget

```rust
pub struct EfficiencyBudget {
    pub scope: EfficiencyScope,
    pub max_cpu_per_unit: Option<FixedPoint>,
    pub max_memory_per_active_unit: Option<u64>,
    pub max_io_per_unit: Option<FixedPoint>,
    pub max_network_overhead_ratio: Option<FixedPoint>,
    pub max_idle_wakeups: Option<u32>,
}
```

---

# 12. Efficiency Budget Is Secondary To Correctness

Hard rule.

---

# 13. Policy Precedence

```text
correctness
> security/privacy
> reliability
> latency SLO
> efficiency
> cost
```

---

# 14. Hard rule.

---

# 15. Efficiency Profile

```rust
pub enum EfficiencyProfile {
    Realtime,
    Interactive,
    Balanced,
    Background,
    BatterySaver,
    ThroughputOptimized,
}
```

Realtime favors latency, Interactive favors human responsiveness, Balanced is the default, Background may batch/defer, BatterySaver coalesces optional work, and ThroughputOptimized serves bulk processing.

**Hard rule:** efficiency profile derives from workload semantics, not user importance.

---

# 16. Compute Efficiency

CPU work should be proportional to useful work.

```rust
pub struct CpuEfficiency {
    pub cpu_time: Duration,
    pub work_units: u64,
}
```

Avoid busy polling. Prefer event-driven Tokio tasks that sleep on readiness. Spin loops are permitted only for proven bounded low-level hot paths.

---

# 17. Work Coalescing

Combine similar pending work such as presence updates, index refreshes, cache refreshes, sync flushes, and notification digests.

```rust
pub struct CoalescingPolicy {
    pub max_delay: Duration,
    pub max_items: usize,
    pub semantic_key: CoalescingKeyClass,
}

pub enum CoalescingKeyClass {
    Service,
    Tenant,
    ConversationLocal,
    Resource,
}
```

**Hard rules:**

- never coalesce across tenant/isolation boundaries where semantics differ;
- no global behavioral-user key;
- never coalesce non-idempotent writes or distinct authorization contexts.

---

# 18. Batching

```rust
pub struct BatchingPolicy {
    pub max_items: usize,
    pub max_bytes: usize,
    pub max_wait: Duration,
}
```

Batching reduces syscall, transaction, framing, and fsync overhead, but every batch has a maximum wait and size. Tail latency gates from Part 112 remain authoritative.

---

# 19. Bounded CPU Parallelism

CPU-bound work may use Rayon, but background work never consumes all cores.

```rust
pub struct CpuClassBudget {
    pub class: WorkClass,
    pub max_parallelism: usize,
    pub priority: CpuPriorityClass,
}

pub enum CpuPriorityClass {
    Critical,
    Interactive,
    Normal,
    Background,
    Idle,
}
```

Security-critical work receives appropriate reserve and cannot be demoted merely to save power.

---

# 20. Crypto Efficiency

Optimize implementation without weakening cryptography.

Allowed:

```text
safe batch verification
hardware acceleration from trusted implementations
session-state reuse
buffer reuse
```

Forbidden:

```text
smaller unsafe keys
signature skipping
nonce reuse
software-key fallback around HSM policy
```

---

# 21. Memory Efficiency

Memory tracks the active working set rather than total history.

```rust
pub struct MemoryEfficiencyBudget {
    pub baseline_bytes: u64,
    pub per_active_conversation_bytes: u64,
    pub per_active_transfer_bytes: u64,
    pub peak_bytes: u64,
}
```

Large histories are paged/virtualized. Attachments are streamed. Search indexes remain on disk. The UI cannot retain entire message history simply because it has been viewed before.

---

# 22. Buffer Pooling

```rust
pub struct BufferPoolPolicy {
    pub buffer_size_classes: Vec<usize>,
    pub max_cached_per_class: usize,
}
```

Pools are bounded. Large buffers have short lifetimes. Zero-copy is preferred for media and attachments where it measurably helps, but unsafe complexity is not justified by microbenchmarks alone.

---

# 23. Memory Pressure

Response order:

```text
trim caches
release reusable pools
reduce prefetch
pause background work
shed optional work
```

Never discard authoritative committed state.

---

# 24. Cache Efficiency

```rust
pub struct CacheEfficiency {
    pub hit_rate: FixedPoint,
    pub bytes: u64,
    pub saved_work: FixedPoint,
}
```

Cache value is evaluated against memory cost and saved computation/I/O. A high hit rate does not justify an oversized cache. One-off giant objects should not be admitted automatically.

---

# 25. Prefetch

```rust
pub struct PrefetchPolicy {
    pub max_bytes: usize,
    pub max_items: usize,
    pub allowed_profiles: BTreeSet<EfficiencyProfile>,
}
```

Prefer deterministic context such as the next page or already-known attachment manifest. Behavioral surveillance is not a prefetch strategy.

---

# 26. Disk I/O Efficiency

Track write amplification:

```rust
pub struct WriteAmplification {
    pub logical_bytes: u64,
    pub physical_bytes: u64,
}
```

Tune WAL/checkpoints, batch safe writes, use prepared statements and indexes, and avoid hot-path full scans. Durability semantics such as WAL/fsync are never removed simply to win a benchmark.

---

# 27. Database Connection Efficiency

```rust
pub struct DbConnectionBudget {
    pub minimum: u16,
    pub target: u16,
    pub maximum: u16,
}
```

Oversized pools can reduce database efficiency through contention. Pool size must be tied to actual DB capacity and isolation policy.

---

# 28. Storage Efficiency

Classify storage:

```text
authoritative
cache
backup
temporary
derived
```

Compression is content-aware. Sensitive cross-user deduplication is not a baseline optimization because equality leakage can reveal information. Tenant-local deduplication requires explicit review and policy.

---

# 29. Temporary State

Temporary files, upload fragments, intermediate indexes, and staging blobs have bounded lifecycles and follow Part 67 secure deletion/cryptographic erasure rules.

---

# 30. Logging & Telemetry Efficiency

Use structured logging with bounded cardinality, rollups, sampling, and retention.

Never log private message content for efficiency analysis. Avoid persistent per-request traces when aggregate counters/histograms are sufficient.

---

# 31. Network Efficiency

Optimize with:

```text
connection reuse
compact framing
safe batching
bounded compression
duplicate-transfer prevention
resumable chunks
```

Security/privacy overhead is not automatically waste.

---

# 32. Keepalive Policy

```rust
pub struct KeepalivePolicy {
    pub foreground_interval: Duration,
    pub background_interval: Option<Duration>,
    pub battery_saver_interval: Option<Duration>,
}
```

Background clients must not emit constant high-frequency keepalives. Liveness, battery state, and OS lifecycle rules determine safe intervals.

---

# 33. Network Coalescing

Typing/presence/nonurgent state may be coalesced. Revocation, urgent control, security alerts, and correctness-critical ACKs may not be delayed beyond policy.

---

# 34. Network Efficiency Metric

```rust
pub struct NetworkEfficiency {
    pub payload_bytes: u64,
    pub transferred_bytes: u64,
}
```

The ratio is interpreted by traffic class; anonymity padding/cover traffic is protected overhead.

---

# 35. Cover Traffic

Cover traffic is intentional privacy capacity. It cannot be removed below the configured privacy floor in order to improve efficiency.

---

# 36. Mixnet Efficiency

Optimize implementation through efficient framing, buffer reuse, batched crypto, and bounded queues. Do not reduce mix delay, path diversity, or cover traffic below privacy policy.

---

# 37. Relay Efficiency

Use connection reuse, buffer pooling, and aggregate technical accounting. Do not persist user-linked flow histories.

---

# 38. Attachment Efficiency

Use chunked resumable transfer, encrypted-chunk retransmission avoidance, range requests, bounded parallelism, and content-aware compression. No unsafe global plaintext deduplication.

---

# 39. Media Efficiency

Dynamic codec complexity/bitrate is allowed. Hardware codecs may be used where trusted/available, with software fallback. E2EE and relay/privacy policy remain mandatory.

---

# 40. Background Work

```rust
pub enum BackgroundWorkClass {
    Indexing,
    Compaction,
    Backup,
    SyncMaintenance,
    CacheRefresh,
    TelemetryRollup,
    UpdateCheck,
}
```

```rust
pub struct BackgroundWorkBudget {
    pub cpu_fraction: FixedPoint,
    pub io_bytes_per_second: Option<u64>,
    pub network_bytes_per_second: Option<u64>,
}
```

Background work yields to realtime, control-plane, direct messaging, and interactive work.

---

# 41. Work Consolidation

```rust
pub struct ConsolidationWindow {
    pub max_delay: Duration,
}
```

Examples:

```text
index maintenance
backup metadata flush
cache expiry maintenance
telemetry rollup
update checks
```

A shared scheduler coordinates timers so that many subsystems do not independently wake the machine.

---

# 42. Wakeup Consolidation

Especially important on Android/mobile platforms.

Prefer:

```text
one coordinated background wake
```

over:

```text
many independent periodic timers
```

OS lifecycle constraints remain authoritative.

---

# 43. Energy Awareness

Energy is a first-class resource.

```rust
pub enum EnergyState {
    MainsPower,
    BatteryNormal,
    BatteryLow,
    BatteryCritical,
    ThermalLimited,
}
```

---

# 44. Energy Policy

```rust
pub struct EnergyPolicy {
    pub state: EnergyState,
    pub allowed_profiles: BTreeSet<EfficiencyProfile>,
    pub max_background_cpu: FixedPoint,
}
```

Security, emergency communication, revocation, and correctness-critical work remain protected.

---

# 45. Battery Saver

May:

```text
defer indexing
reduce prefetch
coalesce background sync
reduce keepalive frequency
pause bulk transfer
lower optional media complexity
```

May not:

```text
disable encryption
drop committed messages
disable authorization
indefinitely defer security revocation
```

---

# 46. Thermal Awareness

Thermal pressure may reduce video encode complexity, pause bulk work, and reduce background CPU. Core control/security/messaging semantics remain intact.

---

# 47. Battery Telemetry

Prefer local-only battery/thermal data. Optional uploaded measurements are coarse aggregates and cannot form a device fingerprint or persistent behavioral history.

---

# 48. Idle Efficiency

Idle means idle.

```rust
pub struct IdleEfficiencyBudget {
    pub max_cpu_percent: FixedPoint,
    pub max_wakeups_per_minute: u32,
    pub max_network_bytes_per_hour: u64,
}
```

Goals:

```text
near-zero CPU
few wakeups
minimal background traffic
no busy polling
```

---

# 49. Android Lifecycle

Foreground/background/suspended states are explicit. Android Doze restrictions are treated truthfully; SIAR does not promise continuous background networking when the OS cannot provide it.

---

# 50. Push / Wake

Use OS-supported wake mechanisms where necessary. Push messages are opaque wake hints, not private message payloads.

---

# 51. Desktop Idle

Desktop daemon remains event-driven. Dioxus UI does not poll the daemon continuously for changes.

---

# 52. Server Idle

Scale-to-zero is allowed only for services whose reliability class permits it. Security/control/quorum services keep minimum protected capacity.

---

# 53. Work Placement Efficiency

Placement may consider resource/energy efficiency only after:

```text
security
privacy
residency
reliability
availability
```

constraints are satisfied.

---

# 54. Multi-Tenant Consolidation

Shared pools improve efficiency but require strong tenant isolation, fair scheduling, quota boundaries, and noisy-neighbor protection.

No per-user resource profile is needed.

---

# 55. Work Stealing

CPU work-stealing pools may improve utilization, but work-class priority and tenant/security boundaries remain enforced.

---

# 56. NUMA Awareness

Optional for large servers. Not part of the initial baseline.

---

# 57. Data Layout

Hot-path structures prefer compact typed Rust data:

```text
small enums
vectors
arrays
bounded maps
borrowed slices
```

Avoid pointer-heavy structures where measurement shows poor locality.

---

# 58. Internal Data Formats

Use:

```text
Postcard → runtime/internal binary
RON      → human configuration
JSON     → external interoperability only where needed
```

Avoid repeated serialization/deserialization transformations in hot paths.

---

# 59. Compression Policy

Do not compress already-compressed media unnecessarily. Select compression according to payload class, size, CPU budget, and latency budget.

---

# 60. Storage Tiering

Cold storage may move to cheaper/slower tiers only if recovery, residency, retention, and latency policy allow it.

---

# 61. Backup Efficiency

Incremental/delta techniques are allowed where recovery independence remains valid. Deduplication cannot create cross-tenant/privacy leakage.

---

# 62. Compaction

Log/KV/index compaction runs as bounded background work with explicit I/O/CPU limits.

---

# 63. Efficiency Metrics

Safe examples:

```text
CPU-seconds / 1000 operations
bytes allocated / operation
disk bytes / logical byte
network overhead ratio
idle wakeups / minute
battery energy / workload class
```

Never dimension by user identity.

---

# 64. Efficiency Baseline

Each release can have an efficiency baseline.

```rust
pub struct EfficiencyRegression {
    pub dimension: EfficiencyDimension,
    pub baseline: FixedPoint,
    pub candidate: FixedPoint,
    pub delta: FixedPoint,
}
```

Significant CPU, memory, network, I/O, idle, or battery regressions can block release certification.

---

# 65. Release Integration

Part 108 readiness may include:

```text
CPU regression
memory regression
idle-wakeup regression
network-overhead regression
battery/thermal regression
```

---

# 66. Performance Integration

Part 112 remains the latency authority. An optimization that saves CPU but violates p99 latency is rejected.

---

# 67. Capacity Forecast Integration

Part 111 forecasts use measured post-optimization resource cost only after real evidence verifies improvement.

---

# 68. SLO Integration

Part 109 error budgets are not casually spent for efficiency. Critical reliability objectives remain protected.

---

# 69. Resilience Integration

Part 110 prevents aggressive consolidation from introducing common-mode failure or a new single point of failure.

---

# 70. Capacity Integration

Part 101 retains admission, protected pools, load shedding, and headroom policy. Efficiency does not replace capacity engineering.

---

# 71. FinOps Integration

Part 102 may value efficiency savings, but financial pressure cannot force unsafe changes.

---

# 72. Geographic Governance Integration

Part 103 filters candidate regions before energy/cost/efficiency optimization.

---

# 73. Desired-State Integration

Part 105 compiles approved efficiency, background, pool, timer, and energy policies into desired state.

---

# 74. Inventory Integration

Part 106 associates efficiency measurements with service/assets/resource pools, never user graphs.

---

# 75. Change Integration

Part 107 treats major consolidation, batching, storage, scheduling, or energy changes as governed production changes.

---

# 76. Organizational Governance Integration

Part 104 assigns service/capacity/efficiency ownership without turning efficiency metrics into individual workforce scoring.

---

# 77. Privacy Boundary

Efficiency cannot justify:

```text
cross-user sensitive dedup
behavioral prefetch profiling
trace stitching across anonymous boundaries
global user resource fingerprints
```

---

# 78. Efficiency Policy Service

```rust
pub trait EfficiencyPolicyService {
    fn budget(
        &self,
        scope: EfficiencyScope,
    ) -> Result<EfficiencyBudget, EfficiencyError>;

    fn evaluate(
        &self,
        scope: EfficiencyScope,
    ) -> Result<EfficiencyEvaluation, EfficiencyError>;
}
```

---

# 79. Efficiency Evaluation

```rust
pub struct EfficiencyEvaluation {
    pub scope: EfficiencyScope,
    pub cpu: Option<CpuEfficiency>,
    pub memory: Option<MemoryEfficiencySummary>,
    pub io: Option<IoEfficiencySummary>,
    pub network: Option<NetworkEfficiency>,
    pub energy: Option<EnergyEfficiencySummary>,
}
```

---

# 80. Work Consolidation Service

```rust
pub trait WorkConsolidationService {
    fn plan(
        &self,
        work: &[PendingWork],
        policy: &ConsolidationPolicy,
    ) -> Result<ConsolidationPlan, EfficiencyError>;
}
```

Planning does not bypass the scheduler or change authority.

---

# 81. Energy-Aware Scheduler

```rust
pub trait EnergyAwareScheduler {
    fn scheduling_profile(
        &self,
        energy: EnergyState,
        class: WorkClass,
    ) -> EfficiencyProfile;
}
```

Hard security/privacy/reliability policy is evaluated before efficiency profile.

---

# 82. Efficiency Error Taxonomy

```rust
pub enum EfficiencyError {
    BudgetMissing,
    TelemetryMissing,
    UnsafeOptimization,
    IsolationViolation,
    LatencyBudgetConflict,
    ReliabilityConflict,
    PrivacyConflict,
    ScopeViolation,
    Unauthorized,
    Internal,
}
```

---

# 83. Observability

Safe dimensions:

```text
service
capability
platform
region
work class
resource pool
```

Forbidden dimensions:

```text
stable user identity
message content
contact graph
employee ranking
```

---

# 84. Efficiency SLO Examples

```text
desktop idle CPU below target
mobile idle wakeups below target
message write amplification below target
attachment retransmission overhead below target
background indexing within CPU budget
```

---

# 85. Failure Modes

## 85.1 Over-Aggressive Batching

Symptoms:

```text
p95/p99 latency rise
bursty I/O
large queue spikes
```

Response: reduce batch size/wait.

## 85.2 Memory Retention

Investigate:

```text
cache size
buffer pool
task lifetime
reference cycles at FFI/UI boundaries
```

## 85.3 Connection Churn

Improve connection reuse without excessive keepalive.

## 85.4 Wake Storm

Consolidate timers and background tasks.

## 85.5 Efficiency Optimization Causes Reliability Regression

Rollback or adjust; efficiency never outranks reliability.

---

# 86. Testing

Required scenarios:

```text
idle client
interactive messaging
bulk attachment
background indexing
battery saver
thermal pressure
```

Tests include:

```text
idle CPU
wakeup budget
working-set memory
buffer-pool bound
batch max-wait
safe coalescing
background CPU yield
write amplification
connection reuse
mixnet privacy preservation
cross-user dedup rejection
battery/thermal scheduling
crypto/HSM preservation
p99 regression
resilience/SPOF checks
release regression
```

---

# 87. Fuzzing

Fuzz:

```text
batching policy
coalescing policy
buffer-pool policy
energy policy
efficiency budget
```

---

# 88. Property Tests

Properties:

```text
batch wait can never exceed configured maximum
buffer pool can never retain more than configured limits
battery-saving policy can never disable mandatory security/privacy operations
sensitive cross-user data can never be deduplicated by baseline policy
```

---

# 89. Formal Verification Targets

Strong candidates:

```text
batching bounds
buffer-pool bounds
energy-policy precedence
work-class scheduling
```

Kani:
- resource-budget invariants
- policy precedence
- bounded pool arithmetic

TLA+:
- interactive/background contention
- consolidation windows
- energy-state transitions

Loom:
- concurrent buffer checkout/cancel/return
- timer coalescing
- background scheduler race conditions

---

# 90. Performance of the Efficiency Subsystem

The subsystem itself must remain cheap:

```text
aggregate counters
bounded histograms
periodic rollups
no persistent per-operation history
```

---

# 91. Storage

Store separately:

```text
efficiency budgets
aggregate efficiency metrics
regression baselines
energy policies
batching policies
coalescing policies
```

No user-behavior warehouse.

---

# 92. Partitioning

By:

```text
service
capability
platform
region
work class
resource pool
```

Never by user.

---

# 93. Crate Layout

Recommended:

```text
crates/
├── siar-efficiency-core/
├── siar-compute-efficiency/
├── siar-memory-efficiency/
├── siar-io-efficiency/
├── siar-network-efficiency/
├── siar-work-consolidation/
├── siar-energy-policy/
├── siar-idle-efficiency/
├── siar-efficiency-regression/
├── siar-efficiency-observability/
└── siar-efficiency-testkit/
```

### `siar-efficiency-core`

Owns:

```text
EfficiencyScope
EfficiencyDimension
EfficiencyProfile
errors
```

### `siar-compute-efficiency`

CPU/work-unit accounting and bounded parallelism.

### `siar-memory-efficiency`

Working-set, buffer, cache, and pool governance.

### `siar-io-efficiency`

Read/write amplification and batch policies.

### `siar-network-efficiency`

Connection reuse, coalescing, and overhead ratios.

### `siar-work-consolidation`

Batch/coalescing/wakeup scheduling.

### `siar-energy-policy`

Battery/thermal-aware scheduling.

### `siar-idle-efficiency`

Idle CPU/wakeup/network budgets.

### `siar-efficiency-regression`

Candidate-vs-baseline efficiency gates.

### `siar-efficiency-observability`

Aggregate efficiency health only.

### `siar-efficiency-testkit`

Compute/memory/I/O/battery/privacy tests.

---

# 94. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. Resource efficiency is always subordinate to correctness, security, privacy, reliability, residency, and required performance targets; no efficiency optimization may weaken a hard invariant.
2. Efficiency metrics are scoped to services, capabilities, platforms, work classes, regions, or resource pools and never use stable user identity, private content, contact graphs, or behavioral profiles.
3. CPU, worker pools, buffers, caches, connection pools, batching, coalescing, retries, background tasks, and retained memory are explicitly bounded; efficiency cannot rely on unlimited accumulation.
4. Work consolidation and batching always have maximum delay/size bounds and cannot merge operations across isolation, authorization, tenant, or semantic boundaries.
5. Memory use tracks active working set rather than total historical data; large histories/files are paged, streamed, chunked, virtualized, or stored rather than retained wholesale in RAM.
6. Disk/network/storage efficiency never disables durability, encryption, integrity verification, secure deletion, or required recovery semantics.
7. Sensitive cross-user deduplication is not a baseline optimization because equality leakage and cross-user information exposure are privacy risks.
8. Battery/thermal/idle policies may defer optional/background/bulk work but cannot indefinitely delay critical security, control-plane, emergency, or correctness-preserving operations.
9. Privacy-required cover traffic, mix delay, relay policy, E2EE, HSM usage, authorization checks, and other security/privacy overhead are protected resource classes, not removable inefficiency.
10. Consolidation, shared pools, placement, or cost optimization cannot create unreviewed single points of failure, noisy-neighbor isolation violations, or prohibited residency changes.
11. Efficiency telemetry and regression evidence are aggregate and bounded-cardinality and cannot become a user/device behavior history, employee-performance system, or cross-anonymity trace substrate.
12. Efficiency engineering integrates with performance budgets, capacity forecasting, SLOs, resilience, DR, FinOps, geographic governance, desired state, inventory, change/release governance, and organizational ownership without creating a bypass around hard security/privacy controls.
```

---

# 95. Initial Production Scope

Implement first:

```text
typed EfficiencyScope/EfficiencyDimension/EfficiencyProfile
CPU/work-unit metrics
bounded Rayon/Tokio concurrency classes
memory working-set budgets
buffer-pool limits
cache efficiency metrics
batching/coalescing policies
write/read amplification metrics
DB connection pool budgets
connection reuse/keepalive policies
network overhead metrics
idle CPU/wakeup/network budgets
battery/thermal energy states
background-work deferral
mobile wakeup consolidation
mixnet/cover-traffic protected resource policy
efficiency regression baselines
release integration
privacy-safe dashboards
efficiency testkit
```

Then add:

```text
adaptive batching constrained by p99 latency
NUMA-aware scheduling
advanced allocator profiling
hardware energy counters where available
energy-aware server placement
formal resource-bound verification
automated efficiency recommendations constrained by security/privacy/reliability policy
```

---

# 96. Definition of Done

Part 113 is complete when:

- efficiency scopes/resources are typed;
- CPU/memory/I/O/network/idle budgets exist;
- concurrency, pools, batches, and coalescing are bounded;
- working-set memory does not scale with total history;
- write/network amplification is observable;
- background work can yield/defer safely;
- battery/thermal state influences optional work only;
- idle clients avoid busy polling and wake storms;
- sensitive cross-user dedup is forbidden by baseline;
- security/privacy overhead remains protected;
- efficiency regressions can gate releases;
- consolidation cannot create unreviewed SPOFs;
- no user behavioral efficiency profiling exists;
- efficiency/privacy/fuzz/formal tests are specified.

---

# 97. Final Architecture

```text
                   OPERATIONAL WORK
                         │
                         ▼
                  RESOURCE ACCOUNTING
                         │
                         ▼
                   EFFICIENCY BUDGETS
                         │
          ┌──────────────┼──────────────┐
          │              │              │
         CPU           MEMORY         I/O/NET
          │              │              │
          └──────────────┼──────────────┘
                         ▼
                 WORK CONSOLIDATION
                         │
                         ▼
               ENERGY-AWARE SCHEDULING
                         │
                         ▼
                 EFFICIENCY REGRESSION
```

Efficiency-engineering safety model:

```text
bounded resource use
+
working-set discipline
+
batch/coalesce with limits
+
connection/buffer reuse
+
energy-aware scheduling
+
idle efficiency
+
privacy-aware storage/network optimization
+
release regression gates
```

not:

```text
save CPU by skipping crypto, save bandwidth by removing cover traffic, deduplicate sensitive data globally, and poll constantly in the background
```

---

# 98. Final Principle

Efficiency means eliminating unnecessary work, not eliminating guarantees.

The correct model is:

```text
measure resource cost
+
bound every pool and queue
+
consolidate safe work
+
reuse memory/connections
+
defer optional background work
+
respect battery and thermal state
+
protect privacy/security overhead
+
verify efficiency changes against latency and reliability
```

This architecture gives SIAR a privacy-preserving efficiency-engineering foundation for compute, memory, disk, storage, network, batching, work consolidation, mobile energy awareness, idle efficiency, and resource regression control while preserving the anonymity, local-first, least-authority, durability, reliability, and anti-surveillance guarantees established across Parts 34–112.
