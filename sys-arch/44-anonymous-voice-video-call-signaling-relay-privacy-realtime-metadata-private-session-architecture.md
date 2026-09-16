# Core System Architecture Part 44 — Anonymous Voice/Video Call Signaling, Relay Privacy, Realtime Metadata Protection & Private Realtime Session Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 44  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:** Core System Architecture Parts 34–43  
**Primary purpose:** define the complete privacy-preserving realtime call architecture for SIAR, including anonymous call signaling, endpoint-hiding media relays, call-session keying, ICE/STUN/TURN leakage control, relay privacy modes, media metadata minimization, realtime traffic shaping, wake/privacy behavior, group calls, lifecycle, failure semantics, and provider-neutral Rust interfaces.

---

# 1. Purpose

Anonymous messaging and anonymous realtime calling are fundamentally different.

Messaging can tolerate:

```text
seconds of delay
mixing
batching
store-and-forward
cover traffic
```

Interactive calls cannot.

Voice/video typically require:

```text
low latency
stable jitter
continuous packet flow
high bandwidth
bidirectional realtime delivery
```

Therefore SIAR must not pretend that ordinary Loopix-style mixnet routing is suitable for the full media plane of an interactive call.

The governing principle is:

> **SIAR should provide strong anonymity for call signaling and strong endpoint privacy for realtime media, while describing the remaining metadata exposure accurately and never calling a relay-private media session "fully anonymous" unless the entire media path satisfies that guarantee.**

---

# 2. Architectural Position

```text
Call Invitation / Signaling
        │
        ▼
Mixnet / Anonymous Mailbox
        │
        ▼
Private Realtime Session Setup
        │
        ├── Relay-private media
        ├── Multi-relay media
        ├── Anonymous signaling only
        └── Future low-latency anonymous media
        │
        ▼
Encrypted Realtime Media
```

---

# 3. Core Separation

Keep distinct:

```text
Signaling Privacy
Media Content Security
Endpoint Privacy
Traffic Metadata Privacy
```

---

# 4. Security Properties

Call content must provide:

```text
E2EE
authentication
forward secrecy
session key rotation
```

---

# 5. Privacy Properties

Depending on mode:

```text
hide peer IP
hide call invitation relationship
reduce relay knowledge
reduce call timing metadata
reduce device/media fingerprinting
```

---

# 6. Realtime Privacy Modes

```rust
pub enum RealtimePrivacyMode {
    Standard,
    RelayPrivate,
    AnonymousSignaling,
    MultiRelayPrivate,
    MaximumRealtimePrivacy,
}
```

---

# 7. Standard

Normal SIAR call.

Possible direct peer media.

Lowest latency.

---

# 8. RelayPrivate

Media must use relay.

Peers do not learn each other's IP addresses.

---

# 9. AnonymousSignaling

Call setup/signaling goes through:

```text
mixnet
anonymous mailbox
```

Media uses approved privacy relay.

---

# 10. MultiRelayPrivate

Media traverses:

```text
two independently operated relay layers
```

where practical.

---

# 11. MaximumRealtimePrivacy

Strictest practical realtime profile.

May require:

```text
anonymous signaling
no direct ICE candidates
relay-only media
operator diversity
traffic padding where practical
privacy-safe push/wake
no direct fallback
```

---

# 12. Important Naming Rule

Do not call this:

```text
Maximum Anonymity
```

unless the media-path threat model supports that statement.

Prefer:

```text
Maximum Realtime Privacy
```

---

# 13. Call Types

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

# 14. Call Session ID

```rust
pub struct PrivateCallId(pub [u8; 16]);
```

Random.

---

# 15. Call ID Scope

Do not expose:

```text
ConversationId
AccountId
DeviceId
```

to relay provider.

---

# 16. Anonymous Call Invitation

```rust
pub struct AnonymousCallInvite {
    pub call_id: PrivateCallId,
    pub call_kind: CallKind,
    pub offer: EncryptedCallOffer,
    pub expires_at: Timestamp,
    pub reply_capability: Option<AnonymousReplyCapability>,
}
```

---

# 17. Signaling Path

Recommended:

```text
caller
→ E2EE call invite
→ mixnet
→ callee mailbox
→ callee
```

---

# 18. Call Accept Path

```text
callee
→ anonymous reply capability
→ mixnet
→ caller
```

---

# 19. Signaling Must Not Contain Raw IP Candidates

Hard rule for strict modes.

---

# 20. SDP/ICE Metadata

Conventional realtime stacks can leak:

```text
local IP
public IP
interface addresses
hostnames
codec/device detail
```

---

# 21. Strict Signaling Sanitization

Remove/avoid:

```text
host candidates
server-reflexive candidates
local interface addresses
device labels
```

---

# 22. ICE Policy

```rust
pub enum IcePrivacyPolicy {
    Normal,
    RelayOnly,
    RelayOnlyNoSrflxExposure,
}
```

---

# 23. Relay-Only

For privacy modes:

```text
do not expose host candidates
do not expose direct public address
```

---

# 24. STUN

STUN can reveal public IP to STUN server.

---

# 25. STUN Policy

In strict mode:

```text
avoid standalone STUN where not necessary
```

Use relay allocation path.

---

# 26. TURN-Like Relay

Relay sees:

```text
client IP
call timing
traffic volume
```

but peers do not see each other.

---

# 27. Relay Trust

Relay is a metadata observer.

Therefore one relay gives:

```text
endpoint privacy from peer
```

not full anonymity from infrastructure.

---

# 28. Multi-Relay Concept

```text
Caller
→ Relay A
→ Relay B
→ Callee
```

---

# 29. Operator Diversity

Relay A and B should be independently operated.

---

# 30. Multi-Relay Benefit

No single relay necessarily knows both endpoints.

---

# 31. Multi-Relay Cost

Adds:

```text
latency
jitter
bandwidth
complexity
```

---

# 32. Multi-Relay Applicability

Most suitable for:

```text
audio
low-resolution video
```

depending network quality.

---

# 33. Relay Chain

```rust
pub struct RealtimeRelayChain {
    pub hops: Vec<RealtimeRelayDescriptor>,
}
```

---

# 34. Relay Descriptor

```rust
pub struct RealtimeRelayDescriptor {
    pub relay_id: RealtimeRelayId,
    pub endpoint: RelayEndpoint,
    pub operator: OperatorId,
    pub capabilities: RealtimeRelayCapabilities,
}
```

---

# 35. Relay Capabilities

```rust
pub struct RealtimeRelayCapabilities {
    pub udp: bool,
    pub quic_datagram: bool,
    pub tcp_fallback: bool,
    pub max_bitrate: u64,
    pub supports_padding: bool,
}
```

---

# 36. Relay Selection

Hard constraints first:

```text
privacy
operator diversity
capacity
health
```

Then:

```text
latency
geography
```

---

# 37. Route Selection Trait

```rust
pub trait RealtimeRelaySelector {
    fn select(
        &self,
        policy: &RealtimeRoutePolicy,
        available: &[RealtimeRelayDescriptor],
    ) -> Result<RealtimeRelayChain, RealtimeRouteError>;
}
```

---

# 38. Realtime Route Policy

```rust
pub struct RealtimeRoutePolicy {
    pub privacy_mode: RealtimePrivacyMode,
    pub max_hops: u8,
    pub require_operator_diversity: bool,
    pub allow_tcp_fallback: bool,
}
```

---

# 39. No Silent Direct Fallback

Strict mode:

```text
relay unavailable
≠
direct call
```

---

# 40. Failure UX

```text
Private relay path unavailable.
Call was not connected directly.
```

---

# 41. Ask-Before-Downgrade

Optional policy:

```text
Use a less-private direct connection for this call?
```

Explicit.

---

# 42. Downgrade Scope

Default:

```text
this call only
```

---

# 43. Call Session Keying

Use independent call-session E2EE keys.

---

# 44. Key Separation

Separate:

```text
messaging ratchet keys
call signaling keys
media session keys
relay credentials
```

---

# 45. Media Key Agreement

Authenticated through existing SIAR identity/session trust.

---

# 46. Relay Never Receives Media Plaintext

Hard rule.

