# Core System Architecture Part 37 — Cover Traffic, Traffic Shaping, Timing Obfuscation & Loop Traffic Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 37  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:**  
- Part 34 — Mixnet, Loopix, Sphinx, Nym & High-Anonymity Transport Architecture  
- Part 35 — Anonymous Mailbox, Offline Receiving & Reply Capability Architecture  
- Part 36 — Sphinx Packet, Cell, Fragmentation & Anonymous Message Framing Architecture  

**Primary purpose:** define the complete traffic-analysis-resistance subsystem for SIAR's high-anonymity transport plane, including cover traffic, Loopix-style loop traffic, randomized send scheduling, timing obfuscation, batching, traffic-volume normalization, idle traffic, queueing, congestion handling, resource budgets, provider integration, observability, simulation, and production safety.

---

# 1. Purpose

End-to-end encryption hides content.

Mix routing hides route relationships.

Fixed-size cells hide payload length.

But an observer can still learn from:

```text
when packets are sent
how often packets are sent
how long sessions remain active
whether traffic starts immediately after user action
whether packet bursts correlate between sender and receiver
whether traffic stops when user goes offline
```

Part 37 addresses that problem.

The governing principle is:

> **High-anonymity transport must shape observable traffic independently from user activity strongly enough that real messages do not map trivially to network timing.**

---

# 2. Architectural Position

```text
Application / E2EE
       │
       ▼
Anonymous Framing
       │
       ▼
Traffic Shaping Scheduler
       │
       ├── real cells
       ├── cover cells
       ├── loop cells
       ├── batching
       └── randomized delay
       │
       ▼
Anonymous Provider
       │
       ▼
Mixnet
```

---

# 3. Threats Addressed

This subsystem primarily reduces exposure to:

```text
timing correlation
volume correlation
burst correlation
online/offline correlation
send-action correlation
recipient-receive correlation
long-term traffic-pattern analysis
```

---

# 4. Threats Not Fully Solved

Traffic shaping does not eliminate:

```text
endpoint compromise
malicious plugins
OS telemetry leakage
global long-term intersection attacks
provider compromise
application-level metadata leaks
```

It is one layer of defense.

---

# 5. Cover Traffic

Cover traffic consists of packets that:

```text
look like real traffic externally
carry no user payload
exist only to obscure traffic patterns
```

---

# 6. Loop Traffic

Loop traffic is a special class of cover traffic that:

```text
leaves the client
travels through the mix network
returns to the same client
```

It can help:

```text
measure network health
maintain background traffic
mask real sends
```

---

# 7. Traffic Classes

```rust
pub enum TrafficClass {
    Real,
    Cover,
    Loop,
    Control,
}
```

Externally, these should be as indistinguishable as the provider permits.

---

# 8. Scheduler Architecture

```text
Real Message Queue
      │
Cover Traffic Generator
      │
Loop Traffic Generator
      │
      ▼
Traffic Shaping Scheduler
      │
      ▼
Provider Send Queue
```

---

# 9. Scheduler Goal

The scheduler decides:

```text
when a cell is emitted
which class is emitted
which anonymity profile applies
```

---

# 10. Scheduling Modes

```rust
pub enum TrafficShapingMode {
    Disabled,
    Adaptive,
    Standard,
    Maximum,
}
```

---

# 11. Disabled

Only acceptable for:

```text
development
testing
non-anonymous modes
```

Not for Maximum Anonymity.

---

# 12. Adaptive

Balances:

```text
privacy
battery
bandwidth
foreground state
network condition
```

---

# 13. Standard

Maintains moderate continuous cover and randomized timing.

---

# 14. Maximum

Prioritizes traffic-analysis resistance over battery/bandwidth efficiency.

---

# 15. Emission Process

Preferred model:

```text
schedule emission opportunities
then choose real vs cover
```

rather than:

```text
user sends message
→ immediately send packet
```

---

# 16. Emission Slot

```rust
pub struct EmissionSlot {
    pub scheduled_at: Instant,
    pub budget_class: TrafficBudgetClass,
}
```

---

# 17. Randomized Timing

Use provider-appropriate stochastic scheduling.

A Loopix-inspired design can use:

```text
Poisson process
```

for transmission opportunities.

---

# 18. Exponential Inter-Arrival

