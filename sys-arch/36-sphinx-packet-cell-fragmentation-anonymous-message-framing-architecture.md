# Core System Architecture Part 36 — Sphinx Packet, Cell, Fragmentation & Anonymous Message Framing Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 36  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:**  
- Part 34 — Mixnet, Loopix, Sphinx, Nym & High-Anonymity Transport Architecture  
- Part 35 — Anonymous Mailbox, Offline Receiving & Reply Capability Architecture  

**Primary purpose:** define the complete anonymous packet and message-framing substrate for SIAR, including Sphinx-style packet layering, normalized cells, fragmentation/reassembly, padding, versioning, reply-block embedding, replay protection, integrity boundaries, provider adaptation, parser hardening, and performance/resource limits.

---

# 1. Purpose

SIAR's high-anonymity transport plane needs a transport framing system that does not expose application payload shape directly.

Without normalization, an observer may correlate traffic using:

```text
message length
fragment count
packet timing
attachment size
message type
reply structure
```

even when payload contents are encrypted.

The governing principle is:

> **Anonymous transport packets must reveal as little application structure as practical, while remaining bounded, versioned, replay-safe, and provider-independent.**

---

# 2. Architectural Position

```text
Application Message
      │
      ▼
Application E2EE
      │
      ▼
Anonymous Message Framing
      │
      ├── message envelope
      ├── fragmentation
      ├── padding
      ├── cell normalization
      └── reassembly metadata
      │
      ▼
Provider Packet Layer
      │
      ├── Sphinx-style packet
      └── provider-specific encoding
      │
      ▼
Mixnet
```

---

# 3. Sphinx Is Below SIAR E2EE

Correct layering:

```text
SIAR E2EE payload
    ↓
anonymous framing
    ↓
Sphinx/provider packetization
```

Never:

```text
Sphinx replaces SIAR E2EE
```

---

# 4. Design Goals

The framing layer MUST support:

```text
fixed/normalized packet sizes
message fragmentation
message reassembly
bounded memory
versioning
corruption detection
duplicate suppression
reply capability embedding
provider adaptation
small-control-message efficiency
attachment chunking
padding
forward compatibility
fuzz-safe parsing
```

---

# 5. Non-Goals

The framing layer does not:

```text
define user identity
perform application E2EE
decide route policy
provide presence
store chat history
choose mix nodes directly
invent new cryptography
```

---

# 6. Packet vs Cell vs Message

Use precise terminology.

```text
Application Message
    user/domain object

Anonymous Envelope
    provider-neutral encrypted message container

Cell
    normalized framing unit

Sphinx Packet
    routed anonymous packet carrying one or more cell payload portions
```

---

# 7. Recommended Layering

```text
Message
  ↓
E2EE Envelope
  ↓
AnonymousMessageFrame
  ↓
Fragment(s)
  ↓
Cell(s)
  ↓
ProviderPacket
  ↓
Sphinx Packet
```

---

# 8. Anonymous Message Frame

```rust
pub struct AnonymousMessageFrame {
    pub version: AnonymousFrameVersion,
    pub class: AnonymousMessageClass,
    pub frame_id: AnonymousFrameId,
    pub expires_at: Timestamp,
    pub payload: Bytes,
}
```

---

# 9. Frame ID

```rust
pub struct AnonymousFrameId(pub [u8; 16]);
```

Must be:

```text
cryptographically random
non-sequential
non-user-derived
```

---

# 10. Frame ID Scope

Frame ID is transport-scoped.

Do not reuse:

```text
MessageId
ConversationId
AccountId
DeviceId
```

---

# 11. Message Classes

```rust
pub enum AnonymousMessageClass {
    Control,
    Text,
    Receipt,
    Bootstrap,
    MailboxControl,
    ReplyCapability,
    AttachmentChunk,
}
```

---

# 12. Message Class Leakage

If class is visible outside encrypted framing, it can leak metadata.

Preferred:

```text
class encrypted inside provider-protected body
```

---

# 13. Fixed-Size Cell Principle

Cells should use a small number of normalized sizes.

Example classes:

```rust
pub enum CellSizeClass {
    Small,
    Standard,
    Large,
}
```

