# Core System Architecture Part 145 — Anonymous Network Extension Observability, Runtime Health, Crash Reporting, Diagnostics, Developer Telemetry, SLOs & Privacy-Preserving Extension Operations Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 145  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 51, 63, 92, 94–98, 109, 112, 120–126, 133–144

**Primary purpose:** define SIAR's extension operations architecture for extension-scoped observability, runtime health, crash reporting, diagnostics, developer telemetry, SLOs/SLIs, sampling, redaction, incident triggers, support bundles, privacy-preserving aggregation, and anti-surveillance operational insight.

---

# 1. Purpose

Extensions need operational visibility.

Developers and operators must be able to answer:

```text
Is the extension running?
Is it crashing?
Is it slow?
Is it hitting resource limits?
Are permission denials expected?
Are updates causing regressions?
Is background work failing?
```

But observability can easily become surveillance.

Poorly designed telemetry can expose:

```text
message contents
contact graphs
user behavior
device fingerprints
cross-extension activity
anonymous routing metadata
private file names
```

The governing principle is:

> **SIAR extension observability must reveal enough technical state to operate, debug, qualify, and support extensions while remaining extension-scoped, privacy-minimized, bounded in cardinality, and structurally incapable of becoming user-behavior monitoring or cross-extension tracing.**

---

# 2. Architectural Position

```text
                 EXTENSION RUNTIME
                        │
                        ▼
              OBSERVABILITY ADAPTER
                        │
          ┌─────────────┼─────────────┐
          │             │             │
       METRICS         LOGS         HEALTH
          │             │             │
          └─────────────┼─────────────┘
                        ▼
                 REDACTION / POLICY
                        │
                        ▼
                 LOCAL AGGREGATION
                        │
              ┌─────────┼─────────┐
              │                   │
          USER/ADMIN           DEVELOPER
              │                   │
              └─────────┬─────────┘
                        ▼
                 SUPPORT / SLO / INCIDENT
```

---

# 3. Core Separation

Keep distinct:

```text
health
metrics
logs
traces
crash report
diagnostic bundle
developer analytics
user support data
incident evidence
```

---

# 4. Non-Goals

Part 145 does not create:

```text
full-session recording
global distributed tracing across anonymous paths
keystroke/clickstream telemetry
message-content logging
developer access to individual user behavior
```

---

# 5. Observability Identity

```rust
pub struct ExtensionObservabilityId(pub [u8; 16]);
```

---

# 6. Extension Scope

Every observability signal binds to:

```rust
pub struct ExtensionObservabilityScope {
    pub extension: ExtensionId,
    pub version: ExtensionVersion,
    pub runtime: Option<ExtensionRuntimeId>,
    pub device_class: Option<PlatformClass>,
}
```

---

# 7. Hard Rule

No signal exists without explicit extension scope.

---

# 8. Signal Class

```rust
pub enum ExtensionObservabilitySignalClass {
    Health,
    Metric,
    Log,
    Crash,
    Diagnostic,
    SloMeasurement,
    IncidentIndicator,
}
```

---

# 9. Data Classification

```rust
pub enum ExtensionObservabilityDataClass {
    PublicTechnical,
    InternalTechnical,
    RestrictedTechnical,
    Forbidden,
}
```

---

# 10. Forbidden Examples

```text
message body
clipboard content
API secret
private attachment content
contact/social graph
precise route metadata
```

---

# 11. Hard Rule

Forbidden data never enters observability pipeline.

---

# 12. Runtime Health

```rust
pub enum ExtensionRuntimeHealth {
    Starting,
    Healthy,
    Degraded,
    Unhealthy,
    Quarantined,
    Stopped,
    Unknown,
}
```

---

# 13. Hard Rule

Unknown is not Healthy.

---

# 14. Health Snapshot

```rust
pub struct ExtensionHealthSnapshot {
    pub extension: ExtensionId,
    pub runtime: ExtensionRuntimeId,
    pub state: ExtensionRuntimeHealth,
    pub reason: Option<ExtensionHealthReason>,
}
```

---

# 15. Health Reason

```rust
pub enum ExtensionHealthReason {
    StartupFailure,
    CrashLoop,
    ResourceThrottled,
    DependencyUnavailable,
    PermissionBlocked,
    CompatibilityIssue,
    PolicyBlocked,
    StorageIssue,
    NetworkIssue,
}
```

---

# 16. Hard Rule

Health reasons are coarse technical classes.

---

# 17. No User Blame

Do not encode user identity or behavior as health reason.

---

# 18. Heartbeat

Optional.

---

# 19. Hard Rule

Heartbeat frequency bounded and adaptive.

---

# 20. Idle Extension

May have no active heartbeat.

---

# 21. Hard Rule

