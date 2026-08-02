//! File transfer over `iroh-blobs`: BLAKE3-verified streaming, so we never
//! buffer a whole file in memory to hash or send it, and the receiver gets
//! integrity-checked bytes without a separate checksum step.
//!
//! Adaptive compression: before adding a file to the local blob store, its
//! extension is checked against `ALREADY_COMPRESSED_EXTS`. Formats already
//! packed with their own entropy-dense compression (images, video, audio,
//! archives) are stored as-is — spending CPU on zstd there is close to pure
//! loss and can occasionally make the file slightly *larger*. Anything else
//! (text, source code, uncompressed docs, logs, CSVs) is zstd-compressed
//! first, and the `Body::File { compressed, .. }` flag on the announcement
//! envelope tells the receiver whether to reverse that after downloading.
//!
//! ## API accuracy note
//! Like `net::registry`, the exact `iroh-blobs` client method names
//! (`add_path`/`add_bytes`, how a `BlobTicket` is constructed, how a
//! download is driven to completion) are written against the documented
//! shape (`iroh_blobs::store::fs::FsStore`, `BlobsProtocol`,
//! `iroh_blobs::ticket::BlobTicket`) and marked `// VERIFY:` where you
//! should confirm against the pinned version's docs.

use anyhow::{Context, Result};
use iroh::{Endpoint, EndpointId};
use iroh_blobs::store::fs::FsStore;
use iroh_blobs::Hash;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Same rationale as `protocol::dm::NET_TIMEOUT` and friends — bound the
/// one network call in this module, just longer, since a real file can
/// legitimately take a while to move over a slow link.
const FILE_DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(120);

/// Extensions we don't bother zstd'ing — already compressed formats where
/// re-compressing wastes CPU for little to no size benefit.
const ALREADY_COMPRESSED_EXTS: &[&str] = &[
    "jpg", "jpeg", "png", "webp", "gif", "heic", "avif", "mp4", "mkv", "webm", "mov", "avi", "mp3",
    "opus", "aac", "ogg", "flac", "zip", "gz", "zst", "7z", "rar", "xz", "pdf",
];

const ZSTD_LEVEL: i32 = 9; // files aren't latency-sensitive like chat text — spend more CPU for better ratio

fn should_compress(path: &Path) -> bool {
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) => !ALREADY_COMPRESSED_EXTS
            .iter()
            .any(|known| known.eq_ignore_ascii_case(ext)),
        None => true, // no extension — assume compressible (e.g. a plain-text log)
    }
}

pub struct PreparedFile {
    pub hash: Hash,
    pub size_bytes: u64,
    pub compressed: bool,
    pub name: String,
    pub mime: String,
}

/// Fetch a display-picture/status-image blob by hash — same
/// BLAKE3-verified download path as `fetch_incoming` (the `VERIFY:` note
/// on `downloader()`/`.download()` there applies equally here), but
/// returns raw bytes rather than writing to a peer-supplied file name:
/// an avatar's on-disk cache location is always keyed by its own content
/// hash (see `app::fetch_contact_avatar`), never anything the peer names.
/// No decompression step either — avatars are always canonicalized to
/// PNG by `media::decode_avatar` before being added to the blob store in
/// the first place, so what comes back here is already final.
pub async fn fetch_avatar_bytes(
    store: &FsStore,
    endpoint: &Endpoint,
    from: EndpointId,
    hash: Hash,
) -> Result<Vec<u8>> {
    let downloader = store.downloader(endpoint);
    tokio::time::timeout(FILE_DOWNLOAD_TIMEOUT, downloader.download(hash, vec![from]))
        .await
        .map_err(|_| {
            anyhow::anyhow!("avatar download timed out after {FILE_DOWNLOAD_TIMEOUT:?} — the sender may be offline")
        })?
        .context("downloading avatar blob")?;

    let raw = store
        .blobs()
        .get_bytes(hash)
        .await
        .context("reading downloaded avatar blob")?;
    Ok(raw.to_vec())
}

/// Read `path`, adaptively compress, and add the result to the local blob
/// store, ready to be announced over a DM `Body::File` envelope and served
/// to the peer once they fetch it by hash.
pub async fn prepare_outgoing(store: &FsStore, path: &Path) -> Result<PreparedFile> {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "file".to_string());
    let mime = mime_guess_simple(path);

    let raw = tokio::fs::read(path)
        .await
        .with_context(|| format!("reading {}", path.display()))?;

    let (bytes, compressed) = if should_compress(path) {
        let compressed = tokio::task::spawn_blocking(move || {
            zstd::stream::encode_all(raw.as_slice(), ZSTD_LEVEL)
        })
        .await
        .context("compression task panicked")?
        .context("zstd compress")?;
        (compressed, true)
    } else {
        (raw, false)
    };

    let size_bytes = bytes.len() as u64;

    // VERIFY: exact method for adding in-memory bytes to an `FsStore` and
    // getting back the content hash — confirmed the return type carries a
    // `hash` *field* (not a method) in the pinned iroh-blobs version.
    let tag_info = store
        .blobs()
        .add_bytes(bytes)
        .await
        .context("adding file to local blob store")?;
    let hash = tag_info.hash;

    Ok(PreparedFile {
        hash,
        size_bytes,
        compressed,
        name,
        mime,
    })
}

