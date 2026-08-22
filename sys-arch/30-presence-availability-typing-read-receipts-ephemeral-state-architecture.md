# Part 30 — Presence, Availability, Typing, Read Receipts & Ephemeral Realtime State Architecture

## Reusable P2P Communication Platform

**Status:** Architecture specification  
**Part:** 30  
**Primary language:** Rust  
**Primary purpose:** define the ephemeral realtime state plane for presence, availability, typing, recording, delivery/read receipts, device activity, and short-lived conversation state without polluting durable messaging/event-log semantics  
**Primary goals:** low-latency UX, privacy-preserving presence, multi-device aggregation, bounded fan-out, battery-aware updates, graceful offline behavior, deterministic expiry, group scalability, abuse resistance, transport independence, and clean integration with messaging, calls, DTN, E2EE, and diagnostics

---

# 1. Purpose

A polished messenger needs more than durable messages.

Users expect:

```text
online
offline
away
last seen
typing…
recording voice…
delivered
read
active on another device
```

These states are useful, but most of them are:

```text
temporary
best-effort
time-sensitive
not worth durable replication
```

If they are implemented like normal durable messages, the system creates:

```text
unnecessary database writes
large event logs
battery/network overhead
stale states
privacy problems
group fan-out explosions
```

The governing principle is:

> **Durable facts belong in the event/message system; transient hints belong in a separate bounded ephemeral-state plane.**

---

# 2. Architectural Position

```text
                   Dioxus UI
                      │
                      ▼
              Ephemeral State View
                      │
             ┌────────┼────────┐
             │        │        │
         Presence   Typing   Receipts
             │        │        │
             └────────┼────────┘
                      │
              Ephemeral Runtime
                      │
         Capability / Privacy Policy
                      │
                 Peer Transport
          Direct / Relay / Local Path
```

Durable state remains separate:

```text
Message DB
Event Log
Call History
```

---

# 3. Ephemeral vs Durable Matrix

| State | Ephemeral? | Durable? |
|---|---:|---:|
| Online now | yes | no |
| Typing | yes | no |
| Recording voice | yes | no |
| Speaking in call | yes | no |
| Delivery acknowledgement | partly | yes when it affects message state |
| Read receipt | transient transport + durable message state | yes |
| Last seen | derived/optional | maybe bounded metadata |
| User block | no | yes |
| Message sent | no | yes |
| Device revoked | no | yes |

---

# 4. Major State Families

```rust
pub enum EphemeralStateKind {
    Presence,
    Availability,
    Typing,
    VoiceRecording,
    Uploading,
    DeliveryReceipt,
    ReadReceipt,
    ActiveDeviceHint,
    CallAvailability,
}
```

---

# 5. Presence

Presence answers:

```text
is the peer currently reachable/active?
```

It must not imply:

```text
guaranteed delivery
physical location
human attention
```

---

# 6. Presence State

```rust
pub enum PresenceState {
    Offline,
    Connecting,
    Online,
    Idle,
    Away,
    DoNotDisturb,
    Invisible,
    Unknown,
}
```

---

# 7. Unknown vs Offline

Important distinction:

```text
Offline
```

means the system has strong reason to believe peer is not presently reachable.

```text
Unknown
```

means:

```text
no fresh presence data
```

Do not render unknown as confidently offline.

---

# 8. Presence Is Advisory

Message sending should not depend on presence.

Correct:

```text
presence says offline
→ message still goes to outbox/mailbox/DTN according to policy
```

Incorrect:

```text
presence says offline
→ refuse message
```

---

# 9. Availability

Presence and availability are related but different.

Example:

```text
online
but
Do Not Disturb
```

or:

```text
online
but
not accepting calls
```

---

# 10. Availability State

```rust
pub struct AvailabilityState {
    pub presence: PresenceState,
    pub messaging: MessagingAvailability,
    pub calling: CallingAvailability,
}
```

---

# 11. Messaging Availability

```rust
pub enum MessagingAvailability {
    Available,
    Delayed,
    DoNotDisturb,
    Unknown,
}
```

---

# 12. Calling Availability

```rust
pub enum CallingAvailability {
    Available,
    Busy,
    DoNotDisturb,
    CallsDisabled,
    Unknown,
}
```

---

# 13. Presence Scope

Presence can be:

```text
device-scoped
account-aggregated
conversation-scoped
organization-scoped
```

---

# 14. Device Presence

```rust
pub struct DevicePresence {
    pub device: DeviceId,
    pub state: PresenceState,
    pub last_update: MonotonicInstant,
}
```

---

# 15. Account Presence Aggregation

An account may have several devices.

Example:

```text
phone offline
desktop online
tablet idle
```

UI should usually show:

```text
Online
```

because at least one trusted device is reachable.

---

# 16. Aggregation Rule

Example:

```text
if any device Online → Online
else if any Idle → Idle
else if any Away → Away
else if all known Offline → Offline
else → Unknown
```

---

# 17. Invisible Mode

If user selects invisible:

```text
local app may still communicate
```

but peers receive:

```text
no online presence
or deliberately hidden state
```

according to privacy policy.

---

# 18. DND

DND is not offline.

Peer may still receive messages.

It mostly affects:

```text
notifications
calls
typing/read visibility if configured
```

---

# 19. Presence Privacy

User options:

```text
Everyone
Contacts
Verified contacts
Nobody
```

Product may support a smaller set initially.