---

# 47. Media Encryption

Audio/video encrypted end-to-end before relay forwarding.

---

# 48. Media Stream IDs

Provider-facing IDs random.

---

# 49. No Account IDs in Relay Protocol

Hard rule.

---

# 50. Media Transport

Potential:

```text
QUIC datagrams
SRTP-like encrypted RTP
custom low-latency encrypted datagrams
```

---

# 51. Architecture Preference

Keep:

```text
media session abstraction
```

above transport.

---

# 52. Realtime Media Trait

```rust
#[async_trait]
pub trait PrivateRealtimeTransport: Send + Sync {
    async fn establish(
        &self,
        request: PrivateRealtimeSessionRequest,
    ) -> Result<PrivateRealtimeSession, RealtimeError>;

    async fn send_media(
        &self,
        session: &PrivateRealtimeSession,
        packet: EncryptedMediaPacket,
    ) -> Result<(), RealtimeError>;

    async fn close(
        &self,
        session: &PrivateRealtimeSession,
    ) -> Result<(), RealtimeError>;
}
```

---

# 53. Media Packet

```rust
pub struct EncryptedMediaPacket {
    pub stream_id: RealtimeStreamId,
    pub sequence: u64,
    pub payload: Bytes,
}
```

---

# 54. Sequence Privacy

Sequence number may be encrypted/authenticated depending transport.

---

# 55. Audio Codec

Existing architecture preference:

```text
Opus
```

---

# 56. Video Codec

Desktop:

```text
AV1 software
```

Android:

```text
hardware H.264/H.265/AV1
+
software AV1 fallback
```

Codec negotiation remains per direction.

---

# 57. Codec Metadata Leakage

Codec choice can fingerprint:

```text
device
platform
capability
```

---

# 58. Privacy Negotiation

Strict mode can reduce exposed codec list to common profile.

---

# 59. Codec Profile

```rust
pub enum RealtimeCodecPrivacyProfile {
    BestQuality,
    CommonSubset,
    MaximumPrivacy,
}
```

---

# 60. Maximum Privacy Codec Profile

Advertise minimal standard set.

---

# 61. Resolution Leakage

Video resolution reveals:

```text
device capability
network quality
```

---

# 62. Resolution Bucketing

Use coarse profiles:

```text
Low
Medium
High
```

---

# 63. Bitrate Leakage

Call traffic volume is inherently observable.

---

# 64. Constant Bitrate

Audio can use approximately bounded bitrate.

---

# 65. Video Padding

Can reduce bitrate variation at substantial bandwidth cost.

---

# 66. Realtime Padding Policy

```rust
pub enum RealtimePaddingPolicy {
    None,
    AudioOnly,
    CoarseBucket,
    Aggressive,
}
```

---

# 67. Audio Padding

Useful because voice activity creates obvious on/off pattern.

---

# 68. Silence Suppression

Good for bandwidth.

Bad for privacy.

---

# 69. Strict Mode

May disable aggressive silence suppression.

---

# 70. Comfort Noise

Can maintain traffic continuity.

---

# 71. Audio Traffic Shaping

Potential:

```text
fixed packet interval
coarse packet size
cover audio packets during silence
```

---

# 72. Video Traffic Shaping

Harder.

Can smooth bitrate but not fully hide scene complexity.

---

# 73. Honest Limitation

High-resolution video remains highly fingerprintable by volume.

---

# 74. Screen Share

Even more bursty.

---

# 75. Strict Screen Share

Can require relay privacy but not claim strong traffic unobservability.

---

# 76. Call Timing Leakage

Relay sees:

```text
call start
call duration
traffic volume
```

---

# 77. Mixnet Signaling Benefit

Hides relationship during:

```text
invite
accept
decline
```

from normal network path.

---

# 78. Media Path Residual Risk

Realtime relay still exposes:

```text
session timing
```

to relay infrastructure.

---

# 79. Call Wake

Incoming-call push is difficult for anonymity.

---

# 80. Push Wake Privacy

Push provider can learn:

```text
incoming call timing
```

---

# 81. Maximum Realtime Privacy

May disable normal push.

---

# 82. Alternative

