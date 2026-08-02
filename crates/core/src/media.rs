//! Image codec entry point for DPs (display pictures / avatars) and status
//! images. One rule drives this whole module: **decode whatever the local
//! user picked, always store/transmit PNG.** A peer receiving a DP or a
//! status image never needs to support a codec just because the sender's
//! photo picker happened to produce that format — they only ever have to
//! decode PNG, which every `image`-crate build (this one included) always
//! can. This is the same "canonicalize at the edge" shape as
//! `net::transfer`'s adaptive compression, just for images instead of
//! arbitrary files.
//!
//! Formats accepted as *input* (what a local file picker might hand us):
//! JPEG, PNG (both via the pure-Rust `image` crate — no system libjpeg/
//! libpng needed), and JPEG XL (via `jxl-oxide`, decode-only — see
//! `Cargo.toml`'s dependency comment for why there's no JXL encode path).
//! Format is sniffed from the bytes themselves, not trusted from a file
//! extension or claimed MIME type — a peer's file could claim anything.

use anyhow::{Context, Result};
use image::ImageFormat;

/// Cap on the *decoded* pixel buffer, checked before any resize work —
/// protects against a maliciously-crafted tiny file that decompresses to
/// an enormous canvas (a decompression-bomb: JPEG/PNG/JXL can all express
/// huge dimensions in a few header bytes). 8192×8192 RGBA is 256 MiB,
/// already generous for a DP or status image and comfortably below
/// anything that would meaningfully threaten memory on a desktop.
const MAX_DIMENSION: u32 = 8192;

/// Long edge a DP gets downscaled to before re-encoding — no point
/// carrying a 12-megapixel phone photo around as a profile picture that's
/// displayed at a few dozen pixels across. Status images get to keep more
/// detail (`STATUS_MAX_EDGE`) since they're viewed full-screen.
const AVATAR_MAX_EDGE: u32 = 512;
const STATUS_MAX_EDGE: u32 = 1920;

pub struct DecodedImage {
    pub width: u32,
    pub height: u32,
    /// Canonical PNG bytes — this is what actually gets stored locally
    /// and sent to peers (via `iroh-blobs`, same as any other file; see
    /// `net::transfer`). Never re-decode a peer's original bytes on the
    /// receive side for this reason — there might not have been any if
    /// the sender's format was JXL, and there's no need to care either
    /// way once this canonicalization has already happened at the source.
    pub png_bytes: Vec<u8>,
}

/// Decode `input` (sniffed, not trusted from any extension/MIME claim),
/// downscale to fit within `max_edge` on the long side (aspect-preserving,
/// only ever shrinks — a smaller source image is left as-is rather than
/// upscaled), and re-encode as PNG.
fn decode_and_canonicalize(input: &[u8], max_edge: u32) -> Result<DecodedImage> {
    let dynamic = decode_any(input)?;
    let (w, h) = (dynamic.width(), dynamic.height());
    anyhow::ensure!(
        w <= MAX_DIMENSION && h <= MAX_DIMENSION,
        "image is {w}x{h}, larger than the {MAX_DIMENSION}x{MAX_DIMENSION} limit"
    );

    let resized = if w.max(h) > max_edge {
        // Lanczos3: noticeably better quality than the crate's cheaper
        // filters for a downscale this aggressive (a 4000px photo down to
        // a 512px avatar) — worth the extra CPU for something computed
        // once at DP-set time, not per frame the way `net::calls::audio`'s
        // resampler is.
        dynamic.resize(max_edge, max_edge, image::imageops::FilterType::Lanczos3)
    } else {
        dynamic
    };

    let mut png_bytes = Vec::new();
    resized
        .write_to(&mut std::io::Cursor::new(&mut png_bytes), ImageFormat::Png)
        .context("re-encoding image as PNG")?;

    Ok(DecodedImage {
        width: resized.width(),
        height: resized.height(),
        png_bytes,
    })
}

/// Decode a display picture: downscaled to `AVATAR_MAX_EDGE`.
pub fn decode_avatar(input: &[u8]) -> Result<DecodedImage> {
    decode_and_canonicalize(input, AVATAR_MAX_EDGE)
}

/// Decode a status image: downscaled to `STATUS_MAX_EDGE` (status is
/// viewed full-screen, so it keeps more detail than an avatar does).
pub fn decode_status_image(input: &[u8]) -> Result<DecodedImage> {
    decode_and_canonicalize(input, STATUS_MAX_EDGE)
}

