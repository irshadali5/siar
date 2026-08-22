# Part 18 — Network Diagnostics & Path Visualization Architecture

## Reusable P2P Communication Platform

**Status:** Architecture specification  
**Part:** 18 of 24  
**Primary language:** Rust  
**Primary UI:** Dioxus  
**Primary goals:** explainable networking, real-time path diagnostics, direct/relay/local/DTN visibility, transport transition history, multipath visualization, privacy-preserving metrics, failure diagnosis, operator observability, user-friendly health states, reusable diagnostics APIs

---

# 1. Purpose

A sophisticated P2P communication system can use:

```text
Iroh direct
Iroh relay
LAN
Bluetooth LE
Bluetooth Classic
Wi-Fi Direct
Wi-Fi Aware
multipath
DTN
gateway nodes
```

Without good diagnostics, a user or developer may only see:

```text
Connecting…
```

while the actual system could be:

```text
trying LAN
waiting for NAT traversal
falling back to relay
discovering BLE
switching to Wi-Fi Direct
resuming chunks
```

A production-ready platform therefore needs a first-class observability system.

The core rule is:

> **The network stack must be able to explain what path it is using, why it chose that path, what changed, and why an operation failed—without exposing unnecessary sensitive metadata.**

---

# 2. Architectural Position

```text
Transports / Routing / DTN / Multipath
             ↓
      Diagnostic Events
             ↓
     Network Observability Core
      ├── live state
      ├── path graph
      ├── measurements
      ├── history
      ├── failure reasoning
      └── privacy filtering
             ↓
    ┌────────┼─────────┐
    │        │         │
 Dioxus     CLI     Operator API
```

---

# 3. Diagnostics Is Not Logging

Separate:

```text
structured logs
metrics
traces
live diagnostics
historical path events
```

Each has a different purpose.

---

# 4. Diagnostics Responsibilities

Part 18 owns:

```text
network status model
path state model
path visualization model
routing explanation
transport health
connection timelines
multipath visualization
DTN route/hop status
network measurements
failure reason classification
diagnostic snapshots
safe export bundles
```

It does not own:

```text
routing decisions
transport implementation
message state
file state
identity authorization
```

---

# 5. Core Diagnostic Model

```rust
pub struct NetworkDiagnostics {
    pub connectivity: ConnectivityState,
    pub active_paths: Vec<PathDiagnostic>,
    pub candidates: Vec<PathCandidateDiagnostic>,
    pub recent_events: Vec<NetworkDiagnosticEvent>,
    pub health: NetworkHealthSummary,
}
```

---

# 6. Connectivity State

```rust
pub enum ConnectivityState {
    Offline,
    LocalOnly,
    InternetLimited,
    Connected,
    Degraded,
}
```

---

# 7. User Meaning

## Offline

```text
No usable communication path.
```

## LocalOnly

```text
LAN/Bluetooth/Wi-Fi/DTN available, Internet unavailable.
```

## InternetLimited

```text
Internet exists but remote P2P reachability is degraded.
```

## Connected

```text
Normal usable paths.
```

## Degraded

```text
Communication works but with meaningful failures/poor quality.
```

---

# 8. Path Types

```rust
pub enum DiagnosticPathKind {
    IrohDirect,
    IrohRelay,
    Lan,
    BluetoothLe,
    BluetoothClassic,
    WifiDirect,
    WifiAware,
    Cellular,
    Dtn,
    Gateway,
}
```

Some are underlay hints while others are logical communication paths.

---

# 9. Path Identity

```rust
pub struct PathId([u8; 16]);
```

Path IDs are local diagnostic identifiers.

Do not expose raw peer IP as identity.

---

# 10. Path Diagnostic

```rust
pub struct PathDiagnostic {
    pub id: PathId,
    pub kind: DiagnosticPathKind,
    pub state: PathState,
    pub role: PathRole,
    pub quality: PathQuality,
    pub started_at: Timestamp,
    pub last_changed_at: Timestamp,
}
```

---

# 11. Path State

```rust
pub enum PathState {
    Candidate,
    Probing,
    Connecting,
    Authenticating,
    Ready,
    Active,
    Standby,
    Degraded,
    Failed,
    Closed,
}
```

---

# 12. Path Role

```rust
pub enum PathRole {
    Primary,
    Secondary,
    Standby,
    Control,
    Bulk,
    Redundant,
    DtnCarry,
}
```

---

# 13. Path Quality

```rust
pub struct PathQuality {
    pub latency: Option<LatencyClass>,
    pub throughput: Option<ThroughputClass>,
    pub loss: Option<LossClass>,
    pub jitter: Option<JitterClass>,
    pub stability: StabilityClass,
}
```

User-facing UI should prefer coarse quality classes.

---

# 14. Latency Class

```rust
pub enum LatencyClass {
    Excellent,
    Good,
    Fair,
    Poor,
    VeryPoor,
    Unknown,
}
```

Raw milliseconds remain available in developer mode.

---

# 15. Throughput Class

```rust
pub enum ThroughputClass {
    Tiny,
    Low,
    Medium,
    High,
    VeryHigh,
    Unknown,
}
```

---

# 16. Loss Class

```rust
pub enum LossClass {
    NoneObserved,
    Low,
    Moderate,
    High,
    Unknown,
}
```

---