Conceptually:

```text
delay ~ Exp(lambda)
```

where:

```text
lambda = average event rate
```

---

# 19. Do Not Roll Custom Crypto/Statistics Casually

Use established, reviewed implementations where provider already provides traffic shaping.

SIAR policy should express:

```text
privacy level
rate class
resource budget
```

rather than hardcoding obscure mathematical tuning throughout app code.

---

# 20. Traffic Rate Classes

```rust
pub enum TrafficRateClass {
    Low,
    Medium,
    High,
    MaximumPrivacy,
}
```

---

# 21. Profile Mapping

Example:

```text
Anonymous
→ Medium

Maximum Anonymity
→ High / MaximumPrivacy
```

---

# 22. Idle Cover Traffic

A key property of strong traffic shaping:

```text
traffic may continue even when user sends nothing
```

This reduces:

```text
activity/no-activity correlation
```

---

# 23. Idle Traffic Tradeoff

Consumes:

```text
battery
bandwidth
provider capacity
```

Therefore it must be explicit.

---

# 24. Real-vs-Cover Substitution

At each scheduled send opportunity:

```text
if real cell waiting
    send real cell
else
    send cover/loop cell
```

---

# 25. Constant Envelope

A stronger profile can maintain roughly constant:

```text
packet emission envelope
```

within a bounded time window.

---

# 26. Constant-Rate Mode

Possible future mode:

```rust
pub enum EmissionPattern {
    Poisson,
    ConstantRate,
    Batched,
    ProviderNative,
}
```

---

# 27. Provider-Native Preferred

If Nym/provider already implements appropriate traffic generation:

```text
reuse provider semantics
```

rather than duplicating at SIAR layer.

---

# 28. SIAR Scheduling Role

SIAR still controls:

```text
whether to enqueue real message
whether strict anonymity is active
batching eligibility
privacy budget
background behavior
```

---

# 29. Loop Packet Generation

Loop packets should be:

```text
same size
same outer framing
same scheduling class
```

as ordinary anonymous packets where provider permits.

---

# 30. Loop Destination

Returns to sender through anonymous route.

---

# 31. Loop Purpose

Can provide:

```text
cover
latency observation
path-health estimation
```

---

# 32. Loop Measurement Privacy

Do not log per-hop path identity.

Only aggregate:

```text
round-trip class
success/failure
```

---

# 33. Loop Frequency

Bounded and policy-controlled.

---

# 34. Cover Traffic Generator

```rust
pub trait CoverTrafficGenerator {
    fn next_cover_cell(
        &mut self,
        profile: &TrafficProfile,
    ) -> Result<AnonymousCell, CoverTrafficError>;
}
```

---

# 35. Loop Traffic Generator

```rust
pub trait LoopTrafficGenerator {
    fn next_loop_cell(
        &mut self,
        profile: &TrafficProfile,
    ) -> Result<AnonymousCell, LoopTrafficError>;
}
```

---

# 36. Traffic Profile

```rust
pub struct TrafficProfile {
    pub mode: TrafficShapingMode,
    pub rate: TrafficRateClass,
    pub emission_pattern: EmissionPattern,
    pub batch_policy: BatchPolicy,
    pub power_policy: AnonymousPowerPolicy,
}
```

---

# 37. Batch Policy

```rust
pub enum BatchPolicy {
    None,
    ShortWindow,
    PrivacyOptimized,
}
```

---

# 38. Batching Goal

Delay small messages slightly so multiple sends can be scheduled less directly from user action.

---

# 39. Short Window

Example:

```text
50–500 ms
```

depending profile.

---

# 40. Privacy-Optimized Window

Could be longer.

Use only when acceptable for UX.

---

# 41. Do Not Batch Realtime-Critical Control Blindly

Some control messages may have maximum delay bounds.

---

# 42. Delay Budget

```rust
pub struct DelayBudget {
    pub target: Duration,
    pub max: Duration,
}
```

---

# 43. Latency Classes

```rust
pub enum AnonymousLatencyClass {
    Interactive,
    Balanced,
    Privacy,
    MaximumPrivacy,
}
```

---

# 44. Interactive

For anonymous text that should still feel reasonably responsive.

---

# 45. Maximum Privacy

Accepts larger delay to reduce timing correlation.

---

