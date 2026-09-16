# Core System Architecture Part 138 — Anonymous Network Extension Background Execution, Durable Jobs, Triggers, Scheduling, Wakeups, Mobile Lifecycle, Quotas & Privacy-Preserving Automation Runtime Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 138  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 13, 31, 34–42, 75, 101, 113–114, 130–137

**Primary purpose:** define SIAR's extension background-execution architecture for durable jobs, event/time/manual triggers, scheduling, wakeups, retries, idempotency, mobile lifecycle constraints, desktop daemon execution, battery/thermal/network awareness, offline operation, resource quotas, background capability attenuation, observability, anti-abuse enforcement, and privacy-preserving automation.

---

# 1. Purpose

Extensions often need to perform useful work when the interactive UI is not active.

Examples include:

```text
syncing
scheduled exports
webhook retries
local indexing
maintenance
periodic polling
delayed notifications
background automation
```

But background execution creates risks:

```text
battery drain
hidden network activity
persistent tracking
resource starvation
infinite retries
unexpected wakeups
silent privilege retention
```

The governing principle is:

> **SIAR extension background execution must be explicit, durable where needed, bounded by capability and resource policy, lifecycle-aware, battery/network conscious, retry-safe, and unable to become an invisible surveillance or persistence mechanism.**

---

# 2. Architectural Position

```text
                 EXTENSION INTENT
                       │
                       ▼
                 JOB DEFINITION
                       │
             ┌─────────┼─────────┐
             │         │         │
           TIME      EVENT     MANUAL
             │         │         │
             └─────────┼─────────┘
                       ▼
                 SCHEDULER / QUEUE
                       │
                       ▼
                 WAKEUP CONTROLLER
                       │
                       ▼
                RUNTIME SUPERVISOR
                       │
                       ▼
                  BOUNDED EXECUTION
```

---

# 3. Core Separation

Keep distinct:

```text
job definition
job instance
trigger
schedule
wakeup
runtime grant
execution attempt
retry policy
result
```

---

# 4. Non-Goals

Part 138 does not create:

```text
unbounded background loops
always-on polling
hidden wake locks
background privilege escalation
persistent user tracking
```

---

# 5. Job Identity

```rust
pub struct ExtensionJobId(pub [u8; 16]);
```

Stable for a logical job definition.

---

# 6. Job Instance Identity

```rust
pub struct ExtensionJobRunId(pub [u8; 16]);
```

Unique per execution attempt.

---

# 7. Job Definition

```rust
pub struct ExtensionJobDefinition {
    pub job_id: ExtensionJobId,
    pub extension: ExtensionId,
    pub trigger: ExtensionTrigger,
    pub policy: ExtensionJobPolicy,
    pub capability_scope: BackgroundCapabilityScope,
}
```

---

# 8. Trigger Types

```rust
pub enum ExtensionTrigger {
    Manual,
    OneShot(Timestamp),
    Periodic(PeriodicSchedule),
    Event(ExtensionEventTrigger),
    Deferred(DeferredCondition),
}
```

---

# 9. Hard Rule

Every background execution begins from an explicit registered trigger.

---

# 10. Periodic Schedule

```rust
pub struct PeriodicSchedule {
    pub interval: Duration,
    pub flex: Option<Duration>,
}
```

---

# 11. Minimum Interval

Policy-defined.

---

# 12. Hard Rule

Extensions cannot request arbitrarily tight periodic execution.

---

# 13. Flexible Scheduling

Prefer flex windows where exact timing is unnecessary.

---

# 14. Hard Rule

Battery/resource efficiency outranks exactness unless semantics truly require exact timing.

---

# 15. One-Shot Trigger

Used for:

```text
delayed send
cleanup
scheduled export
retry after rate limit
```

---

# 16. Hard Rule

One-shot jobs persist durably if product semantics require survival across process restart.

---

# 17. Event Trigger

```rust
pub struct ExtensionEventTrigger {
    pub event_class: ExtensionEventClass,
    pub scope: ExtensionEventScope,
    pub debounce: Option<Duration>,
}
```

---

# 18. Hard Rule

Event trigger scope cannot exceed extension grants.

---

# 19. Event Debounce

Recommended for noisy signals.

---

# 20. Hard Rule

No trigger storm from high-frequency events.

---

# 21. Deferred Conditions

```rust
pub enum DeferredCondition {
    NetworkAvailable,
    UnmeteredNetwork,
    Charging,
    DeviceIdle,
    BatteryAbove(u8),
    ThermalBelow(ThermalLevel),
}
```

---

# 22. Hard Rule

Condition-based jobs wait instead of busy-polling.

---

# 23. Job Policy

```rust
pub struct ExtensionJobPolicy {
    pub max_runtime: Duration,
    pub max_attempts: u32,
    pub retry_policy: RetryPolicy,
    pub resource_budget: ExtensionJobResourceBudget,
}
```

---

# 24. Resource Budget

```rust
pub struct ExtensionJobResourceBudget {
    pub max_cpu_time: Duration,
    pub max_memory_bytes: u64,
    pub max_network_bytes: u64,
    pub max_storage_writes: u64,
}
```

---

# 25. Hard Rule

Every job has explicit bounded resources.

---

# 26. Background Capability Scope

Background work receives an attenuated capability subset.

```rust
pub struct BackgroundCapabilityScope {
    pub permissions: BTreeSet<ExtensionPermissionClass>,
    pub scope: EffectiveExtensionScope,
}
```

---

# 27. Hard Rule

Background authority can never exceed interactive authority.

---

# 28. Default Narrowing

Examples:

```text
interactive: read message content + send
background: metadata-only sync
```

---

# 29. Hard Rule

Sensitive foreground-only permissions may be unavailable in background.

---

# 30. Camera/Microphone

Unavailable to generic background jobs by default.

---

# 31. Hard Rule

No hidden background camera/microphone capture.

---

# 32. Clipboard

Background access denied by default.

---

# 33. Hard Rule

No silent clipboard harvesting.

---

# 34. Local Network

Background local-network scans denied unless explicit and justified.

---

# 35. Hard Rule

No silent device/network discovery loops.

---

# 36. Anonymous Mode

Further reduces background authority.

---

# 37. Hard Rule

Background execution cannot weaken privacy route or expose peer metadata.

---

# 38. Job Lifecycle

```rust
pub enum ExtensionJobState {
    Registered,
    Waiting,
    Ready,
    Running,
    RetryScheduled,
    Succeeded,
    Failed,
    Cancelled,
    Expired,
    Quarantined,
}
```

---

# 39. No Registered→Running Direct

Hard rule.

---

# 40. Job Run State

```rust
pub enum ExtensionJobRunState {
    Starting,
    Running,
    Completing,
    Completed,
    Failed,
    Cancelled,
    Unknown,
}
```

---

# 41. Unknown

Used when completion state cannot be proven after crash.

---

# 42. Hard Rule

Unknown is not automatically retried unless idempotency semantics allow it.

---

# 43. Durable Scheduler

Scheduler persists:

```text
job definition
next eligible time
retry state
attempt count
trigger checkpoint
```

---

# 44. Hard Rule

Durable jobs survive host restart without losing governance state.

---

# 45. Scheduler Storage

Postgres/server or local durable DB depending execution context.

---

# 46. Hard Rule

Local-first jobs can live entirely on device.

---

# 47. Job Queue

Bounded.

---

# 48. Hard Rule

No unbounded pending job accumulation.

---

# 49. Queue Admission

```rust
pub enum JobAdmissionDecision {
    Admit,
    Delay,
    Reject,
    Coalesce,
}
```

---

# 50. Hard Rule

Scheduler applies admission before queueing.

---

# 51. Coalescing

Useful for:

```text
sync
index refresh
presence-derived maintenance
```

---

# 52. Hard Rule

Multiple redundant triggers may collapse into one pending job.

---

# 53. Wakeup Controller

Abstracts platform-specific wake semantics.

---

# 54. Wakeup Request

```rust
pub struct ExtensionWakeupRequest {
    pub job: ExtensionJobId,
    pub earliest: Timestamp,
    pub latest: Option<Timestamp>,
    pub constraints: WakeupConstraints,
}
```

---

# 55. Constraints

```rust
pub struct WakeupConstraints {
    pub network: NetworkConstraint,
    pub charging: Option<bool>,
    pub min_battery_percent: Option<u8>,
    pub max_thermal: Option<ThermalLevel>,
}
```

---

# 56. Hard Rule

Wakeups are requests, not guarantees, especially on mobile.

---

# 57. Mobile Truthfulness

UI/docs must not promise exact background execution when OS lifecycle cannot guarantee it.

---

# 58. Hard Rule

No false “runs every X minutes” claim on constrained mobile platforms.

---

# 59. Android Lifecycle

Use platform scheduling primitives as narrow adapters.

---

# 60. Rust Core Ownership

Rust owns:

```text
job model
trigger semantics
retry/idempotency
quota
policy
```

Android adapter owns:

```text
WorkManager/JobScheduler integration
OS wake lifecycle
permission/lifecycle bridging
```

---

# 61. Hard Rule

Kotlin/Android layer cannot invent background authority.

---

# 62. Android Background Modes

Possible mappings:

```text
deferrable periodic work
one-shot constrained work
foreground service only when truly user-visible/required
push-triggered wake
```

---

# 63. Hard Rule

Foreground service not used to bypass background restrictions.

---

# 64. Notification Requirement

If OS requires foreground notification, user must see truthful state.

---

# 65. Hard Rule

No hidden persistent foreground service.

---

# 66. Doze / App Standby

Scheduler accepts delayed execution.

---

# 67. Hard Rule

Retry logic understands OS deferral separately from job failure.

---

# 68. Push Wake

Push can wake host to reevaluate jobs.

---

# 69. Hard Rule

Push payload remains opaque/minimal.

---

# 70. No Sensitive Job Payload In Push

Hard rule.

---

# 71. Desktop Lifecycle

Desktop daemon may execute background jobs while UI closed.

---

# 72. Hard Rule

Idle CPU near zero when no work.

---

# 73. Desktop Wakeup

Use:

```text
timer
event
network change
local IPC
```

without polling.

---

# 74. Hard Rule

No spin loops.

---

# 75. Embedded / Headless

Same job model can execute under daemon.

---

# 76. Hard Rule

Headless mode preserves identical authorization semantics.

---

# 77. Battery Awareness

```rust
pub enum BatteryState {
    Charging,
    High,
    Medium,
    Low,
    Critical,
    Unknown,
}
```

---

# 78. Hard Rule

Optional work throttled/deferred on low battery.

---

# 79. Critical Battery

Only essential extension work allowed.

---

# 80. Hard Rule

Extensions cannot self-declare essential class without host policy.

---

# 81. Thermal Awareness

```rust
pub enum ThermalLevel {
    Normal,
    Warm,
    Hot,
    Critical,
}
```

---

# 82. Hard Rule

Background CPU-heavy work throttled under thermal stress.

---

# 83. Network Awareness

