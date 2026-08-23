//! §7 "Bundle Payload". A real cross-crate integration, not placeholder
//! types: `Blob`/`Chunk` reuse `siar_blob_manifest::BlobId` (this same
//! session's Part 05 crate) and `Event` reuses `siar_event_log::EventId`
//! (this same session's Part 04 crate) — exactly the kind of "large
//! content referenced by blob/chunk identity" §6 asks for, using the
//! actual types this workspace now has for that rather than inventing
//! parallel ones a third time.

use serde::{Deserialize, Serialize};
use siar_blob_manifest::BlobId;
use siar_event_log::EventId;

/// §7: "Inline payloads must be strictly bounded." Not enforced by
/// this enum itself (see [`MAX_INLINE_PAYLOAD_BYTES`] and
/// [`PayloadReference::validate`] for the actual check) — a bare data
/// type can't refuse to be constructed with an oversized `Vec`, so
/// validation is a separate, explicit step a caller must run, the same
/// posture `siar_blob_manifest::manifest::build_manifest` already takes
/// toward its own size limits.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PayloadReference {
    Inline(Vec<u8>),
    Blob(BlobId),
    Chunk { blob: BlobId, chunk_index: u32 },
    Event(EventId),
}

/// A conservative bound for `Inline` — large enough for a short text
/// message's ciphertext, small enough that a bundle carrying it stays
/// cheap to spray to several encounter peers at once (§23). Not a spec
/// constant (the source text states the requirement, not a number);
/// flagged as this crate's own reasonable choice, same as
/// `siar_blob_manifest::ManifestLimits::default`'s own equivalent note.
pub const MAX_INLINE_PAYLOAD_BYTES: usize = 16 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("inline payload is {actual} bytes, over the {max}-byte limit — use Blob/Chunk instead")]
pub struct InlinePayloadTooLarge {
    pub actual: usize,
    pub max: usize,
}

impl PayloadReference {
    pub fn validate(&self) -> Result<(), InlinePayloadTooLarge> {
        if let Self::Inline(bytes) = self {
            if bytes.len() > MAX_INLINE_PAYLOAD_BYTES {
                return Err(InlinePayloadTooLarge { actual: bytes.len(), max: MAX_INLINE_PAYLOAD_BYTES });
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_inline_payload_within_the_bound_is_valid() {
        let payload = PayloadReference::Inline(vec![0u8; 100]);
        assert!(payload.validate().is_ok());
    }

    #[test]
    fn an_oversized_inline_payload_is_rejected() {
        let payload = PayloadReference::Inline(vec![0u8; MAX_INLINE_PAYLOAD_BYTES + 1]);
        assert_eq!(
            payload.validate(),
            Err(InlinePayloadTooLarge { actual: MAX_INLINE_PAYLOAD_BYTES + 1, max: MAX_INLINE_PAYLOAD_BYTES })
        );
    }

    #[test]
    fn a_blob_reference_has_no_size_bound_to_validate() {
        let ciphertext = vec![1u8; 1_000_000];
        let payload = PayloadReference::Blob(BlobId::from_ciphertext(&ciphertext));
        assert!(payload.validate().is_ok());
    }
}