# 17. Stability Class

```rust
pub enum StabilityClass {
    Stable,
    Fluctuating,
    Unstable,
    Unknown,
}
```

---

# 18. Measurements

Raw local metrics may include:

```text
RTT
throughput
loss
jitter
connection age
reconnect count
path switch count
queue delay
```

---

# 19. Do Not Over-Probe

Prefer measurements from actual transport traffic.

Synthetic probes should be:

```text
small
rate-limited
purposeful
```

especially on mobile.

---

# 20. Diagnostic Event Stream

```rust
pub enum NetworkDiagnosticEvent {
    CandidateDiscovered(...),
    PathProbeStarted(...),
    PathConnected(...),
    PathAuthenticated(...),
    PathActivated(...),
    PathDegraded(...),
    PathFailed(...),
    PathSwitched(...),
    RelaySelected(...),
    GatewayReached(...),
    DtnForwarded(...),
}
```

---

# 21. Event Classification

Separate:

```text
operational event
user-visible event
security-sensitive event
```

Most low-level changes remain developer diagnostics only.

---

# 22. Routing Explanation

Part 03 should produce a structured explanation.

```rust
pub struct RoutingExplanation {
    pub selected: PathId,
    pub reason: RoutingReason,
    pub rejected: Vec<RejectedPathReason>,
}
```

---

# 23. Routing Reason

```rust
pub enum RoutingReason {
    LowestLatency,
    ExistingConnection,
    LocalPreferred,
    DirectPreferred,
    RelayFallback,
    MeteredAvoidance,
    BatteryPolicy,
    EmergencyPriority,
    FileThroughput,
    MultipathBenefit,
    DtnOnlyAvailable,
}
```

---

# 24. Rejected Path Reason

```rust
pub enum RejectedPathReason {
    Unreachable,
    PolicyDenied,
    MeteredDenied,
    BatteryDenied,
    CapabilityMissing,
    AuthenticationFailed,
    ResourceLimit,
    TooSlow,
    SharedFailureDomain,
}
```

---

# 25. Explainability Contract

A routing decision should be answerable as:

```text
Selected LAN because:
- peer is local
- LAN is authenticated
- lower latency
- no metered cost

Did not use relay because:
- direct local path is healthy
```

---

# 26. No Opaque Scoring Only

Do not expose only:

```text
score = 73.42
```

Provide semantic reason codes.

Scores may exist internally.

---

# 27. User-Level Network Status

Normal user should see:

```text
Direct connection
Relayed connection
Nearby connection
Offline — waiting for nearby device
```

Not raw QUIC internals.

---

# 28. Advanced Diagnostics Mode

Developer/advanced view can expose:

```text
transport
relay
RTT
loss
throughput
path transitions
routing reasons
```

---

# 29. Path Visualization

A useful path diagram:

```text
This Device
    │
   LAN
    │
Peer Device
```

Relay:

```text
This Device
    │
  Internet
    │
Iroh Relay
    │
  Internet
    │
Peer
```

---

# 30. DTN Visualization

Example:

```text
Alice
  ↓ BLE
Bob
  ↓ carried offline
Carol
  ↓ Wi-Fi
Gateway
  ↓ Iroh
Destination
```

This is especially useful for emergency diagnostics.

---

# 31. Path Graph Model

```rust
pub struct DiagnosticGraph {
    pub nodes: Vec<DiagnosticNode>,
    pub edges: Vec<DiagnosticEdge>,
}
```

---

# 32. Diagnostic Node

```rust
pub enum DiagnosticNodeKind {
    LocalDevice,
    PeerDevice,
    Relay,
    Gateway,
    DtnRelay,
    NetworkBoundary,
}
```

---

# 33. Diagnostic Edge

```rust
pub struct DiagnosticEdge {
    pub from: DiagnosticNodeId,
    pub to: DiagnosticNodeId,
    pub transport: DiagnosticPathKind,
    pub state: PathState,
    pub metrics: EdgeMetrics,
}
```

---

# 34. Privacy-Safe Node Labels

Normal UI labels:

```text
This device
Relay
Nearby relay
Destination
```

Advanced local diagnostics may show trusted peer device names.

---

# 35. Do Not Reveal Relay Internals Unnecessarily

Normal user generally does not need:

```text
relay hostname
IP
region code
```

Advanced diagnostics can show them.

---

# 36. Multipath Visualization

Example:

```text
                 ┌─ Wi-Fi ──────┐
This Device ─────┤               ├─ Peer
                 └─ Cellular ───┘
```

Show roles:

```text
Wi-Fi: primary
Cellular: standby
```

or:

```text
Wi-Fi: 75% file chunks
Cellular: 25%
```

---

# 37. Multipath State

```rust
pub struct MultipathDiagnostic {
    pub strategy: MultipathStrategy,
    pub plan_epoch: u64,
    pub paths: Vec<MultipathPathDiagnostic>,
    pub duplicate_bytes: u64,
}
```

---

# 38. File Path Visualization

For large transfer:

```text
File 72%
├── LAN      60%
└── Relay    12%
```

Actual visualization should distinguish:

```text
overall transfer completion
path contribution
```

---

# 39. File Diagnostics

Part 05 exposes:

```text
chunks completed
chunks in flight
retries
current paths
resume state
```

Diagnostics should not scan raw file data.

