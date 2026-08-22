# Part 03 — Transport & Routing Policy Engine Architecture

## Reusable P2P Communication Platform

**Status:** Architecture specification  
**Part:** 03 of 24  
**Primary language:** Rust  
**Core transport:** Iroh-first, transport-neutral  
**Primary goals:** intelligent path selection, transport abstraction, failover, multipath readiness, battery awareness, DTN compatibility, predictable policy, reusable across messaging/files/calls/emergency/ERP products

---

# 1. Purpose

The communication platform must support multiple transport mechanisms without forcing each application feature to understand the differences between them.

Potential paths include:

```text
Iroh direct
Iroh relay
local LAN
Wi-Fi Direct
Wi-Fi Aware
Bluetooth Classic
Bluetooth LE
application-level mesh
DTN/store-carry-forward
future transports
```

The transport/routing policy engine is responsible for deciding:

```text
Which path should carry this operation?
When should the system retry?
When should it switch paths?
When should it use multiple paths?
When should it defer?
When should it fall back to DTN?
```

The application should express **intent and constraints**, not transport mechanics.

---

# 2. Fundamental Rule

Do not let feature code implement transport selection.

Bad:

```text
Messaging:
    try Iroh
    if failed try LAN
    if failed try Bluetooth
```

Bad:

```text
File transfer:
    if Wi-Fi then use Wi-Fi
    else use Internet
```

Correct:

```text
Application Operation
       ↓
Delivery Requirements
       ↓
Routing Policy Engine
       ↓
Path Candidates
       ↓
Scoring + Constraints
       ↓
Selected Route
       ↓
Transport Adapter
```

This separation is essential for reuse.

---

# 3. Architectural Position

```text
Application
   ↓
Messaging / Files / Calls / Emergency
   ↓
Delivery Intent
   ↓
Routing Policy Engine
   ↓
Transport Manager
   ↓
Iroh / LAN / Wi-Fi / Bluetooth / DTN
```

Identity feeds destination resolution into routing.

Transport adapters feed measured path state into routing.

The routing engine sits between application semantics and transport mechanics.

---

# 4. Core Responsibilities

The routing engine owns:

- candidate path discovery
- route scoring
- policy validation
- fallback order
- path health tracking
- retry decisions
- transport switching
- path stickiness
- multipath planning
- bandwidth-aware selection
- latency-aware selection
- battery-aware selection
- metered-network awareness
- emergency priority handling
- DTN eligibility
- route diagnostics
- path history
- per-peer route memory

It does **not** own:

- message semantics
- file chunking
- codec negotiation
- cryptographic identity
- user interface
- transport implementation details

---

# 5. Main Abstractions

Recommended types:

```rust
pub struct RouteRequest;
pub struct RouteDecision;
pub struct PathCandidate;
pub struct PathMetrics;
pub struct DeliveryRequirements;
pub struct RoutingPolicy;
pub struct RoutePlan;
pub struct RouteHealth;
```

Transport identity and account/device identity remain separate types from Part 02.

---

# 6. Delivery Requirements

Feature layers describe what they need.

```rust
pub struct DeliveryRequirements {
    pub class: DeliveryClass,
    pub priority: Priority,
    pub max_latency: Option<Duration>,
    pub min_bandwidth: Option<Bitrate>,
    pub durable: bool,
    pub allow_metered: bool,
    pub allow_relay: bool,
    pub allow_bluetooth: bool,
    pub allow_dtn: bool,
    pub allow_multipath: bool,
    pub expiry: Option<Timestamp>,
    pub max_cost: Option<NetworkCost>,
}
```

These fields should be strongly typed where practical.

---

# 7. Delivery Classes

```rust
pub enum DeliveryClass {
    Realtime,
    Interactive,
    Reliable,
    Bulk,
    DelayTolerant,
}
```

Examples:

```text
video call frame → Realtime
text message → Interactive/Reliable
receipt → Interactive
file transfer → Bulk/Reliable
SOS → Reliable/DelayTolerant
typing indicator → Realtime but non-durable
```

---

# 8. Priority

Use a small explicit scale.

```rust
pub enum Priority {
    Critical,
    High,
    Normal,
    Low,
    Background,
}
```

Priority affects scheduling but does not override hard constraints.

Example:

```text
Critical SOS
```

can use Bluetooth/DTN if needed.

A `Critical` 4 GB video should still not be forced over BLE if impossible.

---

# 9. Path Candidate

Each possible route is normalized:

```rust
pub struct PathCandidate {
    pub path_id: PathId,
    pub transport: TransportKind,
    pub peer: DeviceId,
    pub endpoint: TransportEndpoint,
    pub metrics: PathMetrics,
    pub capabilities: PathCapabilities,
    pub health: RouteHealth,
}
```

Routing works on this generic representation.

---

# 10. Transport Kinds

```rust
pub enum TransportKind {
    IrohDirect,
    IrohRelay,
    LocalLan,
    WifiDirect,
    WifiAware,
    BluetoothClassic,
    BluetoothLe,
    MeshRelay,
    Dtn,
}
```

Future transports can be added without changing messaging/file APIs.

---

# 11. Path Capabilities

```rust
pub struct PathCapabilities {
    pub reliable_stream: bool,
    pub datagram: bool,
    pub large_files: bool,
    pub realtime_media: bool,
    pub peer_discovery: bool,
    pub store_and_forward: bool,
    pub metered: bool,
}
```

This prevents impossible route choices.

---

# 12. Path Metrics

Metrics should include:

```rust
pub struct PathMetrics {
    pub rtt: Option<Duration>,
    pub estimated_bandwidth: Option<Bitrate>,
    pub packet_loss: Option<Ratio>,
    pub jitter: Option<Duration>,
    pub stability: StabilityScore,
    pub energy_cost: EnergyCost,
    pub monetary_cost: NetworkCost,
    pub signal_quality: Option<SignalQuality>,
    pub last_success: Option<Timestamp>,
}
```

Not all transports expose all metrics.

Missing metrics must be represented explicitly.

---

# 13. Metrics Confidence

Every metric should have confidence/age metadata.

Example:

```rust
MeasuredValue<T> {
    value: T,
    observed_at: Timestamp,
    confidence: Confidence,
}
```

Routing must not treat a 20-minute-old bandwidth estimate as current truth.

---

# 14. Route Health

```rust
pub enum RouteHealth {
    Healthy,
    Degraded,
    Suspect,
    Unreachable,
    Unknown,
}
```

Health is derived from:

- recent failures
- timeouts
- connection churn
- path changes
- transport errors

---

# 15. Path Discovery Sources

Candidates may come from:

```text
Iroh discovery
known endpoint cache
LAN discovery
Wi-Fi Direct/Aware discovery
Bluetooth discovery
recent peer encounter
DTN forwarding table
manual/QR endpoint
```

Discovery is separate from routing.

Routing consumes candidates.

---

# 16. Destination Resolution

Routing should accept logical targets:

```rust
pub enum Destination {
    Account(AccountId),
    Device(DeviceId),
    Group(GroupId),
}
```

Resolution flow:

```text
AccountId
   ↓
Device Directory
   ↓
active devices
   ↓
known transport endpoints
   ↓
candidate paths
```

Part 02 provides device membership.

---

# 17. Account-Level Routing

A request to:

```text
send to account
```

may produce a route plan to multiple devices.

Example:

```text
Bob:
Phone reachable via Iroh direct
Laptop offline
Tablet reachable via LAN
```

Messaging may fan out.

File transfer may choose a specific target.

Routing must support both.

---

# 18. Route Plan

A route decision can be more than one path.

```rust
pub struct RoutePlan {
    pub primary: PathCandidate,
    pub fallbacks: Vec<PathCandidate>,
    pub replicas: Vec<PathCandidate>,
    pub strategy: RouteStrategy,
}
```

Strategies:

```rust
pub enum RouteStrategy {
    Single,
    Failover,
    Redundant,
    Multipath,
    DelayTolerant,
}
```

---

# 19. Single Route

For ordinary traffic:

```text
one best path
```

Example:

```text
small text message
→ Iroh direct
```

This avoids unnecessary duplication.

---

# 20. Failover Route

Plan:

```text
Primary:
Iroh direct

Fallback:
Iroh relay

Fallback:
LAN

Fallback:
DTN
```

Failover does not send simultaneously.

It switches when policy or health dictates.

---

# 21. Redundant Route

For critical traffic:

```text
send same logical operation
over more than one independent route
```

Example:

```text
SOS
├── Internet
└── nearby mesh
```

Receiver deduplicates via stable operation/message ID.

Use redundancy sparingly.

---

# 22. Multipath Route

Multipath means splitting one operation across multiple paths.

Useful for:

```text
large file
future live media
high-resilience transfer
```

Example:

```text
Chunk ranges:
Wi-Fi → chunks 0–49
Internet → chunks 50–99
```

This should be a later optimization, not required for v1 routing.

---

# 23. Delay-Tolerant Route

If no continuous path exists:

```text
persist operation
 ↓
select DTN bundle policy
 ↓
wait for peer encounter
 ↓
forward opportunistically
```

Routing should treat DTN as a legitimate delivery strategy, not merely an error fallback.

---

# 24. Route Scoring

A candidate route receives a score.

Conceptually:

```text
score =
    reachability
  + latency suitability
  + bandwidth suitability
  + stability
  + energy suitability
  + monetary-cost suitability
  + privacy policy
  + recent success
  - congestion
  - failure penalty
```

Do not hard-code one universal numeric formula.

Use typed weighted policy.

---

# 25. Hard Constraints vs Soft Preferences

Hard constraint:

```text
realtime media requires realtime-capable path
```

Soft preference:

```text
prefer direct over relay
```

Evaluation order:

```text
1. eliminate paths violating hard constraints
2. score remaining paths
3. apply stickiness/hysteresis
4. produce route plan
```

---

# 26. Path Scoring Interface

```rust
pub trait PathScorer {
    fn score(
        &self,
        candidate: &PathCandidate,
        req: &DeliveryRequirements,
        context: &RoutingContext,
    ) -> RouteScore;
}
```

Allow product profiles to supply custom scorers.

---

# 27. Policy Profiles

Recommended defaults:

```text
Balanced
LowLatency
LowPower
LowCost
HighReliability
Emergency
BulkTransfer
```

Applications can use:

```rust
RoutingPolicyProfile::Balanced
```

instead of manually setting dozens of fields.

---

# 28. Balanced Policy

Typical:

```text
prefer direct
prefer unmetered
avoid high battery cost
use relay if needed
use DTN for durable traffic only
```

Suitable for ordinary messenger usage.

---

# 29. Low-Latency Policy

For calls:

```text
minimize RTT
penalize jitter/loss
avoid path switching
avoid DTN
avoid BLE
```

Bandwidth and stability matter more than monetary cost unless user policy forbids.

---

# 30. Low-Power Policy

For battery saver:

```text
prefer existing connection
avoid active discovery
avoid Wi-Fi Direct setup
reduce multipath
avoid Bluetooth scanning escalation
```

The routing engine should understand setup cost, not just transmission cost.

---

# 31. Low-Cost Policy

For metered networks:

```text
prefer LAN/Wi-Fi
delay bulk traffic
allow small control traffic
```

Useful for large files.

---

# 32. High-Reliability Policy

For important business data:

```text
prefer proven paths
allow retry
possibly redundant control delivery
persist before send
```

---

# 33. Emergency Policy

Emergency mode may:

```text
allow mesh
allow DTN
increase discovery
allow redundancy
prioritize critical payloads
ignore some cost preferences
```

but should still remain battery-aware.

---

# 34. Route Stickiness

Do not switch routes whenever a slightly better score appears.

Example:

```text
Iroh direct stable
Wi-Fi briefly appears
```

Do not churn connection unnecessarily.

Use hysteresis:

```text
switch only if:
new_score > current_score + threshold
or current route degraded
```

---

# 35. Path Hysteresis

```rust
pub struct HysteresisPolicy {
    pub switch_threshold: RouteScoreDelta,
    pub minimum_hold_time: Duration,
    pub degraded_override: bool,
}
```

