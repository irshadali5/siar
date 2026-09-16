# Core System Architecture Part 60 — Anonymous Network Time, Clock Privacy, Replay Windows, Epoch Coordination & Temporal Metadata Resistance Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 60  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 34–59  

**Primary purpose:** define how SIAR handles time, monotonic clocks, wall-clock synchronization, replay protection, epoch coordination, timestamp privacy, expiration, delayed delivery, key rotation, federation time compatibility, and temporal metadata resistance without turning clocks into a cross-service correlation mechanism.

---

# 1. Purpose

Time is necessary for:

```text
expiry
replay protection
key rotation
directory epochs
mailbox retention
provider descriptors
credit validity
governance policy
federation agreements
```

Time is also dangerous.

Precise timestamps can reveal:

```text
when a user was active
when two contacts communicated
when a device came online
when a mailbox was fetched
when a message crossed domains
```

The governing principle is:

> **SIAR should use the least precise and least globally correlated notion of time required for correctness, while keeping security-critical freshness and replay protection explicit and bounded.**

---

# 2. Architectural Position

```text
Hardware / OS Clock
        │
        ├── Monotonic Clock
        └── Wall Clock
               │
               ▼
        Time Abstraction Layer
               │
               ├── epochs
               ├── expiry
               ├── replay windows
               ├── coarse timestamps
               └── privacy shaping
               │
               ▼
        Protocols / Storage / Governance
```

---

# 3. Core Separation

Keep distinct:

```text
wall-clock time
monotonic duration
protocol epoch
key epoch
replay sequence
user-visible timestamp
provider retention deadline
governance validity window
```

---

# 4. Non-Goals

Part 60 does not require:

```text
globally synchronized user clocks
precise per-message server timestamps
cross-service universal event IDs
continuous NTP-style user tracking
```

---

# 5. Time Domains

```rust
pub enum TimeDomain {
    MonotonicLocal,
    WallClockLocal,
    ProtocolEpoch,
    SecurityEpoch,
    GovernanceEpoch,
    ProviderEpoch,
}
```

---

# 6. Monotonic Time

Use for:

```text
timeouts
retry delays
backoff
queue residence
session timers
```

---

# 7. Why Monotonic

Wall clock can:

```text
jump backward
jump forward
change with timezone
change with user settings
```

---

# 8. Monotonic Instant Wrapper

```rust
pub struct MonoInstant(std::time::Instant);
```

---

# 9. Wall Clock

Use only when real-world chronology is required.

---

# 10. Examples

```text
policy validity
certificate expiry
public transparency
legal retention
signed catalog validity
```

---

# 11. Wall Time Type

```rust
pub struct WallTimeUtc(pub i64);
```

Represent:

```text
UTC-based protocol time
```

not local timezone.

---

# 12. User Local Time

Presentation only.

---

# 13. Timezone

Never part of security decision.

---

# 14. Hard Rule

Do not use locale/timezone as protocol identity signal.

---

# 15. Clock Trust Classes

```rust
pub enum ClockTrust {
    TrustedEnough,
    Skewed,
    Unknown,
    Untrusted,
}
```

---

# 16. TrustedEnough

Within allowed skew for operation.

---

# 17. Skewed

Known deviation but bounded.

---

# 18. Unknown

No reliable wall-time source.

---

# 19. Untrusted

Detected rollback/manipulation.

---

# 20. Clock Source

Potential:

```text
OS clock
signed directory time
signed governance time
provider response time
```

---

# 21. No Single Mandatory Time Server

Hard rule.

---

# 22. Time Source Diversity

For critical freshness:

```text
multiple authenticated sources
```

can improve confidence.

---

# 23. Time Observation

```rust
pub struct TimeObservation {
    pub source: TimeSourceId,
    pub wall_time: WallTimeUtc,
    pub received_at: MonoInstant,
    pub assurance: TimeAssurance,
}
```

---

# 24. Time Assurance

```rust
pub enum TimeAssurance {
    LocalOnly,
    AuthenticatedRemote,
    QuorumObserved,
}
```

---

# 25. Time Quorum

Optional for high-security control plane.

---

# 26. Client Recommendation

Do not require remote time quorum for ordinary messaging.

---

# 27. Clock Skew Budget

