# Core System Architecture Part 34 — Mixnet, Loopix, Sphinx, Nym & High-Anonymity Transport Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 34  
**Status:** Architecture specification  
**Primary language:** Rust  
**Scope:** high/full anonymity transport integration alongside SIAR's existing direct, relay, LAN, Wi-Fi, Bluetooth, mesh, and DTN transport systems  
**Design intent:** add strong network-metadata privacy without replacing or weakening SIAR's existing transport, E2EE, local-first, emergency, realtime, or offline capabilities

---

# 1. Purpose

SIAR already has multiple transport classes intended for different operating environments:

```text
Direct Internet
QUIC / Iroh
Relay
LAN
Wi-Fi Direct / Wi-Fi Aware
Bluetooth
Local mesh
DTN / store-carry-forward
```

These transports optimize primarily for:

```text
reachability
latency
bandwidth
reliability
offline operation
local communication
emergency communication
```

They do **not**, by themselves, provide strong resistance against network metadata analysis.

A network observer may still infer information such as:

```text
which device is communicating
when communication occurs
how often communication occurs
traffic volume
likely sender/receiver relationships
route endpoints
session timing
```

even when message contents are protected by end-to-end encryption.

Part 34 introduces an independent **High-Anonymity Transport Plane** based on mix-network principles and compatible providers such as Nym.

The governing principle is:

> **SIAR must treat anonymity as an explicit routing/security property, not as a side effect of encryption or relaying.**

---

# 2. Design Goals

The subsystem MUST support:

```text
strong metadata protection
sender/receiver relationship hiding
mixnet routing
Sphinx-style packet encapsulation
cover traffic
randomized delays
store-and-forward reception
anonymous replies
strict downgrade prevention
battery/bandwidth-aware policy
integration with SIAR E2EE
integration with SIAR multipath routing
integration with SIAR DTN
provider abstraction
Nym integration
future SIAR-native mixnet
privacy-safe diagnostics
testability
```

---

# 3. Non-Goals

Part 34 does not attempt to:

```text
replace SIAR E2EE
replace Iroh/QUIC
replace LAN/Bluetooth/mesh
route realtime video through a high-latency mixnet by default
claim perfect anonymity
invent new cryptography
expose mixnet protocol complexity directly to ordinary users
```

---

# 4. Core Architectural Principle

The high-anonymity system is an additional transport plane:

```text
SIAR Application
      │
      ▼
Application E2EE
      │
      ▼
Routing Policy Engine
      │
      ├── Performance Transport Plane
      │     ├── Iroh / QUIC
      │     ├── Relay
      │     ├── LAN
      │     ├── Wi-Fi Direct
      │     └── Bluetooth
      │
      ├── Delay-Tolerant Plane
      │     └── DTN / Store-Carry-Forward
      │
      └── High-Anonymity Plane
            ├── Nym Provider
            └── SIAR Native Mixnet [future]
```

---

# 5. Encryption Is Not Anonymity

The architecture must preserve a strict distinction:

```text
Application E2EE
    protects message content

Transport encryption
    protects transport sessions

Mixnet anonymity
    protects communication metadata and linkage
```

Therefore:

```text
E2EE ≠ anonymity
anonymity ≠ E2EE
```

SIAR should normally use:

```text
E2EE + anonymous transport
```

for anonymous messaging.

---

# 6. Mixnet Concept

A mixnet routes messages through multiple intermediate mix nodes.

Each mix node:

```text
receives packets
cryptographically transforms packets
delays packets
reorders packets
forwards packets
```

The purpose is to make it difficult for an observer to correlate:

```text
input packet
↔
output packet
```

---

# 7. Loopix Role

Loopix provides a useful architectural model for low-latency anonymous messaging.

Important ideas include:

```text
stratified mix topology
Poisson-distributed delays
cover traffic
loop traffic
provider/mailbox nodes
offline recipient support
fixed-size packetization
```

SIAR should adopt the **design concepts**, not blindly copy every protocol detail.

---

# 8. Sphinx Role

Sphinx is not itself a complete anonymity network.

Within this architecture it represents the packet construction/routing layer that can provide:

```text
layered routing encryption
per-hop transformation
hidden route information
fixed-size or normalized packet structure
integrity protection
unlinkability between hops
```

Mixing, delays, topology, cover traffic, and route policies are still required.

---

# 9. Nym Role

Nym should be SIAR's first external high-anonymity provider.

Reason:

```text
deployed mixnet
Loopix-derived architecture
Sphinx packet design
existing node infrastructure
existing client/runtime ecosystem
practical route to production experimentation
```

SIAR must integrate Nym through an abstraction rather than embedding Nym assumptions throughout the codebase.

---

# 10. High-Level Architecture

```text
┌─────────────────────────────────────────────────────────┐
│                    SIAR APPLICATION                     │
├─────────────────────────────────────────────────────────┤
│ Identity │ Messaging │ Files │ Groups │ Sync │ Plugins │
├─────────────────────────────────────────────────────────┤
│                  Application E2EE                       │
├─────────────────────────────────────────────────────────┤
│              Routing / Privacy Policy Engine            │
├───────────────┬───────────────────┬─────────────────────┤
│ Realtime      │ Delay-Tolerant    │ High-Anonymity      │
│ Transport     │ Transport         │ Transport           │
├───────────────┼───────────────────┼─────────────────────┤
│ Iroh / QUIC   │ DTN               │ AnonymousTransport  │
│ Relay         │ Store/Forward     │   ├─ NymAdapter     │
│ LAN           │ Opportunistic     │   └─ NativeMixnet   │
│ Wi-Fi Direct  │ Mesh              │      [future]       │
│ Bluetooth     │                   │                     │
└───────────────┴───────────────────┴─────────────────────┘
```

---

# 11. Privacy Routing Modes

Ordinary users should select **intent**, not protocols.

```rust
pub enum PrivacyRoutingMode {
    Standard,
    Private,
    Anonymous,
    MaximumAnonymity,
}
```

---

# 12. Standard Mode

Intent:

```text
lowest practical latency
normal SIAR security
normal transport fallback
```

Allowed paths:

```text
direct
relay
LAN
Wi-Fi Direct
Bluetooth
DTN
```

---

# 13. Private Mode

Intent:

```text
reduce direct endpoint exposure
```

Possible rules:

```text
prefer relay
avoid direct peer-IP exposure
restrict opportunistic paths
retain reasonable latency
```