This reduces:

- packet reordering
- reconnect overhead
- codec disruption
- battery use

---

# 36. Route Failure Classification

Classify failures:

```text
Temporary
TransportUnavailable
AuthenticationFailure
PolicyDenied
RemoteRejected
Permanent
Unknown
```

Routing should retry only when appropriate.

---

# 37. Failure Examples

```text
Timeout → Temporary
Bluetooth disabled → TransportUnavailable
Revoked device → AuthenticationFailure
Metered forbidden → PolicyDenied
Unsupported extension → RemoteRejected
Invalid destination → Permanent
```

Do not blindly retry all failures.

---

# 38. Retry Policy

```rust
pub struct RetryPolicy {
    pub max_attempts: Option<u32>,
    pub initial_backoff: Duration,
    pub max_backoff: Duration,
    pub jitter: Ratio,
    pub retry_on_network_change: bool,
}
```

Durable messages may retry indefinitely until expiry.

Typing indicators should not retry.

---

# 39. Retry on Connectivity Change

A connectivity event can bypass backoff.

Example:

```text
message waiting
 ↓
Wi-Fi restored
 ↓
retry immediately
```

This improves perceived latency.

---

# 40. Path Memory

Maintain per-peer route history:

```text
last successful path
recent RTT
recent failures
known local route
last Internet gateway
```

Use it as a hint.

Do not treat historical state as proof of current reachability.

---

# 41. Route Cache

Cache:

```text
Destination
Best Known Path
Fallbacks
Observed At
TTL
```

Invalidate on:

- device directory update
- network transition
- transport shutdown
- authentication failure

---

# 42. Network Transition Events

Routing must react to:

```text
Wi-Fi connected
Wi-Fi disconnected
cellular connected
Bluetooth enabled
Bluetooth disabled
LAN route discovered
Iroh path changed
app backgrounded
battery saver enabled
```

Events come from platform/transport layers.

---

# 43. Route Re-Evaluation

Do not recompute all routes on every small event.

Use targeted invalidation:

```text
Wi-Fi changed
→ reevaluate Wi-Fi-related candidates
```

This preserves efficiency.

---

# 44. Transport Setup Cost

Some paths require setup.

Example:

```text
existing Iroh session → cheap
Wi-Fi Direct group creation → expensive
Bluetooth pairing → expensive
```

Route scoring should include setup latency/cost.

For a 200-byte message, creating Wi-Fi Direct may be worse than using relay.

---

# 45. Existing Connection Preference

Prefer already-established healthy sessions.

This reduces:

- latency
- CPU
- radio activation
- battery
- handshake cost

unless another path clearly dominates.

---

# 46. Connection Pool Integration

Transport Manager should expose:

```text
active connection
connecting
idle
unreachable
```

Routing can reuse pooled connections.

Do not create a connection per message.

---

# 47. Peer Session Abstraction

Routing should route through authenticated sessions, not raw transport addresses.

```text
DeviceId
 ↓
Path
 ↓
Transport Connection
 ↓
Authenticated Session
```

This ties routing back to Part 02 identity.

---

# 48. Security Constraints

Routing cannot override:

```text
identity trust
authorization
encryption requirements
revocation
```

A high-scoring path to a revoked device must never be selected.

---

# 49. Privacy Policy

Routing may consider privacy preferences:

```text
PreferDirect
AvoidRelay
AllowRelay
AvoidMetered
NearbyOnly
InternetOnly
```

These are user/application policies.

Do not make privacy decisions implicit.

---

# 50. Direct vs Relay Preference

Default:

```text
direct healthy path
>
relay path
```

But not absolutely.

A severely degraded direct path may be worse than a stable relay.

Scoring should reflect actual quality.

---

# 51. LAN Preference

For local same-router communication:

```text
LAN direct
```

often provides:

- low latency
- high bandwidth
- no Internet dependency

It may outrank Internet relay when authenticated and available.

---

# 52. Wi-Fi Direct/Aware

Useful when:

```text
no router
nearby devices
high bandwidth required
```

But setup overhead means it should not be created for every tiny message.

Use threshold policy:

```text
large transfer
active call
explicit nearby session
```

---

# 53. Bluetooth Classic

Potentially useful for:

```text
small/medium messages
fallback
local sync
```

but generally lower bandwidth than Wi-Fi.

Routing should understand its throughput class.

---

# 54. Bluetooth LE

BLE is suited for:

```text
discovery
control
SOS
small messages
DTN handoff
```

Do not route:

```text
large video
realtime AV1 calls
huge file
```

over BLE unless explicitly forced for tiny chunks/emergency policy.

---

# 55. Mesh Forwarding

Mesh path candidate represents:

```text
next hop
estimated route utility
hop budget
relay trust policy
```

It is not the same as direct transport.

Routing decides whether to hand payload to mesh/DTN subsystem.

---

# 56. DTN Routing Boundary

The Routing Policy Engine decides:

```text
DTN allowed?
priority?
expiry?
replication budget?
```

The DTN subsystem decides:

```text
which encountered peer receives bundle?
```

Do not duplicate DTN algorithms inside general routing.

---

# 57. Delivery Semantics

Feature layers must label operations.

Examples:

```text
typing:
non-durable
expires quickly
no DTN

message:
durable
retry
DTN allowed

file chunk:
resumable
bulk
DTN optional

call frame:
realtime
drop stale frames
```

This prevents incorrect route behavior.

---

# 58. Operation Descriptor

```rust
pub struct OperationDescriptor {
    pub operation_id: OperationId,
    pub destination: Destination,
    pub requirements: DeliveryRequirements,
    pub estimated_size: ByteCount,
    pub content_class: ContentClass,
}
```

Routing receives metadata, not application payload plaintext.

---

# 59. Content Class

```rust
pub enum ContentClass {
    Control,
    Text,
    Metadata,
    Thumbnail,
    Voice,
    Image,
    File,
    RealtimeAudio,
    RealtimeVideo,
    Emergency,
}
```

This helps policy without exposing private contents.

---

# 60. Size-Aware Routing

A path good for 1 KB may be poor for 1 GB.

Scoring must account for:

```text
setup cost
transfer time
energy
reliability
```

Estimate:

```text
completion_time ≈ setup + bytes / bandwidth
```

with uncertainty.

---

# 61. Deadline-Aware Routing

Realtime operations may have deadlines.

Example:

```text
video frame useful for 100 ms
```

If estimated arrival exceeds deadline:

```text
drop
```

rather than queue indefinitely.

---

# 62. Expiry-Aware Routing

Durable operations can have expiry.

Example:

```text
SOS expires after 24h
typing after 5s
```

Routing should stop retrying after expiry.

---

# 63. Queue Architecture

Use per-class queues:

```text
Critical
Control
Interactive
Bulk
Background
```

Each bounded.

Scheduler prevents bulk traffic from blocking interactive control.

---

# 64. Weighted Fair Scheduling

Example weights:

```text
Critical: 10
Control: 8
Interactive: 6
Normal: 4
Bulk: 2
Background: 1
```

Actual values require testing.

Emergency critical traffic can temporarily preempt.

---

# 65. Backpressure

If transport cannot keep up:

```text
producer
 ↓
bounded route queue
 ↓
backpressure
```

Feature receives:

```text
QueueFull
Deferred
```

or async wait.

Never buffer unbounded data.

---

# 66. Per-Transport Queues

Maintain separate bounded queues per transport/session.

This prevents:

```text
stalled Bluetooth
```

from blocking:

```text
healthy Iroh
```

---

# 67. Per-Peer Fairness

One peer transferring a huge file must not starve messages to others.

Scheduler should include peer fairness.

---

# 68. Per-Extension Fairness

Files should not starve:

```text
messaging
receipts
presence control
```

Use extension-aware quotas.

---

# 69. Route Planning for Messaging

Typical plan:

```text
1. existing authenticated direct route
2. local LAN
3. Iroh relay
4. nearby transport
5. DTN
```

Actual ordering depends on policy and measured health.

---

# 70. Route Planning for Files

Typical:

```text
1. high-bandwidth direct
2. LAN/Wi-Fi Direct
3. Iroh direct
4. relay
5. Bluetooth only if small/allowed
6. DTN if policy permits
```

---

# 71. Route Planning for Calls

Call path requirements:

```text
low RTT
low jitter
sufficient bandwidth
stable
realtime
```

Fallback:

```text
video
 ↓
lower bitrate
 ↓
audio
 ↓
call ends / switches to voice message
```

Routing supplies path quality signals to media adaptation.

---

# 72. Route Planning for Emergency

SOS may choose:

```text
Internet direct
+
nearby mesh copy
```

or:

```text
LAN
+
DTN
```

depending on connectivity.

Stable operation ID ensures deduplication.

---

# 73. Path Diversity

For redundant delivery, prefer independent failure domains.

Example:

```text
Wi-Fi Internet
+
BLE mesh
```

is more diverse than:

```text
two logical streams over same Wi-Fi path
```

Routing should understand shared-underlay relationships where possible.

---

# 74. Underlay Group

Add:

```rust
pub struct UnderlayId;
```

Candidates sharing same underlay can be grouped.

Example:

```text
Iroh direct over Wi-Fi
Iroh relay over Wi-Fi
```

share:

```text
same Wi-Fi radio/network
```

This matters for redundancy.

---

# 75. Multipath Chunk Scheduler

For future large-file multipath:

```text
file ranges
 ↓
path allocator
 ↓
per-path congestion/throughput
 ↓
dynamic chunk reassignment
```

Do not assign fixed 50/50 split.

Use measured throughput.

---

# 76. Path Collapse

If one multipath leg fails:

```text
remaining chunks
→ reassigned
```

Transfer should continue.

---

# 77. Duplicate Chunk Handling

Content-addressed or range-based file transfer makes redundant/multipath delivery idempotent.

Receiver verifies and discards duplicate chunks.

---

# 78. Realtime Multipath

For calls, future options include:

```text
redundant keyframes
audio on best path
video on high-bandwidth path
```

This is advanced and should be optional.

---

# 79. Congestion Signals

Transport adapters should report:

```text
send queue
RTT trend
loss
throughput
connection congestion
```

Routing should not implement transport congestion control itself.

It uses transport-level signals.

---

# 80. Route Stability Score

Derive from:

```text
connection lifetime
failure rate
path changes
packet loss
timeout history
```

A path with slightly worse RTT but much better stability may win.

---

# 81. Cold vs Warm Path

Path state:

```text
Warm:
existing authenticated connection

Cold:
requires discovery/connection
```

Warm path gets a setup-cost advantage.

---

# 82. Metered Networks

Platform reports:

```text
metered
unmetered
unknown
```

Application policy can:

```text
block bulk
allow control
allow emergency
```

Do not use one global allow/deny.

---

# 83. Roaming

Represent separately if platform provides it.

A user may allow cellular but forbid roaming bulk transfer.

---

# 84. Battery Cost

Transport energy cost can be coarse:

```rust
pub enum EnergyCost {
    VeryLow,
    Low,
    Medium,
    High,
    VeryHigh,
}
```

No need for fake precision initially.

---

# 85. Battery-Aware Inputs

Routing context may include:

```text
battery level class
charging state
battery saver
thermal state
foreground/background
```

Part 13 will deepen battery scheduling.

---

# 86. Background Restrictions

Mobile OS may disallow certain path setup in background.

Candidate path must include:

```text
currently usable
requires foreground
```

Routing should not repeatedly attempt impossible background operations.

---

# 87. Platform Policy Integration

Android/iOS adapters expose:

```text
network type
metered
roaming
power saver
background restrictions
radio availability
```

Rust routing policy remains authoritative.

---

# 88. Path Acquisition

Routing can request:

```text
bring up Wi-Fi Direct
start BLE scan
establish Iroh session
```

through transport manager.

But active acquisition must be policy-limited because it costs power/time.

---

# 89. Passive vs Active Candidates

```rust
pub enum CandidateState {
    Active,
    PassiveKnown,
    RequiresDiscovery,
    RequiresSetup,
}
```

Scoring can penalize setup.

---

# 90. Discovery Budget