```rust
pub struct ClockSkewBudget {
    pub max_past: Duration,
    pub max_future: Duration,
}
```

---

# 28. Different Protocols Need Different Budgets

Examples:

```text
call signaling: seconds/minutes
directory snapshot: minutes/hours
backup archive: hours/days
```

---

# 29. No Universal Skew Constant

Hard rule.

---

# 30. Protocol Time Policy

```rust
pub struct ProtocolTimePolicy {
    pub skew: ClockSkewBudget,
    pub replay_window: ReplayWindowPolicy,
    pub expiry_mode: ExpiryMode,
}
```

---

# 31. Epochs

Epochs reduce reliance on precise timestamps.

---

# 32. Epoch Type

```rust
pub struct Epoch(pub u64);
```

---

# 33. Epoch Duration

Protocol-specific.

---

# 34. Examples

```text
mixnet topology epoch
mailbox capability epoch
presence epoch
provider descriptor epoch
```

---

# 35. Epoch Boundary

Prefer coarse.

---

# 36. Benefit

Users within same epoch are less distinguishable.

---

# 37. Epoch Schedule

```rust
pub struct EpochSchedule {
    pub duration: Duration,
    pub grace_before: Duration,
    pub grace_after: Duration,
}
```

---

# 38. Grace Window

Supports skew and rolling transition.

---

# 39. Never Require Millisecond Epoch Alignment

Hard rule.

---

# 40. Overlap

Old + new epoch may both be accepted briefly.

---

# 41. Epoch Transition

```text
prepare next
→ overlap
→ prefer next
→ retire previous
```

---

# 42. Security Epoch

Monotonic counter for:

```text
revocation
key transition
anti-rollback
```

---

# 43. Security Epoch Does Not Need Wall Clock

Hard rule where counter suffices.

---

# 44. Sequence Number

Use when ordering is enough.

---

# 45. Sequence Type

```rust
pub struct SequenceNumber(pub u64);
```

---

# 46. Replay Protection

Prefer:

```text
nonce
sequence
epoch
```

over precise timestamp alone.

---

# 47. Replay Key

```rust
pub struct ReplayKey {
    pub scope: ReplayScope,
    pub epoch: Epoch,
    pub nonce: Nonce,
}
```

---

# 48. Replay Scope

```rust
pub enum ReplayScope {
    Session,
    Device,
    Mailbox,
    Provider,
    FederationPeer,
}
```

---

# 49. No Global Replay Database

Hard rule.

---

# 50. Replay Cache

Service-scoped.

---

# 51. Replay Window

```rust
pub struct ReplayWindowPolicy {
    pub accepted_epochs: u8,
    pub max_entries: usize,
    pub ttl: Duration,
}
```

---

# 52. Replay Cache Bound

Mandatory.

---

# 53. Replay Cache Privacy

Must not become:

```text
long-term packet history
```

---

# 54. Expire Aggressively

After security window.

---

# 55. Packet Replay

Mixnet/Sphinx layer should use reviewed replay semantics.

---

# 56. No Custom Cryptographic Replay Primitive

Hard rule.

---

# 57. Mailbox Replay

Duplicate encrypted object may reappear.

---

# 58. Application Dedup

Use:

```text
MessageId
mailbox item token
```

not wall-clock equality.

---

# 59. Federation Replay

Per-peer short-lived nonce window.

---

# 60. Payment Replay

Part 47 uses exact spend/double-spend semantics.

---

# 61. Governance Replay

Part 53 uses monotonic policy version + trust epoch.

---

# 62. Upgrade Replay

Part 52 uses release security epoch.

---

# 63. Naming Replay

Part 59 uses record version/expiry.

---

# 64. Expiry Semantics

Need explicit model.

---

# 65. Expiry Modes

```rust
pub enum ExpiryMode {
    MonotonicDuration,
    WallClockDeadline,
    EpochBound,
    SequenceBound,
}
```

---

# 66. MonotonicDuration

Use for:

```text
session timeout
temporary local auth
```

---

# 67. WallClockDeadline

Use for:

```text
policy expires at date
```

---

# 68. EpochBound

Use for:

```text
provider descriptor
presence token
```

---

# 69. SequenceBound

Use for:

```text
revocation generation
```

