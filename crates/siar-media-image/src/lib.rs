//! Still-image codec layer (codecs2.md's "Are image codecs required
//! software/hardware?" section).
//!
//! Deliberately separate from `siar-media-core` (video calls) and
//! `siar-media-audio` (voice): codecs2.md draws a hard line between the
//! two — realtime video needs peer-to-peer hardware capability
//! negotiation (`siar-media-core::negotiation`), but "image formats do
//! not need peer-to-peer codec negotiation. If Alice sends a JPEG, Bob
//! receives that JPEG file." So this crate has no negotiation, no
//! capability struct, no hardware path at all — just:
//!
//! ```text
//! incoming bytes -> format detection -> safe software decoder
//!     -> RGBA image -> thumbnail generator -> preview generator
//! ```
//!
//! v1 policy per codecs2.md: JPEG + PNG + WebP decode/encode (all
//! required/recommended, all pure-Rust via the `image` crate). AVIF and
//! HEIC are explicitly out of scope for this crate — codecs2.md calls
//! AVIF "optional" and HEIC a platform-integration concern, and pulling
//! either in blind (no compiler here to verify the dependency chain)
//! would be guessing rather than building, the same reasoning that kept
//! openmls/Wi-Fi Aware/Bluetooth Classic out of the last passes.

mod decode;
mod error;
mod format;
mod thumbnail;

pub use decode::{decode_image, DecodedImage};
pub use error::ImageError;
pub use format::ImageFormat;
pub use thumbnail::{generate_preview, generate_thumbnail, EncodedImage, PREVIEW_MAX_DIMENSION, THUMBNAIL_MAX_DIMENSION};
