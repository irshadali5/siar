# Part 07 — Capability Negotiation Architecture

## Reusable P2P Communication Platform

**Status:** Architecture specification  
**Part:** 07 of 24  
**Primary language:** Rust  
**Primary goals:** safe protocol compatibility, feature discovery, version negotiation, transport/media/file/DTN capability exchange, downgrade resistance, efficient session setup, cross-platform interoperability, reusable across messaging/files/calls/emergency/ERP/custom products

---

# 1. Purpose

A reusable communication platform cannot assume every peer has the same:

- application version
- protocol version
- transport support
- media codecs
- hardware capabilities
- file-transfer features
- DTN features
- storage limits
- background capabilities
- security algorithms
- device class
- platform restrictions

Two peers must first answer:

```text
What do you support?
What do I support?
What is mutually usable?
What is required?
What must be rejected?
What may safely degrade?
```

This is the responsibility of the capability negotiation subsystem.

Its purpose is to create a **verified, bounded, versioned capability contract** for every authenticated session and every reusable extension.

---

# 2. Fundamental Rule

Do not use:

```text
if app_version >= X
```

as a substitute for capability negotiation.

Do not infer:

```text
Android = H.264
Desktop = AV1 software
Iroh = files
```

Instead:

```text
Authenticated Peer
      ↓
Capability Advertisement
      ↓
Version / Policy Filtering
      ↓
Mutual Capability Set
      ↓
Per-Extension Negotiation
      ↓
Session Capability Contract
```

The rest of the platform consumes the negotiated result.

---

# 3. Architectural Position

```text
Transport Connected
      ↓
Identity Authentication
      ↓
Core Protocol Handshake
      ↓
Capability Negotiation
      ↓
Extension Activation
      ↓
Messaging / Files / Calls / DTN / Custom
```

Capability negotiation depends on:

```text
Part 01 — Protocol Extension System
Part 02 — Multi-Device Identity
```

and feeds:

```text
Part 03 — Routing
Part 05 — Files
Part 06 — DTN
media/calls
UI
```

---

# 4. Capability Categories

Capabilities should be grouped into domains.

Recommended categories:

```text
Core Protocol
Security
Transport
Messaging
Files
Groups
Presence
DTN
Media
Platform
Device Resources
Application Extensions
```

Do not create one unstructured giant map.

---

# 5. Capability Identifier

Use stable typed IDs.

```rust
pub struct CapabilityId {
    pub namespace: CapabilityNamespace,
    pub code: u32,
}
```

Example canonical names:

```text
core.protocol.v1
files.resume
files.parallel_chunks
dtn.spray_wait
media.video.av1.software
media.video.h264.hardware
platform.wifi_aware
```

Wire representation can be compact numeric IDs after namespace agreement.

---

# 6. Capability Namespace

```rust
pub enum CapabilityNamespace {
    Core,
    Security,
    Transport,
    Messaging,
    Files,
    Groups,
    Presence,
    Dtn,
    Media,
    Platform,
    Resource,
    Application(NamespaceId),
}
```

This prevents collisions and supports third-party extensions.

---

# 7. Capability Descriptor

```rust
pub struct CapabilityDescriptor {
    pub id: CapabilityId,
    pub version: CapabilityVersion,
    pub requirement: CapabilityRequirement,
    pub parameters: CapabilityParameters,
    pub stability: CapabilityStability,
}
```

---

# 8. Required vs Optional

```rust
pub enum CapabilityRequirement {
    Required,
    Optional,
}
```

Required means:

```text
this operation/extension cannot work safely without it
```

Optional means:

```text
use if mutually supported
```

Unknown required capability:

```text
negotiation failure
```

Unknown optional capability:

```text
ignore safely
```

---

# 9. Capability Versioning

A capability may have its own version:

```rust
pub struct CapabilityVersion {
    pub major: u16,
    pub minor: u16,
}
```

Do not make all feature evolution depend only on application version.

---

# 10. Protocol Version vs Capability Version

Separate:

```text
files/1 protocol
```

from:

```text
files.resume capability v2
```

This lets a stable protocol major gain backward-compatible features.

---

# 11. Parameterized Capabilities

Some capabilities need parameters.

Example:

```text
files.max_chunk_size = 4 MiB
files.max_parallel = 8
media.max_video_width = 3840
dtn.max_bundle_size = 1 MiB
```

Use typed bounded parameter structures.

Do not use:

```text
HashMap<String, String>
```

for core behavior.

---

# 12. Capability Parameters

Conceptually:

```rust
pub enum CapabilityParameters {
    None,
    U32(u32),
    U64(u64),
    RangeU32 { min: u32, max: u32 },
    BitSet(CapabilityBits),
    Bytes(BoundedBytes),
}
```

Prefer domain-specific structs for complex capability groups.

---

# 13. Capability Advertisement

Each peer sends an authenticated advertisement.