---

# 40. Message Diagnostics

Message status:

```text
Queued
Connecting
Sent locally
Relayed
Delivered
Read
```

Network diagnostics can explain current delivery path.

---

# 41. Emergency Diagnostics

Critical message should display:

```text
Stored locally
Sent over Internet
Carried by 2 nearby relay devices
Reached gateway
Delivered
```

This is more informative than a single spinner.

---

# 42. DTN Privacy

Do not display the real identity of arbitrary relay peers unless authorized.

Use:

```text
Nearby relay 1
Nearby relay 2
```

---

# 43. DTN Hop Count

Can expose:

```text
Copies carried: 3
Gateway reached: yes
```

rather than exact social graph.

---

# 44. Connection Timeline

Example:

```text
14:20:01 Direct path probing
14:20:02 Relay connected
14:20:04 Direct path succeeded
14:20:04 Switched relay → direct
```

Useful for debugging.

---

# 45. Timeline Model

```rust
pub struct PathTimelineEvent {
    pub timestamp: Timestamp,
    pub path_id: Option<PathId>,
    pub kind: PathTimelineKind,
    pub reason: Option<DiagnosticReason>,
}
```

---

# 46. Timeline Retention

Keep bounded.

Normal:

```text
recent session
```

Advanced persistent diagnostics:

```text
limited hours/days
```

Do not retain indefinite mobility/network history.

---

# 47. Diagnostic Ring Buffer

Use:

```text
bounded in-memory ring buffer
```

for high-frequency events.

Persist only selected important diagnostic events if enabled.

---

# 48. Sampling

High-frequency path metrics should be sampled/coalesced.

Example:

```text
1-second or multi-second snapshots
```

rather than per-packet.

---

# 49. Historical Metrics

Useful rolling history:

```text
RTT
throughput
loss
active path
```

for the current transfer/session.

---

# 50. Time Series

Dioxus may graph:

```text
latency over time
throughput over time
```

for advanced users.

---

# 51. Avoid Fake Precision

If transport only provides coarse estimate:

```text
show coarse
```

not invented exact values.

---

# 52. Direct vs Relay Detection

Expose:

```rust
pub enum ReachabilityMode {
    Direct,
    Relayed,
    Local,
    Dtn,
    Mixed,
}
```

---

# 53. Connection Upgrade

If:

```text
Relay → Direct
```

show:

```text
Connection improved to direct
```

only if user-facing notification is useful.

Usually record silently.

---

# 54. Connection Downgrade

If:

```text
Direct → Relay
```

communication still works.

Normal UI may remain:

```text
Connected
```

Advanced view shows downgrade.

---

# 55. Failure Taxonomy

Do not expose generic:

```text
Network error
```

for everything.

Use structured failures.

---

# 56. Failure Domains

```rust
pub enum NetworkFailureDomain {
    Dns,
    Internet,
    NatTraversal,
    Relay,
    Lan,
    Bluetooth,
    Wifi,
    Authentication,
    Capability,
    Policy,
    Resource,
    Storage,
    Timeout,
    PeerUnavailable,
}
```

---

# 57. Failure Cause

```rust
pub struct DiagnosticFailure {
    pub domain: NetworkFailureDomain,
    pub code: DiagnosticErrorCode,
    pub retryable: bool,
    pub user_action: Option<UserActionHint>,
}
```

---

# 58. User Action Hint

Examples:

```text
Enable Bluetooth
Allow Nearby Devices permission
Connect to Wi-Fi
Free storage
Disable battery saver for this transfer
Ask peer to update app
```

Only suggest actions that are actually relevant.

---

# 59. Root Cause Chain

An operation may fail because:

```text
Wi-Fi Direct unavailable
+
Bluetooth too slow
+
relay unreachable
```

Diagnostics should preserve the chain.

---

# 60. Failure Tree

```rust
pub struct FailureTree {
    pub operation: OperationId,
    pub attempts: Vec<PathAttemptFailure>,
}
```

---

# 61. Example Failure Explanation

```text
File transfer could not start.

LAN:
Peer not reachable on local network.

Wi-Fi Direct:
Permission unavailable.

Iroh direct:
NAT traversal failed.

Relay:
Relay connection timed out.
```

This is actionable.

---

# 62. Routing Attempt History

Keep:

```text
path tried
start/end time
result
reason
```

bounded per operation.

---

# 63. Connection Correlation ID

```rust
pub struct ConnectionAttemptId([u8; 16]);
```

Used across logs/traces/diagnostics.

---

# 64. Operation Correlation

Path diagnostics should attach to:

```text
MessageId
TransferId
EmergencyId
CallId
```

where applicable.

---

# 65. No Global Raw Packet Capture by Default

Packet capture is sensitive and expensive.

Do not build always-on packet logging.

---

# 66. Optional Expert Capture

A developer build may enable:

```text
protocol metadata capture
```

with explicit user action.

Payload plaintext still excluded by default.

---

# 67. Diagnostic Levels

```rust
pub enum DiagnosticLevel {
    User,
    Advanced,
    Developer,
    Operator,
}
```

---

# 68. User Level

Shows:

```text
connected/offline
direct/relayed/nearby
transfer state
actionable problem
```

---

# 69. Advanced Level

Shows:

```text
path kind
quality
relay/local
reconnects
multipath
DTN state
```

