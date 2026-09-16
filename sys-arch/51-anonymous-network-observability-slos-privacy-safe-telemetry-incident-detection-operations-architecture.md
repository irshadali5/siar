# Core System Architecture Part 51 — Anonymous Network Observability, SLOs, Privacy-Safe Telemetry, Incident Detection & Operations Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 51  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 34–50  

**Primary purpose:** define the complete observability and operations architecture for SIAR's anonymous infrastructure, including privacy-safe metrics, logs, traces, SLOs, error budgets, incident detection, alerting, operator dashboards, forensic boundaries, retention, redaction, anomaly detection, differential privacy where useful, and strict anti-surveillance controls.

---

# 1. Purpose

A production anonymous network must be operable.

Operators need to answer questions such as:

```text
Is the mixnet healthy?
Are mailboxes delivering?
Are relays saturated?
Is a region failing?
Are clients seeing high error rates?
Did a deployment regress latency?
Is a provider misbehaving?
```

But traditional observability can destroy anonymity if it records:

```text
per-user request IDs
route IDs
mailbox IDs
contact relationships
precise timestamps
peer IPs
cross-service trace IDs
```

The governing principle is:

> **SIAR must observe the health of systems without observing the social graph or reconstructing individual communication behavior.**

---

# 2. Architectural Position

```text
Services / Clients
       │
       ▼
Privacy Classification
       │
       ▼
Safe Metrics / Logs / Events
       │
       ▼
Aggregation / Redaction
       │
       ▼
SLO / Incident Detection
       │
       ▼
Operations / Alerts / Response
```

---

# 3. Core Separation

Keep distinct:

```text
operational observability
security audit
privacy telemetry
user diagnostics
billing/accounting
forensics
```

---

# 4. Non-Goals

Part 51 does not create:

```text
packet-level distributed tracing
global user session tracking
contact graph analytics
behavioral user profiling
conversation-level dashboards
```

---

# 5. Observability Principles

Mandatory:

```text
aggregate first
local first
minimal labels
short retention
bounded cardinality
no user identifiers
no cross-service correlation IDs
```

---

# 6. Observability Types

```rust
pub enum ObservabilitySignal {
    Metric,
    Log,
    Event,
    AuditRecord,
    LocalDiagnostic,
}
```

---

# 7. Metrics

Best for:

```text
rates
latency distributions
capacity
error counts
```

---

# 8. Logs

Only for:

```text
state changes
operator actions
faults
```

with strict field allowlists.

---

# 9. Events

Structured operational events.

---

# 10. Audit Records

For:

```text
governance
configuration
security-sensitive operator actions
```

---

# 11. Local Diagnostics

Client-side detail retained locally.

---

# 12. Telemetry Privacy Classes

```rust
pub enum TelemetryPrivacyClass {
    SafeAggregate,
    SensitiveAggregate,
    LocalOnly,
    Forbidden,
}
```

---

# 13. SafeAggregate

Can be collected remotely.

Examples:

```text
node CPU bucket
aggregate error rate
provider availability
```

---

# 14. SensitiveAggregate

Needs:

```text
minimum cohort
coarse buckets
short retention
privacy review
```

---

# 15. LocalOnly

Examples:

```text
provider selection history
recent route failures
contact-specific delivery state
```

---

# 16. Forbidden

Examples:

```text
AccountId
DeviceId
ConversationId
ContactId
MailboxId
FrameId
exact route
reply capability
```

---

# 17. Telemetry Schema Registry

Every remote field must be registered.

---

# 18. Telemetry Field Record

```rust
pub struct TelemetryFieldDefinition {
    pub name: &'static str,
    pub privacy_class: TelemetryPrivacyClass,
    pub retention: RetentionClass,
    pub aggregation: AggregationRule,
}
```

---

# 19. Unknown Field

Rejected by default.

---

# 20. Allowlist Model

Hard rule:

```text
only explicitly approved fields may leave process
```

---

# 21. Metrics Architecture

Recommended:

```text
local counters
→ aggregation
→ coarse labels
→ remote metrics
```

---

# 22. No High-Cardinality Labels

Avoid:

```text
provider instance ID if unnecessary
route ID
mailbox ID
client ID
```

---

# 23. Allowed Labels

Examples:

```text
service_type
region_class
error_class
protocol_version
health_class
```

---

# 24. Cardinality Budget

```rust
pub struct MetricCardinalityBudget {
    pub max_label_sets: usize,
}
```

---

# 25. Cardinality Overflow

Drop/coarsen.

---

# 26. Timestamp Precision

Avoid nanosecond/millisecond precision remotely unless essential.

---

# 27. Time Buckets

Possible:

```text
1 minute
5 minutes
1 hour
```

depending metric.

---

# 28. Client Telemetry

Default:

```text
local-only
```

for strict anonymity modes.

---

# 29. Opt-In Remote Telemetry

If supported:

```text
aggregate
redacted
coarsened
```

---

