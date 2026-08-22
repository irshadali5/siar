use crate::decode::DecodedImage;
use crate::error::ImageError;
use image::{imageops::FilterType, DynamicImage, RgbaImage};

/// architecture.md §25's three-tier design ("8 KB thumbnail / 100 KB
/// preview / 4 MB original") expressed as pixel caps rather than byte
/// targets — byte size still depends on image content/JPEG quality, but
/// bounding the longest side is what actually controls it in practice.
pub const THUMBNAIL_MAX_DIMENSION: u32 = 160;
pub const PREVIEW_MAX_DIMENSION: u32 = 1024;

/// JPEG output quality (0-100). Chosen conservatively low for
/// thumbnails since architecture.md §25's whole point is "don't
/// download a 50 MB image to display the conversation list" — visible
/// compression artifacts on a 160px thumbnail are an acceptable
/// trade-off for size.
const THUMBNAIL_JPEG_QUALITY: u8 = 60;
const PREVIEW_JPEG_QUALITY: u8 = 80;

pub struct EncodedImage {
    pub width: u32,
    pub height: u32,
    /// Always JPEG here: thumbnails/previews are generated artifacts,
    /// not user-supplied files, so this crate controls the format and
    /// picks the one with the best size/quality trade-off for
    /// photographic content. (`siar_domain::MediaType::ImageJpeg` is the
    /// matching tag for the attachment reference this becomes.)
    pub jpeg_bytes: Vec<u8>,
}

/// Generates a small thumbnail per architecture.md §25's tiered
/// delivery ("Fetch progressively"). Resizes down only — never
/// upscales — so a source image already smaller than the target cap is
/// returned near-unchanged rather than blurred larger.
pub fn generate_thumbnail(source: &DecodedImage) -> Result<EncodedImage, ImageError> {
    resize_and_encode(source, THUMBNAIL_MAX_DIMENSION, THUMBNAIL_JPEG_QUALITY)
}

/// Same idea at architecture.md §25's "preview" tier — bigger than a
/// thumbnail, still much smaller than the original.
pub fn generate_preview(source: &DecodedImage) -> Result<EncodedImage, ImageError> {
    resize_and_encode(source, PREVIEW_MAX_DIMENSION, PREVIEW_JPEG_QUALITY)
}

fn resize_and_encode(source: &DecodedImage, max_dimension: u32, quality: u8) -> Result<EncodedImage, ImageError> {
    let buffer = RgbaImage::from_raw(source.width, source.height, source.rgba.clone())
        .ok_or_else(|| ImageError::Codec("decoded RGBA buffer length did not match width*height*4".to_string()))?;
    let dynamic = DynamicImage::ImageRgba8(buffer);

    let (target_w, target_h) = fit_within(source.width, source.height, max_dimension);
    let resized = if target_w == source.width && target_h == source.height {
        dynamic
    } else {
        dynamic.resize(target_w, target_h, FilterType::Triangle)
    };

    let mut jpeg_bytes = Vec::new();
    let rgb = resized.to_rgb8();
    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg_bytes, quality);
    rgb.write_with_encoder(encoder)
        .map_err(|e| ImageError::Codec(e.to_string()))?;

    Ok(EncodedImage {
        width: rgb.width(),
        height: rgb.height(),
        jpeg_bytes,
    })
}

/// Scales `(width, height)` down (never up) so the longer side is at
/// most `max_dimension`, preserving aspect ratio.
fn fit_within(width: u32, height: u32, max_dimension: u32) -> (u32, u32) {
    let longest = width.max(height);
    if longest <= max_dimension || longest == 0 {
        return (width, height);
    }
    let scale = max_dimension as f64 / longest as f64;
    let new_w = ((width as f64 * scale).round() as u32).max(1);
    let new_h = ((height as f64 * scale).round() as u32).max(1);
    (new_w, new_h)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decode::decode_image;
    use image::{ImageBuffer, Rgba};

    fn fixture(width: u32, height: u32) -> DecodedImage {
        let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
            ImageBuffer::from_fn(width, height, |x, y| Rgba([(x % 256) as u8, (y % 256) as u8, 128, 255]));
        let mut buf = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
            .unwrap();
        decode_image(&buf).unwrap()
    }

    #[test]
    fn thumbnail_never_exceeds_the_cap_on_the_long_side() {
        let source = fixture(3000, 1500);
        let thumb = generate_thumbnail(&source).unwrap();
        assert!(thumb.width.max(thumb.height) <= THUMBNAIL_MAX_DIMENSION);
        assert!(!thumb.jpeg_bytes.is_empty());
    }

    #[test]
    fn preserves_aspect_ratio() {
        let source = fixture(4000, 2000); // 2:1
        let thumb = generate_thumbnail(&source).unwrap();
        // integer rounding means "close to" 2:1, not bit-exact
        let ratio = thumb.width as f64 / thumb.height as f64;
        assert!((ratio - 2.0).abs() < 0.05);
    }

    #[test]
    fn never_upscales_a_small_source() {
        let source = fixture(20, 10);
        let thumb = generate_thumbnail(&source).unwrap();
        assert_eq!((thumb.width, thumb.height), (20, 10));
    }

    #[test]
    fn preview_cap_is_larger_than_thumbnail_cap() {
        assert!(PREVIEW_MAX_DIMENSION > THUMBNAIL_MAX_DIMENSION);
    }
}