---

# 14. Initial Recommendation

Prefer:

```text
one dominant standard cell size
```

for maximum anonymity.

Additional sizes can improve efficiency but weaken size indistinguishability.

---

# 15. Cell Size Tradeoff

Larger cells:

```text
less fragmentation
better throughput
more padding waste for tiny messages
```

Smaller cells:

```text
better efficiency for short control messages
more fragments for large payloads
```

---

# 16. Cell Payload Budget

Each cell must reserve bytes for:

```text
version
fragment header
authentication/integrity
provider overhead
padding
```

---

# 17. Cell Layout

Conceptual:

```text
+------------------------------+
| Cell Version                 |
| Frame/Fragment Metadata      |
| Encrypted Fragment Payload   |
| Padding                      |
| Integrity / Authentication   |
+------------------------------+
```

Exact cryptographic layout belongs to provider/established packet format.

---

# 18. Do Not Invent Sphinx Cryptography

Use audited/provider-compatible implementation.

SIAR owns only:

```text
framing semantics
bounds
mapping
integration
```

---

# 19. Fragmentation

If payload exceeds one cell:

```text
frame
→ fragment 0
→ fragment 1
→ fragment 2
...
```

---

# 20. Fragment Header

```rust
pub struct FragmentHeader {
    pub frame_id: AnonymousFrameId,
    pub index: u32,
    pub total: u32,
    pub payload_len: u32,
}
```

---

# 21. Fragment Count Bound

Hard limit.

Example concept:

```text
max fragments per frame
```

must be configurable and enforced before allocation.

---

# 22. Maximum Frame Size

```rust
pub struct FramingLimits {
    pub max_frame_bytes: u64,
    pub max_fragments: u32,
    pub max_inflight_reassemblies: u32,
    pub max_reassembly_bytes: u64,
}
```

---

# 23. Attachment Strategy

Large files should not become one gigantic anonymous frame.

Use:

```text
attachment object
→ chunk sequence
→ each chunk independently framed
```

---

# 24. Attachment Chunk Identity

Separate:

```rust
pub struct AnonymousAttachmentChunkId {
    pub transfer: TransferId,
    pub chunk_index: u32,
}
```

But provider-facing fields should not expose raw `TransferId`.

Use transport-scoped opaque IDs externally.

---

# 25. Reassembly

Recipient stores partial fragments in bounded staging.

---

# 26. Reassembly Key

```text
FrameId
+
version
```

---

# 27. Reassembly State

```rust
pub struct ReassemblyState {
    pub frame_id: AnonymousFrameId,
    pub expected_fragments: u32,
    pub received: BitSet,
    pub received_bytes: u64,
    pub expires_at: Timestamp,
}
```

---

# 28. Reassembly Completion

When all fragments present:

```text
validate bounds
→ reconstruct frame
→ validate integrity/authentication
→ pass to E2EE layer
```

---

# 29. Incomplete Reassembly

If expired:

```text
discard
```

---

# 30. Reassembly Timeout

Must be bounded.

---

# 31. Fragment Reordering

Expected and supported.

---

# 32. Duplicate Fragments

Ignore after authenticated dedup.

---

# 33. Conflicting Duplicate

If same fragment index has different authenticated contents:

```text
security anomaly
```

reject frame.

---

# 34. Corrupt Fragment

Drop.

Do not retain indefinitely.

---

# 35. Fragment Authentication Boundary

Each fragment/cell must be protected by provider-established integrity/authentication.

---

# 36. Application Integrity Boundary

Final payload still requires:

```text
SIAR E2EE authentication
```

---

# 37. Two Integrity Layers

```text
transport packet integrity
+
application E2EE integrity
```

Both are useful.

---

# 38. Padding

Padding hides exact payload length.

---

# 39. Padding Policy

```rust
pub enum PaddingPolicy {
    None,
    ToCellBoundary,
    FixedFrameClass,
    MaximumAnonymity,
}
```

---

# 40. Standard Anonymous Mode

Recommended:

```text
ToCellBoundary
```

---

# 41. Maximum Anonymity

Recommended:

```text
FixedFrameClass
+
normalized batching
```

where practical.

---

# 42. Padding Bytes

Generated cryptographically/randomly as required by framing/provider.

---

# 43. Do Not Encode Length in Padding Pattern

Hard rule.

---

# 44. Length Field

Only include minimal authenticated length metadata needed for reassembly.

---

# 45. Final Fragment Length

Can be encoded inside encrypted/authenticated frame metadata.

---

# 46. Size Bucketing

Potential:

```text
512 B
1 KiB
2 KiB
4 KiB
```

but fewer buckets improve anonymity.

---

# 47. Packet Normalization Policy

```rust
pub struct PacketNormalizationPolicy {
    pub cell_size: CellSizeClass,
    pub padding: PaddingPolicy,
    pub batch_class: BatchClass,
}
```

---

# 48. Batch Class

```rust
pub enum BatchClass {
    Immediate,
    ShortDelay,
    PrivacyBatch,
}
```

---

# 49. Batch Timing

Framing layer can tag desired class.

Actual delay scheduling belongs to anonymity transport/provider.

---

# 50. No Content-Based Timing Leakage

Avoid:

```text
attachments always delayed
text always immediate
```

if privacy mode aims to hide class.

---

# 51. Message Type Hiding

Maximum anonymity may map multiple application classes to same cell/framing profile.

---

# 52. Control Messages

Tiny control messages should still use normalized cells.

---

# 53. Receipts

Avoid unique tiny packet signatures.

---

# 54. Reply Capabilities

Reply capability material may be embedded inside encrypted anonymous frame.

---

# 55. Reply Descriptor Container

```rust
pub struct ReplyCapabilityEnvelope {
    pub version: u16,
    pub capability: SecretBytes,
    pub expires_at: Timestamp,
}
```

---

# 56. Reply Capability Secrecy

Never expose in:

```text
logs
debug dumps
support bundles
```

---

# 57. SURB Embedding

Provider adapter maps:

```text
ReplyCapabilityEnvelope
→ provider SURB/reply-block format
```

---

# 58. Provider-Neutral Core

Core should not know:

```text
Nym SURB binary layout
```

---

# 59. Provider Packet Adapter

```rust
pub trait AnonymousPacketAdapter {
    type ProviderPacket;

    fn encode_cell(
        &self,
        cell: AnonymousCell,
        route: &AnonymousRouteContext,
    ) -> Result<Self::ProviderPacket, PacketError>;

    fn decode_cell(
        &self,
        packet: Self::ProviderPacket,
    ) -> Result<AnonymousCell, PacketError>;
}
```

---

# 60. Anonymous Cell

```rust
pub struct AnonymousCell {
    pub version: CellVersion,
    pub frame_id: AnonymousFrameId,
    pub fragment_index: u32,
    pub fragment_total: u32,
    pub payload: Bytes,
}
```

---

# 61. Cell Version

```rust
pub struct CellVersion(pub u16);
```

---

# 62. Version Strategy

Support explicit:

```text
current
previous compatible
future unsupported
```

---

# 63. Version Negotiation

Do not negotiate per packet in a linkable way if avoidable.

Use capability negotiation via established secure channel.

---

# 64. Unknown Version

Reject safely.

---

# 65. Forward Compatibility

Optional extension area can be authenticated and length-prefixed.

---

# 66. Extension Budget

Bound extension bytes.

---

# 67. Parser Design

Parsers must be:

```text
total-length checked
allocation-bounded
integer-overflow safe
duplicate-field safe
version aware
```

---

# 68. No Unbounded Vec Allocation from Network Length

Hard rule.

---

# 69. Integer Safety

Use checked conversions.

---

# 70. Malformed Fragment Count

Reject before allocation.

---

# 71. Oversized Payload Length

Reject.

---

# 72. Zero Fragment Total

Reject.

---

# 73. Fragment Index >= Total

Reject.

---

# 74. Frame Expiry

Reject expired frame early where safe.

---

# 75. Replay Resistance

Framing layer should support transport replay detection.

---

# 76. Replay Keys

Potential:

```text
packet tag
frame ID
fragment ID
provider replay tag
```

