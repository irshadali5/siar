# Core System Architecture Part 40 — Anonymous Attachment Transfer, Rendezvous, Large-File Privacy & High-Bandwidth Anonymous Data Architecture

## SIAR — Secure, Local-First, Multi-Transport Communication Platform

**Series:** Core System Architecture  
**Part:** 40  
**Status:** Architecture specification  
**Primary language:** Rust  
**Builds on:**  
- Part 34 — Mixnet, Loopix, Sphinx, Nym & High-Anonymity Transport Architecture  
- Part 35 — Anonymous Mailbox, Offline Receiving & Reply Capability Architecture  
- Part 36 — Sphinx Packet, Cell, Fragmentation & Anonymous Message Framing Architecture  
- Part 37 — Cover Traffic, Traffic Shaping, Timing Obfuscation & Loop Traffic Architecture  
- Part 38 — Native Loopix-Inspired Mixnet Topology, Mix Nodes, Layering & Packet Forwarding Architecture  
- Part 39 — Mixnet Directory, Node Admission, Identity, Sybil Resistance & Topology Governance Architecture  

**Primary purpose:** define the complete privacy-preserving large-object transport architecture for SIAR, including anonymous attachment manifests, capability-based rendezvous, mixnet-assisted coordination, encrypted chunk transfer, anonymous high-bandwidth fetch, resumability, multi-source transfer, cover-aware scheduling, traffic-analysis resistance, abuse controls, storage lifecycle, and provider-neutral Rust interfaces.

---

# 1. Purpose

Small messages and small files can be sent entirely through the mixnet.

Large files create a different problem.

Examples:

```text
photos
videos
documents
archives
voice recordings
backups
software packages
large media
```

Sending gigabytes entirely as fixed-size Sphinx cells is usually:

```text
slow
expensive
battery-heavy
bandwidth-heavy
harmful to network capacity
```

But simply switching to:

```text
direct QUIC
```

would reveal network metadata and destroy the anonymity guarantee.

The governing principle is:

> **Large anonymous data transfers must decouple message anonymity from bulk-data transport while preserving endpoint hiding, capability security, encryption, resumability, and strict no-silent-downgrade guarantees.**

---

# 2. Architectural Position

```text
Message / Attachment Reference
          │
          ▼
Application E2EE
          │
          ▼
Anonymous Attachment Manifest
          │
          ▼
Mixnet Signaling / Rendezvous
          │
          ▼
Anonymous Bulk Data Transport
          │
   ┌──────┼────────┐
   │      │        │
Relay   Rendezvous Provider   Mixnet-only
   │      │        │
   └──────┴────────┘
          │
          ▼
Encrypted Chunk Retrieval
          │
          ▼
Verification / Reassembly
```

---

# 3. Core Separation

Keep two distinct planes:

```text
Control Plane
    attachment offer
    manifest
    authorization capability
    rendezvous information
    acknowledgements

Data Plane
    encrypted bulk chunks
```

---

# 4. Why Separation Matters

This allows SIAR to use:

```text
mixnet for privacy-sensitive signaling
```

while selecting a more scalable but still anonymous/privacy-preserving path for:

```text
large encrypted payloads
```

---

# 5. Attachment Security Layers

```text
Original File
    ↓
Per-Attachment Encryption
    ↓
Chunking
    ↓
Chunk Integrity
    ↓
Anonymous Bulk Transport
```

---

# 6. E2EE Is Still Required

Anonymous transport protects:

```text
metadata
network identities
route relationships
```

Attachment encryption protects:

```text
file contents
```

Use both.

---

# 7. Attachment Object Model

```rust
pub struct AnonymousAttachmentDescriptor {
    pub attachment_id: AttachmentId,
    pub manifest_id: AnonymousManifestId,
    pub size: u64,
    pub media_type: AttachmentMediaType,
    pub chunking: ChunkingProfile,
    pub transfer_policy: AnonymousAttachmentPolicy,
}
```

---

# 8. Attachment ID

Application-local identifier.

Do not expose directly to providers.

---

# 9. Manifest ID

Transport-scoped random identifier:

```rust
pub struct AnonymousManifestId(pub [u8; 16]);
```

---

# 10. Manifest Contents

```rust
pub struct AnonymousAttachmentManifest {
    pub version: ManifestVersion,
    pub manifest_id: AnonymousManifestId,
    pub encrypted_name: Option<Bytes>,
    pub total_size: u64,
    pub chunk_count: u32,
    pub chunk_hashes: Vec<ChunkDigest>,
    pub content_key_ref: AttachmentKeyReference,
    pub transport: AnonymousBulkTransportDescriptor,
    pub expiry: Timestamp,
}
```

---

# 11. Manifest Privacy

Sensitive metadata should be encrypted.

Avoid exposing:

```text
filename
exact MIME type
contact identity
conversation ID
```

to bulk transport provider.

---

# 12. Size Leakage

Exact file size itself can leak information.

---

# 13. Size Hiding

Potential strategies:

```text
padding
bucketed size classes
dummy trailing chunks
fixed chunk sizes
```

---

# 14. Size Class