Absence of heartbeat does not equal failure unless architecture requires it.

---

# 22. Pull Health

Host can inspect local runtime state directly.

---

# 23. Hard Rule

Prefer direct local state over network heartbeat where possible.

---

# 24. Metrics

Use bounded cardinality.

---

# 25. Metric Definition

```rust
pub struct ExtensionMetricDefinition {
    pub name: ExtensionMetricName,
    pub kind: ExtensionMetricKind,
    pub labels: BTreeSet<ExtensionMetricLabelKey>,
}
```

---

# 26. Metric Kind

```rust
pub enum ExtensionMetricKind {
    Counter,
    Gauge,
    Histogram,
}
```

---

# 27. Safe Metrics

Examples:

```text
runtime_start_total
runtime_crash_total
host_call_latency
queue_depth
resource_throttle_total
background_job_failure_total
update_failure_total
```

---

# 28. Hard Rule

No per-user metric labels.

---

# 29. Forbidden Labels

```text
user ID
conversation ID
message ID
contact ID
full URL
file path
raw error text
```

---

# 30. Hard Rule

Label schema predeclared.

---

# 31. Cardinality Budget

```rust
pub struct ExtensionMetricCardinalityBudget {
    pub max_series: u32,
    pub max_label_values_per_key: u32,
}
```

---

# 32. Hard Rule

Cardinality enforced per extension.

---

# 33. Histogram Buckets

Static/policy-defined.

---

# 34. Hard Rule

No extension-defined arbitrary bucket explosion.

---

# 35. Metrics Scope

Prefer aggregate dimensions:

```text
extension version
platform class
error class
operation class
```

---

# 36. Logs

Structured, extension-scoped.

---

# 37. Log Record

```rust
pub struct ExtensionLogRecord {
    pub level: ExtensionLogLevel,
    pub event_code: ExtensionLogEventCode,
    pub fields: BTreeMap<ExtensionLogFieldKey, ExtensionLogFieldValue>,
}
```

---

# 38. Log Level

```rust
pub enum ExtensionLogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}
```

---

# 39. Hard Rule

Event code preferred over free-form log text for operational paths.

---

# 40. Debug Text

Allowed only in local/developer mode with redaction.

---

# 41. Hard Rule

Production extension logs never contain secrets/private content.

---

# 42. Structured Error Class

```rust
pub enum ExtensionOperationalErrorClass {
    CapabilityDenied,
    PolicyDenied,
    Timeout,
    ResourceLimit,
    DependencyFailure,
    NetworkFailure,
    StorageFailure,
    CompatibilityFailure,
    InternalFailure,
}
```

---

# 43. Hard Rule

Do not log raw third-party response bodies by default.

---

# 44. File Paths

Normalize/redact.

---

# 45. Hard Rule

No full home-directory path in telemetry.

---

# 46. Network Destinations

Use origin class or approved hostname only where necessary.

---

# 47. Hard Rule

No raw IP/path metadata in anonymous mode.

---

# 48. Log Retention

Short by default.

---

# 49. Hard Rule

Logs are operational data, not archive.

---

# 50. Local Ring Buffer

Recommended for recent logs.

---

# 51. Ring Buffer Limits

```rust
pub struct ExtensionLogBufferPolicy {
    pub max_records: u32,
    pub max_bytes: u64,
    pub max_age: Duration,
}
```

---

# 52. Hard Rule

Bounded in count, bytes, and age.

---

# 53. Crash Reporting

First-class.

---

# 54. Crash Identity

```rust
pub struct ExtensionCrashId(pub [u8; 16]);
```

---

# 55. Crash Report

```rust
pub struct ExtensionCrashReport {
    pub crash_id: ExtensionCrashId,
    pub extension: ExtensionId,
    pub version: ExtensionVersion,
    pub class: ExtensionCrashClass,
    pub environment: ExtensionCrashEnvironment,
}
```

---

# 56. Crash Environment

```rust
pub struct ExtensionCrashEnvironment {
    pub platform: PlatformClass,
    pub host_version: Version,
    pub sdk_version: SdkVersion,
    pub privacy_mode: PrivacyRoutingMode,
}
```

---

# 57. Hard Rule

Crash report excludes private user state.

---

# 58. Stack Trace

Possible for extension code.

---

# 59. Hard Rule

Stack traces symbolize package code, not user data.

---

# 60. Native Crash Dump

High-risk.

---

# 61. Hard Rule

Full core dumps disabled by default.

---

# 62. Minidump

Preferred for native extension if supported.

---

# 63. Hard Rule

Minidump filtered/sanitized before upload.

---

# 64. WASM Trap Report

Contains:

```text
module digest
function index/name
trap class
bounded stack
```

---

# 65. Hard Rule