This is **not** full anonymity.

---

# 14. Anonymous Mode

Intent:

```text
strong network metadata protection
```

Preferred route:

```text
approved mixnet provider
```

Fallback policy must be explicit.

---

# 15. Maximum Anonymity Mode

Intent:

```text
strict anonymity over availability/latency
```

Required policies may include:

```text
mixnet-only routing
no direct fallback
no ordinary relay fallback
mandatory packet normalization
cover traffic
metadata-minimizing application behavior
privacy-safe logs
restricted plugins
reduced presence
reduced receipts
no external preview fetch
```

---

# 16. Downgrade Prevention

This is a critical security boundary.

```rust
pub enum AnonymityFallbackPolicy {
    NeverFallback,
    AnonymousPathsOnly,
    AskBeforeDowngrade,
}
```

Maximum-anonymity default:

```text
NeverFallback
```

---

# 17. Forbidden Silent Downgrade

Never:

```text
Nym unavailable
→ silently use direct QUIC
```

because doing so can reveal metadata the user explicitly asked to protect.

---

# 18. Route Failure UX Semantics

Correct:

```text
Anonymous route unavailable.
Message remains queued.
```

Incorrect:

```text
Connection restored through direct route.
```

without explicit user approval.

---

# 19. Route Requirements

```rust
pub struct RouteRequirements {
    pub anonymity: AnonymityRequirement,
    pub latency: LatencyRequirement,
    pub bandwidth: BandwidthRequirement,
    pub reliability: ReliabilityRequirement,
    pub power: PowerRequirement,
    pub offline: OfflineRequirement,
}
```

---

# 20. Anonymity Requirement

```rust
pub enum AnonymityRequirement {
    None,
    HidePeerAddress,
    MixnetPreferred,
    MixnetRequired,
    Maximum,
}
```

---

# 21. Transport Capability Model

```rust
pub struct TransportCapabilities {
    pub supports_realtime: bool,
    pub supports_offline_delivery: bool,
    pub supports_anonymous_reply: bool,
    pub hides_sender_network_identity: bool,
    pub hides_receiver_network_identity: bool,
    pub provides_cover_traffic: bool,
    pub fixed_size_packets: bool,
}
```

---

# 22. Anonymous Transport Interface

```rust
#[async_trait]
pub trait AnonymousTransport: Send + Sync {
    async fn start(
        &self,
        config: AnonymousTransportConfig,
    ) -> Result<(), AnonymousTransportError>;

    async fn stop(
        &self,
    ) -> Result<(), AnonymousTransportError>;

    async fn send(
        &self,
        request: AnonymousSendRequest,
    ) -> Result<AnonymousDeliveryId, AnonymousTransportError>;

    async fn receive(
        &self,
    ) -> Result<AnonymousEnvelope, AnonymousTransportError>;

    async fn status(
        &self,
    ) -> AnonymousTransportStatus;

    fn capabilities(
        &self,
    ) -> AnonymousCapabilities;
}
```

---

# 23. Provider Implementations

Initial:

```text
NymAnonymousTransport
TestAnonymousTransport
```

Future:

```text
SiarNativeMixnetTransport
```

---

# 24. Provider Registry

```rust
pub trait AnonymousTransportRegistry {
    fn available(
        &self,
    ) -> Vec<AnonymousProviderDescriptor>;

    fn resolve(
        &self,
        provider: AnonymousProviderId,
    ) -> Option<Arc<dyn AnonymousTransport>>;
}
```

---

# 25. Provider Independence

Core messaging code must never depend directly on:

```text
Nym SDK types
Nym addresses
Nym packet types
Nym node-list internals
```

These belong inside:

```text
nym-adapter
```

---

# 26. Workspace Proposal

```text
crates/
├── siar-anonymity-core/
├── siar-anonymity-policy/
├── siar-anonymity-routing/
├── siar-anonymity-mailbox/
├── siar-anonymity-cover/
├── siar-anonymity-packet/
├── siar-anonymity-observability/
├── siar-anonymity-testkit/
├── siar-anonymity-nym/
└── siar-anonymity-native/        # future
```

---

# 27. Layer Boundaries

```text
siar-anonymity-core
    provider-neutral types

siar-anonymity-policy
    anonymity/routing rules

siar-anonymity-routing
    route orchestration

siar-anonymity-mailbox
    anonymous asynchronous receiving

siar-anonymity-cover
    cover-traffic policy

siar-anonymity-packet
    packet framing/normalization abstractions

siar-anonymity-observability
    privacy-safe metrics

siar-anonymity-nym
    Nym-specific implementation

siar-anonymity-testkit
    deterministic tests
```

---

# 28. Envelope Layering

Recommended:

```text
SIAR Message
     │
     ▼
Application E2EE Envelope
     │
     ▼
Anonymous Transport Envelope
     │
     ▼
Mixnet Packetization
     │
     ▼
Sphinx/Provider Packet
```

---

# 29. Application E2EE Remains Authoritative

The mixnet provider must never become responsible for SIAR's end-to-end identity/session security.

---

# 30. Anonymous Envelope

```rust
pub struct AnonymousEnvelope {
    pub version: AnonymousEnvelopeVersion,
    pub message_class: AnonymousMessageClass,
    pub payload: Bytes,
    pub expiry: Timestamp,
    pub reply: Option<AnonymousReplyDescriptor>,
}
```

---

# 31. Message Classes

```rust
pub enum AnonymousMessageClass {
    Control,
    Text,
    AttachmentChunk,
    Receipt,
    Bootstrap,
    MailboxControl,
}
```

---

# 32. Packet Size Normalization

Anonymous messages should be converted into normalized cells/packets.

Purpose:

```text
reduce traffic fingerprinting
hide payload-length variation
make cover traffic less distinguishable
```

---

# 33. Fragmentation

Large application payload:

```text
payload
→ anonymous chunks
→ fixed/normalized packet cells
```

---

# 34. Reassembly

Recipient:

```text
cells
→ authenticated reassembly
→ E2EE envelope
```

---

# 35. Fragment Metadata

Must avoid globally linkable identifiers where possible.

---

# 36. Reassembly ID

Use cryptographically random scoped identifier.

---

# 37. Fragment Limits

Bound:

```text
max fragments
max total message bytes
max lifetime
max incomplete assemblies
```

---

# 38. Cover Traffic