```rust
pub enum AnonymousAttachmentSizeClass {
    Small,
    Medium,
    Large,
    VeryLarge,
}
```

---

# 15. Size Padding Policy

```rust
pub enum AttachmentPaddingPolicy {
    None,
    ChunkBoundary,
    SizeBucket,
    MaximumPrivacy,
}
```

---

# 16. Maximum Privacy

Can pad to:

```text
coarse size bucket
```

at bandwidth cost.

---

# 17. Chunking

Attachment layer chunking is independent from Part 36 cell fragmentation.

---

# 18. Chunking Profile

```rust
pub struct ChunkingProfile {
    pub chunk_size: u32,
    pub padded_chunk_size: u32,
    pub hashing: ChunkHashAlgorithm,
}
```

---

# 19. Chunk Size

Choose based on:

```text
transport
latency
memory
resume efficiency
bandwidth overhead
```

---

# 20. Initial Recommendation

Use:

```text
256 KiB–1 MiB
```

range subject to benchmarking.

---

# 21. Chunk Encryption

Each chunk must be encrypted/authenticated.

---

# 22. Per-Attachment Key

```rust
pub struct AttachmentContentKey(SecretBytes);
```

---

# 23. Key Derivation

Derive per-chunk nonce/key material safely.

Do not reuse nonce.

---

# 24. Key Distribution

Attachment content key is delivered inside:

```text
SIAR E2EE message
```

not through public bulk transport metadata.

---

# 25. Chunk Digest

Digest covers encrypted or plaintext representation according to design.

Preferred:

```text
integrity verified after decryption
+
authenticated encryption tag
```

---

# 26. Merkle Tree

For large attachments, manifest can use:

```text
Merkle root
```

instead of storing every chunk hash.

---

# 27. Merkle Root

```rust
pub struct AttachmentMerkleRoot(pub [u8; 32]);
```

---

# 28. Why Merkle Tree

Supports:

```text
chunk verification
partial fetch
multi-source
compact manifest
```

---

# 29. Attachment Offer Flow

```text
Sender prepares file
→ encrypts/chunks
→ uploads or stages anonymous bulk data
→ creates manifest
→ sends manifest/capability via mixnet
→ recipient fetches chunks anonymously
```

---

# 30. Rendezvous Concept

A rendezvous layer coordinates sender and receiver without revealing direct endpoint information.

---

# 31. Rendezvous Types

```rust
pub enum AnonymousRendezvousMode {
    StoreAndFetch,
    RelayStream,
    CapabilityObjectStore,
    MixnetOnly,
}
```

---

# 32. StoreAndFetch

Sender uploads encrypted chunks to privacy-preserving storage.

Recipient later fetches them.

---

# 33. RelayStream

Both sides connect to intermediary.

Provider forwards encrypted bytes.

---

# 34. Capability Object Store

Recipient gets unguessable capability URL/token through E2EE.

---

# 35. MixnetOnly

Entire attachment goes through mixnet.

Best privacy, worst efficiency.

---

# 36. Transport Policy

```rust
pub enum AnonymousAttachmentPolicy {
    MixnetOnly,
    AnonymousRendezvous,
    PrivateRelay,
    AskBeforeDowngrade,
    DisallowLargeTransfer,
}
```

---

# 37. Strict No-Downgrade

Maximum Anonymity must never silently use:

```text
direct peer QUIC
normal relay
public cloud URL
```

unless explicitly allowed by policy.

---

# 38. Bulk Transport Capability

```rust
pub struct AnonymousBulkTransportCapabilities {
    pub hides_sender_ip: bool,
    pub hides_receiver_ip: bool,
    pub supports_resume: bool,
    pub supports_multi_source: bool,
    pub supports_range_fetch: bool,
    pub supports_padding: bool,
}
```

---

# 39. Eligibility Rule

Strict anonymous transfer requires:

```text
hides_sender_ip = true
hides_receiver_ip = true
```

or equivalent privacy guarantee.

---

# 40. Rendezvous Capability

```rust
pub struct AnonymousFetchCapability {
    pub object_id: AnonymousObjectId,
    pub secret: SecretBytes,
    pub expires_at: Timestamp,
    pub max_downloads: u32,
}
```

---

# 41. Capability Properties

Must be:

```text
unguessable
short-lived
scoped
revocable where possible
rate-limited
```

---

# 42. Capability Is Bearer Secret

Anyone possessing it may fetch.

Treat like secret.

---

# 43. Never Log Capability

Hard rule.

---

# 44. Object ID

Random provider-facing identifier.

---

# 45. No Filename in Provider Path

Hard rule.

---

# 46. Anonymous Object Store

Provider stores:

```text
encrypted chunks
opaque object IDs
expiry
quota metadata
```

---

# 47. Provider Must Not Know Content

Chunks are already encrypted.

---

# 48. Provider Must Not Know Recipient Identity

Recipient fetches via capability and anonymous/privacy-preserving transport.

---

# 49. Sender Upload Privacy

Sender upload itself must hide sender network identity.

---

# 50. Upload Paths

Potential:

```text
mixnet-assisted upload
anonymous relay
Tor-like proxy
Nym-compatible bulk path
SIAR anonymous gateway
```

depending provider.

---

# 51. Recipient Fetch Privacy

Recipient similarly fetches through approved anonymous path.

---

# 52. Rendezvous Provider Abstraction

```rust
#[async_trait]
pub trait AnonymousRendezvousProvider: Send + Sync {
    async fn create_object(
        &self,
        policy: AnonymousObjectPolicy,
    ) -> Result<AnonymousUploadSession, RendezvousError>;

    async fn upload_chunk(
        &self,
        session: &AnonymousUploadSession,
        chunk: EncryptedChunk,
    ) -> Result<ChunkUploadReceipt, RendezvousError>;

    async fn finalize(
        &self,
        session: &AnonymousUploadSession,
    ) -> Result<AnonymousFetchCapability, RendezvousError>;

    async fn fetch_chunk(
        &self,
        capability: &AnonymousFetchCapability,
        index: u32,
    ) -> Result<EncryptedChunk, RendezvousError>;
}
```

---

# 53. Upload Session

```rust
pub struct AnonymousUploadSession {
    pub object_id: AnonymousObjectId,
    pub provider: AnonymousProviderId,
    pub expires_at: Timestamp,
}
```

---

# 54. Upload Session Secrecy

Do not expose to UI/plugins.

---

# 55. Finalization

Only after all required chunks uploaded.

---

# 56. Incomplete Upload

Provider garbage-collects after TTL.

---

# 57. Upload Resume

Need durable local state.

---

# 58. Upload State

```rust
pub enum AnonymousUploadState {
    Preparing,
    Uploading,
    Paused,
    Finalizing,
    Ready,
    Failed,
    Expired,
}
```

---

# 59. Download State

```rust
pub enum AnonymousDownloadState {
    WaitingForManifest,
    Fetching,
    Paused,
    Verifying,
    Complete,
    Failed,
    Expired,
}
```

---

# 60. Resumability

Each chunk independently retrievable.

---

# 61. Resume Bitmap

```rust
pub struct ChunkBitmap {
    // compact received/uploaded state
}
```

---

# 62. Persist Resume State

Locally, encrypted.

---

# 63. Crash Recovery

After restart:

```text
resume unfinished upload/download
```

without leaking new metadata.

---

# 64. Chunk Scheduling

Do not always request chunks sequentially if that leaks exact access pattern.

---

# 65. Sequential Fetch

Simplest.

May reveal:

```text
object size
progress
```

to provider.

---

# 66. Randomized Fetch Order

Could reduce pattern leakage but hurts locality.

---

# 67. Initial Recommendation

Use:

```text
windowed pseudo-random order
```

for strict mode if provider supports range fetch.

---

# 68. Fetch Window

```text
N chunks in flight
```

bounded.

---

# 69. Concurrency

```rust
pub struct BulkTransferConcurrency {
    pub uploads: u16,
    pub downloads: u16,
}
```

---

# 70. Maximum Concurrency

Depends on:

```text
battery
network
provider
```

---

# 71. Traffic Shaping Interaction

Bulk transfer should integrate with Part 37.

---

# 72. Bulk Transfer Shaping

Potential:

```text
bandwidth caps
burst smoothing
chunk request jitter
dummy fetches
padding
```

---

# 73. Dummy Fetches

Can hide exact chunk count at high cost.

---

# 74. Strict Mode Option

```rust
pub enum BulkTrafficShaping {
    None,
    Jittered,
    Padded,
    MaximumPrivacy,
}
```

---

# 75. Maximum Privacy Bulk Mode

Could use:

```text
fixed rate window
dummy chunks
size padding
randomized chunk order
```

---

# 76. Tradeoff

Can multiply bandwidth significantly.

---

# 77. Battery Policy

Bulk anonymous transfer can be expensive.

---

# 78. Power Modes

```text
Balanced
PreserveBattery
MaintainAnonymity
```

---

# 79. Mobile Data

Adaptive mode may pause large anonymous attachments.

---

# 80. User Setting

```text
Download large anonymous files on Wi-Fi only
```

---

# 81. Maximum Anonymity

Do not silently switch to direct path just because Wi-Fi absent.

---

# 82. Sender Availability

Store-and-fetch mode allows sender to go offline after upload.

---

# 83. RelayStream Mode

Sender and receiver may need overlapping availability.

---

# 84. Store-and-Fetch Preferred

For asynchronous messaging.

---

# 85. RelayStream Use Case

Large temporary media where storage provider not desired.

---

# 86. Rendezvous Coordination

Manifest includes enough to locate approved provider anonymously.

---

# 87. Provider Descriptor

```rust
pub struct AnonymousBulkTransportDescriptor {
    pub provider: AnonymousProviderId,
    pub mode: AnonymousRendezvousMode,
    pub object_hint: Bytes,
    pub capability_binding: CapabilityBinding,
}
```

---

# 88. Object Hint Privacy

Must not expose:

```text
contact ID
conversation ID
filename
```

---

# 89. Capability Binding

Bind capability to:

```text
manifest hash
attachment key context
expiry
```

---

# 90. Manifest Authentication

Manifest sent inside E2EE.

---

# 91. Manifest Hash

```rust
pub struct AttachmentManifestDigest(pub [u8; 32]);
```

---

# 92. Capability Replay

Bearer token may be reused.

---

# 93. Use Limits

```text
max_downloads
max_bytes
expiry
```

---

# 94. Single-Use Capability

Possible for sensitive file.

---

# 95. Multi-Device Recipient

One attachment may be fetched by multiple trusted devices.

---

# 96. Capability Fanout

Option:

```text
one capability per device
```

for better revocation/accounting.

---

# 97. Shared Capability

Simpler but more linkable.

---

# 98. Recommended Strict Mode

Per-device capability.

---

# 99. Device Revocation

Revoke unused fetch capability if provider supports.

---

# 100. Capability Expiry

Short enough to reduce abuse, long enough for offline recipient.

---

# 101. Offline Recipient

Manifest can wait in mailbox.

Capability lifetime must account for mailbox delay.

---

# 102. Refresh Capability

Sender can issue replacement through mixnet if old expires.

---

# 103. Sender Offline After Expiry

File may become unrecoverable unless provider retention/capability supports it.

---

# 104. Retention Policy

```rust
pub struct AnonymousObjectPolicy {
    pub retention: Duration,
    pub max_downloads: u32,
    pub max_bytes: u64,
    pub padding: AttachmentPaddingPolicy,
}
```

---

# 105. Provider Storage Quota

Bounded.

---

# 106. Expired Object

Provider deletes encrypted chunks.

---

# 107. Deletion Semantics

Best effort unless provider supports verifiable deletion.

---

# 108. No False Deletion Claim

Do not say:

```text
permanently erased
```

unless guaranteed.

---

# 109. Local Sender Copy

Independent from provider copy.

---

# 110. Local Recipient Copy

Independent.

---

# 111. Anonymous Upload Cache

Sender may stage encrypted chunks locally.

---

# 112. Staging Encryption

Required.

---

# 113. Partial Download Cache

Encrypted.

---

# 114. File Completion

Do not expose final file until:

```text
all chunks present
manifest verified
content decrypted
integrity confirmed
```

---

# 115. Safe Open

Only after full verification.

---

# 116. Streaming Media

Could allow verified chunk-by-chunk streaming.

---

# 117. Streaming Preconditions

Need:

```text
per-chunk AEAD
ordered chunk integrity
safe decoder isolation
```

---

# 118. Video Streaming

Possible future.

---

# 119. Seek

Requires range fetch + chunk map.

---

# 120. High-Bandwidth Anonymous Data

Could include:

```text
large file transfer
media streaming
backup restore
software distribution
```

---

# 121. Separate Bulk Transport Plane

Recommended:

```text
anonymous messaging plane
anonymous bulk plane
```

---

# 122. Bulk Plane Does Not Carry Chat Control

Keep failure domains separate.

---

# 123. Provider Selection

```rust
pub trait AnonymousBulkProviderSelector {
    fn select(
        &self,
        requirements: &BulkTransferRequirements,
        available: &[AnonymousBulkProvider],
    ) -> Result<AnonymousProviderId, BulkSelectionError>;
}
```

---

# 124. Requirements

```rust
pub struct BulkTransferRequirements {
    pub anonymity: AnonymityRequirement,
    pub size: u64,
    pub resume: bool,
    pub streaming: bool,
    pub multi_device: bool,
}
```

---

# 125. Selection Hard Constraints

First:

```text
endpoint hiding
encryption compatibility
size limit
resume requirement
```

Then optimize:

```text
speed
cost
battery
```

---

# 126. No Security Averaging

Fast non-anonymous provider cannot beat anonymous requirement.

---

# 127. Provider Failover

Possible:

```text
provider A unavailable
→ provider B
```

only if approved anonymity class.

---

# 128. Cross-Provider Migration

Upload may need restart or chunk copy.

---

# 129. Multi-Provider Redundancy

Advanced:

```text
split or replicate chunks across providers
```

---

# 130. Replication

Improves availability.

Increases:

```text
metadata exposure
storage
bandwidth
```

---

# 131. Erasure Coding

Potential future:

```text
k-of-n chunks
```

across providers.

---

# 132. Erasure Coding Benefit

Resilience without full replication.

---

# 133. Privacy Risk

Cross-provider correlation.

Needs separate privacy review.

---

# 134. Initial Recommendation

Single approved provider per attachment.

---

# 135. Multi-Source Transfer

Could fetch chunks from:

```text
provider cache
sender relay
trusted peer cache
```

---

# 136. Privacy Constraint

All sources must satisfy anonymity policy.

---

# 137. Peer-Assisted Anonymous Transfer

Future.

Potentially:

```text
anonymous content addressing
capability-based chunk requests
```

---

# 138. Content Addressing

Use encrypted/randomized identifiers.

---

# 139. Avoid Plaintext Hash as Public Object ID

Could reveal known-file membership.

---

# 140. Private Content ID

