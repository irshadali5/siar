# Part 05 — Robust File / Blob Subsystem Architecture

## Reusable P2P Communication Platform

**Status:** Architecture specification  
**Part:** 05 of 24  
**Primary language:** Rust  
**Primary goals:** content-addressed storage, resumable transfer, integrity, encryption, low-copy streaming, deduplication, partial availability, multi-transport delivery, crash recovery, reuse across messaging/files/emergency/ERP/custom products

---

# 1. Purpose

The communication platform needs a dedicated binary-data subsystem that can safely and efficiently handle:

- chat attachments
- photos
- videos
- voice messages
- documents
- large files
- application packages
- emergency images
- diagnostic bundles
- ERP documents
- backups
- future product-specific binary payloads

The blob subsystem must be reusable independently from messaging.

The architectural rule is:

> **Messages reference blobs; messages do not own file-transfer mechanics.**

This separation allows:

```text
messaging-only application
file-only application
messaging + files
emergency application
ERP document transfer
headless relay
```

to reuse the same file/blob engine.

---

# 2. Core Principles

The subsystem must provide:

- content addressing
- deterministic integrity verification
- streaming I/O
- bounded memory usage
- chunked transfer
- resumable transfer
- partial availability
- optional parallel transfer
- encryption
- local staging
- deduplication
- storage quotas
- garbage collection
- reference tracking
- transport independence
- crash consistency
- DTN compatibility
- multi-device targeting
- diagnostics
- policy-driven retention

It must not depend on:

- Dioxus
- messenger screens
- Android UI
- a specific transport
- a specific database
- a central server

---

# 3. Architectural Position

```text
Application
    ↓
Messaging / File Share / ERP / Emergency
    ↓
File Service
    ↓
Blob Manager
    ↓
Chunk Store / Manifest / Encryption
    ↓
Transfer Planner
    ↓
Routing Policy Engine
    ↓
Iroh / LAN / Wi-Fi / Bluetooth / DTN
```

The file subsystem owns file semantics.

The routing layer owns path selection.

The transport layer moves bytes.

The storage layer persists blobs/chunks.

---

# 4. Main Crates

Recommended split:

```text
comm-files/
comm-blob-store/
comm-blob-types/
comm-transfer/
comm-transfer-protocol/
comm-storage-filesystem/
```

Potential workspace:

```text
crates/
├── comm-files/
├── comm-blob-store/
├── comm-transfer/
├── comm-transfer-protocol/
└── comm-storage-filesystem/
```

Avoid one huge crate if the subsystem grows large.

---

# 5. Blob Concept

A blob is an immutable binary object.

```rust
pub struct BlobId([u8; 32]);
```

A blob is identified by content-derived identity or a secure encrypted-object identity.

The blob does not inherently know:

```text
conversation
sender
recipient
file name
UI
```

Metadata references the blob externally.

---

# 6. Content Addressing

Conceptually:

```text
content bytes
    ↓
cryptographic hash
    ↓
BlobId
```

Use a strong cryptographic hash such as BLAKE3 or another explicitly selected algorithm.

Benefits:

- integrity verification
- deduplication
- immutable identity
- resumability
- peer caching
- partial transfer verification
- distributed lookup

---

# 7. Plaintext vs Ciphertext Addressing

This is a major design decision.

## Option A — plaintext-addressed

```text
hash(plaintext)
```

Pros:

- stronger deduplication

Cons:

- leaks equality of identical content
- problematic for privacy

## Option B — ciphertext-addressed

```text
encrypt
 ↓
hash(ciphertext)
```

Pros:

- better privacy
- avoids plaintext equality leakage

Cons:

- less cross-user deduplication

For a privacy-focused communication platform, prefer:

> **Encrypt first, then content-address the ciphertext.**

---

# 8. Blob Identity Layers

Useful model:

```text
LogicalAttachmentId
      ↓
EncryptedBlobId
      ↓
ChunkIds
```

A message may reference a logical attachment record.

The actual storage identity is the encrypted blob.

---

# 9. Blob Descriptor

```rust
pub struct BlobDescriptor {
    pub blob_id: BlobId,
    pub size: u64,
    pub chunking: ChunkingDescriptor,
    pub encryption: EncryptionDescriptor,
    pub media_type: Option<MediaType>,
    pub manifest_id: Option<ManifestId>,
}
```

Do not trust remote-declared size blindly.

---

# 10. File Metadata

User-facing metadata should be separate from blob identity.

```rust
pub struct FileMetadata {
    pub display_name: Option<FileName>,
    pub media_type: Option<MediaType>,
    pub logical_size: u64,
    pub created_at: Option<Timestamp>,
}
```

This metadata can be encrypted inside the application message if privacy requires.

---

# 11. Chunking

Large blobs must be chunked.

```text
Blob
 ├── Chunk 0
 ├── Chunk 1
 ├── Chunk 2
 └── ...
```

Chunking enables:

- resume
- parallelism
- corruption isolation
- partial transfer
- DTN carriage
- progress reporting
- cache reuse

---

# 12. Fixed vs Content-Defined Chunking

## Fixed-size chunks

Example:

```text
1 MiB
4 MiB
```

Pros:

- simple
- fast
- easy range requests

## Content-defined chunking

Pros:

- better dedup across modified files

Cons:

- more CPU
- more metadata
- more complexity

For v1, prefer:

> **Fixed-size chunking with a configurable chunk size.**

---

# 13. Chunk Size Policy

Chunk size should depend on workload.

Example ranges:

```text
small file:
single chunk

medium:
256 KiB – 1 MiB chunks

large:
1 MiB – 4 MiB chunks
```

Do not hard-code one size for every transport.

The manifest records the chosen scheme.

---

# 14. Chunk Identity

Each chunk can have:

```rust
pub struct ChunkHash([u8; 32]);
```

The manifest includes chunk hashes.

This allows verifying individual chunks before whole-object completion.

---

# 15. Blob Manifest

```rust
pub struct BlobManifest {
    pub version: u16,
    pub blob_id: BlobId,
    pub total_size: u64,
    pub chunk_size: u32,
    pub chunks: Vec<ChunkDescriptor>,
}
```

Chunk descriptor:

```rust
pub struct ChunkDescriptor {
    pub index: u32,
    pub offset: u64,
    pub size: u32,
    pub hash: ChunkHash,
}
```