---

# 20. Per-Peer Presence Privacy

Future advanced setting:

```text
allow presence for selected peers
```

but avoid complex ACL UI in v1 unless needed.

---

# 21. Last Seen

`last_seen` is privacy-sensitive.

Do not derive it simply from every network packet.

---

# 22. Last Seen Policy

Possible:

```rust
pub enum LastSeenVisibility {
    Everyone,
    Contacts,
    Nobody,
}
```

---

# 23. Last Seen Granularity

To reduce tracking precision:

```text
exact minute
coarse time
today
recently
```

Product choice.

---

# 24. Coarse Last Seen

Privacy-friendly examples:

```text
recently
today
this week
```

instead of exact timestamp.

---

# 25. Last Seen Is Optional

The system remains fully functional without storing/displaying it.

---

# 26. Presence Transport

Prefer:

```text
existing secure peer channel
```

or lightweight gossip/presence protocol where appropriate.

Do not create heavyweight connection churn just for presence.

---

# 27. Presence Protocol

Example ALPN/logical protocol:

```text
presence/1
```

---

# 28. Presence Advertisement

```rust
pub struct PresenceAdvertisement {
    pub device: ScopedDeviceId,
    pub state: PresenceState,
    pub revision: u64,
    pub expires_after_ms: u32,
}
```

---

# 29. TTL

Presence must expire automatically.

Example:

```text
online TTL: 30–90 s
```

if no refresh.

Exact values tune by platform/power mode.

---

# 30. No Permanent Online State

If a device crashes:

```text
presence automatically expires
```

without requiring explicit offline packet.

---

# 31. Graceful Offline

On clean shutdown, may send:

```text
Offline
```

but correctness never depends on it.

---

# 32. Presence Heartbeat

Refresh interval should be less than TTL.

Example:

```text
refresh every 20–30 s
TTL 60–90 s
```

Desktop can be more frequent than battery-constrained mobile.

---

# 33. Battery-Aware Presence

Part 13 can reduce frequency.

Example:

```text
foreground: 20 s
background: 60 s
battery saver: 120 s
```

with corresponding TTL adjustment.

---

# 34. Mobile Background

Do not rely on continuous background presence heartbeats.

Android/iOS may suspend the app.

Presence becomes:

```text
Unknown/Offline
```

until push/wake/reconnect.

---

# 35. Push Is Not Presence

Push token availability does not mean:

```text
user online
```

---

# 36. Direct Connection Hint

An active authenticated Iroh connection is useful evidence of reachability.

But still separate:

```text
transport reachable
```

from:

```text
human active
```

---

# 37. Activity State

```rust
pub enum UserActivity {
    Active,
    Idle,
    Background,
    Unknown,
}
```

---

# 38. UI Activity Detection

Use local platform/UI events.

Do not transmit every input event.

---

# 39. Idle Threshold

Example:

```text
5–15 minutes
```

product-configurable.

---

# 40. Typing Indicator

Typing is:

```text
conversation-scoped
short-lived
best-effort
```

---

# 41. Typing State

```rust
pub enum TypingState {
    Started,
    Stopped,
}
```

---

# 42. Typing Event

```rust
pub struct TypingEvent {
    pub conversation: ConversationId,
    pub sender: DeviceId,
    pub revision: u64,
    pub expires_after_ms: u32,
}
```

---

# 43. Never Persist Typing

Do not write typing events into:

```text
SQLite history
offline event log
DTN
backup
```

---

# 44. Typing TTL

Example:

```text
3–8 seconds
```

without refresh.

---

# 45. Typing Start

Send when:

```text
composer transitions empty/not-typing
→ actively typing
```

---

# 46. Typing Refresh

Refresh periodically while typing.

Do not send on every keystroke.

---

# 47. Typing Stop

Send when:

```text
message sent
composer cleared
input loses focus
typing idle timeout reached
```

But TTL remains fallback.

---

# 48. Typing Throttle

Example:

```text
at most 1 update per 2–3 s
```

per conversation/device.

---

# 49. Group Typing

For groups, show:

```text
Alice is typing…
Alice and Bob are typing…
Several people are typing…
```

Do not render huge participant list.

---

# 50. Group Typing Fan-Out

Avoid O(N²) chatter in large groups.

Potential strategies:

```text
gossip topic
group relay
coalesced presence channel
```

depending group architecture.

---

# 51. Typing Privacy

User may disable:

```text
Send typing indicators
```

Receiving typing can remain enabled.

---

# 52. Voice Recording Indicator

When composing a voice message:

```text
Recording audio…
```

can be ephemeral.

---

# 53. Voice Recording State

```rust
pub enum RecordingState {
    Started,
    Stopped,
}
```

---

# 54. Recording Privacy

Optional user setting.

Do not transmit microphone audio until actual voice message/call pipeline does so.

---

# 55. Uploading Indicator

Potential:

```text
Sending a file…
```

but usually local transfer progress is enough.

Remote "uploading" hints should be used sparingly.

---

# 56. Delivery Receipt

Delivery means:

> recipient device/application has durably accepted the message.

Not merely:

```text
transport packet arrived
```

---

# 57. Receipt Stages

```rust
pub enum MessageReceiptState {
    Sent,
    Delivered,
    Read,
}
```

---

# 58. Sent

`Sent` means:

```text
sender successfully committed/transmitted according to sender-side semantics
```

Exact product semantics should align with Part 04/outbox.

