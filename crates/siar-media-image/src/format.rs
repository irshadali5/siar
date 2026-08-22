/// Detected purely from the byte header, never trusted from a
/// caller-supplied MIME string or filename extension (plan.md §68:
/// remote-influenced values like `MimeType`/`Filename` are validated,
/// not trusted at face value).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    Png,
    Jpeg,
    WebP,
    /// Recognized (ISOBMFF `ftyp` box, `avif`/`avis` brand) but not
    /// decoded — see `ImageError::UnsupportedFormat`.
    Avif,
    /// Recognized (ISOBMFF `ftyp` box, `heic`/`heix`/`heim`/`heis`/`mif1`
    /// brand) but not decoded — codecs2.md treats HEIC as a
    /// platform-integration concern, not a crate this workspace owns.
    Heic,
}

const PNG_MAGIC: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
const JPEG_MAGIC: [u8; 3] = [0xFF, 0xD8, 0xFF];

/// Inspects the byte header and returns the detected format, or `None`
/// if nothing in this policy's allow-list matched.
pub fn detect(bytes: &[u8]) -> Option<ImageFormat> {
    if bytes.len() >= PNG_MAGIC.len() && bytes[..PNG_MAGIC.len()] == PNG_MAGIC {
        return Some(ImageFormat::Png);
    }
    if bytes.len() >= JPEG_MAGIC.len() && bytes[..JPEG_MAGIC.len()] == JPEG_MAGIC {
        return Some(ImageFormat::Jpeg);
    }
    if is_riff_webp(bytes) {
        return Some(ImageFormat::WebP);
    }
    if let Some(brand) = isobmff_major_brand(bytes) {
        if brand == *b"avif" || brand == *b"avis" {
            return Some(ImageFormat::Avif);
        }
        if matches!(&brand, b"heic" | b"heix" | b"heim" | b"heis" | b"mif1") {
            return Some(ImageFormat::Heic);
        }
    }
    None
}

fn is_riff_webp(bytes: &[u8]) -> bool {
    bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP"
}

/// ISOBMFF files (AVIF/HEIC containers) start with a `ftyp` box:
/// 4-byte size, "ftyp", 4-byte major brand.
fn isobmff_major_brand(bytes: &[u8]) -> Option<[u8; 4]> {
    if bytes.len() < 12 || &bytes[4..8] != b"ftyp" {
        return None;
    }
    let mut brand = [0u8; 4];
    brand.copy_from_slice(&bytes[8..12]);
    Some(brand)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_png() {
        let mut bytes = PNG_MAGIC.to_vec();
        bytes.extend_from_slice(&[0; 16]);
        assert_eq!(detect(&bytes), Some(ImageFormat::Png));
    }

    #[test]
    fn detects_jpeg() {
        let mut bytes = JPEG_MAGIC.to_vec();
        bytes.extend_from_slice(&[0; 16]);
        assert_eq!(detect(&bytes), Some(ImageFormat::Jpeg));
    }

    #[test]
    fn detects_webp() {
        let mut bytes = b"RIFF".to_vec();
        bytes.extend_from_slice(&[0; 4]); // chunk size, irrelevant here
        bytes.extend_from_slice(b"WEBP");
        assert_eq!(detect(&bytes), Some(ImageFormat::WebP));
    }

    #[test]
    fn detects_avif_without_decoding_it() {
        let mut bytes = vec![0, 0, 0, 20];
        bytes.extend_from_slice(b"ftyp");
        bytes.extend_from_slice(b"avif");
        bytes.extend_from_slice(&[0; 8]);
        assert_eq!(detect(&bytes), Some(ImageFormat::Avif));
    }

    #[test]
    fn detects_heic_without_decoding_it() {
        let mut bytes = vec![0, 0, 0, 20];
        bytes.extend_from_slice(b"ftyp");
        bytes.extend_from_slice(b"heic");
        bytes.extend_from_slice(&[0; 8]);
        assert_eq!(detect(&bytes), Some(ImageFormat::Heic));
    }

    #[test]
    fn rejects_garbage() {
        assert_eq!(detect(b"not an image, just text"), None);
    }

    #[test]
    fn rejects_truncated_headers() {
        assert_eq!(detect(&PNG_MAGIC[..4]), None);
        assert_eq!(detect(b"RIFF"), None);
    }
}