---

# 70. Developer Level

Shows:

```text
raw RTT
throughput
loss
routing reasons
path state transitions
protocol version
capabilities
```

---

# 71. Operator Level

For headless/relay infrastructure:

```text
fleet/node health
connections
relay utilization
failure rate
```

No user plaintext.

---

# 72. Privacy Filtering

All diagnostic output passes through:

```text
DiagnosticRedactor
```

before UI/export/telemetry.

---

# 73. Redaction Policy

Remove or hash:

```text
IP addresses
peer IDs
ephemeral proximity IDs
file names
message contents
precise location
```

unless explicitly required and authorized.

---

# 74. Local vs Exported Diagnostics

Local advanced view may show more.

Exported diagnostic bundle should be more aggressively redacted.

---

# 75. Diagnostic Export

User can generate:

```text
diagnostic bundle
```

for support.

Contents:

```text
build version
platform
runtime health
network state
recent failures
relay/direct ratios
resource state
redacted path timeline
```

---

# 76. Diagnostic Bundle Excludes

By default:

```text
message plaintext
file contents
private keys
auth tokens
contact graph
exact location
```

---

# 77. Export Manifest

```rust
pub struct DiagnosticBundleManifest {
    pub version: u16,
    pub created_at: Timestamp,
    pub sections: Vec<DiagnosticSection>,
    pub redaction_level: DiagnosticRedactionLevel,
}
```

---

# 78. Support Token

Optional:

```text
short diagnostic ID
```

can correlate a user report with server-side aggregate logs if user consents.

---

# 79. Local Doctor Tool

CLI:

```text
comm doctor network
```

checks:

```text
Iroh endpoint
relay reachability
LAN
Bluetooth availability
Wi-Fi capability
routing policy
resource pressure
```

---

# 80. Doctor Modes

```text
basic
full
privacy-safe export
```

---

# 81. Doctor Should Not Require Internet

Local checks can still diagnose:

```text
Bluetooth
LAN
database
permissions
```

offline.

---

# 82. Active Network Test

Full doctor may intentionally:

```text
probe configured relay
perform test handshake
measure RTT
```

with user/admin action.

---

# 83. Relay Diagnostics

Part 11 provides:

```text
relay selected
region
health
latency
TLS/connect result
```

---

# 84. Direct Path Diagnostics

Show:

```text
direct path available
NAT traversal duration
path upgraded from relay
```

where adapter provides data.

---

# 85. NAT Diagnostics

Do not expose misleading simplistic NAT labels if not reliably known.

Prefer:

```text
direct connection succeeded
direct connection failed
relay fallback used
```

---

# 86. Proximity Diagnostics

Part 14 can expose:

```text
BLE available
BLE permission
Wi-Fi Aware available
Wi-Fi Direct available
LAN peers
```

---

# 87. Permission Diagnostics

Example:

```text
Bluetooth: unavailable because permission denied
```

rather than just:

```text
Bluetooth failed
```

---

# 88. Battery Diagnostics

Part 13 provides reason:

```text
Multipath disabled by battery saver
DTN scan reduced
```

---

# 89. Resource Diagnostics

Part 08 provides:

```text
transfer deferred: storage pressure
connection denied: connection limit
```

---

# 90. Capability Diagnostics

Part 07 can explain:

```text
Video unavailable because peer does not support compatible codec.
```

---

# 91. Version Diagnostics

Show:

```text
local protocol version
peer negotiated version
```

in developer mode.

---

# 92. Protocol Mismatch

User-level:

```text
The other device needs an app update.
```

Developer:

```text
files/1 required, peer supports none
```

---

# 93. Authentication Failure

Normal UI:

```text
Could not verify the other device.
```

Never expose internal keys.

---

# 94. Security Diagnostic Events

Examples:

```text
downgrade suspected
certificate revoked
identity mismatch
```

Must be clearly distinguished from network failure.

---

# 95. Security vs Connectivity

Do not say:

```text
network unavailable
```

if actual reason is:

```text
peer identity rejected
```

---

# 96. Path Visualization UI Architecture

Dioxus components:

```text
NetworkStatusBadge
PathSummaryCard
PathGraph
PathTimeline
TransferPathView
DtnJourneyView
DiagnosticFailureView
NetworkDoctorView
```

---

# 97. Dioxus State Source

UI subscribes through Part 16 runtime API.

Do not query transports directly.

---

# 98. UI View Model

```rust
pub struct NetworkDiagnosticsViewModel {
    pub summary: NetworkSummaryVm,
    pub path_graph: PathGraphVm,
    pub timeline: Vec<TimelineVm>,
    pub actions: Vec<DiagnosticActionVm>,
}
```

---

# 99. Normal Network Screen

Suggested:

```text
Connection
──────────
Connected directly

Quality
Good

Active path
Wi-Fi → Peer

Fallback
Relay available
```

---

# 100. Advanced Network Screen

Suggested sections:

```text
Overview
Active paths
Candidate paths
Routing decisions
Recent transitions
DTN
Relay
Permissions
Power/resource constraints
```

---

# 101. Visual Path States

Use distinguishable visual states:

```text
active
standby
degraded
failed
```

Do not rely solely on color; use icons/line styles/text for accessibility.

---

# 102. Accessibility

Path graphs should have equivalent textual description.

