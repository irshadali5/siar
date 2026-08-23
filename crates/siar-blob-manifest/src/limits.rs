//! §16 "Manifest Limits".

/// §16: "Do not allow a tiny network frame to declare millions of
/// chunks and force huge allocation." Values are this crate's own
/// reasonable defaults — the spec names the three fields to bound but
/// not specific numbers, so [`ManifestLimits::default`] is a real,
/// working starting point (documented as such), not a transcribed
/// spec constant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ManifestLimits {
    pub max_chunks: u32,
    pub max_manifest_bytes: u64,
    pub max_file_bytes: u64,
}

impl Default for ManifestLimits {
    fn default() -> Self {
        Self {
            max_chunks: 1_000_000,
            // ~24 bytes/chunk descriptor (index u32 + offset u64 + size
            // u32 + 32-byte hash) * max_chunks, rounded up generously.
            max_manifest_bytes: 64 * 1024 * 1024,
            max_file_bytes: 10 * 1024 * 1024 * 1024, // 10 GiB
        }
    }
}