---

# 16. Manifest Limits

Bound:

```text
max chunks
max manifest size
max file size
```

Do not allow a tiny network frame to declare millions of chunks and force huge allocation.

---

# 17. Manifest Hierarchy

For extremely large files, consider hierarchical manifests later.

Example:

```text
Root manifest
 ├── Segment manifest A
 ├── Segment manifest B
 └── Segment manifest C
```

This avoids massive flat manifests.

Not required initially.

---

# 18. Encryption Model

Recommended:

```text
plaintext file
    ↓
per-blob random key
    ↓
chunk encryption
    ↓
ciphertext chunks
    ↓
content addressing
```

Use authenticated encryption.

Each chunk must be independently verifiable/authenticated.

---

# 19. Per-Blob Key

```rust
pub struct BlobEncryptionKey(SecretBytes);
```

The key is not stored in public blob metadata.

It is distributed through the E2EE application layer.

---

# 20. Chunk Nonces

Never reuse AEAD nonce/key pairs.

Derive chunk nonce safely from:

```text
blob key
chunk index
domain separation
```

or use securely generated per-chunk nonces recorded in authenticated metadata.

The exact construction should use a vetted cryptographic design.

---

# 21. Encryption Metadata

```rust
pub struct EncryptionDescriptor {
    pub scheme: EncryptionSchemeId,
    pub version: u16,
}
```

Do not expose raw secret key.

---

# 22. Metadata Privacy

Sensitive metadata can include:

```text
file name
MIME type
dimensions
duration
creation time
```

Decide which metadata is:

```text
public transport metadata
encrypted application metadata
local-only metadata
```

Default should minimize exposure.

---

# 23. Message Attachment Reference

Messaging stores a reference:

```rust
pub struct AttachmentReference {
    pub blob: BlobDescriptor,
    pub metadata: EncryptedOrVisibleMetadata,
}
```

The message does not contain the file bytes.

---

# 24. File-Only Transfer

A file-sharing app can use:

```text
FileOffer
FileAccept
Transfer
Blob
```

without creating a conversation.

This is a core reusability requirement.

---

# 25. Transfer Identity

```rust
pub struct TransferId([u8; 16]);
```

A transfer is distinct from a blob.

Same blob may have:

```text
multiple transfers
multiple recipients
multiple retries
```

---

# 26. Transfer State Machine

```text
Created
 ↓
Offered
 ↓
Accepted
 ↓
Preparing
 ↓
Transferring
 ↓
Verifying
 ↓
Completed
```

Alternative states:

```text
Paused
Deferred
Cancelled
Failed
Expired
```

---

# 27. Transfer Record

```rust
pub struct TransferRecord {
    pub transfer_id: TransferId,
    pub blob_id: BlobId,
    pub direction: TransferDirection,
    pub peer: DeviceId,
    pub state: TransferState,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}
```

---

# 28. Transfer Journal

High-frequency operational state belongs in a transfer journal/projection.

Examples:

```text
chunk bitmap
bytes verified
active path
retry count
```

Do not append a permanent event for every packet.

Part 04 event log stores meaningful semantic transitions.

---

# 29. Resume Bitmap

For fixed chunking:

```text
received:
[1,1,1,0,0,1,...]
```

Store efficiently as:

```text
bitset
ranges
```

On restart, receiver knows exactly what is missing.

---

# 30. Range-Based Resume

Alternative:

```text
received ranges
```

Example:

```text
0..4 MiB
8..12 MiB
```

Useful when transport supports ranges.

Choose one canonical internal form.

---

# 31. Resume Protocol

Receiver sends:

```text
manifest known
missing chunks:
3,4,9,10
```

Sender transfers only missing chunks.

No full restart.

---

# 32. Partial Availability

Receiver should be able to access:

```text
thumbnail
preview
first playable media segment
```

before full object completion where application allows.

The blob store exposes safe partial-read status.

---

# 33. Progressive Images

Image flow:

```text
thumbnail blob
preview blob
original blob
```

Prefer separate objects rather than relying only on arbitrary partial decode.

This simplifies UI and transport policy.

---

# 34. Progressive Video/File Handling

For large video/files, partial transfer may be usable if format supports it.

The blob layer should not assume every file is stream-playable.

Media layer decides.

---

# 35. Transfer Planner

```text
TransferPlanner
├── destination resolution
├── chunk scheduling
├── route request generation
├── concurrency limits
├── retry planning
└── policy evaluation
```

It consumes Part 03 routing decisions.

---

# 36. Transport Independence

Transfer engine must not contain:

```text
Iroh-specific send()
Bluetooth-specific write()
```

Instead:

```text
Transfer Engine
     ↓
Transport/Session abstraction
```

This allows transfer over:

```text
Iroh
LAN
Wi-Fi Direct
Bluetooth
DTN
```

---

# 37. Streaming I/O

Never:

```text
read entire 2 GB file into Vec<u8>
```

Use:

```text
File
 ↓
bounded reader
 ↓
chunk buffer
 ↓
encrypt
 ↓
transport
```

Receiver:

```text
transport
 ↓
chunk
 ↓
verify/decrypt
 ↓
temp/staging storage
```

---

# 38. Low-Copy Data Path

Use types like:

```text
Bytes
BytesMut
Arc<[u8]>
```

where appropriate.

Avoid repeated:

```text
Vec → clone → encrypt copy → clone → send
```

Do not sacrifice safety for theoretical zero-copy.

---

# 39. Buffer Pool

For high-throughput transfers, a reusable bounded buffer pool can reduce allocations.

```text
BufferPool
├── max buffers
├── chunk size classes
└── backpressure
```

Do not create unbounded pools.

---

# 40. Backpressure

Pipeline:

```text
disk reader
 ↓
bounded chunk queue
 ↓
encryptor
 ↓
bounded transport queue
 ↓
network
```

If network slows:

```text
reader slows
```

Memory remains bounded.

---

# 41. Parallelism

Support configurable parallel chunk transfer.

Example:

```text
4 concurrent chunks
```

But increase only when:

```text
bandwidth
CPU
storage
battery
```

justify it.

---

# 42. Per-Transport Parallelism

Iroh/LAN:

```text
higher parallelism
```

Bluetooth:

```text
very low parallelism
```