---

# 70. Avoid Wall Clock Where Not Needed

Hard rule.

---

# 71. Timestamp Privacy

User-visible timestamps can reveal behavior.

---

# 72. Local Message Time

Store locally.

---

# 73. Server Timestamp

Avoid if client time is sufficient.

---

# 74. Delivery Time

Use coarse status.

---

# 75. Example

Instead of:

```text
delivered at 13:04:12.318
```

remote protocol may use:

```text
delivered within epoch E
```

---

# 76. UI Can Render Local Precision

Only from local event data.

---

# 77. Remote Precision

Should be minimized.

---

# 78. Timestamp Classes

```rust
pub enum TimestampPrecision {
    Exact,
    Second,
    Minute,
    FiveMinute,
    Hour,
    Day,
    Epoch,
}
```

---

# 79. Privacy Mode Policy

Maximum Anonymity should prefer coarser remote precision.

---

# 80. Precision Reduction

```rust
pub fn coarsen_time(
    t: WallTimeUtc,
    precision: TimestampPrecision,
) -> WallTimeUtc;
```

---

# 81. No Hidden Exact Timestamp Alongside Coarse Field

Hard rule.

---

# 82. Metadata Layer

If exact timestamp not needed:

```text
do not transmit it
```

---

# 83. Timing Correlation

Even without explicit timestamps, packet timing leaks.

---

# 84. Mitigations

Part 37:

```text
cover traffic
delay
batching
scheduled slots
```

---

# 85. Temporal Metadata Budget

```rust
pub struct TemporalMetadataBudget {
    pub max_precision: TimestampPrecision,
    pub max_event_rate_visibility: EventRateClass,
}
```

---

# 86. Event Rate Class

```rust
pub enum EventRateClass {
    Hidden,
    Coarse,
    Visible,
}
```

---

# 87. Strict Mode

Prefer:

```text
Hidden/Coarse
```

---

# 88. User Activity Timestamp

Do not expose exact:

```text
last seen
last online
last active
```

by default.

---

# 89. Presence

Part 45/30 use:

```text
Available
Away
Unknown
```

rather than exact last-active time.

---

# 90. Typing

Ephemeral.

---

# 91. Typing Event TTL

Monotonic/local short TTL.

---

# 92. Read Receipts

Prefer sequence-based.

---

# 93. Why

Exact read time leaks activity.

---

# 94. Read Receipt

```rust
pub struct ReadReceipt {
    pub through_sequence: MessageSequence,
}
```

---

# 95. Optional Timestamp

If product offers, local policy controlled.

---

# 96. Maximum Anonymity

No read timestamp.

---

# 97. Message Ordering

Use logical ordering.

---

# 98. Message Sequence

```rust
pub struct MessageSequence(pub u64);
```

---

# 99. Device Ordering

Per conversation/device.

---

# 100. Distributed Ordering

Do not pretend global total order.

---

# 101. Lamport/HLC

Could help local-first merge.

---

# 102. Hybrid Logical Clock

Potential for sync.

---

# 103. Privacy Risk

HLC physical component can expose time.

---

# 104. Recommendation

Keep HLC local/storage-level where possible.

---

# 105. Wire Exposure

Coarsen/strip physical time if not required.

---

# 106. HLC Type

```rust
pub struct HybridLogicalTime {
    pub physical_bucket: u64,
    pub logical: u32,
}
```

---

# 107. Bucket Physical Component

Avoid exact milliseconds.

---

# 108. Local-First Merge

Can use logical causality.

---

# 109. Vector Clocks

Potential for some multi-device state.

---

# 110. Cost

Can grow with device count.

---

# 111. Use Sparingly

Hard rule.

---

# 112. Causal Timestamp

Per scoped state only.

---

# 113. No Global Causal Clock

Hard rule.

---

# 114. Device Clock Privacy

Do not send raw boot time.

---

# 115. Device Uptime

Infrastructure node may expose bucketed uptime.

---

# 116. Client Uptime

Local only.

---

# 117. Clock Fingerprinting

Differences in clock skew can identify devices.

---

# 118. Threat

Adversary observes:

```text
consistent timestamp offset
drift pattern
```

---

# 119. Mitigation

Do not transmit raw device clock.

