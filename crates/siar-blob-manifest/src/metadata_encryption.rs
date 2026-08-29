//! Encrypted file metadata (Part 28 §31).
//!
//! §31: "where practical, encrypt file name, caption, MIME metadata,
//! thumbnail, document metadata... routing infrastructure should see
//! only what it needs." [`crate::descriptor::FileMetadata`] already
//! exists as a plaintext struct holding exactly this data
//! (`display_name`, `media_type`, `logical_size`, `created_at_millis`)
//! — nothing in this crate encrypted it before this module. This is
//! the real gap §31 asks to close: `encrypt_file_metadata`/
//! `decrypt_file_metadata` wrap it the same way
//! [`crate::encryption::encrypt_blob`]/[`crate::encryption::decrypt_blob`]
//! already wrap the file's own content — same key, same framing
//! (nonce-prepended ciphertext), different AAD domain so a metadata
//! ciphertext can never be mistaken for (or swapped into) a content
//! ciphertext slot even though both are protected under the same
//! per-blob key.
//!
//! Reusing [`crate::ids::BlobEncryptionKey`] rather than deriving a
//! second key is a deliberate, documented choice, not a shortcut: §29
//! ("File Content Key... use separate derivation/context from message
//! keys") is about separating file keys from *message* keys, a
//! distinction this crate's whole design already maintains by staying
//! independent of `siar-crypto` (see `lib.rs`'s own top doc) — it says
//! nothing about needing a *second* file-scoped key for metadata
//! specifically. Domain-separating the AAD (this module's
//! `METADATA_AAD_DOMAIN`) rather than the key achieves the one property
//! that actually matters here — a metadata ciphertext decrypted with
//! the content AAD (or vice versa) fails authentication — without
//! doubling the key-management surface for a single blob.

use chacha20poly1305::aead::{Aead, Payload};
use chacha20poly1305::{aead::KeyInit, aead::OsRng, ChaCha20Poly1305, Nonce};
use rand_core::RngCore;

use crate::descriptor::FileMetadata;
use crate::encryption::EncryptionError;
use crate::ids::BlobEncryptionKey;

const NONCE_LEN: usize = 12;

/// Mirrors `siar_crypto::domains::CryptoDomain::FileMetadata`'s label
/// exactly (`comm/file-metadata/v1`) — duplicated as a literal here,
/// not imported, since this crate deliberately stays independent of
/// `siar-crypto` (see `lib.rs`'s own top doc comment). What matters for
/// domain separation is the two modules agreeing on the same *string
/// value*, not sharing a Rust type — if this ever drifts out of sync
/// with `siar-crypto`'s copy, that's a real defect to fix by updating
/// both literals to match, not a reason to introduce the cross-crate
/// dependency this crate's design has so far deliberately avoided.
const METADATA_AAD_DOMAIN: &[u8] = b"comm/file-metadata/v1";

fn cipher(key: &BlobEncryptionKey) -> ChaCha20Poly1305 {
    ChaCha20Poly1305::new((&key.0).into())
}

/// Encrypts `metadata` under `key` (the same key protecting that blob's
/// content). Returns nonce-prepended ciphertext, matching
/// [`crate::encryption::encrypt_blob`]'s own framing.
pub fn encrypt_file_metadata(
    key: &BlobEncryptionKey,
    metadata: &FileMetadata,
) -> Result<Vec<u8>, EncryptionError> {
    let plaintext =
        postcard::to_allocvec(metadata).map_err(|_| EncryptionError::EncryptionFailed)?;

    let mut nonce_bytes = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let sealed = cipher(key)
        .encrypt(
            nonce,
            Payload {
                msg: &plaintext,
                aad: METADATA_AAD_DOMAIN,
            },
        )
        .map_err(|_| EncryptionError::EncryptionFailed)?;

    let mut framed = Vec::with_capacity(NONCE_LEN + sealed.len());
    framed.extend_from_slice(&nonce_bytes);
    framed.extend_from_slice(&sealed);
    Ok(framed)
}

/// Decrypts and deserializes a `FileMetadata` produced by
/// [`encrypt_file_metadata`]. Fails (rather than returning a partially-
/// trusted value) if the ciphertext was sealed under a different AAD
/// domain — including, deliberately, a content ciphertext from
/// [`crate::encryption::encrypt_blob`] under the same key: that AAD is
/// empty, this one isn't, so the two can never be decrypted into each
/// other's slot even by a caller that mixes them up.
pub fn decrypt_file_metadata(
    key: &BlobEncryptionKey,
    ciphertext: &[u8],
) -> Result<FileMetadata, EncryptionError> {
    if ciphertext.len() < NONCE_LEN {
        return Err(EncryptionError::CiphertextTooShort {
            actual: ciphertext.len(),
        });
    }
    let (nonce_bytes, sealed) = ciphertext.split_at(NONCE_LEN);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher(key)
        .decrypt(
            nonce,
            Payload {
                msg: sealed,
                aad: METADATA_AAD_DOMAIN,
            },
        )
        .map_err(|_| EncryptionError::DecryptionFailed)?;

    postcard::from_bytes(&plaintext).map_err(|_| EncryptionError::DecryptionFailed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::descriptor::FileName;
    use crate::encryption::generate_blob_key;
    use siar_domain::MediaType;

    fn sample_metadata() -> FileMetadata {
        FileMetadata {
            display_name: Some(FileName::new("vacation-photo.jpg").unwrap()),
            media_type: Some(MediaType::ImageJpeg),
            logical_size: 4_096,
            created_at_millis: Some(1_700_000_000_000),
        }
    }

    #[test]
    fn round_trips() {
        let key = generate_blob_key();
        let metadata = sample_metadata();
        let ciphertext = encrypt_file_metadata(&key, &metadata).unwrap();
        let recovered = decrypt_file_metadata(&key, &ciphertext).unwrap();
        assert_eq!(recovered, metadata);
    }

    #[test]
    fn wrong_key_fails_decryption() {
        let key = generate_blob_key();
        let wrong_key = generate_blob_key();
        let ciphertext = encrypt_file_metadata(&key, &sample_metadata()).unwrap();
        assert!(decrypt_file_metadata(&wrong_key, &ciphertext).is_err());
    }

    #[test]
    fn a_content_ciphertext_under_the_same_key_does_not_decrypt_as_metadata() {
        use crate::encryption::encrypt_blob;

        let key = generate_blob_key();
        let (content_ciphertext, _descriptor) = encrypt_blob(&key, b"file bytes").unwrap();
        // Same key, wrong AAD domain (content uses none, metadata uses
        // METADATA_AAD_DOMAIN) — must fail, not silently decrypt into
        // garbage that happens to parse.
        assert!(decrypt_file_metadata(&key, &content_ciphertext).is_err());
    }

    #[test]
    fn tampered_metadata_ciphertext_fails_decryption() {
        let key = generate_blob_key();
        let mut ciphertext = encrypt_file_metadata(&key, &sample_metadata()).unwrap();
        let last = ciphertext.len() - 1;
        ciphertext[last] ^= 0xFF;
        assert!(decrypt_file_metadata(&key, &ciphertext).is_err());
    }
}