# 30. Maximum Anonymity

Remote telemetry:

```text
off by default
```

---

# 31. Node Telemetry

Infrastructure nodes can report:

```text
CPU
memory
queue depth
aggregate throughput
aggregate errors
```

---

# 32. No Packet Trace IDs

Hard rule.

---

# 33. Mix Node Metrics

Safe examples:

```text
cells_received_total
cells_forwarded_total
delay_queue_depth_bucket
replay_reject_total
```

---

# 34. Forbidden Mix Node Metrics

No:

```text
incoming peer + outgoing peer pair
per-packet timestamp
route correlation key
```

---

# 35. Mailbox Metrics

Safe:

```text
objects_deposited_total
objects_fetched_total
storage_bytes_bucket
expired_objects_total
```

---

# 36. Forbidden Mailbox Metrics

No:

```text
mailbox ID labels
sender-recipient timing pair
capability token
```

---

# 37. Relay Metrics

Safe:

```text
active_sessions_bucket
packet_loss_bucket
jitter_bucket
bitrate_bucket
allocation_failure_total
```

---

# 38. Forbidden Relay Metrics

No:

```text
peer pair
CallId
raw IP pair
```

---

# 39. Bulk Provider Metrics

Safe:

```text
objects_active
bytes_stored_bucket
upload_failure_total
download_failure_total
```

---

# 40. Bridge Metrics

Safe:

```text
connections_bucket
handshake_failures
transport_class
```

---

# 41. Directory Metrics

Safe:

```text
catalog_age
topology_publish_success
revocation_feed_age
quorum_health
```

---

# 42. Provider Marketplace Metrics

Safe:

```text
provider_count
capacity_class_distribution
health_distribution
```

---

# 43. Resource Scheduler Metrics

Safe:

```text
allocation_success_rate
reservation_reject_rate
congestion_class
```

---

# 44. Accounting Metrics

Safe:

```text
credits_redeemed_total
refund_total
settlement_batch_success
```

without user linkage.

---

# 45. Logging Architecture

Use structured logs with typed fields.

---

# 46. Log Level Policy

```rust
pub enum SafeLogLevel {
    Error,
    Warn,
    Info,
    DebugLocal,
}
```

---

# 47. Production Remote Logs

Prefer:

```text
Error
Warn
selected Info
```

---

# 48. DebugLocal

Never exported automatically.

---

# 49. Structured Log Allowlist

```rust
pub trait SafeLogEvent {
    fn privacy_class(&self) -> TelemetryPrivacyClass;
    fn to_redacted_fields(&self) -> RedactedLogFields;
}
```

---

# 50. Sensitive Type Formatting

Sensitive types must not implement raw:

```text
Display
Debug
```

---

# 51. Redacted Wrapper

```rust
pub struct Redacted<T>(pub T);
```

---

# 52. Secret Types

Never serializable to telemetry.

---

# 53. Compile-Time Guard

Where possible:

```text
telemetry DTOs cannot contain secret/domain identity types
```

---

# 54. Log Sampling

High-frequency errors may be sampled.

---

# 55. Sampling Privacy

Sampling key must not be user identity.

---

# 56. Event Deduplication

Dedup repeated system errors.

---

# 57. Crash Reporting

High privacy risk.

---

# 58. Crash Dump Policy

Do not upload raw memory dump by default.

---

# 59. Crash Report

Use:

```text
stack trace
build ID
error class
platform class
```

---

# 60. No Secrets in Crash Report

Hard rule.

---

# 61. Memory Scrubbing

Sensitive buffers should be zeroized where practical.

---

# 62. Core Dump

Disable or tightly restrict in production privacy-sensitive nodes.

---

# 63. Client Crash Reporting

Maximum Anonymity:

```text
local-only by default
```

---

# 64. Support Bundle

User-generated.

---

# 65. Support Bundle Contents

Safe:

```text
build/version
feature flags
coarse network state
error classes
catalog versions
```

---

# 66. Support Bundle Exclusions

No:

```text
contact names
message content
mailbox IDs
provider tokens
route history
payment secrets
```

---

# 67. SLO Architecture

SLOs must be service-specific.

---

# 68. SLI

Service Level Indicator.

Examples:

```text
delivery success
latency
availability
freshness
durability
```

---

# 69. SLO

Target for SLI.

---

# 70. Error Budget

Allowed failure amount.

---

# 71. SLO Type

```rust
pub struct ServiceLevelObjective {
    pub service: ObservedService,
    pub indicator: ServiceLevelIndicator,
    pub target: SloTarget,
    pub window: SloWindow,
}
```

---

# 72. Observed Services

```rust
pub enum ObservedService {
    Mixnet,
    Directory,
    Mailbox,
    Bulk,
    RealtimeRelay,
    Bridge,
    ProviderCatalog,
    Accounting,
}
```

---

# 73. Mixnet SLO Examples

```text
cell forwarding success
queue latency
packet reject rate
```

---