---

# 120. Normalize Protocol Time

Use:

```text
epoch
coarse UTC
server-issued nonce
```

---

# 121. No Raw Clock Drift Telemetry

Hard rule for clients.

---

# 122. Timezone Fingerprinting

Do not expose timezone to providers.

---

# 123. Locale

Separate from protocol.

---

# 124. Scheduled Operations

Examples:

```text
delayed message
scheduled deletion
scheduled backup
```

---

# 125. Local Scheduler

Prefer local.

---

# 126. Remote Scheduled Delivery

Requires service.

---

# 127. Privacy Risk

Server learns scheduled time.

---

# 128. Strict Mode

Prefer:

```text
local send-at-time
```

when device availability allows.

---

# 129. Remote Schedule

If required, use coarse slot.

---

# 130. Scheduled Slot

```rust
pub struct ScheduledSlot {
    pub epoch: Epoch,
    pub slot: u16,
}
```

---

# 131. Delayed Message

Encrypt before scheduling.

---

# 132. Provider Sees

At most:

```text
coarse execution window
```

---

# 133. Self-Destruct / Expiring Messages

Complex in distributed systems.

---

# 134. Hard Truth

Cannot guarantee deletion from recipient-controlled devices.

---

# 135. Expiry Semantics

Means:

```text
client should hide/delete after deadline
```

not magical remote erasure.

---

# 136. User Notice

Must be honest.

---

# 137. Expiring Message

```rust
pub struct MessageExpiryPolicy {
    pub mode: ExpiryMode,
    pub expiry: ExpiryValue,
}
```

---

# 138. Offline Recipient

If expiry passes before receive:

policy decides:

```text
drop
or
deliver with expired marker
```

---

# 139. Strict Privacy

Often drop.

---

# 140. Mailbox TTL

Provider can delete ciphertext after TTL.

---

# 141. TTL Precision

Coarse.

---

# 142. Retention Timing

Part 55 legal retention may require wall-clock deadlines.

---

# 143. Separate Legal Time From Messaging Time

Hard rule.

---

# 144. Clock Rollback

Device clock moves backward.

---

# 145. Detection

Compare:

```text
stored last-seen wall clock
monotonic elapsed
signed remote observations
```

---

# 146. Rollback Response

Security-sensitive operations may pause.

---

# 147. Never Auto-Extend Expired Credential Due To Clock Rollback

Hard rule.

---

# 148. Clock Jump Forward

Could falsely expire credentials.

---

# 149. Grace Policy

Can re-check authenticated time.

---

# 150. Clock State

```rust
pub struct ClockState {
    pub trust: ClockTrust,
    pub estimated_skew: Option<Duration>,
}
```

---

# 151. Clock Repair

User/system can resynchronize.

---

# 152. No Need To Expose Exact Remote Time

Hard rule.

---

# 153. Time Authority

Avoid central global dependency.

---

# 154. Signed Time Beacon

Optional.

---

# 155. Time Beacon

```rust
pub struct SignedTimeBeacon {
    pub domain: TimeAuthorityDomain,
    pub coarse_time: WallTimeUtc,
    pub epoch: Epoch,
    pub valid_for: Duration,
    pub signature: TimeAuthoritySignature,
}
```

---

# 156. Beacon Precision

Coarse.

---

# 157. Multiple Sources

Recommended for critical control plane.

---

# 158. Beacon Privacy

Fetching beacon should be cacheable/mirrored.

---

# 159. No Per-User Time Query

Hard rule.

---

# 160. Directory Epoch Coordination

Part 39.

---

# 161. Epoch Publication

Before activation.

---

# 162. Client Preload

Can fetch next epoch early.

---

# 163. Avoid Synchronized Stampede

Use jitter.

---

# 164. Topology Epoch Transition

```text
publish next
→ warm cache
→ activate
→ overlap
→ retire old
```

---

# 165. Mixnet Epoch

Route uses one topology epoch.

---

# 166. Mid-Route Epoch Change

Existing packet remains valid under original bounded rules.

---

# 167. No Forced Immediate Re-route

Hard rule unless revoked node/security emergency.

---

# 168. Cover Traffic Epoch

Part 37 schedules may use local randomized phase.

---

