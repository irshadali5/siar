#![forbid(unsafe_code)]

//! siar-blob-manifest: a first slice of "Part 05 — Robust File/Blob
//! Subsystem Architecture" (the fifth of the architecture documents
//! supplied so far). This workspace already has attachment *metadata*
//! shapes (`siar_domain::attachment::{AttachmentReference, MediaType,
//! BlobSize}`) and a working single-shot 1:1 attachment send/fetch
//! path (`siar-messaging::MessageService::send_attachment`/
//! `fetch_attachment`, wired all the way to `apps/android`) — but
//! nothing implementing §15's actual chunked manifest, §26's transfer
//! state machine, or §29's resume bitmap. This crate is that.
//!
//! ## Scope: §207 "Implementation Phases" 1, plus real slices of 2/3/5
//!
//! - [`ids`] — §5-8 `BlobId`/`ChunkHash`/`LogicalAttachmentId`/
//!   `ManifestId`, §19 `BlobEncryptionKey`'s type shape.
//!   [`ids::BlobId::from_ciphertext`]/[`ids::ChunkHash::from_ciphertext_chunk`]
//!   make §7's own recommended choice ("encrypt first, then
//!   content-address the ciphertext") the *only* way to construct
//!   these types, not just a documented convention a caller could
//!   ignore.
//! - [`descriptor`] — §9 `BlobDescriptor`, §10 `FileMetadata` (with a
//!   real bounded [`descriptor::FileName`], not a bare `String`), §18/
//!   §21 `EncryptionDescriptor`/`EncryptionAlgorithm` (naming
//!   `ChaCha20Poly1305` specifically — this workspace's already-
//!   established AEAD choice, not a new one introduced here).
//! - [`chunking`] — §11-13: real fixed-size chunking (§12's own v1
//!   recommendation over content-defined chunking) plus a size-class
//!   heuristic for §13's small/medium/large chunk-size guidance.
//! - [`manifest`] — §15 `BlobManifest`/`ChunkDescriptor` PLUS §16's
//!   limits actually enforced at construction time
//!   ([`manifest::build_manifest`] rejects an oversized file/manifest/
//!   chunk-count before allocating anything for it), not just typed
//!   and left unchecked.
//! - [`verify`] — §14's own stated benefit made real: per-chunk hash
//!   verification before a whole blob is complete, plus whole-blob
//!   verification that catches a manifest whose `blob_id` was forged
//!   independent of its per-chunk hashes.
//! - [`resume`] — §29 `ResumeBitmap` + §30 range-based resume
//!   (`missing_ranges` returns contiguous `[start, end)` runs, not one
//!   entry per missing chunk).
//! - [`transfer_state`] — §26's named states as a real state machine
//!   (`TransferState::transition`) that rejects illegal transitions
//!   instead of a bare enum a caller could set to anything.
//!
//! Every module is covered by tests exercising real bytes/hashes/state
//! transitions — including a tamper-detection test for both per-chunk
//! and whole-blob verification, not just the happy path.
//!
//! ## What's explicitly NOT here
//!
//! - **No actual encryption.** [`descriptor::EncryptionDescriptor`]
//!   names the algorithm; nothing in this crate calls
//!   `chacha20poly1305` to actually encrypt/decrypt a chunk. §207 Phase
//!   3 ("Encryption and import") is a real, separate integration with
//!   `siar-crypto`, not attempted here.
//! - **No local store.** §207 Phase 2: no filesystem blob storage, no
//!   staging, no SQLite metadata, no reference tracking/GC — this
//!   crate produces and verifies manifests as pure values; persisting
//!   them is a `siar-storage` integration this crate doesn't depend on.
//! - **No transfer protocol wire messages.** §207 Phase 4 (offer/
//!   accept/manifest/chunk-request/ACK/complete as actual
//!   `siar-protocol` envelope kinds) — [`transfer_state::TransferState`]
//!   models the state machine an implementation of that protocol would
//!   drive, but doesn't define the wire messages themselves.
//! - **No transport/routing integration.** §207 Phase 6 — nothing here
//!   touches `siar-transport`, `siar-routing`, or the new
//!   `siar-routing-policy` from this same workspace, though a real
//!   integration would plug `RoutePlan`/candidate-path selection in
//!   exactly where a transfer's `TransferState::InProgress` chunk
//!   requests get dispatched.
//! - **No resource management.** §207 Phase 7 — quotas, storage
//!   pressure, backpressure — not attempted (`siar-routing-policy`'s
//!   own `RetryPolicy`/backoff exists in this workspace but isn't
//!   wired to this crate's transfer state machine either).
//! - **Progressive images/video (§33-34), manifest hierarchy for huge
//!   files (§17, explicitly "not required initially" per the spec's own
//!   text), streaming/low-copy I/O and buffer pools (§37-39),
//!   parallelism/adaptive concurrency (§41-43), file offer/auto-accept/
//!   authorization policy (§44-46), quotas/sparse files/staging
//!   (§47-50), and everything from roughly §51 onward** — a genuinely
//!   small slice of a 208-section document.

pub mod chunking;
pub mod descriptor;
pub mod ids;
pub mod limits;
pub mod manifest;
pub mod resume;
pub mod transfer_state;
pub mod verify;

pub use chunking::{chunk_fixed_size, ChunkSizeClass};
pub use descriptor::{BlobDescriptor, ChunkingDescriptor, EncryptionAlgorithm, EncryptionDescriptor, FileMetadata, FileName, FileNameTooLong};
pub use ids::{BlobEncryptionKey, BlobId, ChunkHash, LogicalAttachmentId, ManifestId};
pub use limits::ManifestLimits;
pub use manifest::{build_manifest, BlobManifest, ChunkDescriptor, ManifestError};
pub use resume::ResumeBitmap;
pub use transfer_state::{InvalidTransition, TransferEvent, TransferState};
pub use verify::{verify_chunk, verify_complete_blob};