```rust
pub enum NetworkConstraint {
    None,
    Any,
    Unmetered,
    LocalOnly,
    AnonymousPathAvailable,
}
```

---

# 84. Hard Rule

Network constraints describe prerequisites, not permission.

---

# 85. Metered Network

Large optional jobs deferred unless user/admin policy allows.

---

# 86. Hard Rule

No hidden large background transfer on metered connection.

---

# 87. Anonymous Path Available

Strict-anonymity job waits for acceptable route.

---

# 88. Hard Rule

No fallback to direct route.

---

# 89. Offline Execution

Jobs may run locally without internet if dependencies permit.

---

# 90. Hard Rule

Network-unavailable does not equal job failure when offline semantics exist.

---

# 91. Offline Queue

Remote-dependent work remains queued until conditions satisfied.

---

# 92. Hard Rule

Queue age/expiry explicit.

---

# 93. Job Expiry

```rust
pub struct JobExpiryPolicy {
    pub expires_at: Option<Timestamp>,
    pub on_expiry: ExpiryAction,
}
```

---

# 94. Expiry Action

```rust
pub enum ExpiryAction {
    Drop,
    MarkFailed,
    Notify,
}
```

---

# 95. Hard Rule

Expired jobs do not execute late silently.

---

# 96. Retry Policy

```rust
pub enum RetryPolicy {
    None,
    Fixed {
        delay: Duration,
        max_attempts: u32,
    },
    Exponential {
        base: Duration,
        max_delay: Duration,
        max_attempts: u32,
    },
}
```

---

# 97. Jitter

Recommended for distributed/server jobs.

---

# 98. Hard Rule

Retry count always bounded.

---

# 99. No Infinite Retry

Hard rule.

---

# 100. Retryable Error

Explicit classification.

```rust
pub enum ExtensionJobErrorClass {
    RetryableTransport,
    RetryableRateLimit,
    PermissionDenied,
    InvalidInput,
    ResourceLimit,
    PermanentRemoteFailure,
    Internal,
}
```

---

# 101. Hard Rule

PermissionDenied is not blindly retried.

---

# 102. Rate Limit Retry

Honor bounded retry-after.

---

# 103. Hard Rule

No aggressive retry storm.

---

# 104. Idempotency

Durable/background side effects need idempotency where retries are possible.

---

# 105. Idempotency Key

```rust
pub struct ExtensionJobIdempotencyKey(pub [u8; 16]);
```

---

# 106. Hard Rule

Same logical operation reuses same key across retry.

---

# 107. Persist-Before-Execute

For durable side effects:

```text
persist job
persist idempotency key
then execute
```

---

# 108. Hard Rule

No untracked side effect before durable job state.

---

# 109. Completion Receipt

```rust
pub struct ExtensionJobCompletionReceipt {
    pub run: ExtensionJobRunId,
    pub state: ExtensionJobRunState,
    pub committed_effects: Vec<OpaqueEffectRef>,
}
```

---

# 110. Hard Rule

Completion state is recorded before deleting retry state.

---

# 111. Crash Recovery

On restart:

```text
load Running/Unknown runs
determine idempotency status
reconcile
retry only if safe
```

---

# 112. Hard Rule

Do not treat crash as definite failure.

---

# 113. Lease

Distributed/server jobs may use execution lease.

---

# 114. Lease Record

```rust
pub struct ExtensionJobLease {
    pub run: ExtensionJobRunId,
    pub worker: WorkerId,
    pub expires_at: Timestamp,
    pub fencing_token: u64,
}
```

---

# 115. Hard Rule

Expired worker cannot commit after newer fencing token.

---

# 116. Local Device Jobs

No distributed lease needed when single executor.

---

# 117. Hard Rule

Do not over-engineer local single-process scheduling.

---

# 118. Event-Driven Automation

Event trigger should receive minimal event projection.

---

# 119. Hard Rule

Trigger event does not automatically grant payload read.

---

# 120. Example

```text
"new message exists"
```

does not imply:

```text
"read message content"
```

---

# 121. Hard Rule

Trigger Authority Separate From Processing Authority

---

# 122. Event Trigger Metadata

Coarse by default.

---

# 123. Hard Rule

No stable global correlation IDs.

---

# 124. Automation Chain

A job may schedule another job only through brokered scheduler capability.

---

# 125. Hard Rule

No recursive unbounded job spawning.

---

# 126. Spawn Depth

Bounded.

---

# 127. Hard Rule

Job fan-out bounded.

---

# 128. Fan-Out Budget

```rust
pub struct JobFanoutBudget {
    pub max_children: u32,
    pub max_depth: u16,
}
```

---

# 129. Hard Rule

One trigger cannot create unbounded work graph.

---

# 130. Job Dependency

```rust
pub struct ExtensionJobDependency {
    pub job: ExtensionJobId,
    pub depends_on: ExtensionJobId,
}
```

---

# 131. Hard Rule

Dependency graph acyclic.

---

# 132. DAG Validation

At registration.

---

# 133. Hard Rule

No hidden dynamic cycle.

---

# 134. Workflow Job

For multi-step automation:

```rust
pub struct ExtensionWorkflow {
    pub workflow_id: ExtensionWorkflowId,
    pub steps: Vec<ExtensionWorkflowStep>,
}
```

---

# 135. Hard Rule

Workflow steps explicit/durable.

---

# 136. Saga Compensation

Where irreversible external steps exist.

---

# 137. Hard Rule

Compensation is explicit, not assumed.

---

# 138. Secrets In Background

Jobs may reference SecretRef.

---

# 139. Hard Rule

