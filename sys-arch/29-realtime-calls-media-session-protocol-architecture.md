# Part 29 — Realtime Calls & Media Session Protocol Architecture

## Reusable P2P Communication Platform

**Status:** Architecture specification  
**Part:** 29  
**Primary language:** Rust  
**Primary purpose:** define the complete realtime call/session control plane for secure audio/video communication over the existing Iroh + Dioxus P2P platform  
**Primary goals:** deterministic call state, secure signaling, transport-independent sessions, low-latency media negotiation, reconnection, hold/resume, camera/screen-share changes, multipath resilience, E2EE binding, group-call evolution, crash-safe call history, and production-grade failure handling

---

# 1. Purpose

The platform already includes:

```text
Iroh transport
multipath routing
hardware/software video codecs
Rust-first audio DSP
Android zero-copy media
E2EE/key management
capability negotiation
resource limits
battery policy
diagnostics
daemon/headless mode
```

What is still missing is the **call/session protocol** that coordinates all of them.

A realtime call is not merely:

```text
open QUIC stream
send Opus
send H.264
```

A production call requires:

```text
ringing
accept/reject
busy
timeout
media negotiation
device selection
session authentication
codec selection
bitrate constraints
camera mute
microphone mute
hold/resume
route changes
path handoff
relay fallback
reconnection
termination
history
```

The governing principle is:

> **The call is a logical secure session; network paths, codecs, devices, and surfaces are replaceable implementation resources inside that session.**

---

# 2. Architectural Position

```text
                    User / Dioxus UI
                           │
                           ▼
                    Call Controller
                           │
                ┌──────────┼───────────┐
                │          │           │
             Signaling   Security    Policy
                │          │           │
                └──────────┼───────────┘
                           │
                    Media Session
                ┌──────────┴───────────┐
                │                      │
              Audio                  Video
                │                      │
              Opus            H264/H265/AV1
                │                      │
                └──────────┬───────────┘
                           │
                     Media Transport
                           │
             Iroh Direct / Relay / Multipath
```

---

# 3. Call Protocol Layers

Separate:

```text
Call Signaling
Call State Machine
Media Negotiation
Security Binding
Transport Selection
Media Transport
Device/Surface Control
UI State
```

Do not mix all of these into one monolithic call task.

---

# 4. Logical Call Identity

```rust
pub struct CallId([u8; 16]);
```

Properties:

```text
globally unique
stable across reconnects
independent of transport connection
```

Do not use QUIC connection ID as call identity.

---

# 5. Participant Identity

```rust
pub struct CallParticipant {
    pub account: AccountId,
    pub device: DeviceId,
}
```

Logical user and actual device are separate.

---

# 6. Call Session Identity

A specific established device-to-device media session gets:

```rust
pub struct CallSessionId([u8; 16]);
```

A reconnect may preserve:

```text
CallId
```

while creating a new:

```text
CallSessionId
```

---

# 7. Call Types

```rust
pub enum CallKind {
    Audio,
    Video,
    ScreenShare,
    GroupAudio,
    GroupVideo,
}
```

---

# 8. Call Direction

```rust
pub enum CallDirection {
    Incoming,
    Outgoing,
}
```

---

# 9. High-Level Call State

```rust
pub enum CallState {
    Idle,
    Creating,
    RingingOutgoing,
    RingingIncoming,
    Negotiating,
    Connecting,
    Active,
    Reconnecting,
    Held,
    Ending,
    Ended,
    Failed,
}
```

---

# 10. Detailed Outgoing State Flow

```text
Idle
 ↓
Creating
 ↓
Send Offer
 ↓
RingingOutgoing
 ↓
Remote Accept
 ↓
Negotiating
 ↓
Connecting
 ↓
Active
```

Alternative endings:

```text
Rejected
Busy
Timeout
Cancelled
Failed
```

---

# 11. Incoming State Flow

```text
Idle
 ↓
Offer Received
 ↓
Validate
 ↓
RingingIncoming
 ↓
Accept
 ↓
Negotiating
 ↓
Connecting
 ↓
Active
```

---

# 12. Call Offer

```rust
pub struct CallOffer {
    pub call_id: CallId,
    pub caller: DeviceId,
    pub callee: DeviceId,
    pub kind: CallKind,
    pub media_caps: MediaCapabilitySet,
    pub security: CallSecurityOffer,
    pub created_at: Timestamp,
    pub expires_at: Timestamp,
}
```

---

# 13. Call Offer Security

Offer must be authenticated and bound to:

```text
caller identity
callee identity
CallId
supported media
security version
expiry
```

Part 28 handles the cryptographic binding.

---

# 14. Offer Expiry

A call offer should expire quickly.

Example:

```text
30–90 seconds
```

depending UX.

Expired calls must not ring later via DTN.

---

# 15. Calls Are Not DTN

Realtime call setup requires live reachability.

Do not use Part 06 store-carry-forward for:

```text
live call offer
media
ringing state
```

If peer is offline:

```text
send missed-call event
```

afterward if desired.

---

# 16. Missed Call Event

Missed-call history may use durable messaging/event log.

```rust
pub struct MissedCallEvent {
    pub call_id: CallId,
    pub caller: AccountId,
    pub kind: CallKind,
    pub timestamp: Timestamp,
}
```

---

# 17. Ringing Semantics

Only one device may answer, or policy may allow multiple devices.

For ordinary account:

```text
ring all active devices
first accepted device wins
others receive answered-elsewhere
```

---

# 18. Multi-Device Ringing

Flow:

```text
Caller
  ↓
Offer to recipient account devices
  ↓
Phone rings
Desktop rings
Tablet rings
  ↓
Desktop accepts
  ↓
Phone/Tablet → AnsweredElsewhere
```

---

# 19. Call Arbitration

Recipient device acceptance must be atomic logically.

Use:

```text
CallAcceptToken
```