/// Sniff `input`'s format from its own bytes (magic-number based, via
/// `image::guess_format` for JPEG/PNG) and decode with whichever codec
/// applies. JPEG XL isn't in `image`'s own format registry (that's the
/// whole reason `jxl-oxide` is a separate dependency), so it's checked
/// first via its own magic-number signature — the raw codestream
/// signature (`FF 0A`) and the more common ISOBMFF-container signature
/// (`00 00 00 0C 4A 58 4C 20 0D 0A 87 0A`) both start unambiguously
/// differently from any JPEG/PNG magic, so trying JXL first can't
/// misidentify a real JPEG/PNG.
fn decode_any(input: &[u8]) -> Result<image::DynamicImage> {
    const JXL_CODESTREAM_SIG: &[u8] = &[0xFF, 0x0A];
    const JXL_CONTAINER_SIG: &[u8] = &[0x00, 0x00, 0x00, 0x0C, 0x4A, 0x58, 0x4C, 0x20];

    if input.starts_with(JXL_CODESTREAM_SIG) || input.starts_with(JXL_CONTAINER_SIG) {
        return decode_jxl(input);
    }

    let format = image::guess_format(input).context("unrecognized image format")?;
    image::load_from_memory_with_format(input, format)
        .with_context(|| format!("decoding {format:?} image"))
}

/// Written against `jxl-oxide` 0.12's confirmed public API
/// (`docs.rs/jxl-oxide/latest/jxl_oxide/struct.Render.html` /
/// `struct.FrameBuffer.html`) — `Render` itself exposes no public field or
/// method called `image`; the accessor is `image_all_channels()`, which
/// returns a `FrameBuffer` carrying its own `width()`/`height()`/
/// `channels()` and an interleaved `f32` sample buffer via `buf()`.
/// Reading `channels()` directly like this is also more robust than
/// branching on `JxlImage::pixel_format()` would be: it's the actual
/// layout of the buffer already handed to us, not a separate declaration
/// that has to be trusted to match.
fn decode_jxl(input: &[u8]) -> Result<image::DynamicImage> {
    use jxl_oxide::JxlImage;

    let jxl_image = JxlImage::builder()
        .read(std::io::Cursor::new(input))
        // `.context()` doesn't apply here: `read` returns
        // `Result<_, Box<dyn Error + Send + Sync>>`, and anyhow's
        // `Context` trait needs the error type to itself implement
        // `std::error::Error` — a boxed trait object doesn't, so the
        // blanket impl doesn't fire. Map it by hand instead of pulling in
        // anything else just for this one call.
        .map_err(|e| anyhow::anyhow!("parsing JPEG XL header: {e}"))?;

    let render = jxl_image
        .render_frame(0)
        // Same reason `.context()` doesn't work on `.read()` above: this
        // also returns `Result<_, Box<dyn Error + Send + Sync>>`.
        .map_err(|e| anyhow::anyhow!("decoding JPEG XL frame: {e}"))?;

    // All extra channels included, orientation already applied — exactly
    // the "one buffer, already right-side-up" shape `decode_and_canonicalize`
    // above wants; `Render::stream()` would need driving through
    // `ImageStream`'s own read interface for no benefit here.
    let fb = render.image_all_channels();
    let (width, height, channels) = (fb.width() as u32, fb.height() as u32, fb.channels());
    let samples = fb.buf(); // interleaved f32, len = width*height*channels

    let mut rgba = Vec::with_capacity(width as usize * height as usize * 4);
    #[inline(always)]
    fn to_u8(v: f32) -> u8 {
        (v.clamp(0.0, 1.0) * 255.0) as u8
    }
    for px in samples.chunks_exact(channels) {
        match channels {
            1 => {
                let g = to_u8(px[0]);
                rgba.extend_from_slice(&[g, g, g, 255]);
            }
            2 => {
                let g = to_u8(px[0]);
                rgba.extend_from_slice(&[g, g, g, to_u8(px[1])]);
            }
            3 => rgba.extend_from_slice(&[to_u8(px[0]), to_u8(px[1]), to_u8(px[2]), 255]),
            4 => rgba.extend_from_slice(&[to_u8(px[0]), to_u8(px[1]), to_u8(px[2]), to_u8(px[3])]),
            // A JPEG XL with more than 4 interleaved channels (multiple
            // spot colors, depth/alpha-adjacent extra channels beyond a
            // plain alpha, etc.) isn't a shape a photo-style DP/status
            // image should ever produce; fail clearly rather than
            // silently misreading the layout.
            other => anyhow::bail!("unsupported JPEG XL channel count: {other}"),
        }
    }

    image::RgbaImage::from_raw(width, height, rgba)
        .map(image::DynamicImage::ImageRgba8)
        .context("JPEG XL frame buffer didn't match its own reported dimensions")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_garbage_bytes() {
        assert!(decode_avatar(b"not an image").is_err());
    }

    #[test]
    fn round_trips_a_tiny_png() {
        // 1x1 red pixel PNG — small enough to inline here rather than
        // needing a fixture file.
        let mut png_bytes = Vec::new();
        let buf = image::RgbaImage::from_pixel(4, 4, image::Rgba([255, 0, 0, 255]));
        image::DynamicImage::ImageRgba8(buf)
            .write_to(&mut std::io::Cursor::new(&mut png_bytes), ImageFormat::Png)
            .unwrap();
        let decoded = decode_avatar(&png_bytes).unwrap();
        assert_eq!((decoded.width, decoded.height), (4, 4));
    }
}
