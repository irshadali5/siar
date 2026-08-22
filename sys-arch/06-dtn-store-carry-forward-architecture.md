# Part 06 — DTN / Store-Carry-Forward Architecture

## Reusable P2P Communication Platform

**Status:** Architecture specification  
**Part:** 06 of 24  
**Primary language:** Rust  
**Primary goals:** communication without continuous end-to-end connectivity, store-carry-forward, opportunistic routing, transport independence, emergency resilience, bounded replication, privacy, crash safety, reusable across messaging/files/ERP/custom applications

---

# 1. Purpose

A normal network assumes that a route exists between sender and receiver at the time of communication.

A disaster-resilient P2P system cannot make that assumption.

A sender may have:

```text
no Internet
no router
no direct Wi-Fi path
no direct Bluetooth path to destination
```

yet there may still be a sequence of future encounters:

```text
Alice
 ↓
Bob
 ↓
Carol
 ↓
Gateway
 ↓
Destination
```

The DTN subsystem provides this behavior.

DTN means:

> **Persist the payload locally, carry it across time, and forward it when useful connectivity appears.**

This architecture must work with:

- text messages
- message receipts
- emergency reports
- SOS
- small file metadata
- selected file chunks
- authority alerts
- custom application events

It must not require a central server.

---

# 2. Fundamental Principle

Do not design DTN as:

```text
Bluetooth forwarding logic
```

or:

```text
Emergency-only forwarding
```

Instead:

```text
Application Payload
      ↓
DTN Bundle
      ↓
Durable Relay Store
      ↓
Encounter Protocol
      ↓
Forwarding Policy
      ↓
Any Available Transport
```

Transport can be:

```text
BLE
Bluetooth Classic
Wi-Fi Direct
Wi-Fi Aware
LAN
Iroh
future radio
```

DTN is a routing/delivery model, not a radio technology.

---

# 3. Architectural Position

```text
Messaging / Files / Emergency / Custom App
                  ↓
           Delivery Intent
                  ↓
       Transport/Routing Engine
                  ↓
       DTN Adapter / DTN Engine
                  ↓
      Durable Bundle Store
                  ↓
     Encounter / Reconciliation
                  ↓
   BLE / Wi-Fi / LAN / Iroh / Relay
```

Part 03 decides whether DTN is allowed.

Part 06 decides how DTN forwarding happens.

---

# 4. DTN Responsibilities

The DTN subsystem owns:

- bundle creation
- durable bundle persistence
- expiry
- hop/replication limits
- peer encounter exchange
- inventory summaries
- bundle deduplication
- opportunistic forwarding
- relay storage quotas
- next-hop scoring
- custody-like state
- gateway handoff
- delivery confirmation propagation
- eviction
- route history
- privacy-aware routing metadata
- priority scheduling
- multi-transport handoff

It does not own:

- chat semantics
- message rendering
- file decoding
- account UI
- codec logic
- raw Bluetooth APIs

---

# 5. Bundle Concept

A DTN bundle is an opaque transferable object.

```rust
pub struct BundleId([u8; 32]);
```

A bundle represents:

```text
payload + routing metadata + security metadata + delivery policy
```

The relay does not need to understand the application payload.

---

# 6. Bundle Structure

Conceptually:

```rust
pub struct DtnBundle {
    pub bundle_id: BundleId,
    pub source: DtnSource,
    pub destination: DtnDestination,
    pub payload_type: PayloadTypeId,
    pub created_at: Timestamp,
    pub expires_at: Timestamp,
    pub priority: DtnPriority,
    pub hop_limit: u8,
    pub replication_budget: u8,
    pub forwarding_class: ForwardingClass,
    pub payload_ref: PayloadReference,
    pub integrity: BundleIntegrity,
}
```

Large content should usually be referenced by blob/chunk identity rather than embedded directly.

---

# 7. Bundle Payload

Supported forms:

```rust
pub enum PayloadReference {
    Inline(Bytes),
    Blob(BlobId),
    Chunk {
        blob: BlobId,
        chunk_index: u32,
    },
    Event(EventId),
}
```

Inline payloads must be strictly bounded.

---

# 8. Bundle Security Principle

Private payloads should already be end-to-end encrypted before entering DTN.

Flow:

```text
plaintext
 ↓
application E2EE
 ↓
ciphertext payload
 ↓
DTN bundle
 ↓
unknown relay peers
```

Relay nodes should not need conversation/file decryption keys.

---

# 9. Relay Trust Model

A relay may be:

```text
unknown
known contact
trusted device
organization node
emergency gateway
```

But forwarding private ciphertext should not require trusting the relay with plaintext.

Separate:

```text
trusted to carry
```

from:

```text
trusted to read
```

---

# 10. Bundle Destination

Use opaque destination identifiers.

Avoid plaintext:

```text
phone number
email
display name
```

in relay-visible headers.

Possible destination forms:

```rust
pub enum DtnDestination {
    DeviceOpaque(RouteToken),
    AccountOpaque(RouteToken),
    GroupOpaque(RouteToken),
    LocalBroadcast(BroadcastScope),
}
```

---

# 11. Routing Tokens

A routing token should be:

- opaque
- short-lived where possible
- difficult to correlate long-term
- sufficient for authorized forwarding

Permanent account IDs should not be advertised unnecessarily in public BLE beacons or relay headers.

---

# 12. Bundle ID

`BundleId` must be stable across retries and replicas.

This enables:

```text
deduplication
delivery tracking
idempotency
```

Do not create a new BundleId for every hop.