or server/P2P arbitration protocol so two devices do not establish duplicate sessions accidentally.

---

# 20. Call Response

```rust
pub enum CallResponse {
    Accept(CallAccept),
    Reject(CallRejectReason),
    Busy,
    AnsweredElsewhere,
}
```

---

# 21. Reject Reasons

```rust
pub enum CallRejectReason {
    UserDeclined,
    Busy,
    Unsupported,
    PermissionDenied,
    SecurityPolicy,
    ResourceUnavailable,
}
```

---

# 22. Busy Policy

User may configure:

```text
busy if already in call
allow call waiting
allow second audio-only call
```

Default:

```text
single active call
```

---

# 23. Call Waiting

If supported:

```text
active call
+
incoming call
```

UI options:

```text
reject
hold current + answer
end current + answer
```

---

# 24. Call Timeout

Outgoing ringing timeout:

```text
e.g. 45–60 s
```

Should be configurable by product, not protocol hardcoded.

---

# 25. Call Cancellation

Caller can cancel during:

```text
RingingOutgoing
Negotiating
Connecting
```

Send explicit:

```text
CallCancel
```

---

# 26. Call Accept

```rust
pub struct CallAccept {
    pub call_id: CallId,
    pub accepter_device: DeviceId,
    pub selected_media: NegotiatedMedia,
    pub security: CallSecurityAccept,
}
```

---

# 27. Call Security Handshake

Call signaling and media session must be bound to Part 28 E2EE/device identity.

The media session should never accept:

```text
unauthenticated codec packets
```

from a transport that merely knows the CallId.

---

# 28. Security Binding

Bind:

```text
CallId
CallSessionId
caller DeviceId
callee DeviceId
media parameters
transport session
security epoch
```

into authenticated transcript.

---

# 29. Media Key Separation

Derive distinct keys for:

```text
audio
video
screen-share
control
```

even if underlying secure transport already exists.

This makes application E2EE independent of route/relay.

---

# 30. Transport Encryption vs Media E2EE

Iroh/QUIC protects transport.

Application media E2EE protects:

```text
relay traversal
future transport substitution
media recording boundary
logical call security
```

according to Part 28.

---

# 31. Media Capability Negotiation

Each peer advertises:

```text
audio codecs
video codecs
hardware decode support
hardware encode support
max resolution
max FPS
screen-share support
simulcast support if later added
```

---

# 32. Audio Capability

Initial:

```text
Opus
48 kHz timeline
mono voice
optional stereo
FEC
DTX
```

---

# 33. Video Capability

```text
H.264
H.265
AV1
```

Hardware/software implementation is local policy.

---

# 34. Local Codec Implementation Is Not Wire Semantics

Do not negotiate:

```text
hardware AV1
```

as protocol requirement.

Negotiate:

```text
AV1
```

Then each side chooses hardware/software backend locally.

---

# 35. Codec Preference

Example policy:

```text
AV1
H.265
H.264
```

but actual preference depends on:

```text
device power
hardware support
network
latency
compatibility
```

---

# 36. Audio Always First

If video negotiation fails but audio succeeds:

```text
fall back to audio call
```

unless user explicitly required video-only semantics.

---

# 37. Media Negotiation DTO

```rust
pub struct NegotiatedMedia {
    pub audio: Option<NegotiatedAudio>,
    pub video: Option<NegotiatedVideo>,
    pub screen_share: Option<NegotiatedVideo>,
}
```

---

# 38. Negotiated Audio

```rust
pub struct NegotiatedAudio {
    pub codec: AudioCodec,
    pub sample_rate: u32,
    pub channels: u8,
    pub packet_ms: u16,
    pub fec: bool,
    pub dtx: bool,
}
```

---

# 39. Negotiated Video

```rust
pub struct NegotiatedVideo {
    pub codec: VideoCodec,
    pub profile: CodecProfile,
    pub max_resolution: Resolution,
    pub max_fps: u32,
}
```

---

# 40. Resolution Is Upper Bound

Negotiation sets maximum.

Runtime may dynamically reduce:

```text
resolution
FPS
bitrate
```

without renegotiating every small adaptation.

---

# 41. Major Media Change

Renegotiation required for:

```text
codec switch
adding/removing screen share
changing channel count materially
adding new media stream
```

---

# 42. Minor Media Adaptation

No full signaling renegotiation needed for:

```text
bitrate
FPS
resolution within allowed envelope
FEC level
jitter target
```

---

# 43. Media Session State

```rust
pub enum MediaSessionState {
    Created,
    Negotiated,
    Starting,
    Running,
    Reconfiguring,
    Suspended,
    Stopping,
    Stopped,
    Failed,
}
```

---

# 44. Audio Session

Owns:

```text
capture
DSP
Opus encode
jitter
Opus decode
playback
```

from Part 26.

---

# 45. Video Session

Owns:

```text
camera
encoder
packetizer
decoder
Surface renderer
```

from Part 25.

---

# 46. Media Transport

Use separate logical channels:

```text
call-control
audio
video
screen-share
feedback
```

---

# 47. QUIC Stream vs Datagram

Media architecture may use:

```text
QUIC datagrams
unidirectional streams
hybrid
```

depending Iroh support/performance.

The call protocol remains transport-neutral.

---

# 48. Audio Delivery Semantics

Prefer low latency over perfect reliability.

Lost old audio packet:

```text
do not wait indefinitely
```

Use:

```text
FEC
PLC
jitter buffering
```

---

# 49. Video Delivery Semantics

Delta video frames are disposable after deadline.

Keyframes/config are more important.

---

# 50. Control Reliability

Call control messages should be reliably delivered:

```text
mute state
hold
hangup
codec change
keyframe request
```

---

# 51. Media Feedback

Receiver sends:

```text
RTT
loss
jitter
decode health
render stalls
available bitrate estimate
```

---

# 52. Congestion Controller

Inputs:

```text
packet loss
RTT
throughput
relay/direct path
multipath state
battery
thermal
```

Outputs:

```text
audio bitrate
video bitrate
video FPS
resolution
FEC
```

---

# 53. Media Priority

Recommended:

```text
1. call control
2. audio
3. video keyframe/config
4. video delta
5. screen share delta
6. bulk file traffic
```

---

# 54. Path Selection

Part 03/12 selects:

```text
direct Iroh
relay
LAN
multipath
```

Call session should not restart merely because path changes.

---

# 55. Path Independence

Correct:

```text
CallId remains
transport path changes
media continues
```

Incorrect:

```text
Wi-Fi changed
→ new call
```

---

# 56. Network Handoff

Examples:

```text
Wi-Fi → cellular
LAN → relay
direct → relay
relay → direct
```

Call transitions into:

```text
Reconnecting
```

only if media actually stalls.

---

# 57. Warm Path

Part 12 may maintain a secondary path.

For call control/audio:

```text
fast failover
```

is valuable.

---

# 58. Multipath Audio

Potential:

```text
send selective redundant audio
```

during severe path instability.

Do not duplicate all traffic by default.

---

# 59. Multipath Video

May route:

```text
keyframes on reliable path
bulk delta on best throughput path
```

later.

Start simpler.

---

# 60. Reconnection State

```rust
pub enum ReconnectState {
    None,
    Detecting,
    SearchingPath,
    Reauthenticating,
    ResumingMedia,
    Failed,
}
```

---

# 61. Reconnect Grace Window

When path disappears:

```text
do not end call instantly
```

Use short grace window.

Example:

```text
5–15 seconds
```

depending UX.

---

# 62. During Reconnect

UI:

```text
Reconnecting…
```

Media policy:

```text
pause video
retain audio buffers minimally
search alternate path
```

---

# 63. Resume After Reconnect

Need:

```text
fresh transport binding
security revalidation
keyframe request
jitter reset
```

---

# 64. Do Not Reuse Stale Decoder State Blindly

After long reconnect:

```text
request fresh video keyframe
```

---

# 65. Audio Resume

Reset/adapt jitter buffer carefully.

Do not replay seconds of stale audio.

---

# 66. Call Hold

Hold is logical state.

```rust
pub enum HoldState {
    Active,
    LocalHold,
    RemoteHold,
    BothHold,
}
```

---

# 67. Local Hold

Typical:

```text
stop/pause microphone transmission
stop/pause camera
continue call-control heartbeat
```

---

# 68. Remote Hold

UI shows:

```text
Call on hold
```

Audio may be silence or optional hold tone.

---

# 69. Resume

Re-enable media gradually.

Request keyframe if video paused long enough.

---

# 70. Microphone Mute

Mute is not hold.

```text
call active
audio receive continues
capture encoded as silence/DTX
```

---

# 71. Camera Disable

Video sender stops camera/encoder.

Audio remains active.

Remote decoder becomes inactive/placeholder.

---

# 72. Camera Enable

Requires:

```text
media-control message
encoder start
keyframe
```

No full call restart.

---

# 73. Camera Switch

Front/rear camera switch should remain within same media stream if possible.

---

# 74. Speaker Route Change

Audio route changes:

```text
speaker
earpiece
Bluetooth
wired
```

handled by Part 26 without signaling unless capabilities fundamentally change.

---

# 75. Screen Share

Screen share is a separate media source.

State:

```rust
pub enum ScreenShareState {
    Off,
    Starting,
    Active,
    Stopping,
}
```

---

# 76. Screen Share Negotiation

Peer must support:

```text
screen-share media stream
```

and compatible video codec.

---

# 77. Screen Share Encoding

Android:

```text
screen capture surface
→ hardware encoder surface
```

same Part 25 zero-copy principles.

---

# 78. Screen Share Priority

Depending call:

```text
screen share
```

may outrank camera video.

Audio still outranks both.

---

# 79. Media Stream IDs

```rust
pub struct MediaStreamId(u32);
```

Possible:

```text
1 audio
2 camera video
3 screen share
```

---

# 80. Stream Descriptor

```rust
pub struct MediaStreamDescriptor {
    pub id: MediaStreamId,
    pub kind: MediaStreamKind,
    pub direction: MediaDirection,
    pub codec: MediaCodec,
}
```

---

# 81. Direction

```rust
pub enum MediaDirection {
    SendOnly,
    ReceiveOnly,
    SendReceive,
    Inactive,
}
```

---

# 82. Call Control Messages

Core set:

```text
Offer
Ringing
Accept
Reject
Cancel
MediaReady
MuteChanged
VideoChanged
Hold
Resume
ScreenShareStart
ScreenShareStop
PathChanged
Renegotiate
Hangup
Ack
```

---

# 83. Call Control Versioning

Use dedicated protocol:

```text
call/1
```

with forward-compatible required/optional features.

---

# 84. Unknown Optional Control Message

Ignore safely.

Unknown required semantic:

```text
terminate/renegotiate with typed error
```

---

# 85. Call Control Idempotency

Repeated:

```text
Accept
Hangup
Hold
```

must not corrupt state.

---

# 86. Sequence Numbers

```rust
pub struct CallControlSeq(pub u64);
```

Use for:

```text
ordering
duplicate detection
```

---

# 87. State Revision

Optional:

```rust
pub struct CallRevision(pub u64);
```

Every material call-state update increases revision.

---

# 88. Stale Control Frame

If:

```text
revision < current
```

ignore safely.

---

# 89. Call Termination

```rust
pub enum CallEndReason {
    LocalHangup,
    RemoteHangup,
    Declined,
    Busy,
    Timeout,
    NetworkLost,
    SecurityError,
    MediaFailure,
    ResourceLimit,
    AppShutdown,
}
```

---

# 90. Hangup

Hangup should be reliable where possible.

But if network disappears:

```text
local call still ends
```

