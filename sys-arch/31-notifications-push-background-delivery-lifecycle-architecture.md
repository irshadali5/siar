# Part 31 — Notifications, Push Wake, Background Delivery & OS Lifecycle Architecture

## Reusable P2P Communication Platform

**Status:** Architecture specification  
**Part:** 31  
**Primary language:** Rust  
**Primary purpose:** define a secure, battery-efficient, cross-platform notification and background-delivery architecture for messaging, calls, files, emergency traffic, and system events  
**Primary goals:** wake-only push, local notifications, reliable background synchronization, privacy-preserving notification previews, Android/iOS lifecycle integration, call notifications, foreground-service behavior, offline fetch after wake, notification de-duplication, battery-aware scheduling, and strict separation between push infrastructure and actual encrypted content delivery

---

# 1. Purpose

A production P2P messenger cannot assume that the application process is always running.

On mobile operating systems, the app may be:

```text
foreground
background
suspended
killed
battery-restricted
network-restricted
```

Yet the user still expects:

```text
new-message notification
incoming-call notification
missed-call notification
file-transfer completion
security alerts
emergency alerts
```

The architecture must therefore support background wake and notification delivery without turning push infrastructure into the message transport or weakening E2EE.

The governing principle is:

> **Push wakes the application; it does not carry the authoritative user message.**

---

# 2. Architectural Position

```text
Remote Peer
    │
    ▼
Encrypted Message / Call Event
    │
    ├── Direct P2P if app is reachable
    │
    └── Encrypted mailbox / relay if app sleeps
                     │
                     ▼
              Push Wake Service
                     │
                     ▼
                OS wakes app
                     │
                     ▼
              Rust Background Sync
                     │
                     ▼
          Fetch encrypted pending state
                     │
                     ▼
              Verify / Decrypt / Persist
                     │
                     ▼
              Local Notification
```

---

# 3. Push Is Not Message Delivery

Bad architecture:

```text
push payload contains plaintext message
```

Recommended:

```text
push payload:
    wake reason
    opaque account/device token
    coarse event class
```

Then:

```text
app wakes
→ authenticates
→ fetches encrypted message
→ verifies
→ persists
→ notifies user
```

---

# 4. Why This Matters

If push payload contains message text:

```text
push provider sees content
OS notification stack may expose content
delivery semantics become tied to cloud push
```

Wake-only push preserves:

```text
P2P-first architecture
E2EE
self-hosting options
provider independence
```

---

# 5. Notification Categories

```rust
pub enum NotificationKind {
    Message,
    Mention,
    FileTransfer,
    IncomingCall,
    MissedCall,
    SecurityAlert,
    DeviceAdded,
    DeviceRevoked,
    EmergencyAlert,
    System,
}
```

---

# 6. Notification Importance

```rust
pub enum NotificationImportance {
    Silent,
    Normal,
    High,
    Critical,
}
```

Do not allow arbitrary plugins to request `Critical`.

Part 17/28 policy re-authorizes.

---

# 7. Background Runtime States

```rust
pub enum AppLifecycleState {
    Foreground,
    Background,
    Suspended,
    Terminated,
    HeadlessWake,
}
```

---

# 8. Rust Owns Logical Lifecycle

Platform adapters report:

```text
foreground
background
wake
network restored
power mode changed
```

Rust decides:

```text
what work to run
what queues to flush
whether to notify
```

---

# 9. Platform Adapter Boundary

```rust
pub trait PlatformNotificationBackend {
    async fn show(
        &self,
        notification: LocalNotification,
    ) -> Result<NotificationId, NotificationError>;

    async fn cancel(
        &self,
        id: NotificationId,
    ) -> Result<(), NotificationError>;
}
```

---

# 10. Push Adapter Boundary

```rust
pub trait PushWakeBackend {
    async fn register_device(
        &self,
    ) -> Result<PushWakeToken, PushError>;

    async fn unregister_device(
        &self,
    ) -> Result<(), PushError>;
}
```

---

# 11. Push Token Is Not Identity

Do not equate:

```text
push token
```

with:

```text
AccountId
DeviceId
```

Push tokens are platform/provider routing identifiers.

---

# 12. Push Token Rotation

Mobile push tokens may change.

Rust must support:

```text
register new
invalidate old
publish token change
```

without creating new device identity.

---

# 13. Push Token Privacy

Avoid storing push token in broadly readable peer metadata.

Prefer:

```text
trusted notification service mapping
```

or encrypted/self-hosted routing.

---

# 14. Wake Token

If using a self-hosted wake service, define:

```rust
pub struct WakeToken([u8; 32]);
```

Rotatable and unrelated to device identity.

---

# 15. Push Payload

Keep minimal:

```rust
pub struct PushWakeHint {
    pub version: u8,
    pub reason: WakeReason,
    pub opaque_token: OpaqueWakeToken,
}
```

---

# 16. Wake Reasons

```rust
pub enum WakeReason {
    PendingMessage,
    IncomingCall,
    SecurityEvent,
    EmergencyEvent,
    GenericSync,
}
```

---

# 17. No Sensitive Push Metadata

Avoid putting:

```text
sender
message text
file name
conversation ID
contact name
```

into push payload unless product explicitly accepts the metadata trade-off.

---

# 18. Android Push

Common Android architecture:

```text
push provider
→ platform callback
→ small Kotlin/Java bridge if required
→ Rust wake entrypoint
```

The bridge should pass:

```text
opaque wake reason/token
```

not message content.

---

# 19. Kotlin Role

Kotlin may be needed for:

```text
FCM callback
notification channel creation
foreground-service bootstrap
PendingIntent actions
```

Rust remains authoritative for:

```text
message state
call state
notification policy
deduplication
sync
```

---

# 20. Android Background Execution

Android may restrict arbitrary background execution.

Use the correct platform mechanism:

```text
push callback
foreground service for active call
scheduled background work
short background task
```

depending operation.

---