# 46. Traffic Shaping Queue

Separate logical queues:

```text
High Priority Control
Real Message
Attachment
Cover
Loop
```

---

# 47. Priority Does Not Mean Immediate Send

Hard rule.

Priority only influences which cell fills the next allowed emission slot.

---

# 48. Queue Types

```rust
pub struct TrafficQueues {
    pub control: VecDeque<AnonymousCell>,
    pub real: VecDeque<AnonymousCell>,
    pub attachment: VecDeque<AnonymousCell>,
}
```

Cover and loop may be generated on demand.

---

# 49. Scheduling Selection

Conceptually:

```text
if control eligible
    send control
else if real eligible
    send real
else if attachment eligible
    send attachment
else
    send cover/loop
```

within shaping constraints.

---

# 50. Control Starvation Protection

Critical protocol control cannot starve indefinitely.

---

# 51. Attachment Fairness

Large attachment transfer must not monopolize every slot.

---

# 52. Weighted Fairness

Potential:

```rust
pub struct TrafficWeights {
    pub control: u32,
    pub message: u32,
    pub attachment: u32,
}
```

---

# 53. Cover Is Filler

Cover occupies otherwise unused scheduled opportunities.

---

# 54. Loop Ratio

Some cover opportunities become:

```text
loop traffic
```

according to profile.

---

# 55. Timing Obfuscation

User action should not deterministically create immediate outbound packet.

---

# 56. Send-Time Decoupling

```text
user presses Send
→ message persisted immediately
→ cell enters queue
→ scheduler emits according to anonymity profile
```

---

# 57. UI Semantics

User sees:

```text
Queued anonymously
Sending anonymously
Delivered
```

---

# 58. User Expectation

Maximum Anonymity should explain:

```text
messages may be intentionally delayed
```

---

# 59. Burst Flattening

If user sends ten messages rapidly:

```text
do not necessarily emit ten immediate bursts
```

---

# 60. Burst Queue

Scheduler smooths burst into emission schedule.

---

# 61. Receive-Side Timing

Mailbox fetch behavior should also avoid exact user-trigger correlation.

---

# 62. Fetch Shaping

Possible:

```text
continuous receive
randomized fetch
batch fetch
cover fetch
```

---

# 63. Send/Receive Symmetry

Strong anonymity requires both directions to avoid obvious activity correlation.

---

# 64. Traffic Volume Normalization

Fixed-size cells normalize packet size.

Traffic shaping must also normalize:

```text
packet count over time
```

to some degree.

---

# 65. Volume Bucketing

Potential privacy window:

```text
per minute
per 5 minutes
```

---

# 66. Do Not Reveal Exact User Message Volume

Where possible.

---

# 67. High-Volume User

If real traffic exceeds cover schedule:

```text
queue grows
or schedule rate adapts
```

---

# 68. Adaptive Rate Increase

Can increase emission rate gradually.

---

# 69. Sudden Rate Increase Leak

Avoid instantly doubling rate at exact moment of user burst.

---

# 70. Rate Ramp

Use:

```text
smoothed rate transition
```

---

# 71. Rate Hysteresis

Avoid rapid up/down oscillation.

---

# 72. Traffic Budget

```rust
pub struct TrafficBudget {
    pub bytes_per_hour: u64,
    pub cells_per_minute: u32,
    pub burst_allowance: u32,
}
```

---

# 73. Budget Types

```rust
pub enum TrafficBudgetClass {
    BatterySaving,
    Balanced,
    PrivacyFocused,
    Maximum,
}
```

---

# 74. Maximum Mode

May treat budget as soft unless OS/network restrictions intervene.

---

# 75. Battery Policy

```text
Adaptive
PreserveBattery
MaintainAnonymity
```

from Part 34.

---

# 76. Battery-Aware Behavior

PreserveBattery can reduce:

```text
cover rate
loop rate
poll frequency
```

but must not silently switch transport.

---

# 77. MaintainAnonymity

Keeps configured shaping as long as platform allows.

---

# 78. Android Background Constraints

Potential issues:

```text
Doze
battery saver
background limits
process suspension
network restrictions
```

---

# 79. Android Strategy

Possible:

```text
foreground service for explicitly enabled always-on anonymity
WorkManager-style bounded maintenance
provider-native background runtime where allowed
```