Do not wait indefinitely for remote ACK.

---

# 91. Termination Cleanup

Order:

```text
mark ending
stop new media
stop camera/mic
stop encoder/decoder
release surfaces/audio routes
close media streams
persist call history
release transport/session resources
```

---

# 92. Call History

Call history is durable metadata, not media.

```rust
pub struct CallHistoryEntry {
    pub call_id: CallId,
    pub peer: AccountId,
    pub direction: CallDirection,
    pub kind: CallKind,
    pub started_at: Timestamp,
    pub connected_at: Option<Timestamp>,
    pub ended_at: Timestamp,
    pub end_reason: CallEndReason,
}
```

---

# 93. Privacy of Call History

Call history can reveal:

```text
who
when
duration
```

Treat as private metadata.

Encrypt locally according to Part 28 local-storage policy.

---

# 94. Do Not Store Media By Default

Call history must not imply:

```text
recording
```

Audio/video is not stored unless user explicitly records.

---

# 95. Missed Calls

A call is missed if:

```text
incoming
not answered
before expiry/end
```

---

# 96. Call Duration

Duration begins when:

```text
media/session connected
```

not when ringing started.

---

# 97. Crash During Call

If app/process crashes:

```text
call ends
```

Remote eventually detects heartbeat/media/control loss.

Local next startup records:

```text
abnormal termination
```

if reconstructible.

---

# 98. Durable Call Intent

Live calls themselves are ephemeral.

Do not attempt to crash-recover a destroyed media session as if nothing happened.

Use:

```text
new reconnect/re-call
```

after process restart.

---

# 99. Call Heartbeat

Control plane may send periodic liveness.

But media activity itself can also indicate health.

Avoid excessive heartbeat traffic.

---

# 100. Liveness Timeout

Differentiate:

```text
media silent
```

from:

```text
transport dead
```

VAD silence is normal.

Use control/transport signals.

---

# 101. Security Failure During Call

Examples:

```text
identity mismatch
rekey failure
revocation arrives
```

Policy may:

```text
terminate immediately
```

for high-risk failure.

---

# 102. Device Revocation Mid-Call

If remote device becomes revoked:

```text
end call
```

and show security reason.

---

# 103. Session Rekey

Long calls should support periodic media/session key rotation.

No visible interruption.

---

# 104. Group Calls

Initial group architecture should avoid premature complexity.

Possible starting model:

```text
small mesh
```

for very small participant counts.

---

# 105. Mesh Group Call

For N participants:

```text
each peer sends to others
```

Advantages:

```text
simple
fully P2P
```

Disadvantages:

```text
uplink grows O(N)
battery/CPU high
```

---

# 106. Mesh Limit

Practical only for small groups.

Example:

```text
3–4 participants
```

depending device/network.

---

# 107. SFU Evolution

For larger group calls, introduce:

```text
selective forwarding node
```

later.

This can be:

```text
self-hosted
organization-hosted
possibly trusted only for routing, not plaintext
```

with media E2EE maintained where architecture supports.

---

# 108. Do Not Build SFU Into v1 Unless Needed

Start with:

```text
1:1 excellent
small group later
```

Production quality is more valuable than premature large-group complexity.

---

# 109. Group Call ID

```rust
pub struct GroupCallId([u8; 16]);
```

---

# 110. Group Participant State

```rust
pub struct GroupParticipantState {
    pub participant: CallParticipant,
    pub audio: MediaDirection,
    pub video: MediaDirection,
    pub screen_share: bool,
    pub speaking: bool,
}
```

---

# 111. Active Speaker

Use Part 26 VAD/audio levels.

UI only.

Do not alter cryptographic identity.

---

# 112. Group Join

Requires:

```text
membership authorization
group security state
media capability negotiation
```

---

# 113. Group Leave

Participant leaves cleanly:

```text
remove media streams
update UI
```

---

# 114. Group Device Revocation

If participant device revoked:

```text
terminate its media
advance group security state
```

---

# 115. Simulcast

Future optimization:

```text
sender encodes multiple resolutions
```

for group/SFU.

Do not require initially.

---

# 116. SVC

AV1 scalable video coding may eventually improve group efficiency.

Treat as optional capability.

---

# 117. Call Recording

Optional feature.

Require explicit user action.

---

# 118. Recording Consent

Product policy may require:

```text
visible recording indicator
```

and possibly participant notification.

---

# 119. Recording Architecture

Prefer storing:

```text
compressed media
```

without unnecessary decode/re-encode.

---

# 120. Recording Encryption

Stored recording should be locally encrypted.

---

# 121. Call Notes / Metadata

Separate user-generated notes from actual recording.

---

# 122. Call Notifications

Incoming call uses platform notification/foreground service as needed.

Rust owns call state; platform adapter owns notification mechanics.

---

# 123. Android Foreground Service

Long-running call may require foreground-service integration according to Android policy.

Keep service control behind narrow Android adapter.

---

# 124. Notification Actions

Incoming call actions:

```text
Accept
Decline
```

must map into Rust `CallController`.

---

# 125. Lockscreen Call UX

Platform-specific surface.

Do not duplicate call state in Kotlin.

Kotlin sends action event to Rust.

---

# 126. Dioxus Call UI

Screens:

```text
incoming call
outgoing ringing
active audio call
active video call
reconnecting
held
ended
```

---

# 127. Call View Model

```rust
pub struct CallViewModel {
    pub state: CallState,
    pub peer_display: String,
    pub duration: Duration,
    pub muted: bool,
    pub video_enabled: bool,
    pub route: AudioRoute,
    pub quality: CallQualityClass,
}
```

---

# 128. UI Does Not Own Media State

Dioxus issues:

```text
Mute
Unmute
EnableVideo
DisableVideo
Hangup
```

Rust controller decides resulting state.

---

# 129. Video Surface Integration

Dioxus receives:

```text
RendererId
```