```rust
pub struct CapabilityAdvertisement {
    pub device_id: DeviceId,
    pub account_generation: u64,
    pub core_protocols: Vec<ProtocolSupport>,
    pub extensions: Vec<ExtensionCapabilitySet>,
    pub security: SecurityCapabilities,
    pub transport: TransportCapabilitiesSummary,
    pub platform: PlatformCapabilitiesSummary,
    pub resources: ResourceCapabilities,
    pub issued_at: Timestamp,
    pub nonce: HandshakeNonce,
}
```

The advertisement is bound to the authenticated session transcript.

---

# 14. Authentication Binding

Capabilities must not be accepted before peer identity is authenticated.

Otherwise attacker could advertise:

```text
fake codec
fake protocol
fake relay
fake device features
```

Handshake binding should include:

```text
peer identity
session transcript
capability advertisement hash
```

---

# 15. Replay Resistance

Capability advertisements must be freshness-bound.

Use:

```text
session nonce
transcript hash
short lifetime
```

Do not accept old signed capability sets as current runtime truth.

---

# 16. Device Certificate vs Runtime Capability

Part 02 device certificate may contain durable capability classes.

Example:

```text
headless
relay
link authority
```

Runtime negotiation contains ephemeral/current capabilities:

```text
Wi-Fi enabled
AV1 encoder currently available
storage quota
battery policy
```

Do not confuse them.

---

# 17. Durable vs Ephemeral Capabilities

```rust
pub enum CapabilityLifetime {
    IdentityBound,
    SessionBound,
    Dynamic,
}
```

Examples:

```text
Device role → IdentityBound
files.resume support → SessionBound
battery available → Dynamic
Wi-Fi enabled → Dynamic
```

---

# 18. Negotiation Output

```rust
pub struct NegotiatedCapabilities {
    pub core: NegotiatedCore,
    pub extensions: HashMap<ProtocolId, NegotiatedExtensionCapabilities>,
    pub security: NegotiatedSecurity,
    pub transport: NegotiatedTransportHints,
    pub media: NegotiatedMedia,
    pub limits: NegotiatedLimits,
}
```

This object is immutable for the session generation.

Dynamic changes create updates/versioned snapshots.

---

# 19. Intersection Rule

For simple Boolean capabilities:

```text
negotiated = local ∩ remote
```

For ranges:

```text
negotiated range = overlap(local, remote)
```

For maximum limits:

```text
effective max = min(local_max, remote_max, policy_max)
```

For security:

```text
effective = strongest mutually supported and locally permitted
```

Do not always use simple intersection.

---

# 20. Policy Filter

Negotiation is:

```text
local support
∩
remote support
∩
local security policy
∩
application policy
∩
runtime platform availability
```

A capability can be supported by code but disabled by policy.

---

# 21. Hard Policy

Examples:

```text
minimum protocol version
disallow insecure algorithm
forbid public relay
max accepted file size
```

Hard policy cannot be overridden by peer advertisement.

---

# 22. User Policy

Examples:

```text
disable read receipts
disable nearby forwarding
disable mobile-data files
disable H.265 due compatibility concerns
```

User policy changes effective capabilities.

---

# 23. Application Policy

ERP example:

```text
files required
DTN disabled
messaging optional
```

Emergency app:

```text
DTN required
nearby required where platform supports
```

---

# 24. Negotiation State Machine

```text
TransportConnected
       ↓
IdentityAuthenticated
       ↓
CapabilitiesOffered
       ↓
CapabilitiesReceived
       ↓
Validate
       ↓
Intersect
       ↓
PolicyFilter
       ↓
RequiredCheck
       ↓
Confirmed
       ↓
Active
```

Failure states:

```text
VersionMismatch
RequiredCapabilityMissing
SecurityPolicyFailure
MalformedAdvertisement
DowngradeDetected
```

---

# 25. Two-Phase Confirmation

For security-sensitive negotiations:

```text
Offer
 ↓
Selection
 ↓
Confirmation hash
```

Both peers confirm the same negotiated capability set.

This prevents ambiguity.

---

# 26. Negotiation Transcript Hash

Canonical encode:

```text
local advertisement
remote advertisement
selected capabilities
session nonce
```

Then derive:

```text
NegotiationHash
```

Both peers verify equality.

---

# 27. Canonical Encoding

Capability ordering must be canonical before hashing.

Example:

```text
sort by namespace + code + version
```

Do not hash arbitrary HashMap iteration order.

---

# 28. Downgrade Attacks

An attacker may attempt to remove strong features from negotiation.

Example:

```text
both peers support strong algorithm
attacker forces weaker
```

Mitigations:

- authenticated transcript
- minimum local policy
- remembered peer security baseline where appropriate
- explicit downgrade detection

---

# 29. Security Capability Negotiation

Examples:

```text
signature schemes
AEAD schemes
KDF versions
session protocol versions
group security versions
```

Security selection must use:

```text
strongest mutually supported
+
locally allowed
```

not peer-preferred order alone.

---

# 30. Algorithm Agility

Represent algorithms by IDs.

```rust
pub struct AlgorithmId(u16);
```

Do not bake algorithm names into every public type if future migration matters.

---

# 31. Minimum Security Floor

Local policy:

```text
MinimumSecurityProfile::Modern
```

can reject older capability sets.

Backward compatibility must never override unacceptable security.

---

# 32. Remembered Security Baseline

For previously verified contacts, optionally remember:

```text
highest previously observed security generation
```

If a future session unexpectedly drops below it:

```text
DowngradeSuspected
```

Do not auto-reject all capability loss; device/platform changes can be legitimate.

Use policy.

---

# 33. Extension Capability Negotiation

Each protocol extension from Part 01 owns its capability schema.

Example:

```text
messaging/1
  text
  reply
  edit
  reactions
  read_receipts
```

Negotiation should be delegated to extension-specific logic.

---

# 34. Extension Negotiator Trait

```rust
pub trait ExtensionNegotiator {
    fn advertise(&self) -> ExtensionCapabilitySet;

    fn negotiate(
        &self,
        remote: &ExtensionCapabilitySet,
        policy: &CapabilityPolicy,
    ) -> Result<NegotiatedExtensionCapabilities, CapabilityError>;
}
```

---

# 35. File Capabilities

Example:

```text
files/1
├── fixed_chunking
├── resume
├── parallel_chunks
├── ciphertext_addressing
├── partial_read
├── multi_source
└── max_chunk_size
```

Part 05 consumes the negotiated set.

---

# 36. File Limit Negotiation

Sender:

```text
max chunk = 4 MiB
```

Receiver:

```text
max chunk = 1 MiB
```

Effective:

```text
1 MiB
```

Local policy may reduce further.

---

# 37. DTN Capabilities

Example:

```text
dtn/1
├── direct_only
├── relay_ack
├── spray_wait
├── gateway_handoff
├── blob_chunk
├── local_broadcast
└── max_bundle_size
```

Part 06 uses only negotiated features.

---

# 38. Transport Capabilities

Transport-level:

```text
reliable_stream
datagram
local_discovery
relay
multipath
store_and_forward
```

These describe current session/adapter abilities.

Routing uses them.

---

# 39. Platform Capabilities

Examples:

```text
Bluetooth LE
Bluetooth Classic
Wi-Fi Direct
Wi-Fi Aware
background nearby
hardware secure store
camera
microphone
PiP
```

Do not expose raw Android/iOS API details to upper layers.

---

# 40. Media Capabilities

For calls:

```text
Audio:
  Opus

Video:
  AV1 software
  AV1 hardware
  H.264 hardware
  H.265 hardware
```

Also parameters:

```text
encode
decode
max resolution
max fps
bit depth
hardware/software
```

---

# 41. Media Capability Type

```rust
pub struct VideoCodecCapability {
    pub codec: VideoCodec,
    pub direction: CodecDirection,
    pub implementation: CodecImplementation,
    pub max_width: u32,
    pub max_height: u32,
    pub max_fps: u16,
    pub profiles: CodecProfileSet,
}
```

---

# 42. Codec Selection

Selection policy may prioritize:

```text
compatibility
hardware efficiency
quality
battery
CPU
```

Example Android:

```text
AV1 HW if both support and efficient
else H.265 if policy allows
else H.264
```

Desktop software-only policy may differ.

---

# 43. Codec Negotiation Is Not Routing

Capability negotiation decides:

```text
what can be encoded/decoded
```

Routing decides:

```text
which network path carries media
```

Keep them separate.

---

# 44. Codec Runtime Failure

A capability can be negotiated but fail at runtime.

Example:

```text
broken vendor MediaCodec
```

Media subsystem reports:

```text
capability degraded
```

and renegotiates or falls back.

Capability advertisement is not an infallible guarantee.

---

# 45. Dynamic Capability Update

Support runtime update:

```text
camera permission revoked
Bluetooth disabled
Wi-Fi becomes available
hardware codec unavailable due thermal issue
```

Do not restart whole identity session unnecessarily.

---

# 46. Capability Epoch

```rust
pub struct CapabilityEpoch(u64);
```

Every dynamic capability update increments local epoch.

Peer can detect stale updates.

---

# 47. Capability Update Message

```rust
pub struct CapabilityUpdate {
    pub epoch: CapabilityEpoch,
    pub changed: Vec<CapabilityDelta>,
    pub transcript_binding: SessionBinding,
}
```

Keep updates bounded.

---

# 48. Full Snapshot vs Delta

Initial handshake:

```text
full capability snapshot
```

Runtime:

```text
delta
```

If peer detects missed delta:

```text
request full snapshot
```

---

# 49. Capability Cache

Cache negotiated peer capabilities for UX/performance hints.

Store:

```text
peer/device
capability set hash
observed_at
session generation
```

But cache is not authoritative for new sessions.

---

# 50. Cache Uses

Useful for:

- pre-disable unsupported UI actions
- estimate route options
- avoid unnecessary offers
- prepare call UI

Always revalidate on authenticated session.

---

# 51. Capability Staleness

Cached capability should have:

```text
fresh
stale
unknown
```

Do not present stale support as guaranteed.

---

# 52. Peer Upgrade

If peer upgrades:

```text
new capability epoch
```