---

# 59. Delivered

A robust definition:

```text
recipient persisted message durably
```

Then it can send delivery receipt.

---

# 60. Transport ACK Is Not Delivery Receipt

Do not confuse:

```text
QUIC stream write ACK
```

with:

```text
recipient application stored message
```

---

# 61. Delivery Receipt Event

```rust
pub struct DeliveryReceipt {
    pub conversation: ConversationId,
    pub through_seq: MessageSequence,
    pub receiver_device: DeviceId,
}
```

---

# 62. Cumulative Receipts

Instead of:

```text
one receipt per message
```

prefer:

```text
delivered through sequence N
```

where sequence model permits.

---

# 63. Sparse Receipt

If ordering/gaps exist, use:

```text
through N
+
exceptions
```

or explicit message IDs for unusual cases.

---

# 64. Read Receipt

Read means:

```text
message crossed product-defined read threshold
```

not simply:

```text
app received it
```

---

# 65. Read Threshold

Possible definition:

```text
conversation visible
message rendered/within viewport
app foreground
```

Use a clear rule.

---

# 66. Read-Through Sequence

Prefer:

```rust
pub struct ReadReceipt {
    pub conversation: ConversationId,
    pub through_seq: MessageSequence,
}
```

---

# 67. Read Receipt Privacy

User setting:

```text
Send read receipts
```

If disabled:

```text
do not transmit read status
```

except possibly organization-managed policy if explicitly configured.

---

# 68. Group Read Receipts

Scaling challenge.

For small groups:

```text
per-member read state
```

For large groups:

```text
read count
or
limited visible participants
```

depending product.

---

# 69. Group Receipt Storage

Do not create one DB row per member/message unnecessarily.

Use cumulative state:

```text
member X read through sequence N
```

---

# 70. Receipt Durability

Unlike typing, delivery/read status modifies durable message state.

Therefore:

```text
receipt transport event
```

may be ephemeral in transit, but its resulting state should persist.

---

# 71. Receipt Retry

If receipt packet lost:

```text
next cumulative receipt supersedes it
```

This reduces retry complexity.

---

# 72. Idempotency

Applying same receipt repeatedly must be safe.

---

# 73. Monotonicity

Read state must only move forward.

```text
ReadThrough 105
```

followed by stale:

```text
ReadThrough 99
```

must not regress.

---

# 74. Device-Level Receipts

Multi-device recipient may read on phone but desktop has not.

Logical account read semantics:

```text
if any authorized device reads
→ account considered read
```

is common.

---

# 75. Multi-Device Receipt Aggregation

Maintain device state internally if useful.

Expose account-level result to sender.

---

# 76. Self-Sync

Recipient's other devices should learn:

```text
read through N
```

so unread badges remain consistent.

---

# 77. Own-Device Read Sync

This is durable account/device sync, not peer-facing typing.

---

# 78. Delivery Across Devices

If message delivered to one recipient device:

```text
Delivered
```

may be shown.

If product wants stronger semantics:

```text
delivered to all devices
```

can be advanced detail.

---

# 79. Receipt Privacy Boundary

Sender should not learn:

```text
which exact device read
```

unless product intentionally exposes it.

Prefer account-level receipt.

---

# 80. Active Device Hint

Optional local-account feature:

```text
Active on Desktop
```

for user's own devices.

Do not expose device-level activity to arbitrary contacts by default.

---

# 81. Call Availability

Part 29 can consume:

```text
CallingAvailability
```

to influence UI.

Presence does not guarantee call acceptance.

---

# 82. Busy State

If user is already in a call:

```text
Busy
```

may be advertised according to privacy.

---

# 83. Presence During Calls

Could show:

```text
Online
```

rather than:

```text
In a call
```

unless user explicitly allows call-state visibility.

---

# 84. Sensitive Presence Metadata

Avoid exposing:

```text
screen on
app foreground
device model
network type
exact battery
```

to peers.

---

# 85. Scoped Presence

Remote peers need only semantic state.

---

# 86. Presence E2EE

Presence for trusted peers should travel over authenticated encrypted channels.

Public presence, if ever added, needs a separate privacy/security model.

---

# 87. Presence Authentication

Reject spoofed:

```text
Alice is online
```

unless signed/session-authenticated by Alice's authorized device.

---

# 88. Presence Revision

Each device maintains monotonic:

```text
revision
```

during an incarnation.

Higher revision supersedes lower.

---

# 89. Restart Handling

After process restart, revision can reset only if paired with:

```text
DeviceInstanceId/session incarnation
```

or a fresh presence epoch.

---

# 90. Presence Epoch

```rust
pub struct PresenceEpoch(pub u64);
```

or random session instance ID.

---

# 91. Stale Presence

If old presence arrives after a new epoch:

```text
ignore
```

---

# 92. Ephemeral Envelope

```rust
pub struct EphemeralEnvelope {
    pub kind: EphemeralStateKind,
    pub source: ScopedDeviceId,
    pub revision: u64,
    pub expires_after_ms: u32,
    pub payload: Bytes,
}
```

---

# 93. TTL Validation

Remote cannot set:

```text
24-hour typing TTL
```

Host clamps per event kind.

---

# 94. Max TTL Policy

Example:

```text
typing ≤ 10 s
recording ≤ 15 s
presence ≤ 180 s
```

Exact values product-configured.

---

# 95. Ephemeral State Table