No plaintext secret persisted in job payload.

---

# 140. Secret Use

Brokered at execution time.

---

# 141. Hard Rule

Expired/revoked secret causes safe failure.

---

# 142. Job Payload

Versioned and bounded.

---

# 143. Hard Rule

Large files referenced by handles/object IDs, not embedded.

---

# 144. Job Payload Schema

```rust
pub struct ExtensionJobPayload {
    pub schema: SchemaId,
    pub version: SchemaVersion,
    pub bytes: Vec<u8>,
}
```

---

# 145. Hard Rule

Strict max bytes.

---

# 146. Schema Compatibility

Part 129 applies.

---

# 147. Hard Rule

Unknown job payload version cannot execute privileged code path.

---

# 148. Scheduling Priority

```rust
pub enum ExtensionJobPriority {
    InteractiveDeferred,
    Normal,
    Background,
    Maintenance,
}
```

---

# 149. Hard Rule

Extensions cannot claim host-critical priority.

---

# 150. Host Priority

Core SIAR jobs outrank extension jobs.

---

# 151. Hard Rule

Extensions share fair scheduling.

---

# 152. Per-Extension Concurrency

```rust
pub struct ExtensionConcurrencyLimit {
    pub max_running_jobs: u32,
}
```

---

# 153. Hard Rule

Concurrency bounded per extension and globally.

---

# 154. Global Concurrency Budget

Protects device/server.

---

# 155. Hard Rule

Background extension work yields to interactive/core workload.

---

# 156. CPU Budget

Measured per run/period.

---

# 157. Hard Rule

Exceeded CPU budget → throttle/cancel/quarantine according to policy.

---

# 158. Network Budget

Per run/hour/day depending platform.

---

# 159. Hard Rule

Unexpected large transfer blocked.

---

# 160. Storage Write Budget

Protects flash/endurance and DB.

---

# 161. Hard Rule

No write amplification storm.

---

# 162. Wakeup Budget

```rust
pub struct ExtensionWakeupBudget {
    pub max_wakeups_per_hour: u32,
    pub max_exact_wakeups_per_day: u32,
}
```

---

# 163. Hard Rule

Wakeup budget enforced.

---

# 164. Exact Wakeup

Reserved for genuinely time-sensitive semantics.

---

# 165. Hard Rule

No exact scheduling for ordinary polling.

---

# 166. Wakeup Coalescing

Multiple extension jobs may share one device wake.

---

# 167. Hard Rule

Coalescing must not leak cross-extension data.

---

# 168. Background Network Batch

Optional batching for efficient radio wake.

---

# 169. Hard Rule

Privacy route requirements remain per job.

---

# 170. User Visibility

Users/admins can inspect:

```text
extensions with background permission
job classes
wake frequency class
network usage class
last coarse execution status
```

---

# 171. Hard Rule

No detailed surveillance timeline by default.

---

# 172. Background Permission

Separate install/runtime grant.

---

# 173. Hard Rule

Extension cannot run background work unless explicitly allowed.

---

# 174. Background Permission Class

```rust
pub enum ExtensionBackgroundPermission {
    ScheduledWork,
    EventTriggeredWork,
    NetworkBackgroundWork,
    PersistentDaemonWork,
}
```

---

# 175. Hard Rule

PersistentDaemonWork highest scrutiny.

---

# 176. User Disable

User/admin can disable extension background execution.

---

# 177. Hard Rule

Disabling cancels/suspends queued work safely.

---

# 178. Disable Semantics

```rust
pub enum BackgroundDisableMode {
    Pause,
    CancelPending,
    StopAndRevoke,
}
```

---

# 179. Hard Rule

UI explains data/result consequences.

---

# 180. Tenant Policy

Managed environment may restrict:

```text
allowed triggers
max frequency
network class
background capability set
```

---

# 181. Hard Rule

Tenant policy can restrict, not broaden.

---

# 182. Device Policy

Battery saver / enterprise device policy may further restrict.

---

# 183. Hard Rule

Extension consent cannot override OS/device denial.

---

# 184. Privacy Mode Policy

Maximum anonymity may prohibit certain scheduled external network jobs.

---

# 185. Hard Rule

No covert periodic beacon.

---

# 186. Polling

Discouraged when event-driven alternative exists.

---

# 187. Hard Rule

Use push/event/subscription instead of short-interval polling.

---

# 188. External Polling

Requires explicit destination/network permission.

---

# 189. Hard Rule

Polling interval bounded.

---

# 190. Polling Jitter

Useful for fleet privacy/load smoothing.

---

# 191. Hard Rule

Jitter must not weaken exact semantic deadline if one exists.

---

# 192. Webhook Retry

Durable background job.

---

# 193. Hard Rule

Signed payload/replay protection from Part 130/131 applies.

---

# 194. Retry Queue

Bounded.

---

# 195. Hard Rule

Permanent HTTP errors stop retry according to policy.

---

# 196. Local Indexing

Background CPU/storage job.

---

# 197. Hard Rule

Runs local-only unless extension has explicit network sink.

---

# 198. Maintenance Jobs

Examples:

```text
cache cleanup
index compaction
retention expiry
state migration
```

---

# 199. Hard Rule

Maintenance cannot access broader data than owning extension policy.

---

# 200. Retention / Deletion Jobs

Part 136.

---

# 201. Hard Rule

Privacy/deletion jobs get protected execution priority relative to optional extension jobs.

---

# 202. Security Revocation Job

Revocation enforcement should not wait behind normal background queue.

---

# 203. Hard Rule

Security/privacy governance jobs outrank extension automation.

---