future sessions negotiate more features automatically.

No central server migration required.

---

# 53. Peer Downgrade

If peer intentionally disables feature:

```text
capability disappears
```

Application should degrade gracefully.

Example:

```text
edit no longer supported
```

Disable edit UI for that peer/session.

---

# 54. Required Capability per Operation

Even inside active session, an operation may require capability.

Example:

```text
send edited message
requires messaging.edit
```

Before send:

```text
check negotiated capability
```

Do not assume extension presence implies every feature.

---

# 55. Operation Capability Requirement

```rust
pub struct OperationCapabilityRequirement {
    pub extension: ProtocolId,
    pub required: CapabilitySet,
}
```

Routing/application validates before work begins.

---

# 56. Capability-Based UI

UI consumes a simplified view model:

```text
CanSendFiles
CanVideoCall
CanUseNearby
CanResumeTransfer
```

It should not parse raw protocol capabilities.

---

# 57. UI Capability Adapter

```rust
pub struct UiCapabilities {
    pub can_message: bool,
    pub can_send_file: bool,
    pub can_audio_call: bool,
    pub can_video_call: bool,
    pub can_dtn: bool,
}
```

Derived from negotiated capability state.

---

# 58. Unknown UI Capability

If peer state unknown:

```text
action may remain available
```

and negotiation can happen on demand.

Do not over-disable based on stale cache.

---

# 59. Lazy Extension Negotiation

Some heavy extensions can negotiate only when needed.

Example:

```text
calls
```

Core session advertises:

```text
calls extension supported
```

Detailed codec negotiation occurs when call starts.

---

# 60. Hierarchical Negotiation

Recommended:

```text
Core session
 ↓
extension support
 ↓
feature subset
 ↓
operation-specific parameters
```

Do not send huge codec/file capability payload on every connection if unused.

---

# 61. Capability Advertisement Size

Bound:

```text
max capabilities
max parameters
max bytes
```

A peer must not force huge negotiation payloads.

---

# 62. Compact Encoding

Use numeric IDs and bitsets for common capabilities.

Example:

```text
MessagingCapabilityBits
FilesCapabilityBits
```

This reduces handshake size.

---

# 63. Bitset Evolution

Reserve bits carefully.

Unknown bits:

```text
ignored if optional
```

Do not reuse removed bits.

---

# 64. Sparse Custom Capabilities

Third-party/custom extensions may use:

```text
bounded sorted vector of IDs
```

rather than allocating giant bitsets.

---

# 65. Capability Registry

Maintain registry:

```rust
pub struct CapabilityRegistry {
    descriptors: HashMap<CapabilityId, CapabilityDefinition>,
}
```

The registry provides:

- canonical names
- validation
- version semantics
- parameter codec
- security classification

No global singleton required.

---

# 66. Capability Definition

```rust
pub struct CapabilityDefinition {
    pub id: CapabilityId,
    pub name: &'static str,
    pub max_version: CapabilityVersion,
    pub parameter_schema: ParameterSchema,
    pub security_class: SecurityClass,
}
```

---

# 67. Security Class

```rust
pub enum SecurityClass {
    Informational,
    Functional,
    SecuritySensitive,
    Critical,
}
```

Unknown critical capability behavior should be conservative.

---

# 68. Mandatory Unknown Critical Capability

If remote marks:

```text
unknown critical capability required
```

reject that extension/session operation.

Never silently ignore.

---

# 69. Capability Dependency

Some capabilities depend on others.

Example:

```text
files.parallel_chunks
requires files.chunking
```

Model:

```rust
pub struct CapabilityDependency {
    pub capability: CapabilityId,
    pub requires: CapabilitySet,
}
```

Validation must reject inconsistent advertisement.

---

# 70. Capability Conflict

Some are mutually exclusive.

Example:

```text
compression mode A
compression mode B
```

Negotiator selects one or fails according to rules.

---

# 71. Preference Ordering

For selectable modes:

```text
local preference
remote support
policy
```

Example codec preference.

Do not let peer alone choose mode.

---

# 72. Negotiation Determinism

Given same:

```text
local set
remote set
policy
```

both peers should derive same result.

This simplifies testing and transcript confirmation.

---

# 73. Selection Function

Prefer pure:

```rust
fn negotiate(
    local: &CapabilitySet,
    remote: &CapabilitySet,
    policy: &CapabilityPolicy,
) -> Result<NegotiatedCapabilities, CapabilityError>
```

No network side effects.

---

# 74. Canonical Preference Rule

For choices such as algorithm/codec:

```text
filter unsafe
find mutual
rank by local policy
tie-break canonically
```

Both sides should reach same choice if symmetric agreement required.

---

# 75. Asymmetric Capabilities

Some capabilities differ by direction.

Example:

```text
can_encode AV1
can_decode AV1
```

Do not represent as one Boolean.

---

# 76. Direction

```rust
pub enum CapabilityDirection {
    Send,
    Receive,
    Both,
}
```

Useful for:

- codecs
- file upload/download limits
- relay capability
- call media

---