Persistent anonymous mailbox/daemon receive.

---

# 83. Mobile Tradeoff

Always-on anonymous receive costs battery.

---

# 84. Wake Modes

```rust
pub enum PrivateCallWakeMode {
    PushHint,
    AnonymousPersistentReceive,
    ForegroundOnly,
}
```

---

# 85. Push Hint

Opaque.

No:

```text
caller ID
call ID
conversation ID
```

---

# 86. Push Dedup

Opaque wake token only.

---

# 87. After Wake

App retrieves actual invite through:

```text
anonymous mailbox
```

---

# 88. Incoming Call Authentication

Never ring before invite authentication.

---

# 89. Caller Display

Resolved locally from authenticated application identity.

---

# 90. Relay Credentials

Issued via encrypted signaling.

---

# 91. Relay Credential Scope

Bound to:

```text
call
relay
expiry
bandwidth class
```

---

# 92. Relay Credential

```rust
pub struct RealtimeRelayCapability(SecretBytes);
```

---

# 93. Credential Secrecy

Never log.

---

# 94. Relay Admission

Authenticate cheaply before allocating large media resources.

---

# 95. DoS Protection

Bound:

```text
sessions per capability
bitrate
duration
```

---

# 96. Call Spam

Anonymous call invitations can be abusive.

---

# 97. Unknown Caller Policy

Default:

```text
requests
silent notification
no auto-ring
```

depending privacy/settings.

---

# 98. Call Capability

Trusted contacts can receive call capability.

---

# 99. Call Invitation Capability

```rust
pub struct CallInvitationCapability(SecretBytes);
```

---

# 100. Capability Scope

Can limit:

```text
audio only
video allowed
expiry
call count
```

---

# 101. Anonymous Contact Calls

May require one-time invitation capability.

---

# 102. Blocking

Blocked contact's call capabilities revoked/ignored.

---

# 103. Group Calls

More complex.

---

# 104. Group Call Architecture

Potential:

```text
SFU
MCU
mesh
```

---

# 105. Peer Mesh

Leaks peer addresses unless relayed.

Not suitable for strict privacy.

---

# 106. Privacy-Preserving SFU

Recommended initial scalable approach.

---

# 107. SFU Sees

```text
encrypted media streams
timing
bitrate
participant connections
```

---

# 108. E2EE with SFU

Media remains end-to-end encrypted.

---

# 109. SFU Participant Privacy

SFU can still correlate participants.

---

# 110. Multi-SFU Chain

Possible but expensive.

---

# 111. Group Call Privacy Modes

```rust
pub enum GroupCallPrivacyMode {
    RelayPrivate,
    AnonymousSignaling,
    MultiRelay,
}
```

---

# 112. Group Call Membership Privacy

Strict anonymous group call may not reveal full participant identities.

---

# 113. Group-Local Pseudonyms

Reuse Part 43 group-local identities.

---

# 114. Participant Stream ID

Random per call.

---

# 115. SFU Does Not Receive AccountId

Hard rule.

---

# 116. Speaking Indicator

Can leak activity.

---

# 117. Strict Mode

Local UI derives active speaker from decrypted media.

Do not publish separate identity-rich speaking events externally.

---

# 118. Participant Join/Leave

Group call participants necessarily observe some session presence.

---

# 119. Hide Exact Group Roster

Possible:

```text
only active participants
```

not full group membership.

---

# 120. Group Call Keying

Use group-call session key schedule.

---

# 121. Key Rotation

On:

```text
participant join
participant leave
device removal
```

depending protocol.

---

# 122. Large Group Calls

SFU cluster.

---

# 123. SFU Sharding

Can shard by:

```text
region
capacity
```

---

# 124. Privacy Constraint

Do not shard based on user identity.

---

# 125. SFU Operator Diversity

Future multi-relay mode.

---

# 126. Network Path

Caller should not see callee network endpoint.

---

# 127. NAT Traversal

Strict relay mode avoids direct hole punching.

---

# 128. ICE Candidate Policy

Only relay candidates.

---

# 129. Host Candidate Collection

Can be disabled in strict mode.

---

# 130. mDNS Candidates