# 169. Avoid Global Tick Alignment

Hard rule.

---

# 170. Why

Global synchronized emission creates fingerprintable boundaries.

---

# 171. Randomized Phase Offset

```rust
pub struct SchedulePhaseOffset(pub Duration);
```

---

# 172. Per-Client

Random.

---

# 173. Persist?

Could rotate periodically.

---

# 174. Avoid Stable Long-Term Phase Fingerprint

Hard rule.

---

# 175. Mailbox Polling

Periodic exact polling leaks pattern.

---

# 176. Better

```text
push wake
jittered polling
batched fetch
```

---

# 177. Maximum Anonymity

Use cover/batch policy where possible.

---

# 178. Poll Schedule

```rust
pub struct PollSchedule {
    pub base: Duration,
    pub jitter: Duration,
}
```

---

# 179. Doze/Mobile Reality

Android may delay wakeups.

---

# 180. Do Not Claim Constant Timing Cover On Mobile

Hard rule.

---

# 181. Provider Fetch Timing

Can leak activity.

---

# 182. Mitigation

Use:

```text
fetch batching
cover fetch
push-less periodic windows
```

depending mode.

---

# 183. Realtime Calls

Timing is inherently visible to relays.

---

# 184. Cannot Hide Fully

Hard truth.

---

# 185. Mitigation

Can pad:

```text
bitrate
short silence
```

but call duration remains difficult.

---

# 186. Call Start Time

Do not log per-user remotely.

---

# 187. Relay SLO Metrics

Aggregate only.

---

# 188. Bulk Transfers

Large files reveal duration/volume.

---

# 189. Timing Mitigation

Chunk pacing/batching can reduce correlation.

---

# 190. Cost

Latency + bandwidth.

---

# 191. Policy Controlled

Hard rule.

---

# 192. Credits/Payments

Time can link:

```text
funding
→ immediate spend
```

---

# 193. Mitigation

Part 47:

```text
prepay
delay use
batch settlement
```

---

# 194. Credit Expiry

Prefer coarse epoch/day.

---

# 195. No Per-Packet Spend Timestamp

Hard rule.

---

# 196. Provider Pricing Epoch

Part 48/54.

---

# 197. Price Changes

Coarse epochs.

---

# 198. No High-Frequency Personalized Pricing

Hard rule.

---

# 199. Governance Time

Policy effective_at and expiry.

---

# 200. Activation Delay

Allows propagation.

---

# 201. Emergency Policy

Can activate immediately.

---

# 202. Still Versioned

Hard rule.

---

# 203. Federation Time

Domains may have skew.

---

# 204. Federation Agreement

Uses validity window + grace.

---

# 205. Foreign Clock Not Trusted Directly

Hard rule.

---

# 206. Peering Negotiation

Use local validated clock policy.

---

# 207. Federation Epoch Mapping

Domains may use different epoch lengths.

---

# 208. No Requirement For Shared Global Epoch

Hard rule.

---

# 209. Cross-Domain Message

Carries:

```text
local domain epoch context
```

only if required.

---

# 210. Translate At Boundary

Federation gateway maps validity to local policy.

---

# 211. No Raw Client Timestamp Crossing Domains

Hard rule.

---

# 212. Federated Replay Window

Per domain pair.

---

# 213. Federation Time Policy

```rust
pub struct FederationTimePolicy {
    pub max_skew: Duration,
    pub replay_ttl: Duration,
    pub agreement_grace: Duration,
}
```

---

# 214. Disaster / Partition

Clock sources may disappear.

---

# 215. Part 50 Integration

Use cached signed epochs/policies within freshness window.

---

# 216. If Freshness Expires

Strict mode may stop new sessions.

---

# 217. Existing Local Sessions

Can continue if monotonic conditions remain valid.

---

# 218. No Fabricated Time

Hard rule.

---

# 219. Offline Local Mesh

Can use:

```text
monotonic duration
local sequence
relative expiry
```

---

# 220. Rejoin Internet

Reconcile with signed wall-time/epoch.

---

# 221. Offline Message Timestamp

Can be user-device local.

---

# 222. Trust Label

UI may distinguish:

```text
device time
network-confirmed time
```

if needed.

---

# 223. No False Precision

Hard rule.

---