---

# 80. No Fake Guarantee

If OS suspends client:

```text
Maximum Anonymity degraded
```

must be surfaced.

---

# 81. Background Degradation

Example:

```text
Anonymous background traffic is limited by Android battery restrictions.
```

---

# 82. Desktop Daemon

Desktop can maintain:

```text
continuous shaping
mailbox receive
cover traffic
```

in background daemon.

---

# 83. Sleep/Suspend

On suspend:

```text
traffic stops
```

which itself reveals offline state.

---

# 84. Resume Behavior

Do not send giant burst immediately.

Use:

```text
warm-up/ramp
```

---

# 85. Warm-Up Phase

```rust
pub enum ShapingRuntimeState {
    Starting,
    WarmingUp,
    Steady,
    Degraded,
    Suspended,
}
```

---

# 86. Warm-Up Goal

Avoid:

```text
resume
→ instant traffic spike
```

---

# 87. Cool-Down

Likewise, user closing UI should not always immediately stop background anonymity traffic if policy supports persistent runtime.

---

# 88. Session Independence

Cover traffic lifetime should not map exactly to app-window lifetime.

---

# 89. Network Switching

Wi-Fi → mobile data:

```text
scheduler continues with policy-adjusted budget
```

---

# 90. Metered Network

Adaptive mode may reduce cover rate.

---

# 91. Maximum Mode on Metered Network

User should be warned:

```text
high data usage
```

but privacy policy remains.

---

# 92. Data Saver

If OS blocks background transfer:

```text
degraded
```

---

# 93. Congestion

Provider can become congested.

---

# 94. Congestion Signal

```rust
pub struct ProviderCongestion {
    pub queue_depth: u32,
    pub send_latency: Duration,
    pub state: CongestionState,
}
```

---

# 95. Congestion State

```rust
pub enum CongestionState {
    Normal,
    Elevated,
    Severe,
}
```

---

# 96. Congestion Response

Options:

```text
slow emission
reduce cover
preserve real backlog
drop only disposable cover
```

---

# 97. Never Drop Durable Real Message Silently

Hard rule.

---

# 98. Cover Drop

Safe under congestion if profile permits.

---

# 99. Maximum Mode Congestion

May retain more cover but must still avoid destabilizing provider.

---

# 100. Backpressure

If provider send queue full:

```text
scheduler stops producing
```

---

# 101. Bounded Queue

Required.

---

# 102. Queue Limits

```rust
pub struct TrafficQueueLimits {
    pub max_real_cells: u32,
    pub max_attachment_cells: u32,
    pub max_provider_pending: u32,
}
```

---

# 103. Cover Queue

Do not pre-buffer huge cover queue.

Generate on demand.

---

# 104. Queue Overflow

Real data:

```text
backpressure sender/framer
```

---

# 105. Attachment Throttling

Large attachments yield to messages.

---

# 106. Message Latency Bound

Interactive profile may reserve enough real slots to avoid pathological delays.

---

# 107. Privacy vs Latency

Explicit policy tradeoff.

---

# 108. Traffic Shaping Policy Resolver

```rust
pub trait TrafficShapingPolicyResolver {
    fn resolve(
        &self,
        privacy: PrivacyRoutingMode,
        power: AnonymousPowerPolicy,
        network: NetworkCostClass,
        foreground: bool,
    ) -> TrafficProfile;
}
```

---

# 109. Hard Constraint

`MaximumAnonymity` must never resolve to:

```text
Disabled
```

unless subsystem is unavailable, in which case transport becomes unavailable/degraded rather than weaker.

---

# 110. Provider Integration

If provider already performs:

```text
cover traffic
loop traffic
delay
```

SIAR should not double-generate blindly.

---

# 111. Capability Advertisement

```rust
pub struct TrafficShapingCapabilities {
    pub provider_cover: bool,
    pub provider_loop: bool,
    pub provider_delay: bool,
    pub custom_rate_control: bool,
}
```

---

# 112. Effective Ownership

```text
provider-native
or
SIAR-managed
```

for each mechanism.

---

# 113. Double-Shaping Risk

Two independent shaping layers can:

```text
increase latency
waste bandwidth
distort intended distribution
```

---

# 114. Provider Adapter Contract