No linear-memory dump by default.

---

# 66. Panic Payload

Sanitize.

---

# 67. Hard Rule

Do not trust extension panic text as safe.

---

# 68. Crash Fingerprint

```rust
pub struct ExtensionCrashFingerprint(pub Digest);
```

---

# 69. Fingerprint Inputs

Use:

```text
extension version
crash class
symbolized stack frames
```

not user/device identity.

---

# 70. Hard Rule

Fingerprint cannot become cross-user tracking ID.

---

# 71. Crash Aggregation

Aggregate by fingerprint/version/platform.

---

# 72. Hard Rule

Developer sees counts/trends, not user identities.

---

# 73. Crash Upload

Opt-in/policy-governed where required.

---

# 74. Hard Rule

Local diagnostics remain useful without upload.

---

# 75. Crash Queue

Encrypted local pending queue.

---

# 76. Hard Rule

Bounded and expiring.

---

# 77. Diagnostic Bundle

Platform-controlled.

---

# 78. Bundle Identity

```rust
pub struct ExtensionDiagnosticBundleId(pub [u8; 16]);
```

---

# 79. Bundle Contents

Possible:

```text
extension/version
health state
recent redacted logs
resource usage summary
compatibility state
permission state summary
recent crash fingerprints
```

---

# 80. Hard Rule

No private message content, raw secrets, full file paths, or social graph.

---

# 81. Diagnostic Bundle Policy

```rust
pub struct ExtensionDiagnosticBundlePolicy {
    pub include_logs: bool,
    pub include_crash_summary: bool,
    pub include_resource_summary: bool,
    pub max_age: Duration,
}
```

---

# 82. Hard Rule

Bundle content deterministic from policy.

---

# 83. User Review

User/admin can preview high-level bundle categories before sharing where UX permits.

---

# 84. Hard Rule

Support upload is explicit.

---

# 85. Support Reference

Bundle can produce opaque support case reference.

---

# 86. Hard Rule

Support staff never need access to anonymous messaging identity.

---

# 87. Developer Telemetry

Developer-facing telemetry is aggregate.

---

# 88. Allowed Developer Dimensions

```text
extension version
platform
operation class
error class
crash fingerprint
```

---

# 89. Forbidden Developer Dimensions

```text
user identity
tenant identity unless explicitly admin-owned aggregate
message content
conversation ID
contact graph
precise location
```

---

# 90. Hard Rule

No per-user developer dashboard.

---

# 91. Enterprise Tenant Telemetry

Tenant admins may receive tenant-scoped technical aggregate if policy permits.

---

# 92. Hard Rule

Tenant admin does not receive personal/private content telemetry.

---

# 93. Local-Only Diagnostics

Some sensitive technical metrics can remain local.

---

# 94. Hard Rule

Not every metric needs server export.

---

# 95. Telemetry Export Policy

```rust
pub enum ExtensionTelemetryExportPolicy {
    LocalOnly,
    AggregateOnly,
    DiagnosticOnDemand,
}
```

---

# 96. Hard Rule

No arbitrary raw telemetry upload mode for third-party extension.

---

# 97. Sampling

First-class.

---

# 98. Sampling Policy

```rust
pub struct ExtensionSamplingPolicy {
    pub metric_sample_rate: f32,
    pub log_sample_rate: f32,
    pub crash_sample_rate: f32,
}
```

---

# 99. Hard Rule

Sampling policy platform-controlled.

---

# 100. Error Sampling

Critical errors may bypass ordinary log sampling but still obey redaction.

---

# 101. Hard Rule

Sampling does not bypass privacy classification.

---

# 102. Trace Model

Full distributed tracing is restricted.

---

# 103. Local Trace Span

```rust
pub struct ExtensionLocalSpan {
    pub span_id: ExtensionSpanId,
    pub operation: ExtensionOperationClass,
    pub started_at: MonotonicTimestamp,
}
```

---

# 104. Hard Rule

Span IDs local/session-scoped.

---

# 105. No Global Trace ID Across Anonymous Network

Hard rule.

---

# 106. Cross-Boundary Trace

Use coarse operation correlation if absolutely necessary.

---

# 107. Hard Rule

Never propagate stable trace IDs through anonymous/mixnet traffic.

---

# 108. Anonymous Mode Observability

Further minimize:

```text
exact timestamps
network route metadata
peer identifiers
device fingerprints
```

---

# 109. Hard Rule

Operational usefulness never justifies anonymity downgrade.

---

# 110. Time Bucketing

Use coarse windows where exact timestamps unnecessary.

---

# 111. Hard Rule

Avoid detailed temporal user-activity reconstruction.

---

# 112. Resource Telemetry

Track:

```text
CPU
memory
network bytes
storage bytes
queue depth
wakeups
```