---

# 13. Bundle Immutability

Once created, routing-critical bundle fields should be immutable except hop-local metadata.

Example immutable:

```text
bundle_id
destination
expiry
payload identity
origin signature
```

Hop-local mutable state:

```text
remaining replication copies
local forwarding history
last-seen timestamp
```

Keep these separate.

---

# 14. Bundle Envelope vs Local Record

Separate:

```text
WireBundle
```

from:

```text
LocalBundleRecord
```

Local record may contain:

```text
storage path
local retry count
peer history
custody state
```

These do not belong on wire.

---

# 15. Durable Store

Every accepted DTN bundle must be persisted before claiming it is being carried.

```text
Create bundle
 ↓
write durable record
 ↓ COMMIT
eligible for forwarding
```

Part 04 event log can record semantic lifecycle transitions.

---

# 16. Bundle Store Interface

```rust
pub trait BundleStore: Send + Sync {
    async fn put(&self, bundle: StoredBundle) -> Result<(), DtnStoreError>;
    async fn get(&self, id: BundleId) -> Result<Option<StoredBundle>, DtnStoreError>;
    async fn mark_forwarded(&self, ...);
    async fn mark_delivered(&self, ...);
    async fn remove(&self, id: BundleId) -> Result<(), DtnStoreError>;
    async fn list_candidates(&self, query: ForwardQuery)
        -> Result<Vec<StoredBundle>, DtnStoreError>;
}
```

---

# 17. Storage Classes

Bundles can be:

```text
LocalOrigin
Relay
CriticalEmergency
DeliveryReceipt
```

Each can have different retention and eviction policies.

---

# 18. Bundle State Machine

```text
Created
 ↓
Stored
 ↓
Eligible
 ↓
Forwarded
 ↓
ForwardedAgain
 ↓
DestinationReached
 ↓
Acknowledged
 ↓
Completed
```

Alternative terminal states:

```text
Expired
Evicted
Cancelled
Rejected
```

---

# 19. Delivery Semantics

Do not equate:

```text
forwarded
```

with:

```text
delivered
```

The UI and API must distinguish:

```text
Stored locally
Carried by relay
Reached gateway
Reached destination device
Acknowledged by destination
```

---

# 20. Expiry

Every DTN bundle should have explicit expiry.

Examples:

```text
typing indicator → not DTN
SOS → 24–72h
normal message → application-defined
temporary route hint → minutes
```

Expired bundles are never forwarded.

---

# 21. Hop Limit

Each bundle may have:

```text
hop_limit
```

Every forward decrements local remaining hop budget.

At zero:

```text
do not forward further
```

This prevents uncontrolled propagation.

---

# 22. Replication Budget

Hop count alone is insufficient.

Use:

```text
replication_budget
```

Example:

```text
normal message = 2
important message = 4
SOS = 8
```

This bounds duplicate copies.

---

# 23. Spray-and-Wait Baseline

A good initial opportunistic strategy is:

```text
Spray:
create a limited number of copies

Wait:
copies seek destination/gateway
```

This is more controlled than epidemic flooding.

---

# 24. Epidemic Routing

Full epidemic routing:

```text
send everything to everyone
```

is unsuitable as the default because it causes:

- battery drain
- bandwidth waste
- storage explosion
- metadata leakage

Use only for tightly bounded critical emergency classes, if at all.

---

# 25. Direct Delivery

If encountered peer is the destination:

```text
deliver immediately
```

This always outranks relay forwarding.

---

# 26. Gateway Delivery

If peer has reliable Internet or destination reachability:

```text
forward to gateway
```

Gateway can bridge:

```text
BLE → Iroh
Wi-Fi → Internet
LAN → remote peer
```

without decrypting private content.

---

# 27. Encounter Protocol

When two DTN-capable peers meet:

```text
HELLO
 ↓
authenticated capability exchange
 ↓
inventory summary
 ↓
request useful bundles
 ↓
transfer
 ↓
acknowledge stored copies
```

Do not immediately dump all bundles.

---

# 28. Encounter Identity

Nearby discovery can use ephemeral IDs.

Before forwarding sensitive bundles:

```text
authenticate peer/session
```

when policy requires.

Unknown relays may still be eligible for opaque emergency/relay traffic according to application policy.

---

# 29. Inventory Summary

Do not exchange full BundleId lists if store is large.

Initial options:

```text
Bloom filter
range summary
bounded recent-ID list
```

Later:

```text
IBLT
Merkle-set reconciliation
```

---

# 30. Bloom Filter Use

A Bloom filter can summarize:

```text
bundles I already have
```

False positives are acceptable if they only cause missed redundant transfer opportunities.

Do not use Bloom filters for authorization/security decisions.

---

# 31. Reconciliation Flow

```text
Peer A summary
 ↓
Peer B identifies potentially useful missing bundles
 ↓
B requests IDs or classes
 ↓
A sends selected bundles
```

Selection depends on:

```text
priority
expiry
destination likelihood
replication budget
storage
battery
transport bandwidth
```

---

# 32. Peer Utility Score

A peer can receive a forwarding score.

Conceptually:

```text
score =
    destination likelihood
  + gateway probability
  + recent encounter usefulness
  + available storage
  + connectivity quality
  + energy capacity
  - congestion
```

Use deterministic heuristics initially.

---

# 33. Destination Likelihood

Useful hints:

```text
peer recently saw destination
peer is destination's own device
peer is known gateway
peer belongs to same local group
```

Do not require a global social graph.

---

# 34. Encounter History