High-anonymity modes may require cover traffic.

Cover traffic makes traffic patterns less directly correlated with real communication.

---

# 39. Cover Traffic Policy

```rust
pub enum CoverTrafficPolicy {
    Disabled,
    Adaptive,
    Standard,
    Maximum,
}
```

---

# 40. Cover Traffic Constraints

Cover traffic consumes:

```text
bandwidth
battery
relay/mix capacity
```

Therefore policy must be explicit and resource-aware.

---

# 41. Adaptive Cover Traffic

Could consider:

```text
device power state
metered network
battery level
foreground/background
anonymity mode
```

without leaking per-message intent.

---

# 42. Maximum Anonymity

May override battery/network optimization if user explicitly chooses it.

The user should be informed.

---

# 43. Delay Distribution

A Loopix-inspired provider may use randomized delay.

Conceptually:

```text
packet
→ delay sampled from configured distribution
→ next mix hop
```

---

# 44. Delay Must Be Provider-Owned

Core SIAR should express:

```text
privacy/latency requirement
```

not manually implement provider-specific Poisson parameters.

---

# 45. Delay Policy

```rust
pub enum AnonymousLatencyClass {
    InteractiveMessaging,
    Balanced,
    MaximumPrivacy,
}
```

---

# 46. No False Realtime Promise

High-anonymity message delivery can take seconds or longer.

UI must communicate this honestly.

---

# 47. Mailbox / Store-and-Forward

Anonymous communication must support offline recipients.

Architecture:

```text
Sender
  │
  ▼
Mixnet
  │
  ▼
Anonymous Mailbox / Provider
  │
  ▼
Recipient later retrieves
```

---

# 48. Mailbox Goals

```text
recipient can be offline
provider cannot read payload
mailbox identifier minimizes linkability
bounded retention
authenticated retrieval
```

---

# 49. Mailbox Abstraction

```rust
pub trait AnonymousMailbox {
    async fn deposit(
        &self,
        envelope: AnonymousMailboxEnvelope,
    ) -> Result<MailboxDepositReceipt, MailboxError>;

    async fn fetch(
        &self,
        cursor: Option<MailboxCursor>,
    ) -> Result<MailboxBatch, MailboxError>;

    async fn acknowledge(
        &self,
        ids: &[MailboxItemId],
    ) -> Result<(), MailboxError>;
}
```

---

# 50. Mailbox Identity

Do not reuse publicly visible:

```text
AccountId
DeviceId
```

as mailbox address.

Use anonymity-specific addressing.

---

# 51. Mailbox Rotation

Support rotating mailbox/routing identifiers.

---

# 52. Multi-Device Mailboxes

Each SIAR device may have separate anonymous receiving identity.

Benefits:

```text
reduced correlation
independent revocation
device-local capability
```

---

# 53. Account-Level Anonymous Fanout

Application layer can fan out encrypted payloads to:

```text
device mailbox A
device mailbox B
device mailbox C
```

---

# 54. Anonymous Replies

A sender may need a reply path without exposing a stable receiver address.

Use provider-compatible anonymous reply mechanisms such as:

```text
reply blocks / SURB-like tokens
```

---

# 55. Reply Descriptor

```rust
pub struct AnonymousReplyDescriptor {
    pub kind: AnonymousReplyKind,
    pub token: SecretBytes,
    pub expires_at: Timestamp,
    pub max_uses: u32,
}
```

---

# 56. Reply Token Security

Reply tokens are sensitive.

Do not:

```text
log
sync unnecessarily
expose to plugins
include in diagnostics
```

---

# 57. Single-Use Preferred

Where provider permits:

```text
single-use anonymous reply token
```

reduces correlation/replay risk.

---

# 58. Reply Pool

Clients may maintain a bounded pool of reply blocks for asynchronous responses.

---

# 59. Reply Pool Replenishment

Must occur independently enough that it does not trivially correlate with a specific incoming message.

---

# 60. Anonymous Contact Bootstrap

SIAR can support anonymous introduction flows.

Potential:

```text
Invitation
→ temporary anonymous address
→ mixnet exchange
→ E2EE identity negotiation
→ optional persistent relationship
```

---

# 61. Bootstrap Privacy

Do not expose:

```text
normal direct endpoint
relay endpoint
device IP
```

during strict anonymous bootstrap.

---

# 62. Identity vs Anonymity

Users may communicate anonymously while still cryptographically authenticating to each other.

These are separate properties.

---

# 63. Pseudonymous Mode

Future SIAR may support:

```text
stable pseudonymous application identity
+
anonymous network transport
```

---

# 64. Anonymous Mode

Could instead support:

```text
ephemeral application identity
+
anonymous network transport
```

depending product feature.

---

# 65. Route Policy Engine Integration

Existing multipath engine should become privacy-aware.

Concept:

```text
Route Candidate
+
Route Capabilities
+
Privacy Constraints
+
Performance Constraints
→ Valid Candidate Set
```

---

# 66. Route Filtering

First:

```text
eliminate routes violating hard requirements
```

Then:

```text
rank remaining routes
```

---

# 67. Hard Constraint Example

```text
MaximumAnonymity
```

rejects:

```text
DirectInternet
Relay
LAN
Bluetooth
```

unless they are part of an explicitly approved anonymous overlay.

---

# 68. Preference Example

```text
Anonymous
```

may:

```text
prefer Nym
allow approved alternate anonymous provider
```

---

# 69. Path Scoring

```rust
pub struct RouteScore {
    pub privacy: f32,
    pub latency: f32,
    pub reliability: f32,
    pub bandwidth: f32,
    pub power: f32,
}
```

Hard constraints always precede scoring.

---

# 70. Never Average Away Security Requirement

A very fast direct path must never beat a mandatory mixnet requirement.

---

# 71. Multipath Interaction

For ordinary mode:

```text
parallel candidate paths possible
```

For strict anonymity:

```text
only anonymity-compatible paths participate
```

---

# 72. Route Migration

If anonymous route changes between mix providers:

```text
new provider session
```

must preserve application delivery semantics without exposing identity linkage unnecessarily.

---

# 73. Failover

Allowed:

```text
Nym Provider A
→ Nym Provider B
```

if anonymity policy accepts both.

---

# 74. Cross-Class Failover

Not allowed without policy:

```text
Mixnet
→ ordinary relay
```

---

# 75. DTN Integration

DTN and anonymity are complementary.

Potential:

```text
E2EE message
→ anonymous envelope
→ mixnet
```

or:

```text
E2EE message
→ anonymous-capable DTN carrier
```

But ordinary DTN relays may leak proximity or route metadata.

---

# 76. Anonymous DTN Classification

```rust
pub enum DtnPrivacyClass {
    Ordinary,
    MetadataMinimized,
    AnonymousCompatible,
}
```

---

# 77. Maximum Anonymity + DTN

Only use DTN routes explicitly certified as compatible with anonymity guarantees.

---

# 78. Bluetooth/LAN Interaction

These can reveal:

```text
physical proximity
local network membership
device radio identifiers
```

Therefore they should normally be disabled in Maximum Anonymity mode.

---

# 79. Nearby Discovery

Strict anonymity profile should disable:

```text
automatic local discovery
```

unless the user explicitly activates a local-proximity feature.

---

# 80. Presence Integration

Presence leaks communication metadata.

Anonymity profile may set:

```text
presence = disabled / coarse
```

---

# 81. Typing Indicators

Maximum anonymity default:

```text
off
```

---

# 82. Read Receipts

Maximum anonymity default:

```text
off
```

or:

```text
delayed/batched
```

---

# 83. Delivery Receipts

Transport reliability may still require internal ACKs.

They should not automatically become user-visible read receipts.

---

# 84. Timestamp Precision

High-anonymity mode may reduce exposed timestamp precision.

---

# 85. Link Preview Privacy

Disable external automatic URL fetches in strict anonymity mode.

---

# 86. Avatar Privacy

Do not fetch avatars from external origins directly.

Use:

```text
SIAR-encrypted/local data
```

---

# 87. Plugin Restrictions

Plugins with:

```text
NetworkAccess
ReadMessages
ReadContacts
```

can undermine anonymity.

---

# 88. Maximum Anonymity Plugin Policy

Possible:

```text
network-capable plugins suspended
message-reading plugins restricted
external integrations disabled
```

---

# 89. Plugin Capability Gate

```rust
pub enum AnonymityPluginPolicy {
    Normal,
    RestrictNetworkPlugins,
    DisableSensitivePlugins,
}
```

---

# 90. Telemetry

Maximum-anonymity mode:

```text
no identifying telemetry
```

Prefer local-only diagnostics.

---

# 91. Logging

Never log:

```text
mix route
full anonymous address
reply block
mailbox secret
packet payload
direct-peer identity correlation
```

---

# 92. Privacy-Safe Metrics

Possible:

```text
anonymous send success
anonymous send latency bucket
provider availability
queue depth
cover traffic bytes
```

without peer identities.

---

# 93. Diagnostics

Normal UX:

```text
Anonymous route connected
Anonymous route unavailable
Messages waiting for anonymous route
```

---

# 94. Developer Diagnostics

Could expose:

```text
provider name
connection state
latency class
cover traffic state
packet queue size
```

but not route secrets.

---

# 95. Do Not Expose Mix Route

Showing every mix hop can undermine operational privacy and is generally unnecessary.

---

# 96. Anonymous Transport Status

```rust
pub enum AnonymousTransportStatus {
    Disabled,
    Starting,
    Ready,
    Degraded,
    Unavailable,
    Suspended,
}
```

---

# 97. Anonymous Delivery State

```rust
pub enum AnonymousDeliveryState {
    Queued,
    Packetizing,
    Submitted,
    Mixing,
    MailboxDeposited,
    Delivered,
    Failed,
    Expired,
}
```

---

# 98. User-Facing Simplification

Display:

```text
Queued
Sending anonymously
Delivered
Waiting for anonymous route
```

not every internal state.

---

# 99. Large Attachments

Mixnets are inefficient for large files.

Recommended architecture:

```text
small attachment
→ mixnet chunks

large attachment
→ privacy-preserving rendezvous/token
→ separate approved private transport
```

only if anonymity guarantees are preserved.

---

# 100. Large File Policy

```rust
pub enum AnonymousAttachmentPolicy {
    MixnetOnly,
    AnonymousRendezvous,
    AskUser,
    DisallowLargeTransfer,
}
```

---

# 101. No Silent Large-File Downgrade

Never:

```text
mixnet too slow
→ direct file transfer
```

without explicit policy/user approval.

---

# 102. Attachment Threshold

Configured by:

```text
provider capabilities
network cost
battery
latency
```

not one hardcoded global value.

---

# 103. Realtime Calls

Mixnets are generally not appropriate as the media plane for interactive voice/video.

---

# 104. Anonymous Call Signaling

Possible:

```text
call invitation/signaling through mixnet
```

but media may reveal endpoints unless separately protected.

---

# 105. Call Privacy Classes

```rust
pub enum CallPrivacyMode {
    Standard,
    RelayPrivate,
    AnonymousSignaling,
}
```

Do not call the media session "fully anonymous" unless the media path actually satisfies that property.

---

# 106. Emergency Mode Interaction

Emergency routing prioritizes:

```text
reachability
latency
survivability
```

Maximum anonymity may conflict.

---

# 107. Emergency Policy

The user should explicitly choose between:

```text
Emergency Reachability
Maximum Anonymity
```

where these goals conflict.

---

# 108. Never Silently Disable Emergency Reachability

If Maximum Anonymity prevents local/Bluetooth routes, explain clearly.

---

# 109. Power Management

Mixnets consume extra resources because of:

```text
persistent connectivity
cover traffic
packet processing
mailbox polling
```

---

# 110. Battery Policy

```rust
pub enum AnonymousPowerPolicy {
    Adaptive,
    PreserveBattery,
    MaintainAnonymity,
}
```

---

# 111. Maintain Anonymity

Can keep cover traffic despite battery cost if explicitly selected.

---

# 112. Battery Saver

If OS battery saver limits background activity:

```text
anonymity transport may degrade
```

Never silently switch to direct transport.

---

# 113. Android Background Constraints

Nym/mixnet runtime must integrate with:

```text
foreground service where justified
background restrictions
Doze
network changes
battery saver
```

---

# 114. Android Process Death

Durable anonymous outbound queue remains in Rust storage.

---

# 115. Desktop Runtime

Can run anonymity transport in:

```text
daemon
```

independently of UI.

---

# 116. Queue Durability

Anonymous outgoing messages must be persisted before attempting delivery.

---

# 117. Queue Record