# 74. Mailbox SLO Examples

```text
deposit availability
fetch availability
durability
```

---

# 75. Directory SLO Examples

```text
topology freshness
revocation freshness
catalog availability
```

---

# 76. Realtime SLO Examples

```text
relay allocation success
media packet loss
jitter
```

---

# 77. Bulk SLO Examples

```text
upload success
download success
resume success
```

---

# 78. Privacy SLOs

Must exist separately.

---

# 79. Privacy SLO Examples

```text
zero direct fallback in strict mode
zero forbidden telemetry fields
zero capability secret leakage
zero route trace IDs
```

---

# 80. Privacy Error Budget

For critical privacy invariants:

```text
zero
```

---

# 81. Availability Error Budget

Can be nonzero.

---

# 82. Do Not Trade Privacy Error Budget for Availability

Hard rule.

---

# 83. SLO Windows

Possible:

```text
1h
24h
7d
30d
```

---

# 84. Multi-Window Burn Rate

Useful for alerting.

---

# 85. Fast Burn Alert

Large error-budget consumption quickly.

---

# 86. Slow Burn Alert

Persistent degradation.

---

# 87. Alert Types

```rust
pub enum AlertSeverity {
    Info,
    Warning,
    High,
    Critical,
}
```

---

# 88. Critical Alerts

Examples:

```text
directory quorum lost
revocation stale
privacy invariant violation
mailbox durability failure
key compromise
```

---

# 89. High Alerts

Examples:

```text
regional capacity loss
relay allocation failure spike
```

---

# 90. Warning

Examples:

```text
increased latency
capacity trend
```

---

# 91. Alert Routing

Operators receive alerts.

---

# 92. Alert Payload Privacy

No user-specific identifiers.

---

# 93. Incident Detection

Sources:

```text
SLO burn
health degradation
security audit
anomaly detection
operator report
client aggregate failure
```

---

# 94. Incident Class

```rust
pub enum IncidentClass {
    Availability,
    Performance,
    Security,
    Privacy,
    Capacity,
    DataIntegrity,
    Governance,
}
```

---

# 95. Privacy Incident

Highest sensitivity.

---

# 96. Examples

```text
forbidden ID emitted
direct fallback
capability logged
route correlation enabled
```

---

# 97. Privacy Incident Response

Immediate:

```text
contain
disable affected feature
rotate secrets if needed
preserve safe evidence
```

---

# 98. Incident ID

Operational only.

---

# 99. No User Correlation Through Incident ID

Hard rule.

---

# 100. Incident State

```rust
pub enum IncidentState {
    Detected,
    Triaged,
    Contained,
    Mitigated,
    Monitoring,
    Resolved,
    Postmortem,
}
```

---

# 101. Incident Evidence

Infrastructure-only.

---

# 102. Packet Captures

Forbidden in production ordinary operation.

---

# 103. Exception

Controlled testnet/staging only.

---

# 104. Forensic Boundary

Production forensics may inspect:

```text
node config
binary hash
operator actions
aggregate metrics
security audit
```

---

# 105. Production Forensics Must Not Reconstruct User Traffic

Hard rule.

---

# 106. Security Audit Log

Contains:

```text
admin login
config change
key rotation
policy change
deployment
revocation
```

---

# 107. Security Audit Record

```rust
pub struct SecurityAuditRecord {
    pub actor: OperatorActorId,
    pub action: AuditAction,
    pub object: AuditObjectClass,
    pub timestamp: Timestamp,
    pub result: AuditResult,
}
```

---

# 108. Audit Actor

Operator/admin identity is acceptable.

---

# 109. Audit Object

Infrastructure object only.

---

# 110. No User Objects

Hard rule.

---

# 111. Audit Immutability

Append-only.

---

# 112. Audit Integrity

Signed/hash chained.

---

# 113. Audit Retention

Longer than ordinary logs.

---

# 114. Privacy

Still excludes user communications.

---

# 115. Operator Dashboard

Should answer:

```text
what service is unhealthy?
which region?
which provider class?
which version?
```

---

# 116. Dashboard Must Not Show

```text
who is talking
who is online
which contact pair
```

---

# 117. Dashboard Views

Recommended:

```text
Network Overview
Directory
Mixnet
Mailbox
Realtime
Bulk
Bridges
Capacity
Incidents
Deployments
```

---

# 118. Network Overview

Shows:

```text
health class
SLO status
capacity class
incident count
```

---

# 119. Region View

Coarse region health.

---

# 120. Provider View

Provider operational status.

---

# 121. No User Drill-Down

Hard rule.

---

# 122. Node View

Can show:

```text
CPU
memory
queue
version
uptime
```

---

# 123. Node IP

Operator-private infrastructure data.

Not user privacy data.

---

# 124. Deployment Correlation

Track build/version across nodes.

---

# 125. Release Health

Compare:

```text
error rate
latency
resource use
```

before/after deployment.

---

# 126. Canary Release