Screen reader example:

```text
Primary path: Wi-Fi direct to peer. Secondary path: cellular standby.
```

---

# 103. Mobile UI

On mobile, use progressive disclosure.

Default:

```text
simple status
```

Tap:

```text
connection details
```

Tap advanced:

```text
full diagnostics
```

---

# 104. Desktop UI

Can show richer:

```text
graph
timeline
metrics charts
```

---

# 105. Headless CLI

Examples:

```text
comm network status
comm network paths
comm network timeline
comm network doctor
```

---

# 106. Machine-Readable Output

CLI supports:

```text
RON
JSON
```

JSON is justified for external tooling.

---

# 107. Runtime Diagnostic API

```rust
pub trait NetworkDiagnosticApi {
    async fn snapshot(
        &self,
        level: DiagnosticLevel,
    ) -> Result<NetworkDiagnosticSnapshot, DiagnosticError>;

    fn subscribe(
        &self,
        filter: DiagnosticFilter,
    ) -> DiagnosticEventStream;
}
```

---

# 108. Diagnostic Snapshot

```rust
pub struct NetworkDiagnosticSnapshot {
    pub generated_at: Timestamp,
    pub health: NetworkHealthSummary,
    pub paths: Vec<PathDiagnostic>,
    pub routing: Vec<RoutingExplanation>,
    pub failures: Vec<DiagnosticFailure>,
}
```

---

# 109. Diagnostic Event Filtering

Clients can subscribe to:

```text
all
routing
transport
DTN
relay
proximity
security
```

---

# 110. Backpressure

Part 08 applies to diagnostics.

High-frequency telemetry must not crash the daemon/UI.

Use:

```text
bounded queues
sampling
coalescing
latest-value channels
```

---

# 111. Diagnostics Must Not Affect Networking

If diagnostics consumer is slow:

```text
drop/coalesce diagnostics
```

Never slow critical networking.

---

# 112. Diagnostic Priority

Diagnostic events are generally:

```text
low/background
```

except security/user-action states.

---

# 113. Metrics Aggregator

```rust
pub struct NetworkMetricsAggregator {
    // rolling counters/windows
}
```

Consumes events without blocking transports.

---

# 114. Rolling Windows

Examples:

```text
10 seconds
1 minute
15 minutes
```

for local quality summaries.

---

# 115. Histograms

Useful for:

```text
RTT
connection establishment time
transfer throughput
```

operator metrics.

---

# 116. Cardinality Discipline

Do not label exported metrics with raw:

```text
PeerId
MessageId
TransferId
```

unless local-only diagnostic.

---

# 117. OpenTelemetry

Optional operator integration:

```text
metrics
traces
logs
```

Keep behind feature flag/adapter.

---

# 118. Prometheus

Headless/relay deployments may expose metrics.

Not required in mobile/desktop binary.

---

# 119. Structured Logging

Use `tracing` ecosystem.

Recommended fields:

```text
subsystem
operation kind
path kind
reason code
duration
result
```

---

# 120. Span Architecture

Example:

```text
send_message
  ├── route_plan
  ├── connect
  ├── authenticate
  └── transmit
```

---

# 121. File Transfer Spans

```text
file_transfer
  ├── manifest
  ├── route selection
  ├── chunk workers
  └── finalize
```

Avoid one trace span per byte/chunk for huge files unless sampled.

---

# 122. DTN Tracing

Track:

```text
bundle created
encounter
forwarded
gateway
destination ACK
```

with privacy-safe relay identifiers.

---

# 123. Emergency Tracing

Critical delivery diagnostics should be locally detailed.

Export remains redacted.

---

# 124. Failure Reason Stability

Diagnostic reason codes should be stable API values.

This helps support tooling.

---

# 125. Diagnostic Error Code

```rust
pub struct DiagnosticErrorCode(pub u32);
```

Map canonical codes to documentation.

---

# 126. Self-Service Troubleshooting

For common issues:

```text
Bluetooth permission denied
relay unreachable
storage full
peer outdated
```

provide guided remediation.

---

# 127. Remediation Action

```rust
pub enum DiagnosticAction {
    OpenPermissions,
    Retry,
    SwitchNetwork,
    FreeStorage,
    UpdateApp,
    RunNetworkTest,
}
```

---

# 128. UI Must Not Invent Diagnosis

Actions come from structured diagnostics.

Do not make UI guess based on strings.

---

# 129. Network Doctor Architecture

```text
Check platform state
 ↓
Check local interfaces
 ↓
Check proximity permissions
 ↓
Check Iroh endpoint
 ↓
Check configured relays
 ↓
Check routing policy
 ↓
Check resource/power constraints
 ↓
Generate report
```

---

# 130. Doctor Check Model

```rust
pub struct DiagnosticCheckResult {
    pub check: DiagnosticCheckId,
    pub status: CheckStatus,
    pub explanation: DiagnosticReason,
    pub action: Option<DiagnosticAction>,
}
```

---

# 131. Check Status

```rust
pub enum CheckStatus {
    Pass,
    Warning,
    Fail,
    Skipped,
}
```

---

# 132. Active Probe Consent

Network doctor may create actual connections.

Make this explicit if it may consume:

```text
mobile data
battery
```

---

# 133. Offline Doctor

