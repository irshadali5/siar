# Part 13 — Battery-Aware Scheduling Architecture

## Reusable P2P Communication Platform

**Status:** Architecture specification  
**Part:** 13 of 24  
**Primary language:** Rust  
**Primary goals:** energy-efficient communication, battery-aware task scheduling, thermal protection, radio-use minimization, mobile background adaptation, graceful degradation, emergency overrides, durable work preservation, reusable across messaging/files/DTN/calls/multipath/headless deployments

---

# 1. Purpose

A communication platform can be technically correct and still be unusable if it drains the battery, overheats the device, or repeatedly wakes radios and background services.

This is especially important because the platform may perform:

- P2P discovery
- Iroh connectivity
- Bluetooth scanning
- Wi-Fi Direct/Aware setup
- DTN store-carry-forward
- large file transfers
- multipath networking
- AV1 software encoding
- indexing
- hashing
- encryption
- background synchronization
- emergency relay work

The scheduling architecture must therefore understand energy as a first-class constraint.

The core rule is:

> **Durable work should survive low-power conditions, but execution intensity should adapt to battery, charging, thermal, foreground/background, and user policy.**

The platform should slow down, defer, batch, or downgrade before it becomes a battery or thermal problem.

---

# 2. Architectural Position

```text
Platform Signals
├── Battery
├── Charging
├── Thermal
├── Foreground/Background
├── Network Metering
└── OS Restrictions
        ↓
Battery Policy Engine
        ↓
Effective Runtime Policy
        ↓
Schedulers
├── Routing
├── Multipath
├── Files
├── DTN
├── Discovery
├── Media
├── Sync
└── Background Workers
```

The battery layer does not own feature semantics.

It tells each subsystem:

```text
how aggressively it may execute
```

---

# 3. Battery-Aware Scheduling Is Not a Separate App Mode

Do not scatter logic like:

```text
if battery < 20% { ... }
```

through every crate.

Correct:

```text
Battery/Power State
       ↓
Power Policy
       ↓
Effective Limits
       ↓
Feature Schedulers
```

This ensures consistent behavior.

---

# 4. Core Power Inputs

Recommended signals:

```text
battery level class
charging state
power saver
thermal state
foreground/background
screen state if useful
network metering
roaming
external power availability
device resource profile
```

Do not require exact battery percentage everywhere.

---

# 5. Battery Level Class

Prefer coarse classes:

```rust
pub enum BatteryLevelClass {
    Critical,
    Low,
    Medium,
    High,
    Full,
    Unknown,
}
```

This reduces overfitting and privacy leakage.

---

# 6. Charging State

```rust
pub enum ChargingState {
    NotCharging,
    ChargingSlow,
    ChargingFast,
    Full,
    Unknown,
}
```

A charging device can safely execute more background work.

---

# 7. Thermal State

```rust
pub enum ThermalState {
    Nominal,
    Elevated,
    Serious,
    Critical,
    Unknown,
}
```

Thermal pressure may be more important than battery percentage for software AV1, hashing, compression, and large transfers.

---

# 8. App Activity State

```rust
pub enum AppActivityState {
    ForegroundInteractive,
    ForegroundIdle,
    BackgroundAllowed,
    BackgroundRestricted,
    SuspendedLikely,
}
```

Platform adapters map Android/iOS/desktop behavior into this neutral model.

---

# 9. Power Policy Profiles

Recommended:

```rust
pub enum PowerPolicyProfile {
    Performance,
    Balanced,
    Saver,
    Emergency,
    AlwaysOnNode,
}
```

---

# 10. Performance Profile

Use when:

```text
charging
desktop
server
user explicitly requests
```

Characteristics:

```text
higher transfer concurrency
more aggressive discovery
multipath allowed
background hashing/indexing
```

---

# 11. Balanced Profile

Default consumer behavior.

Characteristics:

```text
normal responsiveness
moderate background work
limited discovery duty cycle
multipath only when useful
```

---

# 12. Saver Profile

Characteristics:

```text
single-path preferred
background bulk deferred
reduced scanning
reduced CPU-heavy work
lower AV1 complexity
coalesced sync
```

---

# 13. Emergency Profile

Emergency mode is not simply:

```text
maximum power always
```

It should prioritize:

```text
reachability
critical relay work
SOS
authority alerts
```

while suppressing:

```text
noncritical bulk
analytics
thumbnail generation
background indexing
```

---

# 14. Always-On Node Profile

For:

```text
desktop relay
Raspberry Pi
server
vehicle gateway on external power
```

Characteristics:

```text
continuous discovery
higher relay quota
DTN gateway active
little battery concern
```

Still enforce thermal/resource safety.

---

# 15. Power State Snapshot

```rust
pub struct PowerState {
    pub battery: BatteryLevelClass,
    pub charging: ChargingState,
    pub thermal: ThermalState,
    pub activity: AppActivityState,
    pub saver_enabled: bool,
    pub metered_network: bool,
}
```

---

# 16. Effective Power Budget

Convert platform state into execution budget.

```rust
pub struct PowerBudget {
    pub cpu_class: CpuBudgetClass,
    pub radio_class: RadioBudgetClass,
    pub background_class: BackgroundBudgetClass,
    pub max_parallel_bulk: u8,
    pub multipath_allowed: bool,
    pub aggressive_discovery_allowed: bool,
}
```

---

# 17. CPU Budget Class

```rust
pub enum CpuBudgetClass {
    Minimal,
    Low,
    Normal,
    High,
}
```

---

# 18. Radio Budget Class

```rust
pub enum RadioBudgetClass {
    Minimal,
    Opportunistic,
    Normal,
    Aggressive,
}
```

---

# 19. Background Budget Class

```rust
pub enum BackgroundBudgetClass {
    CriticalOnly,
    Deferred,
    Limited,
    Normal,
}
```

---

# 20. Hard Safety vs Power Preference

Battery policy may reduce throughput.

It must not violate hard protocol/resource safety.

Example:

```text
low battery
```

does not mean:

```text
skip encryption verification
```

Security invariants remain absolute.

---

# 21. Durable Work Preservation

If battery becomes low:

```text
message
file
DTN bundle
```

should not disappear.

State changes:

```text
Active
→ DeferredByPower
```

and resumes later.

---

# 22. Ephemeral Work

Can be dropped:

```text
typing
presence refresh
stale video frames
diagnostic probes
```

when power budget tight.

---

# 23. Scheduler Integration

Every background-capable subsystem should expose:

```text
pause
resume
reduce_concurrency
change_priority
```

through a common scheduling interface.

---

# 24. Power-Aware Work Descriptor

```rust
pub struct PowerAwareWork {
    pub priority: WorkPriority,
    pub energy_class: EnergyClass,
    pub durable: bool,
    pub deadline: Option<Timestamp>,
    pub deferrable: bool,
}
```

---

# 25. Energy Class

```rust
pub enum EnergyClass {
    Tiny,
    Low,
    Medium,
    High,
    VeryHigh,
}
```

Examples:

```text
delivery ACK → Tiny
text send → Low
photo upload → Medium
5 GB transfer → High
AV1 software transcode → VeryHigh
```

---

# 26. Scheduling Decision

```rust
pub enum PowerDecision {
    Run,
    RunThrottled,
    Defer,
    Drop,
    Reject,
}
```

Durable normal work usually gets:

```text
Run / RunThrottled / Defer
```

not Drop.

---

# 27. Decision Inputs

```text
priority
deadline
battery
charging
thermal
foreground/background
resource pressure
network cost
user policy
```

---

# 28. Priority Interaction

Critical work can override saver policy.

Example:

```text
SOS
```

may run even on low battery.

But it should still use efficient transport ordering.

---

# 29. Critical Battery Reserve

At critically low battery:

```text
preserve energy for:
SOS
small messages
identity/security
delivery ACK
```

Pause:

```text
bulk file
relay bulk
background sync
preview generation
```

---

# 30. Radio Wakeup Cost

Radio activation is expensive.

Prefer:

```text
batch multiple operations
```

into one wake window rather than waking repeatedly.

---

# 31. Wake Window

```rust
pub struct WakeWindow {
    pub started_at: Timestamp,
    pub max_duration: Duration,
    pub work_budget: WorkBudget,
}
```

During window:

```text
send pending small messages
sync receipts
refresh capability
```

---

# 32. Batch Small Work

Useful for:

```text
receipts
presence
small sync
delivery ACK
```

Avoid one radio activation per event.

---

# 33. Network Reuse

If radio already active for:

```text
message send
```

allow queued compatible small work to piggyback.