# 224. Temporal Privacy Policy

```rust
pub struct TemporalPrivacyPolicy {
    pub remote_precision: TimestampPrecision,
    pub allow_read_time: bool,
    pub allow_last_seen: bool,
    pub timing_shaping: TimingShapingMode,
}
```

---

# 225. Timing Shaping Modes

```rust
pub enum TimingShapingMode {
    None,
    Adaptive,
    Standard,
    Maximum,
}
```

---

# 226. Part 37 Integration

Implementation delegated there.

---

# 227. Temporal Metadata Classification

```rust
pub enum TemporalMetadataClass {
    LocalOnly,
    CoarseRemote,
    SecurityCritical,
    ForbiddenRemote,
}
```

---

# 228. LocalOnly

Examples:

```text
exact local message receipt time
```

---

# 229. CoarseRemote

Examples:

```text
provider descriptor valid-until
```

---

# 230. SecurityCritical

Examples:

```text
policy expiry
revocation freshness
```

---

# 231. ForbiddenRemote

Examples:

```text
exact user last-active timestamp
```

in strict mode.

---

# 232. Time API

Use typed wrappers.

---

# 233. No Generic `SystemTime` Everywhere

Hard rule.

---

# 234. Time Provider Trait

```rust
pub trait TimeProvider: Send + Sync {
    fn monotonic_now(&self) -> MonoInstant;
    fn wall_now(&self) -> Result<WallTimeUtc, ClockError>;
    fn clock_state(&self) -> ClockState;
}
```

---

# 235. Epoch Service

```rust
pub trait EpochService {
    fn current(
        &self,
        schedule: &EpochSchedule,
    ) -> Result<Epoch, ClockError>;

    fn accepts(
        &self,
        epoch: Epoch,
        schedule: &EpochSchedule,
    ) -> bool;
}
```

---

# 236. Replay Guard

```rust
pub trait ReplayGuard {
    fn check_and_record(
        &self,
        key: ReplayKey,
    ) -> Result<ReplayDecision, ReplayError>;
}
```

---

# 237. Replay Decision

```rust
pub enum ReplayDecision {
    Fresh,
    Duplicate,
    Expired,
}
```

---

# 238. Temporal Privacy Gate

```rust
pub trait TemporalPrivacyGate {
    fn sanitize(
        &self,
        event: TemporalEvent,
        policy: &TemporalPrivacyPolicy,
    ) -> SanitizedTemporalEvent;
}
```

---

# 239. Expiry Evaluator

```rust
pub trait ExpiryEvaluator {
    fn evaluate(
        &self,
        expiry: &ExpiryValue,
        context: &TimeContext,
    ) -> Result<ExpiryStatus, ClockError>;
}
```

---

# 240. Expiry Status

```rust
pub enum ExpiryStatus {
    Valid,
    Grace,
    Expired,
    Unknown,
}
```

---

# 241. Unknown

Should fail closed for security-critical credential.

---

# 242. User Content Expiry

May degrade gracefully.

---

# 243. Different Fail Behavior

Hard rule.

---

# 244. Security Credential

Unknown time -> do not accept.

---

# 245. Local Draft

Unknown time -> no problem.

---

# 246. Replay Persistence

Some replay state survives restart.

---

# 247. Security-Critical Replay Cache

Persist bounded summary if needed.

---

# 248. Mix Packet Replay

Follow underlying protocol requirements.

---

# 249. Bloom Filter

Potential for bounded replay memory.

---

# 250. False Positives

Must be understood.

---

# 251. Cuckoo Filter

Potential alternative.

---

# 252. Use Reviewed Strategy

Hard rule.

---

# 253. Replay Cache Rotation

Per epoch.

---

# 254. Destroy Old Cache

After grace.

---

# 255. No Long-Term Replay Archive

Hard rule.

---

# 256. Temporal Observability

Safe:

```text
clock skew bucket
epoch transition success
replay reject count
policy freshness bucket
```

---

# 257. Forbidden Telemetry

No:

```text
raw client timestamp
clock drift fingerprint
exact user activity time
```

---

# 258. Clock Skew Metric

Infrastructure nodes only or coarse opt-in client.

---

# 259. Maximum Anonymity

No remote client clock skew telemetry.

---