Routing should not trigger unlimited scans.

Use:

```text
discovery budget
cooldown
priority-based escalation
```

Example:

```text
normal message:
no aggressive BLE scan

SOS:
aggressive discovery allowed
```

---

# 91. Route Escalation Ladder

A practical model:

```text
Stage 1:
use active connections

Stage 2:
use known endpoints

Stage 3:
perform lightweight discovery

Stage 4:
perform expensive proximity setup

Stage 5:
fall back to DTN
```

This controls resource cost.

---

# 92. Timeout by Stage

Each stage has bounded timeout.

Do not wait forever trying one path before considering alternatives.

---

# 93. Hedged Requests

For latency-sensitive small operations:

```text
start primary
after short delay if no progress
start fallback
```

First success wins.

Useful for:

```text
control
important text
```

Potentially expensive, so use selectively.

---

# 94. Deduplication Requirement

Hedged or redundant sends require stable:

```text
OperationId
MessageId
BundleId
```

Receiver must be idempotent.

---

# 95. Route Diagnostics

Expose user/developer-readable plan:

```text
Selected:
Iroh Direct

Reason:
Healthy existing session
RTT 42 ms
Unmetered Wi-Fi

Fallback:
Iroh Relay
```

Normal users do not need this.

Diagnostics do.

---

# 96. Path Visualization

Part 18 will expand this.

Routing should already preserve enough structured data to visualize:

```text
Alice Phone
  ↓ Wi-Fi
Iroh Direct
  ↓
Bob Laptop
```

or:

```text
Alice
 ↓ BLE
Relay Peer
 ↓ Internet
Bob
```

---

# 97. Route Decision Explainability

Each decision should include machine-readable reason codes:

```rust
pub enum RouteReason {
    ExistingHealthyConnection,
    LowestLatency,
    HighestBandwidth,
    PolicyPreferred,
    DirectPreferred,
    RelayFallback,
    EmergencyRedundancy,
    DtnOnlyAvailable,
}
```

This is excellent for debugging.

---

# 98. Metrics Collection

Useful routing metrics:

```text
route selection count
failover count
path switch count
direct vs relay
DTN fallback count
average route setup latency
queue delay
retry count
```

---

# 99. Privacy of Metrics

Do not export:

```text
peer identity
precise IP
contact graph
location
```

without explicit policy.

Local diagnostics can retain more detailed ephemeral information.

---

# 100. Routing State Store

Persist only useful durable routing hints:

```text
last successful transport
recent gateway
known endpoint freshness
```

Do not persist every packet metric indefinitely.

---

# 101. Startup Behavior

At startup:

```text
load route hints
 ↓
start core transport
 ↓
revalidate candidates
```

Never assume persisted route is still valid.

---

# 102. Suspend/Resume

On mobile resume:

```text
invalidate stale metrics
refresh platform network state
reassess active sessions
retry durable operations
```

Do not immediately launch all discovery mechanisms.

---

# 103. Process Death

Routing correctness must not depend on in-memory plans.

Durable operations are persisted by message/file/outbox layers.

After restart:

```text
reconstruct route
```

from current conditions.

---

# 104. Route Plan Lifetime

Plans are short-lived.

```rust
RoutePlan {
    created_at,
    valid_until,
}
```

Do not cache route decisions indefinitely.

---

# 105. Path Authorization

Transport availability is insufficient.

Candidate must pass:

```text
device active
identity trusted enough
operation authorized
extension supported
```

before selection.

---

# 106. Extension Capability Integration

Part 01 supplies extension negotiation.

Routing can eliminate a path/session where remote does not support required protocol extension.

Example:

```text
files/1 required
remote only supports messaging
→ path invalid for file operation
```

---

# 107. Device Capability Integration

Part 02 supplies device capabilities.

Example:

```text
call to Bob
phone supports video
headless relay does not
```

Routing should target appropriate device subset.

---

# 108. Policy Layering

Use layered policy:

```text
system hard safety policy
 ↓
application policy
 ↓
user preference
 ↓
operation requirements
 ↓
current network context
```

Hard safety constraints cannot be overridden by UI preference.

---

# 109. System Policy

Examples:

```text
never send unencrypted private message
never route to revoked device
never exceed hard size limit
```

---

# 110. Application Policy

ERP example:

```text
documents must not use unknown relay peers
```

Messenger:

```text
normal messages may use relay
```

Emergency app:

```text
SOS may use DTN
```

---

# 111. User Policy

Examples:

```text
no mobile data for files
prefer local connections
disable Bluetooth relay
battery saver
```

---

# 112. Operation Policy

Specific operation may require:

```text
deadline
minimum bandwidth
specific device
```

---

# 113. Policy Conflict

If constraints conflict:

```text
large file
+
no metered
+
only metered path exists
```

Result:

```text
DeferredByPolicy
```

not silent policy violation.

---

# 114. Policy Result Types

```rust
pub enum RouteDecisionResult {
    Routed(RoutePlan),
    Deferred(DeferredReason),
    Rejected(RejectReason),
    Unreachable,
}
```

This makes behavior explicit.

---

# 115. Deferred Reasons

```text
WaitingForUnmetered
WaitingForPeer
WaitingForWifi
BatteryPolicy
BackgroundRestriction
NoSuitablePathYet
```

Useful for UI.

---

# 116. UI-Friendly State

Routing core can expose neutral state:

```text
Sending
WaitingForConnection
WaitingForWiFi
CarriedByNearbyPeer
Delivered
```

Product UI maps it into wording.

Do not expose raw transport errors to normal users.

---

# 117. Route Policy Configuration

Use typed config:

```rust
RoutingConfig {
    direct_preference,
    relay_policy,
    bluetooth_policy,
    dtn_policy,
    multipath_policy,
    retry_policy,
    hysteresis,
}
```

Validate at startup.

---

# 118. No Global Singleton

Each `CommunicationRuntime` owns its routing engine.

This enables:

```text
multiple identities
tests
embedded runtimes
tenant isolation
```

---

# 119. Routing Engine API