---

# 113. Hard Rule

Resource metrics are extension/runtime aggregates.

---

# 114. No Per-User Resource Attribution

Hard rule.

---

# 115. Runtime Health Check

```rust
pub trait ExtensionHealthService {
    fn snapshot(
        &self,
        extension: ExtensionId,
    ) -> Result<ExtensionHealthSnapshot, ExtensionObservabilityError>;
}
```

---

# 116. Metric Service

```rust
pub trait ExtensionMetricService {
    fn record(
        &self,
        extension: ExtensionId,
        metric: ExtensionMetricPoint,
    ) -> Result<(), ExtensionObservabilityError>;
}
```

---

# 117. Log Service

```rust
pub trait ExtensionLogService {
    fn emit(
        &self,
        extension: ExtensionId,
        record: ExtensionLogRecord,
    ) -> Result<(), ExtensionObservabilityError>;
}
```

---

# 118. Crash Service

```rust
pub trait ExtensionCrashService {
    fn record(
        &self,
        report: ExtensionCrashReport,
    ) -> Result<ExtensionCrashId, ExtensionObservabilityError>;
}
```

---

# 119. Diagnostic Service

```rust
pub trait ExtensionDiagnosticService {
    fn build_bundle(
        &self,
        extension: ExtensionId,
        policy: ExtensionDiagnosticBundlePolicy,
    ) -> Result<ExtensionDiagnosticBundleId, ExtensionObservabilityError>;
}
```

---

# 120. SLI

Service Level Indicator.

Examples:

```text
runtime startup success
host-call latency
crash-free runtime sessions
background job success
update success
```

---

# 121. SLI Definition

```rust
pub struct ExtensionSliDefinition {
    pub sli_id: ExtensionSliId,
    pub name: String,
    pub numerator: ExtensionMetricExpression,
    pub denominator: ExtensionMetricExpression,
}
```

---

# 122. Hard Rule

SLI denominator explicit.

---

# 123. Missing Telemetry

Missing data yields Unknown.

---

# 124. Hard Rule

Missing data is never treated as success.

---

# 125. SLO

```rust
pub struct ExtensionSlo {
    pub slo_id: ExtensionSloId,
    pub sli: ExtensionSliId,
    pub target: FixedPointRatio,
    pub window: Duration,
}
```

---

# 126. Hard Rule

No floating vague SLO like “usually reliable”.

---

# 127. SLO Scope

Can be:

```text
extension version
platform
runtime class
operation class
```

---

# 128. Hard Rule

No user-scoped SLO.

---

# 129. Error Budget

Optional for mature extensions.

---

# 130. Hard Rule

Error budget governs release/change velocity, not user access or privacy.

---

# 131. Security/Privacy Hard Invariants

Never represented as error-budgeted objectives.

---

# 132. Hard Rule

“Some privacy failures are within budget” is forbidden.

---

# 133. Burn Rate

Can monitor technical availability/reliability SLOs.

---

# 134. Hard Rule

Burn rate uses aggregate signals only.

---

# 135. SLO Violation

May trigger:

```text
rollout pause
extension quarantine
investigation
developer alert
```

---

# 136. Hard Rule

SLO violation does not automatically expose more telemetry.

---

# 137. Developer Alert

Could notify publisher of:

```text
crash spike
compatibility regression
resource regression
update failure
```

---

# 138. Hard Rule

Alert contains aggregate technical context.

---

# 139. Incident Trigger

```rust
pub enum ExtensionIncidentTrigger {
    CrashSpike,
    SandboxViolation,
    SecurityPolicyViolation,
    PrivacyPolicyViolation,
    ResourceAbuse,
    UpdateRegression,
    DependencyRevocation,
}
```

---

# 140. Hard Rule

Incident trigger is technical evidence, not speculation about user behavior.

---

# 141. Incident Integration

Part 96/120.

---

# 142. Hard Rule

Extension incident records are scoped to extension/package/runtime.

---

# 143. No User Incident Profile

Hard rule.

---

# 144. Privacy Incident

Examples:

```text
unexpected external sink
forbidden metadata exposure
retention failure
lock-screen leak
```

---

# 145. Hard Rule

Privacy incident immediately raises investigation priority.

---

# 146. Security Incident

Examples:

```text
sandbox escape attempt
stale capability use
unauthorized file/network access
signature anomaly
```

---

# 147. Hard Rule

Security incident can quarantine extension independently from ordinary SLO state.

---

# 148. Telemetry Schema Registry

```rust
pub struct ExtensionTelemetrySchemaRegistry {
    pub version: ExtensionTelemetrySchemaVersion,
    pub metrics: BTreeMap<ExtensionMetricName, ExtensionMetricDefinition>,
    pub logs: BTreeMap<ExtensionLogEventCode, ExtensionLogSchema>,
}
```

