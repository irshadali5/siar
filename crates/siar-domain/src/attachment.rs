//! Attachment metadata (plan.md §22, §25). This crate stays
//! infra-free (plan.md §86) so it holds only the *shape* of an
//! attachment reference — the actual hashing/encryption lives in
//! `siar-crypto`, blob transfer in `siar-transport`.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// A conservative allow-list rather than a free-form string (plan.md
/// §68: validate remote-influenced values) — `MediaType::Other` covers
/// anything else without letting an arbitrary string masquerade as a
/// trusted type the UI might render specially.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MediaType {
    ImagePng,
    ImageJpeg,
    ImageWebp,
    AudioOpus,
    VideoMp4,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlobSize(u64);

#[derive(Debug, Error, PartialEq, Eq)]
pub enum BlobSizeError {
    #[error("attachment is {0} bytes, exceeds the {MAX_ATTACHMENT_BYTES}-byte limit")]
    TooLarge(u64),
}

/// plan.md §61's decode-limits discipline applied to attachments: a
/// generous but finite cap (200 MiB) so a malicious/broken peer can't
/// declare an unbounded size and force an unbounded download attempt.
/// Chosen as a Phase-4 starting point, not a permanent constant — tune
/// once there's real usage data (plan.md §90's metrics).
pub const MAX_ATTACHMENT_BYTES: u64 = 200 * 1024 * 1024;

impl BlobSize {
    pub fn parse(bytes: u64) -> Result<Self, BlobSizeError> {
        if bytes > MAX_ATTACHMENT_BYTES {
            return Err(BlobSizeError::TooLarge(bytes));
        }
        Ok(Self(bytes))
    }

    pub fn bytes(&self) -> u64 {
        self.0
    }
}

/// What actually rides inside a message envelope for an attachment —
/// not the blob itself (plan.md §22's diagram: message carries a
/// reference, the blob travels separately over its own protocol).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttachmentReference {
    /// BLAKE3 hash of the *encrypted* blob (siar-crypto's `BlobHash`,
    /// represented here as raw bytes to keep this crate crypto-free).
    pub blob_hash: [u8; 32],
    pub encrypted_size: BlobSize,
    pub media_type: MediaType,
    /// The per-attachment AEAD key (siar-crypto's `AttachmentKey`,
    /// likewise represented as raw bytes here) — travels only inside
    /// the already-encrypted envelope payload (plan.md §23), never
    /// alongside the blob request itself.
    pub attachment_key: [u8; 32],
    pub thumbnail: Option<Box<AttachmentReference>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_reasonable_sizes() {
        assert!(BlobSize::parse(4 * 1024 * 1024).is_ok());
    }

    #[test]
    fn rejects_oversized_attachments() {
        let err = BlobSize::parse(MAX_ATTACHMENT_BYTES + 1).unwrap_err();
        assert_eq!(err, BlobSizeError::TooLarge(MAX_ATTACHMENT_BYTES + 1));
    }
}