```rust
pub trait AnonymousTrafficAdapter {
    fn capabilities(&self) -> TrafficShapingCapabilities;

    async fn submit(
        &self,
        cell: AnonymousCell,
        class: TrafficClass,
    ) -> Result<(), AnonymousTransportError>;
}
```

---

# 115. Nym Integration

Nym adapter should expose what Nym already handles and only let SIAR supplement missing policy-level behavior.

---

# 116. Native Mixnet Future

SIAR-native provider can implement full scheduler directly.

---

# 117. Loopix-Inspired Scheduler

Future native mode:

```text
Poisson send opportunities
cover traffic
loop traffic
provider mailbox traffic
randomized delays
```

---

# 118. Client Send Process

Conceptual:

```text
while running:
    wait next emission slot
    choose eligible real cell
    else generate cover/loop
    submit
```

---

# 119. Receive Process

Conceptual:

```text
receive provider packets
→ process
→ schedule mailbox/loop actions
→ avoid immediate correlated response where policy allows
```

---

# 120. Response Timing

Replies should not always immediately follow received message.

---

# 121. Reply Delay

Can use:

```text
application UX delay budget
+
traffic scheduler
```

---

# 122. Human Typing Still Leaks Timing

Even with mixnet, if send follows immediately after typing stops, some correlation may remain.

Maximum mode may introduce extra random delay.

---

# 123. Typing Indicator

Disabled by default in Maximum Anonymity.

---

# 124. Read Receipts

Disabled or delayed.

---

# 125. Delivery Receipts

Can be batched.

---

# 126. Presence

Off/coarse.

---

# 127. Traffic-Shaping Profile Includes Metadata Policy

Possible:

```rust
pub struct MaximumAnonymityBehavior {
    pub typing: bool,
    pub read_receipts: bool,
    pub presence: bool,
    pub external_fetches: bool,
}
```

---

# 128. External Fetch Suppression

No automatic:

```text
link preview
avatar fetch
plugin HTTP request
```

during strict profile unless separately approved.

---

# 129. Plugins

Network plugins can create timing fingerprints.

---

# 130. Plugin Policy

Maximum mode may:

```text
suspend network-capable plugins
```

---

# 131. Background Plugin Tasks

Also restricted.

---

# 132. Diagnostics

Normal:

```text
Anonymous traffic protection: Active
```

---

# 133. Advanced Diagnostics

Could show:

```text
Traffic mode
Rate class
Cover active
Loop active
Queue depth
Provider congestion
```

---

# 134. Do Not Show Exact Schedule

Avoid exposing:

```text
next emission time
exact lambda
exact loop route
```

to ordinary UI.

---

# 135. Logs

Allowed aggregate:

```text
scheduler started
profile changed
provider congested
```

---

# 136. Forbidden Logs

Do not log:

```text
real vs cover timestamps per peer
exact send schedule
frame IDs tied to contacts
loop route
```

---

# 137. Metrics

Privacy-safe aggregates:

```text
cover bytes
real bytes
loop success rate
queue depth bucket
latency bucket
provider congestion
```

---

# 138. Avoid High-Cardinality Labels

No:

```text
ConversationId
AccountId
MailboxId
FrameId
```

---

# 139. Ratio Metrics

Potential:

```text
cover-to-real ratio
```

only aggregated.

---

# 140. Exact Ratio Privacy

Do not expose remotely if it reveals usage patterns.

---

# 141. Local Metrics Preferred

Maximum mode:

```text
local-only metrics
```

---

# 142. Support Bundle

Include only:

```text
traffic shaping mode
provider capability flags
aggregate queue health
```

---

# 143. User Settings

Suggested:

```text
Anonymous Traffic Protection
    Adaptive
    Standard
    Maximum
```

---

# 144. Advanced User Setting

Optional:

```text
Preserve Battery
Balanced
Maintain Anonymity
```

---

# 145. Do Not Expose Raw Lambda

Normal users should not tune stochastic parameters.

---

# 146. Developer Settings

Can expose synthetic simulation parameters in test builds only.

---

# 147. Abuse Resistance

Attackers may try to exploit cover generation.

---

# 148. Resource Exhaustion

Bound:

```text
CPU
bandwidth
memory
loop generation
```

---

# 149. Malicious Provider Backpressure

Provider cannot cause unbounded local queue growth.

