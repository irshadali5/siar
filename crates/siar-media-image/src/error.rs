use thiserror::Error;

#[derive(Debug, Error)]
pub enum ImageError {
    #[error("could not determine image format from the byte header")]
    UnrecognizedFormat,

    /// codecs2.md's image policy lists AVIF as "optional" and HEIC as a
    /// platform-integration concern — both are recognized by magic
    /// bytes (so callers get an honest error, not a silent
    /// misdetection) but neither is decoded here yet.
    #[error("{0:?} is recognized but not yet supported by this crate")]
    UnsupportedFormat(super::ImageFormat),

    /// plan.md §61's decode-limits discipline: never decode an
    /// arbitrary remote-declared size. Mirrors
    /// `siar_domain::attachment::MAX_ATTACHMENT_BYTES` in spirit but
    /// applied to *decoded pixel* dimensions, which is the actual
    /// memory-blowup vector for images (a tiny compressed file can
    /// still decompress into a huge pixel buffer — "decompression
    /// bomb").
    #[error("declared image dimensions {width}x{height} exceed the {max}px-per-side decode limit")]
    DimensionsTooLarge { width: u32, height: u32, max: u32 },

    #[error("underlying codec error: {0}")]
    Codec(String),
}