---

# 77. Provider Replay Protection

Prefer provider/Sphinx implementation where available.

---

# 78. SIAR Reassembly Replay Cache

Still keep bounded local cache.

---

# 79. Replay Cache Entry

```rust
pub struct ReplayKey {
    pub frame_id: AnonymousFrameId,
    pub fragment_index: u32,
}
```

---

# 80. Replay Cache TTL

At least as long as valid transport lifetime.

---

# 81. Frame Lifetime

```rust
pub struct AnonymousFramePolicy {
    pub max_lifetime: Duration,
}
```

---

# 82. Expiry Hiding

If expiry metadata must be provider-visible, use coarse buckets.

Prefer encrypted expiry.

---

# 83. Timestamp Precision

Avoid exact user-action timing where possible.

---

# 84. Message Batching

Multiple small E2EE messages could optionally be packed into one anonymous frame.

---

# 85. Batch Envelope

```rust
pub struct AnonymousBatchFrame {
    pub version: u16,
    pub items: Vec<AnonymousBatchItem>,
}
```

---

# 86. Batch Limits

Bound:

```text
item count
total bytes
lifetime spread
```

---

# 87. Batching Tradeoff

Benefits:

```text
less overhead
better traffic normalization
```

Costs:

```text
delivery coupling
latency
larger loss impact
```

---

# 88. Initial Recommendation

Do not batch unrelated conversations in v1.

---

# 89. Conversation Correlation

Batching unrelated recipients can complicate routing/privacy semantics.

---

# 90. Same-Recipient Batch

Safer initial option.

---

# 91. Control/Data Separation

Avoid exposing class via distinct packet sizes.

Use same normalized cell where possible.

---

# 92. Provider MTU

Different providers may have different packet capacity.

---

# 93. Effective Cell Capacity

```text
provider packet size
-
routing header
-
crypto overhead
-
SIAR framing overhead
=
cell payload capacity
```

---

# 94. MTU Calculator

```rust
pub trait PacketCapacityCalculator {
    fn payload_capacity(
        &self,
        profile: &AnonymousPacketProfile,
    ) -> Result<usize, PacketError>;
}
```

---

# 95. No Magic Constants Scattered in Code

Centralize packet sizing.

---

# 96. Provider Packet Profile

```rust
pub struct AnonymousPacketProfile {
    pub provider: AnonymousProviderId,
    pub size_class: CellSizeClass,
    pub version: CellVersion,
}
```

---

# 97. Fragmenter

```rust
pub trait AnonymousFragmenter {
    fn fragment(
        &self,
        frame: &AnonymousMessageFrame,
        profile: &AnonymousPacketProfile,
    ) -> Result<Vec<AnonymousCell>, FramingError>;
}
```

---

# 98. Streaming Fragmentation

For large data, avoid collecting all fragments in memory.

Prefer iterator/stream:

```rust
pub trait StreamingFragmenter {
    type Stream: Stream<Item = Result<AnonymousCell, FramingError>>;

    fn fragment_stream(
        &self,
        source: PayloadSource,
        profile: AnonymousPacketProfile,
    ) -> Self::Stream;
}
```

---

# 99. Streaming Reassembly

Likewise support bounded incremental reassembly.

---

# 100. Attachment Chunking Before Framing

Recommended:

```text
file
→ encrypted attachment chunks
→ anonymous frames
→ cells
```

---

# 101. Why Two Chunk Layers

Attachment chunking solves:

```text
resume
integrity
storage
```

Cell fragmentation solves:

```text
transport packet size
```

Keep them separate.

---

# 102. Do Not Couple File Chunk Size to Sphinx Cell Size

Hard rule.

---

# 103. Reassembly Storage

Small frames:

```text
memory
```

Large frames:

```text
bounded temp storage
```

---

# 104. Threshold

Benchmark-defined.

---

# 105. Temp Storage Encryption

Reassembly temp files must remain encrypted-at-rest.

---

# 106. Reassembly Garbage Collection

Periodic cleanup:

```text
expired
invalid
orphaned
```

---

# 107. Fragment Storm Defense