# 21. Do Not Fight the OS

Do not build:

```text
permanent hidden background loop
```

that tries to bypass Android power management.

This causes:

```text
battery drain
OEM kills
store-policy risk
unreliable behavior
```

---

# 22. Android Foreground Service

Use for legitimate ongoing work such as:

```text
active audio/video call
user-visible transfer if required
```

not for keeping idle messenger permanently alive.

---

# 23. Foreground Service Types

Declare only required Android foreground-service categories.

Release pipeline Part 27 should validate them.

---

# 24. Active Call Service

Part 29 call session may bind to:

```text
foreground service
```

while active.

If UI process/activity recreates:

```text
call controller remains canonical
```

if service/daemon architecture preserves it.

---

# 25. Incoming Call Notification

Flow:

```text
wake hint
 ↓
Rust fetch/validate call offer
 ↓
call policy
 ↓
platform incoming-call notification
```

Do not show ringing UI from unauthenticated push content alone.

---

# 26. Call Validation Before Ring

Before ringing:

```text
authenticate caller
check blocklist
check rate limits
check call expiry
check permissions/policy
```

---

# 27. Notification Actions

Possible:

```text
Reply
Mark Read
Accept Call
Decline Call
Mute
```

Actions route back into Rust commands.

---

# 28. Action Idempotency

Repeated OS action callback must be safe.

Example:

```text
Mark Read twice
```

should not create duplicate events.

---

# 29. Direct Reply

If supporting notification quick reply:

```text
OS input
→ Rust command
→ durable message outbox
→ normal E2EE send
```

Do not send directly from platform callback bypassing message pipeline.

---

# 30. Notification IDs

Stable local ID:

```rust
pub struct NotificationId(u64);
```

Use deterministic grouping/dedup where useful.

---

# 31. Conversation Notification Grouping

Group by:

```text
conversation
```

rather than one notification per message indefinitely.

---

# 32. Notification Collapse

If 20 new messages arrive:

```text
1 conversation notification
```

with summary.

---

# 33. Global Summary

Optional:

```text
5 conversations have unread messages
```

---

# 34. Notification Privacy Modes

```rust
pub enum NotificationPreviewMode {
    Full,
    SenderOnly,
    Generic,
}
```

---

# 35. Full Preview

Example:

```text
Alice: Are you coming?
```

---

# 36. Sender Only

Example:

```text
New message from Alice
```

---

# 37. Generic

Example:

```text
New message
```

---

# 38. Locked vs Unlocked Device

Platform adapter may support:

```text
full preview when unlocked
generic when locked
```

depending OS capability.

---

# 39. Sensitive Conversations

Allow per-conversation:

```text
hide preview
```

---

# 40. Emergency Notifications

Part 17 policy may map verified emergency events to:

```text
High/Critical
```

with stronger attention behavior where platform allows.

---

# 41. Emergency Authenticity

Critical notification must never be produced merely because push payload says:

```text
"emergency"
```

Rust verifies authority signature first.

---

# 42. Security Alerts

Examples:

```text
new device added
device revoked
identity changed
recovery used
```

These may deserve high-priority local notifications.

---

# 43. Notification Sound

Rust policy selects semantic sound category.

Platform backend maps to OS channel/sound.

---

# 44. Vibration

Same semantic model.

---

# 45. Android Notification Channels

Create stable channels:

```text
Messages
Calls
Security
Emergency
Transfers
```

OS/user can customize them.

---

# 46. Do Not Create Endless Channels

Android notification channels persist.

Use a stable small set.

---

# 47. Channel Migration

If channel semantics change:

```text
version carefully
```

but avoid creating new channel every app update.

---

# 48. Notification Badge

Unread count derives from durable conversation/read state.

Do not keep separate badge truth.

---

# 49. Badge Projection

```text
durable unread projection
→ badge count
```

---

# 50. Notification De-Duplication

Same logical event may arrive via:

```text
direct
relay
mailbox
push wake
device sync
```

Use stable event/message/call IDs.

---

# 51. Message Notification Dedup

Key:

```text
MessageId
```

---

# 52. Call Notification Dedup

Key:

```text
CallId
```

---

# 53. Security Notification Dedup

Key:

```text
SecurityEventId
```

---

# 54. Notification State Store

Small local durable store:

```rust
pub struct NotificationProjection {
    pub source_id: EventId,
    pub notification_id: NotificationId,
    pub shown: bool,
    pub dismissed: bool,
}
```

---

# 55. Why Persist Some Notification State

OS/app process may restart.

Persist enough to avoid:

```text
showing same security alert repeatedly
```

---

# 56. Do Not Persist Ephemeral UI Details

No need to store:

```text
animation state
notification expanded state
```

---

# 57. Background Sync Trigger

Wake event should enqueue:

```text
BackgroundSyncReason
```

not perform large logic in OS callback.

---

# 58. Background Sync Reason

```rust
pub enum BackgroundSyncReason {
    PushWake(WakeReason),
    NetworkRestored,
    ScheduledMaintenance,
    UserAction,
    AppForegrounded,
}
```

---

# 59. Background Sync Scheduler

```rust
pub struct BackgroundSyncScheduler {
    // coalesces requests
}
```

---

# 60. Coalescing

If 10 pushes arrive quickly:

```text
run one sync
```

not 10 independent fetches.

---

# 61. Sync State

```rust
pub enum BackgroundSyncState {
    Idle,
    Scheduled,
    Running,
    Backoff,
}
```

---

# 62. Sync Work

Typical wake sync:

```text
open local DB
load identity
establish transport/mailbox connection
fetch pending envelopes
verify
decrypt
persist
advance receipts
create notifications
flush outbox if allowed
close/idle
```

---

# 63. Minimal Wake

Do not start:

```text
camera
video decoder
plugin marketplace
full search indexing
```

for a simple message wake.

---

# 64. Wake Budget

OS may give short execution window.

Prioritize:

```text
security
message persistence
call signaling
emergency
```