DTN:

```text
bundle-level scheduling
```

The transfer engine uses route capabilities.

---

# 43. Adaptive Concurrency

Future optimization:

```text
increase concurrency
while throughput improves

reduce
when:
loss rises
latency rises
memory pressure
battery saver
```

Start with deterministic limits.

---

# 44. File Offer Protocol

Sender may send:

```rust
pub struct FileOffer {
    pub transfer_id: TransferId,
    pub blob: BlobDescriptor,
    pub encrypted_metadata: Bytes,
    pub expiry: Option<Timestamp>,
}
```

Receiver decides:

```text
accept
reject
defer
metadata-only
```

---

# 45. Auto-Accept Policy

Applications can define:

```text
contacts only
small files
Wi-Fi only
specific MIME types
emergency thumbnails
```

File subsystem enforces policy supplied by application.

---

# 46. Transfer Authorization

Before receiving:

```text
peer identity
authorization
size policy
storage quota
content policy
```

must be checked.

Connection alone does not imply permission.

---

# 47. Quotas

Support:

```text
max file size
max concurrent transfers
max per-peer bytes
max daily received bytes
max relay storage
max cache size
```

This protects resource usage.

---

# 48. Storage Reservation

Before accepting a large transfer:

```text
reserve expected storage
```

or verify sufficient quota.

Avoid accepting 20 GB then failing at 99%.

---

# 49. Sparse Files

Where filesystem supports:

```text
sparse allocation
```

can help partial/range downloads.

Use platform capability abstraction.

Not mandatory for correctness.

---

# 50. Temporary Staging

Incoming incomplete objects should live in:

```text
staging/
```

not final blob namespace.

After full verification:

```text
atomic move/commit
```

to final blob store.

---

# 51. Atomic Finalization

Flow:

```text
receive chunks
 ↓
verify each
 ↓
verify full manifest/blob
 ↓
fsync as policy requires
 ↓
atomic rename/commit
 ↓
mark Complete
```

Never expose incomplete object as complete.

---

# 52. Crash Recovery

On restart:

```text
load transfer journal
 ↓
inspect staging files
 ↓
verify recorded completed chunks
 ↓
resume missing chunks
```

Do not trust temp files blindly.

---

# 53. Temp File Naming

Use opaque safe IDs:

```text
<TransferId>.partial
```

Do not use untrusted file names as filesystem paths.

---

# 54. Path Traversal Protection

Incoming display name:

```text
../../etc/passwd
```

must remain just a display string.

Never construct storage paths directly from remote filenames.

---

# 55. Filename Sanitization

When exporting to user filesystem:

```text
sanitize
normalize
resolve collision
```

Examples:

```text
photo.jpg
photo (1).jpg
```

Keep original display name separately if useful.

---

# 56. MIME Type Validation

Remote MIME type is only a hint.

Where security matters:

```text
sniff/inspect content safely
```

before opening.

Do not execute files based on declared MIME.

---

# 57. Executable Content

Downloads may contain:

```text
APK
EXE
script
archive
```

The communication platform should not auto-execute received files.

Opening is an explicit application/user action.

---

# 58. Archive Safety

If application extracts archives:

```text
zip-slip/path traversal
decompression bomb
symlink
```

must be handled in a separate safe extraction subsystem.

Blob layer itself stores opaque bytes.

---

# 59. Deduplication

If ciphertext-addressed:

```text
same encrypted object
→ same blob
```

can deduplicate locally.

Different encryption keys generally prevent cross-message plaintext deduplication.

That is acceptable for privacy.

---

# 60. Reference Counting

Track logical references:

```text
message attachment
file-share history
draft
cache pin
DTN bundle
backup
```

Blob becomes GC-eligible when no required references remain.

---

# 61. Pinning

```rust
pub enum PinReason {
    UserSaved,
    ActiveTransfer,
    MessageReference,
    DtnRequired,
    BackupPending,
}
```

Pinned blobs are not evicted.

---

# 62. Cache vs Durable Blob

Classify:

```text
Durable
Cache
Temporary
Relay
```

Different retention rules apply.

---

# 63. Garbage Collection

GC flow:

```text
scan unreferenced blobs
 ↓
respect grace period
 ↓
verify no active transfer
 ↓
delete atomically
```

Never delete a blob solely because it is old.

---

# 64. Grace Period

A short grace period protects against:

```text
race between unreference/re-reference
projection lag
transaction timing
```

Use configurable policy.

---

# 65. Cache Eviction

Evict in order:

```text
expired temp
unreferenced previews
old cache
completed relay blobs
large low-priority objects
```

Do not evict:

```text
user-saved
active
critical DTN
```

unless emergency storage policy explicitly allows.

---

# 66. Storage Pressure

States:

```rust
pub enum StoragePressure {
    Normal,
    Elevated,
    Critical,
    Full,
}
```

Effects:

```text
reduce prefetch
pause bulk receive
evict cache
reject new large files
preserve critical data
```

---

# 67. Relay Blob Storage

A DTN relay may store encrypted blobs/chunks for others.

Relay storage must be:

- quota-limited
- opaque
- expiring
- separately accounted
- lower trust

Relay node does not get content keys.

---

# 68. DTN Chunk Strategy

Large files should not be blindly copied through BLE mesh.

Policy may allow:

```text
metadata only
thumbnail
selected chunks
full file only on capable relay
```

DTN subsystem uses file chunk identities and priorities.

---

# 69. Emergency Files

Emergency mode can classify:

```text
SOS text → highest
location → highest
thumbnail → high
voice → high
full image → medium
video → low/bulk
```

Blob engine supplies objects; Part 17 decides priority.

---

# 70. Multi-Device File Delivery

Destination may be:

```text
account
specific device
all devices
```

Example:

```text
send 4 GB file to Bob's laptop only
```

Part 02 identity resolves target devices.

---

# 71. Own-Device Blob Sync

Policies:

```text
metadata everywhere
thumbnail everywhere
full file on demand
full file to selected devices
```

Avoid replicating every large blob to every device automatically.

---

# 72. Peer Discovery of Blob Availability

Peers can advertise:

```text
I have BlobId X
```

only where privacy policy permits.

Do not globally broadcast private content hashes.

Availability exchange should occur inside authorized sessions.

---

# 73. Swarming

Future capability:

```text
download different chunks from multiple authorized peers
```

Example:

```text
Alice laptop has chunks
Alice phone has chunks
recipient fetches from both
```

This can improve speed/resilience.

Not necessary for initial release.

---

# 74. Source Selection

For each chunk, potential sources may include:

```text
original sender
sender's other device
authorized cache
relay
```

Routing chooses path/source according to policy.

---

# 75. Trust of Sources

Content addressing means untrusted transport/source can provide bytes, but receiver still verifies hashes and decryption/authentication.

Authorization still controls who can learn blob existence/keys.

---

# 76. Full Blob Verification

After all chunks:

```text
reconstruct ordered ciphertext
 ↓
verify root/blob identity
```

Do not rely only on per-chunk hashes if the manifest/root binding is incomplete.

---

# 77. Manifest Authentication

The manifest must be authenticated.

Possible:

```text
manifest covered by E2EE message signature/authentication
```

or separately signed/authenticated.

An attacker must not be able to reorder/replace chunk descriptors.

---

# 78. Merkle Tree Option

For large blobs:

```text
Merkle root
```

can authenticate chunks efficiently.

Benefits:

- independent chunk verification
- partial proofs
- swarming

A flat chunk-hash manifest may be sufficient initially.

---

# 79. Chunk Compression

Do not compress already compressed media blindly.

Potential compression for:

```text
text archive
structured data
```

should be content-aware and application-controlled.

Never compress encrypted ciphertext expecting useful savings.

---

# 80. File Compression Architecture

If compression is used:

```text
plaintext
 ↓
optional compression
 ↓
encryption
 ↓
chunking/addressing
```

or chunking before encryption depending chosen format.

The order must be explicitly specified.

---

# 81. Compression Bomb Protection

If decompression occurs:

```text
max output size
ratio limits
streaming decode
```

must be enforced.

---

# 82. Media Derivatives

Blob subsystem should support related blobs:

```text
original
thumbnail
preview
transcoded derivative
waveform
```

Represent relationships in metadata, not inside storage identity.

---

# 83. Image Derivatives

For a photo:

```text
original encrypted blob
thumbnail encrypted blob
preview encrypted blob
```

This provides faster conversation rendering and mesh transfer.

---

# 84. Voice Note Representation

Voice note:

```text
Opus blob
waveform metadata
duration
```

The file/blob system stores it as ordinary binary content.

---

# 85. Video Attachment

Video file attachment is not the same as realtime call media.

Video attachment:

```text
blob transfer
```

Call:

```text
realtime media protocol
```

Keep these separate.

---

# 86. Blob Read API

```rust
pub trait BlobReader {
    async fn read_range(
        &self,
        range: ByteRange,
    ) -> Result<Bytes, BlobError>;

    async fn stream(
        &self,
    ) -> Result<BlobStream, BlobError>;
}
```

Avoid APIs that require materializing entire objects.

---

# 87. Blob Write API

```rust
pub trait BlobWriter {
    async fn write_chunk(
        &mut self,
        index: u32,
        data: Bytes,
    ) -> Result<(), BlobError>;

    async fn finalize(
        self,
    ) -> Result<BlobId, BlobError>;
}
```

Writer verifies expected size/hash.

---

# 88. Blob Store Trait

```rust
pub trait BlobStore: Send + Sync {
    async fn contains(&self, id: BlobId) -> Result<bool, BlobError>;
    async fn stat(&self, id: BlobId) -> Result<BlobStat, BlobError>;
    async fn open(&self, id: BlobId) -> Result<Box<dyn BlobReader>, BlobError>;
    async fn begin_write(&self, descriptor: BlobDescriptor)
        -> Result<Box<dyn BlobWriter>, BlobError>;
    async fn delete(&self, id: BlobId) -> Result<(), BlobError>;
}
```

---

# 89. Metadata Store

Blob bytes and metadata can be separated:

```text
filesystem:
ciphertext blobs

SQLite:
descriptors
references
transfer state
quotas
```

This is a strong practical architecture.

---

# 90. Directory Layout

Example:

```text
blob-root/
├── objects/
│   ├── ab/
│   │   └── <blob-id>
│   ├── cd/
│   └── ...
├── staging/
├── temp/
└── quarantine/
```

Prefix sharding avoids giant directories.

---

# 91. Quarantine

Malformed or suspicious received objects may be placed in:

```text
quarantine
```

for diagnostics before deletion.

Do not expose them to normal applications.

---

# 92. Filesystem Permissions

Blob directories should use restrictive permissions.

On mobile, app sandbox provides additional isolation.

On desktop, set application-owned paths carefully.

---

# 93. Database Schema

Conceptual tables:

```text
blobs
blob_references
blob_derivatives
transfers
transfer_chunks
transfer_sources
blob_pins
storage_quotas
```

---

# 94. Blob Table

Example fields:

```text
blob_id
size
state
created_at
last_accessed
storage_class
reference_count
```

Do not store content path from untrusted metadata.

---

# 95. Transfer Chunk Table

```text
transfer_id
chunk_index
state
verified
bytes_received
retry_count
```

Use compact representation for very large chunk counts where needed.

---

# 96. Chunk Bitmap Optimization

For thousands of chunks, storing one SQL row per chunk may be expensive.

Alternative:

```text
compressed bitmap / roaring bitmap / bitset blob
```

Benchmark before choosing.

---

# 97. Durable Progress

Progress displayed to user should derive from:

```text
verified bytes
```

not merely bytes received into memory.

This gives truthful resume semantics.

---

# 98. Transfer ACK

Sender should distinguish:

```text
transport ACK
chunk accepted
blob verified
user/application accepted
```

These are different states.

---

# 99. Chunk ACK

Receiver can ACK:

```text
chunk index verified
```

or use range/bitmap acknowledgements.

Avoid ACK per tiny frame if chunk is already large.

---

# 100. Completion ACK

Final:

```text
BlobVerified
```

means receiver persisted and verified complete blob.

Only then should sender mark transfer complete according to protocol.

---

# 101. Transfer Cancellation

Cancellation must:

```text
stop scheduling
cancel active streams
persist Cancelled
apply staging retention policy
```

User may optionally keep partial data for resume.

---

# 102. Pause

Pause differs from cancel.

Pause retains:

```text
journal
partial chunks
metadata
```