In memory:

```rust
pub struct EphemeralStateStore {
    // key → value + expiry
}
```

---

# 96. No Primary DB Requirement

Most ephemeral state lives:

```text
memory only
```

---

# 97. Optional Bounded Presence Cache

Could persist:

```text
last seen coarse timestamp
```

if feature enabled.

Typing/recording never persisted.

---

# 98. Expiry Scheduler

Use efficient timer wheel/min-heap instead of one Tokio task per ephemeral item.

---

# 99. Timer Wheel

Useful at scale:

```text
presence
typing
group indicators
```

---

# 100. Expiry Event

When TTL ends:

```text
state removed
UI notified
```

No network packet required.

---

# 101. Coalescing

If presence updates:

```text
Online rev 10
Online rev 11
Online rev 12
```

only latest matters.

---

# 102. Bounded Channel

Use bounded async channel for ephemeral updates.

---

# 103. Backpressure Policy

If overloaded:

```text
drop stale typing
coalesce presence
preserve durable receipts
```

---

# 104. Priority

Recommended:

```text
delivery/read receipts
presence
typing
recording
low-value UI hints
```

But durable message/control/call traffic still outranks most ephemeral state.

---

# 105. Traffic Priority Relative to Core

Example:

```text
Call control
Audio
Security
Message delivery
Receipts
Presence
Typing
Bulk files
Background sync
```

Exact ordering depends product, but typing should never delay a message.

---

# 106. Battery Policy

Typing indicator can be suppressed under extreme battery/network constraints.

Read receipts can batch.

Presence heartbeat frequency can reduce.

---

# 107. Metered Networks

Presence overhead is small, but avoid pointless chatter.

Use connection/session reuse.

---

# 108. Offline Behavior

If no live path:

```text
do not queue typing
do not queue old presence
do not queue recording indicator
```

They are stale by the time network returns.

---

# 109. Receipts Offline

Delivery/read receipts are different.

They should be represented durably enough to synchronize later.

---

# 110. Receipt Coalescing Offline

Instead of queueing every read:

```text
store latest read-through sequence
```

Then send one update when connected.

---

# 111. DTN

Do not send:

```text
typing
presence
recording
```

through DTN.

They lose meaning by delayed delivery.

---

# 112. DTN Receipts

Delivery/read receipts can traverse delayed channels if useful, because they represent durable message state.

---

# 113. Relay

Presence can use relay if there is an active lightweight channel.

Do not force relay connection solely for high-frequency typing unless product policy justifies it.

---

# 114. LAN / Local Mesh

Presence/typing work normally over local peer paths without Internet.

---

# 115. Internetless Operation

Two devices on same router can still exchange:

```text
presence
typing
receipts
```

over local transport.

---

# 116. Multi-Path Duplication

Do not blindly send ephemeral state over every path.

Select one control path.

Receipts can be retried/cumulative.

---

# 117. Path Handoff

Presence state survives path changes logically.

Transport path is replaceable.

---

# 118. Group Presence

Large group presence is expensive.

Do not show:

```text
online status for 10,000 members
```

via full mesh fan-out.

---

# 119. Group Presence Policy

Options:

```text
only active conversation participants
recently active subset
server/relay aggregate
none
```

---

# 120. Small Groups

Direct fan-out is acceptable.

---

# 121. Large Groups

Use:

```text
gossip aggregation
group presence service
bounded active-member set
```

if feature is required.

---

# 122. Presence Subscriptions

Only subscribe to peers currently relevant.

Examples:

```text
open conversation
visible contact list
active call setup
```

---

# 123. Do Not Track Every Contact Continuously

This improves:

```text
battery
privacy
network load
```

---

# 124. Lazy Presence

When opening conversation:

```text
subscribe/request fresh presence
```

---

# 125. Contact List Presence

May subscribe only to visible rows.

---

# 126. UI Virtualization Integration

Dioxus contact list:

```text
visible contacts
→ presence subscriptions
```

offscreen contacts can use stale/unknown state.

---

# 127. Subscription Lease

Presence subscription itself can have TTL.

---

# 128. Presence Subscription API

```rust
pub trait PresenceService {
    async fn subscribe(
        &self,
        peer: AccountId,
    ) -> Result<PresenceSubscription, PresenceError>;
}
```

---

# 129. Typing API

```rust
pub trait TypingService {
    async fn set_typing(
        &self,
        conversation: ConversationId,
        active: bool,
    ) -> Result<(), TypingError>;
}
```

---

# 130. Receipt API

```rust
pub trait ReceiptService {
    async fn mark_read_through(
        &self,
        conversation: ConversationId,
        seq: MessageSequence,
    ) -> Result<(), ReceiptError>;
}
```

---

# 131. UI State Model

```rust
pub struct ConversationEphemeralState {
    pub peer_presence: PresenceSummary,
    pub typers: Vec<AccountId>,
    pub recorders: Vec<AccountId>,
    pub read_through: Option<MessageSequence>,
}
```

---

# 132. View Model

Dioxus consumes a read-only projection.

It does not manage TTL itself.

---

# 133. Typing UI

If one peer:

```text
Typing…
```

If multiple:

```text
Alice and Bob are typing…
```

If many:

```text
Several people are typing…
```

---

# 134. Accessibility

Typing indicator should have:

```text
screen-reader label
non-animation fallback
```

---

# 135. Motion