When Internet unavailable, it should still return useful results rather than fail whole diagnostic process.

---

# 134. Historical Path Record

For selected operations, persist compact path transition summaries.

Example:

```text
Message:
Relay → Direct → Delivered
```

This may aid support.

---

# 135. Persistence Policy

Default persistent network history should be minimal.

Potential:

```text
last failure code
last successful path class
operation transition summary
```

---

# 136. No Location History via Network Metadata

Do not use Wi-Fi/Bluetooth diagnostics to build persistent movement history.

---

# 137. Retention Configuration

```rust
pub struct DiagnosticRetention {
    pub event_buffer: usize,
    pub persistent_days: u16,
    pub raw_metrics_minutes: u16,
}
```

Defaults conservative.

---

# 138. Diagnostic Storage Class

Part 08 should classify diagnostic persistence as:

```text
Cache/Operational
```

not authoritative user data.

Can be evicted under pressure.

---

# 139. Crash Recovery

Part 09 does not need to restore live path objects.

After restart:

```text
new path discovery
```

Persisted recent failure summaries may survive.

---

# 140. Daemon Integration

Part 16 daemon owns the diagnostic aggregator.

Multiple UI/CLI clients subscribe.

---

# 141. Client Isolation

One diagnostics-heavy client cannot force unbounded work.

Per-client level/rate limits apply.

---

# 142. Headless Node Diagnostics

Headless mode exposes:

```text
local CLI
admin API
metrics
health
```

---

# 143. Relay Fleet Diagnostics

Part 11 operators need:

```text
relay health
active connections
traffic
failure rate
region state
```

The client diagnostic model should remain distinct from fleet metrics.

---

# 144. Fleet vs Client

Client:

```text
Why is my path relayed?
```

Operator:

```text
Why did relay-region-IN error rate spike?
```

Different views, shared reason codes where useful.

---

# 145. Versioned Diagnostic Schema

Diagnostics may be consumed externally.

Version DTOs.

```text
diagnostics/1
```

---

# 146. FFI Integration

Part 19 can expose:

```text
network summary
path list
diagnostic events
```

without exposing Rust internals.

---

# 147. Third-Party Extension Diagnostics

Part 22 extensions may register:

```text
diagnostic reason codes
health component
```

through a bounded namespace.

---

# 148. Plugin Diagnostics

Part 24 plugins should not access sensitive core diagnostics by default.

Use permission scopes.

---

# 149. Security Permission Scopes

Example:

```text
DiagnosticsBasic
DiagnosticsAdvanced
DiagnosticsSensitiveLocal
```

---

# 150. Sensitive Diagnostics

Examples:

```text
IP address
relay host
full DeviceId
```

require explicit advanced/local permission.

---

# 151. Test Strategy

Unit:

```text
reason mapping
redaction
quality classes
state transitions
```

Integration:

```text
routing → diagnostics
multipath → graph
DTN → journey
```

---

# 152. Property Tests

Invariants:

```text
redacted export contains no private keys
normal diagnostic level never exposes raw IP
path graph edges reference existing nodes
failure chain remains bounded
```

---

# 153. Fuzzing

Part 10 fuzz:

```text
diagnostic export parser
admin diagnostic protocol
path graph serialization
```

---

# 154. Slow Consumer Test

Diagnostic UI stops reading.

Expected:

```text
network remains unaffected
diagnostic queue remains bounded
```

---

# 155. Path Flap Test

Rapid:

```text
direct ↔ relay
```

Expected:

```text
timeline coalesced
no UI storm
```

---

# 156. Multipath Test

Paths activate/deactivate.

Graph correctly updates roles and epoch.

---

# 157. DTN Test

Multi-hop journey displays:

```text
carried
gateway
delivered
```

without exposing unknown relay identities.

---

# 158. Privacy Export Test

Generate support bundle.

Assert absence of:

```text
message plaintext
private keys
raw contact IDs
precise location
raw nearby IDs
```

---

# 159. Permission Failure Test

Bluetooth denied.

Doctor returns:

```text
Fail
domain=Bluetooth
action=OpenPermissions
```

---

# 160. Relay Failure Test

Relay unavailable.

Diagnostics show:

```text
direct attempt
relay timeout
alternate candidate
```

---

# 161. Capability Failure Test

Peer lacks file capability.

Do not report:

```text
network error
```

Report:

```text
CapabilityMissing
```

---

# 162. Resource Failure Test

Storage full.

Network path may be healthy.

Transfer diagnosis must say:

```text
Storage
```

not network.

---

# 163. Authentication Failure Test

Path connected physically but peer verification fails.

UI shows security verification failure.

---

# 164. Crash Test

Daemon crashes and restarts.

Live diagnostics reset.

Recent persisted diagnostic summary remains bounded.

---

# 165. Performance

Diagnostic hot-path overhead should be minimal.

Avoid:

```text
large allocations
string formatting
JSON serialization
```

inside packet/stream loops.

---

# 166. Structured Internal Events

Use enums/IDs internally.

Format strings only at presentation/export boundary.

---

# 167. Zero/Low Allocation Events

Where performance-sensitive:

```rust
struct PathMetricSample {
    path: PathId,
    rtt_micros: u32,
    bytes: u64,
}
```

bounded channel to aggregator.

---

# 168. Sampling Budget

If event rate too high:

```text
sample
aggregate
coalesce
```

---

# 169. Metrics Clock

Use monotonic time for durations.

Wall clock for human timeline labels.

---

# 170. Clock Drift

Do not compute RTT from wall-clock timestamps.

---

# 171. Diagnostic IDs

Recommended:

```text
NET-DIRECT-001
NET-RELAY-002
NET-BLE-003
NET-POLICY-004
```

Useful for documentation/support.

---

# 172. Documentation Link Mapping

UI can map known reason code to local help article.

Avoid server dependency for basic explanations.

---

# 173. Dioxus Component Tree

```text
NetworkDiagnosticsPage
├── NetworkOverviewCard
├── ActivePathCard
├── PathGraph
├── QualityMetricsPanel
├── RoutingReasonPanel
├── RecentTransitions
├── DtnJourneyPanel
└── TroubleshootingPanel
```

---

# 174. Mobile Layout

Mobile:

```text
Overview
 ↓
Current path
 ↓
Problem/action
 ↓
Advanced details collapsed
```

---

# 175. Desktop Layout

Desktop may use:

```text
left path graph
right status/details
bottom timeline
```

---

# 176. Visual Simplicity

Do not turn diagnostics into a network-engineering cockpit by default.

Advanced information must be opt-in.

---

# 177. Color Accessibility

Never communicate:

```text
healthy/degraded/failed
```

only via green/yellow/red.

Use:

```text
icons
labels
line styles
```

---

# 178. Graph Scalability

Normal path graph is small.

DTN history could be larger.

Bound visualization:

```text
last N logical hops
aggregate unknown relays
```

---

# 179. DTN Long Journey

Instead of rendering 50 nodes:

```text
This device
→ 12 nearby relay hops
→ Gateway
→ Destination
```

---

# 180. Topology Is Approximate

The platform usually knows only observed path segments.

Do not imply a complete Internet topology.

---

# 181. Uncertainty Representation

Use:

```text
Unknown intermediate network
```

where topology not known.

---

# 182. Diagnostic Graph Semantics

Graph represents:

```text
logical communication route
```

not physical router-by-router topology.

---

# 183. Network Quality Summary

```rust
pub struct NetworkHealthSummary {
    pub state: ConnectivityState,
    pub quality: OverallQuality,
    pub primary_path: Option<PathId>,
    pub issue: Option<DiagnosticFailure>,
}
```

---

# 184. Overall Quality

```rust
pub enum OverallQuality {
    Excellent,
    Good,
    Fair,
    Poor,
    Unusable,
    Unknown,
}
```

---

# 185. Quality Calculation

Should use:

```text
operation context
latency
loss
throughput
stability
```

A 1 Mbps link may be excellent for text but poor for video.

---

# 186. Context-Aware Quality

```rust
pub enum DiagnosticWorkload {
    Messaging,
    FileTransfer,
    AudioCall,
    VideoCall,
    Dtn,
}
```

Quality should be workload-specific when useful.

---

# 187. Example

```text
Network:
Good for messaging
Poor for video call
```

This is more informative than a universal signal score.

---

# 188. Diagnostic Snapshot Consistency

Snapshot should represent one coherent observation point as much as practical.

Use:

```text
snapshot generation
```

---

# 189. Snapshot Generation

```rust
pub struct DiagnosticGeneration(u64);
```

Helps UI discard stale updates.

---

# 190. Live Update Delta

```rust
pub struct DiagnosticDelta {
    pub generation: DiagnosticGeneration,
    pub changes: Vec<DiagnosticChange>,
}
```

---

# 191. Missed Delta

Client detects gap:

```text
request fresh snapshot
```

same approach as Part 16.

---

# 192. Public Crate Structure

```text
crates/comm-diagnostics/
├── src/
│   ├── lib.rs
│   ├── snapshot.rs
│   ├── event.rs
│   ├── path.rs
│   ├── graph.rs
│   ├── quality.rs
│   ├── routing.rs
│   ├── failure.rs
│   ├── timeline.rs
│   ├── metrics.rs
│   ├── retention.rs
│   ├── redaction.rs
│   ├── export.rs
│   ├── doctor.rs
│   └── error.rs
└── Cargo.toml
```

---

# 193. UI Crate

```text
crates/comm-ui-diagnostics/
├── src/
│   ├── overview.rs
│   ├── path_graph.rs
│   ├── timeline.rs
│   ├── metrics.rs
│   ├── dtn.rs
│   ├── troubleshooting.rs
│   └── view_model.rs
```

Dioxus-only dependency stays here.

---

# 194. Headless/Admin Components

```text
crates/comm-diagnostics-cli/
crates/comm-diagnostics-otel/
```

optional.

---

# 195. Public API

```rust
let snapshot = diagnostics
    .snapshot(DiagnosticLevel::Advanced)
    .await?;
```

---

# 196. Path Query

```rust
let paths = diagnostics.paths_for(transfer_id).await?;
```

---

# 197. Explanation Query

```rust
let why = diagnostics
    .explain_operation(operation_id)
    .await?;
```

This is a major usability feature.

---

# 198. Doctor API

```rust
let report = diagnostics
    .run_doctor(NetworkDoctorMode::Basic)
    .await?;
```

---

# 199. Export API

```rust
let bundle = diagnostics
    .export(DiagnosticExportPolicy::PrivacySafe)
    .await?;
```