Small provider/node subset.

---

# 127. Canary Privacy

Do not route only identifiable user segment to canary.

---

# 128. Better

Random infrastructure subset.

---

# 129. Feature Flags

Infrastructure-level.

---

# 130. User-Targeted Feature Flags

High privacy risk.

---

# 131. Strict Mode

Avoid stable user-targeting key.

---

# 132. Cohort Flagging

If needed:

```text
random ephemeral cohort
```

not user identity.

---

# 133. Client Diagnostics

Local health view can show:

```text
anonymous path available
mailbox sync state
relay path state
catalog freshness
```

---

# 134. Client Diagnostics Detail

Can be richer locally.

---

# 135. Exporting Diagnostics

Explicit user action.

---

# 136. Automatic Upload

Disabled in Maximum Anonymity.

---

# 137. Event Correlation

Cross-service event correlation is dangerous.

---

# 138. No Global Correlation ID

Hard rule.

---

# 139. Per-Service Ephemeral Operation ID

Allowed locally.

---

# 140. Lifetime

Minutes/hours.

---

# 141. No Cross-Service Reuse

Hard rule.

---

# 142. Distributed Tracing

Traditional distributed tracing is incompatible with strong anonymity.

---

# 143. Alternative

Use:

```text
synthetic probes
aggregate phase metrics
service-local ephemeral traces
```

---

# 144. Synthetic Probes

Critical observability tool.

---

# 145. Probe Types

```text
mixnet loop probe
mailbox deposit/fetch probe
relay allocation probe
bulk upload/download probe
directory fetch probe
```

---

# 146. Synthetic Identity

Dedicated test identity.

---

# 147. No Real User Traffic

Hard rule.

---

# 148. Probe Scheduling

Randomized.

---

# 149. Probe Label

Explicit synthetic marker internally.

---

# 150. Avoid Network Fingerprinting

Probe traffic should not dominate/stand out excessively.

---

# 151. Loop Traffic

Part 37 can provide health signal.

---

# 152. Loop Result

Aggregate:

```text
success
latency bucket
```

---

# 153. No Full Route Trace

Hard rule.

---

# 154. Anomaly Detection

Useful for:

```text
DDoS
capacity collapse
malicious node behavior
software regression
```

---

# 155. Anomaly Inputs

Only aggregate telemetry.

---

# 156. No User Behavioral Model

Hard rule.

---

# 157. Anomaly Detector

```rust
pub trait OperationalAnomalyDetector {
    fn evaluate(
        &self,
        window: AggregateTelemetryWindow,
    ) -> Vec<OperationalAnomaly>;
}
```

---

# 158. Anomaly Types

```rust
pub enum OperationalAnomaly {
    ErrorSpike,
    LatencyShift,
    CapacityDrop,
    VersionRegression,
    RegionFailure,
    ProviderOutlier,
}
```

---

# 159. Detection Methods

Start simple:

```text
thresholds
rolling baselines
EWMA
percentile changes
```

---

# 160. ML Detection

Future.

---

# 161. ML Privacy Constraint

Input must remain aggregate.

---

# 162. No Raw User Traces for Training

Hard rule.

---

# 163. Differential Privacy

Useful for public or cross-operator aggregate statistics.

---

# 164. DP Use Cases

```text
network usage distribution
provider capacity statistics
public transparency metrics
```

---

# 165. DP Is Not Needed Everywhere

Internal aggregate operational metrics may already be safe enough.

---

# 166. DP Budget

If used, centrally governed.

---

# 167. Cohort Minimum

No aggregate published below threshold.

---

# 168. Small Provider Protection

Exact provider metrics may identify operator workload.

---

# 169. Public Metrics

Use coarse classes.

---

# 170. Internal Operator Metrics

Can be more detailed for own infrastructure.

---

# 171. Cross-Operator Data Sharing

Minimize.

---

# 172. Federated Operations

Operators may share:

```text
signed health class
capacity class
incident status
```

---

# 173. Do Not Share Raw Logs

Hard rule.

---

# 174. Observability Transport

Metrics transport itself should be authenticated/encrypted.

---

# 175. Telemetry Endpoint

Separate from user traffic endpoint.

---

# 176. Authentication

Node identity.

---

# 177. No User Token

Hard rule.

---

# 178. Metric Batching

Send in intervals.

---

# 179. Why

Reduces fine-grained timing leakage.

---

# 180. Batch Jitter

Required.

---

# 181. Retry

Telemetry retry is low priority.

---

# 182. Telemetry Loss

Acceptable.

---

# 183. Do Not Block Service on Telemetry

Hard rule.

---

# 184. Backpressure

Drop low-priority telemetry first.

---

# 185. Local Buffer

Bounded.

---

# 186. No Persistent Massive Telemetry Queue

Hard rule.

---

# 187. Retention Classes

```rust
pub enum RetentionClass {
    Ephemeral,
    Short,
    Operational,
    Audit,
}
```

---