```rust
pub trait RoutingEngine {
    async fn plan(
        &self,
        request: RouteRequest,
    ) -> Result<RouteDecision, RoutingError>;

    async fn report_result(
        &self,
        report: RouteResultReport,
    );
}
```

---

# 120. Transport Manager API

```rust
pub trait TransportManager {
    async fn candidates(
        &self,
        destination: &ResolvedDestination,
    ) -> Result<Vec<PathCandidate>, TransportError>;

    async fn acquire(
        &self,
        path: &PathCandidate,
    ) -> Result<TransportSession, TransportError>;
}
```

Routing does not own low-level sockets.

---

# 121. Feedback Loop

After each operation:

```text
transport result
 ↓
routing feedback
 ↓
metrics update
 ↓
health update
```

The engine learns local path quality.

---

# 122. Avoid ML Initially

Do not start with a machine-learning route selector.

Deterministic policy is:

- easier to test
- explainable
- safer
- easier to debug

ML can be explored later if measurements justify it.

---

# 123. Deterministic Scoring

Given same:

```text
policy
metrics
context
```

route decision should be reproducible.

This improves testing.

---

# 124. Simulated Routing Tests

Build fake paths:

```text
Path A:
10 ms
1 Mbps
metered

Path B:
50 ms
100 Mbps
unmetered
```

Expected:

```text
text → A or B depending policy
large file → B
```

---

# 125. Policy Property Tests

Examples:

```text
revoked path never selected
forbidden metered path never selected
realtime operation never uses DTN
expired operation never routed
```

---

# 126. Chaos Tests

Simulate:

```text
Wi-Fi flaps
relay latency spikes
BLE disappears
mobile data activates
battery saver enables
```

Assert:

```text
no route storm
no infinite retry loop
bounded queues
```

---

# 127. Failover Test

```text
primary healthy
 ↓
mid-transfer failure
 ↓
fallback acquired
 ↓
operation resumes if semantics allow
```

Message retry and file resume differ.

Routing only coordinates path change.

Feature layer owns semantic resume.

---

# 128. File Resume Integration

Routing says:

```text
new path available
```

File engine says:

```text
resume missing ranges
```

Do not make routing understand file chunk state.

---

# 129. Messaging Retry Integration

Routing says:

```text
path failed
```

Messaging outbox decides:

```text
retry same message ID
```

Routing does not create duplicate message semantics.

---

# 130. Call Path Change Integration

Routing reports:

```text
new path
quality update
```

Call/media engine handles:

```text
rebind
renegotiate
adapt bitrate
```

---

# 131. Security Event Integration

If routing gets:

```text
authentication failure
revoked device
```

it must:

```text
remove candidate
invalidate route cache
emit security-relevant event
```

Do not treat as ordinary transient failure.

---

# 132. Blacklisting

Temporary path blacklist can be used for repeated failure.

```rust
PathPenalty {
    until,
    reason,
}
```

Do not permanently blacklist transport based on one transient error.

---

# 133. Peer Abuse

If peer is abusive:

```text
rate limit
quarantine
block
```

This is separate from path quality.

A high-bandwidth malicious peer must not win scoring.

---

# 134. Local-Only Mode

Application can enforce:

```text
no Internet
```

Then routing considers only:

```text
LAN
Wi-Fi Direct/Aware
Bluetooth
mesh
DTN
```

Useful for private/offline deployments.

---

# 135. Internet-Only Mode

Some enterprise policy may forbid proximity.

Then:

```text
Iroh direct/relay
```

only.

---

# 136. Nearby-Only Mode

Useful for disaster/private transfer:

```text
LAN
Wi-Fi Direct
Wi-Fi Aware
Bluetooth
mesh
```

No Internet paths.

---

# 137. Route Scope

```rust
pub enum RouteScope {
    Any,
    InternetOnly,
    LocalOnly,
    NearbyOnly,
}
```

Applications can request a scope.

---

# 138. Emergency Override

Emergency policy may override:

```text
user preference to avoid relay
```

only if the user explicitly enabled such emergency behavior.

Do not silently violate user privacy choices.

---

# 139. User Consent

Policies involving:

```text
cellular charges
relay usage
relay storage
nearby forwarding
```

should have explicit product-level consent settings.

Routing consumes the resulting policy.

---

# 140. Bandwidth Reservation

Future call + file coexistence:

```text
reserve bandwidth for realtime audio/video
```

Bulk transfer yields.

Routing/scheduler can coordinate class-based quotas.

---

# 141. Traffic Shaping

Per-class rate caps:

```text
Bulk max 5 Mbps while call active
Background max 1 Mbps
```

This avoids call degradation.

---

# 142. Connection Admission

Limit:

```text
max simultaneous peer sessions
max expensive radio sessions
```

Routing may reject or defer low-priority new connections.

---

# 143. Thermal Awareness

On mobile/desktop:

```text
thermal pressure
```

may reduce:

```text
multipath
Wi-Fi Direct setup
background bulk
```

Media subsystem also adapts separately.

---

# 144. Memory Pressure

Under memory pressure:

```text
reduce route queues
pause bulk acquisition
drop stale realtime packets
```

Durable operations remain persisted.

---

# 145. Transport Adapter Contract

Each adapter should report:

```text
availability
capabilities
health
metrics
setup cost
current session state
```

and support:

```text
connect/acquire
close
send/stream
```

---

# 146. Iroh Adapter

Reports:

```text
direct vs relay path
RTT
session availability
address info
```

but routing API should not expose Iroh-specific types upward.

---

# 147. LAN Adapter

Reports:

```text
local reachability
interface
estimated throughput class
```

Identity/authentication still applies.

---

# 148. Bluetooth Adapter

Reports:

```text
BLE vs Classic
proximity
estimated bandwidth
paired/available state
```

Do not use Bluetooth MAC as identity.

---

# 149. Wi-Fi Direct/Aware Adapter

Reports:

```text
support
current group/session
setup cost
available bandwidth class
background restrictions
```

---

# 150. DTN Adapter

General routing sees:

```text
store-and-forward path available
```

with:

```text
uncertain latency
delivery probability
replication policy
```

Detailed peer encounter logic remains in Part 06.

---

# 151. Route Probability