```rust
pub struct AnonymousOutboxRecord {
    pub delivery_id: AnonymousDeliveryId,
    pub envelope_ref: EncryptedPayloadRef,
    pub provider: AnonymousProviderId,
    pub policy: AnonymousDeliveryPolicy,
    pub state: AnonymousDeliveryState,
    pub created_at: Timestamp,
    pub expires_at: Timestamp,
}
```

---

# 118. Payload Storage

Encrypted at rest.

---

# 119. Retry

Mixnet retry semantics must avoid:

```text
duplicate user-visible messages
replay amplification
```

---

# 120. Application Message ID

Deduplication remains application-level.

---

# 121. Anonymous Delivery ID

Transport-specific.

Do not equate it with:

```text
MessageId
```

---

# 122. Replay Protection

Use:

```text
authenticated envelope IDs
expiry
dedup store
reply-token use limits
```

---

# 123. Expiry

Anonymous packets/messages need bounded lifetime.

---

# 124. Expired Message

User-facing:

```text
Message expired before an anonymous route became available.
```

---

# 125. Clock Considerations

Do not rely solely on exact wall-clock synchronization for security-critical replay prevention.

---

# 126. Key Material

Mixnet/provider keys must be separated from:

```text
SIAR identity keys
E2EE ratchet keys
device trust keys
backup keys
```

---

# 127. Key Domains

```text
identity key
device key
session E2EE key
anonymous provider key
mailbox key
reply-token secret
```

all separate.

---

# 128. Key Storage

Use existing secure key-store architecture.

---

# 129. Key Rotation

Support:

```text
provider identity rotation
mailbox rotation
reply-token rotation
```

---

# 130. Provider Compromise

A compromised provider should not expose SIAR plaintext or primary identity private keys.

---

# 131. Node Discovery

Provider adapter obtains mixnet topology through provider-approved mechanism.

---

# 132. Core SIAR Does Not Interpret Nym Blockchain Internals

Keep adapter boundary.

---

# 133. Topology Cache

Provider may keep:

```text
signed/validated topology snapshot
```

according to provider requirements.

---

# 134. Stale Topology

May degrade route availability.

Do not silently use untrusted topology.

---

# 135. Censorship / Blocking

High-anonymity provider connectivity may be blocked.

Architecture should support:

```text
provider transport adapters
bridges/obfuscation if provider supports
multiple entry mechanisms
```

without compromising security.

---

# 136. Censorship Mode

Future:

```rust
pub enum AnonymousAccessMode {
    Normal,
    RestrictedNetwork,
    Bridge,
}
```

---

# 137. Sybil Resistance

If SIAR later builds a native mixnet, node admission and Sybil resistance become major separate architecture topics.

Do not improvise them inside Part 34.

---

# 138. Native Mixnet Future

Potential future SIAR-native architecture:

```text
Client
  │
Gateway
  │
Layer 1 Mixes
  │
Layer 2 Mixes
  │
Layer 3 Mixes
  │
Mailbox/Gateway
  │
Recipient
```

---

# 139. Native Mixnet Components

Would require:

```text
directory/topology
node identity
node selection
mix packet format
delays
cover traffic
loop traffic
mailboxes
reply blocks
abuse resistance
node incentives/governance
Sybil resistance
measurement
upgrade protocol
```

Each deserves its own later specification.

---

# 140. Do Not Implement New Cryptography

Use:

```text
well-reviewed established constructions
audited crates
provider libraries
```

where possible.

---

# 141. Rust Safety Boundary

Prefer safe Rust.

Any FFI/native code must be isolated behind narrow wrappers.

---

# 142. Dependency Policy

Anonymous transport dependencies need stricter review because compromise can undermine anonymity.

---

# 143. Supply Chain

Require:

```text
pinned dependencies
cargo-deny
SBOM
audit
signed releases where available
reproducible-build effort
```

---

# 144. Threat Model

Adversaries may include:

```text
local network observer
ISP
relay observer
mix node operator
malicious mix node
colluding mix nodes
global passive observer
active packet injector
malicious recipient
malicious sender
compromised plugin
compromised provider
```

---

# 145. Global Passive Observer

Maximum-anonymity architecture should be designed with this adversary in mind, while being honest that implementation and operational weaknesses can still leak information.

---

# 146. Active Attacker

Must consider:

```text
packet tagging
delay manipulation
replay
dropping
traffic shaping
selective denial
```

---

# 147. Intersection Attacks

Long-term user activity patterns can still reveal information.

Mitigations include:

```text
cover traffic
persistent receive behavior
batching
timing normalization
```

but cannot guarantee perfect protection.

---

# 148. Endpoint Compromise

A mixnet cannot protect anonymity if:

```text
device malware
plugin leakage
OS compromise
```

reveals user activity.

---

# 149. Contact Correlation

Application features themselves can create linkability.

Maximum-anonymity profile must minimize these.

---

# 150. Abuse Resistance

Anonymous systems can be abused.

Architecture should support provider/core mechanisms for:

```text
rate limits
resource quotas
mailbox quotas
spam control
invitation capabilities
```

without requiring global identity exposure.

---

# 151. Anonymous Rate Limiting

Prefer:

```text
privacy-preserving tokens
capability tokens
local quotas
```

rather than identity-based tracking where possible.

---

# 152. Spam

Unknown anonymous inbound messages should default to:

```text
requests/quarantine
```

---

# 153. Attachment Spam

Do not auto-download anonymous attachments from unknown senders.

---

# 154. Malware Safety

Anonymous transport does not imply trusted content.

Existing file validation/sandbox rules remain.

---

# 155. Metadata Budget

Every feature in anonymous mode should declare what metadata it emits.

---

# 156. Metadata Classification

```rust
pub enum MetadataExposure {
    NoneExpected,
    LocalOnly,
    ProviderVisible,
    NetworkVisible,
    PeerVisible,
}
```

---

# 157. Privacy Review

Any new feature that runs under Maximum Anonymity must be reviewed for:

```text
network fetches
timing
identifiers
telemetry
plugins
background traffic
```

---

# 158. Configuration

Human-readable RON:

```ron
(
    mode: MaximumAnonymity,
    provider: "nym",
    fallback: NeverFallback,
    cover_traffic: Maximum,
    power_policy: MaintainAnonymity,
)
```

---

# 159. Secrets Not in RON

Never store:

```text
private keys
mailbox secrets
reply tokens
```