# 77. Device-Specific Capabilities

Part 02 allows multiple devices.

Account-level capability must be derived from device set.

Example:

```text
Bob account:
Phone can video
Laptop cannot
```

Call routing targets the capable device.

Do not flatten all devices into one Boolean permanently.

---

# 78. Account Capability View

A convenience derived view may say:

```text
Account has at least one video-capable device
```

but routing still resolves actual device.

---

# 79. Route Integration

Part 03 can filter candidates based on required capability.

Example:

```text
file transfer requires files/1
```

Candidate session without it:

```text
invalid for operation
```

---

# 80. DTN Integration

DTN forwarding peer may support:

```text
relay small bundles
blob chunks
gateway handoff
```

Negotiated capability determines which bundle classes can be exchanged.

---

# 81. File Integration

Part 05 uses:

```text
chunk size
parallelism
resume
partial transfer
```

from negotiated file capability contract.

---

# 82. Offline Event Log Integration

Part 04 may persist:

```text
meaningful peer capability generation
```

as cache/projection if useful.

Do not permanently journal every ephemeral capability fluctuation.

---

# 83. Capability Snapshot Persistence

Store:

```text
last_verified_peer_capabilities
```

as cache.

Mark with:

```text
observed session
timestamp
hash
```

---

# 84. No Capability Trust from Server Alone

A directory server may publish:

```text
peer capability hint
```

Use only as hint.

Final capability contract comes from authenticated peer negotiation.

---

# 85. Relay Capability Advertisement

A relay may advertise:

```text
max bundle
gateway
storage class
```

Routing/DTN uses as hints.

Do not trust self-reported capacity for security-sensitive decisions.

---

# 86. Capability Attestation

Optional platform attestation can prove some hardware/platform properties.

It is supplemental.

Do not make capability negotiation depend on remote attestation for normal use.

---

# 87. Resource Capabilities

Examples:

```text
max concurrent transfers
max inbound file size
max relay bytes
max active streams
```

These are negotiated limits, not promises of permanent capacity.

---

# 88. Dynamic Resource Limits

At runtime:

```text
storage pressure
battery
memory
```

can reduce effective limits.

Capability update may advertise changed capacity class.

---

# 89. Coarse Resource Classes

Prefer:

```text
Low
Normal
High
```

for sensitive/battery-related hints rather than exact values when privacy matters.

---

# 90. Capability Privacy

Capability sets can fingerprint devices.

Minimize exposure.

Do not advertise exact:

```text
model
GPU
codec vendor
RAM
battery %
```

unless necessary.

---

# 91. Privacy Profiles

Possible:

```text
Minimal
Standard
Diagnostic
```

Normal peers receive only required feature capabilities.

Detailed hardware capability is available only when needed.

---

# 92. Progressive Disclosure

Initial handshake:

```text
supports media
```

Call setup:

```text
detailed codec capabilities
```

File transfer setup:

```text
detailed transfer limits
```

This reduces metadata leakage and handshake size.

---

# 93. Capability Fingerprinting Risk

A rare combination such as:

```text
exact codec profile
exact max resolution
exact Bluetooth feature
```

may identify device class.

Document this risk.

---

# 94. Capability Hash

Store canonical capability set hash:

```rust
pub struct CapabilitySetHash([u8; 32]);
```

Useful for:

- cache validation
- delta detection
- transcript binding

---

# 95. Delta Update

```text
old hash
new hash
changed capabilities
```

Peer verifies expected previous epoch/hash.

This prevents applying delta to wrong base.

---

# 96. Missed Update Recovery

If peer sees unexpected epoch:

```text
request full capability snapshot
```

Do not attempt to infer missing deltas.

---

# 97. Session Reconnect

On reconnect:

```text
new authenticated negotiation
```

Cached hash may let peers shortcut if both explicitly support safe resume.

Start without shortcut initially.

---

# 98. Session Resumption Optimization

Future:

```text
capability hash unchanged
+
session resumption authenticated
```

can avoid sending full sets.

But correctness first.

---

# 99. Interoperability

Stable capability IDs and semantics allow non-Rust implementations later.

Do not rely on:

```text
Rust enum memory layout
usize
HashMap order
```

Wire representation must be explicit.

---

# 100. Postcard Use

Postcard is suitable for compact capability structures.

Rules:

- fixed-width protocol-relevant integers
- bounded vectors
- versioned schemas
- canonical ordering before transcript hash
- no unbounded recursive structures

---

# 101. Wire Structure

Potential:

```rust
pub struct CapabilityAdvertisementV1 {
    pub core_version: u16,
    pub extension_sets: Vec<ExtensionCapabilitiesV1>,
    pub security: SecurityCapabilitiesV1,
    pub summary: CapabilitySummaryV1,
}
```

Detailed optional negotiation follows extension open.

---

# 102. Capability Summary

Compact initial summary can use:

```text
extension IDs
major versions
feature-group bits
```

Then request detail lazily.

---

# 103. Negotiation Error Model