Still unnecessary if direct disabled.

---

# 131. TURN Allocation

Use privacy-preserving credentials.

---

# 132. Relay DNS

Potential leak.

Use trusted resolver/access path.

---

# 133. Censorship Resistance

Part 41 pluggable transports may carry relay control.

---

# 134. Media Over Restricted Networks

Possible fallback:

```text
UDP
→ QUIC
→ TCP
→ WebSocket-like tunnel
```

but latency worsens.

---

# 135. No Direct Fallback

Still enforced.

---

# 136. Media Access Resolver

```rust
pub trait RealtimeAccessResolver {
    fn resolve(
        &self,
        policy: &RealtimeRoutePolicy,
        network: &NetworkCapabilities,
    ) -> Result<RealtimeAccessPlan, RealtimeRouteError>;
}
```

---

# 137. Realtime Access Plan

```rust
pub struct RealtimeAccessPlan {
    pub relay_chain: RealtimeRelayChain,
    pub transport: RealtimeMediaTransportKind,
}
```

---

# 138. Media Transport Kinds

```rust
pub enum RealtimeMediaTransportKind {
    Udp,
    QuicDatagram,
    Tcp,
    WebSocketTunnel,
}
```

---

# 139. Preferred Order

Normally:

```text
UDP/QUIC datagram
```

for realtime.

---

# 140. TCP

Fallback only.

---

# 141. Quality Degradation

Strict privacy path may reduce call quality.

---

# 142. UX

Explain:

```text
Private relay path is increasing call latency.
```

---

# 143. Do Not Suggest Direct Automatically

Hard rule.

---

# 144. Adaptive Media Quality

Reduce:

```text
resolution
frame rate
bitrate
```

before privacy downgrade.

---

# 145. Audio Priority

Under congestion:

```text
preserve audio
degrade video
```

---

# 146. Video Disable

If necessary.

---

# 147. Relay Congestion

Use:

```text
bitrate adaptation
relay reselection
```

---

# 148. Mid-Call Relay Migration

Possible.

---

# 149. Migration Privacy

New relay credentials via encrypted signaling.

---

# 150. Seamless Migration

Maintain same logical CallId.

---

# 151. Relay Failure

Try approved alternate private relay.

---

# 152. Multi-Relay Failure

Can collapse to one relay only if policy allows.

---

# 153. Strict Mode

If required multi-relay unavailable:

```text
call reconnecting / ends
```

not silent weaker route.

---

# 154. Call State Machine

```rust
pub enum PrivateCallState {
    Idle,
    Inviting,
    Ringing,
    Establishing,
    Connected,
    Reconnecting,
    Ending,
    Ended,
    Failed,
}
```

---

# 155. Privacy Route State

```rust
pub enum RealtimePrivacyRouteState {
    Satisfied,
    DegradedWithinPolicy,
    RequiresDowngrade,
    Unavailable,
}
```

---

# 156. UI Must Distinguish

```text
call quality degraded
```

from:

```text
privacy guarantee degraded
```

---

# 157. Privacy Badge

Possible:

```text
Relay Private
Multi-Relay Private
Anonymous Signaling
```

---

# 158. Do Not Show Raw Relay Hops

Normal UI.

---

# 159. Advanced Diagnostics

Could show:

```text
relay count
operator diversity satisfied
transport
codec
RTT/jitter/loss
```

---

# 160. Forbidden Diagnostics

No:

```text
relay credential
peer IP
full route identifiers
```

---

# 161. Audio Route

Device-local:

```text
speaker
earpiece
wired
Bluetooth
```

No effect on network anonymity.

---

# 162. Bluetooth Device Name

May be sensitive locally.

Do not send to peer/relay.

---

# 163. Camera Metadata

Do not send:

```text
camera model
sensor ID
```

unless required.

---

# 164. Device Capability Minimization

Advertise:

```text
supported privacy-safe codec profile
```

not full hardware fingerprint.

---

# 165. Video Orientation

Necessary metadata.

---

# 166. Resolution Negotiation

Use coarse bucket.

---

# 167. Hardware Encoder Fingerprint

Packet patterns can differ.