Avoid excessive animated dots if user prefers reduced motion.

---

# 136. Presence UI

Prefer simple:

```text
Online
Away
Last seen recently
```

Avoid misleading precision.

---

# 137. Delivery/Read UI

Potential states:

```text
clock → queued
single check → sent
double check → delivered
colored double check → read
```

Exact iconography is product/UI decision.

Architecture exposes semantics, not icon assumptions.

---

# 138. Group Receipt UI

For small groups:

```text
Seen by Alice, Bob
```

For larger:

```text
Seen by 23
```

---

# 139. Receipt Detail Screen

Optional:

```text
Delivered to
Read by
```

for group messages.

---

# 140. Privacy Setting Changes

If user turns off read receipts:

```text
future receipts stop
```

Do not attempt to retract already disclosed reads.

---

# 141. Typing Setting Changes

Apply immediately.

---

# 142. Presence Setting Changes

Send updated hidden/invisible state or simply stop publishing and let TTL expire.

Prefer explicit privacy-state update when useful.

---

# 143. Blocked Peer

Blocked peer should receive no new:

```text
presence
typing
read receipts
```

unless protocol requires a neutral response.

---

# 144. Block Semantics

Do not reveal:

```text
"You were blocked"
```

through presence behavior more than necessary.

---

# 145. Abuse Resistance

Attackers can abuse ephemeral channels to:

```text
wake device
spam UI
consume CPU
create notification-like effects
```

Use quotas.

---

# 146. Per-Peer Limits

Examples:

```text
typing events/sec
presence updates/min
receipt updates/sec
```

---

# 147. Invalid TTL

Clamp/reject.

---

# 148. Invalid Conversation

Typing for conversation peer is not a member of:

```text
reject
```

---

# 149. Receipt Authorization

Only legitimate recipient/member can send receipt for message/conversation.

---

# 150. Forged Read Receipt

Authentication prevents arbitrary peer from claiming another user's read state.

---

# 151. Receipt Sequence Validation

Receipt cannot acknowledge:

```text
message sequence that never existed
```

without being bounded/ignored.

---

# 152. Future Sequence

If receipt claims far-future sequence:

```text
reject as protocol violation
```

---

# 153. Receipt Regression

Ignore lower sequence than current.

---

# 154. Presence Flood

Coalesce before reaching UI.

---

# 155. Typing Flood

Throttle and collapse to:

```text
typing=true
```

until TTL.

---

# 156. Untrusted Peer Presence

Unknown peers do not get automatic presence subscription.

---

# 157. Presence Discovery vs Contact Discovery

Do not confuse:

```text
discover nearby peer
```

with:

```text
show them permanently online
```

---

# 158. Security

All peer-facing ephemeral state is authenticated.

---

# 159. E2EE

For trusted conversations, ephemeral payloads can use session encryption from Part 28.

---

# 160. Privacy of Read Receipts

Read receipts can reveal behavior/time patterns.

Default should be user-controllable.

---

# 161. Timing Privacy

Optional batching can reduce exact behavioral leakage.

Example:

```text
send read receipt within 0–2 s randomized/coalesced window
```

if product wants.

---

# 162. Last Seen Leakage

Exact last-seen timestamps can enable surveillance.

Prefer coarse display.

---

# 163. Presence Leakage

Do not expose per-device:

```text
phone online
desktop online
```

to contacts by default.

Account-level aggregate only.

---

# 164. Call Presence Leakage

Do not reveal:

```text
currently in a call
```

unless explicit product feature.

---

# 165. Diagnostics

Part 18 adds:

```text
presence subscriptions
presence RTT
typing sends
typing drops/coalesces
receipt queue depth
receipt latency
```

---

# 166. No User Behavioral Logs

Do not persist detailed history:

```text
Alice was typing at 11:03:12
```

for diagnostics.

---

# 167. Developer Metrics

Aggregate:

```text
ephemeral frames sent
coalesced
dropped
expired
```

---

# 168. Receipt Metrics

Can measure:

```text
message delivery latency
read latency
```

locally, but avoid telemetry with identifiable peer unless explicit privacy policy.

---

# 169. Event Types

Internal:

```rust
pub enum EphemeralEvent {
    PresenceChanged,
    TypingChanged,
    RecordingChanged,
    DeliveryAdvanced,
    ReadAdvanced,
}
```

---

# 170. Actor Model

Suggested services:

```text
PresenceActor
TypingActor
ReceiptActor
EphemeralExpiryActor
```

or one bounded `EphemeralRuntime` if simpler.

---

# 171. Avoid Actor Explosion

Do not create one actor per peer/conversation unless measured scale requires it.

---

# 172. Central Ephemeral Runtime

Recommended:

```rust
pub struct EphemeralRuntime {
    // bounded state maps
    // timer wheel
    // subscriptions
    // transport sender
}
```

---

# 173. State Key

```rust
pub enum EphemeralKey {
    Presence(AccountId),
    Typing(ConversationId, AccountId),
    Recording(ConversationId, AccountId),
}
```

---

# 174. Memory Bounds

Limit:

```text
tracked peers
tracked conversations
group typers
presence subscriptions
```

---

# 175. LRU Eviction

If too many inactive presence entries:

```text
evict oldest
```

after TTL/subscription expiry.

---

# 176. Active Conversation Priority

Keep ephemeral state for:

```text
currently open conversation
active call
visible contacts
```

before background contacts.

---