# 204. Scheduler Priority Precedence

```text
security/privacy enforcement
> core reliability
> user-visible interactive deferred
> normal extension jobs
> maintenance/bulk extension work
```

---

# 205. Hard Rule

Extensions cannot alter this precedence.

---

# 206. Job Registration API

```rust
pub trait ExtensionJobService {
    fn register(
        &self,
        definition: ExtensionJobDefinition,
    ) -> Result<ExtensionJobId, ExtensionJobError>;

    fn cancel(
        &self,
        job: ExtensionJobId,
    ) -> Result<(), ExtensionJobError>;
}
```

---

# 207. Execution API

```rust
pub trait ExtensionJobExecutor {
    async fn execute(
        &self,
        run: ExtensionJobRun,
    ) -> Result<ExtensionJobCompletionReceipt, ExtensionJobError>;
}
```

---

# 208. Scheduler API

```rust
pub trait ExtensionScheduler {
    fn next_ready(
        &self,
        now: Timestamp,
        capacity: SchedulerCapacity,
    ) -> Result<Vec<ExtensionJobRun>, ExtensionJobError>;
}
```

---

# 209. Wakeup API

```rust
pub trait ExtensionWakeupController {
    fn request(
        &self,
        request: ExtensionWakeupRequest,
    ) -> Result<WakeupReceipt, ExtensionJobError>;
}
```

---

# 210. Background Policy API

```rust
pub trait ExtensionBackgroundPolicyService {
    fn effective_policy(
        &self,
        extension: ExtensionId,
    ) -> Result<EffectiveBackgroundPolicy, ExtensionJobError>;
}
```

---

# 211. Error Taxonomy

```rust
pub enum ExtensionJobError {
    JobUnknown,
    TriggerInvalid,
    ScheduleTooFrequent,
    BackgroundPermissionDenied,
    CapabilityDenied,
    ConstraintUnsatisfied,
    ResourceBudgetExceeded,
    WakeupBudgetExceeded,
    RetryExhausted,
    PayloadVersionUnsupported,
    JobExpired,
    Quarantined,
    Unauthorized,
    Internal,
}
```

---

# 212. Job Quarantine

Repeated abuse/crash/resource violation can quarantine jobs or entire extension background execution.

---

# 213. Hard Rule

Quarantine state visible to user/admin.

---

# 214. Abuse Detection

Based on:

```text
wake frequency
retry storms
resource overruns
spawn fan-out
policy violations
```

not content.

---

# 215. Hard Rule

No payload-content surveillance for ordinary scheduler enforcement.

---

# 216. Background Observability

Safe metrics:

```text
jobs queued
jobs run
success/failure class
wakeups
CPU time class
network bytes class
retry count
```

---

# 217. Forbidden:

```text
private payloads
user behavior
contact graph
exact message content
```

---

# 218. Hard Rule

No per-user automation dossier.

---

# 219. Coarse Last Run

UI can show:

```text
recently
today
failed
paused
```

instead of detailed timeline if precision unnecessary.

---

# 220. Hard Rule

Minimize temporal fingerprinting.

---

# 221. SLOs

Examples:

```text
eligible durable job begins within target window
revocation cancels background authority within target
retry queue remains bounded
mobile wakeup count remains within policy
```

---

# 222. Security SLO

```text
0 background job executes with expired/revoked grant
0 plaintext secret persisted in job payload
0 extension creates unlimited wakeups
```

---

# 223. Privacy SLO

```text
0 covert background camera/microphone access
0 silent privacy-route downgrade
0 user-behavior tracking through hidden periodic jobs
```

---

# 224. Failure Modes

```text
retry storm
wake storm
stale capability
job duplication
mobile OS deferral
```

---

# 225. Retry Storm

Bound attempts + jitter + circuit breaker.

---

# 226. Wake Storm

Per-extension/global wake budgets + coalescing.

---

# 227. Stale Capability

Grant epoch checked before execution and before side-effect commit.

---

# 228. Job Duplication

Idempotency + durable run state.

---

# 229. OS Deferral

Reported as Deferred, not Failed.

---

# 230. Circuit Breaker

Optional for repeated external dependency failure.

```rust
pub enum ExtensionCircuitState {
    Closed,
    Open,
    HalfOpen,
}
```

---

# 231. Hard Rule

Circuit breaker scoped per integration/destination where possible.

---

# 232. Dependency Health

Do not probe continuously.

---

# 233. Hard Rule

Use bounded health windows.

---

# 234. Batch Scheduling

Combine compatible jobs when safe.

---

# 235. Hard Rule

Do not combine jobs across isolation boundaries in ways that mix data.

---

# 236. Scheduler Fairness

Weighted fair scheduling across extensions.

---

# 237. Hard Rule

One extension cannot monopolize all background capacity.

---

# 238. Deadline Jobs

Some jobs have latest completion deadline.

---

# 239. Deadline Record

```rust
pub struct ExtensionJobDeadline {
    pub latest_start: Option<Timestamp>,
    pub latest_finish: Option<Timestamp>,
}
```

---

# 240. Hard Rule

Deadline cannot elevate extension above security/privacy/core reliability work.

---

# 241. Deadline Miss

Explicit result.

---

# 242. Hard Rule

No late execution if semantics prohibit it.

---

# 243. Time Privacy

Avoid unnecessary exact timestamps in extension-facing status.

---

# 244. Hard Rule

Jobs cannot use scheduler as high-resolution clock side channel beyond policy.

---

# 245. Timer Granularity

Coarsen for anonymous mode where exactness not required.

---

# 246. Hard Rule