Limit per source capability/mailbox.

---

# 108. Incomplete Frame Quota

```rust
pub struct ReassemblyQuota {
    pub max_frames: u32,
    pub max_bytes: u64,
    pub max_fragments_per_frame: u32,
}
```

---

# 109. Eviction Policy

Prefer:

```text
oldest expired
then oldest inactive
```

Never evict active high-priority frame blindly if avoidable.

---

# 110. Priority

Possible:

```rust
pub enum AnonymousFramePriority {
    Control,
    Message,
    Attachment,
    Background,
}
```

---

# 111. Priority Visibility

Priority should not become externally distinguishable if anonymity profile forbids it.

---

# 112. Internal Scheduling Only

Use priority locally.

---

# 113. Error Taxonomy

```rust
pub enum FramingError {
    UnsupportedVersion,
    InvalidHeader,
    InvalidFragmentCount,
    FragmentTooLarge,
    FrameTooLarge,
    ReassemblyLimitExceeded,
    DuplicateConflict,
    IntegrityFailure,
    Expired,
    ProviderCapacityTooSmall,
    Internal,
}
```

---

# 114. Corruption Handling

Corrupt packet:

```text
drop
record redacted metric
do not crash
```

---

# 115. Reassembly Failure UX

Normally hidden.

Application may see:

```text
anonymous delivery failed
```

only after retries/expiry.

---

# 116. Provider Error Separation

Framing errors are distinct from:

```text
route error
mailbox error
provider unavailable
```

---

# 117. Message Delivery State

Framing stage can expose:

```text
Preparing
Packetizing
Queued
```

but ordinary UX should simplify.

---

# 118. Observability

Privacy-safe metrics:

```text
frames created
cells created
average fragments bucket
reassembly success
reassembly timeout
invalid packet count
padding overhead bucket
```

---

# 119. Forbidden Metrics

Do not record:

```text
exact per-contact frame sizes
exact per-message timing
frame ID mapped to AccountId
```

---

# 120. Padding Overhead Metric

Aggregate only.

---

# 121. Diagnostics

Developer view may show:

```text
cell size class
provider profile
reassembly queue depth
invalid packet count
```

---

# 122. Do Not Show Raw Sphinx Header

No operational need for normal diagnostics.

---

# 123. Debug Dumps

Must redact secret/path material.

---

# 124. Testkit

Add deterministic:

```text
framer
fragmenter
reassembler
fake provider capacity
corruption injector
```

---

# 125. Unit Tests

Test:

```text
single-cell frame
multi-cell frame
exact boundary
one byte over boundary
max frame
empty payload
```

---

# 126. Fragment Order Tests

```text
in order
reverse
random
duplicate
missing
```

---

# 127. Corruption Tests

```text
bad version
bad count
bad index
bad length
tampered payload
```

---

# 128. Limit Tests

```text
max fragments
max bytes
max inflight frames
```

---

# 129. Replay Tests

Same cell delivered repeatedly.

---

# 130. Duplicate Conflict Test

Same:

```text
FrameId + index
```

different contents.

Must reject frame.

---

# 131. Expiry Tests

```text
before expiry
at expiry
after expiry
```

---

# 132. Version Tests

```text
current
previous compatible
future unknown
```

---

# 133. Provider Capacity Tests

Different provider MTUs.

---

# 134. Padding Tests

Ensure:

```text
same size class
```

for payloads within bucket.

---

# 135. Length-Hiding Tests

Verify exact original payload length not exposed outside encrypted length metadata.

---

# 136. Fuzzing

Fuzz:

```text
cell parser
fragment parser
batch parser
extension parser
reassembly state machine
```

---

# 137. Property Tests

Properties:

```text
reassemble(fragment(frame)) == frame
duplicate identical fragment is idempotent
fragment count bound always enforced
malformed length never allocates beyond configured limit
```

---

# 138. Differential Tests

Where provider library exposes reference packet encoder/decoder:

```text
compare behavior
```

without reimplementing cryptography.

---

# 139. Memory Tests

Malformed traffic cannot cause unbounded growth.

---

# 140. CPU Exhaustion Tests