```rust
pub struct PrivateContentId([u8; 32]);
```

Derived with secret context.

---

# 141. Deduplication

Cross-user dedup can leak content equality.

---

# 142. Strict Privacy

Disable provider-visible global dedup.

---

# 143. Local Dedup

Safe if device-local and encrypted.

---

# 144. Compression

Compress before encryption where safe.

---

# 145. Compression Side Channels

If attacker controls plaintext and observes size, compression may leak.

---

# 146. Attachment Compression Policy

Do not automatically compress already-compressed media.

---

# 147. Size Padding After Compression

Required for privacy bucket.

---

# 148. Metadata Scrubbing

Optional sender-side process:

```text
remove EXIF
remove GPS
remove author metadata
```

---

# 149. Privacy Warning

Media may contain sensitive embedded metadata.

---

# 150. Maximum Anonymity Default

Offer:

```text
Strip metadata
```

before send.

---

# 151. File Name Privacy

Filename encrypted.

---

# 152. Timestamps

Avoid preserving original file-system timestamps in provider metadata.

---

# 153. MIME Type

Provider should see only generic binary object.

---

# 154. Thumbnail

Thumbnail should be separately encrypted.

---

# 155. Preview Transfer

Small preview may go through mixnet.

---

# 156. Full File

Bulk plane.

---

# 157. Preview Correlation

Use independent IDs/keys where useful.

---

# 158. Abuse Resistance

Anonymous file hosting can be abused.

---

# 159. Risks

```text
storage abuse
malware
illegal content
spam
bandwidth exhaustion
capability sharing
```

---

# 160. Provider Does Not Need Plaintext Inspection

Use capability/rate/resource controls.

---

# 161. Upload Quotas

```text
per capability
per operator policy
per anonymous client token
```

---

# 162. Privacy-Preserving Quota Tokens

Future option.

---

# 163. Unknown Sender Limits

Receiver default:

```text
no auto-download
small preview only
manual approval
```

---

# 164. Malware Handling

Downloaded file remains untrusted.

---

# 165. File Validation

Use existing file safety subsystem.

---

# 166. Executables

Warn before open.

---

# 167. Archive Bombs

Scan metadata/limits safely.

---

# 168. Decompression Limits

Bound:

```text
nested depth
expanded size
entry count
```

---

# 169. Resource Exhaustion

Bound:

```text
concurrent downloads
temporary storage
hashing workers
decode workers
```

---

# 170. Backpressure

Bulk transfer yields to:

```text
calls
messages
emergency
security
```

---

# 171. Priority

```rust
pub enum BulkTransferPriority {
    Normal,
    Background,
    UserVisible,
}
```

---

# 172. Emergency

Anonymous bulk transfer should not starve emergency transport.

---

# 173. Bandwidth Scheduler

```rust
pub trait AnonymousBulkScheduler {
    async fn enqueue(
        &self,
        transfer: BulkTransferJob,
    ) -> Result<(), BulkTransferError>;

    async fn pause(
        &self,
        id: TransferId,
    ) -> Result<(), BulkTransferError>;

    async fn resume(
        &self,
        id: TransferId,
    ) -> Result<(), BulkTransferError>;
}
```

---

# 174. Bandwidth Classes

```rust
pub enum BulkBandwidthClass {
    Low,
    Balanced,
    High,
    UnlimitedByApp,
}
```

---

# 175. Adaptive Bandwidth

Respond to:

```text
metered network
battery saver
foreground
provider congestion
```

---

# 176. QoS

Anonymous bulk transfer gets lower priority than message control.

---

# 177. Traffic Analysis

Bulk transfers are inherently easier to fingerprint by volume.

---

# 178. Mitigations

```text
size padding
fixed chunk size
jittered requests
bandwidth shaping
cover fetch
provider indirection
```

---

# 179. Limits of Hiding Large Transfers

A multi-gigabyte transfer is difficult to conceal completely.

Be honest.

---

# 180. UX Wording

Prefer:

```text
Anonymous transfer
```

not:

```text
untraceable file transfer
```

---

# 181. Transfer State UX

```text
Preparing
Uploading privately
Waiting for recipient
Downloading privately
Verifying
Complete
```

---

# 182. No IP Disclosure

UI should not expose provider endpoints unless advanced diagnostics.

---

# 183. Failed Rendezvous

```text
Anonymous file route unavailable
```

No silent direct fallback.

---

# 184. Progress

Show:

```text
bytes transferred
estimated completion
```

locally.

---

# 185. Progress Privacy

Do not report exact progress remotely unless needed.

---

# 186. Read Receipts

File downloaded ≠ file opened.

---

# 187. Download ACK

Transport-level.

---

# 188. Application Receipt

Optional E2EE.

---

# 189. Manifest ACK

Can confirm manifest received.

---

# 190. Chunk ACK

Usually internal/provider-level.

---

# 191. Retry

Chunk-level retry.

---

# 192. Retry Dedup

Provider object/chunk IDs ensure idempotency.

---

# 193. Hash Mismatch

Refetch chunk.

---

# 194. Repeated Hash Mismatch