before secondary tasks.

---

# 65. Background Work Priority

```text
1. Incoming call
2. Emergency/security
3. Pending message fetch
4. Receipts/outbox
5. File continuation
6. Maintenance
```

---

# 66. File Transfers in Background

Large transfers should obey platform rules.

Possible:

```text
pause
foreground-service transfer
resume later
```

depending user intent and OS.

---

# 67. User-Initiated Transfer

If user explicitly starts large upload/download:

```text
background continuation
```

may be justified.

---

# 68. Opportunistic Background Sync

Do not keep radio awake for low-value background cache cleanup.

---

# 69. Battery-Aware Scheduling

Part 13 applies.

Examples:

```text
battery saver
→ skip thumbnail prefetch

critical battery
→ only message/call/security/emergency
```

---

# 70. Network-Aware Background Sync

On metered/cellular:

```text
fetch message metadata/content
delay large attachments
```

according to auto-download policy.

---

# 71. Offline Wake

Push may arrive but network unavailable.

Store:

```text
pending sync intent
```

and retry when network returns.

---

# 72. Retry

Use:

```text
bounded exponential backoff + jitter
```

---

# 73. Retry Is Not Notification Spam

Do not show repeated:

```text
sync failed
```

notifications unless user action is needed.

---

# 74. Network Restored Event

Platform/network monitor triggers:

```text
resume background sync
flush outbox
```

---

# 75. Mailbox Fetch

If using encrypted mailbox:

```text
wake
→ fetch opaque envelopes
→ verify/decrypt
→ persist
```

---

# 76. Mailbox Cursor

Use durable cursor/checkpoint.

---

# 77. Duplicate Mailbox Envelope

Message ID dedup prevents duplicate UI notification.

---

# 78. Push Loss

Push delivery is not guaranteed.

Therefore the app also syncs on:

```text
foreground
network reconnect
periodic allowed background opportunity
peer reconnect
```

---

# 79. Push Delay

A delayed push should not cause stale typing/call offer to appear.

Call offer expiry and ephemeral TTL still apply.

---

# 80. Push Provider Outage

Messaging still works when:

```text
app already active/reachable
LAN/direct path available
manual foreground sync
```

Push is acceleration, not correctness.

---

# 81. Self-Hosted Push/Wake

For sovereign deployment, architecture may support:

```text
self-hosted wake gateway
```

where platform allows.

On Android, device push integration may still rely on OS/provider constraints depending app distribution/device environment.

---

# 82. Desktop Notifications

Desktop apps usually do not need cloud push.

If daemon/app running:

```text
peer event
→ Rust runtime
→ OS notification
```

---

# 83. Desktop Background

Options:

```text
tray app
daemon
system service
```

according to product.

---

# 84. Linux Notification Backend

Use desktop notification service where available.

Headless node may use:

```text
logs
admin alerts
email/webhook plugin
```

instead of desktop popups.

---

# 85. Windows Notification Backend

Use native Windows notification APIs through Rust/platform adapter.

---

# 86. macOS Notification Backend

Use native notification center APIs through Rust/Apple adapter.

---

# 87. iOS Architecture

If iOS is added:

```text
APNs wake/notification
background execution limits
CallKit integration
```

behind Rust-owned state/policy.

---

# 88. Browser Notifications

If web-compatible host exists:

```text
Service Worker
Web Push
Notification API
```

can map into same semantic layer.

Browser remains a separate platform adapter.

---

# 89. Headless Notifications

Daemon can emit:

```text
admin event
local IPC event
system journal
optional webhook/email extension
```

No GUI assumption.

---

# 90. Embedded Linux Node

Emergency/edge node may drive:

```text
LED
buzzer
local display
admin alert
```

through Part 20 platform adapters.

---

# 91. Notification Policy Engine

```rust
pub trait NotificationPolicyEngine {
    fn decide(
        &self,
        event: &NotificationCandidate,
        ctx: &NotificationContext,
    ) -> NotificationDecision;
}
```

---

# 92. Notification Candidate

```rust
pub struct NotificationCandidate {
    pub kind: NotificationKind,
    pub source_id: EventId,
    pub conversation: Option<ConversationId>,
    pub sender: Option<AccountId>,
    pub urgency: NotificationImportance,
}
```

---

# 93. Notification Context

Includes:

```text
app foreground
conversation visible
DND
notification preview mode
peer muted
OS lock state if available
battery
```

---

# 94. Suppress When Conversation Open

If user already viewing conversation:

```text
do not show duplicate OS notification
```

Maybe use:

```text
in-app sound/badge
```

depending settings.

---

# 95. Muted Conversation

Muted conversation:

```text
persist unread state
```

but suppress sound/pop-up according to mute policy.

---

# 96. Mention Override

Group mention may override ordinary mute if user config allows.

---

# 97. DND

App-level DND can suppress:

```text
messages
calls
```

except explicitly allowed categories.

---

# 98. OS DND

Respect OS-level notification controls.

Do not attempt to bypass.

---

# 99. Emergency Exceptions

Only use critical/override OS pathways where:

```text
platform permits
user/admin policy permits
authority is verified
```

---

# 100. Notification Candidate Pipeline

```text
Durable Event
   ↓
Deduplicate
   ↓
Privacy Policy
   ↓
Mute/DND Policy
   ↓
Foreground Visibility Check
   ↓
Importance Mapping
   ↓
Platform Notification
```

---

# 101. Notification Content Builder

Separate semantic event from presentation.

```rust
pub trait NotificationRenderer {
    fn render(
        &self,
        candidate: &NotificationCandidate,
        privacy: NotificationPreviewMode,
    ) -> LocalNotification;
}
```

---

# 102. Local Notification

```rust
pub struct LocalNotification {
    pub title: String,
    pub body: String,
    pub importance: NotificationImportance,
    pub group: Option<NotificationGroupKey>,
    pub actions: Vec<NotificationAction>,
}
```