No high-frequency timer API for extensions.

---

# 247. Monotonic Clock

Use for durations/retries.

---

# 248. Hard Rule

Wall clock only where business semantics need it.

---

# 249. Time Zone

Explicit for user-facing schedules.

---

# 250. Hard Rule

Do not infer location from time zone silently.

---

# 251. Cron-Like Schedules

Optional human-facing syntax.

---

# 252. Internal Model

Compile cron into typed schedule.

---

# 253. Hard Rule

No raw cron string interpretation on hot path.

---

# 254. DST Behavior

Explicit.

---

# 255. Hard Rule

Schedule semantics define skip/duplicate behavior around clock changes.

---

# 256. Example DST Policy

```rust
pub enum DstPolicy {
    SkipMissing,
    RunOnceOnNextValid,
    DeduplicateRepeated,
}
```

---

# 257. Hard Rule

User-visible schedules document DST semantics.

---

# 258. Time-Window Scheduling

```rust
pub struct FlexibleTimeWindow {
    pub earliest: Timestamp,
    pub latest: Timestamp,
}
```

---

# 259. Hard Rule

Scheduler may choose efficient execution point within window.

---

# 260. Persistent Daemon Extensions

Rare.

---

# 261. Hard Rule

PersistentDaemonWork requires stronger review/certification.

---

# 262. Daemon Heartbeat

Coarse health only.

---

# 263. Hard Rule

No tight heartbeat.

---

# 264. Auto-Suspend

Inactive daemon extension can be suspended.

---

# 265. Hard Rule

No permanent resident process without explicit product need.

---

# 266. Foreground Promotion

If background work becomes user-visible/long-running, runtime may promote to foreground mode where OS supports.

---

# 267. Hard Rule

Promotion requires truthful user-visible state.

---

# 268. Background-to-Foreground Capability

Does not broaden data permissions.

---

# 269. Hard Rule

Foreground status changes execution allowance, not authorization scope.

---

# 270. Cancellation

User/admin/platform may cancel job.

---

# 271. Hard Rule

Cancellation is idempotent.

---

# 272. Cancellation Safe Point

Long-running jobs periodically check cancellation.

---

# 273. Hard Rule

No indefinite non-cancellable extension task.

---

# 274. Compensating Cleanup

Job may register cleanup for temporary resources.

---

# 275. Hard Rule

Cleanup budget bounded.

---

# 276. Cancellation And External Side Effects

Do not imply rollback if already committed.

---

# 277. Hard Rule

Completion receipt records committed effects.

---

# 278. Job Result Data

Minimal.

---

# 279. Hard Rule

Do not persist private result payload if only status needed.

---

# 280. Result Retention

Short by default.

---

# 281. Hard Rule

Job history is not a behavioral analytics source.

---

# 282. User/Developer Debugging

Provide:

```text
job class
status
error class
next retry
quota state
```

not private payload.

---

# 283. Hard Rule

No full job payload view in generic support UI.

---

# 284. Support Bundle

Redacted.

---

# 285. Hard Rule

No secrets/message content.

---

# 286. Job Definition Versioning

```rust
pub struct ExtensionJobDefinitionVersion(pub u32);
```

---

# 287. Hard Rule

Old job definitions migrate explicitly.

---

# 288. Extension Update

Updating extension may change job definitions.

---

# 289. Hard Rule

Scheduler computes diff:

```text
added jobs
removed jobs
frequency changes
capability changes
network changes
```

---

# 290. Background Scope Expansion

Requires review/re-consent.

---

# 291. Hard Rule

Silent new periodic job is forbidden.

---

# 292. Job Removal

Cancels pending future runs.

---

# 293. Hard Rule

In-flight run policy explicit.

---

# 294. Package Revocation

All background jobs paused/cancelled and grants invalidated.

---

# 295. Hard Rule

Revoked extension cannot continue daemon/background work.

---

# 296. Uninstall

Cancels all jobs.

---

# 297. Hard Rule

No orphan scheduled jobs.

---

# 298. Reinstall

Does not restore high-risk background grants automatically.

---

# 299. Hard Rule

Backup/Restore

Backups may restore job definitions/history, not live wakeups/tokens.

---

# 300. Hard Rule

Restored jobs revalidate permission/policy before rescheduling.

---

# 301. Account Recovery

Fresh device/runtime security state required.

---

# 302. Hard Rule

No recovered stale background tokens.

---

# 303. Multi-Device Scheduling

Job scope may be:

```rust
pub enum ExtensionJobExecutionScope {
    ThisDevice,
    AnyOwnedDevice,
    TenantWorker,
}
```

---

# 304. Hard Rule

Default is ThisDevice for device-local automation.

---

# 305. AnyOwnedDevice

Requires dedup/leader selection.

---

# 306. Hard Rule

Only one executor commits logical side effect.

---

# 307. Device Election

Use deterministic/lease-based selection.

---

# 308. Hard Rule

No permanent global leader identity if unnecessary.

---

# 309. Tenant Worker

Server-side managed automation.

---

# 310. Hard Rule

Tenant worker uses tenant-scoped capability and data policy.

---

# 311. Cross-Device Privacy

Scheduling metadata minimized.

---

# 312. Hard Rule

Do not expose detailed device activity history.

---

# 313. Scheduler Policy

```rust
pub struct ExtensionSchedulerPolicy {
    pub min_periodic_interval: Duration,
    pub max_jobs_per_extension: u32,
    pub max_concurrent_per_extension: u32,
    pub wakeup_budget: ExtensionWakeupBudget,
}
```

---

# 314. Signed/versioned.