Provider/file corruption security event.

---

# 195. Final Verification Failure

Do not expose file as complete.

---

# 196. Cancellation

Sender:

```text
stop upload
revoke capability
delete provider object if possible
```

---

# 197. Recipient Cancel

Stop fetching.

Already downloaded encrypted chunks may be deleted.

---

# 198. Revocation Limits

Capability already copied cannot always be recalled.

---

# 199. Single-Use Capability Helps

For sensitive files.

---

# 200. Backup Interaction

Anonymous attachment provider is not backup storage by default.

---

# 201. Restore

Restored conversation may contain expired attachment references.

---

# 202. UI

Show:

```text
Attachment no longer available
```

---

# 203. Archive

Exported archive includes downloaded local attachment if available.

---

# 204. Multi-Device Sync

Each device can fetch independently.

---

# 205. Download Ownership

Device-local by default.

---

# 206. Server-Side Team Storage

Not part of strict anonymous transfer unless explicitly integrated.

---

# 207. Plugin Access

Plugins never receive fetch capability secrets by default.

---

# 208. Plugin File Access

After file is downloaded and authorized, plugin may access through normal capability system.

---

# 209. Diagnostics

Normal:

```text
Anonymous file transfer: Active
Provider: Available
```

---

# 210. Advanced Diagnostics

Could show:

```text
provider type
chunk size
resume state
padding mode
bandwidth class
```

---

# 211. Forbidden Diagnostics

Do not show:

```text
capability token
object secret
peer IP
route hops
```

---

# 212. Metrics

Privacy-safe aggregate:

```text
upload success
download success
bytes bucket
latency bucket
resume count
verification failures
```

---

# 213. No Per-Contact Metrics

Hard rule.

---

# 214. Support Bundle

Redact:

```text
capabilities
object IDs if linkable
file names
peer identity
```

---

# 215. Storage Schema

Potential:

```text
anonymous_attachment_manifests
anonymous_upload_jobs
anonymous_download_jobs
anonymous_chunk_state
anonymous_rendezvous_objects
anonymous_fetch_capabilities
```

---

# 216. `anonymous_upload_jobs`

Fields:

```text
transfer_id
manifest_id
provider
state
uploaded_bitmap
expiry
```

---

# 217. `anonymous_download_jobs`

Fields:

```text
transfer_id
manifest_id
provider
state
received_bitmap
temp_path_ref
```

---

# 218. Secret References

Capability secrets stored in secure storage, not DB plaintext.

---

# 219. Transactionality

Critical:

```text
chunk verified
→ persist state
→ mark bitmap
```

atomically.

---

# 220. Crash Safety

Never mark chunk complete before durable write.

---

# 221. Temp File

Use atomic rename after full verification.

---

# 222. Cleanup

Garbage-collect:

```text
expired upload staging
partial downloads
expired capabilities
abandoned manifests
```

---

# 223. Cleanup Policy

Never delete user-authoritative local file without explicit policy.

---

# 224. Provider Garbage Collection

TTL-based.

---

# 225. Provider Encryption

Transport provider may also encrypt storage.

But SIAR file encryption remains mandatory.

---

# 226. Key Separation

Separate:

```text
message E2EE key
attachment content key
provider capability secret
```

---

# 227. Key Rotation

Attachment key immutable per object.

Re-share requires new key/object.

---

# 228. Re-Send

Create new manifest/capability.

---

# 229. Forwarding Attachment

Forwarding should not reuse original capability by default.

---

# 230. Why

Original sender may revoke/expire it.

---

# 231. Forward Flow

```text
download/decrypt locally
→ create new attachment encryption
→ upload new object
→ new capability
```

---

# 232. Zero-Copy Optimization

Within device:

```text
stream file
→ encrypt
→ hash
→ upload
```

without loading full file.

---

# 233. Streaming Pipeline

```text
reader
→ chunker
→ encryptor
→ hasher
→ uploader
```

---

# 234. Download Pipeline

```text
fetch
→ verify transport
→ decrypt
→ verify content
→ write
```

---

# 235. Parallelism

Bounded.

---

# 236. Worker Pools

Separate:

```text
I/O
crypto
hashing
media preview
```

---

# 237. Android SAF

Use file descriptors/URIs.

No giant JNI byte arrays.

---

# 238. Desktop

Use file handles/paths with secure sandboxing as appropriate.

---

# 239. UI Boundary

UI gets:

```text
transfer ID
progress
state
file handle after completion
```

not raw chunk buffers.

---

# 240. Anonymous Bulk Transport Trait

```rust
#[async_trait]
pub trait AnonymousBulkTransport: Send + Sync {
    async fn prepare_upload(
        &self,
        request: BulkUploadRequest,
    ) -> Result<BulkUploadSession, BulkTransferError>;

    async fn upload_chunk(
        &self,
        session: &BulkUploadSession,
        chunk: EncryptedChunk,
    ) -> Result<(), BulkTransferError>;

    async fn finalize_upload(
        &self,
        session: &BulkUploadSession,
    ) -> Result<AnonymousFetchCapability, BulkTransferError>;

    async fn download_chunk(
        &self,
        capability: &AnonymousFetchCapability,
        index: u32,
    ) -> Result<EncryptedChunk, BulkTransferError>;

    async fn revoke(
        &self,
        capability: &AnonymousFetchCapability,
    ) -> Result<(), BulkTransferError>;
}
```