```rust
pub enum CapabilityError {
    Malformed,
    UnsupportedCoreVersion,
    RequiredCapabilityMissing,
    VersionMismatch,
    SecurityPolicyFailure,
    InconsistentDependency,
    DowngradeSuspected,
    LimitExceeded,
    TranscriptMismatch,
}
```

---

# 104. User-Visible Mapping

Normal UI should show:

```text
File transfer unsupported
Video calling unavailable
Nearby forwarding unavailable
```

not:

```text
Capability 0x0421 missing
```

Diagnostics can expose technical detail.

---

# 105. Required Capability Failure

Example:

```text
files-only app
requires files/1
peer lacks files/1
```

Connection core may remain alive, but file operation fails cleanly.

Do not necessarily tear down entire session.

---

# 106. Security Failure

If required security capability fails:

```text
terminate trusted session
```

This is different from optional feature mismatch.

---

# 107. Extension Isolation

One failed extension negotiation should not necessarily break unrelated extensions.

Example:

```text
calls unsupported
messaging works
files work
```

---

# 108. Capability Lifecycle

```text
Unknown
 ↓
Advertised
 ↓
Negotiating
 ↓
Negotiated
 ↓
Active
 ↓
Updated
 ↓
Invalidated
```

---

# 109. Invalidation

Invalidate capability contract when:

```text
identity changes
session reauth
device revoked
protocol reset
critical update mismatch
```

---

# 110. Runtime Capability Failure

If a feature fails repeatedly:

```text
mark degraded locally
```

Optionally send update.

Example:

```text
hardware AV1 encoder broken
→ remove encode capability for current session
```

---

# 111. Capability Health

```rust
pub enum CapabilityHealth {
    Available,
    Degraded,
    TemporarilyUnavailable,
    DisabledByPolicy,
}
```

Do not conflate support with current health.

---

# 112. Capability Query API

```rust
let caps = client.capabilities(peer).await?;

if caps.supports(FILE_RESUME) {
    ...
}
```

Provide higher-level helpers too.

---

# 113. Operation Guard API

```rust
client
    .require_capabilities(peer, OperationKind::VideoCall)
    .await?;
```

This can return structured fallback info.

---

# 114. Fallback Suggestions

Example:

```text
Video unsupported
Audio supported
```

Capability engine can produce:

```rust
FallbackPlan::AudioOnly
```

or data used by call layer.

---

# 115. File Fallback

Example:

```text
parallel unsupported
resume supported
```

File engine uses:

```text
single-stream resumable transfer
```

---

# 116. DTN Fallback

Example:

```text
blob-chunk relay unsupported
small inline bundles supported
```

Then:

```text
send text/SOS
defer large file
```

---

# 117. Routing Fallback

Example:

```text
Wi-Fi Direct unsupported
Bluetooth supported
```

Routing adapts automatically.

---

# 118. Capability Negotiation for Custom Apps

ERP extension:

```text
com.example.erp/approval/1
```

can register:

```text
approval-v2
attachments
offline-signature
```

without modifying core negotiation logic.

---

# 119. Third-Party Capability Registry

Namespaced capability IDs prevent collision.

Core may reserve:

```text
0x0000_0000..0x0FFF_FFFF
```

and application namespaces map separately.

Exact scheme should be documented.

---

# 120. Capability Documentation

Every stable capability should document:

```text
canonical name
ID
version
parameters
dependencies
security class
default policy
wire behavior
fallback behavior
```

---

# 121. Generated Docs

Registry metadata can generate:

```text
capability matrix
protocol compatibility table
```

for developers.

---

# 122. Compatibility Matrix

Example:

```text
Peer A       Peer B       Result
messaging/1  messaging/1  works
files/1      none         messaging only
AV1 HW       H264 HW      H264
DTN spray    DTN direct   direct-only DTN
```

---

# 123. Golden Tests

Store stable negotiation cases.

Example:

```text
local set
remote set
policy
expected negotiated set
```

These prevent accidental behavior changes.

---

# 124. Property Tests

Invariants:

```text
negotiated capability is supported by both peers
hard policy is never violated
required missing capability never becomes active
result is deterministic
effective max never exceeds either peer max
```

---

# 125. Fuzzing

Fuzz:

```text
capability advertisement
parameter decoding
dependency graphs
delta updates
canonical ordering
transcript hash inputs
```

Bound allocations.

---

# 126. Downgrade Tests

Simulate attacker stripping:

```text
strong security capability
```

Negotiation must detect transcript mismatch or policy failure.

---

# 127. Malformed Capability Tests

Examples:

```text
duplicate IDs with conflicting versions
invalid range
cyclic dependency
too many entries
oversized parameter
unknown required critical
```

Reject deterministically.

---

# 128. Large Capability Set Test

Third-party ecosystem may grow.

Ensure:

```text
bounded
fast
canonical
```

negotiation with hundreds of capabilities.

Do not design for millions.

---

# 129. Performance

Capability negotiation occurs per session/extension open, not per packet.

Optimize for:

```text
small wire size
fast validation
deterministic selection
cacheable result
```

Do not micro-optimize at expense of clarity/security.

---