Hard to eliminate fully.

Document residual risk.

---

# 168. Screen Sharing

Capture permission remains platform-controlled.

---

# 169. Screen Share Notification

OS indicators remain.

---

# 170. Application Privacy

Do not include window/app names in signaling unless user explicitly chooses.

---

# 171. Call Recording

Local feature only unless all parties informed per product/legal policy.

---

# 172. Recording Privacy

Recording is separate from transport privacy.

---

# 173. Call History

Store locally encrypted.

---

# 174. Call History Privacy

Maximum privacy may allow:

```text
disable call history
```

---

# 175. Relay Logs

Minimize:

```text
session timing
bytes
errors
```

---

# 176. No Caller Identity Logs

Hard rule.

---

# 177. IP Retention

Operationally minimize.

---

# 178. Abuse/Security Logs

If needed:

```text
short-lived
policy-documented
```

---

# 179. Privacy-Safe Metrics

Aggregate:

```text
call success
relay success
latency buckets
packet loss buckets
codec usage
```

---

# 180. Forbidden Metrics

No:

```text
CallId
AccountId
DeviceId
peer pair
```

---

# 181. Support Bundle

Redact:

```text
relay credentials
peer endpoints
call invitation capabilities
```

---

# 182. Call Signaling Database

Potential:

```text
private_call_sessions
private_call_invites
private_relay_allocations
private_call_events
```

---

# 183. Durable vs Ephemeral

Durable:

```text
call invite state
privacy policy
```

Ephemeral:

```text
media sequence
relay packet state
jitter buffer
```

---

# 184. Crash Recovery

Call media session cannot simply be restored.

---

# 185. Android Process Death

If foreground-service call survives:

```text
rebind UI
```

otherwise:

```text
call ends/reconnects
```

---

# 186. Fresh Media Keys

After process restart/re-establish.

---

# 187. Desktop Daemon

Can own call signaling/session state.

---

# 188. UI Reattach

Dioxus window can reconnect to active call session.

---

# 189. Android Foreground Service

Required for ongoing call behavior.

---

# 190. PiP

Video UI can enter PiP.

---

# 191. Surface Rebinding

Media surfaces can be rebound without changing privacy route.

---

# 192. Background Camera/Mic Rules

Follow platform restrictions.

---

# 193. Push Permission

Declining notifications means incoming calls may only appear when app active unless persistent anonymous receive enabled.

---

# 194. Call Wake Policy

Must communicate.

---

# 195. Incoming Call Privacy on Lock Screen

Respect preview setting.

---

# 196. Hidden Caller Preview

Could show:

```text
Incoming private call
```

instead of identity.

---

# 197. Caller Identity Reveal

After unlock/setting allows.

---

# 198. Emergency Calls

Do not imply connection to official emergency services.

---

# 199. Emergency vs Privacy

If user prioritizes reachability, direct/emergency transport may be separate explicit mode.

---

# 200. No Silent Emergency Override

Hard rule.

---

# 201. Contact Verification

Call signaling inherits contact identity trust.

---

# 202. Identity Change

If peer identity changed:

```text
warn before establishing high-trust private call
```

---

# 203. Call Key Verification

Could expose:

```text
SAS/fingerprint
```

for sensitive calls.

---

# 204. Verification UI

Optional:

```text
Verify call
```

---

# 205. SAS

Derived from authenticated media session keys.

---

# 206. Relay Cannot Forge SAS

Assuming E2EE/authentication correct.

---

# 207. Group Call Verification

More complex.

Could display:

```text
group security state
```

rather than pairwise SAS for large calls.

---

# 208. Realtime Session Trait

```rust
#[async_trait]
pub trait PrivateCallService: Send + Sync {
    async fn create_invite(
        &self,
        request: PrivateCallInviteRequest,
    ) -> Result<PrivateCallId, RealtimeError>;

    async fn accept(
        &self,
        invite: PrivateCallId,
    ) -> Result<PrivateRealtimeSession, RealtimeError>;

    async fn decline(
        &self,
        invite: PrivateCallId,
    ) -> Result<(), RealtimeError>;

    async fn end(
        &self,
        call: PrivateCallId,
    ) -> Result<(), RealtimeError>;
}
```