Maintain bounded history:

```text
peer
last_seen
frequency
successful forwards
gateway_seen
```

This can improve forwarding.

Do not retain unlimited mobility history.

---

# 35. Privacy of Encounter History

Encounter logs can reveal movement patterns.

Keep:

- local
- bounded
- coarse
- retention-limited

Do not upload them by default.

---

# 36. DTN Priority

Recommended:

```rust
pub enum DtnPriority {
    Critical,
    High,
    Normal,
    Low,
    Bulk,
}
```

Examples:

```text
SOS → Critical
delivery receipt → High
text → Normal
thumbnail → Low/Normal
large file chunk → Bulk
```

---

# 37. Scheduler

Per encounter:

```text
Critical
 ↓
Control/ACK
 ↓
Text
 ↓
Small emergency media
 ↓
Voice
 ↓
Thumbnail
 ↓
Bulk chunks
```

Use weighted fairness to avoid total starvation.

---

# 38. Transport-Aware Scheduling

BLE:

```text
small bundles only
```

Wi-Fi Direct:

```text
larger bundles/chunks
```

Iroh:

```text
full functionality
```

DTN scheduler must understand transport capacity.

---

# 39. Bundle Size Classes

Example:

```text
Tiny < 4 KiB
Small < 64 KiB
Medium < 1 MiB
Large >= 1 MiB
```

These are tunable.

Routing policy can forbid large bundles on BLE.

---

# 40. File Integration

Part 05 blob subsystem should provide:

```text
BlobId
ChunkId
Thumbnail BlobId
```

DTN may carry:

```text
manifest
thumbnail
selected chunks
```

rather than full multi-gigabyte file by default.

---

# 41. Chunk Carriage

A DTN bundle can wrap:

```text
Blob X
Chunk 17
```

The receiving node stores opaque chunk ciphertext.

Later it can forward or deliver it.

---

# 42. Chunk Deduplication

Chunk identity enables:

```text
same chunk seen twice
→ store once
```

This reduces relay storage.

---

# 43. Manifest First

For large files:

```text
manifest
 ↓
recipient/relay decides useful chunks
```

Do not blindly send random chunks without object context.

---

# 44. Thumbnail-First Emergency Media

Emergency photo flow:

```text
SOS
 ↓
8–20 KB thumbnail
 ↓
preview
 ↓
full photo later on fast path
```

This greatly improves utility over constrained mesh.

---

# 45. Voice Notes

Low-bitrate Opus voice can be DTN-carried in chunks.

Priority typically above full-resolution images but below text/location.

---

# 46. Realtime Media

DTN does not carry realtime call frames.

Calls require:

```text
continuous path
```

If unavailable:

```text
voice note
text
SOS
```

becomes fallback.

---

# 47. Message Integration

Messaging creates:

```text
MessageQueued
```

If no direct route:

```text
routing engine chooses DTN
 ↓
DTN bundle created
```

The original MessageId remains stable.

---

# 48. Message Deduplication

Destination may receive same message via:

```text
Internet
+
DTN relay
```

Recipient deduplicates by MessageId.

Bundle deduplication and message deduplication are distinct.

---

# 49. Delivery ACK

Destination emits a delivery acknowledgement.

ACK itself can be:

```text
direct
or DTN-carried
```

This lets origin eventually know destination reached.

---

# 50. ACK Compression

Acknowledgements can aggregate:

```text
multiple delivered BundleIds
```

into compact summaries.

Useful in low-bandwidth encounters.

---

# 51. Custody-Like Transfer

A relay may say:

```text
I have durably stored this bundle
```

Origin can then reduce its replication pressure.

Do not call this strict DTN custody transfer unless implementing full custody semantics.

Use a simpler term such as:

```text
DurableRelayAck
```

initially.

---

# 52. Durable Relay Ack

Means:

```text
relay persisted bundle
```

not:

```text
destination received
```

These must never be confused.

---

# 53. Copy Retirement

Origin may retire redundant copies after:

```text
destination ACK
```

or according to policy after sufficient durable relay acknowledgements.

Critical data may retain extra copies longer.

---

# 54. Eviction

Relay storage eviction order:

```text
expired
 ↓
delivered
 ↓
cancelled
 ↓
bulk low-priority
 ↓
old normal
 ↓
high
 ↓
critical last
```

Never use FIFO blindly.

---

# 55. Storage Quotas

Per node:

```text
max DTN bytes
max bundles
max per-peer bytes
max critical reserve
```

Unknown peers should have stricter quotas.

---

# 56. Critical Reserve

Reserve storage for:

```text
SOS
authority alerts
delivery ACKs
```

so bulk content cannot consume all relay capacity.

---

# 57. Per-Peer Quota

Prevent one malicious peer from filling store.

```text
unknown peer = low quota
verified contact = moderate quota
trusted authority = policy-defined
```

Authorization and quota are separate.

---

# 58. Admission Control

Before accepting a relay bundle:

```text
validate size
verify envelope
check expiry
check replication budget
check quota
check forwarding policy
check duplicate
```

Only then persist.

---

# 59. Bundle Validation

Validate:

```text
version
header size
payload size
expiry
hop limit
signature/integrity
destination format
```

Remote input is hostile.

---

# 60. Oversized Bundle

Reject before allocation/storage reservation.

Never trust remote size fields.

---

# 61. Malformed Metadata

Malformed bundle should not affect other stored bundles.

Parsing must be isolated and bounded.

---

# 62. Bundle Encryption

Relay-visible header should reveal only what is needed.