---

# 103. Sensitive Text

Title/body are constructed only after:

```text
E2EE verification/decryption
privacy policy
```

---

# 104. Generic Fallback

If message cannot yet be decrypted but wake is trusted:

```text
New message
```

may be shown.

---

# 105. Decryption Failure

Do not show corrupted ciphertext contents.

Show:

```text
Unable to process message
```

only if user action needed.

---

# 106. Notification Click

Click opens semantic destination:

```text
conversation
call screen
security center
transfer screen
```

---

# 107. Deep Link

Use internal typed destination:

```rust
pub enum NotificationDestination {
    Conversation(ConversationId),
    Call(CallId),
    SecurityEvent(SecurityEventId),
    Transfer(TransferId),
}
```

---

# 108. Do Not Trust Raw Intent Strings

Platform intent/action payloads are untrusted input.

Validate IDs and action type.

---

# 109. Cold Start from Notification

Flow:

```text
OS launches app
 ↓
platform adapter parses intent
 ↓
Rust validates destination
 ↓
runtime initializes
 ↓
navigation occurs
```

---

# 110. Cold Start Race

Notification click may arrive before DB/runtime ready.

Queue:

```text
PendingLaunchAction
```

until startup reaches appropriate phase.

---

# 111. Startup Phases

```text
platform bootstrap
secure store
DB
identity
runtime
navigation
```

---

# 112. Background Headless Entry

Push callback should be able to start a lightweight Rust background entrypoint without rendering full Dioxus UI.

---

# 113. Headless Wake Runtime

```rust
pub struct HeadlessWakeRuntime {
    // minimal services
}
```

---

# 114. Headless Services

Load only:

```text
identity
secure store
network
mailbox
message persistence
call signaling
notification policy
```

---

# 115. Do Not Start UI

No Dioxus initialization required for headless wake if platform integration permits.

---

# 116. Dioxus Reconciliation

When UI later opens:

```text
read durable DB projections
```

and naturally reflects messages received during background wake.

---

# 117. Background Call Offer

If call offer arrives during headless wake:

```text
verify
persist minimal call state
show incoming-call notification
```

---

# 118. Call Accept from Notification

Action:

```text
Accept
```

may need to start:

```text
foreground call service
audio runtime
Dioxus/Call UI
```

depending platform.

---

# 119. Call Decline

Can often complete headlessly:

```text
send reject
persist history
cancel notification
```

---

# 120. Missed Call

If call expires before answer:

```text
cancel incoming-call UI
show missed-call notification
```

if policy permits.

---

# 121. Notification Cancellation

When message read elsewhere:

```text
cancel/update local notification
```

through multi-device read sync if appropriate.

---

# 122. Notification Synchronization Across Devices

Do not try to remotely command OS notification trays directly.

Sync durable state:

```text
message read
call answered elsewhere
```

then each device updates its own local notification UI.

---

# 123. Answered Elsewhere

Part 29:

```text
phone ringing
desktop answers
```

Phone receives:

```text
AnsweredElsewhere
```

then cancels call notification.

---

# 124. Read Elsewhere

If desktop reads conversation:

```text
phone may clear notification
```

after read sync.

---

# 125. Notification Race

Message arrives and user opens conversation at same time.

Use durable read state + notification dedup to avoid stale popup.

---

# 126. Debounce Notification

A short delay can allow:

```text
message arrives
conversation already open
```

to suppress unnecessary OS notification.

Keep delay tiny.

---

# 127. Notification Rate Limits

Prevent peer from generating unlimited notifications.

Per peer/conversation quotas.

---

# 128. Unknown Peer Notification Policy

Unknown sender may get:

```text
silent request notification
```

rather than normal alert.

---

# 129. Spam Conversation

If repeated notifications from same unknown peer:

```text
collapse
rate-limit
```

---

# 130. File Transfer Notifications

Examples:

```text
Download complete
Upload failed
Storage full
```

Only user-relevant events.

---

# 131. Transfer Progress

Avoid per-percent OS notifications.

Use:

```text
persistent progress notification
```

only for long-running explicit transfer if platform UX warrants it.

---

# 132. Background Transfer Failure

If automatic background download fails:

```text
usually no user notification
```

unless user explicitly initiated it.

---

# 133. Security Notification Priority

Examples:

```text
new device
identity reset
recovery used
```

should be more prominent than ordinary file completion.

---

# 134. Notification Storage Limits

Keep bounded notification projection.

Old notification state can be pruned.

---

# 135. Pruning

Safe to remove state for:

```text
old dismissed ordinary notifications
```

after retention period.

Keep durable security audit separately.

---

# 136. Notification Actions Security

Action token should bind:

```text
notification ID
event ID
action type
expiry
```

to prevent forged external intents.

---

# 137. Android PendingIntent Safety

Use immutable/mutable flags correctly and explicit intents where possible.

Keep platform details inside adapter.

---

# 138. External Intent Attack

Do not let arbitrary third-party app trigger:

```text
send message
accept call
mark arbitrary message read
```

through exported components.

---

# 139. Exported Components

Part 27 release audit validates only necessary exported Android components.

---

# 140. Background Service Authentication

Internal IPC between platform service and Rust runtime should be local and authenticated by process/app boundary.

---

# 141. Plugin Notifications

Plugins may request notifications only through permissioned semantic API.

---

# 142. Plugin Permission

```text
RegisterNotifications
```

from Parts 21/24.

---

# 143. Plugin Priority Limit

Plugin cannot produce:

```text
Critical emergency
Security alert
Incoming call
```

unless explicitly authorized.

---

# 144. Plugin Notification Rate

Bound per plugin.

---

# 145. WASM Notification API

WASM receives semantic:

```text
request-notification
```

host applies:

```text
permission
rate limit
privacy
importance clamp
```

---

# 146. Notification Telemetry

Do not send:

```text
notification content
sender names
message text
```

as analytics.

Aggregate metrics only if enabled.

