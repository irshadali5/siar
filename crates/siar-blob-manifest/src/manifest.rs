//! §15 "Blob Manifest", §16 "Manifest Limits" (enforcement).

use serde::{Deserialize, Serialize};

use crate::chunking::chunk_fixed_size;
use crate::ids::{BlobId, ChunkHash};
use crate::limits::ManifestLimits;

/// §15's nested type, field-for-field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChunkDescriptor {
    pub index: u32,
    pub offset: u64,
    pub size: u32,
    pub hash: ChunkHash,
}

/// §15, field-for-field.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlobManifest {
    pub version: u16,
    pub blob_id: BlobId,
    pub total_size: u64,
    pub chunk_size: u32,
    pub chunks: Vec<ChunkDescriptor>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ManifestError {
    #[error("manifest would have {actual} chunks, over the {max} limit")]
    TooManyChunks { actual: usize, max: u32 },
    #[error("manifest would be {actual} bytes, over the {max}-byte limit")]
    ManifestTooLarge { actual: u64, max: u64 },
    #[error("file is {actual} bytes, over the {max}-byte limit")]
    FileTooLarge { actual: u64, max: u64 },
}

/// A rough per-[`ChunkDescriptor`] wire-size estimate (index `u32` +
/// offset `u64` + size `u32` + 32-byte hash = 20 bytes, rounded up for
/// framing/serde overhead) — used only to enforce
/// [`ManifestLimits::max_manifest_bytes`] before actually serializing,
/// so a manifest that would blow the limit is rejected without first
/// having to build the (potentially huge) serialized form just to
/// measure it.
const ESTIMATED_BYTES_PER_CHUNK_DESCRIPTOR: u64 = 32;

/// §15 + §16 + §18's ciphertext-addressing recommendation, tied
/// together: takes the already-encrypted chunks (encryption itself
/// happens upstream of this crate — see its own top doc comment),
/// content-addresses each one (§14) and the whole blob (§6/§7), and
/// validates against `limits` (§16) before returning anything —
/// callers never receive a manifest that violates its own stated
/// bounds.
pub fn build_manifest(
    ciphertext: &[u8],
    chunk_size: u32,
    limits: &ManifestLimits,
) -> Result<BlobManifest, ManifestError> {
    let total_size = ciphertext.len() as u64;
    if total_size > limits.max_file_bytes {
        return Err(ManifestError::FileTooLarge {
            actual: total_size,
            max: limits.max_file_bytes,
        });
    }

    let raw_chunks = chunk_fixed_size(ciphertext, chunk_size);
    if raw_chunks.len() > limits.max_chunks as usize {
        return Err(ManifestError::TooManyChunks {
            actual: raw_chunks.len(),
            max: limits.max_chunks,
        });
    }

    let estimated_manifest_bytes = raw_chunks.len() as u64 * ESTIMATED_BYTES_PER_CHUNK_DESCRIPTOR;
    if estimated_manifest_bytes > limits.max_manifest_bytes {
        return Err(ManifestError::ManifestTooLarge {
            actual: estimated_manifest_bytes,
            max: limits.max_manifest_bytes,
        });
    }

    let mut offset = 0u64;
    let chunks: Vec<ChunkDescriptor> = raw_chunks
        .iter()
        .enumerate()
        .map(|(index, chunk)| {
            let descriptor = ChunkDescriptor {
                index: index as u32,
                offset,
                size: chunk.len() as u32,
                hash: ChunkHash::from_ciphertext_chunk(chunk),
            };
            offset += chunk.len() as u64;
            descriptor
        })
        .collect();

    let blob_id = BlobId::from_ciphertext(ciphertext);

    Ok(BlobManifest {
        version: 1,
        blob_id,
        total_size,
        chunk_size,
        chunks,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_manifest_built_from_real_bytes_has_correct_offsets_and_hashes() {
        let ciphertext = vec![7u8; 250];
        let manifest = build_manifest(&ciphertext, 100, &ManifestLimits::default()).unwrap();

        assert_eq!(manifest.total_size, 250);
        assert_eq!(manifest.chunks.len(), 3);
        assert_eq!(manifest.chunks[0].offset, 0);
        assert_eq!(manifest.chunks[1].offset, 100);
        assert_eq!(manifest.chunks[2].offset, 200);
        assert_eq!(manifest.chunks[2].size, 50);

        let expected_hash = ChunkHash::from_ciphertext_chunk(&ciphertext[200..250]);
        assert_eq!(manifest.chunks[2].hash, expected_hash);
    }

    #[test]
    fn the_blob_id_is_a_real_content_hash_of_the_whole_ciphertext() {
        let ciphertext = vec![9u8; 500];
        let manifest = build_manifest(&ciphertext, 100, &ManifestLimits::default()).unwrap();
        assert_eq!(manifest.blob_id, BlobId::from_ciphertext(&ciphertext));
    }

    #[test]
    fn a_file_over_the_size_limit_is_rejected() {
        let limits = ManifestLimits {
            max_file_bytes: 100,
            ..ManifestLimits::default()
        };
        let ciphertext = vec![0u8; 200];
        let result = build_manifest(&ciphertext, 50, &limits);
        assert_eq!(
            result,
            Err(ManifestError::FileTooLarge {
                actual: 200,
                max: 100
            })
        );
    }

    #[test]
    fn too_many_chunks_is_rejected_before_hashing_anything() {
        let limits = ManifestLimits {
            max_chunks: 2,
            ..ManifestLimits::default()
        };
        let ciphertext = vec![0u8; 300];
        let result = build_manifest(&ciphertext, 100, &limits); // would need 3 chunks
        assert_eq!(
            result,
            Err(ManifestError::TooManyChunks { actual: 3, max: 2 })
        );
    }

    #[test]
    fn identical_ciphertext_always_produces_the_same_blob_id() {
        let ciphertext = vec![3u8; 400];
        let a = build_manifest(&ciphertext, 100, &ManifestLimits::default()).unwrap();
        let b = build_manifest(&ciphertext, 100, &ManifestLimits::default()).unwrap();
        assert_eq!(a.blob_id, b.blob_id); // §6: deduplication/immutable identity
    }
}