For DTN/mesh, exact RTT may be unknown.

Use:

```text
delivery likelihood
expected delay class
```

instead of fake precise latency.

---

# 152. Metric Types Must Match Reality

Do not force:

```text
Bluetooth RSSI
Iroh RTT
DTN encounter probability
```

into one misleading number.

Use typed metric categories, then derive normalized scoring.

---

# 153. Scoring Normalization

A scorer can normalize:

```text
LatencyScore
BandwidthScore
EnergyScore
ReliabilityScore
CostScore
```

then combine with policy weights.

---

# 154. Policy Weight Example

```rust
RouteWeights {
    latency: 0.30,
    bandwidth: 0.20,
    reliability: 0.25,
    energy: 0.15,
    cost: 0.10,
}
```

Do not expose floating-point tuning directly to ordinary users.

Profiles configure these internally.

---

# 155. Integer Score Option

For deterministic portability, use fixed-point/integer score.

Example:

```text
0..10_000
```

Avoid floating point if reproducibility matters.

---

# 156. Route Decision Logging

Log:

```text
operation id
selected path
reason codes
candidate count
policy profile
```

not payload content.

---

# 157. Developer Diagnostics

Advanced screen can show:

```text
Destination: Bob Phone
Selected: Iroh Direct
Score: 8240
Fallback: Iroh Relay
Reason: existing session + low RTT
```

Useful for field testing.

---

# 158. Route History

Keep bounded recent history:

```text
last N decisions
```

for diagnostics.

Do not retain indefinitely.

---

# 159. Telemetry Export

If enabled:

```text
aggregate route success
direct/relay ratio
failover rate
```

Redact peer identity.

---

# 160. API Example: Text Message

```rust
let req = RouteRequest::for_device(bob_phone)
    .class(DeliveryClass::Interactive)
    .priority(Priority::Normal)
    .estimated_size(512)
    .allow_dtn(true);

let decision = router.plan(req).await?;
```

---

# 161. API Example: Large File

```rust
let req = RouteRequest::for_device(bob_laptop)
    .class(DeliveryClass::Bulk)
    .priority(Priority::Low)
    .estimated_size(file_size)
    .allow_metered(false)
    .allow_multipath(true);
```

---

# 162. API Example: SOS

```rust
let req = RouteRequest::for_account(target)
    .class(DeliveryClass::DelayTolerant)
    .priority(Priority::Critical)
    .allow_dtn(true)
    .allow_redundancy(true)
    .expiry(sos_expiry);
```

---

# 163. API Example: Video Call

```rust
let req = RouteRequest::for_device(peer)
    .class(DeliveryClass::Realtime)
    .priority(Priority::High)
    .min_bandwidth(required_video_rate)
    .max_latency(Duration::from_millis(200))
    .allow_dtn(false);
```

---

# 164. Route Result Report

Transport/session reports:

```rust
pub struct RouteResultReport {
    pub path_id: PathId,
    pub operation_id: OperationId,
    pub outcome: RouteOutcome,
    pub observed_metrics: ObservedMetrics,
}
```

This closes feedback loop.

---

# 165. Route Outcome

```text
Success
Timeout
ConnectionFailed
AuthenticationFailed
RemoteRejected
PolicyBlocked
Partial
Cancelled
```

---

# 166. Partial Outcome

Useful for:

```text
file transfer
```

Routing can report:

```text
bytes transferred before failure
```

Feature layer uses it for resume.

---

# 167. Cancellation

Route acquisition and send operations must support cancellation.

Example:

```text
user cancels file
```

Stop:

```text
path acquisition
queued operation
retry timer
```

where safe.

---

# 168. Graceful Path Switch

For long-lived sessions:

```text
prepare new path
 ↓
authenticate
 ↓
transfer state
 ↓
switch
 ↓
close old path
```

Avoid dropping old route before new one is ready if seamless handoff is possible.

---

# 169. Make-Before-Break

Use when:

```text
call
large transfer
```

and policy permits.

This reduces interruption.

---

# 170. Break-Before-Make

Use when:

```text
resource constrained
security requires old path termination
```

Make strategy explicit.

---

# 171. Multi-Device Route Aggregation

Account target may use:

```text
parallel routes to multiple devices
```

with per-device plan.

Do not flatten all devices into one route score.

---

# 172. Device Preference

Application may mark:

```text
primary device
preferred file device
call-capable device
```

Routing uses as policy input, not immutable identity.

---

# 173. Group Routing

Group messaging does not mean:

```text
one path to group
```

It may:

```text
fan out to devices
or use group dissemination protocol
```

Routing provides per-destination transport choices.

---

# 174. Broadcast Routing

Emergency/local broadcast may use:

```text
local dissemination
mesh
DTN
```

with separate duplication controls.

Do not reuse ordinary unicast blindly.

---

# 175. Route Constraints by Content Sensitivity

High-sensitivity content may forbid:

```text
untrusted store-and-forward
```

even if encrypted, depending on application policy.

Represent:

```text
forwarding_allowed
relay_allowed
```

explicitly.

---

# 176. Storage Cost

DTN route has storage cost.

Route policy can consider:

```text
relay storage quota
local storage pressure
```

before accepting large bundles.

---

# 177. Route Planning Under Storage Pressure

If:

```text
DTN store nearly full
```

prefer:

```text
direct path
```

or reject/defer low-priority bulk traffic.

---

# 178. Emergency Storage Override

Critical SOS may evict:

```text
expired
low-priority
bulk
```

DTN items according to Part 17/06 policy.

General routing marks priority.

---

# 179. Route Policy Persistence

Persist user/application settings.

Do not persist transient:

```text
current best score
RTT samples
```

as authoritative policy.

---

# 180. Dynamic Policy Update

Policy can change at runtime:

```text
user enables battery saver
emergency mode starts
call begins
```

Routing re-evaluates relevant active operations.

---

# 181. Call-Induced Policy Change

During call:

```text
bulk file traffic throttled
realtime priority increased
path switching hysteresis increased
```

After call:

```text
normal policy restored
```

---

# 182. Emergency-Induced Policy Change

Emergency mode may:

```text
enable proximity
allow DTN
increase critical queue weight
```

while still honoring user opt-in and safety constraints.

---

# 183. Testing Matrix

Test combinations:

```text
direct + relay
LAN + Internet
Wi-Fi + BLE
BLE only
DTN only
metered only
battery saver
background restricted
```

for each delivery class.

---

# 184. Route Selection Golden Tests

Example expected cases:

```text
Existing direct healthy + text
→ direct

Direct degraded + relay stable + text
→ relay

Large file + unmetered LAN
→ LAN

Typing + only DTN
→ reject/drop

SOS + Internet unavailable + BLE mesh
→ DTN/mesh
```

---

# 185. Property Tests

Invariants:

```text
revoked device never selected
forbidden transport never selected
expired operation never retried
realtime never routed through DTN
hard minimum bandwidth respected
```

---

# 186. Fuzzing

Fuzz:

```text
route request parser if network-exposed
policy config parsing
metrics updates
candidate sets
state transitions
```

Routing is mostly internal, but malformed transport telemetry should not panic.

---

# 187. Benchmarking

Benchmark:

```text
candidate scoring
route planning
cache lookup
path update
large peer sets
```

Target route decision should be very fast relative to network operations.

---

# 188. Scalability

Do not linearly rescore thousands of irrelevant candidates for every packet.

Cache per-peer candidates.

Route at:

```text
operation/session level
```

not per small network packet.

---

# 189. Call Routing Frequency

For calls, quality metrics update frequently.

Do not rerun full planner every frame.

Use:

```text
path monitor
```

and trigger reevaluation only when thresholds cross.

---

# 190. File Routing Frequency

For files:

```text
reevaluate on:
path failure
significant quality change
new superior high-bandwidth path
policy change
```

not per chunk.

---

# 191. Message Routing Frequency

Each message can reuse:

```text
healthy session route
```

until invalidated.

No need for expensive full scoring every time.

---

# 192. Architecture Modules

Recommended crate:

```text
comm-routing/
├── src/
│   ├── lib.rs
│   ├── request.rs
│   ├── requirements.rs
│   ├── candidate.rs
│   ├── metrics.rs
│   ├── scorer.rs
│   ├── policy.rs
│   ├── planner.rs
│   ├── health.rs
│   ├── cache.rs
│   ├── scheduler.rs
│   ├── retry.rs
│   ├── diagnostics.rs
│   └── error.rs
```

---

# 193. Related Crates

```text
comm-transport
comm-discovery
comm-session
comm-identity
comm-dtn
comm-types
```

The routing crate should not import:

```text
Dioxus
Kotlin
Android APIs
messenger UI
```

---

# 194. Error Types

```rust
pub enum RoutingError {
    NoCandidate,
    PolicyConflict,
    IdentityResolution,
    TransportUnavailable,
    ResourceLimit,
    Cancelled,
    Internal,
}
```

---

# 195. No `anyhow` in Public Routing API

Use typed errors.

`anyhow` remains acceptable at top-level application bootstrap.

---

# 196. Initial Production Scope

Implement first:

```text
candidate collection
hard constraints
weighted scoring
direct/relay/LAN/Bluetooth/DTN categories
failover
route cache
hysteresis
retry policy
bounded queues
diagnostics
```

Defer:

```text
true multipath aggregation
advanced redundancy optimization
predictive route learning
ML
```

---

# 197. Implementation Phases

## Phase 1 — Types and policy

Implement:

```text
DeliveryRequirements
PathCandidate
PathMetrics
RoutingPolicy
RoutePlan
```

## Phase 2 — Candidate collection

Integrate:

```text
Iroh
LAN
Bluetooth
Wi-Fi
```

## Phase 3 — Scoring

Implement:

```text
hard constraints
weighted scores
existing-path preference
```

## Phase 4 — Failover

Implement:

```text
fallback plan
retry policy
hysteresis
```

## Phase 5 — DTN

Integrate:

```text
delay-tolerant route option
```

## Phase 6 — Resource scheduling

Implement:

```text
priority queues
fairness
backpressure
```

## Phase 7 — Diagnostics/testing

Implement:

```text
reason codes
route history
simulation
property tests
benchmarks
```

---

# 198. Definition of Done

Part 03 is complete when:

- applications submit delivery intent, not transport names
- account destinations resolve to device paths
- all path candidates are normalized
- hard constraints eliminate invalid paths
- remaining paths are scored deterministically
- existing healthy paths are reused
- direct/relay/LAN/Bluetooth/DTN can coexist
- failover works
- path switching uses hysteresis
- policy conflicts defer/reject explicitly
- routing honors metered/battery/background policy
- queues are bounded
- bulk traffic cannot starve control traffic
- DTN is supported for eligible durable traffic
- realtime traffic never routes through DTN
- routing provides reason codes/diagnostics
- route state survives process restart only as hints
- security/revocation always outrank route quality
- routing is reusable by messaging, files, calls, emergency, ERP
- simulated path tests cover failures and transitions

---

# 199. Relationship to Other Parts

Part 03 depends on:

```text
01 — Protocol Extension System
02 — Multi-Device Identity
```

It feeds directly into:

```text
04 — Offline Event Log
05 — Robust File / Blob Subsystem
06 — DTN / Store-Carry-Forward
07 — Capability Negotiation Expansion
08 — Resource Limits & Backpressure
11 — Relay / Self-Hosted Infrastructure
12 — Multipath Networking
13 — Battery-Aware Scheduling
14 — Proximity Abstraction
17 — Emergency Priority Architecture
18 — Network Diagnostics & Path Visualization
```

---

# 200. Final Principle

The transport/routing policy engine should make this application-level request possible:

```text
"Deliver this securely to Bob.
It is a normal interactive message.
It may use DTN.
It should avoid metered data if possible."
```

without the messaging layer knowing whether delivery ultimately occurs through:

```text
Iroh direct
Iroh relay
LAN
Wi-Fi Direct
Bluetooth
mesh
DTN
```

That separation is what makes the communication platform adaptable, reusable, power-aware, resilient, and capable of surviving both ordinary network changes and severe infrastructure failure.