Large invalid fragment storms stay bounded.

---

# 141. Concurrency Tests

Multiple frames reassembling concurrently.

---

# 142. Crash Tests

Crash during:

```text
fragment write
reassembly
temp-file flush
completion
```

---

# 143. Restart Recovery

Incomplete reassembly may:

```text
resume
or discard
```

according to policy.

---

# 144. Recommended Initial Policy

For small message frames:

```text
discard incomplete on restart
```

Sender/mailbox can retry.

For large attachment chunk frames:

```text
resume if safely persisted
```

---

# 145. Security Invariants

Mandatory:

```text
1. Frame IDs are random and transport-scoped.
2. MessageId/ConversationId/AccountId are never exposed as frame IDs.
3. Fragment count and lengths are bounded before allocation.
4. Duplicate identical fragments are idempotent.
5. Conflicting duplicates invalidate the frame.
6. SIAR E2EE remains above anonymous framing.
7. Provider/Sphinx cryptography is not reimplemented ad hoc.
8. Reply capability material is never logged.
9. Large attachments are chunked separately from cell fragmentation.
10. Unknown versions fail safely.
11. Reassembly memory/storage is bounded.
12. Padding policy cannot silently weaken Maximum Anonymity mode.
```

---

# 146. Crate Layout

Recommended:

```text
crates/
├── siar-anonymity-frame/
├── siar-anonymity-cell/
├── siar-anonymity-fragment/
├── siar-anonymity-reassembly/
├── siar-anonymity-padding/
├── siar-anonymity-packet-profile/
└── siar-anonymity-framing-testkit/
```

---

# 147. `siar-anonymity-frame`

Owns:

```text
AnonymousMessageFrame
frame version
message class
batch envelope
```

---

# 148. `siar-anonymity-cell`

Owns:

```text
AnonymousCell
CellVersion
size classes
serialization
```

---

# 149. `siar-anonymity-fragment`

Owns:

```text
fragment headers
fragmenter
streaming fragmentation
```

---

# 150. `siar-anonymity-reassembly`

Owns:

```text
inflight state
dedup
expiry
quota
completion
```

---

# 151. `siar-anonymity-padding`

Owns:

```text
padding policy
size normalization
```

---

# 152. `siar-anonymity-packet-profile`

Owns:

```text
provider capacity
overhead calculation
cell profile
```

---

# 153. Dependency Direction

```text
message/E2EE
    ↓
frame
    ↓
fragment/cell
    ↓
provider adapter
```

Never reverse.

---

# 154. Serialization Format

Internal provider-neutral framing may use:

```text
postcard
```

if bounded and explicitly versioned.

---

# 155. RON

Only for human-readable configuration/testing fixtures.

---

# 156. JSON

Not used in packet framing.

---

# 157. Zero-Copy

Use zero-copy/borrowed parsing only when safe and beneficial.

---

# 158. Parser Safety Over Micro-Optimization

Prefer:

```text
clear bounds
checked offsets
small copies
```

over unsafe parsing tricks.

---

# 159. `Bytes`

Suitable for immutable payload slices.

---

# 160. Secret Types

Reply capability buffers use:

```text
zeroizing secret wrapper
```

where practical.

---

# 161. Packet Builder API

```rust
pub trait AnonymousPacketFramer {
    fn frame(
        &self,
        input: FramingInput,
        profile: AnonymousPacketProfile,
    ) -> Result<FramedAnonymousMessage, FramingError>;
}
```

---

# 162. Framing Input

```rust
pub struct FramingInput {
    pub class: AnonymousMessageClass,
    pub payload: Bytes,
    pub expiry: Timestamp,
    pub reply: Option<ReplyCapabilityEnvelope>,
}
```

---

# 163. Framed Output

```rust
pub struct FramedAnonymousMessage {
    pub frame_id: AnonymousFrameId,
    pub cells: Vec<AnonymousCell>,
}
```

For large payloads use streaming API.

---

# 164. Reassembler API

```rust
pub trait AnonymousReassembler {
    fn ingest(
        &mut self,
        cell: AnonymousCell,
    ) -> Result<ReassemblyOutcome, FramingError>;
}
```

