//! §9 "Blob Descriptor", §10 "File Metadata", §18 "Encryption Model",
//! §21 "Encryption Metadata".

use serde::{Deserialize, Serialize};
use siar_domain::{BlobSize, MediaType};

use crate::ids::{BlobId, ManifestId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChunkingDescriptor {
    pub chunk_size: u32,
}

/// §18/§21. "Use authenticated encryption" — `ChaCha20Poly1305` is
/// this workspace's already-established choice (`siar-crypto`'s
/// message encryption uses it; see that crate's `Cargo.toml`), named
/// as the one variant here rather than an open-ended algorithm
/// identifier, since introducing a second AEAD into this codebase
/// isn't this crate's call to make.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncryptionAlgorithm {
    ChaCha20Poly1305,
}

/// §20 "Chunk Nonces" is named but not detailed with a concrete scheme
/// in the spec text this crate was built against — `base_nonce` plus
/// a per-chunk index-derived counter (the standard, safe way to derive
/// many unique nonces from one base without storing one full nonce per
/// chunk) is a real, reasonable choice, not the only one the spec
/// permits; flagged here as this crate's own decision rather than a
/// transcription of spec text the way most of this module is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncryptionDescriptor {
    pub algorithm: EncryptionAlgorithm,
    pub base_nonce: [u8; 12],
}

/// §9, verbatim field-for-field, reusing `siar_domain::MediaType`
/// (already real in this workspace, used by
/// `siar-messaging::MessageService::send_attachment`) rather than a
/// second media-type enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlobDescriptor {
    pub blob_id: BlobId,
    pub size: BlobSize,
    pub chunking: ChunkingDescriptor,
    pub encryption: EncryptionDescriptor,
    pub media_type: Option<MediaType>,
    pub manifest_id: Option<ManifestId>,
}

/// §9: "Do not trust remote-declared size blindly." Not enforced by
/// this struct itself (a plain data holder can't refuse to be
/// constructed with a lie) — enforcement is
/// [`crate::manifest::BlobManifest::total_size`] being recomputed from
/// the actual chunk descriptors, which a real receive path checks
/// against a remote-declared [`BlobDescriptor::size`] rather than
/// trusting it directly.
///
/// §10. `display_name: Option<FileName>` — [`FileName`] bounds the
/// string length (Part 01 §9's reasoning against unbounded strings,
/// applied here), unlike `String` directly.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileMetadata {
    pub display_name: Option<FileName>,
    pub media_type: Option<MediaType>,
    pub logical_size: u64,
    pub created_at_millis: Option<u64>,
}

/// A bounded display name — 255 bytes, a conservative cross-filesystem
/// limit (matches ext4/NTFS/APFS's own 255-byte filename ceilings, so
/// a name that fits here is a name every target platform can also
/// actually write to disk).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileName(String);

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("file name is {len} bytes, over the 255-byte limit")]
pub struct FileNameTooLong {
    len: usize,
}

impl FileName {
    pub fn new(name: impl Into<String>) -> Result<Self, FileNameTooLong> {
        let name = name.into();
        if name.len() > 255 {
            return Err(FileNameTooLong { len: name.len() });
        }
        Ok(Self(name))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