---

# 241. Manifest Service

```rust
pub trait AnonymousAttachmentManifestService {
    fn create_manifest(
        &self,
        source: &AttachmentSource,
        policy: &AnonymousAttachmentPolicy,
    ) -> Result<AnonymousAttachmentManifest, ManifestError>;

    fn verify_manifest(
        &self,
        manifest: &AnonymousAttachmentManifest,
    ) -> Result<(), ManifestError>;
}
```

---

# 242. Transfer Coordinator

```rust
#[async_trait]
pub trait AnonymousAttachmentCoordinator {
    async fn send(
        &self,
        request: AnonymousAttachmentSendRequest,
    ) -> Result<TransferId, BulkTransferError>;

    async fn receive(
        &self,
        manifest: AnonymousAttachmentManifest,
    ) -> Result<TransferId, BulkTransferError>;

    async fn pause(
        &self,
        transfer: TransferId,
    ) -> Result<(), BulkTransferError>;

    async fn resume(
        &self,
        transfer: TransferId,
    ) -> Result<(), BulkTransferError>;
}
```

---

# 243. Error Taxonomy

```rust
pub enum BulkTransferError {
    AnonymousRouteUnavailable,
    ProviderUnavailable,
    CapabilityExpired,
    CapabilityInvalid,
    UploadQuotaExceeded,
    DownloadQuotaExceeded,
    ChunkVerificationFailed,
    ManifestInvalid,
    StorageFull,
    Cancelled,
    Unsupported,
    Internal,
}
```

---

# 244. No Direct-Fallback Error

Could be explicit:

```text
RequiresDowngrade
```

---

# 245. Ask-Before-Downgrade

If user policy allows:

```text
This file is too large for the current anonymous path.
Use a less-private relay for this transfer?
```

---

# 246. Temporary Scope

Downgrade applies only:

```text
this transfer
```

unless user chooses broader.

---

# 247. Audit Event

Local:

```text
attachment privacy downgrade approved
```

without file content.

---

# 248. Testkit

Need fake bulk provider.

---

# 249. Fake Provider Features

```text
upload
range fetch
resume
expiry
quota
corruption
delay
revocation
```

---

# 250. Unit Tests

Test:

```text
manifest
chunking
Merkle root
capability binding
padding
```

---

# 251. Resume Tests

Crash mid-upload/download.

---

# 252. Corruption Tests

Tamper chunk.

---

# 253. Expiry Tests

Capability expires before fetch.

---

# 254. Revocation Tests

Capability revoked before use.

---

# 255. Multi-Device Tests

Per-device capability.

---

# 256. Provider Failure Tests

Provider unavailable mid-transfer.

---

# 257. Migration Tests

Switch approved provider.

---

# 258. Traffic Shaping Tests

Verify jitter/rate policies.

---

# 259. Size-Hiding Tests

Files in same bucket produce indistinguishable outer padded size where policy requires.

---

# 260. Metadata Leak Tests

Provider-facing manifest contains no raw:

```text
filename
ConversationId
AccountId
DeviceId
```

---

# 261. Capability Leak Tests

Secret canary absent from:

```text
logs
support bundles
UI
plugins
```

---

# 262. Storage Exhaustion Tests

Partial download stops safely.

---

# 263. Integrity Tests

Final file only marked complete after verification.

---

# 264. Retry Tests

Corrupt chunk refetched.

---

# 265. Abuse Tests

```text
huge fake manifest
chunk flood
capability guessing
quota exhaustion
```

---

# 266. Fuzzing

Fuzz:

```text
manifest parser
chunk headers
Merkle proofs
capability descriptors
resume state
```

---

# 267. Property Tests

Properties:

```text
reassembled verified file == original
no chunk marked complete before durable write
capability expiry always blocks new fetch
strict anonymity never chooses direct transfer
```

---

# 268. Performance Tests

Measure:

```text
encryption throughput
hashing throughput
upload/download throughput
resume overhead
memory
CPU
battery
padding overhead
```

---

# 269. Scale Tests

Synthetic:

```text
1 MB
100 MB
1 GB
10 GB
```

within product-supported limits.

---

# 270. Android Tests

```text
SAF
background restrictions
battery saver
Wi-Fi only
process death
```

---

# 271. Desktop Tests

```text
daemon resume
sleep/wake
large file
disk pressure
```

---

# 272. Privacy Simulation

Model:

```text
size leakage
transfer timing
provider visibility
chunk request pattern
```

---

# 273. No Privacy Proof from Simulation

Hard rule.

---

# 274. Security Invariants

Mandatory:

```text
1. Attachment contents are encrypted before anonymous bulk transport.
2. Capability secrets are bearer secrets and are never logged.
3. Provider-visible IDs are random and unrelated to SIAR application IDs.
4. Strict anonymity never silently falls back to direct/ordinary relay transfer.
5. File completion is impossible before full integrity verification.
6. Chunk upload/download state is crash-safe and resumable.
7. Provider does not receive plaintext filenames or conversation identity.
8. Cross-user provider-visible deduplication is disabled in strict privacy mode.
9. Multi-device strict mode prefers per-device capabilities.
10. Large attachment traffic yields to messages, calls, security, and emergency traffic.
11. Partial data never appears as complete file.
12. Forwarding creates a new attachment object/capability by default.
```

---

# 275. Recommended Crate Layout

```text
crates/
├── siar-anonymous-attachment-core/
├── siar-anonymous-attachment-manifest/
├── siar-anonymous-attachment-chunk/
├── siar-anonymous-attachment-crypto/
├── siar-anonymous-rendezvous/
├── siar-anonymous-bulk-transport/
├── siar-anonymous-bulk-scheduler/
├── siar-anonymous-bulk-storage/
├── siar-anonymous-bulk-observability/
└── siar-anonymous-bulk-testkit/
```

---

# 276. `siar-anonymous-attachment-core`

Owns IDs/policies/errors.

---

# 277. `siar-anonymous-attachment-manifest`

Manifest creation/verification.

---

# 278. `siar-anonymous-attachment-chunk`

Chunking/resume/Merkle.

---

# 279. `siar-anonymous-attachment-crypto`

Attachment AEAD/key handling.

---

# 280. `siar-anonymous-rendezvous`

Capability/rendezvous coordination.

---

# 281. `siar-anonymous-bulk-transport`

Provider-neutral bulk trait.

---

# 282. `siar-anonymous-bulk-scheduler`

Bandwidth/priority/shaping.

---

# 283. `siar-anonymous-bulk-storage`

Encrypted staging/temp lifecycle.

---

# 284. `siar-anonymous-bulk-observability`

Privacy-safe metrics.

---

# 285. `siar-anonymous-bulk-testkit`

Fake provider/fault injection.

---

# 286. Initial Production Scope

Implement first:

```text
encrypted attachment manifest
fixed-size chunking
Merkle root/integrity
store-and-fetch rendezvous
random object IDs
short-lived fetch capabilities
single approved anonymous bulk provider
upload/download resume
per-device capability
Wi-Fi-only option
strict no-direct-fallback
privacy-safe diagnostics
```

Then add:

```text
relay-stream mode
multi-provider failover
privacy padding buckets
randomized chunk order
dummy fetches
anonymous media streaming
peer-assisted transfer
erasure coding
```

---

# 287. Definition of Done

Part 40 is complete when:

- large attachment transfer is separated into control and bulk-data planes
- attachment contents are encrypted before upload
- manifests are E2EE-protected and provider-visible metadata is minimized
- random provider-facing object IDs are used
- rendezvous capabilities are short-lived, scoped, and secret
- store-and-fetch asynchronous transfer is supported
- upload/download resume is crash-safe
- fixed-size chunking and optional size padding are defined
- Merkle/integrity verification prevents partial/corrupt completion
- strict anonymity never silently falls back to direct transport
- multi-device capability behavior is defined
- traffic shaping/bandwidth/battery policies integrate with Part 37
- provider failover and future multi-provider options are bounded by anonymity class
- anonymous transfer UX and downgrade prompts are explicit
- provider quotas, spam/malware/resource controls are defined
- diagnostics/metrics avoid file/contact correlation
- Android SAF/background constraints and desktop large-file behavior are defined
- fuzz, abuse, corruption, resume, expiry, capability-leak, metadata-leak, and scale tests are specified

---

# 288. Final Architecture

```text
                       ATTACHMENT SOURCE
                               │
                               ▼
                      Encrypt / Chunk / Hash
                               │
                               ▼
                    Anonymous Attachment Manifest
                               │
                ┌──────────────┴──────────────┐
                │                             │
         Mixnet Control Plane          Anonymous Bulk Plane
                │                             │
       Manifest / Capability          Encrypted Chunks
                │                             │
                └──────────────┬──────────────┘
                               ▼
                          RECIPIENT
                               │
                               ▼
                    Fetch / Resume / Verify
                               │
                               ▼
                         Reassemble File
```

Strict-mode transfer:

```text
Large file
   ↓
Encrypt
   ↓
Upload through approved anonymous rendezvous
   ↓
Send capability through mixnet/E2EE
   ↓
Recipient fetches anonymously
   ↓
Verify
   ↓
Expose local file
```

---

# 289. Final Principle

Large anonymous transfers should not force SIAR to choose between:

```text
privacy
or
practical bandwidth
```

The correct model is:

```text
mixnet-protected coordination
+
encrypted bulk objects
+
anonymous rendezvous capabilities
+
resumable chunk transfer
+
strict endpoint hiding
+
no silent downgrade
```

not:

```text
large file too slow on mixnet
→ reveal sender and receiver over direct QUIC
```

This architecture gives SIAR a practical high-bandwidth anonymity path that preserves the guarantees established in Parts 34–39 while remaining usable for real-world files and media.