not decoded frames.

Part 25 handles Surface lifecycle.

---

# 130. Audio UI Integration

Dioxus receives:

```text
audio level
route
mute state
```

not PCM buffers.

---

# 131. Call Quality UI

Simple states:

```rust
pub enum CallQualityClass {
    Excellent,
    Good,
    Fair,
    Poor,
    Reconnecting,
}
```

Derived from diagnostics.

---

# 132. Advanced Call Diagnostics

Developer view:

```text
path
RTT
loss
audio jitter
audio bitrate
video codec
video bitrate
FPS
resolution
hardware/software backend
```

---

# 133. Privacy of Diagnostics

Do not show/store:

```text
raw media
private keys
precise remote IP by default
```

---

# 134. Battery Policy

Part 13 can reduce:

```text
video resolution
FPS
software AV1 use
multipath redundancy
```

before compromising audio.

---

# 135. Thermal Policy

If device overheats:

```text
disable high-cost video processing
reduce FPS/resolution
prefer hardware codec
```

---

# 136. Resource Limits

Part 08 bounds:

```text
active calls
audio streams
video streams
codec instances
jitter buffers
pending call offers
```

---

# 137. Incoming Call Flood

Unknown peers must not generate unlimited:

```text
ringing
notifications
wakeups
```

Use Part 28 abuse controls.

---

# 138. Call Request Policy

Unknown sender:

```text
silent request
restricted ring
or blocked
```

depending privacy setting.

---

# 139. Contact-Only Calling

Optional setting:

```text
only trusted contacts can ring
```

---

# 140. Organization Calls

Managed deployments may allow:

```text
staff-only
role-based
emergency authority
```

but still through explicit policy.

---

# 141. Emergency Call Mode

Part 17 may permit special:

```text
high-priority emergency audio
```

but security/authorization remains mandatory.

---

# 142. Emergency Degradation Ladder

```text
video
→ low video
→ audio
→ low bitrate audio
→ voice note
→ text
```

---

# 143. Bluetooth Calling

Bluetooth audio route handled by Part 26.

Bluetooth transport is generally unsuitable for full realtime call media except specialized local scenarios.

---

# 144. LAN Calls

LAN/direct path is ideal:

```text
low latency
high bandwidth
```

---

# 145. Relay Calls

Relay is valid fallback.

Media E2EE means relay does not require plaintext.

---

# 146. Offline Local Calls

If two devices share LAN/local connectivity but no Internet:

```text
call can work
```

provided discovery/routing/session negotiation succeeds.

---

# 147. Internetless Mesh Call

Possible through local Wi-Fi/router mesh.

Do not require cloud signaling if peer discovery/contact route is available.

---

# 148. Signaling Transport

Call signaling itself can use:

```text
existing secure peer protocol
```

over any available path.

No separate centralized signaling server is architecturally required.

---

# 149. Optional Rendezvous Assistance

If peer discovery needs assistance:

```text
relay/discovery infrastructure
```

may help.

Call semantics remain P2P.

---

# 150. Call Offer Routing

Use Part 03 route policy.

Preference:

```text
direct secure peer
→ relay
```

depending reachability.

---

# 151. Signaling Reliability

Offer/accept/hangup must be reliable and idempotent.

---

# 152. Signaling Retry

Retries use:

```text
same CallId
same logical operation ID
```

not new calls.

---

# 153. Duplicate Offer

Recipient detects same CallId:

```text
do not ring twice
```

---

# 154. Call Glare

Both users call each other simultaneously.

Need deterministic resolution.

---

# 155. Glare Resolution

Example:

```text
compare CallId / device IDs deterministically
```

One call wins; the other becomes merged/cancelled.

---

# 156. Call Merge

Optional behavior:

```text
simultaneous call offers
→ treat as one call
```

Useful UX.

---

# 157. Permission Handling

Before starting capture:

```text
microphone permission
camera permission
screen-capture permission
```

must be available.

---

# 158. Permission Denied

Audio call cannot begin without microphone if user intends two-way audio.

Video may downgrade to audio if camera denied.

---

# 159. Camera Permission Mid-Call

If user later enables camera:

```text
request permission
then add video
```

---

# 160. Screen Share Permission

Requires explicit per-session platform consent where OS demands.

---

# 161. Device Availability

Call startup checks:

```text
microphone
speaker/output
camera if needed
codec resources
```

---

# 162. Media Resource Admission

Before accept:

```text
can this device actually allocate media resources?
```

If not:

```text
accept audio-only
or reject with ResourceUnavailable
```

---

# 163. Prewarming

For outgoing video call, prewarm:

```text
codec capability
camera permission
audio route
```

without opening camera too early.

---

# 164. Ringing Battery Cost

Do not activate full media pipeline before accept.

---

# 165. MediaReady

Both sides can exchange:

```text
MediaReady
```

after local codecs/audio are initialized.

---

# 166. Connection Established

Call enters `Active` when minimum required media path is ready.

For audio call:

```text
audio send/receive ready
```

For video call:

```text
audio ready
video may join milliseconds later
```

---

# 167. First Media Packet

Do not wait indefinitely.

Startup timeout triggers:

```text
media failure
```

or downgrade.

---

# 168. Audio-First Startup

For video call:

```text
establish audio first
then video
```

improves perceived connection time.

---

# 169. Video Warm-Up

Camera/encoder can start after call accepted.

---

# 170. Keyframe on Start

First decodable video frame must include correct config/keyframe.

---

# 171. Codec Failure on Start

If AV1 fails locally:

```text
fallback H.265
fallback H.264
```

if negotiated alternatives exist.

---

# 172. Negotiated Alternative Set

Offer can provide ordered acceptable codecs.

Accept selects primary plus optional fallback set.

---

# 173. Mid-Call Codec Fallback

If hardware AV1 crashes repeatedly:

```text
renegotiate H.264/H.265
```