---

# 147. Useful Metrics

```text
push wake received
sync started
sync succeeded
message notification shown
call notification shown
notification deduped
```

---

# 148. Push Provider Diagnostics

Track:

```text
token registration failure
wake delay class
provider unavailable
```

without logging token.

---

# 149. Token Logging

Never log full push token.

Use redacted hash if troubleshooting requires identity.

---

# 150. Wake Latency

Measure:

```text
remote event
→ wake
→ fetch
→ notification
```

with privacy-safe local telemetry.

---

# 151. Notification Latency Budget

For ordinary messages:

```text
seconds acceptable
```

For incoming calls:

```text
much lower
```

Call wake path gets higher priority.

---

# 152. Incoming Call Fast Path

```text
push wake
→ validate offer
→ ring
```

Skip low-priority sync.

---

# 153. Emergency Fast Path

```text
wake
→ verify authority/signature
→ persist
→ alert
```

before background maintenance.

---

# 154. Security Fast Path

```text
wake
→ verify security event
→ persist
→ alert
```

---

# 155. Message Fetch Batching

If multiple messages pending:

```text
fetch in batch
persist transactionally
build grouped notification
```

---

# 156. Notification Summary

Example:

```text
Alice: 3 new messages
```

instead of 3 separate popups.

---

# 157. Foreground In-App Notifications

When app foreground:

```text
toast/banner
conversation update
sound
```

may replace OS notification.

---

# 158. In-App Banner

Dioxus UI component consumes:

```text
NotificationCandidate
```

after policy decides OS/in-app presentation.

---

# 159. Dioxus Is Presentation Only

It does not own:

```text
notification dedup
unread truth
push token
background sync
```

---

# 160. Notification Center State

App may show:

```text
security events
missed calls
transfer alerts
```

from durable projections, not OS tray introspection.

---

# 161. Badge Count

Derived from:

```text
unread conversations
pending calls/security alerts
```

according to product.

---

# 162. Notification Mute

Per conversation:

```text
1 hour
8 hours
1 week
forever
```

stored durably.

---

# 163. Mention Exceptions

Optional.

---

# 164. Call Mute

User may allow:

```text
messages muted
calls still ring
```

or mute both.

Separate settings.

---

# 165. Emergency Exceptions

User/admin policy determines whether emergency alerts bypass ordinary mute.

---

# 166. Quiet Hours

Optional app-level schedule.

```rust
pub struct QuietHours {
    pub start: LocalTime,
    pub end: LocalTime,
}
```

---

# 167. Time Zone Changes

Quiet hours use local time zone.

Recalculate on zone change.

---

# 168. Critical Security Events During Quiet Hours

May override quiet hours if user policy permits.

---

# 169. Background Maintenance

Tasks such as:

```text
cache cleanup
expired DTN GC
plugin update check
```

should not be tied to push notifications.

Use platform background scheduler where available.

---

# 170. Work Classes

```rust
pub enum BackgroundWorkClass {
    Urgent,
    UserVisible,
    Sync,
    Maintenance,
}
```

---

# 171. Urgent

```text
incoming call
emergency
security
```

---

# 172. UserVisible

```text
user-started file transfer
```

---

# 173. Sync

```text
messages
receipts
outbox
```

---

# 174. Maintenance

```text
cache cleanup
index compaction
update checks
```

---

# 175. Scheduler Policy

OS/platform adapter maps semantic work class to:

```text
foreground service
background task
job scheduler
push wake
```

---

# 176. Android WorkManager

If used, treat it as platform scheduling implementation detail.

Rust should receive:

```text
BackgroundWorkClass
```

rather than depend on WorkManager types throughout core.

---

# 177. iOS Background Tasks

Same abstraction.

---

# 178. Desktop Scheduler

Can use Tokio/service timer because desktop daemon is typically long-lived.

---

# 179. Embedded Scheduler

Part 20 can run directly under systemd/runtime.

---

# 180. Background Work Idempotency

Every background task should be safe to restart.

---

# 181. Crash During Sync

Durable message/event pipeline ensures:

```text
resume
dedup
```

on next wake.

---

# 182. Crash During Notification Build

Message already persisted.

Next startup can reconstruct whether notification still needed.

---

# 183. Notification Candidate Reconstruction

From durable state:

```text
unread message
not yet notified
```

can regenerate candidate.

---

# 184. Notification Exactly-Once Is Not Required

OS notifications are best-effort UI.

Correctness is:

```text
message persisted once
```

Notification may be:

```text
shown once ideally
```

but duplicate suppression is a UX concern, not data-integrity authority.

---

# 185. Scheduled Reminder Notifications

If product later supports reminders:

```text
separate scheduled-notification subsystem
```

not mixed with push wake.

---

# 186. Notification Localization

Platform-visible text should support localization.

Rust semantic events use localization keys.

---

# 187. Localization Key

```rust
pub struct NotificationTextKey(&'static str);
```

---

# 188. Localized Rendering

Platform/UI layer resolves:

```text
locale
pluralization
```

---

# 189. Sensitive Localization

Do not accidentally include raw message text in generic privacy mode.

---

# 190. Notification Accessibility

Ensure:

```text
clear title
meaningful action labels
screen-reader compatibility
```

---

# 191. Notification Action Count

Keep small.

Too many actions reduce clarity.

---

# 192. Incoming Call UX

Actions:

```text
Accept
Decline
```

Maybe:

```text
Message
```

later.

---

# 193. Message UX

Actions:

```text
Reply
Mark Read
```

where supported.

---

# 194. Security Alert UX

Action:

```text
Review
```

not generic dismiss-only if significant.

---

# 195. Emergency Alert UX

Action:

```text
View details
Acknowledge
```

only if protocol semantics support it.

---

# 196. Notification Sound Privacy

Custom contact names in spoken notifications may expose private data.

Leave to OS/user settings.

---

# 197. Headset Notification

Do not route message audio over call audio incorrectly.