# 188. Ephemeral

Minutes/hours.

---

# 189. Short

Days.

---

# 190. Operational

Weeks.

---

# 191. Audit

Longer, policy-defined.

---

# 192. Sensitive Aggregate Retention

Prefer shortest.

---

# 193. Retention Enforcement

Automatic deletion.

---

# 194. Deletion Verification

Operational job + audit.

---

# 195. Data Lake

Avoid central raw telemetry lake.

---

# 196. Better

Purpose-specific stores.

---

# 197. Metrics Store

Aggregates only.

---

# 198. Log Store

Redacted structured logs.

---

# 199. Audit Store

Operator actions.

---

# 200. Incident Store

Incident summaries/evidence.

---

# 201. Access Control

Separate permissions.

---

# 202. Least Privilege

SRE access ≠ audit admin access.

---

# 203. Break-Glass Access

For critical incidents.

---

# 204. Break-Glass Audit

Mandatory.

---

# 205. No Break-Glass to User Traffic Data

Because it should not exist.

---

# 206. Privacy Review for New Metric

Workflow:

```text
why needed?
what field?
what label?
retention?
could it correlate users?
```

---

# 207. Metric Approval Record

```rust
pub struct ObservabilityApproval {
    pub signal_name: String,
    pub owner: TeamId,
    pub privacy_class: TelemetryPrivacyClass,
    pub approved_retention: RetentionClass,
}
```

---

# 208. CI Schema Gate

Fail build if unregistered remote telemetry field appears.

---

# 209. Static Telemetry Lint

Recommended.

---

# 210. Sensitive Type Denylist

Compile-time/lint check.

---

# 211. Example Forbidden Types

```text
AccountId
DeviceId
ContactId
ConversationId
MailboxId
ReplyCapability
ServiceSpendToken
```

---

# 212. Release Gate

No new remote telemetry without approval.

---

# 213. SLO-as-Code

Store SLO definitions in repository.

---

# 214. Example Layout

```text
ops/
├── slos/
├── alerts/
├── dashboards/
├── runbooks/
└── telemetry-schema/
```

---

# 215. Alert-as-Code

Version controlled.

---

# 216. Dashboard-as-Code

Version controlled.

---

# 217. Runbook Link

Every critical alert maps to runbook.

---

# 218. Incident Runbook

Contains:

```text
symptoms
privacy constraints
diagnostics
containment
recovery
validation
```

---

# 219. Runbook Privacy Section

Mandatory.

---

# 220. Example

Relay outage runbook:

```text
do not enable direct fallback
```

---

# 221. Incident Detection for Privacy

Need explicit detectors.

---

# 222. Privacy Canary

Synthetic assertion:

```text
strict call never exposes host candidate
```

---

# 223. Privacy Canary Examples

```text
no direct route in strict messaging
no raw mailbox ID in logs
no capability token in support bundle
```

---

# 224. Continuous Privacy Monitoring

Run synthetic privacy tests in staging/nightly.

---

# 225. Production Runtime Privacy Guard

Some invariants can assert at runtime.

---

# 226. Example

```rust
debug_assert!(route.is_anonymous());
```

Production:

```text
hard error, not debug-only
```

for security-critical path.

---

# 227. Runtime Guard

```rust
pub fn enforce_privacy_route(
    route: &ResolvedRoute,
    required: PrivacyRoutingMode,
) -> Result<(), PrivacyInvariantViolation>;
```

---

# 228. Privacy Violation Event

Local + critical operator alert.

---

# 229. No Sensitive Context in Alert

Hard rule.

---

# 230. Incident Correlation Window

Aggregate by service/region/version.

---

# 231. Deployment Regression Detection

Compare:

```text
before
after
```

---

# 232. Statistical Guard

Use confidence intervals.

---

# 233. No User-Cohort Regression Analysis

Strict mode.

---

# 234. Version Breakdown

Allowed.

---

# 235. OS/Platform Breakdown

Coarse.

---

# 236. Device Model

Avoid exact model remotely.

---

# 237. Platform Classes

```text
Android
Linux
Windows
macOS
```

---

# 238. Android Version

Coarse major version if needed.

---

# 239. Network Type

Coarse:

```text
Wi-Fi
Cellular
Unknown
```

if opt-in and privacy-reviewed.

---

# 240. Location

No precise location.

---

# 241. Region

Infrastructure region, not user location.

---

# 242. User-Reported Incident

Optional explicit diagnostic upload.

---

# 243. Consent

User sees what is sent.

---

# 244. Redaction Preview

Recommended.

---

# 245. Support Upload Capability

One-time scoped token.

---

# 246. No Persistent Support Account Linkage

Strict mode.

---

# 247. Security Operations

SOC/SRE need visibility into attacks.

---

# 248. DDoS Metrics

Aggregate:

```text
connections
invalid handshakes
rate-limit drops
```

---

# 249. Abuse Metrics

Aggregate only.

---

# 250. No IP Long-Term Analytics