---

# 315. Policy Epoch

```rust
pub struct ExtensionSchedulerPolicyEpoch(pub u64);
```

---

# 316. Hard Rule

Anti-rollback enforced.

---

# 317. Stale Policy

No permissive fail-open.

---

# 318. Hard Rule

Stricter policy can pause jobs immediately.

---

# 319. Job Registry

```rust
pub trait ExtensionJobRegistry {
    fn get(
        &self,
        job: ExtensionJobId,
    ) -> Result<ExtensionJobDefinition, ExtensionJobError>;

    fn list_for_extension(
        &self,
        extension: ExtensionId,
    ) -> Result<Vec<ExtensionJobDefinition>, ExtensionJobError>;
}
```

---

# 320. Hard Rule

Registry excludes user private payloads where unnecessary.

---

# 321. Durable Job Store

Stores minimal execution metadata.

---

# 322. Hard Rule

No full message/content copy into job row unless semantics require it.

---

# 323. Payload Handle

Prefer opaque data refs.

---

# 324. Hard Rule

Referenced data reauthorized at execution time.

---

# 325. This Prevents

```text
permission revocation bypass
stale data reuse
unbounded payload duplication
```

---

# 326. Testing

Need extension-background testkit.

---

# 327. Test Scenarios

```text
periodic sync
one-shot delayed job
event-triggered local job
Android deferred job
retry-after rate limit
```

---

# 328. Permission Test

Background run denied after grant revoked.

---

# 329. Wake Budget Test

Excess wakeups rejected/coalesced.

---

# 330. Retry Test

Retry count bounded.

---

# 331. Idempotency Test

Crash/retry produces one logical side effect.

---

# 332. Unknown State Test

Crash after uncertain commit does not blindly duplicate.

---

# 333. Battery Test

Optional job deferred at critical battery.

---

# 334. Thermal Test

CPU-heavy job throttled at critical thermal state.

---

# 335. Network Test

Unmetered-only job does not run on metered connection.

---

# 336. Anonymous Mode Test

Strict-anonymity job waits rather than direct-fallback.

---

# 337. Event Trigger Test

Event metadata trigger does not imply payload read permission.

---

# 338. Fan-Out Test

Recursive spawning bounded.

---

# 339. Update Diff Test

New periodic job requires review.

---

# 340. Revocation Test

Revoked package has no running/queued background work.

---

# 341. Restore Test

Restored job definitions revalidate grants before rescheduling.

---

# 342. Privacy Test

Job history does not expose detailed user activity.

---

# 343. Fuzzing

Fuzz:

```text
job definitions
trigger expressions
retry policies
schedule windows
payload envelopes
```

---

# 344. Property Tests

Properties:

```text
no background run can start without active background permission
retry count can never exceed configured maximum
child job graph can never exceed fanout/depth budget
revoked capability can never authorize committed background side effect
```

---

# 345. Formal Verification Targets

Strong candidates:

```text
job lifecycle
retry/idempotency
lease/fencing
revocation during execution
```

---

# 346. Kani Candidate

schedule/resource/attempt/grant invariants.

---

# 347. TLA+ Candidate

```text
register → wait → ready → run → retry/succeed/cancel/revoke
```

---

# 348. Loom Candidate

Concurrent:

```text
run
grant revoke
job cancel
completion commit
```

---

# 349. Performance

Scheduler overhead must remain low.

---

# 350. Targets

```text
idle CPU near zero
indexed next-ready query
batched timer handling
bounded executor pools
```

---

# 351. Hard Rule

No periodic full-table scan.

---

# 352. Timing Wheel / Heap

Use appropriate timer structure for local scheduler.

---

# 353. Hard Rule

Server scheduler can use indexed due-time queue.

---

# 354. Batching

Batch compatible wakeups/job dispatch where safe.

---

# 355. Hard Rule

No batching across privacy/isolation boundaries that mixes data.

---

# 356. Storage

Separate:

```text
job definitions
trigger checkpoints
next-run metadata
attempt state
leases/fencing
completion receipts
policy epochs
```

---

# 357. Secrets separate.

---

# 358. Hard Rule

No private behavioral data warehouse.

---

# 359. Partitioning

By:

```text
extension
device
tenant
job class
```

No user analytics partition.

---

# 360. Crate Layout

Recommended:

```text
crates/
├── siar-extension-jobs-core/
├── siar-extension-scheduler/
├── siar-extension-trigger/
├── siar-extension-wakeup/
├── siar-extension-job-executor/
├── siar-extension-retry/
├── siar-extension-job-policy/
├── siar-extension-mobile-adapter/
├── siar-extension-job-observability/
└── siar-extension-job-testkit/
```

---

# 361. `siar-extension-jobs-core`

Owns:

```text
ExtensionJobId
ExtensionJobState
ExtensionTrigger
ExtensionJobError
```

---

# 362. `siar-extension-scheduler`

Due-time ordering/admission/fairness.

---

# 363. `siar-extension-trigger`

Time/event/deferred trigger evaluation.

---

# 364. `siar-extension-wakeup`

Platform-neutral wakeup requests/budgets.

---

# 365. `siar-extension-job-executor`

Runtime invocation, cancellation, completion.

---

# 366. `siar-extension-retry`

Retry/jitter/idempotency/circuit breaker.

---

# 367. `siar-extension-job-policy`

Resource quotas, privacy/tenant/device overlays.

---

# 368. `siar-extension-mobile-adapter`

Android scheduler/lifecycle bridge.

---

# 369. `siar-extension-job-observability`

Aggregate background execution health only.

---