# 177. Persistence Boundary

Only receipt-derived durable state crosses into message DB.

---

# 178. Transactional Receipt Apply

When a read receipt advances:

```text
validate
update message/conversation read projection atomically
emit UI event
```

---

# 179. Receipt Compression

For account sync, store:

```text
conversation_id
read_through
```

not one row for every message.

---

# 180. Device Receipt State

Optionally:

```text
device_read_through
```

for own-device synchronization.

---

# 181. Conversation Sequence

Receipts depend on stable per-conversation message ordering/sequence architecture from messaging core.

---

# 182. If No Contiguous Sequence

Use monotonic logical index assigned by conversation layer.

Do not use wall-clock timestamp.

---

# 183. Edited Messages

Read status remains tied to logical message sequence.

Edit does not make message unread again by default.

---

# 184. Deleted Messages

Receipt progression remains monotonic even if old messages deleted.

---

# 185. Reactions

Do not alter read-through semantics.

---

# 186. Threads

If threaded conversations added:

```text
conversation-level read
thread-level unread
```

may require separate cursors.

---

# 187. Read Cursor

```rust
pub struct ReadCursor {
    pub conversation: ConversationId,
    pub through: MessageSequence,
}
```

---

# 188. Unread Count

Derived:

```text
latest visible sequence - read cursor
```

with deleted/system-message rules.

---

# 189. Mentions

Unread mentions are separate derived projection.

---

# 190. Device Sync

Read cursor sync should be:

```text
max(current, incoming)
```

---

# 191. Conflict Resolution

Monotonic max makes read-state sync simple.

---

# 192. Delivery Cursor

Same concept:

```rust
pub struct DeliveryCursor {
    pub conversation: ConversationId,
    pub through: MessageSequence,
}
```

---

# 193. Partial Delivery

If gaps exist:

```text
through N + gap set
```

temporarily.

Once gaps fill:

```text
advance contiguous cursor
```

---

# 194. Receipt Batching

Batch multiple conversations into one control frame where appropriate.

---

# 195. Batch Limits

Bound:

```text
receipt count
frame bytes
```

---

# 196. Typing Over Slow Link

Typing is first to drop.

---

# 197. Presence Over Slow Link

Reduce heartbeat.

---

# 198. Receipts Over Slow Link

Batch/coalesce but eventually deliver.

---

# 199. Emergency Mode

Part 17 may suppress:

```text
typing
fine-grained presence
```

to conserve bandwidth.

Preserve:

```text
critical delivery/read state only if useful
```

---

# 200. Disaster Mesh

In unstable mesh:

```text
presence accuracy degrades naturally
```

Do not flood mesh trying to maintain perfect online lists.

---

# 201. Partition Semantics

During network partition:

```text
peer state eventually expires
```

to Unknown/Offline.

---

# 202. Merge After Partition

Fresh revision/epoch supersedes stale presence.

---

# 203. Call Reconnect Integration

Part 29 can use presence as a hint, but transport/liveness determines actual reconnect.

---

# 204. Call Busy Integration

Active call controller can expose:

```text
CallingAvailability::Busy
```

if privacy policy allows.

---

# 205. Message Composer Integration

Dioxus composer emits local activity events:

```text
Typed
Cleared
Sent
LostFocus
```

Typing service converts to throttled protocol updates.

---

# 206. Debounce

Do not immediately send typing on every tiny focus event.

Use short debounce.

---

# 207. Recording Composer Integration

Voice recorder lifecycle:

```text
record start
cancel
send
```

maps to recording hints.

---

# 208. Read Detection

UI reports:

```text
highest message sequence truly read
```

to application layer.

Do not let storage layer guess visibility.

---

# 209. Read Detection Conditions

Recommended:

```text
conversation foreground
window/app active
message row visible beyond threshold
```

---

# 210. Scroll Position

If user is reading old history:

```text
do not mark newer offscreen messages read
```

---

# 211. Conversation Open

Opening conversation does not automatically mean all loaded messages read unless viewport rule says so.

---

# 212. Accessibility Cases

Screen-reader navigation should integrate with read semantics carefully.

Do not require visual pixels specifically.

Use:

```text
semantic exposure/read intent
```

where platform can report it.

---

# 213. Desktop Multi-Window

If same conversation open in two windows:

```text
max visible/read cursor
```

wins.

---

# 214. Own Multiple Devices

If phone reads message:

```text
desktop unread badge clears
```

after secure sync.

---

# 215. Receipt Loop Prevention

Own-device sync receipt must not bounce forever.

Use:

```text
origin device
revision
monotonic merge
```

---

# 216. Protocol Version

Dedicated logical protocol:

```text
ephemeral/1
```

or combined control protocol with typed namespaces.

---

# 217. Protocol Separation

Recommended:

```text
presence/1
receipts/1
```

or one:

```text
state/1
```

depending implementation.

Do not mix into raw message payload schema if it creates coupling.

---

# 218. Wire DTOs

Dedicated versioned DTOs:

```text
PresenceFrameV1
TypingFrameV1
ReceiptFrameV1
```

---

# 219. Postcard

Good fit for compact wire encoding.

Do not serialize arbitrary internal structs directly.

---

# 220. Canonical Limits

Define:

```text
max group typers/frame
max receipt batch
max TTL
max peer subscriptions
```

---

# 221. Capability Negotiation

Part 07 advertises:

```text
presence-v1
typing-v1
delivery-receipts-v1
read-receipts-v1
```

---

# 222. Optional Capabilities

If peer does not support typing:

```text
messaging still works
```

---

# 223. Read Receipt Capability

If unsupported/disabled:

```text
stop at Delivered
```

---

# 224. Presence Capability

If unsupported:

```text
Unknown
```

not error.

---

# 225. Interoperability

Part 23 should add test vectors for:

```text
presence TTL
typing expiry
receipt monotonicity
duplicate updates
multi-device aggregation
```

---

# 226. Conformance Tests

Examples:

```text
Typing Started
wait beyond TTL
→ not typing

ReadThrough 20
ReadThrough 17
→ remains 20
```

---

# 227. Fuzzing

Part 10 fuzz:

```text
ephemeral frame parser
TTL values
receipt ranges
group typing sets
revision handling
```

---

# 228. Property Tests

Examples:

```text
read cursor never decreases
delivery cursor never decreases
expired typing never survives
stale presence epoch cannot overwrite fresh
memory use remains bounded
```

---

# 229. Duplicate Test

Same typing/presence/receipt update multiple times.

No duplicate UI artifact.

---

# 230. Reordering Test

Receive:

```text
rev 5
rev 3
rev 4
```

final state remains rev 5.

---

# 231. Presence Loss Test

Drop all heartbeat packets.

State expires.

---

# 232. Clean Shutdown Test

Explicit offline arrives.

UI updates quickly.

---

# 233. Crash Test

No explicit offline.

TTL still clears presence.

---

# 234. Battery Saver Test

Heartbeat interval grows without causing permanent false-online state.

---

# 235. Group Typing Stress

Hundreds of participants.

UI/state remains bounded.

---

# 236. Receipt Storm Test

Read 1,000 messages quickly.

Network should send:

```text
one/few cumulative receipts
```

not 1,000 packets.

---

# 237. Multi-Device Receipt Test

Phone reads through 100.

Desktop reads through 80.

Account state remains:

```text
100
```

---

# 238. Privacy Test

Read receipts disabled.

No read frames emitted.

---

# 239. Block Test

After blocking:

```text
presence/typing/receipts
```

stop according to policy.

---

# 240. Unknown Peer Flood Test

Thousands of presence updates from unknown peer.

Expected:

```text
drop/rate limit
bounded memory
```

---

# 241. Diagnostics Test

No ephemeral user history persists after expiry unless explicitly feature-derived.

---

# 242. Performance Goals

Presence/typing processing should be effectively negligible compared with:

```text
media
file transfer
message encryption
```

---

# 243. Allocation Policy

Reuse small buffers where useful.

Do not overengineer zero-copy for 20-byte typing events.

---

# 244. Serialization Size

Keep typical typing/presence update compact.

---

# 245. Startup

Startup sequence:

```text
load privacy/settings
start ephemeral runtime
connect transport
publish fresh presence when allowed
restore durable receipt cursors
```

Do not restore old typing state.

---

# 246. Shutdown

Optional:

```text
send offline
flush latest durable receipts
```

but correctness relies on TTL and durable receipt state.

---

# 247. Headless Mode

Part 16 daemon can run presence/receipt services without UI.

Typing service may be unused.

---

# 248. Embedded Node

Part 20 node may expose:

```text
Available
GatewayOnline
```

only if product needs peer presence.

Avoid treating infrastructure nodes as human presence.

---

# 249. FFI

Part 19 exposes:

```text
subscribe_presence
set_typing
mark_read
subscribe_receipts
```

at semantic level.

---

# 250. WASM / Plugins

Plugins should not automatically observe:

```text
presence graph
typing
read behavior
```

These are sensitive behavioral signals.

Require explicit permission if ever exposed.

---

# 251. Plugin Permission

Potential future:

```text
ObserveConversationPresence
```

high privacy impact.

Not default.

---

# 252. Telemetry

Do not upload raw:

```text
presence history
typing events
read timestamps
```

to analytics by default.

---

# 253. Suggested Workspace

```text
crates/
├── comm-presence-core/
├── comm-presence-protocol/
├── comm-presence-runtime/
├── comm-typing/
├── comm-receipts/
├── comm-availability/
├── comm-ephemeral-store/
├── comm-ephemeral-diagnostics/
└── comm-ephemeral-testkit/
```

---

# 254. `comm-presence-core`

Owns:

```text
presence types
availability
privacy policy
aggregation rules
```

---

# 255. `comm-presence-protocol`

Owns:

```text
wire DTOs
versioning
TTL limits
revisions
```

---

# 256. `comm-presence-runtime`

Owns:

```text
subscriptions
heartbeats
aggregation
expiry
transport
```

---

# 257. `comm-typing`

Owns:

```text
composer activity
debounce
refresh
TTL
group aggregation
```

---

# 258. `comm-receipts`

Owns:

```text
delivery cursor
read cursor
batch/coalesce
multi-device merge
durable projection updates
```

---

# 259. `comm-availability`

Combines:

```text
presence
DND
call busy
messaging availability
```

---

# 260. `comm-ephemeral-store`

In-memory bounded state plus timer wheel.

---

# 261. `comm-ephemeral-testkit`

Provides:

```text
fake clock
fake transport
multi-device peers
loss/reorder injection
privacy scenarios
```

---

# 262. Public Commands