Hard rule where operationally avoidable.

---

# 251. Ephemeral IP Rate Limiting

In-memory.

---

# 252. Short Retention

Only if required for active defense.

---

# 253. IDS/IPS

Can operate at infrastructure layer.

---

# 254. Payload Inspection

Not possible/allowed for E2EE user content.

---

# 255. Malware Detection

Infrastructure-level binary/config, not user messages.

---

# 256. File Abuse

Handled via capabilities/quota/moderation, not plaintext inspection.

---

# 257. Threat Detection

Examples:

```text
replay floods
invalid Sphinx packets
bridge probe floods
credential abuse
```

---

# 258. Security Event

```rust
pub enum SecurityOperationalEvent {
    ReplayFlood,
    InvalidPacketSpike,
    CredentialAbuse,
    ProbeFlood,
    KeyMismatch,
    UnauthorizedConfigChange,
}
```

---

# 259. Event Aggregation

By node/service/time bucket.

---

# 260. No User Pairing

Hard rule.

---

# 261. Operations Roles

Suggested:

```text
SRE
Security
Privacy
Governance
Support
```

---

# 262. Role Separation

Privacy team can review telemetry schema.

---

# 263. Support

Cannot access infrastructure audit keys.

---

# 264. Governance

Cannot access user content.

---

# 265. Audit of Observability Access

Queries/access logged.

---

# 266. Sensitive Dashboard Access

Role-gated.

---

# 267. Public Status Page

Shows:

```text
service health
incident summaries
```

---

# 268. No User Impact Counts If Too Precise

Use coarse.

---

# 269. Incident Communication

Can state:

```text
Mailbox service degraded in one region
```

---

# 270. Do Not Reveal Sensitive Topology

Hard rule.

---

# 271. Postmortem

Should include:

```text
root cause
timeline
impact
fix
prevention
privacy impact
```

---

# 272. Postmortem Privacy

No user traces.

---

# 273. Privacy Impact Section

Mandatory.

---

# 274. Was anonymity weakened?

Explicit answer.

---

# 275. Was any forbidden telemetry emitted?

Explicit answer.

---

# 276. Was any secret exposed?

Explicit answer.

---

# 277. Continuous Improvement

Postmortem action items become:

```text
alerts
tests
runbooks
invariants
```

---

# 278. Observability Testkit

Need fake metrics/log sinks.

---

# 279. Testkit Features

```text
schema validation
secret canaries
forbidden field detection
retention simulation
alert testing
SLO burn simulation
```

---

# 280. Secret Canary

Inject known secret.

Verify absent from:

```text
logs
metrics
support bundle
crash report
```

---

# 281. Logging Tests

Assert sensitive types cannot serialize into log DTO.

---

# 282. Cardinality Tests

Ensure bounded labels.

---

# 283. SLO Tests

Synthetic failures trigger expected burn alerts.

---

# 284. Alert Tests

No false escalation from one transient sample.

---

# 285. Privacy Alert Tests

Any forbidden field -> critical.

---

# 286. Retention Tests

Old data deleted.

---

# 287. RBAC Tests

Support cannot access audit store.

---

# 288. Incident Drill

Run:

```text
regional outage
privacy violation
key compromise
capacity collapse
```

---

# 289. Dashboard Tests

Missing data handled.

---

# 290. Fuzzing

Fuzz:

```text
telemetry parser
log encoder
support bundle builder
alert rules
```

---

# 291. Property Tests

Properties:

```text
Forbidden telemetry never leaves process
Sensitive types always redact
Strict-mode client telemetry remains local unless opt-in
No metric label contains communication identity types
```

---

# 292. Formal Verification Targets

Good candidates:

```text
telemetry privacy-class gate
retention state machine
incident escalation
privacy invariant alert path
```

---

# 293. Kani Candidate

Telemetry DTO type constraints.

---

# 294. TLA+ Candidate

Incident detection/containment workflow.

---

# 295. Loom Candidate

Concurrent bounded telemetry buffer.

---

# 296. Performance Tests

Measure:

```text
metrics overhead
log overhead
aggregation cost
dashboard query cost
```

---

# 297. Overhead Budget

Observability must not materially degrade anonymity transport.

---

# 298. Client Idle CPU

Telemetry should not create periodic wakeups unnecessarily.

---

# 299. Android Battery

Batch telemetry if enabled.

---

# 300. Desktop Daemon

Can aggregate locally before export.

---

# 301. Storage Schema

Potential:

```text
metric_aggregates
redacted_logs
security_audit
incidents
slo_state
telemetry_schema_registry
```

---

# 302. No Unified Raw Event Table

Hard rule.

---

# 303. Metrics Retention

Short/operational.

---

# 304. Audit Retention

Longer.

---

# 305. Incident Evidence Retention

Case-specific.

---

# 306. Public Transparency Metrics

Possible:

```text
uptime
incident counts
provider count
```

---

# 307. Differential Privacy