---

# 165. Reassembly Outcome

```rust
pub enum ReassemblyOutcome {
    Incomplete,
    Complete(AnonymousMessageFrame),
    Duplicate,
    Rejected,
}
```

---

# 166. Garbage Collection API

```rust
pub trait ReassemblyGc {
    fn collect_expired(
        &mut self,
        now: Timestamp,
    ) -> ReassemblyGcReport;
}
```

---

# 167. Performance Budget

Track:

```text
framing throughput
cells/sec
reassembly latency
memory per inflight frame
padding overhead
```

---

# 168. Small Message Fast Path

A single standard cell should handle common text/control messages where possible.

---

# 169. Fast Path Must Not Change Privacy Semantics

No special visibly smaller packet.

---

# 170. Attachment Slow Path

Streaming fragmenter.

---

# 171. Backpressure

If provider queue is full:

```text
pause fragment production
```

---

# 172. No Pre-Framing Entire Huge File

Hard rule.

---

# 173. Flow Control

```text
source stream
→ fragmenter
→ bounded cell queue
→ provider
```

---

# 174. Bounded Channel

Required between fragmenter and provider.

---

# 175. Cancellation

If send cancelled:

```text
stop generating future cells
```

Already-submitted cells may still arrive.

---

# 176. Cancellation Semantics

Application MessageId still determines final message visibility/state.

---

# 177. Partial Attachment

Recipient must not expose incomplete attachment as complete.

---

# 178. Verification

Attachment layer performs final content integrity verification after all chunks.

---

# 179. Batch Interaction

Do not mix fragments from unrelated frames into one SIAR batch unless provider semantics explicitly support and privacy review approves.

---

# 180. Cover Traffic Interaction

Cover cells must be indistinguishable at provider-visible framing layer where provider design supports.

---

# 181. Dummy Cell

```rust
pub enum CellPayloadKind {
    Real,
    Cover,
}
```

This distinction should remain local/internal and not provider-visible if possible.

---

# 182. Cover Cell Parsing

Recipient/provider runtime can discard dummy cells according to protocol.

---

# 183. Loop Traffic

May use same cell size/profile.

---

# 184. Traffic Analysis Resistance

Framing contributes by normalizing:

```text
size
structure
fragmentation profile
```

but cannot alone hide:

```text
timing
online/offline pattern
volume
```

Hence Part 34 cover/delay policy remains essential.

---

# 185. Provider Compatibility

Different provider implementations may not use identical packet internals.

SIAR core should guarantee:

```text
same application framing semantics
```

while adapter maps to provider packet model.

---

# 186. Nym Adapter

Responsibilities:

```text
map SIAR cell/profile
to Nym packet API
map reply capabilities
respect Nym packet capacity
surface provider errors
```

---

# 187. Native Mixnet Future

Could use:

```text
Sphinx packets directly
```

but still implement same SIAR framing API.

---

# 188. Migration

Switching provider must not require changing application MessageId or E2EE payload semantics.

---

# 189. Wire Compatibility

Framing version should be independent from:

```text
provider SDK version
```

where possible.

---

# 190. Upgrade Strategy

```text
support N and N-1
publish capability
roll out
retire old
```

---

# 191. Downgrade Attack

Never accept older framing version solely because attacker requests it.

Version compatibility is policy-controlled.

---

# 192. Capability Advertisement

Authenticated.

---

# 193. Unknown Extension

Ignore only if:

```text
marked optional
bounded
authenticated
```

---

# 194. Mandatory Extension

Unknown mandatory extension:

```text
reject frame
```

---

# 195. Security Events

Repeated malformed authenticated frames may generate local security event.

---

# 196. Abuse Threshold

Avoid one event per packet.

Coalesce.

---

# 197. Metrics Cardinality

Low.

No frame ID labels in telemetry.

---

# 198. Support Bundle

Include only aggregate:

```text
framing version
cell size class
invalid packet count
reassembly failures
```

---

# 199. UI Exposure

Normal user should not see:

```text
Sphinx
fragment index
cell size
```

except advanced documentation/diagnostics.

---

# 200. Developer UX