```rust
pub enum EphemeralCommand {
    PublishPresence(PresenceState),
    SubscribePresence(AccountId),
    UnsubscribePresence(AccountId),
    SetTyping {
        conversation: ConversationId,
        active: bool,
    },
    SetRecording {
        conversation: ConversationId,
        active: bool,
    },
    MarkReadThrough {
        conversation: ConversationId,
        through: MessageSequence,
    },
}
```

---

# 263. Public Events

```rust
pub enum EphemeralEvent {
    PresenceUpdated {
        account: AccountId,
        presence: PresenceSummary,
    },
    TypingUpdated {
        conversation: ConversationId,
        accounts: Vec<AccountId>,
    },
    RecordingUpdated {
        conversation: ConversationId,
        accounts: Vec<AccountId>,
    },
    DeliveryAdvanced {
        conversation: ConversationId,
        through: MessageSequence,
    },
    ReadAdvanced {
        conversation: ConversationId,
        through: MessageSequence,
    },
}
```

---

# 264. Implementation Phases

## Phase 1 — Receipt Cursors

```text
delivery
read
monotonic merge
durable projections
```

## Phase 2 — Typing

```text
debounce
TTL
privacy
```

## Phase 3 — Presence

```text
device state
account aggregation
subscriptions
TTL
```

## Phase 4 — Availability

```text
DND
busy
calling availability
```

## Phase 5 — Multi-Device

```text
read sync
presence aggregation
answered/busy integration
```

## Phase 6 — Group Scaling

```text
typing aggregation
receipt scaling
presence subscriptions
```

## Phase 7 — Battery / Offline Hardening

```text
background
low power
partitions
coalescing
```

## Phase 8 — Privacy / Abuse Hardening

```text
visibility controls
block behavior
rate limits
behavioral-data minimization
```

## Phase 9 — Interoperability

```text
ephemeral/1 vectors
receipt monotonicity
TTL tests
```

---

# 265. Initial Production Recommendation

For v1, ship:

```text
account-level online/offline/unknown
typing indicator
delivery receipts
read receipts
multi-device read sync
privacy toggles
DND/busy availability
```

Defer initially:

```text
precise last-seen timestamps
per-device public presence
large-group full presence lists
complex custom statuses
plugin access to behavioral state
```

---

# 266. Definition of Done

Part 30 is complete when:

- presence is advisory and never blocks message delivery
- unknown and offline are distinct states
- multi-device presence aggregates correctly
- presence expires automatically without graceful shutdown
- mobile/background presence does not require continuous wakeups
- typing/recording indicators are never persisted or sent through DTN
- typing is throttled and TTL-driven
- group typing remains bounded
- delivery means durable recipient acceptance, not transport ACK
- read receipts use monotonic read-through cursors
- delivery/read receipts coalesce instead of generating one packet per message
- receipt state survives reconnect/offline operation
- read receipt privacy can be disabled
- blocked peers stop receiving behavioral state according to policy
- exact last-seen data is optional and privacy-controlled
- ephemeral state cannot consume unbounded memory
- stale revisions/epochs cannot overwrite fresh state
- battery/network pressure can reduce/suppress low-value ephemeral traffic
- call availability integrates with Part 29 without exposing unnecessary call details
- Dioxus consumes semantic projections rather than managing TTL/state correctness
- plugins/WASM do not receive behavioral signals by default
- Part 23 conformance tests cover TTL, monotonic receipts, duplicate/reordered frames, privacy settings, and multi-device aggregation

---

# 267. Relationship to Earlier Parts

Part 30 integrates directly with:

```text
02 — Multi-Device Identity
03 — Transport / Routing Policy
04 — Offline Event Log
07 — Capability Negotiation
08 — Resource Limits
09 — Crash Recovery
10 — Fuzzing / Protocol Tests
12 — Multipath
13 — Battery-Aware Scheduling
14 — Proximity
16 — Daemon / Headless
17 — Emergency Priority
18 — Diagnostics
19 — C ABI / FFI
21 — Third-Party Extensions
22 — WASM Components
23 — Interoperability
24 — Plugin Ecosystem
28 — E2EE / Privacy
29 — Realtime Calls
```

---

# 268. Final Architecture

```text
                    DIOXUS / SDK
                        │
                        ▼
                Ephemeral Projection
             ┌──────────┼───────────┐
             │          │           │
         Presence     Typing     Receipts
             │          │           │
             └──────────┼───────────┘
                        │
                Ephemeral Runtime
            ┌───────────┼────────────┐
            │           │            │
         TTL/Expiry   Privacy      Coalescing
            │           │            │
            └───────────┼────────────┘
                        │
                 Secure Control Path
                        │
             Direct / Relay / LAN
```

Durable boundary:

```text
Typing/Presence
    → memory only

Delivery/Read receipt
    → control message
    → monotonic durable cursor
    → message/conversation projection
```

---

# 269. Final Principle

Ephemeral state should make the application feel alive without turning every user action into durable distributed data.

The right distinction is:

```text
"Typing…"
    → disposable hint

"Online"
    → expiring reachability hint

"Delivered"
    → durable message fact

"Read"
    → durable user-state fact with privacy controls
```

The architecture should therefore optimize for:

```text
freshness
privacy
low overhead
bounded memory
automatic expiry
coalescing
```

rather than perfect historical consistency.

That is the role of Part 30: provide polished realtime UX while keeping the durable messaging, security, offline, and DTN architecture clean and efficient.