and can resume later.

---

# 103. Deferred Transfer

A transfer may be:

```text
WaitingForWiFi
WaitingForPeer
WaitingForStorage
WaitingForApproval
WaitingForBattery
```

This integrates with Part 03 routing policy.

---

# 104. Retry

Use exponential backoff where appropriate.

Retry at:

```text
transfer/session level
```

not per raw network packet.

Transport handles lower-level retries.

---

# 105. Corrupt Chunk

On hash/authentication failure:

```text
discard chunk
increment failure count
request retransmission
possibly penalize source
```

Repeated corruption may indicate malicious/broken source.

---

# 106. Corrupt Manifest

Reject entire transfer offer if manifest authentication fails.

Do not attempt "best effort" reconstruction.

---

# 107. Multiple Sources

If one source repeatedly sends corrupt chunks:

```text
blacklist source for that transfer
```

while preserving other sources.

---

# 108. Rate Limiting

Per-peer limits:

```text
offers/sec
active transfers
bytes/sec
stored partial bytes
```

Unknown peers get stricter limits.

---

# 109. Abuse Protection

Protect against:

```text
huge fake size
millions of chunks
never-completing transfers
hash mismatch spam
storage exhaustion
offer spam
```

All allocations/storage reservations must be bounded.

---

# 110. Resource Policy

```rust
pub struct FileResourcePolicy {
    pub max_parallel_transfers: usize,
    pub max_parallel_chunks: usize,
    pub max_buffer_bytes: usize,
    pub max_staging_bytes: u64,
    pub max_file_size: u64,
}
```

Applications can tighten limits.

---

# 111. Battery Awareness

Part 13 will deepen this.

File subsystem should already expose knobs:

```text
pause background bulk on low battery
reduce concurrency
prefer Wi-Fi
avoid expensive discovery
```

---

# 112. Memory Pressure

On low memory:

```text
shrink buffer pool
reduce parallel chunks
release previews
pause background transfers
```

Do not lose verified progress.

---

# 113. Thermal Pressure

Large hashing/encryption/transfers can generate heat.

On thermal stress:

```text
reduce concurrency
defer noncritical bulk
```

---

# 114. Hashing Performance

Hash incrementally while streaming.

Do not read file twice if pipeline can safely combine:

```text
read
hash
encrypt
write/send
```

But ensure chosen ciphertext-addressing scheme computes the correct final identity.

---

# 115. Preprocessing Pipeline

Potential outgoing flow:

```text
source file
 ↓
validate metadata
 ↓
optional media derivative generation
 ↓
optional compression
 ↓
encrypt
 ↓
chunk/hash
 ↓
stage blob
 ↓
transfer
```

Exact order must be fixed in protocol spec.

---

# 116. Staging Before Send

For durable resumability, consider preparing encrypted blob locally before transfer.

Pros:

- stable BlobId
- easier retry
- easier multi-recipient send
- no need to reread original after app restart

Cons:

- extra storage

Policy can allow:

```text
stream-only
stage-before-send
```

depending on file size/use.

---

# 117. Stream-Only Mode

For very large data or limited storage:

```text
read source
 ↓
encrypt/chunk
 ↓
send
```

without retaining full encrypted blob.

But resume after source changes becomes harder.

Use only when application semantics permit.

---

# 118. Source Mutation Detection

If transferring from a file path/handle:

```text
file may change
```

Record:

```text
size
mtime if useful
inode/file-id if available
content hash as built
```

If source changes mid-transfer, abort/restart.

Do not silently send mixed versions.

---

# 119. Immutable Source Snapshot

Best production approach where feasible:

```text
copy/import source into managed blob staging
```

Then transfer immutable bytes.

This simplifies correctness.

---

# 120. User File Handles

On Android/iOS, source may be a platform URI/handle.

Platform adapter exposes a stream/descriptor.

Core never assumes POSIX path access.

---

# 121. Android Integration

Kotlin may handle:

```text
Photo Picker
Storage Access Framework
content:// URI
persistable permissions
```

Rust owns:

```text
metadata validation
blob import
encryption
chunking
transfer
```

---

# 122. iOS Integration

Platform adapter handles:

```text
document picker
photo picker
security-scoped URLs where applicable
```

Rust core remains unchanged.

---

# 123. Desktop Integration

Desktop can use native file picker through Dioxus/platform layer.

Blob engine receives:

```text
safe file source handle/path abstraction
```

---

# 124. File Source Abstraction

```rust
pub trait FileSource {
    async fn len(&self) -> Result<u64, FileError>;
    async fn read_at(&self, offset: u64, buf: &mut [u8]) -> Result<usize, FileError>;
}
```

Avoid tying core to `std::fs::File` only.

---

# 125. File Sink Abstraction

For export:

```rust
pub trait FileSink {
    async fn write(&mut self, data: &[u8]) -> Result<(), FileError>;
    async fn commit(self: Box<Self>) -> Result<(), FileError>;
}
```

Platform implementations can handle Android/iOS document destinations.

---

# 126. Export vs Internal Blob

Keep internal encrypted blob separate from user-visible exported file.

Flow:

```text
internal blob
 ↓
decrypt
 ↓
user-selected destination
```

This preserves app sandbox and content integrity.

---

# 127. Local Decryption Cache

Avoid keeping permanent decrypted copies unless explicitly required.

Options:

```text
decrypt on demand
temporary cache
user export
```

Privacy-sensitive default should minimize plaintext persistence.

---

# 128. Thumbnail Cache

Thumbnails may be decrypted and cached.

Use:

```text
bounded LRU
storage pressure eviction
```

Do not let thumbnail cache grow without limit.

---

# 129. Media Decode Isolation

Image/video decode should be separate from blob integrity.

Flow:

```text
verified/decrypted blob
 ↓
media decoder
```

Never decode unverified arbitrary chunks as trusted content unless streaming decoder is hardened and sandboxed appropriately.

---

# 130. Iroh Blobs Integration

If using Iroh's blob capabilities, wrap them behind the platform's blob/transfer abstraction.

Do not let application code depend directly on Iroh-specific blob types.

This allows:

```text
Iroh backend
filesystem backend
future backend
```

without changing message/file APIs.

---

# 131. Internal vs Iroh Content IDs

If Iroh provides its own content hash/address, define mapping clearly:

```text
Platform BlobId
↔
Iroh blob identifier
```

Do not assume external IDs are identical forever.

---

# 132. Direct Transfer Protocol

A custom file control protocol may negotiate:

```text
offer
manifest
missing chunks
pause
resume
cancel
complete
```

Bulk data can use optimized transport-specific streams.

---

# 133. Protocol Extension Integration

Part 01 extension:

```text
files/1
```

Capabilities may include:

```text
chunking
resume
parallel
content addressing
partial read
multi-source
```

---

# 134. Capability Negotiation

Receiver advertises:

```text
max chunk size
parallelism
max object size
resume support
```

Sender chooses common compatible behavior.

---

# 135. Protocol Versioning

Stable:

```text
files/1
```

Major incompatible changes become:

```text
files/2
```

Internal storage schema can evolve independently.

---

# 136. Wire Types vs Domain Types

Do not serialize:

```rust
TransferRecord
```

directly.

Use:

```text
FileOfferV1
ChunkRequestV1
ChunkAckV1
TransferCompleteV1
```

---

# 137. Postcard Usage

Postcard is suitable for compact control metadata.

Do not put large chunk data inside one giant Postcard structure if streaming raw bytes is more efficient.

Use:

```text
small Postcard header
+
streamed chunk bytes
```

---

# 138. Chunk Frame

Conceptually:

```text
ChunkHeader
├── transfer_id
├── blob_id
├── chunk_index
├── size
└── hash/proof metadata

followed by:
chunk bytes
```

Validate before allocating.

---

# 139. Large-Length Safety

Use:

```text
u64 for file sizes
u32 for bounded chunk size
```

but always compare against local limits before conversion/allocation.

Never cast untrusted `u64` directly to `usize`.

---

# 140. Progress Model

```rust
pub struct TransferProgress {
    pub total_bytes: u64,
    pub verified_bytes: u64,
    pub active_bytes_per_sec: Option<u64>,
    pub state: TransferState,
}
```

UI update frequency should be throttled.

---

# 141. ETA

ETA is approximate.

Do not present false precision.

Use:

```text
~2 min
```

or omit when unstable.

---

# 142. UI Independence

Blob subsystem emits state/events.

It does not call:

```text
show progress
open file
display image
```

Dioxus or another UI consumes transfer state.

---

# 143. Notification Integration

Background transfer may emit:

```text
completed
failed
action required
```

through notification abstraction.

Core does not directly call Android notification APIs.

---

# 144. File Acceptance UX

Application may show:

```text
Document.pdf
12.4 MB
From Alice
[Accept] [Decline]
```

Blob engine only provides trusted metadata and state.

---

# 145. Automatic Download Policy

Policy examples:

```text
images on Wi-Fi
voice always
video never
files manual
emergency thumbnail automatically
```

This belongs to application policy integrated with routing/file service.

---

# 146. Data Saver

When enabled:

```text
thumbnail first
defer originals
pause background large files
```

Blob engine exposes dependency relationships between derivative and original.

---

# 147. Storage Classes

```rust
pub enum BlobStorageClass {
    Durable,
    Cached,
    Temporary,
    Relay,
}
```

Each has:

- retention
- quota
- eviction priority

---

# 148. Blob Reference Model

```rust
pub struct BlobReference {
    pub owner_namespace: NamespaceId,
    pub object_id: ObjectId,
    pub blob_id: BlobId,
    pub role: BlobRole,
}
```

Examples:

```text
message attachment
avatar
thumbnail
transfer
backup
```

---

# 149. Reference Integrity

Reference updates should be transactional with corresponding domain state.

Example:

```text
message deleted
 ↓
remove message reference
 ↓
blob becomes GC candidate
```

Do not update reference count loosely outside transactions.

---

# 150. Reference Count Recovery

Reference count is derived metadata.

If inconsistent:

```text
rebuild by scanning authoritative references
```

Do not treat cached count as irreplaceable truth.

---

# 151. GC Mark-and-Sweep Option

For robustness:

```text
mark referenced
 ↓
sweep unreferenced + grace-period
```

can periodically validate counts.

---

# 152. Orphan Detection

Crash may create staged/unreferenced blobs.

Periodic cleanup detects:

```text
no transfer
no reference
older than grace period
```

then removes safely.

---

# 153. Partial Blob Retention

After failed transfer:

```text
keep for resume
```

until:

```text
expiry
user cancel
storage pressure
```

Policy-driven.

---

# 154. Expiry

Transfers and blobs can have:

```text
offer expiry
transfer expiry
relay expiry
cache expiry
```

These are different.

Do not use one timestamp for all semantics.

---

# 155. Message Deletion

Deleting a message may remove a blob reference.

If same blob referenced elsewhere:

```text
do not delete
```

GC handles it.

---

# 156. Disappearing Messages

Blob linked only to disappearing message becomes eligible after:

```text
message expiry
+
retention grace
```

unless user explicitly saved it.

---

# 157. User Save

"Save to device" should create:

```text
user-owned durable reference
```

or export plaintext to user-selected location.

Clarify semantics.

---

# 158. Backup

Backup policy determines whether to include:

```text
all blobs
only referenced durable blobs
thumbnails
relay cache
```

Relay cache should normally be excluded.

---

# 159. Incremental Backup

Content addressing allows efficient:

```text
backup only missing BlobIds
```

Useful for large datasets.

---

# 160. Restore

Restore:

```text
manifest metadata
 ↓
blob bytes
 ↓
hash verification
 ↓
reference rebuild
```

Never trust backup bytes without integrity checks.

---

# 161. Cross-Device Backup

A user's second device may serve as backup source.

The file subsystem treats it as another authorized blob source.

---

# 162. Server/Object Storage Optional Backend

Enterprise/cloud products may use:

```text
S3-compatible
object store
server cache
```

through BlobStore adapter.

The communication SDK should not require cloud object storage.

---

# 163. Server-Side Encryption

If server stores blobs, prefer:

```text
already application-encrypted ciphertext
```

Server need not receive plaintext.

---

# 164. Signed URLs

If a deployment uses object-storage signed URLs, keep that mechanism outside core blob identity.

It is a transport/access adapter.

---

# 165. Public Sharing

If future product supports public share links:

```text
share capability
+
blob identifier
+
decryption/access material
```