---

# 34. Discovery Cost

Continuous:

```text
BLE scan
Wi-Fi scan
Wi-Fi Direct discovery
```

can be expensive.

Use duty cycle.

---

# 35. Discovery Duty Cycle

Example:

```text
Foreground:
frequent

Background balanced:
periodic

Saver:
rare

Emergency:
frequent but bounded
```

---

# 36. Proximity Integration

Part 14 should expose one neutral discovery scheduler.

Battery policy decides:

```text
scan interval
scan duration
transport escalation
```

---

# 37. BLE First

For nearby discovery:

```text
BLE
```

is often suitable as low-energy bootstrap.

Then upgrade to:

```text
Wi-Fi Direct/Aware
```

only when data volume justifies.

---

# 38. Avoid Expensive Upgrade for Tiny Payload

For:

```text
1 KB message
```

do not create a high-energy Wi-Fi Direct session if BLE or existing path suffices.

---

# 39. File Scheduling

File transfer power policy:

```text
small file:
run

large file on battery:
limit concurrency

huge file + saver:
defer unless user foreground

charging:
increase parallelism
```

---

# 40. File Hashing

Large local imports may require hashing/encryption.

Run with bounded CPU parallelism.

If background + low battery:

```text
defer preprocessing
```

unless user explicitly requested immediate send.

---

# 41. Chunk Concurrency

Part 05 file transfer effective chunk workers:

```text
negotiated max
∩ resource max
∩ power max
```

---

# 42. Power-Aware Chunk Parallelism

Example:

```text
Charging: 8
High battery: 4
Low battery: 2
Saver: 1
Critical thermal: 1/pause bulk
```

Values require benchmarking.

---

# 43. Storage I/O

Large disk writes also consume power.

Batch metadata writes where safe.

Avoid excessive fsync for ephemeral state.

Preserve durability for accepted durable work.

---

# 44. DTN Scheduling

DTN can be power hungry due to scanning/relay.

Power policy controls:

```text
relay acceptance
scan duty cycle
bundle size
Wi-Fi upgrade
forwarding count
```

---

# 45. DTN Criticality

At low battery:

```text
own critical bundles
delivery ACK
SOS
```

still forwarded.

Drop/defer:

```text
bulk relay files
low-priority third-party relay
```

---

# 46. Charging Relay Preference

Charging devices can advertise coarse:

```text
HighRelayCapacity
```

and accept more DTN work.

Do not reveal exact battery percentage.

---

# 47. Multipath Scheduling

Part 12 multipath uses multiple radios/paths.

Battery policy may force:

```text
Stripe → Single
Redundant → Single
WarmFailover → allowed for active call
```

---

# 48. Multipath Power Rule

Do not use multiple radios merely for small throughput gain.

Require measurable benefit.

---

# 49. Call Scheduling

Calls are interactive and often high priority.

Battery policy should adapt:

```text
video resolution
frame rate
codec complexity
background effects
```

before dropping call.

---

# 50. AV1 Software Encoding

Software AV1 may be CPU intensive.

Power policy should expose:

```text
max encoder complexity
max resolution
max fps
```

or signal media engine to downgrade.

---

# 51. Hardware Codec Preference

On Android, if supported and suitable:

```text
hardware codec
```

should normally be preferred for battery efficiency.

Codec selection still respects negotiated compatibility.

---

# 52. Software Codec Fallback

If only software AV1 available:

```text
allow while foreground/charging
reduce quality when battery/thermal constrained
```

Do not burn battery for unnecessarily high encode settings.

---

# 53. Audio Priority

When power/thermal constrained:

```text
keep audio
reduce/disable video
```

This is a strong graceful-degradation policy.

---

# 54. Media Degradation Ladder

Example:

```text
1080p video
 ↓
720p
 ↓
480p
 ↓
lower fps
 ↓
audio only
```

Actual values negotiated with media subsystem.

---

# 55. Thermal-First Media Control

If thermal state becomes Serious/Critical:

```text
reduce software encode immediately
```

even if battery high.

---

# 56. Background Call

OS may allow active call background with special service/audio session.

Platform adapter reports allowed execution.

Rust scheduler retains media policy.

---

# 57. Background Sync

Batch:

```text
read receipts
own-device sync
capability refresh
```

when OS gives background opportunity.

Do not constantly wake app.

---