Private payload remains E2EE ciphertext.

Potentially sensitive metadata such as original sender identity may also be minimized/obfuscated depending on routing design.

---

# 63. Source Privacy

Permanent source identity may not need to be visible to relay.

Use:

```text
opaque origin reference
```

if protocol can still authenticate end-to-end.

---

# 64. Destination Privacy

Avoid stable public destination tags.

Use rotating capability/routing tokens where feasible.

Part 14 proximity/discovery architecture can help here.

---

# 65. Linkability Trade-Off

Perfect unlinkability can conflict with efficient routing.

Architecture should document trade-offs explicitly.

Do not claim anonymity simply because payloads are encrypted.

---

# 66. Replay Protection

Same BundleId received repeatedly:

```text
do not duplicate storage
```

But metadata differences for same ID indicate:

```text
protocol violation/tampering
```

---

# 67. Bundle Integrity

Bundle should authenticate:

```text
bundle_id
destination
expiry
payload reference
priority
origin binding
```

so relays cannot alter routing-critical fields unnoticed.

---

# 68. Hop-Local Metadata

Fields changed per hop must not be covered as immutable origin-authenticated data unless represented separately.

Example:

```text
remaining copy budget
local receipt state
```

---

# 69. Cancel Bundle

Origin may issue:

```text
BundleCancel
```

for still-undelivered bundles.

Cancellation propagation is best effort.

Cannot erase already-delivered plaintext.

---

# 70. Expired Bundle Handling

Expired bundles:

```text
do not forward
delete according to retention policy
```

Optional compact expiry summary can help peers discard stale references.

---

# 71. Time Without Internet

Device clocks may be wrong.

Use:

```text
creation timestamp
+
monotonic local age
+
bounded tolerance
```

where possible.

For security-critical expiry, avoid relying solely on untrusted remote wall clock.

---

# 72. Relative Lifetime

Bundle can include:

```text
lifetime duration
```

in addition to creation time.

Local node computes age relative to first trusted observation where appropriate.

---

# 73. Gateway Detection

A peer may advertise:

```text
Internet gateway available
```

as capability.

Routing must verify the path works before trusting strongly.

---

# 74. Gateway Capability

```rust
pub struct GatewayCapability {
    pub internet: bool,
    pub relay_access: bool,
    pub estimated_capacity: CapacityClass,
}
```

This is a hint, not identity.

---

# 75. Gateway Forwarding

Gateway flow:

```text
receive encrypted bundle
 ↓
persist
 ↓
obtain Internet
 ↓
resolve destination
 ↓
Iroh/relay/direct delivery
 ↓
receive destination ACK
 ↓
propagate ACK back via DTN if necessary
```

---

# 76. Cross-Transport Bridge

Example:

```text
Phone A
 ↓ BLE
Phone B
 ↓ Wi-Fi Direct
Laptop C
 ↓ LAN
Gateway D
 ↓ Iroh
Destination
```

Same bundle survives all hops.

---

# 77. Transport Upgrade

During encounter:

```text
discover over BLE
 ↓
both support Wi-Fi Direct
 ↓
upgrade bulk transfer to Wi-Fi
```

DTN session retains bundle semantics.

---

# 78. Transport Downgrade

If Wi-Fi fails:

```text
continue small critical bundles over BLE
bulk pauses
```

No data model reset.

---

# 79. Encounter Session

Define:

```rust
pub struct EncounterSession {
    pub peer: PeerSession,
    pub transport: TransportKind,
    pub capacity: EncounterCapacity,
    pub expires_at: Option<Timestamp>,
}
```

---

# 80. Encounter Capacity

```rust
pub struct EncounterCapacity {
    pub max_bytes: u64,
    pub estimated_duration: Option<Duration>,
    pub bandwidth_class: BandwidthClass,
}
```

Useful for deciding what to exchange first.

---

# 81. Encounter Deadline

A BLE/Wi-Fi peer may disappear quickly.

Scheduler should prioritize:

```text
small critical objects first
```

rather than starting one giant file.

---

# 82. Contact Duration Prediction

Optional heuristic:

```text
recent encounter history
signal trend
transport type
```

can estimate contact duration.

Do not overcomplicate v1.

---

# 83. Store-and-Forward Policy

```rust
pub struct ForwardingPolicy {
    pub allow_unknown_relays: bool,
    pub max_hops: u8,
    pub replication_budget: u8,
    pub max_bundle_size: u64,
    pub expiry: Duration,
}
```

Applications can override safely.

---

# 84. Forwarding Classes

```rust
pub enum ForwardingClass {
    DirectOnly,
    TrustedRelays,
    AnyRelay,
    EmergencyBroadcast,
}
```

This cleanly separates privacy/security expectations.

---

# 85. DirectOnly

Bundle may be stored locally until destination encountered, but not handed to third-party relay.

Useful for highly sensitive payloads.

---

# 86. TrustedRelays

Forward only to:

```text
verified contacts
own devices
organization nodes
```

---

# 87. AnyRelay

Encrypted payload can use unknown relay nodes.

Still subject to:

```text
quota
rate limit
replication budget
```

---

# 88. EmergencyBroadcast

Special controlled dissemination mode.

May use broader forwarding but must remain:

```text
bounded
signed
expiring
rate-limited
```

---

# 89. Local Broadcast

Public emergency alert can target:

```text
local area / encounter domain
```

without individual destination.

This is different from private unicast.

---

# 90. Broadcast Scope

```rust
pub enum BroadcastScope {
    Nearby,
    LocalRegion,
    Organization,
    EmergencyDomain,
}
```