---

# 149. Hard Rule

Telemetry schema is versioned and reviewed.

---

# 150. Unknown Telemetry Field

Rejected or dropped according to strict policy.

---

# 151. Hard Rule

No arbitrary map of extension-defined labels in production export.

---

# 152. Local Debug Mode

May enable richer logs locally.

---

# 153. Hard Rule

Local debug mode visibly indicated.

---

# 154. Developer Mode

User-controlled.

---

# 155. Hard Rule

Developer mode does not weaken sandbox/permissions.

---

# 156. Remote Debugging

High risk.

---

# 157. Hard Rule

Disabled by default in production third-party extensions.

---

# 158. If Enabled

Require:

```text
explicit user/admin action
short-lived session
scoped endpoint
no production secret exposure
```

---

# 159. Hard Rule

No hidden remote shell.

---

# 160. Support Workflow

```text
observe issue
generate redacted bundle
user/admin approves share
support receives bundle
developer sees aggregate/authorized technical data
```

---

# 161. Hard Rule

Support cannot request unrestricted device data through observability channel.

---

# 162. Diagnostic Query API

Bounded.

---

# 163. Hard Rule

No arbitrary SQL/log search over user devices.

---

# 164. Device Diagnostics

Local UI can show:

```text
extension health
recent crash count
resource limit status
compatibility
background state
```

---

# 165. Hard Rule

Avoid detailed user activity history.

---

# 166. Developer Dashboard

Can show:

```text
version adoption aggregate
crash rate
error class
resource regression
compatibility failures
```

---

# 167. Hard Rule

No user drill-down.

---

# 168. Publisher Segmentation

By version/platform only.

---

# 169. Hard Rule

No private cohort segmentation.

---

# 170. Telemetry Retention

```rust
pub struct ExtensionTelemetryRetentionPolicy {
    pub metrics_retention: Duration,
    pub logs_retention: Duration,
    pub crash_retention: Duration,
}
```

---

# 171. Hard Rule

Retention bounded.

---

# 172. Local Retention

Can be shorter.

---

# 173. Hard Rule

User can clear local diagnostic history where product policy permits.

---

# 174. Crash Retention

Long enough for debugging but not indefinite.

---

# 175. Hard Rule

Old crash raw data can be summarized then deleted.

---

# 176. Aggregation

Prefer:

```text
counts
histograms
rates
coarse platform/version dimensions
```

---

# 177. Hard Rule

No raw event stream by default.

---

# 178. Differential Privacy

Optional for very large aggregate ecosystem metrics.

---

# 179. Hard Rule

DP not required where ordinary aggregation/minimization is sufficient.

---

# 180. Telemetry Transport

Encrypted/authenticated.

---

# 181. Hard Rule

Telemetry transport identity separate from anonymous communication identity where feasible.

---

# 182. Offline

Telemetry can queue locally.

---

# 183. Hard Rule

Queue bounded and expiring.

---

# 184. On Reconnect

Send according to current consent/policy, not historical policy alone.

---

# 185. Hard Rule

If telemetry is now disabled, queued optional telemetry can be discarded.

---

# 186. Consent

Telemetry categories exposed clearly.

---

# 187. Hard Rule

Operational telemetry required for local safety can remain local without upload consent.

---

# 188. User-Visible Controls

Possible:

```text
share crash reports
share aggregate diagnostics
developer mode
clear local diagnostic history
```

---

# 189. Hard Rule

Controls reflect actual behavior.

---

# 190. No Fake Privacy Toggle

Hard rule.

---

# 191. Extension-Specific Consent

Optional.

---

# 192. Hard Rule

Extension publisher cannot override platform telemetry consent.

---

# 193. Tenant Policy

Can reduce external telemetry.

---

# 194. Hard Rule

Tenant policy cannot force personal-context private telemetry to publisher.

---

# 195. Enterprise Support

Tenant may allow scoped organization diagnostics.

---

# 196. Hard Rule

Data remains tenant-scoped.

---

# 197. Observability Policy

```rust
pub struct ExtensionObservabilityPolicy {
    pub allowed_signals: BTreeSet<ExtensionObservabilitySignalClass>,
    pub export_policy: ExtensionTelemetryExportPolicy,
    pub cardinality_budget: ExtensionMetricCardinalityBudget,
    pub retention: ExtensionTelemetryRetentionPolicy,
}
```

---

# 198. Signed/versioned.

---

# 199. Policy Epoch

```rust
pub struct ExtensionObservabilityPolicyEpoch(pub u64);
```

---

# 200. Hard Rule

Anti-rollback.

---

# 201. Stale Policy

No permissive fail-open.

---