Platform audio focus policy handles.

---

# 198. Notification During Call

Ordinary message notification should be subtle.

Do not interrupt audio path unnecessarily.

---

# 199. Emergency During Call

Policy may elevate.

Still avoid crashing/restarting media pipeline.

---

# 200. Notification During Screen Share

Privacy-sensitive.

Option:

```text
hide message previews while screen sharing
```

---

# 201. Screen-Share Privacy Mode

Part 29 can signal:

```text
screen_share_active
```

Notification policy switches to:

```text
Generic
```

automatically if enabled.

---

# 202. Lock-Screen Privacy Mode

Same concept.

---

# 203. App Lock

If app has biometric/PIN lock:

```text
notification preview may automatically downgrade
```

---

# 204. Background Message Decryption

If device is locked but secure keystore permits decryption:

```text
message can persist
```

Whether preview is shown depends privacy mode.

---

# 205. Hardware Key Requires Unlock

If key unavailable until user unlocks:

```text
store ciphertext
show generic notification
decrypt later
```

---

# 206. Security Trade-Off

High-security profile may intentionally sacrifice rich background previews.

---

# 207. Push Wake Registration

At startup/account login:

```text
obtain token
bind to DeviceId securely
publish to wake service
```

---

# 208. Token Binding

Wake service mapping:

```text
opaque device routing ID
→ push token
```

should not require conversation plaintext.

---

# 209. Token Revocation

On logout/device revocation:

```text
unregister token
```

---

# 210. Stale Token Cleanup

Wake service removes invalid provider tokens.

---

# 211. Multi-Device Wake

Message for account may wake:

```text
all active authorized devices
```

or only devices requiring delivery.

---

# 212. Device Delivery Policy

If server/mailbox already knows device fan-out:

```text
wake each pending recipient device
```

---

# 213. Avoid Wake Storm

Coalesce multiple pending messages per device.

---

# 214. Wake Suppression

If device has active P2P connection:

```text
no push needed
```

unless delivery/liveness fails.

---

# 215. Online Heuristic

Presence is advisory.

Wake service may suppress push only when it has stronger short-lived evidence that device is reachable.

---

# 216. Push Race

Direct message arrives just before push.

When push wakes:

```text
sync finds nothing
```

This is fine.

---

# 217. Push Duplicate

Multiple pushes for same pending state:

```text
coalesced background sync
```

---

# 218. Push Authentication

Push payload itself may not be strongly confidential.

Treat as untrusted wake hint.

Actual fetched content is authenticated by Part 28.

---

# 219. Malicious Push Provider

Worst case should be:

```text
spurious wakes
```

not:

```text
forged message content
forged call identity
```

Rate-limit wake processing.

---

# 220. Wake Abuse Protection

Bound:

```text
sync frequency
CPU
network
notification creation
```

even if push provider or token abused.

---

# 221. Notification Flood Defense

Per:

```text
peer
conversation
plugin
event class
```

rate limits.

---

# 222. Notification Coalescing Windows

Example:

```text
message group within 1–3 s
```

depending UX.

---

# 223. Call Notifications Are Not Coalesced Like Messages

Every valid incoming call is distinct by `CallId`.

---

# 224. Security Alerts

Should not be hidden inside ordinary message summary.

---

# 225. Emergency Alerts

Separate channel/group.

---

# 226. Durable Notification Preferences

Store:

```text
global notifications enabled
message sound
call sound
security alerts
preview mode
quiet hours
per-conversation mute
```

---

# 227. Preference Sync

Some preferences can sync across user's devices.

OS-specific channel state does not.

---

# 228. OS Preference vs App Preference

OS can disable notifications independently.

App should detect/report:

```text
notifications disabled at system level
```

where platform allows.

---

# 229. Notification Permission

Android/iOS may require user permission.

Rust receives:

```rust
pub enum NotificationPermission {
    Granted,
    Denied,
    NotDetermined,
}
```

---

# 230. Permission UX

Ask when user reaches a feature that benefits from notifications.

Avoid unnecessary first-launch prompt if product UX prefers contextual permission request.

---

# 231. Denied Permission

Messaging still works.

App may show in-app explanation/settings path.

---

# 232. Incoming Calls Without Notification Permission

Platform behavior may be constrained.

Architecture should expose:

```text
CallingAvailability::NotificationsUnavailable
```

internally if this affects reachability.

---

# 233. Notification Health

Diagnostics:

```text
permission
push token registered
last wake
last sync
OS notifications enabled
```

---

# 234. User-Facing Doctor

Example:

```text
Notifications are disabled by Android settings.
Incoming calls may not ring while the app is closed.
```

---

# 235. Developer Diagnostics

Include:

```text
wake provider
token age
background restriction
battery optimization status
last successful background fetch
```

where platform permits.

---

# 236. OEM Battery Restrictions

Some Android OEMs aggressively kill background apps.

Do not rely on undocumented hacks.

Diagnostics may explain:

```text
system battery restriction may delay notifications
```

---

# 237. Battery Optimization Exemption

Do not request exemption by default.

Only if justified by product functionality and platform policy.

---

# 238. Foreground Service User Visibility

Foreground service must have truthful user-visible notification where required.

---

# 239. Background Transfer Service

If user explicitly starts long transfer, foreground service may be appropriate.

---

# 240. Notification Channels and Rust IDs

Map stable semantic:

```text
Message
Call
Security
Emergency
Transfer
```

to platform-specific identifiers.

---

# 241. Platform Mapping Table

```rust
pub struct PlatformNotificationChannelMap {
    // semantic kind → platform channel ID
}
```

---

# 242. No Platform IDs in Domain Core

Domain code should not know:

```text
"messages_v3"
```

Android channel string.

---

# 243. Notification Serialization

Internal IPC can use:

```text
Postcard
```

for compact typed messages.

Push payload may use provider-required format.

---

# 244. JSON Push Wrapper

If provider requires JSON:

```text
outer push envelope JSON
inner opaque wake token
```

No plaintext message content.

---

# 245. Background Sync Protocol

Use existing:

```text
mailbox/sync
```

protocols.

Do not invent push-specific message retrieval protocol unnecessarily.

---

# 246. Notification Database Tables

Possible:

```text
notification_projection
notification_preferences
push_registration
background_sync_checkpoint
```

---

# 247. `push_registration`

Stores:

```text
provider
token reference/encrypted token
registered_at
last_verified
```

---

# 248. Token Storage

Treat as sensitive metadata.

Encrypt at rest if practical.

---

# 249. Notification Projection Retention

Short retention for ordinary notifications.

Longer security audit is separate.

---

# 250. Background Sync Checkpoint

Durable:

```text
mailbox cursor
outbox state
sync epoch
```

---

# 251. Crash Recovery

Part 09 applies:

```text
persist before notify
```

---

# 252. Persist Before Notification

Correct order:

```text
receive
verify
decrypt
persist
commit
then notify
```

Never show a message notification for data not durably accepted.

---

# 253. Why Persist First

If app crashes after notification but before DB write:

```text
user taps notification
message missing
```

Persist-first avoids this.

---

# 254. Read Action

Notification "Mark Read":

```text
advance durable read cursor
then cancel/update notification
```

---

# 255. Reply Action

Notification reply:

```text
persist outbound message
then attempt send
```

---

# 256. Call Accept

Incoming call accept is different:

```text
validate call still live
then transition Part 29 state
```

---

# 257. Expired Call Action

If user taps Accept after call expired:

```text
show missed/ended state
```

not attempt stale connection.

---

# 258. Emergency Acknowledge

If protocol supports ACK:

```text
persist ACK intent
send securely
```

---

# 259. Notification Testing

Required:

```text
foreground
background
suspended
cold start
process killed
network offline
battery saver
multiple messages
multi-device read
call answered elsewhere
```

---

# 260. Android Device Matrix

Test:

```text
Pixel-like stock Android
Samsung
Xiaomi/Redmi
OnePlus/Realme
Motorola
```

because background behavior varies.

---

# 261. Push Delay Test

Send 100 wake events under:

```text
screen off
Doze
battery saver
Wi-Fi
cellular
```

measure latency distribution.

---

# 262. Notification Privacy Test

Lock device.

Verify:

```text
Full
SenderOnly
Generic
```

behave correctly.

---

# 263. Duplicate Message Test

Deliver same message:

```text
direct + mailbox + wake
```

one notification.

---

# 264. Push Duplicate Test

Send same wake repeatedly.

One background sync coalesced.

---

# 265. Message Burst Test

100 messages across 5 conversations.

Expected:

```text
bounded notification count
grouped summaries
no UI freeze
```

---

# 266. Unknown Peer Flood

Thousands of unknown message requests.

Expected:

```text
rate-limited notification behavior
```

---

# 267. Incoming Call Flood

Unknown caller sends repeated offers.

Expected:

```text
Part 28/29 rate limiting
no endless ringing
```

---

# 268. Answered Elsewhere Test

Phone + desktop ring.

Desktop answers.

Phone notification cancels quickly.

---

# 269. Read Elsewhere Test

Desktop reads.

Phone message notification updates/cancels.

---

# 270. Background Decryption Test

Hardware key unavailable until unlock.

Expected:

```text
ciphertext stored
generic notification
decrypt after unlock
```

if platform/key policy configured that way.

---

# 271. Crash After Persist Before Notify

On restart:

```text
reconstruct candidate
notify if still unread/relevant
```

---

# 272. Crash After Notify

No duplicate notification beyond dedup policy.

---

# 273. Offline Push Test

Wake received with no network.

Pending sync waits until connectivity returns.

---

# 274. Provider Outage Test

App foreground/direct connectivity still works.

---

# 275. Notification Permission Revoked

Runtime detects and diagnostics explain.

---

# 276. Foreground Conversation Test

Message arrives while conversation open.

No redundant OS notification.

---

# 277. Screen-Share Privacy Test

Message arrives during active screen share.

Preview automatically generic if policy enabled.

---

# 278. Performance

Background wake path should use minimal:

```text
RAM
CPU
radio time
```

---

# 279. Cold Wake Target

Aim for:

```text
minimal startup subset
```

rather than full app initialization.

---

# 280. Allocation Policy

Notification handling is not a hot media path.

Favor correctness/clarity over exotic zero-copy.

---

# 281. Resource Limits

Part 08 bounds:

```text
pending wake requests
notification projection
sync concurrency
push retries
```

---

# 282. Background Sync Concurrency

Usually:

```text
one sync per account/device
```

with coalescing.

---

# 283. Multi-Account Future

If multi-account supported:

```text
per-account sync queue
global concurrency cap
```

---

# 284. Headless Lock

Use an async mutex/lease to avoid two background syncs racing.

---

# 285. Watchdog

Background sync has deadline.

If exceeded:

```text
cancel safely
retry later
```

---

# 286. Cancellation

Every background task must be cancellation-safe.

---

# 287. Partial Fetch

Persist each validated batch transactionally.

Do not lose all progress because wake window ended.

---

# 288. Sync Cursor Advancement

Advance cursor only after batch durable commit.

---

# 289. Notification Generation After Batch

Generate candidates for committed messages only.

---

# 290. Notification Ordering

Within conversation:

```text
message sequence
```

not push arrival order.

---

# 291. Call Priority Over Message Burst

If incoming call and 50 messages pending:

```text
validate/ring call first
```

---

# 292. Emergency Priority Over Ordinary Sync

Same.

---

# 293. Security Priority

New device/revocation may need prompt attention.

---

# 294. Suggested Workspace

```text
crates/
├── comm-notification-core/
├── comm-notification-policy/
├── comm-background-sync/
├── comm-push-core/
├── comm-push-android/
├── comm-push-apple/
├── comm-notify-android/
├── comm-notify-linux/
├── comm-notify-windows/
├── comm-notify-apple/
├── comm-notification-diagnostics/
└── comm-notification-testkit/
```