Part 20 can show:

```text
Anonymous packet profile: Standard
Framing version: v1
Reassembly queue: Healthy
```

---

# 201. Test Fixtures

Fixture examples:

```text
single_text_frame
multi_fragment_text
reply_capability_frame
small_attachment
expired_frame
corrupt_fragment
```

---

# 202. Golden Binary Fixtures

Useful for stable framing compatibility.

Store synthetic data only.

---

# 203. Compatibility Tests

Decode old supported fixtures in every release.

---

# 204. Round-Trip Tests

```text
encode
→ fragment
→ shuffle
→ reassemble
→ decode
```

---

# 205. End-to-End Integration Test

```text
SIAR E2EE message
→ framing
→ fake Sphinx provider
→ mailbox
→ reassembly
→ E2EE decrypt
```

---

# 206. Adversarial Test

Inject:

```text
truncation
bit flip
duplicate
reorder
oversize
invalid version
expired packet
```

---

# 207. Property: Bounded Allocation

For any input <= parser max:

```text
allocation <= configured bound
```

---

# 208. Property: No ID Leakage

Serialized provider-facing frame must not contain raw:

```text
AccountId
DeviceId
ConversationId
MessageId
```

unless explicitly encrypted within payload.

---

# 209. Property: Padding

Payloads in same configured bucket produce same outer cell length.

---

# 210. Property: Stable Parsing

Malformed input never panics.

---

# 211. Property: Deterministic Reassembly

Same authenticated fragment set yields same frame.

---

# 212. Initial Production Scope

Implement first:

```text
one standard cell size
versioned anonymous frame
bounded fragment header
streaming fragmentation
bounded reassembly
padding to cell boundary
random frame IDs
provider capacity calculation
Nym adapter mapping
reply capability container
replay cache
fuzz/property tests
```

Then add:

```text
multiple size classes
privacy batching
advanced fixed frame classes
conversation-specific profiles
native Sphinx packet provider
```

---

# 213. Definition of Done

Part 36 is complete when:

- packet/message terminology is explicit
- SIAR E2EE remains above anonymous framing
- normalized fixed-size cells are defined
- frame IDs are random and unrelated to application IDs
- fragmentation and reassembly are bounded
- attachment chunking is separate from transport fragmentation
- duplicate, reorder, missing, corrupt, and conflicting fragments are handled
- padding and size-bucketing policies are defined
- reply capability/SURB material can be embedded securely
- provider packet capacity is centrally calculated
- provider-specific Sphinx/Nym packet details remain behind adapters
- parsing is bounded and overflow-safe
- replay cache and expiry rules are defined
- streaming APIs avoid loading huge payloads in memory
- diagnostics/metrics avoid peer correlation
- fuzzing, property tests, binary compatibility fixtures, and adversarial tests are defined
- no new cryptographic construction is introduced casually

---

# 214. Final Architecture

```text
                 APPLICATION MESSAGE
                         │
                         ▼
                   SIAR APPLICATION E2EE
                         │
                         ▼
                 Anonymous Message Frame
                         │
                         ▼
                    Fragmentation
                         │
                         ▼
                  Normalized Cells
                         │
                         ▼
              Provider Packet Adapter
                         │
                         ▼
                 Sphinx / Nym Packet
                         │
                         ▼
                      Mixnet
```

Receive path:

```text
Provider Packet
      ↓
Decode Cell
      ↓
Replay/Dedup Check
      ↓
Bounded Reassembly
      ↓
Anonymous Frame
      ↓
SIAR E2EE Validation/Decrypt
      ↓
Application Message
```

---

# 215. Final Principle

SIAR should hide message structure before traffic enters the anonymity network.

The correct model is:

```text
E2EE
+
random transport frame identity
+
normalized cells
+
bounded fragmentation
+
padding
+
provider-adapted Sphinx packets
```

not:

```text
encrypt the original message
→ send its natural size directly
```

This architecture gives SIAR a stable, provider-neutral anonymous packet substrate that can work with Nym today and a future native Sphinx/Loopix-inspired mixnet later, without coupling application semantics to one provider or exposing raw message structure to the network.