# 202. Hard Rule

Stricter policy takes effect immediately for future telemetry.

---

# 203. Redaction Engine

```rust
pub trait ExtensionTelemetryRedactor {
    fn redact(
        &self,
        record: ExtensionTelemetryRecord,
    ) -> Result<ExtensionTelemetryRecord, ExtensionObservabilityError>;
}
```

---

# 204. Hard Rule

Redaction occurs before export.

---

# 205. Redaction Failure

Drop or quarantine signal.

---

# 206. Hard Rule

Never export unredacted fallback.

---

# 207. Secret Detection

Heuristic supplementary only.

---

# 208. Hard Rule

Schema design is primary privacy defense.

---

# 209. PII/Sensitive Scanner

Optional pre-export safeguard.

---

# 210. Hard Rule

Scanner false-negative risk means it cannot justify arbitrary free-form logs.

---

# 211. Telemetry Envelope

```rust
pub struct ExtensionTelemetryEnvelope {
    pub schema: ExtensionTelemetrySchemaVersion,
    pub extension: ExtensionId,
    pub version: ExtensionVersion,
    pub signals: Vec<ExtensionTelemetrySignal>,
}
```

---

# 212. Hard Rule

Envelope bounded by signal count and bytes.

---

# 213. Batch Upload

Preferred.

---

# 214. Hard Rule

No high-frequency beaconing.

---

# 215. Backoff

Telemetry upload uses bounded retry/backoff.

---

# 216. Hard Rule

Telemetry never competes with core messaging/realtime traffic.

---

# 217. Priority

Low/background by default.

---

# 218. Hard Rule

Security-critical local signals can trigger local action without immediate export.

---

# 219. Crash Loop Handling

Observability drives runtime quarantine.

---

# 220. Hard Rule

Quarantine decision based on local technical thresholds.

---

# 221. Threshold Policy

```rust
pub struct ExtensionCrashLoopPolicy {
    pub max_crashes: u32,
    pub window: Duration,
}
```

---

# 222. Hard Rule

No user-specific threshold.

---

# 223. Resource Regression

Compare version-level aggregate.

---

# 224. Hard Rule

No per-user ranking.

---

# 225. Update Integration

Part 143 rollout can use:

```text
crash rate
startup failure
migration failure
resource regressions
```

---

# 226. Hard Rule

Rollout telemetry remains aggregate/technical.

---

# 227. Certification Integration

Part 144 can ingest observability regressions as evidence invalidation signal.

---

# 228. Hard Rule

Observed regression can suspend certification pending review.

---

# 229. Marketplace Integration

Part 133 may hide/suspend extension when severe operational/security state requires it.

---

# 230. Hard Rule

Marketplace ranking does not use invasive engagement telemetry.

---

# 231. Developer Portal Integration

Part 131 dashboard gets only approved aggregate telemetry.

---

# 232. Hard Rule

Portal cannot drill into individual user/device.

---

# 233. Toolchain Integration

Part 132 local emulator can simulate telemetry.

---

# 234. Hard Rule

Synthetic telemetry clearly labeled.

---

# 235. Test Integration

Part 144 validates observability redaction/schema/cardinality.

---

# 236. Hard Rule

Observability itself is tested.

---

# 237. Observability Testkit

Required scenarios:

```text
private data in error path
crash report
cardinality attack
anonymous-mode trace
support bundle export
```

---

# 238. Redaction Test

Secret-like field never exported.

---

# 239. Cardinality Test

Unbounded label values rejected.

---

# 240. Crash Test

WASM trap report contains no linear-memory dump.

---

# 241. Native Crash Test

Minidump path sanitization works.

---

# 242. Privacy Mode Test

No route/IP/peer identifiers in exported telemetry.

---

# 243. Support Bundle Test

Bundle contains only allowed categories.

---

# 244. Developer Dashboard Test

No user identity dimension available.

---

# 245. Consent Test

Disabling optional telemetry stops future export.

---

# 246. Queue Test

Offline telemetry queue remains bounded.

---

# 247. Revocation Test

Revoked extension can no longer emit external telemetry, except platform-owned security evidence if policy permits.

---

# 248. SLO Test

Missing data produces Unknown.

---

# 249. Incident Test

Sandbox violation triggers local security incident without requiring user behavior data.

---

# 250. Fuzzing

Fuzz:

```text
telemetry envelopes
log fields
crash reports
diagnostic manifests
SLO expressions
```

---

# 251. Property Tests

Properties:

```text
forbidden observability fields can never pass export redaction
metric series count can never exceed configured cardinality budget
anonymous-mode telemetry can never contain raw route/peer identifiers
missing SLI data can never be counted as successful measurement
```

---

# 252. Formal Verification Targets

Strong candidates:

```text
telemetry policy precedence
redaction/export state
crash-loop quarantine
SLO state evaluation
```

---

# 253. Kani Candidate

schema/cardinality/redaction invariants.

---

# 254. TLA+ Candidate

```text
emit → redact → aggregate → export → retain/delete
```

---

# 255. Loom Candidate

Concurrent:

```text
runtime crash
telemetry flush
policy disable
quarantine
```

---

# 256. Performance

Observability must be low-overhead.

---

# 257. CPU Budget

Per extension telemetry path.

---

# 258. Hard Rule

Telemetry processing cannot dominate extension workload.

---

# 259. Memory Budget

Bound buffers.

---

# 260. Hard Rule

No unbounded log/metric queues.

---

# 261. Non-Blocking

Runtime telemetry emit should be non-blocking/bounded.

---

# 262. Hard Rule

If queue full, drop/coalesce according to signal policy.

---

# 263. Loss Semantics

Metrics can aggregate/drop safely; crash/security events have protected bounded channel.

---

# 264. Hard Rule

Protected channel still bounded.

---

# 265. Telemetry Sampling Under Load

Increase sampling/drop low-priority logs.

---

# 266. Hard Rule

Never drop required local security/quarantine signal without fallback state marker.

---

# 267. Storage

Separate:

```text
local metric aggregates
bounded log ring buffers
crash summaries
diagnostic bundle staging
SLO state
incident indicators
```

---

# 268. Hard Rule

No shared cross-extension raw telemetry lake.

---

# 269. Central Aggregate Store

If used, partition by:

```text
extension
version
platform
signal class
```

---

# 270. Hard Rule

No user/person partition.

---

# 271. Diagnostic Bundle Staging

Encrypted and short-lived.

---

# 272. Hard Rule

Auto-delete after upload/expiry.

---

# 273. Crate Layout

Recommended:

```text
crates/
├── siar-extension-observability-core/
├── siar-extension-health/
├── siar-extension-metrics/
├── siar-extension-logging/
├── siar-extension-crash/
├── siar-extension-diagnostics/
├── siar-extension-slo/
├── siar-extension-telemetry-policy/
├── siar-extension-observability-export/
├── siar-extension-observability-testkit/
└── siar-extension-operations-integration/
```

---

# 274. `siar-extension-observability-core`

Owns:

```text
ExtensionObservabilitySignalClass
ExtensionObservabilityDataClass
ExtensionObservabilityError
common IDs
```

---

# 275. `siar-extension-health`

Runtime health snapshots/reasons/quarantine signals.

---

# 276. `siar-extension-metrics`

Bounded metric schemas/cardinality/histograms.

---

# 277. `siar-extension-logging`

Structured logs/redaction/ring buffers.

---

# 278. `siar-extension-crash`

WASM traps/native minidumps/fingerprints.

---

# 279. `siar-extension-diagnostics`

User-approved support bundle generation.

---

# 280. `siar-extension-slo`

SLI/SLO/error-budget technical evaluation.

---

# 281. `siar-extension-telemetry-policy`

Signal classes/export policy/retention/consent.

---

# 282. `siar-extension-observability-export`

Batching/encryption/sampling/aggregate developer export.

---

# 283. `siar-extension-observability-testkit`

redaction/cardinality/crash/privacy/SLO tests.

---

# 284. `siar-extension-operations-integration`

Incident/update/certification/marketplace hooks.

---

# 285. Security, Privacy & Correctness Invariants

Mandatory:

```text
1. Extension health, metrics, logs, crash reports, diagnostic bundles, developer telemetry, SLO measurements, and incident evidence are distinct signal classes with independent schemas, retention, export policy, and privacy constraints.
2. All extension observability signals are extension/version/runtime scoped and use predeclared bounded schemas; user IDs, conversation IDs, message IDs, contact IDs, raw file paths, arbitrary URLs, secrets, private payloads, and unrestricted free-form label values are forbidden by default.
3. Metrics enforce strict series/cardinality budgets, logs use structured event codes, buffers are bounded by bytes/count/age, and telemetry backpressure can drop/coalesce low-priority signals rather than consuming unbounded memory or blocking core runtime work.
4. Crash reporting uses sanitized WASM traps or filtered native minidumps and never captures full extension memory, secret stores, message content, private file contents, or raw host process dumps by default.
5. Correlation/span/crash identifiers are local/session/extension scoped and cannot become stable user/device tracking IDs or global distributed tracing identifiers across anonymous/mixnet traffic.
6. Anonymous/private modes further reduce timestamps, network metadata, peer identifiers, route details, and device fingerprints; operational debugging can never justify weakening the active privacy/anonymity policy.
7. Developer-facing telemetry is aggregate and technical—version/platform/error class/crash fingerprint/resource class—and never supports per-user drill-down, clickstream reconstruction, private tenant discovery, individual device profiling, or behavioral targeting.
8. Diagnostic/support bundles are platform-generated, redacted before export, explicit to share, short-lived, encrypted in staging, and cannot act as a remote shell or unrestricted log/database query path into user devices.
9. SLI/SLO evaluation uses explicit denominators and treats missing telemetry as Unknown; security, privacy, sandbox isolation, tenant isolation, cryptographic integrity, or anonymity invariants can never be represented as consumable error budgets.
10. Observability policy is signed/versioned/anti-rollback, stricter policy applies immediately to future exports, optional queued telemetry honors current consent on reconnect, and redaction failure results in drop/quarantine rather than unredacted fallback.
11. Crash spikes, sandbox violations, privacy-policy violations, resource abuse, dependency revocation, and update regressions may trigger quarantine, rollout pause, evidence invalidation, or incident response without requiring private content inspection or user-level behavioral surveillance.
12. Extension observability integrates with runtime supervision, permissions, data governance, IPC, background jobs, notifications, UI, state sync, dependencies, updates, certification, marketplace, developer portal/toolchain, incident response, and release evidence without creating a side channel around SIAR's security, privacy, anonymity, local-first, or tenant-isolation guarantees.
```

---

# 286. Initial Production Scope

Implement first:

```text
typed observability signal classes
runtime health snapshots
structured metrics
cardinality budgets
structured logs
bounded local log ring buffer
WASM trap reporting
sanitized native minidump path
crash fingerprints
local crash aggregation
diagnostic bundle generation
redaction engine
telemetry schema registry
aggregate-only developer telemetry
sampling policy
offline bounded telemetry queue
SLI/SLO model
crash-loop quarantine integration
update rollout health hooks
certification evidence invalidation hooks
privacy-safe developer portal dashboard
observability testkit
```

Then add:

```text
differentially private ecosystem metrics
advanced local trace visualization
cross-device aggregate health summaries
automatic regression baselining
formal telemetry policy proofs
privacy-preserving tenant operations dashboards
```

---

# 287. Definition of Done

Part 145 is complete when:

- runtime health is typed and scoped;
- metric/log schemas are bounded;
- high-cardinality user/content labels are impossible;
- logs are redacted/retained briefly;
- crashes produce sanitized extension-scoped reports;
- support bundles are explicit and privacy-safe;
- developer telemetry is aggregate-only;
- anonymous mode strips route/peer/device metadata;
- SLOs use explicit denominators and Unknown semantics;
- hard security/privacy invariants are never error-budgeted;
- telemetry queues/sampling/backpressure are bounded;
- update/certification/incident systems can consume technical regressions;
- no observability surface becomes user behavior surveillance;
- redaction/cardinality/crash/privacy/SLO/fuzz/formal tests are specified.

---

# 288. Final Architecture

```text
                 EXTENSION RUNTIME
                        │
                        ▼
                 SIGNAL PRODUCERS
                        │
           ┌────────────┼────────────┐
           │            │            │
        METRICS       LOGS         CRASH
           │            │            │
           └────────────┼────────────┘
                        ▼
                 POLICY / REDACTION
                        │
                        ▼
                LOCAL AGGREGATION
                        │
             ┌──────────┼──────────┐
             │                     │
          USER/ADMIN             DEVELOPER
             │                     │
             └──────────┬──────────┘
                        ▼
                SLO / INCIDENT / SUPPORT
```

Extension-operations safety model:

```text
extension-scoped signals
+
bounded schemas
+
structured logs
+
sanitized crash reporting
+
privacy-aware aggregation
+
local-first diagnostics
+
aggregate developer telemetry
+
explicit SLOs
+
incident/update/certification hooks
```

not:

```text
record every click, message, path, and network event, ship it to the publisher, and call the resulting dataset observability
```

---

# 289. Final Principle

Extension observability is trustworthy when SIAR can answer **what is failing, how often, under which version/platform, and whether the extension is meeting technical objectives** without needing to know **who the user is, what they said, who they contacted, or how they behaved**.

The correct model is:

```text
measure technical health
+
bound metrics and logs
+
sanitize crash data
+
aggregate before export
+
use privacy-safe diagnostics
+
define explicit SLIs/SLOs
+
trigger incidents from technical evidence
+
never turn operations data into surveillance
```

This architecture gives SIAR a privacy-preserving extension operations foundation for runtime health, logs, metrics, crash reporting, diagnostics, developer telemetry, SLOs, incident triggers, rollout regression detection, and support workflows while preserving the anonymity, local-first, least-authority, runtime, permission, data-governance, update, certification, and anti-surveillance guarantees established across Parts 34–144.