# 260. Time Incident Types

```rust
pub enum TimeIncident {
    ClockRollback,
    ClockJump,
    EpochDivergence,
    ReplayFlood,
    TimeSourceUnavailable,
    StalePolicyTime,
}
```

---

# 261. Replay Flood

Security incident.

---

# 262. Epoch Divergence

Could indicate:

```text
partition
bad rollout
malicious source
```

---

# 263. Time Source Compromise

Remove source.

---

# 264. No Clock Source Automatically Gains Trust

Hard rule.

---

# 265. Clock Privacy Tests

Need dedicated simulator.

---

# 266. Scenarios

```text
clock rollback
clock jump forward
timezone change
DST
offline device
partition
stale epoch
replay flood
federation skew
```

---

# 267. Timezone Test

No protocol change.

---

# 268. DST Test

No protocol change.

---

# 269. Clock Rollback Test

Expired credential does not become valid.

---

# 270. Clock Forward Test

Re-authenticated time can recover if safe.

---

# 271. Epoch Overlap Test

Old/new accepted only within grace.

---

# 272. Replay Test

Duplicate rejected.

---

# 273. Replay Cache Bound Test

Memory bounded.

---

# 274. Cache Rotation Test

Old replay state removed after safe window.

---

# 275. Timestamp Coarsening Test

No hidden exact field remains.

---

# 276. Read Receipt Test

Sequence-only strict mode.

---

# 277. Presence Test

No exact last-seen.

---

# 278. Federation Skew Test

Valid peer accepted within policy.

---

# 279. Federation Excess Skew Test

Rejected/degraded safely.

---

# 280. Partition Test

Cached policy works only within freshness bound.

---

# 281. Offline Mesh Test

Relative/monotonic timers work without wall clock.

---

# 282. Scheduled Message Test

Remote provider sees only configured coarse slot.

---

# 283. Fuzzing

Fuzz:

```text
epoch arithmetic
expiry parser
replay key
time beacon
federation time policy
```

---

# 284. Property Tests

Properties:

```text
security epoch never decreases
expired credential never becomes valid due to wall-clock rollback
replay cache accepts a fresh nonce at most once per scope/window
coarsened timestamp contains no finer precision than policy allows
```

---

# 285. Formal Verification Targets

Strong candidates:

```text
epoch transition
replay-window state
clock rollback handling
policy-expiry/grace logic
```

---

# 286. TLA+ Candidate

Multi-node epoch coordination with skew/partition.

---

# 287. Kani Candidate

Epoch arithmetic and overflow.

---

# 288. Loom Candidate

Concurrent replay-cache check/insert.

---

# 289. Performance Tests

Measure:

```text
replay lookup
epoch lookup
timestamp coarsening
time policy evaluation
```

---

# 290. Hot Path Constraint

Replay protection may be hot.

---

# 291. Avoid Blocking Clock Fetch

Hard rule.

---

# 292. Cache Time State

Locally.

---

# 293. Async Refresh

For authenticated remote observations.

---

# 294. Clock Failure

Do not crash network.

---

# 295. Degraded Mode

Use:

```text
monotonic/local state
cached signed time
```

within bounds.

---

# 296. Crate Layout

Recommended:

```text
crates/
├── siar-time-core/
├── siar-clock/
├── siar-epoch/
├── siar-replay/
├── siar-expiry/
├── siar-temporal-privacy/
├── siar-time-beacon/
├── siar-federation-time/
├── siar-time-observability/
└── siar-time-testkit/
```

---

# 297. `siar-time-core`

Owns:

```text
typed time
clock trust
time errors
```

---

# 298. `siar-clock`

Wall/monotonic abstraction.

---

# 299. `siar-epoch`

Epoch schedule/grace/transition.

---

# 300. `siar-replay`

Scoped replay protection.

---

# 301. `siar-expiry`

Expiry semantics.

---

# 302. `siar-temporal-privacy`

Timestamp coarsening/metadata budgets.

---

# 303. `siar-time-beacon`

Authenticated coarse time sources.

---

# 304. `siar-federation-time`

Cross-domain skew/window policy.

---

# 305. `siar-time-observability`

Privacy-safe clock/epoch metrics.

---

# 306. `siar-time-testkit`