must be carefully designed.

Do not turn private BlobId into public URL by default.

---

# 166. Capability-Based File Access

Potential future design:

```text
ReadCapability
WriteCapability
ForwardCapability
```

This integrates with later capability-security architecture.

---

# 167. Forwarding Permission

A file may be:

```text
recipient-only
forwardable
relayable ciphertext
public
```

Do not conflate:

```text
relay allowed
```

with:

```text
recipient may re-share plaintext
```

---

# 168. Relay vs Recipient

Relay:

```text
stores/forwards ciphertext
```

Recipient:

```text
has decryption authorization
```

These are distinct roles.

---

# 169. Multi-Recipient Files

Avoid re-encrypting gigantic file data separately for every recipient if application cryptographic model can safely share a content key via per-recipient encrypted key wrapping.

Conceptually:

```text
one encrypted blob
+
recipient-specific wrapped blob key
```

This can be much more efficient.

---

# 170. Group Attachment

Group message:

```text
one encrypted blob
+
group-authorized key distribution
```

Exact group crypto depends on group architecture.

---

# 171. Key Revocation Limits

If a recipient already obtained:

```text
blob key + ciphertext
```

later revocation cannot erase plaintext they could have copied.

Do not promise remote deletion.

---

# 172. Re-Keying

Future access changes may require:

```text
new blob encryption key
```

and potentially re-encryption.

This is expensive for large files.

Design authorization carefully to avoid unnecessary re-keying.

---

# 173. Encryption Key Caching

Cache active keys only in protected memory/state as necessary.

Do not persist unwrapped keys casually.

---

# 174. Transfer Across Transport Changes

Example:

```text
start on Iroh relay
 ↓
LAN appears
 ↓
pause scheduler
 ↓
new authenticated path
 ↓
resume missing chunks
```

Same TransferId and BlobId.

---

# 175. Bluetooth → Wi-Fi Upgrade

Nearby discovery via BLE can bootstrap:

```text
Wi-Fi Direct/Aware
```

Then file transfer continues on fast path.

No restart.

---

# 176. Wi-Fi → DTN Downgrade

If link disappears:

```text
critical small chunks/metadata may enter DTN
bulk waits
```

according to policy.

---

# 177. Multipath Readiness

Chunk architecture naturally supports Part 12 multipath.

Each path can carry distinct chunks.

No core file format change is needed.

---

# 178. Integrity Under Multipath

Every chunk verifies independently.

Source/path does not affect final blob correctness.

---

# 179. Route Scoring Inputs

File service tells Part 03:

```text
estimated bytes
bulk class
deadline/expiry
metered policy
DTN allowed
multipath allowed
```

Routing returns plan.

---

# 180. Queue Priority

Transfer classes:

```text
EmergencyCritical
InteractiveAttachment
UserRequested
BackgroundSync
Bulk
Relay
```

Part 08 will generalize resource fairness.

---

# 181. Concurrent Transfer Fairness

One 50 GB file must not block:

```text
small image
voice note
SOS attachment
```

Scheduler should use weighted fairness.

---

# 182. Per-Peer Fairness

One peer should not monopolize all upload slots.

---

# 183. Per-Application Fairness

If SDK embedded by multiple product modules, resource quotas can be namespaced.

---

# 184. Performance Metrics

Track:

```text
hash throughput
encryption throughput
disk read/write throughput
network throughput
verified bytes/sec
resume rate
dedup hit rate
GC reclaimed bytes
```

Do not expose content names in telemetry.

---

# 185. Diagnostics

Useful diagnostic snapshot:

```text
active transfers
path
bytes verified
chunk counts
storage usage
cache usage
staging usage
last error
```

---

# 186. Error Model

```rust
pub enum FileError {
    NotFound,
    Unauthorized,
    InvalidManifest,
    HashMismatch,
    DecryptionFailed,
    StorageFull,
    QuotaExceeded,
    UnsupportedCapability,
    SourceChanged,
    Cancelled,
    Expired,
    Transport,
    Storage,
}
```

Use typed errors.

---

# 187. Recoverability Classification

```text
Retryable
Resumable
Permanent
UserActionRequired
```

Example:

```text
network timeout → resumable
hash mismatch → retryable/source penalty
unsupported capability → permanent/fallback
storage full → user action required
```

---

# 188. No `anyhow` in Public File API

Use typed errors in reusable crate boundaries.

`anyhow` is acceptable at application bootstrap/CLI.

---

# 189. Security Invariants

1. Blob bytes are never trusted before integrity/authentication verification.
2. Remote filenames never become storage paths directly.
3. Large declared sizes never cause unchecked allocation.
4. Receiver never marks complete before full verification.
5. Encryption keys do not travel in public blob metadata.
6. Relay nodes do not require plaintext keys.
7. Duplicate chunks are idempotent.
8. Restart does not lose verified progress.
9. Cancellation does not corrupt completed blobs.
10. A malicious peer cannot exceed configured storage/resource quotas.

---

# 190. Testing Layers

Unit:

```text
manifest
chunk math
hashing
reference counting
state machine
```

Integration:

```text
send/receive
pause/resume
crash/restart
multi-device
```

Network:

```text
path switch
loss
duplicates
reordering
```

Security:

```text
tampered chunk
tampered manifest
oversized metadata
path traversal
```

---

# 191. Property Tests

Invariants:

```text
reassembled verified blob equals original
duplicate chunk does not increase verified bytes
completed transfer remains complete after restart
missing chunk bitmap round-trips
GC never deletes pinned referenced blob
```

---

# 192. Fuzz Targets

Fuzz:

```text
manifest parser
file offer
chunk header
resume bitmap
metadata parser
archive-adjacent metadata if supported
```

All allocations bounded.

---

# 193. Crash Injection

Inject crash:

```text
after offer
mid chunk
after chunk write before journal
after journal before ACK
during finalization
during GC
```

Verify safe recovery.

---

# 194. Corruption Tests

Modify:

```text
chunk byte
manifest entry
blob root
metadata
```

Receiver must detect.

---

# 195. Large File Test

Test multi-gigabyte logical files without loading whole file into memory.

Measure:

```text
peak memory
resume latency
throughput
CPU
```

---

# 196. Small File Test

Ensure architecture does not impose huge overhead on:

```text
1 KB
10 KB
100 KB
```