# 130. Memory Bounds

Limit:

```text
max extension count
max capabilities per extension
max parameter bytes
max nested depth
```

---

# 131. Logging

Log:

```text
peer device id hash/redacted
extension
selected versions
feature count
failure reason
```

Do not log sensitive hardware fingerprint detail by default.

---

# 132. Diagnostics

Advanced diagnostics can show:

```text
Core protocol: 1
Messaging: v1
  reply=yes
  edit=no

Files: v1
  resume=yes
  parallel=4
  max chunk=1 MiB

Media:
  AV1 decode software
  H264 encode hardware

DTN:
  spray-wait=yes
  max bundle=64 KiB
```

---

# 133. UI Simplicity

Normal UI should summarize:

```text
Can send files
Can video call
Nearby relay supported
```

Capability internals belong in developer/security screens.

---

# 134. Storage Schema

Optional cache tables:

```text
peer_capability_snapshots
peer_capability_hashes
capability_observed_at
```

Do not persist session secrets here.

---

# 135. Cache Invalidation

Invalidate on:

```text
device certificate change
account generation change
protocol upgrade
session reauthentication
explicit peer update
```

---

# 136. Multi-Device Capability View

For an account:

```text
Phone:
video yes
DTN yes

Laptop:
video no
files yes
```

Application query should be able to ask:

```text
which devices satisfy Operation X?
```

---

# 137. Device Selection Query

```rust
let devices = capabilities
    .devices_supporting(account, OperationKind::VideoCall)
    .await?;
```

Routing then selects reachable device.

---

# 138. Capability-Based Fan-Out

Messaging may fan out to all devices supporting:

```text
messaging/1
```

File transfer may target only devices supporting:

```text
files/1
```

---

# 139. Headless Device Capabilities

Headless node can advertise:

```text
DTN relay
files
gateway
```

without:

```text
calls
UI
camera
```

This is expected, not exceptional.

---

# 140. Emergency Node Capabilities

Emergency relay may advertise:

```text
high relay class
authority alert forwarding
Wi-Fi bridge
Iroh gateway
```

through bounded coarse capabilities.

---

# 141. Capability Negotiation and Resource Limits

Part 08 will enforce runtime limits.

Negotiated values are upper contracts.

Actual scheduler may use less.

Example:

```text
max parallel = 8 negotiated
current battery policy = 2
```

---

# 142. Capability Negotiation and Crash Recovery

After restart:

```text
capability cache may help UX
```

but new network session must reauthenticate and renegotiate.

Do not restore stale session capability contract as authoritative.

---

# 143. Capability Negotiation and Relay Infrastructure

Part 11 relay servers may advertise:

```text
relay protocol version
max session class
regional support
```

but application-level capabilities remain peer-to-peer authenticated.

---

# 144. Capability Negotiation and Multipath

Part 12 can negotiate:

```text
multipath supported
max concurrent paths
chunk striping support
```

without changing core handshake semantics.

---

# 145. Capability Negotiation and Battery Scheduling

Part 13 may dynamically reduce:

```text
relay capacity
background transfer capacity
```

through capability health/updates.

---

# 146. Capability Negotiation and Proximity

Part 14 can advertise coarse proximity transports:

```text
BLE
Wi-Fi Direct
Wi-Fi Aware
```

Detailed hardware fingerprint should remain hidden until needed.

---

# 147. Capability Negotiation and QR/NFC

Part 15 bootstrap payload can include:

```text
core protocol versions
minimal capability summary
```

to avoid attempting impossible link flows.

Full negotiation still occurs after authenticated session.

---

# 148. Capability Negotiation and Daemon

Part 16 daemon exposes capabilities through stable IPC API.

GUI/CLI should not directly inspect hardware adapters.

---

# 149. Capability Negotiation and Emergency Priority

Part 17 can negotiate:

```text
critical relay support
authority alert version
broadcast class
```

with authorization rules.

---

# 150. Capability Negotiation and Diagnostics

Part 18 can visualize:

```text
which path/feature unavailable because capability missing
```

This is crucial for support.

---

# 151. Capability Negotiation and FFI

Part 19 should expose a simplified stable capability query API.

Do not expose internal Rust registry pointers across FFI.

---

# 152. Capability Negotiation and Embedded Linux

Part 20 embedded nodes may advertise minimal subsets.

The protocol should handle small constrained peers cleanly.

---

# 153. Capability Negotiation and WASM

Part 21 WASM components can reuse:

```text
capability types
selection logic
codec
```

without transport-specific runtime.

---

# 154. Capability Negotiation and Third-Party Extensions

Part 22 directly depends on namespaced capability registration.

Custom extensions can define their own capability IDs and parameter codecs.

---

# 155. Capability Negotiation and Interoperability Suite

Part 23 should test:

```text
capability advertisement
required/optional behavior
version mismatch
unknown capability
downgrade detection
```

across implementations.

---

# 156. Capability Negotiation and Plugin Ecosystem

Part 24 plugin/module system can register capability schemas through controlled API.

Plugins must not overwrite existing IDs.

---