Use where public aggregates risk operator/user inference.

---

# 308. No Fake Privacy Label

If DP is not actually implemented, do not claim it.

---

# 309. Operator Federation

Each operator can maintain own detailed operational view.

---

# 310. Cross-Network Aggregator

Receives only coarse signed health summaries.

---

# 311. Health Summary

```rust
pub struct SignedHealthSummary {
    pub service: AnonymousServiceType,
    pub health: ProviderHealthClass,
    pub capacity: CapacityClass,
    pub window: TimeBucket,
    pub signature: OperatorSignature,
}
```

---

# 312. No Traffic Volume Required

Can be optional/coarse.

---

# 313. Health Integrity

Signed.

---

# 314. False Health Report

Independent observers may detect.

---

# 315. Observability Governance

A dedicated privacy review should own:

```text
telemetry schema
retention
public metrics
support bundle policy
```

---

# 316. Change Control

Every new remote telemetry field requires review.

---

# 317. Emergency Telemetry Expansion

Temporary only.

---

# 318. Emergency Expansion Rule

Must:

```text
have expiry
have approval
avoid forbidden fields
```

---

# 319. Auto-Expiry

Required.

---

# 320. No "Temporary" Permanent Logging

Hard rule.

---

# 321. SLO Hierarchy

Network-wide SLOs and service SLOs.

---

# 322. Network SLO

Examples:

```text
anonymous delivery availability
control-plane freshness
privacy invariant compliance
```

---

# 323. Service SLO

Mailbox, relay, etc.

---

# 324. Provider SLO

Per operator/provider.

---

# 325. Client Experience SLI

Can be measured locally.

---

# 326. Remote Experience Sampling

Optional privacy-preserving opt-in.

---

# 327. Synthetic Experience Preferred

Hard rule for default.

---

# 328. Error Budget Policy

If availability budget burned:

```text
freeze risky releases
```

---

# 329. Privacy Violation

Immediately freeze/revert regardless of availability budget.

---

# 330. Deployment Gates

Release requires:

```text
SLO healthy
no active critical privacy incident
telemetry schema clean
privacy canaries passing
```

---

# 331. On-Call Runbook

Must include:

```text
what to inspect
what not to inspect
```

---

# 332. Example Prohibition

Do not request raw user message dumps.

---

# 333. Operations Training

Operators must understand privacy boundaries.

---

# 334. Access Review

Periodic.

---

# 335. Audit Review

Periodic.

---

# 336. Privacy SLO Review

At major release.

---

# 337. Threat Model Integration

Part 42 consumes observability architecture as potential metadata source.

---

# 338. Reliability Integration

Part 50 feeds region/provider health and DR state.

---

# 339. Capacity Integration

Part 49 feeds aggregate congestion/capacity.

---

# 340. Provider Integration

Part 48 feeds provider health/audit state.

---

# 341. Accounting Integration

Part 47 provides separate accounting telemetry.

---

# 342. Moderation Integration

Part 46 moderation evidence remains outside operations telemetry.

---

# 343. Social Graph Integration

Part 45 presence/contact data remains local/private.

---

# 344. Realtime Integration

Part 44 relay metrics coarse only.

---

# 345. Group Integration

Part 43 group traffic does not create per-group metrics.

---

# 346. Attachment Integration

Part 40 file transfer metrics bucketed.

---

# 347. Censorship Integration

Part 41 blocking metrics coarse/regional, not exact user location.

---

# 348. Observability Service Trait

```rust
pub trait PrivacySafeObservability {
    fn metric(
        &self,
        metric: SafeMetric,
    ) -> Result<(), ObservabilityError>;

    fn event(
        &self,
        event: SafeOperationalEvent,
    ) -> Result<(), ObservabilityError>;

    fn audit(
        &self,
        record: SecurityAuditRecord,
    ) -> Result<(), ObservabilityError>;
}
```

---

# 349. Telemetry Gate

```rust
pub trait TelemetryPrivacyGate {
    fn approve(
        &self,
        signal: &OutboundTelemetry,
    ) -> Result<ApprovedTelemetry, TelemetryViolation>;
}
```

---

# 350. SLO Engine

```rust
pub trait SloEngine {
    fn evaluate(
        &self,
        objective: &ServiceLevelObjective,
        window: &MetricWindow,
    ) -> SloStatus;
}
```

---

# 351. Incident Detector

```rust
pub trait IncidentDetector {
    fn detect(
        &self,
        signals: &AggregateSignalWindow,
    ) -> Vec<IncidentCandidate>;
}
```

---

# 352. Retention Engine

```rust
pub trait RetentionEngine {
    fn expire(
        &self,
        now: Timestamp,
    ) -> Result<RetentionReport, ObservabilityError>;
}
```

---

# 353. Error Taxonomy

```rust
pub enum ObservabilityError {
    ForbiddenField,
    CardinalityExceeded,
    SchemaUnregistered,
    RetentionViolation,
    ExportDisabled,
    AuditWriteFailed,
    MetricsUnavailable,
    Internal,
}
```

