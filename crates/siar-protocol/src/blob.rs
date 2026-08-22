//! Blob-transfer protocol (plan.md §22–25's attachment flow, minus
//! `iroh-blobs` — see `siar-transport`'s module docs for why: the
//! published `iroh-blobs` release doesn't yet support the `iroh` version
//! this workspace is built on).
//!
//! Phase-4 scope: one request, one whole-blob response, no chunking or
//! resumable ranges. `iroh-blobs` would give both of those for free
//! (plan.md §22's "verifiable, resumable transfers") — rolling this by
//! hand means Phase 4 deliberately does *not* promise resumability yet.
//! `BlobRequest`/`BlobResponse` are versioned the same way `v1::Envelope`
//! is, so adding a ranged-request variant later doesn't need a breaking
//! wire change.

use serde::{Deserialize, Serialize};

/// Matches `siar_domain::MAX_ATTACHMENT_BYTES` — kept as an independent
/// constant (same reasoning as `limits.rs`'s `MAX_TEXT_FRAME_BYTES`): the
/// wire limit and the domain validation limit are allowed to diverge
/// later without one silently changing the other.
pub const MAX_BLOB_FRAME_BYTES: usize = 200 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlobRequest {
    pub blob_hash: [u8; 32],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlobResponse {
    Found { ciphertext: Vec<u8> },
    NotFound,
}