---

# 150. Quota Interaction

If mailbox quota full:

```text
real traffic waits/errors
cover may continue according to policy
```

---

# 151. Intersection Attack Mitigation

Traffic shaping can help by:

```text
maintaining traffic while idle
reducing exact online/offline correlation
randomizing timing
```

---

# 152. Limits

If user is offline for days:

```text
no client traffic
```

still reveals absence.

---

# 153. Always-On Node Option

Future:

```text
personal anonymity node
```

could maintain traffic independent of user device.

---

# 154. Personal Node

Potentially:

```text
home server
VPS
trusted always-on device
```

---

# 155. Personal Node Privacy Tradeoff

Could improve availability but create stable infrastructure linkability.

Needs separate architecture.

---

# 156. Multi-Device Shaping

Each device may shape independently.

---

# 157. Correlated Devices

Simultaneous activity across devices can create correlation.

---

# 158. Account-Wide Coordination

Avoid centralized exact schedule sharing.

---

# 159. Recommended Initial Policy

Independent per-device shaping.

---

# 160. Conversation-Level Traffic Policies

Do not let one conversation disable global cover if Maximum mode enabled.

---

# 161. Per-Conversation Privacy

Can increase strictness.

---

# 162. Cannot Reduce Mandatory Global Policy

Hard rule.

---

# 163. Emergency Interaction

Emergency traffic may require immediate delivery.

---

# 164. Emergency Bypass

If user explicitly prioritizes emergency reachability:

```text
may bypass shaping delay
```

but this is a privacy downgrade and must be deliberate.

---

# 165. Security Alerts

Critical local security events do not necessarily require network emission.

---

# 166. Key Rotation Traffic

Should use same shaping where practical.

---

# 167. Provider Health Loops

Loop traffic can provide health signal.

---

# 168. Loop RTT

Use broad bucket:

```text
Good
Degraded
Poor
```

---

# 169. No Per-Node RTT

Avoid route fingerprinting.

---

# 170. Traffic Scheduler API

```rust
#[async_trait]
pub trait AnonymousTrafficScheduler: Send + Sync {
    async fn start(
        &self,
        profile: TrafficProfile,
    ) -> Result<(), TrafficShapingError>;

    async fn enqueue_real(
        &self,
        cell: AnonymousCell,
        priority: AnonymousFramePriority,
    ) -> Result<(), TrafficShapingError>;

    async fn update_profile(
        &self,
        profile: TrafficProfile,
    ) -> Result<(), TrafficShapingError>;

    async fn stop(
        &self,
    ) -> Result<(), TrafficShapingError>;

    fn status(
        &self,
    ) -> TrafficShapingStatus;
}
```

---

# 171. Status

```rust
pub struct TrafficShapingStatus {
    pub runtime: ShapingRuntimeState,
    pub mode: TrafficShapingMode,
    pub queue_depth: u32,
    pub cover_active: bool,
    pub loop_active: bool,
    pub congestion: CongestionState,
}
```

---

# 172. Scheduler Clock

Use monotonic clock.

---

# 173. Wall Clock

Not required for emission timing.

---

# 174. Random Source

Use cryptographically secure RNG where schedule unpredictability matters.

---

# 175. Seed Handling

Do not persist schedule seed unnecessarily.

---

# 176. Deterministic Test RNG

Testkit only.

---

# 177. Simulation Architecture

Need discrete-event simulator.

---

# 178. Simulation Purpose

Evaluate:

```text
latency
bandwidth
queue growth
cover ratio
burst flattening
intersection resistance proxies
battery cost
```

---

# 179. Simulator Inputs

```text
user message model
network latency model
provider capacity
rate profile
cover profile
device sleep model
```

---

# 180. Simulator Outputs

```text
delivery latency distribution
queue occupancy
traffic volume
correlation metrics
battery estimate
```

---

# 181. Do Not Treat Simulation as Privacy Proof

Hard rule.

Simulation informs engineering only.

---

# 182. Statistical Tests

Verify scheduler distribution approximately matches configured model.

---

# 183. Example Property

Over long run:

```text
inter-arrival distribution
```

should be within tolerance.

---

# 184. Avoid Brittle Random Tests

Use seeded statistical tests with appropriate tolerances.

---