/// Fetch a file announced by a peer: download the blob by hash from them,
/// verify (handled internally by iroh-blobs' BLAKE3 streaming), decompress
/// if the sender marked it compressed, and write the result to
/// `dest_dir/name`. Returns the final on-disk path.
pub async fn fetch_incoming(
    store: &FsStore,
    endpoint: &Endpoint,
    from: EndpointId,
    hash: Hash,
    name: &str,
    compressed: bool,
    dest_dir: &Path,
) -> Result<PathBuf> {
    tokio::fs::create_dir_all(dest_dir).await?;

    // VERIFY: this is the one call in the whole rewrite I'm least sure of.
    // `FsStore` doesn't expose a plain `.download()` — it exposes
    // `.downloader(&endpoint) -> Downloader`, whose exact
    // download-from-a-specific-peer-by-hash method name/signature I
    // couldn't confirm from available docs (see
    // github.com/n0-computer/iroh-docs/issues/86, where someone else hit
    // this same gap). Check `cargo doc -p iroh-blobs` for `Downloader`'s
    // methods before relying on this.
    //
    // Bounded, same reasoning as every other network call in this
    // codebase (`protocol::dm::NET_TIMEOUT` etc.) — previously this was
    // the one unbounded network call left: if `from` had gone offline
    // between announcing the file and this download starting, it would
    // hang indefinitely with no feedback instead of failing visibly.
    // Longer than the 8s used for plain messages since a real file
    // transfer can legitimately take a while on a slow link — this is a
    // "give up, something's actually wrong" bound, not a "typical transfer
    // time" one.
    let downloader = store.downloader(endpoint);
    tokio::time::timeout(FILE_DOWNLOAD_TIMEOUT, downloader.download(hash, vec![from]))
        .await
        .map_err(|_| {
            anyhow::anyhow!(
                "download timed out after {FILE_DOWNLOAD_TIMEOUT:?} — the sender may be offline"
            )
        })?
        .context("downloading file blob")?;

    let raw = store
        .blobs()
        .get_bytes(hash)
        .await
        .context("reading downloaded blob")?;
    // Copy out of whatever borrowed/ref-counted buffer type the store
    // returns immediately, rather than calling slice methods on it
    // in-place — sidesteps any surprises from that type's own inherent
    // methods shadowing the ones we want.
    let bytes: Vec<u8> = raw.to_vec();

    let final_bytes = if compressed {
        tokio::task::spawn_blocking(move || zstd::stream::decode_all(&bytes[..]))
            .await
            .context("decompression task panicked")?
            .context("zstd decompress")?
    } else {
        bytes
    };

    let dest = dest_dir.join(sanitize_filename(name));
    tokio::fs::write(&dest, final_bytes)
        .await
        .with_context(|| format!("writing {}", dest.display()))?;

    Ok(dest)
}

/// Small, dependency-free mime guess from extension — good enough for
/// showing an icon/preview hint in the chat bubble; not a validated
/// content-sniff, so treat it as a UI hint, not a security boundary.
fn mime_guess_simple(path: &Path) -> String {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "pdf" => "application/pdf",
        "mp4" => "video/mp4",
        "mp3" => "audio/mpeg",
        "opus" => "audio/opus",
        "zip" => "application/zip",
        "txt" | "md" => "text/plain",
        _ => "application/octet-stream",
    }
    .to_string()
}

/// Strip path separators out of a received filename before writing it —
/// a malicious peer announcing a file named `../../.bashrc` should not be
/// able to write outside `dest_dir`.
fn sanitize_filename(name: &str) -> String {
    name.rsplit(['/', '\\'])
        .next()
        .filter(|s| !s.is_empty() && *s != "." && *s != "..")
        .unwrap_or("received_file")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compressed_extensions_are_skipped() {
        assert!(!should_compress(Path::new("photo.jpg")));
        assert!(!should_compress(Path::new("archive.zip")));
        assert!(should_compress(Path::new("notes.txt")));
        assert!(should_compress(Path::new("no_extension_log")));
    }

    #[test]
    fn sanitizes_path_traversal() {
        assert_eq!(sanitize_filename("../../.bashrc"), ".bashrc");
        assert_eq!(sanitize_filename("report.pdf"), "report.pdf");
    }
}