# 58. Android Integration

Kotlin/platform layer reports:

```text
BatteryManager
power saver
thermal status
background restriction
foreground service state
network metered
charging
```

Rust owns policy.

---

# 59. Android Work Scheduling

For deferred noncritical work, platform adapter may use:

```text
WorkManager
JobScheduler-related mechanisms
```

where appropriate.

Rust stores durable intent and decides eligibility.

Kotlin schedules execution opportunity.

---

# 60. Android Foreground Service

Use only when feature truly requires it:

```text
active call
user-visible large transfer
emergency relay mode
```

Do not keep permanent foreground service merely to bypass OS policy.

---

# 61. Android Doze

Doze may delay background network.

Durable outbox/DTN state survives.

On next allowed wake:

```text
resume prioritized work
```

---

# 62. Android App Standby

Same principle:

```text
execution opportunities are external
durable intent is internal
```

---

# 63. iOS Integration

iOS adapter reports:

```text
low power mode
thermal
background task availability
network state
```

Rust policy remains common.

---

# 64. iOS Constraints

Do not assume continuous background Bluetooth/Wi-Fi behavior.

Architecture degrades to:

```text
foreground
system-approved background windows
push-assisted Internet wake
```

where allowed.

---

# 65. Desktop Behavior

Desktop on AC power:

```text
Performance/Balanced
```

Laptop on battery:

```text
Balanced/Saver
```

Use OS power source signals where available.

---

# 66. Headless Linux

If plugged into mains:

```text
AlwaysOnNode
```

Still monitor thermal and UPS/battery if available.

---

# 67. UPS-Aware Server

Optional:

```text
mains lost
UPS battery
```

can trigger:

```text
reduce bulk
preserve relay/control
```

for edge nodes.

---

# 68. Power Policy Engine

```rust
pub trait PowerPolicyEngine {
    fn evaluate(
        &self,
        work: &PowerAwareWork,
        state: &PowerState,
        resources: &ResourceSnapshot,
    ) -> PowerDecision;
}
```

Pure/deterministic where possible.

---

# 69. Power Policy Configuration

```rust
pub struct PowerPolicyConfig {
    pub profile: PowerPolicyProfile,
    pub allow_background_bulk: bool,
    pub allow_metered_bulk: bool,
    pub multipath_on_battery: bool,
    pub emergency_override: bool,
}
```

---

# 70. User Preferences

Examples:

```text
Battery saver
Allow background transfers
Use mobile data for files
Allow emergency relay
```

These feed policy.

---

# 71. Application Preferences

ERP may choose:

```text
document sync deferred on battery
```

Messenger may choose:

```text
small messages always immediate
```

---

# 72. Platform Hard Restrictions

If OS says:

```text
background network unavailable
```

policy cannot override.

It must defer.

---

# 73. Policy Layering

```text
OS hard restriction
 ↓
system safety
 ↓
user policy
 ↓
application policy
 ↓
battery profile
 ↓
operation priority
```

---

# 74. Emergency Override Boundaries

Emergency override may:

```text
increase discovery
allow DTN
use metered network
```

only if product/user policy permits.

Do not silently violate explicit privacy or financial constraints unless product has clearly defined emergency consent.

---

# 75. Scheduler Queues

Power-aware queues:

```text
Critical
Interactive
Normal
Bulk
Background
```

Battery policy changes eligibility/quantum, not durable ownership.

---

# 76. Work Aging

Deferred normal work should eventually run when:

```text
charging
foreground
better battery
```

No permanent starvation.

---

# 77. Charging Trigger

When charging starts:

```text
resume deferred:
large transfers
blob preprocessing
index rebuild
backup
```

with resource limits.

---

# 78. Wi-Fi Trigger

When unmetered Wi-Fi appears:

```text
resume allowed bulk
```

if battery policy permits.

---

# 79. Foreground Trigger

User opens app:

```text
increase priority for visible transfer
```

---

# 80. Thermal Recovery Trigger

Thermal returns Nominal:

```text
restore CPU/media concurrency gradually
```

Use hysteresis.

---

# 81. Hysteresis

Do not oscillate between profiles due to tiny battery/thermal changes.

Example:

```text
enter Low at <20%
leave Low at >25%
```

Exact thresholds platform/policy-specific.

---

# 82. Thermal Hysteresis

Similarly:

```text
Critical → Serious → Elevated
```

restore gradually.

---

# 83. Power State Epoch

```rust
pub struct PowerStateEpoch(u64);
```

Increment on meaningful policy state changes.

Schedulers ignore stale decisions.

---

# 84. Event Coalescing

Battery percentage may update frequently.

Do not broadcast every 1% change if policy class unchanged.

Emit only class/policy changes.

---

# 85. Metrics

Track:

```text
work deferred by power
bulk resumed on charge
multipath disabled by saver
DTN scans reduced
media downgraded thermal
battery mode transitions
```

---

# 86. Privacy

Do not export exact:

```text
battery %
charging timestamp
thermal history
```

to peers/telemetry unless needed.

Use coarse classes.

---

# 87. Peer Capability

Peer may advertise:

```text
relay capacity class
```

but not exact battery.

Example:

```text
LowRelayCapacity
NormalRelayCapacity
HighRelayCapacity
```

---

# 88. Power-Aware Relay Selection

If local device has multiple DTN peers:

```text
prefer charging/high-capacity relay
```

all else equal.

---

# 89. Power-Aware Routing

Part 03 path score can include:

```text
energy cost
```

Battery policy adjusts weight.

Saver:

```text
high energy penalty
```

Emergency:

```text
reliability may outweigh energy
```

---

# 90. Power-Aware Multipath

Part 12 benefit threshold increases when battery low.

Meaning:

```text
secondary path must offer bigger benefit
```

to activate.

---

# 91. Power-Aware Discovery Escalation

Normal message:

```text
do not trigger expensive Wi-Fi Direct discovery
```

Low battery:

```text
use known paths only if possible
```

SOS:

```text
allow escalation
```

---

# 92. Power-Aware File Prefetch

Do not automatically download:

```text
full-resolution video
large attachments
```

in background on low battery.

Maybe fetch:

```text
thumbnail
metadata
```

---

# 93. Power-Aware Own-Device Sync

Own-device sync can prioritize:

```text
message metadata
read state
```

before:

```text
large blobs
```

---

# 94. Power-Aware Search/Indexing

Search indexing is background CPU/storage work.

Pause or reduce when:

```text
Saver
Serious thermal
Critical battery
```

Resume when charging.

---

# 95. Power-Aware Backup

Backup may run when:

```text
charging
unmetered
background allowed
```

unless user manually requests now.

---

# 96. User-Initiated Override

If user taps:

```text
Send now
Download now
```

interactive priority can override some saver deferrals.

Still respect:

```text
OS hard restrictions
critical thermal safety
hard resource limits
```

---

# 97. Override Scope

Override only the requested operation.

Do not switch whole runtime to Performance.

---

# 98. Power Budget Accounting

Do not pretend to know exact joules without hardware telemetry.

Use coarse estimated cost classes.

Measure later on representative devices.

---

# 99. Empirical Tuning

Benchmark:

```text
BLE scan duty cycle
Wi-Fi Direct setup
AV1 software encode
hashing
file chunk concurrency
```

on real devices.

Then tune policy.

---

# 100. Device-Specific Quirks

Some Android vendors have aggressive background killing.

Keep device-specific workarounds in platform adapter, not core policy.

---

# 101. Vendor Workarounds

Avoid hard-coding undocumented hacks unless necessary.

Prefer supported OS scheduling APIs.

---

# 102. Battery Optimization Exemptions

Do not require users to disable OS battery optimization as default architecture.

If a specialized emergency/always-on deployment needs it, document explicitly.

---

# 103. Power-Aware Daemon

Desktop daemon can detect laptop battery.

When battery:

```text
pause background relay/bulk
```

while preserving messages.

---

# 104. Multiple Processes

Only central runtime/daemon should make scheduling decisions.

UI should not start separate uncontrolled transfers.

---

# 105. FFI Integration

Host app can report:

```text
foreground/background
power saver
```

through stable API if platform adapter is external.

Rust core decides.

---

# 106. Plugin Limits

Third-party plugin cannot bypass power policy.

It receives:

```text
effective execution budget
```

and bounded queues.

---

# 107. Power Policy for Extensions

Extension declares:

```text
energy class
priority
durable/ephemeral
```

Runtime maps to actual scheduling.

---

# 108. Misbehaving Extension

If plugin continuously requests high-energy work:

```text
quota/resource policy throttles
```