without ending call if possible.

---

# 174. Audio Codec Failure

Opus is baseline.

If Opus fails locally:

```text
call cannot continue normally
```

because no alternative codec is required initially.

---

# 175. Jitter Recovery

Part 26 adaptive jitter handles transient network change.

---

# 176. Video Decoder Recovery

Part 25:

```text
decoder reset
request keyframe
resume
```

---

# 177. Surface Loss

UI rotation/navigation may destroy Surface.

Call remains active.

Remote video may temporarily pause rendering.

---

# 178. App Background

Policy:

```text
audio continues where OS allows
camera may stop
video receive may suspend rendering
```

---

# 179. Screen Off

Audio call should continue efficiently.

---

# 180. Android Process Importance

Use platform foreground service/lifecycle where required.

---

# 181. Desktop Window Close

If call UI window closes accidentally:

```text
product policy
```

may keep call in background or hang up.

Rust controller remains canonical.

---

# 182. Headless Call Mode

Part 16 daemon can theoretically support:

```text
audio endpoint
automated call agent
intercom
```

through same call protocol.

---

# 183. Embedded Node Calls

Part 20 node may support:

```text
intercom
emergency audio
```

if audio hardware exists.

No Dioxus required.

---

# 184. FFI

Part 19 exposes high-level call API:

```text
start_call
accept_call
reject_call
mute
enable_video
hangup
subscribe_call_events
```

Foreign apps do not control codec internals directly by default.

---

# 185. C ABI Call Handle

```c
typedef comm_handle_t comm_call_handle_t;
```

---

# 186. Async Call Operations

Start/accept return operation/call handles.

Events report state.

---

# 187. Plugin Boundary

Third-party plugin cannot:

```text
listen to raw call audio/video
```

without explicit future high-risk media permission.

---

# 188. Extension Call Protocol

A plugin may define call-related metadata/workflow.

Core media security remains platform-owned.

---

# 189. WASM Components

Do not route realtime PCM/video frames through WASM.

WASM may implement:

```text
policy
workflow
call routing rules
```

not media hot path.

---

# 190. Call Policy Component

Possible input:

```text
peer trust
battery
network
organization policy
```

Output:

```text
allow video
allow unknown caller
preferred route class
```

Core enforces hard limits.

---

# 191. Call History Sync

Call history can sync across devices if desired.

Use encrypted application sync.

---

# 192. History Conflict

Same CallId merges duplicate records.

---

# 193. Missed Call Sync

Missed call shown across devices should deduplicate by CallId.

---

# 194. Privacy Setting

User may choose:

```text
do not sync call history
```

---

# 195. Call Event Log

Ephemeral media is not event-sourced.

Control/history events may be durable:

```text
CallOffered
CallAccepted
CallEnded
```

only if needed for history/sync.

---

# 196. Event Log Scope

Avoid storing every:

```text
mute toggle
bitrate change
packet event
```

as durable event.

Those are ephemeral diagnostics.

---

# 197. Call Metrics

Aggregate:

```text
connect time
duration
reconnect count
average RTT
loss class
codec
```

for local diagnostics.

---

# 198. No Content Metrics

Do not collect:

```text
speech content
video frames
conversation transcript
```

---

# 199. Quality Adaptation State

```rust
pub struct MediaAdaptationState {
    pub audio_bitrate: u32,
    pub video_bitrate: Option<u32>,
    pub fps: Option<u32>,
    pub resolution: Option<Resolution>,
}
```

---

# 200. Congestion Hysteresis

Avoid oscillation:

```text
720p ↔ 1080p every second
```

Use hold periods and confidence.

---

# 201. Recovery Priority

When bandwidth drops:

```text
preserve audio first
then key video
then quality
```

---

# 202. Packet Scheduler

Bounded queues per class.

Do not let video backlog increase call latency.

---

# 203. Audio Queue

Very small.

Old audio frames are discarded.

---

# 204. Video Queue

Small.

Drop stale delta frames before adding latency.

---

# 205. Control Queue

Reliable and bounded.

---

# 206. Keyframe Queue

Avoid sending multiple stale keyframes.

Coalesce requests.

---

# 207. Keyframe Request Storm

Rate-limit remote keyframe requests.

---

# 208. Call Abuse

Unknown peer should not be able to force:

```text
camera activation
microphone activation
hardware codec allocation
```

before user accepts.

---

# 209. Incoming Offer Validation

Before ring:

```text
authenticate sender
check blocklist
check rate limit
check call policy
```

---

# 210. Unknown Caller UI

Show:

```text
unknown caller
```

without granting trust.

---

# 211. Caller ID Spoofing

Display identity must come from authenticated peer/account mapping.

Never trust arbitrary supplied display name alone.

---

# 212. Call Link Feature

If future shareable call links exist:

```text
separate tokenized authorization model
```

not same as trusted contact call.

---

# 213. Call Link Security

Use:

```text
high-entropy token
expiry
room security policy
```

---

# 214. Group Invite Security

Group call invite should be bound to:

```text
group membership
call ID
security epoch
```

---

# 215. Moderation in Group Calls

Possible roles:

```text
host
moderator
participant
```

Actions:

```text
mute request
remove participant
lock room
```

must be authenticated.

---

# 216. Local Mute vs Moderator Mute

A moderator may request/force upstream mute according to product policy.

Still clearly represented in UI.

---

# 217. End-to-End Media Authentication

Receiver must authenticate:

```text
which device produced audio/video
```

not just decrypt generic group media.

---

# 218. Media Frame Identity

Frame metadata includes:

```text
stream ID
sender device
sequence/timestamp
security epoch
```

authenticated.

---

# 219. Screen Share Security

Screen-share stream is independently authenticated/encrypted.

---

# 220. Recording Marker

If recording is active, include local state and optional participant notification according to policy.

---

# 221. Call State Persistence

Persist only enough to reconstruct:

```text
history
missed call
abnormal termination
```

Do not persist live codec/session handles.

---

# 222. Restart Recovery

On app restart:

```text
active call from previous process
→ mark ended_abnormally
```

unless external daemon/media service preserved it intentionally.

---

# 223. Daemon-Preserved Calls

If architecture later puts media in persistent service process:

```text
UI restart
```

may reattach to existing call.

Call protocol supports this because UI is not authoritative.

---

# 224. Call Ownership

```rust
pub enum CallOwner {
    AppProcess,
    Daemon,
    MediaService,
}
```

---

# 225. Reattachment

UI queries:

```text
active call snapshot
```

then binds controls/video renderer.

---

# 226. Snapshot

```rust
pub struct CallSnapshot {
    pub call_id: CallId,
    pub state: CallState,
    pub peer: AccountId,
    pub media: NegotiatedMedia,
    pub muted: bool,
    pub video_enabled: bool,
}
```

---

# 227. Snapshot Is Not Authority

It is a read model.

Commands still go through controller.

---

# 228. Testing — State Machine

Exhaustively test valid/invalid transitions.

Examples:

```text
Idle → Active
```

must fail.

```text
RingingIncoming → Accept → Negotiating
```

must succeed.

---

# 229. Duplicate Signaling Tests

Inject duplicate:

```text
Offer
Accept
Hangup
```

State remains correct.

---

# 230. Simultaneous Call Test

Both sides call at same time.

Glare resolves deterministically.

---

# 231. Multi-Device Answer Test

Two recipient devices accept nearly simultaneously.

Exactly one wins.

---

# 232. Reconnect Test

During active call:

```text
kill Wi-Fi
enable cellular
```

Call should recover where alternate path exists.

---

# 233. Relay Fallback Test

Block direct path.

Call falls back to relay.

---

# 234. Direct Upgrade Test

Start on relay.

Direct path later becomes available.

Optional:

```text
migrate to direct
```

without ending call.

---

# 235. Codec Fallback Test

Force hardware AV1 failure.

Expected:

```text
renegotiate H.265/H.264
```

if supported.

---

# 236. Surface Loss Test

Destroy/recreate Android video Surface during call.

Audio remains uninterrupted.

---

# 237. Audio Route Test

Speaker → Bluetooth → speaker.

Call persists.

---

# 238. Hold Test

Hold/resume repeatedly.

No stale audio/video queues.

---

# 239. Camera Toggle Test

Enable/disable video repeatedly.

No call restart.

---

# 240. Screen Share Test

Start/stop screen share during camera call.

---

# 241. Packet Loss Test

Inject:

```text
1%
5%
10%
```

loss.

Audio remains prioritized.

---

# 242. High Jitter Test

Adaptive audio jitter remains bounded.

---

# 243. Long Call Soak

Run:

```text
2–8 hours
```

with:

```text
network handoffs
mute
camera toggle
route change
```

Track memory/resource leaks.

---

# 244. Resource Exhaustion Test

Force codec allocation failure.

Expected:

```text
video downgrade
audio survives
```

---

# 245. Security Revocation Test

Revoke active remote device.

Call terminates securely.

---

# 246. Call Flood Test

Unknown peer sends hundreds of offers.

Expected:

```text
rate-limited
no notification storm
no codec/camera activation
```

---

# 247. Crash Test

Kill app mid-call.

Remote exits/reconnects according to timeout.

No corrupt call history.

---

# 248. Fuzzing

Part 10 fuzz:

```text
call control parser
state transition sequences
media negotiation
codec lists
stream descriptors
```

---

# 249. Property Tests

Examples:

```text
one CallId cannot be simultaneously Ended and Active
hangup is idempotent
no video stream exists when video capability not negotiated
revoked device never reaches Active
```

---

# 250. Interoperability

Part 23 should define:

```text
call/1
```

specification and vectors.

---

# 251. Call Conformance Profile

```text
offer
ring
accept
reject
cancel
hangup
media negotiation
reconnect
mute/video state
```

---

# 252. Cross-Language Call SDK

Part 19 bindings must expose same state semantics across:

```text
Kotlin
Swift
C++
Python
Dart
```

---

# 253. Performance Benchmarks

Measure:

```text
offer → ringing
accept → audio connected
accept → first video frame
path handoff interruption
reconnect time
```

---

# 254. Target UX Goals

Aim for:

```text
fast ring propagation
sub-second audio startup after accepted path is ready
minimal reconnect interruption
```

Exact targets must come from real-device benchmarks.

---

# 255. Suggested Workspace

```text
crates/
├── comm-call-core/
├── comm-call-protocol/
├── comm-call-signaling/
├── comm-call-security/
├── comm-call-media/
├── comm-call-transport/
├── comm-call-history/
├── comm-call-group/
├── comm-call-diagnostics/
└── comm-call-testkit/
```

---

# 256. `comm-call-core`

Owns:

```text
CallId
CallState
commands
events
controller
```

---

# 257. `comm-call-protocol`

Owns:

```text
call/1 wire DTOs
versioning
control sequencing
```

---

# 258. `comm-call-signaling`

Owns:

```text
offer
ring
accept
reject
cancel
multi-device arbitration
```

---

# 259. `comm-call-security`

Owns:

```text
Part 28 binding
media key derivation
security epochs
device verification
```

---

# 260. `comm-call-media`

Coordinates:

```text
Part 25 video
Part 26 audio
```

---

# 261. `comm-call-transport`

Maps media/control onto:

```text
Iroh direct
relay
multipath
```

---

# 262. `comm-call-history`

Owns private durable call metadata.

---

# 263. `comm-call-group`

Future:

```text
small mesh
group membership
SFU integration abstraction
```

---

# 264. `comm-call-testkit`

Provides:

```text
fake peer
fake transport
fake clock
packet-loss simulator
codec mock
surface mock
```

---

# 265. Controller API

