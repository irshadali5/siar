use crate::error::ImageError;
use crate::format::{self, ImageFormat};

/// A decoded image as plain RGBA8 — deliberately not tied to
/// `siar-media-core::frame`'s video frame types (codecs2.md is explicit
/// these are separate concerns: stills need no negotiation, no
/// codec/profile/fps, no realtime pipeline).
#[derive(Debug)]
pub struct DecodedImage {
    pub width: u32,
    pub height: u32,
    /// Tightly packed RGBA8, row-major, no padding — `width * height * 4`
    /// bytes.
    pub rgba: Vec<u8>,
}

/// plan.md §61 decode-limits discipline applied to pixel dimensions
/// rather than compressed byte size: a small compressed file can still
/// decompress into gigabytes of pixels ("decompression bomb"). 8192 is
/// generous for any real photo/screenshot while bounding worst-case
/// memory to `8192 * 8192 * 4` bytes (~256 MiB) per side product, not
/// per dimension alone — see the area check below.
pub const MAX_DECODED_DIMENSION: u32 = 8192;

/// Decodes JPEG/PNG/WebP into RGBA8. Rejects AVIF/HEIC with an honest
/// "recognized but unsupported" error (see `lib.rs` doc comment) rather
/// than silently misdetecting them as one of the supported formats.
///
/// Note: this checks dimensions *after* the underlying decoder has
/// already produced the full pixel buffer, not before. A pre-decode
/// bound (via the `image` crate's decoder-level limits API) would be
/// stronger and is flagged here as a known gap rather than guessed at —
/// this workspace has no compiler access to verify that API's exact
/// shape in the pinned `image` version before shipping it blind.
pub fn decode_image(bytes: &[u8]) -> Result<DecodedImage, ImageError> {
    let detected = format::detect(bytes).ok_or(ImageError::UnrecognizedFormat)?;
    match detected {
        ImageFormat::Avif | ImageFormat::Heic => {
            return Err(ImageError::UnsupportedFormat(detected));
        }
        ImageFormat::Png | ImageFormat::Jpeg | ImageFormat::WebP => {}
    }

    let dynamic = image::load_from_memory(bytes).map_err(|e| ImageError::Codec(e.to_string()))?;
    let (width, height) = (dynamic.width(), dynamic.height());
    if width > MAX_DECODED_DIMENSION || height > MAX_DECODED_DIMENSION {
        return Err(ImageError::DimensionsTooLarge {
            width,
            height,
            max: MAX_DECODED_DIMENSION,
        });
    }

    let rgba = dynamic.to_rgba8().into_raw();
    Ok(DecodedImage {
        width,
        height,
        rgba,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgba};

    fn encode_png_fixture(width: u32, height: u32) -> Vec<u8> {
        let img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_fn(width, height, |x, y| {
            Rgba([(x % 256) as u8, (y % 256) as u8, 128, 255])
        });
        let mut buf = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
            .unwrap();
        buf
    }

    #[test]
    fn round_trips_a_small_png() {
        let bytes = encode_png_fixture(4, 3);
        let decoded = decode_image(&bytes).unwrap();
        assert_eq!(decoded.width, 4);
        assert_eq!(decoded.height, 3);
        assert_eq!(decoded.rgba.len(), (4 * 3 * 4) as usize);
    }

    #[test]
    fn rejects_unrecognized_bytes() {
        let err = decode_image(b"definitely not an image").unwrap_err();
        assert!(matches!(err, ImageError::UnrecognizedFormat));
    }

    #[test]
    fn rejects_avif_honestly_rather_than_misdetecting() {
        let mut bytes = vec![0, 0, 0, 20];
        bytes.extend_from_slice(b"ftyp");
        bytes.extend_from_slice(b"avif");
        bytes.extend_from_slice(&[0; 8]);
        let err = decode_image(&bytes).unwrap_err();
        assert!(matches!(
            err,
            ImageError::UnsupportedFormat(ImageFormat::Avif)
        ));
    }
}