---

# 209. Relay Allocation Service

```rust
#[async_trait]
pub trait PrivateRelayAllocator: Send + Sync {
    async fn allocate(
        &self,
        policy: &RealtimeRoutePolicy,
    ) -> Result<RealtimeRelayChain, RealtimeRouteError>;

    async fn migrate(
        &self,
        current: &RealtimeRelayChain,
    ) -> Result<RealtimeRelayChain, RealtimeRouteError>;
}
```

---

# 210. Media Privacy Policy

```rust
pub struct MediaPrivacyPolicy {
    pub mode: RealtimePrivacyMode,
    pub ice: IcePrivacyPolicy,
    pub padding: RealtimePaddingPolicy,
    pub codec_profile: RealtimeCodecPrivacyProfile,
    pub allow_direct_fallback: bool,
}
```

---

# 211. Default Strict Values

```text
RelayOnlyNoSrflxExposure
No direct fallback
CommonSubset codec profile
Audio padding/coarse traffic shaping
```

---

# 212. Error Taxonomy

```rust
pub enum RealtimeError {
    InviteExpired,
    AnonymousSignalingUnavailable,
    RelayUnavailable,
    PrivacyRequirementUnsatisfied,
    MediaKeyFailure,
    UnsupportedCodec,
    NetworkRestricted,
    CallRejected,
    Timeout,
    Internal,
}
```

---

# 213. Privacy Requirement Unsatisfied

Distinct from:

```text
network quality poor
```

---

# 214. Testing Strategy

Need:

```text
unit
integration
network simulation
privacy
media quality
lifecycle
adversarial
```

---

# 215. Signaling Tests

Verify:

```text
invite via mixnet
accept via reply capability
expiry
dedup
```

---

# 216. ICE Leak Tests

Strict mode must not emit:

```text
host candidate
srflx candidate to peer
```

---

# 217. Direct Fallback Test

Relay unavailable:

```text
strict call fails/waits
```

---

# 218. Peer IP Leak Test

Peer never receives opposite endpoint IP in relay-private mode.

---

# 219. Relay Credential Leak Test

Canary absent from logs/UI/support bundle.

---

# 220. Codec Fingerprint Test

Strict profile exposes only approved codec subset.

---

# 221. Audio Timing Test

Compare:

```text
speech
silence
```

under padding policy.

---

# 222. Video Volume Test

Measure residual leakage.

---

# 223. Wake Privacy Test

Push payload opaque.

---

# 224. Process Death Test

Android call service behavior correct.

---

# 225. Relay Migration Test

Mid-call migration preserves logical session.

---

# 226. Multi-Relay Test

Independent operators enforced.

---

# 227. Operator Failure Test

One relay dies.

---

# 228. Group Call Test

SFU E2EE, pseudonymous participant IDs.

---

# 229. Group Join/Leave Rekey Test

Old participant cannot decrypt future group-call media where protocol requires.

---

# 230. Abuse Tests

```text
call invite flood
relay allocation flood
bandwidth abuse
invalid capability
```

---

# 231. Fuzzing

Fuzz:

```text
call invite parser
relay descriptor
media control messages
codec negotiation
```

---

# 232. Property Tests

Properties:

```text
strict privacy never selects direct path
relay credential scoped to one call
peer address never enters peer signaling DTO
expired invite cannot establish
```

---

# 233. Formal Verification Targets

Candidates:

```text
privacy route resolver
call state machine
relay migration
downgrade policy
```

---

# 234. Performance Tests

Measure:

```text
one-relay latency
two-relay latency
jitter
packet loss
CPU
battery
bandwidth
padding overhead
```

---

# 235. Quality Thresholds

Define separate targets for:

```text
audio
video
screen share
```

---

# 236. Privacy Lab Integration

Part 42 should evaluate:

```text
call timing correlation
relay pair correlation
audio activity inference
video bitrate fingerprinting
```

---

# 237. Honest Residual Risks

Even strict realtime privacy may still leak:

```text
call duration
session timing
high bandwidth activity
relay connection
```

---