Skew/replay/partition/time-jump simulator.

---

# 307. Error Taxonomy

```rust
pub enum ClockError {
    WallClockUnavailable,
    ClockRollbackDetected,
    ClockSkewExceeded,
    TimeSourceUntrusted,
    EpochInvalid,
    ExpiryUnknown,
    Overflow,
    Internal,
}

pub enum ReplayError {
    Duplicate,
    Expired,
    CapacityExceeded,
    StoreUnavailable,
    Internal,
}
```

---

# 308. Security & Privacy Invariants

Mandatory:

```text
1. Wall-clock time is never used where monotonic duration or sequence/epoch semantics are sufficient.
2. Security-critical replay protection never relies solely on sender-provided timestamps.
3. Security epochs, policy versions, and revocation sequences never decrease.
4. Clock rollback cannot make expired credentials valid again.
5. Replay caches are scoped, bounded, short-lived, and never become packet-history archives.
6. Exact client clocks, timezone, drift, and last-active timestamps are not exposed remotely in strict modes.
7. User-visible local timestamps are separate from remote protocol timing metadata.
8. Epoch transitions include bounded grace rather than requiring perfect synchronization.
9. No global epoch is required across independent federation domains.
10. Temporal metadata is coarsened according to privacy policy and no hidden higher-precision copy is transmitted.
11. Timing shaping uses randomized phases/jitter to avoid synchronized fingerprinting.
12. Clock/time failures reduce availability where necessary but never disable replay or freshness protections.
```

---

# 309. Initial Production Scope

Implement first:

```text
typed wall vs monotonic time
protocol-specific skew budgets
coarse protocol epochs
bounded epoch overlap
scoped replay caches
sequence/nonces for replay protection
timestamp precision policy
sequence-based read receipts
no exact last-seen
jittered mailbox polling
anti-rollback clock state
signed coarse time observations for control plane
federation skew policy
time/replay fault simulator
```

Then add:

```text
multi-source authenticated time quorum
advanced HLC causality layer
privacy-preserving remote scheduling
formal epoch coordination verification
stronger temporal traffic-analysis simulation
```

---

# 310. Definition of Done

Part 60 is complete when:

- wall-clock and monotonic time are separate typed concepts
- protocol epochs and security epochs are distinct
- each time-sensitive protocol defines its own skew and expiry policy
- replay protection is scoped and not timestamp-only
- replay state is bounded and expires
- epoch overlap/grace semantics are explicit
- remote timestamp precision is privacy-policy controlled
- exact user last-active/read/presence times are not required
- timezones and raw clock drift never become protocol identifiers
- mailbox polling and cover schedules use jitter/randomized phase
- clock rollback/forward-jump handling is explicit
- offline/local mesh can operate with monotonic/relative time
- federation domains do not require one global clock/epoch
- observability excludes raw user clock fingerprints
- replay, skew, partition, rollback, expiry, fuzz, and formal tests are specified

---

# 311. Final Architecture

```text
                    OS / HARDWARE CLOCKS
                     ┌───────┴───────┐
                     │               │
                Monotonic         Wall Clock
                     │               │
                     └───────┬───────┘
                             ▼
                     TIME ABSTRACTION
                             │
          ┌──────────────────┼──────────────────┐
          │                  │                  │
       Epochs             Replay             Expiry
          │                  │                  │
          └──────────────────┼──────────────────┘
                             ▼
                   TEMPORAL PRIVACY GATE
                             │
                             ▼
                 PROTOCOL / STORAGE / UI
```

Temporal safety model:

```text
monotonic timers
+
coarse epochs
+
nonce/sequence replay protection
+
bounded skew
+
timestamp coarsening
+
timing shaping
```

not:

```text
precise global timestamps attached to every event
```

---

# 312. Final Principle

Time should support correctness without becoming a universal correlation channel.

The correct model is:

```text
relative time where possible
+
coarse authenticated wall time where necessary
+
explicit epochs
+
bounded replay windows
+
privacy-aware timestamp precision
+
timing-shaping integration
```

This architecture gives SIAR reliable freshness, expiry, replay resistance, epoch coordination, and cross-domain time compatibility while minimizing the temporal metadata available to providers, relays, federated networks, and passive observers.