# 157. Public API Surface

Suggested:

```rust
CapabilityClient
CapabilityRegistry
NegotiatedCapabilities
CapabilityPolicy
CapabilityQuery
```

Keep low-level wire negotiation internal.

---

# 158. Suggested Crate Structure

```text
crates/comm-capability/
├── src/
│   ├── lib.rs
│   ├── id.rs
│   ├── namespace.rs
│   ├── descriptor.rs
│   ├── set.rs
│   ├── parameter.rs
│   ├── registry.rs
│   ├── advertisement.rs
│   ├── negotiate.rs
│   ├── policy.rs
│   ├── security.rs
│   ├── cache.rs
│   ├── update.rs
│   ├── diagnostics.rs
│   └── error.rs
└── Cargo.toml
```

Potential protocol module:

```text
comm-capability-protocol
```

if wire concerns grow.

---

# 159. Error Model

```rust
pub enum CapabilityNegotiationError {
    UnsupportedCore,
    MissingRequired,
    VersionMismatch,
    SecurityPolicy,
    Malformed,
    DependencyViolation,
    Conflict,
    LimitExceeded,
    DowngradeSuspected,
    TranscriptMismatch,
    Cancelled,
}
```

Use typed errors.

---

# 160. Initial Production Scope

Implement first:

```text
core protocol version negotiation
extension support advertisement
required vs optional capabilities
typed file capability parameters
typed DTN capability parameters
basic media codec capability
security algorithm negotiation
canonical capability hashing
two-phase confirmation
capability cache
runtime capability updates
```

Defer initially:

```text
complex attestation
dynamic remote plugin capability code
automatic capability learning
large metadata-rich hardware profiles
```

---

# 161. Implementation Phases

## Phase 1 — Core Types

```text
CapabilityId
CapabilityVersion
CapabilitySet
CapabilityRegistry
```

## Phase 2 — Handshake

```text
advertise
validate
intersect
confirm
```

## Phase 3 — Extension Integration

```text
messaging
files
DTN
```

## Phase 4 — Security

```text
algorithm selection
minimum policy
downgrade detection
transcript hash
```

## Phase 5 — Media/Platform

```text
codec
Bluetooth/Wi-Fi
resource summaries
```

## Phase 6 — Dynamic Updates

```text
epoch
delta
full resync
```

## Phase 7 — Hardening

```text
fuzzing
property tests
golden cases
interoperability tests
```

---

# 162. Definition of Done

Part 07 is complete when:

- peers do not rely on app version to infer feature support
- capability advertisements are authenticated
- required and optional capabilities behave differently
- unknown optional capabilities do not break compatibility
- unknown required capabilities fail safely
- version ranges negotiate deterministically
- security capabilities respect a minimum local security floor
- downgrade attempts are detectable
- file transfer negotiates chunk/resume/parallel limits
- DTN negotiates relay/bundle capabilities
- media negotiates codec direction and implementation
- device-specific capability differences are preserved
- routing can filter paths by required capabilities
- capability cache is treated only as a hint
- dynamic capability updates are versioned
- capability payload sizes are bounded
- canonical transcript hashing exists
- Dioxus consumes simplified derived capability state
- custom third-party namespaces can register capabilities safely
- property/fuzz/golden/interoperability tests exist

---

# 163. Relationship to Earlier Parts

Part 07 builds directly on:

```text
01 — Protocol Extension System
02 — Multi-Device Identity
03 — Transport & Routing Policy Engine
04 — Offline Event Log
05 — Robust File / Blob Subsystem
06 — DTN / Store-Carry-Forward
```

It prepares the platform for:

```text
08 — Resource Limits & Backpressure
09 — Crash Recovery
10 — Protocol Fuzzing & Test Suite
11 — Relay / Self-Hosted Infrastructure
12 — Multipath Networking
13 — Battery-Aware Scheduling
14 — Proximity Abstraction
15 — QR / NFC Bootstrap Pairing
16 — Daemon & Headless Runtime
17 — Emergency Priority Architecture
18 — Network Diagnostics & Path Visualization
19 — C ABI / FFI
20 — Embedded Linux Node
21 — WASM-Compatible Components
22 — Third-Party Protocol Extensions
23 — External Interoperability Suite
24 — Plugin / Module Ecosystem
```

---

# 164. Final Principle

The capability system should let two completely different peers safely discover the strongest mutually compatible behavior.

Example:

```text
Alice Android
  messaging v1
  files resume+parallel
  AV1 HW
  H264 HW
  DTN spray-wait
  Wi-Fi Aware

Bob Linux
  messaging v1
  files resume
  AV1 software
  no H264 hardware
  DTN direct-only
  LAN
```

The negotiated result may become:

```text
Messaging:
  v1

Files:
  resume
  single-path/single-parallel subset

Video:
  AV1

DTN:
  direct-only common subset
```

Neither peer needs to know the other's implementation details.

They only need a verified, versioned contract describing what they can safely do together.

That is what makes the communication platform interoperable, evolvable, modular, and production-ready across different devices, products, and future protocol generations.