# 238. Release Gate

Block strict private-call feature if:

```text
peer IP leak
silent direct fallback
relay credential leak
unauthenticated call ringing
```

---

# 239. Crate Layout

Recommended:

```text
crates/
├── siar-private-call-core/
├── siar-private-call-signaling/
├── siar-private-call-keys/
├── siar-private-call-relay/
├── siar-private-call-route/
├── siar-private-call-media/
├── siar-private-call-padding/
├── siar-private-call-group/
├── siar-private-call-platform/
├── siar-private-call-observability/
└── siar-private-call-testkit/
```

---

# 240. `siar-private-call-core`

Owns:

```text
CallId
privacy modes
call state
errors
```

---

# 241. `siar-private-call-signaling`

Anonymous invite/accept/decline.

---

# 242. `siar-private-call-keys`

Call-session E2EE.

---

# 243. `siar-private-call-relay`

Relay allocation/credentials.

---

# 244. `siar-private-call-route`

One-relay/multi-relay selection.

---

# 245. `siar-private-call-media`

Encrypted audio/video transport.

---

# 246. `siar-private-call-padding`

Realtime metadata shaping.

---

# 247. `siar-private-call-group`

SFU/group-call privacy.

---

# 248. `siar-private-call-platform`

Android/Desktop lifecycle/audio-video integration.

---

# 249. `siar-private-call-observability`

Privacy-safe diagnostics/metrics.

---

# 250. `siar-private-call-testkit`

Fake relays/network faults.

---

# 251. Initial Production Scope

Implement first:

```text
anonymous call signaling over mixnet
relay-only media
no host/direct ICE candidates
peer IP hiding
independent media E2EE
opaque push wake
strict no-direct-fallback
audio padding option
common codec privacy profile
relay migration
Android foreground-service lifecycle
desktop daemon integration
privacy-safe diagnostics
```

Then add:

```text
multi-relay media
group-call private SFU
advanced video shaping
anonymous persistent call wake
relay operator diversity
low-latency anonymity research
```

---

# 252. Definition of Done

Part 44 is complete when:

- signaling and media privacy are cleanly separated
- call invitations/acceptance can travel through mixnet/mailbox
- strict modes never expose raw direct ICE candidates to peers
- relay-only media hides peer network addresses
- media remains end-to-end encrypted through relays
- one-relay and multi-relay privacy models are explicit
- operator diversity rules exist for multi-relay mode
- codec/resolution/device metadata is minimized
- audio/video traffic-shaping limitations are documented
- push wake contains only opaque hints
- group-call SFU privacy boundaries are defined
- Android/Desktop lifecycle and process-death behavior are defined
- relay migration and failure semantics preserve privacy policy
- strict no-direct-fallback behavior is release-tested
- privacy diagnostics do not expose relay credentials/peer IPs
- traffic-analysis residual risks are fed into Part 42 verification

---

# 253. Final Architecture

```text
                     CALLER
                       │
              E2EE Anonymous Invite
                       │
                       ▼
                    Mixnet
                       │
                       ▼
                    CALLEE
                       │
              Anonymous Acceptance
                       │
                       ▼
             Private Relay Allocation
                       │
             ┌─────────┴─────────┐
             │                   │
          Relay A             Relay B
             │                   │
             └─────────┬─────────┘
                       ▼
               E2EE Realtime Media
```

One-relay private path:

```text
Caller
  ↓
Encrypted Media
  ↓
Relay
  ↓
Encrypted Media
  ↓
Callee
```

Strict rule:

```text
Private relay unavailable
=
call waits/fails

not

direct peer fallback
```

---

# 254. Final Principle

Realtime privacy is not the same problem as asynchronous mixnet messaging.

The correct model is:

```text
mixnet-protected signaling
+
relay-only endpoint hiding
+
media E2EE
+
optional multi-relay separation
+
metadata-minimized negotiation
+
honest traffic-analysis limitations
```

not:

```text
route live video through a high-latency mixnet
```

This architecture gives SIAR a practical path to private voice/video calls while preserving strong signaling anonymity and peer endpoint privacy without making misleading claims about what realtime media can hide.