---

# 354. Security Invariants

Mandatory:

```text
1. No AccountId/DeviceId/ContactId/ConversationId/MailboxId appears in remote operational telemetry.
2. No cross-service distributed trace ID exists for user traffic.
3. Maximum Anonymity client telemetry is local-only by default.
4. Sensitive types cannot be emitted through generic logging APIs.
5. Remote telemetry fields are allowlisted and privacy-classified.
6. Privacy-critical invariant violations have zero error budget.
7. Packet captures and raw traffic traces are forbidden in ordinary production operations.
8. Support bundles exclude secrets, social graph data, and route identifiers.
9. Incident response does not introduce temporary privacy downgrades.
10. Telemetry retention is bounded and automatically enforced.
11. Public/cross-operator metrics are coarse and minimum-cohort protected.
12. Observability remains operationally useful without reconstructing individual communication behavior.
```

---

# 355. Recommended Crate Layout

```text
crates/
├── siar-observability-core/
├── siar-metrics/
├── siar-safe-logging/
├── siar-telemetry-schema/
├── siar-privacy-gate/
├── siar-slo/
├── siar-alerting/
├── siar-incident/
├── siar-audit/
├── siar-support-bundle/
├── siar-operations-testkit/
└── siar-observability-lints/
```

---

# 356. `siar-observability-core`

Owns:

```text
privacy classes
retention classes
service types
errors
```

---

# 357. `siar-metrics`

Counters/histograms/aggregation.

---

# 358. `siar-safe-logging`

Typed structured redacted logs.

---

# 359. `siar-telemetry-schema`

Field registry/approval.

---

# 360. `siar-privacy-gate`

Outbound telemetry enforcement.

---

# 361. `siar-slo`

SLI/SLO/error-budget evaluation.

---

# 362. `siar-alerting`

Burn-rate alerts and routing.

---

# 363. `siar-incident`

Incident state/runbook integration.

---

# 364. `siar-audit`

Operator security audit log.

---

# 365. `siar-support-bundle`

Redacted diagnostic export.

---

# 366. `siar-operations-testkit`

Secret canaries, SLO simulations, incident drills.

---

# 367. `siar-observability-lints`

Static checks for forbidden telemetry types.

---

# 368. Initial Production Scope

Implement first:

```text
typed telemetry privacy classes
safe aggregate metrics
structured redacted logs
remote telemetry allowlist
local-only strict client diagnostics
SLO-as-code
multi-window burn alerts
privacy SLOs
security audit log
support-bundle redaction
synthetic health probes
incident state machine
retention enforcement
secret-canary tests
```

Then add:

```text
differentially private public metrics
cross-operator signed health summaries
advanced anomaly detection
formal telemetry type proofs
privacy-preserving client experience sampling
```

---

# 369. Definition of Done

Part 51 is complete when:

- telemetry privacy classes are explicit
- remote telemetry is allowlisted
- forbidden identity/capability fields cannot be emitted
- metrics/logs/audit/support bundles have separate boundaries
- no distributed tracing exists across anonymous user traffic
- SLOs and privacy SLOs are defined
- privacy-critical violations have zero error budget
- burn-rate alerting and incident classes are defined
- operator dashboards answer health questions without user drill-down
- synthetic probes replace packet-level production tracing
- anomaly detection uses aggregate inputs
- retention is purpose-specific, bounded, and auto-enforced
- crash/support reports are redacted
- incident response preserves privacy constraints
- CI/privacy canaries detect telemetry leaks
- public/cross-operator metrics use coarse/minimum-cohort rules
- observability remains sufficient for real operations without becoming a surveillance system

---

# 370. Final Architecture

```text
                   SERVICES / NODES / CLIENTS
                              │
                              ▼
                   PRIVACY CLASSIFICATION
                              │
                 ┌────────────┼────────────┐
                 │            │            │
              Metrics       Logs        Audit
                 │            │            │
                 └────────────┼────────────┘
                              ▼
                    AGGREGATE / REDACT
                              │
                              ▼
                     SLO / ANOMALY ENGINE
                              │
                              ▼
                     ALERT / INCIDENT OPS
```

Strict observability model:

```text
aggregate system health
+
typed redacted logs
+
local diagnostics
+
synthetic probes
+
privacy SLOs
+
bounded retention
```

not:

```text
trace every request
→ correlate every service
→ debug by reconstructing user behavior
```

---

# 371. Final Principle

An anonymous network must be operable **without becoming observable at the level of individual people and relationships**.

The correct model is:

```text
aggregate metrics
+
privacy-classified telemetry
+
local-only sensitive diagnostics
+
synthetic probes
+
SLOs
+
privacy-safe incident response
```

This architecture gives SIAR the operational visibility needed for production reliability while preserving the anonymity, identity-separation, and metadata-minimization guarantees established across Parts 34–50.