```rust
pub trait CallController {
    async fn start_call(
        &self,
        peer: AccountId,
        kind: CallKind,
    ) -> Result<CallId, CallError>;

    async fn accept(&self, call: CallId) -> Result<(), CallError>;

    async fn reject(
        &self,
        call: CallId,
        reason: CallRejectReason,
    ) -> Result<(), CallError>;

    async fn hangup(&self, call: CallId) -> Result<(), CallError>;
}
```

---

# 266. Call Commands

```rust
pub enum CallCommand {
    Start,
    Accept,
    Reject,
    Cancel,
    Hangup,
    SetMuted(bool),
    SetVideo(bool),
    SetHold(bool),
    StartScreenShare,
    StopScreenShare,
    ChangeAudioRoute(AudioRoute),
}
```

---

# 267. Call Events

```rust
pub enum CallEvent {
    Offered,
    Ringing,
    Accepted,
    Connected,
    Reconnecting,
    MediaChanged,
    Held,
    Resumed,
    Ended(CallEndReason),
    Failed(CallFailure),
}
```

---

# 268. Error Model

```rust
pub enum CallError {
    PeerOffline,
    Busy,
    UnsupportedMedia,
    SecurityFailure,
    PermissionDenied,
    ResourceUnavailable,
    TransportFailure,
    MediaFailure,
    Timeout,
    InvalidState,
}
```

---

# 269. Failure Precedence

If both happen:

```text
network failure
and
user hangs up
```

local user hangup should normally become displayed reason.

Define deterministic precedence.

---

# 270. Production Rollout Phases

## Phase 1 — One-to-One Audio Calls

```text
offer
ring
accept
reject
hangup
Opus
direct/relay
```

## Phase 2 — Video Calls

```text
Part 25
codec negotiation
renderer integration
```

## Phase 3 — Resilience

```text
reconnect
network handoff
multipath
```

## Phase 4 — UX Controls

```text
mute
camera
hold
audio route
```

## Phase 5 — Screen Share

```text
second video stream
```

## Phase 6 — Multi-Device Ringing

```text
first-answer-wins
answered elsewhere
```

## Phase 7 — Security Hardening

```text
media key rotation
device revocation
abuse resistance
```

## Phase 8 — Small Group Calls

```text
mesh
active speaker
```

## Phase 9 — Larger Group Evolution

```text
SFU abstraction
simulcast/SVC later
```

---

# 271. Initial Production Recommendation

Ship:

```text
excellent 1:1 audio
excellent 1:1 video
direct + relay
fast reconnect
mute/camera/audio route
multi-device ringing
```

before attempting:

```text
large group calls
complex SFU
simulcast
SVC
call links
recording
```

---

# 272. Definition of Done

Part 29 is complete when:

- calls have a stable logical `CallId`
- call state is independent of any single transport connection
- offer/ring/accept/reject/cancel/hangup are idempotent and versioned
- multi-device ringing supports first-answer-wins
- simultaneous call glare resolves deterministically
- media negotiation supports Opus and H.264/H.265/AV1
- audio can connect before video
- local hardware/software codec implementation remains local policy
- call security is bound to Part 28 device identity/E2EE
- audio/video/control use separate logical media/security contexts
- direct/relay/multipath path changes do not create a new call
- reconnect has a bounded grace window and fresh media recovery
- audio remains higher priority than video/bulk traffic
- mute, camera toggle, hold/resume, audio route changes, and screen-share transitions are explicit
- Android Surface loss does not terminate the call
- Bluetooth/audio-route changes do not restart the logical call
- hardware codec failure can trigger media fallback/renegotiation
- call history is private durable metadata
- live media is not persisted by default
- incoming call abuse cannot activate camera/mic before acceptance
- unknown-call floods are rate-limited
- crash, reconnect, handoff, codec fallback, route-change, and long-call soak tests exist
- the `call/1` protocol is ready for Part 23 interoperability vectors

---

# 273. Relationship to Earlier Parts

Part 29 coordinates:

```text
03 — Transport / Routing Policy
07 — Capability Negotiation
08 — Resource Limits
10 — Fuzzing / Protocol Tests
11 — Relay Infrastructure
12 — Multipath
13 — Battery-Aware Scheduling
16 — Daemon / Headless
17 — Emergency Priority
18 — Diagnostics
19 — C ABI / FFI
20 — Embedded Linux
23 — Interoperability
25 — Android Zero-Copy Video
26 — Rust-First Audio DSP
27 — Android Build Automation
28 — E2EE / Key Management / Privacy
```

---

# 274. Final Architecture

```text
                       CALL CONTROLLER
                              │
             ┌────────────────┼─────────────────┐
             │                │                 │
         Signaling         Security          Policy
             │                │                 │
             └────────────────┼─────────────────┘
                              │
                        Media Session
                    ┌─────────┴─────────┐
                    │                   │
                  Audio               Video
                    │                   │
             Rust DSP + Opus      Android HW /
                    │              AV1 Software
                    └─────────┬─────────┘
                              │
                       Media Transport
                    ┌─────────┼─────────┐
                    │         │         │
                 Direct     Relay    Multipath
```

Call state:

```text
Offer
 ↓
Ringing
 ↓
Accept
 ↓
Secure Negotiation
 ↓
Audio Connected
 ↓
Video Connected
 ↓
Active
 ↓
Path / Codec / Device Changes
 ↓
Reconnect / Resume as needed
 ↓
Hangup
```

---

# 275. Final Principle

A call should survive changes in:

```text
network path
Wi-Fi/cellular state
relay/direct route
camera
audio route
video codec
renderer Surface
battery policy
```

without becoming a different logical conversation.

Therefore the architecture treats:

```text
Call
```

as a persistent logical session, while treating:

```text
transport connection
codec instance
camera
microphone route
decoder
Surface
```

as replaceable resources.

That separation is the foundation of a reliable production calling system.