Geographic scoping may remain approximate/offline.

---

# 91. Authority Alerts

Authority alert requires:

```text
origin signature
trusted issuer
expiry
scope
```

Relays can forward without trusting alert text personally; clients verify authority chain.

---

# 92. Spam Resistance

Open DTN can be abused.

Use:

```text
rate limits
bundle size caps
per-origin quotas
proof-of-trust
issuer policy
critical-class restrictions
```

Do not let any unknown peer mark arbitrary traffic `Critical`.

---

# 93. Priority Authorization

Priority should be constrained by authority.

Example:

```text
unknown peer
cannot claim AuthorityCritical
```

Map requested priority through local policy.

---

# 94. Priority Downgrade

If sender requests disallowed priority:

```text
downgrade
or reject
```

Do not trust remote enum blindly.

---

# 95. Congestion

Relay store/transport congestion signals:

```text
queue depth
storage pressure
encounter bandwidth
battery
```

Forwarding scheduler adapts.

---

# 96. Backpressure

If DTN store is full:

```text
reject/defer low priority
```

Do not buffer incoming bundle indefinitely in memory.

---

# 97. Battery Awareness

Part 13 will expand this.

DTN already needs modes:

```text
Normal
BatterySaver
EmergencyBalanced
EmergencyMaximumReach
```

These control:

```text
scan frequency
forwarding volume
relay acceptance
Wi-Fi upgrade attempts
```

---

# 98. Low Battery Behavior

Example:

```text
<10%
accept only:
critical
own messages
delivery ACK
small high-value bundles
```

Do not kill emergency functionality entirely unless unavoidable.

---

# 99. Charging Node

A charging device may become a better relay candidate.

Advertise coarse:

```text
HighRelayCapacity
```

rather than exact battery percentage.

---

# 100. Thermal Awareness

If device is overheating:

```text
reduce bulk relay
avoid Wi-Fi high-throughput upgrade
keep critical control traffic
```

---

# 101. Discovery Duty Cycle

DTN depends on peer encounters.

Scanning strategy must be battery-aware:

```text
normal low duty cycle
nearby screen active higher
emergency high
battery saver low
```

Part 14 will define the proximity abstraction.

---

# 102. Peer Advertisement

Nearby advertisement should be tiny.

Possible:

```text
protocol version
rotating node token
DTN capability bit
gateway hint
```

Do not include full bundle inventory in advertisements.

---

# 103. Session Authentication

After discovery:

```text
establish secure session
```

then exchange:

```text
inventory
bundle metadata
```

This limits metadata exposure.

---

# 104. Unknown Peer Mode

If product allows anonymous relays:

```text
minimal authenticated ephemeral session
```

may still be used to prevent trivial spoofing/replay.

Application-level identity trust remains separate.

---

# 105. Relay Receipt

After persistence:

```text
RelayStored(bundle_id)
```

can be sent.

Origin records:

```text
one durable copy exists
```

Do not call it delivery.

---

# 106. Delivery Receipt Propagation

Destination creates:

```text
DestinationAck(bundle_id)
```

This may travel back through any path.

Once origin receives it:

```text
mark complete
stop forwarding
```

---

# 107. Ack Aggregation

For many bundles:

```text
AckSummary
```

can contain ranges/Bloom-like compact structure.

Must not create false positive delivery semantics.

Use exact IDs/ranges where correctness matters.

---

# 108. Negative Acknowledgement

Peer can report:

```text
RejectedQuota
Expired
Unsupported
Unauthorized
```

Useful to stop useless retries.

---

# 109. Route Learning

Successful delivery through peer can improve:

```text
future utility score
```

Maintain only bounded local history.

---

# 110. No Global Routing Table

Mobile DTN topology changes too quickly.

Do not implement Internet-like full route tables.

Prefer:

```text
neighbor knowledge
recent encounter utility
limited replication
gateway hints
```

---

# 111. Social-Aware Routing

Potential future optimization:

```text
frequently encountered communities
```

but carries privacy risks.

Not recommended for initial release.

---

# 112. Prophet-Like Routing

Future predictive routing algorithms may use encounter probability.

Keep algorithm behind trait:

```rust
pub trait ForwardingStrategy {
    fn select(
        &self,
        bundle: &StoredBundle,
        peer: &PeerContext,
    ) -> ForwardDecision;
}
```

Start with simple deterministic policy.

---

# 113. ForwardDecision

```rust
pub enum ForwardDecision {
    Skip,
    Forward,
    ForwardCopy { copies: u8 },
    DeliverDirect,
}
```

---

# 114. Strategy Plugability

Possible strategies:

```text
DirectOnly
SprayAndWait
GatewayPreferred
EmergencyFloodLimited
```

Compile-time strategy selection initially.

---

# 115. Bundle Replication Counter

Keep local mutable:

```text
copies_remaining
```

Do not let relay inflate it.

Origin-signed max budget constrains it.

---

# 116. Binary Spray-and-Wait

A practical scheme:

```text
if copies = N
forward floor(N/2)
keep ceil(N/2)
```

until each copy has one remaining.

This spreads efficiently while bounded.

---

# 117. Delivery Probability

Peer can advertise or locally estimate:

```text
gateway_probability
destination_probability
```

Use coarse scores.

Never trust peer self-reported score blindly.

---

# 118. Multi-Recipient Bundles

Prefer separate recipient-authorized payload references rather than exposing a recipient list to relays if privacy matters.

Group dissemination can use group-specific protocol.