---

# 295. `comm-notification-core`

Owns:

```text
semantic notification types
importance
destination
actions
privacy modes
```

---

# 296. `comm-notification-policy`

Owns:

```text
mute
DND
foreground suppression
privacy
rate limits
importance mapping
```

---

# 297. `comm-background-sync`

Owns:

```text
wake coalescing
sync scheduling
deadlines
mailbox fetch
outbox flush
```

---

# 298. `comm-push-core`

Owns:

```text
push registration abstraction
opaque wake token
wake reasons
```

---

# 299. Platform Push Crates

Own unavoidable provider/platform APIs.

---

# 300. `comm-notification-testkit`

Provides:

```text
fake lifecycle
fake push
fake OS notifier
fake clock
network partition
multi-device scenarios
```

---

# 301. Public Commands

```rust
pub enum NotificationCommand {
    RegisterPush,
    UnregisterPush,
    HandleWake(PushWakeHint),
    MarkRead(NotificationId),
    Reply {
        notification: NotificationId,
        text: String,
    },
    AcceptCall(CallId),
    DeclineCall(CallId),
}
```

---

# 302. Public Events

```rust
pub enum NotificationEvent {
    PushRegistered,
    PushRegistrationFailed,
    WakeReceived,
    SyncStarted,
    SyncCompleted,
    NotificationShown(NotificationId),
    NotificationCancelled(NotificationId),
}
```

---

# 303. Implementation Phases

## Phase 1 — Local Notifications

```text
semantic notifications
privacy modes
message grouping
Dioxus integration
```

## Phase 2 — Android Push Wake

```text
registration
wake callback
minimal bridge
headless Rust sync
```

## Phase 3 — Background Sync

```text
mailbox fetch
outbox
receipts
dedup
```

## Phase 4 — Calls

```text
incoming call
accept/decline
foreground service
answered elsewhere
```

## Phase 5 — Security / Emergency

```text
priority channels
verified security alerts
emergency fast path
```

## Phase 6 — Multi-Device

```text
read elsewhere
notification cancellation
device delivery
```

## Phase 7 — Desktop / Apple

```text
native notification backends
APNs/CallKit later
```

## Phase 8 — Hardening

```text
Doze
OEM restrictions
provider outage
flood tests
privacy tests
cold-start tests
```

---

# 304. Initial Production Recommendation

For Android v1:

```text
wake-only push
headless Rust background fetch
message notifications
incoming-call notifications
security notifications
generic/full preview modes
conversation grouping
read/reply actions
foreground call service
```

Do not initially attempt:

```text
plaintext push messages
per-message provider payloads
permanent hidden background daemon
complex plugin notification privileges
```

---

# 305. Definition of Done

Part 31 is complete when:

- push is used as a wake hint rather than authoritative message transport
- push payloads contain no message plaintext by default
- messages are verified/decrypted/persisted before notification
- background wake can run without full Dioxus UI startup
- duplicate pushes coalesce into one sync
- direct/mailbox/push duplicate delivery produces one logical notification
- message notification grouping is conversation-aware
- notification preview privacy has Full/SenderOnly/Generic modes
- incoming calls are authenticated before ringing
- call accept/decline actions route into Part 29
- answered-elsewhere cancels other device call notifications
- read-elsewhere can clear local message notification
- active calls use appropriate Android foreground-service integration
- battery saver and OS background restrictions are respected
- background file work is user-visible/foreground only when justified
- notification channels are stable and bounded
- plugins cannot self-elevate notification priority
- critical/emergency notifications require policy authorization
- push/provider outages do not break foreground/direct P2P messaging
- background sync is idempotent, bounded, and crash-safe
- Android permission/system-notification state is diagnosable
- notification content is not included in telemetry/logging
- cold-start, Doze, OEM restriction, duplicate, flood, offline, and multi-device tests exist

---

# 306. Relationship to Earlier Parts

Part 31 integrates with:

```text
02 — Multi-Device Identity
03 — Routing Policy
04 — Offline Event Log
06 — DTN
07 — Capability Negotiation
08 — Resource Limits
09 — Crash Recovery
11 — Relay / Mailbox Infrastructure
13 — Battery-Aware Scheduling
16 — Daemon / Headless Runtime
17 — Emergency Priority
18 — Diagnostics
20 — Embedded Linux
21 — Third-Party Extensions
22 — WASM Components
24 — Plugin Ecosystem
27 — Android Build / Packaging
28 — E2EE / Security
29 — Realtime Calls
30 — Presence / Receipts / Ephemeral State
```

---

# 307. Final Architecture

```text
                 Remote Encrypted Event
                         │
             ┌───────────┴───────────┐
             │                       │
        App Reachable            App Sleeping
             │                       │
          Direct P2P            Wake-Only Push
             │                       │
             └───────────┬───────────┘
                         │
                  Rust Background Sync
                         │
                  Fetch / Verify / E2EE
                         │
                       Persist
                         │
                  Notification Policy
                         │
              ┌──────────┴──────────┐
              │                     │
          In-App UI          OS Notification
```

Incoming call:

```text
Push Wake
   ↓
Fetch Call Offer
   ↓
Authenticate Caller
   ↓
Check Abuse / Privacy
   ↓
Show Incoming Call
   ↓
Accept / Decline
   ↓
Part 29 Call Controller
```

---

# 308. Final Principle

The notification system should never become a hidden centralized messaging layer.

The correct model is:

```text
Push:
    "wake up, something may be pending"

Rust runtime:
    fetch
    authenticate
    decrypt
    persist
    decide

OS notification:
    inform the user
```

That preserves the core architecture:

```text
P2P-first
E2EE
local-first
offline-capable
self-hostable
battery-aware
```

while still giving users the mobile responsiveness they expect from a production messenger.