in plaintext configuration.

---

# 160. Runtime Config

```rust
pub struct AnonymousTransportConfig {
    pub provider: AnonymousProviderId,
    pub fallback: AnonymityFallbackPolicy,
    pub cover: CoverTrafficPolicy,
    pub power: AnonymousPowerPolicy,
    pub latency: AnonymousLatencyClass,
}
```

---

# 161. Policy Resolution

```text
User Privacy Mode
+
Conversation Policy
+
Enterprise Policy
+
Platform Capability
+
Provider Availability
→ Effective Anonymity Policy
```

---

# 162. Conversation Override

User may choose:

```text
Always use anonymous routing for this conversation
```

---

# 163. Conversation Policy

```rust
pub struct ConversationPrivacyPolicy {
    pub routing_mode: PrivacyRoutingMode,
    pub fallback: AnonymityFallbackPolicy,
}
```

---

# 164. Enterprise/Managed Policy

Organization may:

```text
require anonymity
forbid external mixnet providers
restrict provider list
```

---

# 165. Policy Conflict

Security/privacy strongest mandatory constraint wins unless policy explicitly defines otherwise.

---

# 166. UI Integration

Settings:

```text
Privacy Routing
    Standard
    Private
    Anonymous
    Maximum Anonymity
```

---

# 167. User Education

Maximum Anonymity explanation:

```text
Routes messages through an anonymity network and may increase delay,
battery use, and bandwidth. SIAR will not fall back to less-private
routes without your permission.
```

---

# 168. Per-Conversation Indicator

Subtle:

```text
Anonymous routing
```

Do not clutter every message.

---

# 169. Route Failure Banner

```text
Anonymous route unavailable
Messages are waiting rather than using a less-private connection.
```

---

# 170. Status Semantics

Never say:

```text
Fully anonymous
```

as an absolute guarantee.

Prefer:

```text
Maximum Anonymity mode
```

or:

```text
Anonymous routing enabled
```

---

# 171. Diagnostics Integration

Network diagnostics should add:

```text
Anonymous Transport
Provider
Status
Queued anonymous messages
Cover traffic state
```

---

# 172. Support Bundle Redaction

Exclude:

```text
mailbox address
reply tokens
route hops
provider private keys
peer correlation
```

---

# 173. Search Integration

Local search is safe.

External semantic search/AI integrations should be disabled or separately authorized in Maximum Anonymity mode.

---

# 174. Backup Integration

Backup may include:

```text
anonymous settings
provider-neutral mailbox metadata
```

but should generally not restore stale live anonymous session state blindly.

---

# 175. New Device Restore

Create fresh:

```text
anonymous provider identity
mailbox credentials
reply-token pool
```

where security requires.

---

# 176. Device Revocation

Revoking device should also invalidate/rotate its anonymous receiving capabilities when possible.

---

# 177. Multi-Device Sync

Do not synchronize live reply tokens between devices unless protocol explicitly requires it.

---

# 178. Provider Migration

Architecture must support:

```text
Nym
→ future provider
```

without changing core MessageId/E2EE state.

---

# 179. Migration State

```rust
pub enum AnonymousProviderMigrationState {
    Idle,
    Preparing,
    DualReceive,
    Switching,
    RetiringOld,
    Complete,
    Failed,
}
```

---

# 180. Dual Receive

Temporary overlap may be required to avoid lost messages.

Must be bounded to reduce correlation.

---

# 181. Provider Health

```rust
pub struct AnonymousProviderHealth {
    pub status: AnonymousTransportStatus,
    pub last_success: Option<Timestamp>,
    pub queue_depth: u32,
    pub estimated_latency_class: AnonymousLatencyClass,
}
```

---

# 182. No Peer-Specific Provider Metrics

Avoid telemetry that correlates provider timing with specific contacts.

---

# 183. Testing Strategy

Part 34 requires dedicated:

```text
unit
property
integration
simulation
adversarial
performance
privacy
fault-injection
```

tests.

---

# 184. Unit Tests

Test:

```text
policy resolution
fallback rejection
queue lifecycle
expiry
fragment bounds
```

---

# 185. Property Tests

Properties:

```text
MaximumAnonymity never selects direct path
duplicate anonymous packet never duplicates user message
invalid reply token rejected
fragment reassembly bounded
```

---

# 186. Downgrade Test

Mandatory invariant:

```text
anonymous provider unavailable
+
NeverFallback
→
message remains queued
```

---

# 187. Route Selection Tests

Matrix:

```text
Standard + Direct
Private + Relay
Anonymous + Nym
Maximum + Nym
Maximum + Nym unavailable
```

---

# 188. Network Fault Tests

Simulate:

```text
provider unreachable
topology stale
mix path drops
mailbox unavailable
high latency
packet duplication
packet reordering
```

---

# 189. Offline Tests

Recipient offline:

```text
sender submits
mailbox stores
recipient later retrieves
dedup applies
```

---

# 190. Reply Tests

```text
single-use reply
expired reply
replayed reply
pool depletion
```

---

# 191. Cover Traffic Tests

Verify:

```text
policy active
rate bounded
battery policy applied
real/cover path indistinguishable at application boundary
```

without claiming cryptographic proof from UI tests.

---

# 192. Metadata Leak Tests

Use canary values to ensure:

```text
AccountId
DeviceId
peer address
MessageId
```

do not appear in provider-facing fields unless explicitly required.

---

# 193. Logging Leak Tests

Canary secrets must never appear in:

```text
logs
support bundles
crash reports
diagnostics
```

---

# 194. Plugin Leak Tests

Maximum anonymity:

```text
network plugin blocked/restricted
```

as policy defines.

---

# 195. Link Preview Tests

External fetch suppressed.

---

# 196. Presence Tests

Maximum mode prevents normal high-frequency presence traffic if configured.

---

# 197. Performance Tests

Measure:

```text
send latency
queue throughput
packetization cost
mailbox retrieval
memory
CPU
cover traffic bandwidth
battery impact
```

---

# 198. Android Tests

Measure:

```text
Doze
battery saver
background restriction
process death
network switching
```

---

# 199. Desktop Tests

Measure:

```text
daemon restart
sleep/resume
network switching
```

---

# 200. Provider Adapter Contract Tests

Nym implementation must satisfy provider-neutral test suite.

---

# 201. Fake Mixnet

`siar-anonymity-testkit` should include deterministic simulated mixnet.