---

# 119. Bundle Aggregation

Small related control bundles may be packed into one transfer batch.

But maintain individual BundleIds for deduplication/delivery tracking.

---

# 120. Batch Transfer

Encounter transfer can send:

```text
BatchHeader
bundle count
bundle metadata
streamed payloads
```

This reduces handshake overhead.

---

# 121. Postcard Usage

Use Postcard for:

```text
bundle headers
inventory requests
ack summaries
```

Stream large payload bytes separately.

---

# 122. Versioning

DTN extension:

```text
dtn/1
```

Capabilities:

```text
spray-and-wait
relay-ack
gateway-hint
blob-chunk
broadcast
```

Part 01 extension negotiation applies.

---

# 123. Capability Negotiation

Peer may support:

```text
text bundles
blob chunk relay
public alerts
gateway mode
```

Routing chooses only common capabilities.

---

# 124. Device Identity Integration

Part 02 gives:

```text
AccountId
DeviceId
device certificate
trust state
```

DTN uses these for:

```text
origin authentication
destination resolution
trusted-relay policy
```

---

# 125. Transport/Routing Integration

Part 03 decides:

```text
should this operation use DTN?
```

DTN then manages:

```text
store
replicate
forward
```

If direct route later appears, Part 03 may bypass DTN and deliver immediately.

---

# 126. Event Log Integration

Part 04 records meaningful lifecycle:

```text
BundleCreated
BundleForwarded
DestinationReached
BundleExpired
```

High-frequency encounter metrics stay out of semantic event log.

---

# 127. File/Blob Integration

Part 05 gives:

```text
BlobId
ChunkId
manifest
encrypted chunks
```

DTN stores/forwards references or selected ciphertext chunks.

---

# 128. Crash Recovery

After process death:

```text
load bundle store
 ↓
remove expired
 ↓
restore forwarding state
 ↓
resume encounter logic
```

No bundle accepted durably should vanish silently.

---

# 129. Atomic Bundle Acceptance

Transaction:

```text
validate
reserve storage
write payload/ref
write bundle record
commit
```

Only after commit:

```text
send RelayStored ACK
```

---

# 130. Staging

Large incoming relay payload may use staging.

After verification:

```text
commit to relay blob store
```

Incomplete data remains resumable or disposable per policy.

---

# 131. Partial Relay Transfer

An encounter may end mid-bundle.

For chunked blob bundles:

```text
resume later
```

For small inline bundle:

```text
restart bounded transfer
```

---

# 132. Encounter Resumption

Stable:

```text
BundleId
ChunkId
```

allows next encounter to continue without full restart.

---

# 133. Process Ownership

In daemon mode, DTN engine should live in background daemon/service.

On Android, foreground/background execution rules apply.

UI does not own forwarding lifecycle.

---

# 134. Android Background Restrictions

Kotlin/platform layer reports:

```text
background allowed
foreground service active
Bluetooth scanning constraints
Wi-Fi constraints
```

Rust DTN policy adapts.

Do not hide OS restrictions with busy loops.

---

# 135. iOS Restrictions

iOS proximity/background behavior is more constrained.

DTN architecture must degrade gracefully:

```text
foreground encounters
system-approved background opportunities
push-assisted Internet delivery
```

Do not assume Android-equivalent scanning.

---

# 136. Desktop DTN

Desktop can run:

```text
continuous LAN discovery
Bluetooth if supported
Iroh gateway
large relay store
```

Useful as:

```text
home relay
shelter node
community gateway
```

---

# 137. Headless DTN Node

A Linux/Raspberry Pi node can provide:

```text
BLE
Wi-Fi
Ethernet
Iroh
large store
power supply
```

This becomes extremely useful in disaster deployments.

---

# 138. Emergency Relay Node

Recommended profile:

```text
high relay quota
gateway preferred
critical reserve
authority alert support
continuous discovery
```

No messenger UI required.

---

# 139. Relay Node Identity

A relay can have:

```text
service/device identity
```

without being conversation participant.

This supports accountability without content access.

---

# 140. Self-Hosted Relay/Gateway

Part 11 will expand Internet relay infrastructure.

DTN gateway can hand off to:

```text
self-hosted Iroh relay
organization relay
public relay
```

according to policy.

---

# 141. Security Threats

Protect against:

```text
bundle flooding
storage exhaustion
priority abuse
replay
routing loops
fake gateway claims
malformed bundle
expired spam
metadata tracking
malicious relay dropping bundles
```

DTN cannot guarantee a relay will forward honestly.

Use redundancy when required.

---

# 142. Routing Loop Prevention

Use:

```text
BundleId dedup
hop limit
forward history
replication budget
```

Never rely on hop limit alone.

---

# 143. Forward History

Keep bounded:

```text
which peer received bundle
when
result
```

Avoid sending same bundle back and forth endlessly.

---

# 144. Peer Bounce Prevention

If:

```text
A gave bundle to B
```

then B should not immediately return same copy to A unless protocol explicitly requires reconciliation.

---

# 145. Malicious Drop

A relay can accept then drop.

No architecture can force untrusted relay cooperation.

Mitigations:

```text
multiple copies
trusted gateways
delivery acknowledgements
reputation locally
```

---

# 146. Malicious Modification

End-to-end integrity detects payload/header tampering.

Receiver rejects.

---

# 147. Malicious Fabrication

Origin signatures/authenticated application payloads prevent forged private messages/authority alerts.

Bundle transport itself does not create application trust.

---

# 148. Rate Limiting

Per encounter:

```text
max offers
max bytes
max critical claims
```

Per origin:

```text
bundle count
storage use
```

---

# 149. Unknown-Origin Traffic

Unknown peers may be limited to:

```text
small emergency reports
small relay bundles
```

depending on product policy.

---

# 150. Critical Traffic Abuse

Critical class should require:

```text
local user action
trusted authority
or strict anonymous emergency quota
```

Do not expose unlimited unauthenticated Critical traffic.

---

# 151. Anti-Correlation Measures

Possible:

```text
rotating discovery IDs
bundle batching
coarse timestamps
opaque destinations
```

But metadata privacy remains imperfect.

Document this honestly.

---

# 152. Storage Encryption

Relay store should persist ciphertext.

Local DTN metadata may also be encrypted at rest.

---

# 153. GC and Expiry

Periodic task:

```text
remove expired
remove delivered after grace
remove cancelled
apply quota eviction
```

Must be crash-safe.

---

# 154. Tombstones

Delivery/cancel tombstones may be retained temporarily so stale peers do not reintroduce completed bundles immediately.

---

# 155. Tombstone TTL

Tombstone lifetime should exceed typical encounter delay horizon.

Not permanent by default.

---

# 156. Reintroduced Bundle

If old peer presents a delivered bundle while tombstone exists:

```text
reject
send delivery summary if useful
```

---

# 157. Bundle Store Schema

Conceptual:

```text
bundles
bundle_payload_refs
bundle_forward_history
bundle_tombstones
peer_encounters
peer_utility
dtn_quotas
```

---

# 158. Bundle Table

Fields:

```text
bundle_id
state
priority
created_at
expires_at
hop_limit
copies_remaining
payload_type
payload_ref
origin
destination_token
size
```

---

# 159. Forward History Table

Fields:

```text
bundle_id
peer_id
forwarded_at
result
```

Bound history length.

---

# 160. Encounter Table

Store coarse:

```text
peer
last_seen
encounter_count
gateway_success
```

Avoid detailed long-term location history.

---

# 161. Performance

DTN is not a packet-forwarding hot path.

Prioritize:

```text
correctness
bounded storage
efficient set reconciliation
small metadata
battery efficiency
```

---

# 162. Inventory Performance

For 100k bundles, do not send 100k IDs.

Use compact summaries.

Benchmark:

```text
Bloom
IBLT later
Merkle later
```

---

# 163. Bundle Selection Complexity

Avoid O(all bundles × all peers) during every encounter.

Maintain indexes by:

```text
priority
expiry
destination class
size
```

---

# 164. Encounter Time Budget

If contact lasts seconds:

```text
selection must be fast
```

Precompute candidate queues where useful.

---

# 165. Metrics

Track:

```text
bundles stored
bundles forwarded
bundles delivered
average relay count
expired count
evicted count
bytes relayed
gateway success
encounter success
```

Keep private identities out of exported telemetry.

---

# 166. Diagnostics

Useful:

```text
DTN enabled
relay storage used
nearby peers
gateway seen
pending bundles
critical pending
last successful forward
```

Normal users should see simpler status.

---

# 167. UI State Mapping

Examples:

```text
Stored locally
Waiting for nearby device
Carried by 2 relay devices
Reached gateway
Delivered
Expired
```

Do not show:

```text
custody transfer complete
spray copy count 3
```

outside diagnostics.

---

# 168. Developer View

Advanced diagnostics may show:

```text
BundleId
priority
copies remaining
hop limit
next candidates
last peer
gateway score
```

---

# 169. Testing Layers

Unit:

```text
bundle validation
expiry
replication budget
dedup
eviction
```

Integration:

```text
A → B → C
gateway handoff
ACK return
```

Fault:

```text
peer disappears mid-transfer
duplicate bundle
crash
storage full
```

Security:

```text
fake priority
tampered bundle
replay
oversized metadata
```

---

# 170. Multi-Hop Test

Topology:

```text
A ↔ B ↔ C ↔ D
```

Only D has Internet.

Expected:

```text
A creates bundle
B stores
C stores
D forwards via Internet
destination ACK
ACK eventually returns
```

---

# 171. Partition Test

```text
Cluster 1: A-B-C
Cluster 2: D-E-F
```

Later C meets D.

Relevant bundles reconcile and continue.

---

# 172. Mobility Test

Random peers appear/disappear.

Assert:

```text
bounded copies
no infinite loop
eventual delivery when encounter path exists
```

---

# 173. Duplicate Route Test

Same message arrives via:

```text
Iroh
and DTN
```

Application records one logical message.

---

# 174. Expiry Test

Bundle expires while offline.

On next encounter:

```text
not forwarded
```

and eventually removed.

---

# 175. Storage Pressure Test

Fill relay store.

Ensure:

```text
bulk evicted before critical
critical reserve preserved
```

---

# 176. Battery Saver Test

Enable low-power mode.

Ensure:

```text
reduced scan
critical still forwarded
bulk delayed
```

---

# 177. Crash Test

Crash after:

```text
bundle persisted
before relay ACK
```

After restart:

```text
bundle still present
duplicate resend handled idempotently
```

---

# 178. Fuzz Targets

Fuzz:

```text
bundle header
inventory summary
ack summary
replication metadata
broadcast descriptor
```

All allocations bounded.

---

# 179. Property Tests

Invariants:

```text
copies never exceed origin max budget
expired bundle never forwarded
same BundleId stored once
hop count never increases
delivered tombstone prevents immediate resurrection
```

---

# 180. Simulated Network