---

# 109. Crash Recovery

Power state itself is mostly ephemeral.

On restart:

```text
requery platform
```

Durable deferred work remains.

---

# 110. Deferred Work Persistence

Persist:

```text
operation state
next eligibility hint
```

where needed.

Do not persist exact memory/runtime scheduler state.

---

# 111. Restart on Low Battery

After crash/restart:

```text
do not immediately resume all pending bulk
```

Evaluate current power policy first.

---

# 112. Thundering Herd

Charging event may make thousands of operations eligible.

Use:

```text
priority
batching
resource admission
jitter
```

---

# 113. Recovery Priority

After restart:

```text
identity/security
SOS
messages
DTN critical
active user transfer
bulk
background
```

---

# 114. Battery Policy and Event Log

Part 04 may record meaningful semantic states:

```text
TransferDeferredByPower
```

if product/UI needs.

Do not journal every battery change.

---

# 115. Diagnostics

Advanced:

```text
Power mode: Saver
Battery: Low
Charging: No
Thermal: Elevated
Bulk workers: 1/4
Multipath: disabled
DTN scan: reduced
```

---

# 116. User-Facing Status

Examples:

```text
Large transfer paused to save battery
Will resume while charging
Video quality reduced because device is hot
```

Keep wording actionable.

---

# 117. Notification Policy

Do not spam users for every power deferral.

Notify only when:

```text
user action required
long-running visible transfer paused
critical feature unavailable
```

---

# 118. Low Battery Emergency UX

If critical battery:

```text
Emergency mode is conserving power for messages and SOS.
```

Optional, only if user has enabled emergency mode.

---

# 119. Thermal UX

If call degrades:

```text
Video quality reduced to cool the device.
```

---

# 120. Tests

Unit:

```text
policy decision
hysteresis
priority override
```

Integration:

```text
file + low battery
call + thermal
DTN + saver
```

Platform:

```text
Android power saver
background restriction
charging change
```

---

# 121. Property Tests

Invariants:

```text
Critical durable work is never silently dropped due only to power policy
hard OS restriction is never overridden
bulk cannot run in CriticalOnly background mode
charging cannot exceed hard resource limits
```

---

# 122. Scenario Tests

Example:

```text
battery High
start 5 GB file
battery Low
→ reduce concurrency

enable Saver
→ pause background chunking

plug charger
→ resume
```

---

# 123. Multipath Test

```text
Wi-Fi + cellular
battery Low
```

Expected:

```text
single path
```

unless emergency/user override.

---

# 124. Call Test

```text
AV1 software call
thermal Serious
```

Expected:

```text
reduce resolution/fps
or audio-only
```

---

# 125. DTN Test

```text
battery Critical
```

Expected:

```text
critical bundles continue
bulk relay stops
```

---

# 126. Discovery Test

Saver mode:

```text
BLE scan duty cycle reduced
Wi-Fi escalation rare
```

---

# 127. Charging Test

Charging begins:

```text
deferred indexing/backups eligible
```

but resource limits still enforced.

---

# 128. Background Test

App background-restricted.

Expected:

```text
noncritical work deferred
```

no busy-loop attempts.

---

# 129. Process-Kill Test

Kill while transfer deferred by power.

Restart.

Expected:

```text
still deferred/eligible based on new state
```

---

# 130. Android Real-Device Tests

Need representative devices for:

```text
power saver
thermal
foreground service
background restrictions
hardware codec behavior
```

Emulator alone is insufficient.

---

# 131. Battery Benchmarks

Measure:

```text
idle with runtime
BLE discovery
DTN mode
file upload
AV1 software call
multipath
```

on real hardware.

---

# 132. Energy Regression Tests

Exact joule CI is difficult.

Track coarse trends in dedicated hardware lab if project matures.

---

# 133. Scheduler Performance

Power policy evaluation should be cheap.

Do not perform expensive calculations on every packet/frame.

Evaluate at:

```text
operation start
state change
periodic coarse interval
```

---

# 134. Media Update Frequency

Media may need more frequent thermal adaptation than files.

Still avoid per-frame policy recomputation.

---

# 135. File Update Frequency

Re-evaluate on:

```text
battery class change
charging change
thermal change
foreground/background
```

---

# 136. DTN Update Frequency

Re-evaluate discovery/relay budget on:

```text
power class
emergency mode
charging
```

---

# 137. Suggested Crate Structure

```text
crates/comm-power/
├── src/
│   ├── lib.rs
│   ├── state.rs
│   ├── profile.rs
│   ├── policy.rs
│   ├── budget.rs
│   ├── work.rs
│   ├── scheduler.rs
│   ├── hysteresis.rs
│   ├── platform.rs
│   ├── diagnostics.rs
│   └── error.rs
└── Cargo.toml
```

Platform adapters:

```text
comm-platform-android
comm-platform-ios
comm-platform-desktop
```

---

# 138. Public API

```rust
let decision = power.evaluate(&work, &power_state, &resources);
```

Most features use domain wrappers:

```text
power.files()
power.dtn()
power.media()
```

---

# 139. Platform Adapter Trait

```rust
pub trait PowerStateProvider {
    fn current_state(&self) -> PowerState;
    fn subscribe(&self) -> PowerStateStream;
}
```

---

# 140. Effective Limit API

```rust
let limits = power.effective_limits(WorkDomain::Files);
```

Could return:

```text
parallelism
background allowed
multipath allowed
```

---

# 141. Initial Production Scope

Implement first:

```text
battery level classes
charging state
thermal state
foreground/background state
Balanced/Saver/Emergency profiles
file concurrency throttling
DTN scan throttling
multipath disable/enable
media quality downgrade hooks
Android power state adapter
desktop power adapter
durable deferred work
```

Defer initially:

```text
fine-grained joule accounting
predictive battery models
vendor-specific ML tuning
```

---

# 142. Implementation Phases

## Phase 1 — State Model

```text
BatteryLevelClass
ChargingState
ThermalState
AppActivityState
```

## Phase 2 — Policy Engine

```text
profiles
PowerDecision
hysteresis
```

## Phase 3 — Files/DTN

```text
parallelism
defer/resume
scan duty cycle
```

## Phase 4 — Multipath/Routing

```text
energy penalty
secondary-path suppression
```

## Phase 5 — Media

```text
AV1 software limits
hardware preference
quality ladder
```

## Phase 6 — Platform Adapters

```text
Android
desktop
iOS
```

## Phase 7 — Hardening

```text
real-device tests
battery benchmarks
thermal scenarios
process-kill recovery
```

---

# 143. Definition of Done

Part 13 is complete when:

- power logic is centralized rather than scattered across features
- battery, charging, thermal, and background state feed one shared policy
- durable work is deferred rather than lost
- stale ephemeral work can be dropped
- file concurrency decreases under low-power/thermal pressure
- multipath can collapse to single-path
- DTN scanning/relay intensity adapts
- critical/SOS traffic retains reserved execution capability
- AV1 software encode can be throttled/downgraded
- hardware codecs can be preferred where appropriate
- call media degrades gracefully before failure
- Android/iOS platform restrictions are treated as hard constraints
- charging/foreground transitions can resume deferred work
- hysteresis prevents mode flapping
- exact battery data is not exposed unnecessarily
- process restart reevaluates current power state before resuming work
- property, scenario, real-device, and thermal tests exist

---

# 144. Relationship to Earlier Parts

Part 13 builds on:

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
```

It directly supports:

```text
14 — Proximity Abstraction
15 — QR / NFC Bootstrap Pairing
16 — Daemon & Headless Runtime
17 — Emergency Priority Architecture
18 — Network Diagnostics & Path Visualization
20 — Embedded Linux Node
22 — Third-Party Protocol Extensions
24 — Plugin / Module Ecosystem
```

---

# 145. Final Principle

The battery-aware scheduler should make this behavior normal:

```text
A user starts a 5 GB transfer.

Battery is high:
→ 4 chunk workers
→ Wi-Fi preferred
→ multipath allowed if useful

Battery drops:
→ 2 workers

Battery saver turns on:
→ 1 worker or pause background transfer

Device gets hot:
→ hashing/AV1 work reduced

User starts an urgent call:
→ call gets priority
→ bulk throttles

Phone is plugged in:
→ deferred bulk/indexing resumes
```

And in emergency mode:

```text
Low battery does not mean "everything stops."

It means:
preserve energy for the work that matters most.
```

That is the core purpose of Part 13: keep the communication platform responsive and resilient without turning background networking, AV1, DTN, or large transfers into a battery-drain problem.