---

# 202. Simulated Mixnet Features

```text
configurable layers
latency
drops
duplicates
mailbox
reply token
route failure
```

---

# 203. No Real Network Required for Most Tests

Real Nym integration tests should be separate.

---

# 204. Integration Test Tiers

```text
Fake Mixnet PR tests
Nym test environment nightly
Production-like release validation
```

---

# 205. Fuzzing

Fuzz:

```text
anonymous envelope parser
fragment parser
reply descriptor parser
provider adapter input
```

---

# 206. Resource Exhaustion

Test:

```text
fragment flood
mailbox flood
reply-token flood
queue flood
```

---

# 207. Backpressure

Anonymous queue must have quotas.

---

# 208. Quota Model

```rust
pub struct AnonymousQueueLimits {
    pub max_messages: u32,
    pub max_bytes: u64,
    pub max_incomplete_fragments: u32,
}
```

---

# 209. Queue Full

User-facing:

```text
Anonymous send queue is full.
```

Offer safe management.

---

# 210. Do Not Downgrade Because Queue Is Full

Hard rule.

---

# 211. Monitoring

Local observability:

```text
provider availability
queue depth
cover bytes
success/failure count
latency buckets
```

---

# 212. SLOs

Anonymous transport needs separate SLOs from realtime transport.

Examples:

```text
availability
submission success
mailbox retrieval
bounded queue persistence
```

---

# 213. No Realtime SLA

Do not evaluate mixnet against voice-call latency standards.

---

# 214. Failure Semantics

```rust
pub enum AnonymousTransportError {
    ProviderUnavailable,
    PolicyViolation,
    QueueFull,
    PacketizationFailed,
    MailboxUnavailable,
    ReplyTokenInvalid,
    ReplyTokenExpired,
    TopologyUnavailable,
    Unsupported,
    Internal,
}
```

---

# 215. PolicyViolation

Used when operation would require privacy downgrade.

---

# 216. Security-Critical Errors

Must fail closed:

```text
invalid topology/authentication
reply token integrity failure
packet authentication failure
policy mismatch
```

---

# 217. Unsupported Feature

Example:

```text
live video unavailable in Maximum Anonymity mode
```

should be explicit.

---

# 218. Feature Capability Query

```rust
pub trait PrivacyCapabilityResolver {
    fn evaluate(
        &self,
        feature: FeatureKind,
        mode: PrivacyRoutingMode,
    ) -> FeaturePrivacyCompatibility;
}
```

---

# 219. Compatibility States

```rust
pub enum FeaturePrivacyCompatibility {
    Supported,
    SupportedWithDegradation,
    RequiresDowngrade,
    Unsupported,
}
```

---

# 220. User-Controlled Downgrade

If action requires weaker privacy:

```text
This call cannot use Maximum Anonymity.
Switch to Relay-Private for this call?
```

Explicit choice only.

---

# 221. Temporary Downgrade

Should be scoped:

```text
this action
this call
this conversation
```

not global unless user chooses.

---

# 222. Policy Audit Event

Record locally:

```text
privacy mode changed
downgrade approved
provider changed
```

without sensitive peer metadata where possible.

---

# 223. Privacy Event

```rust
pub enum PrivacyRoutingEvent {
    ModeChanged,
    ProviderChanged,
    DowngradeRequested,
    DowngradeApproved,
    DowngradeRejected,
    AnonymousRouteLost,
    AnonymousRouteRestored,
}
```

---

# 224. Security Center Integration

Could show:

```text
Privacy routing: Maximum Anonymity
Anonymous provider: Ready
Direct fallback: Disabled
```

---

# 225. First-Run Integration

Do not force mixnet setup on every user.

Offer under:

```text
Privacy
Advanced Privacy
```

---

# 226. Recommended Rollout

Phase 1:

```text
provider abstraction
Nym adapter
anonymous text/control messages
no downgrade
durable queue
basic diagnostics
```

---

# 227. Phase 2

```text
anonymous mailbox
reply tokens/SURBs
multi-device receive
attachment chunking
```

---

# 228. Phase 3

```text
cover traffic controls
strict anonymity profile
plugin/presence suppression
provider migration
```

---

# 229. Phase 4

```text
native SIAR mixnet research
```

---

# 230. Do Not Start With Native Mixnet

Building:

```text
mix-node network
directory
Sybil resistance
node selection
incentives
measurements
```

is a major standalone project.

Use a provider adapter first.

---

# 231. Production Readiness Checklist

Before enabling Anonymous mode:

```text
provider adapter audited
downgrade invariants tested
queue crash-safe
logs redacted
mailbox secrets secure
multi-device semantics defined
battery impact measured
provider failure UX complete
```

---

# 232. Maximum Anonymity Readiness

Stricter:

```text
no direct fallback
metadata features suppressed
plugins restricted
external fetch disabled
support bundle redacted
cover traffic configured
reply/mailbox handling tested
```

---

# 233. Security Review Questions

For every anonymous feature ask:

```text
Can this reveal sender IP?
Can this reveal recipient IP?
Can this correlate AccountId to mailbox?
Can this create timing correlation?
Can this leak through plugins?
Can this fetch external resources directly?
Can this silently downgrade?
Can logs expose route secrets?
```

---

# 234. Architectural Invariants

The following are mandatory:

```text
1. E2EE always remains independent of mixnet.
2. MaximumAnonymity never silently selects non-anonymous paths.
3. Provider-specific types never leak into core messaging.
4. Anonymous addresses are separate from AccountId/DeviceId.
5. Reply tokens are secret and never logged.
6. Mixnet failures do not corrupt application message state.
7. Application MessageId provides user-level deduplication.
8. Anonymous queue is durable before send.
9. Large transfers never silently downgrade.
10. Realtime media is never described as anonymous unless its entire path satisfies the claim.
11. Plugins cannot bypass anonymity policy.
12. Diagnostics never reveal full anonymous route secrets.
```

---

# 235. Recommended Crate API Boundary

```text
siar-messaging
    │
    ▼
siar-routing
    │
    ▼
siar-anonymity-core
    │
    ├── siar-anonymity-policy
    ├── siar-anonymity-mailbox
    ├── siar-anonymity-cover
    └── provider
          └── siar-anonymity-nym
```

---

# 236. Dependency Direction

Never:

```text
siar-messaging
→ nym-sdk
```

Instead:

```text
siar-messaging
→ anonymous transport trait
→ nym adapter
```

---

# 237. Example Send Flow

```text
User sends message
      │
      ▼
Messaging validates
      │
      ▼
E2EE envelope created
      │
      ▼
Route policy:
MaximumAnonymity
      │
      ▼
Select AnonymousTransport
      │
      ▼
Persist anonymous outbox
      │
      ▼
Packetize / normalize
      │
      ▼
Nym adapter
      │
      ▼
Mixnet
      │
      ▼
Anonymous mailbox / recipient
      │
      ▼
Recipient unwraps transport
      │
      ▼
E2EE decrypt
      │
      ▼
Message committed
```

---

# 238. Example Provider Failure

```text
Message persisted
      │
      ▼
Nym unavailable
      │
      ▼
Policy = NeverFallback
      │
      ▼
Remain queued
      │
      ▼
UI: "Waiting for anonymous route"
```

No direct route is attempted.

---

# 239. Example Anonymous Reply

```text
Alice
  │
  ├─ sends encrypted message
  └─ includes reply capability
          │
          ▼
        Mixnet
          │
          ▼
Bob receives
  │
  └─ uses reply capability
          │
          ▼
        Mixnet
          │
          ▼
Alice
```

Bob need not learn Alice's network location.

---

# 240. Example Strict Metadata Profile

```text
Maximum Anonymity
├── Mixnet required
├── Never fallback
├── Cover traffic enabled
├── Presence off
├── Typing off
├── Read receipts off/delayed
├── External previews off
├── Network plugins restricted
├── Local diagnostics only
└── Anonymous mailbox enabled
```

---

# 241. Configuration Migration

Existing users default:

```text
Standard
```

Do not silently move current traffic to expensive anonymity mode.

---

# 242. User Opt-In

Anonymous modes should be explicit because they change:

```text
latency
battery
bandwidth
feature compatibility
```

---

# 243. Enterprise Defaults

Managed deployments may choose stricter default.

---

# 244. Documentation

User documentation must explain:

```text
what anonymity protects
what it does not protect
latency tradeoffs
battery/bandwidth tradeoffs
feature limitations
```

---

# 245. No Absolute Claims

Avoid:

```text
impossible to trace
100% anonymous
untraceable
```

Use:

```text
designed to reduce metadata linkage
stronger anonymity protection
mixnet routing
```

---

# 246. Future Extensions

Potential later architecture parts:

```text
Anonymous Mailbox & Reply Capability Architecture
Native Loopix-Inspired Mixnet Architecture
Sphinx Packet & Cell Architecture
Cover Traffic & Traffic-Shaping Architecture
Mixnet Directory / Node Selection / Sybil Resistance
Anonymous Attachment & Rendezvous Architecture
Anonymity Threat Model & Formal Privacy Verification
Censorship Resistance / Bridges / Obfuscation
```

These should build on Part 34 rather than duplicate it.

---

# 247. Initial Production Scope

Implement first:

```text
AnonymousTransport trait
Nym provider adapter
PrivacyRoutingMode
AnonymityFallbackPolicy
strict route filtering
durable anonymous outbox
small text/control messaging
provider-neutral status
basic packet-size normalization
privacy-safe diagnostics
no-downgrade tests
```

Then add:

```text
mailbox
SURB/reply capability integration
multi-device anonymous receiving
cover traffic controls
anonymous attachments
strict metadata profile
```

---

# 248. Definition of Done

Part 34 is complete when:

- high-anonymity transport exists as a separate transport plane
- Mixnet, Loopix, Sphinx, and Nym roles are clearly distinguished
- provider-neutral `AnonymousTransport` abstraction is defined
- Nym is integrated only through an adapter
- Standard/Private/Anonymous/MaximumAnonymity routing modes exist
- `NeverFallback` prevents privacy downgrade
- anonymous routes integrate with the multipath policy engine
- E2EE remains independent and mandatory for protected message contents
- normalized/fixed-size packetization concepts are defined
- durable anonymous queue semantics are defined
- anonymous mailboxes/offline receiving are accounted for
- anonymous reply/SURB-style capabilities are accounted for
- metadata-sensitive presence/typing/receipts/previews/plugins are handled by anonymity profile
- Bluetooth/LAN/DTN interactions are explicitly constrained by privacy mode
- large attachment behavior cannot silently downgrade
- realtime media limitations are explicit
- Android battery/background and desktop daemon behavior are defined
- privacy-safe observability and diagnostics are defined
- threat model covers passive, active, colluding, provider, plugin, and endpoint adversaries
- no-new-cryptography rule is explicit
- testkit, provider contract tests, downgrade invariants, leak tests, fuzzing, and fault injection are defined
- production rollout can begin with Nym without forcing SIAR to build its own mixnet

---

# 249. Final Architecture

```text
                         SIAR APPLICATION
                               │
                               ▼
                         Application E2EE
                               │
                               ▼
                      Privacy Routing Policy
                               │
      ┌────────────────────────┼────────────────────────┐
      │                        │                        │
Standard Paths             DTN Paths           Anonymous Paths
      │                        │                        │
Iroh / QUIC              Store/Forward        AnonymousTransport
Relay                    Opportunistic               │
LAN                      Mesh                        ├── Nym
Wi-Fi Direct                                         │
Bluetooth                                            └── Native Mixnet
                                                         [future]
```

Strict anonymity path:

```text
Application Message
      ↓
E2EE
      ↓
Anonymous Envelope
      ↓
Normalized Cells
      ↓
Sphinx / Provider Packetization
      ↓
Mix Layers
      ↓
Anonymous Mailbox / Reply Path
      ↓
Recipient
      ↓
E2EE Decryption
```

Security invariant:

```text
Maximum Anonymity
+
anonymous provider unavailable
=
WAIT

not

FALL BACK TO DIRECT
```

---

# 250. Final Principle

The high-anonymity architecture should not make every SIAR transport anonymous.

Instead, SIAR should become **privacy-policy driven**:

```text
realtime when realtime matters
local when local matters
DTN when disconnected
mixnet when anonymity matters
```

while preserving strict boundaries so that choosing stronger anonymity never silently degrades into an easier but metadata-leaking route.

This creates a practical path from SIAR's existing multi-transport architecture to strong mixnet-based anonymity today through Nym, while leaving room for a future native Loopix/Sphinx-inspired anonymity network without coupling the rest of the platform to any one provider.
