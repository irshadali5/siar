//! §11 "Chunking", §12 "Fixed vs Content-Defined Chunking", §13 "Chunk
//! Size Policy".

/// §12: "For v1, prefer fixed-size chunking with a configurable chunk
/// size." Content-defined chunking (§12's other option) is
/// deliberately not implemented — the spec's own v1 recommendation is
/// this one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkSizeClass {
    /// §13: "small file: single chunk."
    Small,
    /// §13: "medium: 256 KiB – 1 MiB chunks."
    Medium,
    /// §13: "large: 1 MiB – 4 MiB chunks."
    Large,
}

impl ChunkSizeClass {
    /// §13: "Do not hard-code one size for every transport" — this
    /// crate has no transport awareness (see its own top doc comment),
    /// so it classifies by file size only; a caller with transport
    /// context (MTU, link speed) may reasonably override the returned
    /// value rather than always accepting it.
    pub fn for_file_size(total_size: u64) -> Self {
        const MEDIUM_THRESHOLD: u64 = 1024 * 1024; // 1 MiB
        const LARGE_THRESHOLD: u64 = 16 * 1024 * 1024; // 16 MiB
        if total_size <= MEDIUM_THRESHOLD {
            Self::Small
        } else if total_size <= LARGE_THRESHOLD {
            Self::Medium
        } else {
            Self::Large
        }
    }

    /// A concrete chunk size within this class's own §13 range.
    pub fn default_chunk_size(self, total_size: u64) -> u32 {
        match self {
            Self::Small => total_size.max(1).min(u32::MAX as u64) as u32, // one chunk = the whole (small) file
            Self::Medium => 512 * 1024,
            Self::Large => 4 * 1024 * 1024,
        }
    }
}

/// Splits `data` into fixed-size slices — the mechanical half of §11's
/// chunking; [`crate::manifest::build_manifest`] is what turns the
/// result into hashed, offset-tracked [`crate::manifest::ChunkDescriptor`]s.
/// The last chunk is whatever remains (may be shorter than
/// `chunk_size`) — not padded, since padding would corrupt content
/// addressing (§6) for no benefit.
pub fn chunk_fixed_size(data: &[u8], chunk_size: u32) -> Vec<&[u8]> {
    if chunk_size == 0 || data.is_empty() {
        return if data.is_empty() { vec![] } else { vec![data] };
    }
    data.chunks(chunk_size as usize).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_small_file_becomes_one_chunk() {
        let data = vec![0u8; 100];
        assert_eq!(
            ChunkSizeClass::for_file_size(100).default_chunk_size(100),
            100
        );
        let chunks = chunk_fixed_size(&data, 100);
        assert_eq!(chunks.len(), 1);
    }

    #[test]
    fn fixed_chunking_splits_evenly_with_a_short_last_chunk() {
        let data = vec![0u8; 250];
        let chunks = chunk_fixed_size(&data, 100);
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].len(), 100);
        assert_eq!(chunks[1].len(), 100);
        assert_eq!(chunks[2].len(), 50); // not padded
    }

    #[test]
    fn size_classification_matches_spec_examples() {
        assert_eq!(
            ChunkSizeClass::for_file_size(500_000),
            ChunkSizeClass::Small
        );
        assert_eq!(
            ChunkSizeClass::for_file_size(4_000_000),
            ChunkSizeClass::Medium
        );
        assert_eq!(
            ChunkSizeClass::for_file_size(50_000_000),
            ChunkSizeClass::Large
        );
    }
}