Build deterministic simulator:

```text
nodes
encounters
transport capacities
battery states
gateway availability
```

This is critical for validating DTN algorithms.

---

# 181. Strategy Comparison Harness

Simulator can compare:

```text
DirectOnly
SprayAndWait
GatewayPreferred
EmergencyLimitedFlood
```

Metrics:

```text
delivery rate
delay
bytes transmitted
battery cost proxy
storage use
```

---

# 182. No ML Initially

Do not use ML for forwarding at first.

Deterministic algorithms are:

- explainable
- testable
- predictable
- safer

---

# 183. Public API

High-level:

```rust
let handle = dtn.enqueue(
    destination,
    payload_ref,
    DtnPolicy::default(),
).await?;
```

Applications should not manage peer encounter tables directly.

---

# 184. Bundle Handle

```rust
pub struct BundleHandle {
    pub id: BundleId,
}
```

Operations:

```text
status
cancel
priority
expiry
```

subject to policy.

---

# 185. DTN Events

```rust
pub enum DtnEvent {
    Stored(BundleId),
    Forwarded(BundleId, PeerId),
    RelayConfirmed(BundleId),
    GatewayReached(BundleId),
    DestinationReached(BundleId),
    Expired(BundleId),
    Evicted(BundleId),
}
```

---

# 186. Suggested Crate Structure

```text
crates/comm-dtn/
├── src/
│   ├── lib.rs
│   ├── bundle.rs
│   ├── store.rs
│   ├── state.rs
│   ├── policy.rs
│   ├── scheduler.rs
│   ├── forwarding.rs
│   ├── strategy.rs
│   ├── encounter.rs
│   ├── inventory.rs
│   ├── ack.rs
│   ├── gateway.rs
│   ├── quota.rs
│   ├── gc.rs
│   ├── diagnostics.rs
│   └── error.rs
│
├── tests/
└── Cargo.toml
```

Protocol crate if separated:

```text
comm-dtn-protocol/
```

---

# 187. Error Model

```rust
pub enum DtnError {
    StorageFull,
    QuotaExceeded,
    InvalidBundle,
    Expired,
    Unauthorized,
    Unsupported,
    Duplicate,
    Cancelled,
    Transport,
    Storage,
}
```

---

# 188. Initial Production Scope

Implement first:

```text
durable bundle store
BundleId dedup
expiry
hop limit
replication budget
Spray-and-Wait
BLE/Wi-Fi/Iroh encounter transport abstraction
inventory summary
relay ACK
destination ACK
gateway handoff
quotas
eviction
critical reserve
crash recovery
```

Defer initially:

```text
IBLT
social routing
predictive routing
complex custody transfer
large-scale epidemic mode
```

---

# 189. Implementation Phases

## Phase 1 — Bundle model

```text
BundleId
metadata
expiry
priority
policy
```

## Phase 2 — Durable store

```text
bundle table
payload refs
dedup
GC
```

## Phase 3 — Encounter protocol

```text
HELLO
inventory
request
transfer
relay ACK
```

## Phase 4 — Forwarding strategy

```text
DirectOnly
SprayAndWait
GatewayPreferred
```

## Phase 5 — Routing integration

```text
Part 03 handoff
DTN route plan
gateway path
```

## Phase 6 — File/blob integration

```text
manifest
thumbnail
chunk carriage
```

## Phase 7 — Emergency priority

```text
critical reserve
priority authorization
broadcast
```

## Phase 8 — Hardening

```text
simulation
fuzzing
crash injection
battery/storage tests
```

---

# 190. Definition of Done

Part 06 is complete when:

- a payload can be accepted with no end-to-end route
- accepted bundles are durably stored
- relays can carry ciphertext without content keys
- BundleId deduplicates multiple copies
- forwarding loops are bounded
- hop limit and replication budget are enforced
- expired bundles are never forwarded
- peer encounters reconcile inventories efficiently
- relay ACK is distinct from destination delivery
- destination ACK can propagate back asynchronously
- gateway nodes can bridge local transport to Internet/Iroh
- BLE/Wi-Fi/Iroh can all serve as encounter transports
- large files can use manifests/thumbnails/chunks rather than blind full replication
- storage quotas and critical reserve work
- emergency traffic can outrank bulk traffic
- unknown peers cannot exhaust storage without limits
- crash recovery preserves accepted relay bundles
- UI can distinguish carried vs delivered
- the subsystem works without Dioxus
- file-only/emergency/custom apps can reuse it
- simulation, fuzz, property, partition, and mobility tests exist

---

# 191. Relationship to Earlier Parts

Part 06 builds on:

```text
01 — Protocol Extension System
02 — Multi-Device Identity
03 — Transport & Routing Policy Engine
04 — Offline Event Log
05 — Robust File / Blob Subsystem
```

It directly supports:

```text
07 — Capability Negotiation Expansion
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
20 — Embedded Linux Node
23 — External Interoperability Suite
24 — Plugin / Module Ecosystem
```

---

# 192. Final Principle

The DTN subsystem should make this possible:

```text
Alice sends an SOS
while completely offline.

Her phone stores it.

Later she meets Bob over BLE.
Bob carries it.

Bob later meets Carol over Wi-Fi Direct.
Carol carries it.

Carol reaches Internet.
The bundle is delivered through Iroh.

A destination acknowledgement travels back later.

Alice eventually sees:
Delivered.
```

The original application payload remains the same logical object throughout.

No continuous route was ever required.

That is the defining capability of a true resilient, disaster-capable communication platform.