---

# 200. Error Model

```rust
pub enum DiagnosticError {
    NotAvailable,
    PermissionDenied,
    SnapshotTooLarge,
    ExportFailed,
    Unsupported,
    Cancelled,
}
```

---

# 201. Initial Production Scope

Implement first:

```text
ConnectivityState
PathDiagnostic
RoutingExplanation
failure taxonomy
direct/relay/local indicators
file transfer path status
DTN delivery status
bounded timeline
network doctor
privacy-safe export
Dioxus diagnostics page
CLI diagnostics
```

Then:

```text
multipath graph
historical charts
OpenTelemetry adapter
support bundles
advanced DTN journey visualization
```

Defer initially:

```text
packet capture UI
router-by-router topology maps
automatic remote telemetry upload
complex ML diagnosis
```

---

# 202. Implementation Phases

## Phase 1 — Diagnostic Model

```text
path
state
quality
failure
```

## Phase 2 — Routing Integration

```text
selected/rejected reasons
attempt history
```

## Phase 3 — Transport Integration

```text
Iroh
relay
LAN
BLE
Wi-Fi
```

## Phase 4 — File/DTN/Multipath

```text
transfer paths
DTN journey
multiple paths
```

## Phase 5 — UI / CLI

```text
Dioxus
doctor
timeline
troubleshooting
```

## Phase 6 — Privacy / Export

```text
redaction
support bundle
retention
```

## Phase 7 — Operator Observability

```text
metrics
tracing
OpenTelemetry
headless health
```

## Phase 8 — Hardening

```text
fuzz
privacy tests
slow consumers
path flapping
large histories
```

---

# 203. Definition of Done

Part 18 is complete when:

- the runtime can report whether communication is direct, relayed, local, DTN, or mixed
- every active path has explicit lifecycle state
- routing can explain why one path was selected
- rejected paths have structured reason codes
- file transfers can show which paths carry data
- multipath roles can be visualized
- DTN can show carried/gateway/delivered state without exposing arbitrary relay identities
- diagnostics distinguish network, capability, security, storage, resource, and permission failures
- normal UI remains simple
- advanced/developer views expose detailed metrics
- path timelines are bounded and coalesced
- diagnostic consumers cannot backpressure networking
- support exports redact sensitive data
- network doctor can run online or offline
- Dioxus uses runtime diagnostic APIs rather than transports directly
- headless CLI/admin diagnostics work without UI
- privacy, fuzz, path-flap, slow-consumer, and failure-taxonomy tests exist

---

# 204. Relationship to Earlier Parts

Part 18 builds on and observes:

```text
01 — Protocol Extension System
02 — Multi-Device Identity
03 — Transport & Routing Policy Engine
04 — Offline Event Log
05 — Robust File / Blob Subsystem
06 — DTN / Store-Carry-Forward
07 — Capability Negotiation
08 — Resource Limits & Backpressure
09 — Crash Recovery
10 — Fuzzing & Protocol Test Suite
11 — Relay / Self-Hosted Infrastructure
12 — Multipath Networking
13 — Battery-Aware Scheduling
14 — Proximity Abstraction
15 — QR / NFC Bootstrap
16 — Daemon & Headless Runtime
17 — Emergency Priority Classes
```

It directly supports:

```text
19 — C ABI / FFI
20 — Embedded Linux Node
21 — WASM-Compatible Components
22 — Third-Party Protocol Extensions
23 — External Interoperability Suite
24 — Plugin / Module Ecosystem
```

---

# 205. Final Architecture

```text
                    NETWORK RUNTIME
                         │
      ┌──────────────────┼──────────────────┐
      │                  │                  │
   Routing            Transports          DTN
      │                  │                  │
      └──────────── Diagnostic Events ──────┘
                         │
               Diagnostics Aggregator
              ┌──────────┼───────────┐
              │          │           │
           Snapshot    Timeline    Metrics
              │          │           │
              └──────────┼───────────┘
                         │
                 Privacy Redaction
                         │
          ┌──────────────┼───────────────┐
          │              │               │
       Dioxus UI        CLI          Operator
```

Example user path:

```text
This Device
    │
    │ Wi-Fi / Direct
    ▼
Peer Device
```

Example relayed path:

```text
This Device
    │
    ▼
Iroh Relay
    │
    ▼
Peer Device
```

Example resilient emergency journey:

```text
This Device
    │ BLE
    ▼
Nearby Relay
    │ store/carry
    ▼
Gateway
    │ Iroh
    ▼
Destination
```

---

# 206. Final Principle

A sophisticated networking stack should not be a black box.

When a transfer is slow, the system should be able to say:

```text
The peer is reachable.
Direct Internet connectivity failed.
A relay path is active.
The relay is healthy.
The transfer is throttled because the device is in battery-saver mode.
```

When an emergency message is offline:

```text
Stored locally.
Carried by 2 nearby relay devices.
No Internet gateway yet.
```

When connectivity improves:

```text
Gateway reached.
Delivered.
```

And when something fails, the product should distinguish:

```text
network failure
security failure
permission failure
capability mismatch
resource pressure
storage failure
```

instead of presenting every problem as "Connection failed."

That explainability is essential for users, developers, support teams, emergency operators, and any external product embedding the communication platform.