# 370. `siar-extension-job-testkit`

schedule/retry/revocation/mobile/privacy/formal tests.

---

# 371. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. Extension job definitions, trigger registrations, wakeups, execution runs, retries, completion receipts, and runtime capability grants are distinct states and cannot be conflated.
2. Every background execution requires explicit active background permission plus currently valid attenuated data/capability scope; background work can never exceed foreground authority or revive revoked permissions.
3. Periodic scheduling, wakeups, retries, concurrency, fan-out, CPU, memory, storage, and network consumption are all explicitly bounded; extensions cannot create unlimited polling, wake locks, retry storms, or job graphs.
4. Mobile operating-system deferral is treated as scheduling delay rather than failure, and SIAR never promises exact background timing where the platform cannot guarantee it.
5. Sensitive device permissions—camera, microphone, clipboard, local discovery—are unavailable to generic background execution by default and require stronger explicit user-visible policy if ever supported.
6. Strict privacy/anonymity mode requirements apply equally in background execution; jobs wait, fail, or expire when compliant paths are unavailable and never silently downgrade to direct or less-private transport.
7. Retryable side effects use durable job state, bounded retry policy, and idempotency/fencing where applicable; crash or timeout uncertainty is never treated as definite failure when a side effect may already have committed.
8. Event triggers provide only the trigger projection they are authorized to reveal; receiving a trigger never implicitly grants access to message content, contact data, files, or other sensitive payloads.
9. Platform/tenant/device/privacy policy, battery/thermal/network constraints, security revocation, and user background-disable controls may immediately reduce or pause execution authority; extension consent cannot override them.
10. Package revocation, uninstall, account/device recovery, extension update, or permission migration cannot leave orphan jobs, stale wakeups, reusable runtime tokens, hidden daemons, or persistent background access.
11. Background job history, metrics, status, support diagnostics, and scheduling metadata are minimized and technical; they cannot become a detailed user activity timeline, behavioral profile, or covert extension tracking system.
12. Extension automation integrates with runtime sandboxing, permissions, data governance, IPC, secret brokerage, notification/background OS lifecycle, capacity/resource governance, privacy routing, assurance, risk/PIR, backup/restore, and marketplace revocation without creating a side channel around SIAR's security, privacy, anonymity, local-first, or tenant-isolation guarantees.
```

---

# 372. Initial Production Scope

Implement first:

```text
typed job/trigger/state model
durable local job store
one-shot/periodic/event/deferred triggers
minimum interval/flex windows
background permission class
attenuated job capability scope
bounded retry with exponential backoff/jitter
idempotency keys
crash reconciliation/Unknown state
resource quotas
per-extension/global concurrency
wakeup budgets
battery/thermal/network constraints
Android WorkManager/JobScheduler adapter
desktop daemon scheduler
offline queue semantics
event-trigger debouncing/coalescing
job expiry
fan-out/depth limits
revocation/uninstall cleanup
privacy-safe background observability
extension-job testkit
```

Then add:

```text
multi-device job election
tenant/server worker execution
workflow DAG/saga engine
adaptive wakeup batching
portable offline scheduler bundles
formal idempotency/fencing verification
advanced energy-aware scheduling
```

---

# 373. Definition of Done

Part 138 is complete when:

- background jobs have typed durable lifecycle state;
- triggers are explicit and bounded;
- periodic frequency has a floor;
- background authority is attenuated;
- mobile timing is represented truthfully;
- retries are bounded and idempotent where needed;
- battery/thermal/network constraints are enforced;
- strict privacy routing never silently downgrades;
- wakeups/concurrency/fan-out are bounded;
- revocation/uninstall removes queued/running authority;
- restored jobs revalidate permission;
- support/metrics do not become user activity surveillance;
- scheduling/retry/revocation/mobile/privacy/fuzz/formal tests are specified.

---

# 374. Final Architecture

```text
                  EXTENSION INTENT
                         │
                         ▼
                  DURABLE JOB MODEL
                         │
             ┌───────────┼───────────┐
             │           │           │
           TIME        EVENT      CONDITIONS
             │           │           │
             └───────────┼───────────┘
                         ▼
                  POLICY / ADMISSION
                         │
                         ▼
                 WAKEUP / SCHEDULER
                         │
                         ▼
               ATTENUATED JOB RUNTIME
                         │
                         ▼
              RETRY / COMPLETE / CANCEL
```

Background-execution safety model:

```text
explicit triggers
+
durable job state
+
attenuated capabilities
+
bounded retries
+
wakeup/resource quotas
+
battery/network awareness
+
mobile lifecycle truthfulness
+
revocation-aware execution
```

not:

```text
run forever in the background, poll constantly, keep old permissions, ignore battery/network constraints, and call the resulting activity "automation"
```

---

# 375. Final Principle

Background automation is trustworthy when the platform can explain **why a job exists, what wakes it, what authority it has, how much resource it may consume, when it stops, and what happens after failure or revocation**.

The correct model is:

```text
register explicit jobs
+
use event/deferred scheduling instead of polling
+
attenuate background authority
+
bound wakeups and retries
+
respect mobile/battery/network lifecycle
+
persist before side effects
+
use idempotency
+
cancel/revoke deterministically
+
never turn background execution into hidden surveillance
```

This architecture gives SIAR a privacy-preserving background automation foundation for durable jobs, triggers, scheduling, wakeups, mobile lifecycle, retries, quotas, offline execution, crash recovery, and extension automation while preserving the anonymity, local-first, least-authority, runtime, permission, data-governance, communication, and anti-surveillance guarantees established across Parts 34–137.