# 185. Unit Tests

Test:

```text
slot scheduling
queue selection
profile transitions
cover substitution
loop generation
congestion behavior
```

---

# 186. Property Tests

Properties:

```text
Maximum mode never disables shaping silently
real cells never dropped silently
cover cells can be discarded under congestion
queue bounds enforced
profile downgrade requires policy
```

---

# 187. Burst Tests

Inject:

```text
100 messages at once
```

Verify smoothing.

---

# 188. Idle Tests

No real traffic.

Verify cover continues under profile.

---

# 189. Resume Tests

Device wakes after long sleep.

Verify no huge deterministic burst.

---

# 190. Network Change Tests

Wi-Fi ↔ mobile.

---

# 191. Battery Saver Tests

Adaptive profile adjusts.

Maximum profile reports degradation if OS blocks.

---

# 192. Congestion Tests

Provider queue pressure.

---

# 193. Backpressure Tests

Fragmenter pauses.

---

# 194. Attachment Fairness Tests

Large attachment does not starve text messages.

---

# 195. Loop Tests

Loop traffic generated, returned, measured, discarded.

---

# 196. Privacy Leak Tests

User action timestamp should not map 1:1 to external send in strict mode.

---

# 197. Metric Leak Tests

No per-contact timing in logs/telemetry.

---

# 198. Plugin Tests

Maximum mode blocks/restricts network plugin traffic per policy.

---

# 199. Android Tests

```text
Doze
battery saver
background restriction
process kill
network switch
```

---

# 200. Desktop Tests

```text
daemon restart
sleep/resume
network loss
```

---

# 201. Performance Tests

Measure:

```text
CPU
memory
bandwidth
scheduler wakeups
queue latency
```

---

# 202. Battery Tests

Android benchmark on reference devices.

---

# 203. Bandwidth Tests

Measure hourly cover overhead.

---

# 204. SLOs

Traffic shaping has separate SLOs:

```text
scheduler availability
bounded queue
no silent downgrade
profile enforcement
```

---

# 205. No Traditional Latency SLO for Maximum Mode

Privacy may intentionally add delay.

---

# 206. Failure Taxonomy

```rust
pub enum TrafficShapingError {
    ProviderUnavailable,
    QueueFull,
    InvalidProfile,
    RandomnessUnavailable,
    SchedulerStopped,
    ResourceRestricted,
    Unsupported,
    Internal,
}
```

---

# 207. Resource Restricted

Example:

```text
OS blocks background operation
```

---

# 208. Degraded State

Do not silently weaken.

---

# 209. UI Degradation

```text
Maximum anonymity traffic shaping is limited by system battery restrictions.
```

---

# 210. Configuration

RON example:

```ron
(
    shaping_mode: Maximum,
    rate_class: MaximumPrivacy,
    emission_pattern: ProviderNative,
    batch_policy: PrivacyOptimized,
    power_policy: MaintainAnonymity,
)
```

---

# 211. Secrets

No secret scheduler material in config.

---

# 212. Crate Layout

Recommended:

```text
crates/
├── siar-anonymity-traffic/
├── siar-anonymity-scheduler/
├── siar-anonymity-cover/
├── siar-anonymity-loop/
├── siar-anonymity-congestion/
├── siar-anonymity-traffic-sim/
└── siar-anonymity-traffic-testkit/
```

---

# 213. `siar-anonymity-traffic`

Owns:

```text
TrafficProfile
TrafficClass
rate/budget types
```

---

# 214. `siar-anonymity-scheduler`

Owns:

```text
emission slots
queue selection
profile transitions
```

---

# 215. `siar-anonymity-cover`

Owns cover generation.

---

# 216. `siar-anonymity-loop`

Owns loop traffic generation/measurement.

---

# 217. `siar-anonymity-congestion`

Owns:

```text
backpressure
queue limits
provider congestion
```

---

# 218. `siar-anonymity-traffic-sim`

Discrete-event simulation.

---

# 219. Dependency Direction

```text
framing
   ↓
traffic scheduler
   ↓
provider adapter
```

---

# 220. Provider Does Not Call Application Directly

Hard rule.

---

# 221. Startup

```text
load profile
→ discover provider capability
→ warm up scheduler
→ enter steady state
```

---

# 222. Shutdown

Graceful:

```text
stop new cover
flush durable real queue state
stop scheduler
```

---

# 223. Crash Recovery

Real traffic remains durable outside volatile scheduler.

---

# 224. Cover State

Need not survive crash.

---

# 225. Loop State

Need not survive crash except aggregate health.

---

# 226. Profile Change

Example:

```text
Standard → Maximum
```

transition should ramp rather than abrupt pattern shift when practical.

---

# 227. Privacy Downgrade

```text
Maximum → Standard
```

should require explicit user/policy action.

---

# 228. Policy Audit

Record locally:

```text
traffic profile changed
battery-induced degradation
provider capability changed
```

---

# 229. Audit Privacy

No peer-level identifiers.

---

# 230. Security Invariants

Mandatory:

```text
1. Real messages are decoupled from immediate network emission in strict modes.
2. Maximum Anonymity never disables shaping silently.
3. Cover/loop packets use the same observable size profile as real packets where provider supports it.
4. Real durable messages are never dropped to preserve cover traffic.
5. Cover traffic may be discarded under congestion.
6. Large attachments cannot monopolize the schedule.
7. Provider-native shaping is reused rather than accidentally duplicated.
8. Background OS restrictions produce degradation, not privacy downgrade.
9. Metrics/logs never correlate exact user actions to external emission times.
10. Network plugins cannot bypass shaping policy under Maximum Anonymity.
```

---

# 231. Initial Production Scope

Implement first:

```text
TrafficProfile
provider capability detection
real-vs-cover substitution
basic loop traffic
bounded scheduler queues
adaptive/standard/maximum modes
short batching
rate smoothing
congestion/backpressure
Android/desktop lifecycle integration
privacy-safe diagnostics
simulation/testkit
```

Then add:

```text
advanced Poisson scheduling
constant-rate profiles
cover-aware mailbox polling
multi-provider shaping
advanced intersection-attack mitigations
personal always-on anonymity node
```

---

# 232. Definition of Done

Part 37 is complete when:

- cover traffic, loop traffic, and real traffic are distinct internal classes
- strict modes schedule emission independently from immediate user action
- Loopix-style randomized scheduling is represented
- provider-native traffic shaping can be reused safely
- cover and loop traffic are externally normalized where provider permits
- batching, burst flattening, rate smoothing, and hysteresis are defined
- queue priority and fairness prevent attachment starvation
- congestion/backpressure never silently drops real durable messages
- Android background restrictions and desktop daemon behavior are defined
- resume/sleep/network-switch behavior avoids obvious deterministic bursts
- metadata-sensitive features such as presence/typing/receipts/plugins integrate with shaping policy
- diagnostics and metrics are privacy-safe and low-cardinality
- discrete-event simulation is defined but not treated as proof of anonymity
- statistical, fault, battery, bandwidth, privacy-leak, and performance tests are specified
- security invariants prohibit silent shaping disablement and privacy downgrade

---

# 233. Final Architecture

```text
                   REAL MESSAGE CELLS
                           │
                           │
          ┌────────────────┼────────────────┐
          │                │                │
      COVER CELLS       LOOP CELLS      CONTROL CELLS
          │                │                │
          └────────────────┼────────────────┘
                           ▼
                 TRAFFIC SHAPING SCHEDULER
                           │
                  Randomized Emission
                  Batching / Smoothing
                  Fairness / Backpressure
                           │
                           ▼
                   ANONYMOUS PROVIDER
                           │
                           ▼
                         MIXNET
```

Strict-mode send semantics:

```text
User presses Send
      ↓
Persist message
      ↓
Frame into anonymous cells
      ↓
Queue
      ↓
Wait for scheduled emission opportunity
      ↓
Real cell substitutes cover
      ↓
Provider / Mixnet
```

---

# 234. Final Principle

Strong anonymity requires hiding not only **what** is sent and **where** it goes, but also reducing how clearly an observer can infer **when** real communication occurs.

The correct model is:

```text
normalized cells
+
cover traffic
+
loop traffic
+
randomized emission
+
batching
+
bounded queues
+
resource-aware policy
```

not:

```text
user presses Send
→ packet leaves immediately
```

This gives SIAR the traffic-analysis-resistance layer needed to make its high-anonymity transport materially stronger than ordinary encrypted relaying.