Tiny files may use one chunk.

---

# 197. Bluetooth Test

Test:

```text
small file over BLE
interruption
resume
Wi-Fi upgrade
```

Ensure bounded memory and no restart from zero.

---

# 198. DTN Test

Test:

```text
thumbnail carried
full image deferred
gateway appears
remaining data completes
```

---

# 199. Multi-Source Test

Future:

```text
two sources
same blob
different chunks
```

Receiver verifies one final object.

---

# 200. Storage Pressure Test

Fill storage until:

```text
Elevated
Critical
Full
```

Verify:

- cache eviction
- bulk rejection
- critical preservation
- clear UI state

---

# 201. Suggested Crate Structure

```text
crates/comm-files/
├── src/
│   ├── lib.rs
│   ├── file.rs
│   ├── offer.rs
│   ├── service.rs
│   ├── policy.rs
│   └── error.rs

crates/comm-blob-store/
├── src/
│   ├── lib.rs
│   ├── blob.rs
│   ├── descriptor.rs
│   ├── manifest.rs
│   ├── chunk.rs
│   ├── reader.rs
│   ├── writer.rs
│   ├── refs.rs
│   ├── gc.rs
│   ├── quota.rs
│   └── error.rs

crates/comm-transfer/
├── src/
│   ├── planner.rs
│   ├── scheduler.rs
│   ├── state.rs
│   ├── journal.rs
│   ├── resume.rs
│   ├── progress.rs
│   └── error.rs

crates/comm-transfer-protocol/
├── src/
│   ├── v1/
│   ├── codec.rs
│   └── limits.rs
```

---

# 202. Public API

Keep high-level API simple:

```rust
let transfer = files
    .send_file(peer, source, policy)
    .await?;

transfer.pause().await?;
transfer.resume().await?;
transfer.cancel().await?;
```

Receiving:

```rust
let offer = files.next_offer().await?;
offer.accept(destination_policy).await?;
```

The application does not manipulate chunk protocol directly.

---

# 203. Blob API

Applications needing lower-level access can use:

```rust
let blob = blobs.import(source).await?;
let reader = blobs.open(blob.id()).await?;
```

Keep low-level API separate from ordinary transfer API.

---

# 204. Messaging Integration API

Example:

```rust
let blob = files.import_attachment(source).await?;

messaging
    .send(
        conversation,
        MessageContent::Attachment(blob.reference())
    )
    .await?;
```

Actual file transfer can begin eagerly or on recipient request according to policy.

---

# 205. File-Only API

```rust
files
    .send_file(
        Destination::Device(device_id),
        source,
        FileSendPolicy::default(),
    )
    .await?;
```

No conversation required.

---

# 206. Initial Production Scope

Implement first:

```text
immutable encrypted blobs
fixed-size chunking
chunk hashes
flat manifest
filesystem blob store
SQLite metadata
resumable transfer
pause/cancel
single-source transfer
bounded parallel chunks
routing integration
storage quotas
reference tracking
GC
crash recovery
```

Defer initially:

```text
content-defined chunking
full swarming
hierarchical manifests
public share links
complex cloud object-store adapters
advanced multipath
```

---

# 207. Implementation Phases

## Phase 1 — Blob primitives

```text
BlobId
ChunkHash
BlobDescriptor
Manifest
```

## Phase 2 — Local store

```text
filesystem objects
staging
SQLite metadata
reference tracking
```

## Phase 3 — Encryption and import

```text
per-blob key
chunk encryption
hashing
atomic finalization
```

## Phase 4 — Transfer protocol

```text
offer
accept
manifest
chunk request
ACK
complete
```

## Phase 5 — Resume

```text
journal
bitmap/ranges
restart recovery
```

## Phase 6 — Routing

```text
Iroh/LAN/Bluetooth/DTN
path switching
```

## Phase 7 — Resource management

```text
quotas
GC
storage pressure
backpressure
```

## Phase 8 — Hardening

```text
fuzzing
crash injection
corruption tests
large-file benchmarks
```

---

# 208. Definition of Done

Part 05 is complete when:

- file transfer works independently from messaging
- messages reference blobs instead of embedding file bytes
- files can be imported without loading entire content into memory
- blobs are immutable and integrity-addressed
- privacy-sensitive mode addresses ciphertext rather than plaintext
- chunking supports resume
- individual chunks can be verified
- transfer state survives process death
- incomplete data remains in staging until verified
- completed blob finalization is atomic
- storage paths never use remote filenames directly
- quotas prevent storage exhaustion
- GC never deletes referenced/pinned blobs
- files can pause/resume/cancel
- large transfer can switch transport without starting from zero
- Bluetooth/DTN can carry policy-appropriate subsets
- multi-device destination targeting works
- relay nodes can store ciphertext without decryption keys
- Dioxus/Kotlin/iOS UI layers are not dependencies
- public API is small and typed
- fuzz, corruption, crash, resume, and large-file tests exist

---

# 209. Relationship to Earlier Parts

Part 05 builds on:

```text
01 — Protocol Extension System
02 — Multi-Device Identity
03 — Transport & Routing Policy Engine
04 — Offline Event Log
```

It directly supports:

```text
06 — DTN / Store-Carry-Forward
07 — Capability Negotiation Expansion
08 — Resource Limits & Backpressure
09 — Crash Recovery
10 — Protocol Fuzzing & Test Suite
11 — Relay / Self-Hosted Infrastructure
12 — Multipath Networking
13 — Battery-Aware Scheduling
14 — Proximity Abstraction
16 — Daemon & Headless Runtime
17 — Emergency Priority Architecture
18 — Network Diagnostics & Path Visualization
20 — Embedded Linux Node
23 — External Interoperability Suite
```

---

# 210. Final Principle

The file/blob subsystem should make this possible:

```text
A 5 GB file
can be:
  imported once
  encrypted once
  identified immutably
  transferred in bounded chunks
  paused
  resumed
  switched from relay to LAN
  partially carried by DTN where policy allows
  verified independently
  referenced by a message
  sent without messaging
  cached safely
  garbage-collected when no longer referenced
```

without any application layer needing to understand the raw transfer mechanics.

That is what makes the subsystem reusable, efficient, crash-safe, privacy-preserving, and suitable as a shared binary-data foundation for the entire communication platform.
