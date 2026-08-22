//! Attachment encryption (plan.md §23): a fresh random key per
//! attachment, deliberately *not* the conversation `Session` key — a
//! blob keeps its own key so it can outlive any one session (re-shared,
//! forwarded, cached) without depending on which session encrypted it.
//!
//! plan.md §24: because each attachment gets its own random key/nonce,
//! identical plaintext produces different ciphertext — that's a
//! deliberate privacy tradeoff (no accidental plaintext-content
//! deduplication leaking "these two blobs are the same file" to
//! whatever stores the ciphertext), not an oversight.

use crate::CryptoError;
use chacha20poly1305::{
    aead::{Aead, KeyInit, OsRng},
    ChaCha20Poly1305, Nonce,
};
use rand_core::RngCore;
use serde::{Deserialize, Serialize};

const NONCE_LEN: usize = 12;

/// Serializable so it can travel inside an already-encrypted message
/// envelope (plan.md §23: "send attachment key only within the encrypted
/// message envelope" — never on its own, never alongside the blob).
#[derive(Clone, Serialize, Deserialize, zeroize::Zeroize, zeroize::ZeroizeOnDrop)]
pub struct AttachmentKey([u8; 32]);

impl AttachmentKey {
    pub fn generate() -> Self {
        let mut bytes = [0u8; 32];
        OsRng.fill_bytes(&mut bytes);
        Self(bytes)
    }

    /// Reconstructs a key from bytes carried in an `AttachmentReference`
    /// (plan.md §23 — the key travels inside the already-encrypted
    /// envelope, this is the receiving side turning those bytes back
    /// into something `decrypt_attachment` can use).
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn to_bytes(&self) -> [u8; 32] {
        self.0
    }

    fn cipher(&self) -> ChaCha20Poly1305 {
        ChaCha20Poly1305::new((&self.0).into())
    }
}

/// The content hash of the *ciphertext* (plan.md §22: BLAKE3 content
/// addressing) — this is what gets requested over the wire, so a peer
/// asking "send me blob X" is asking for the encrypted bytes, never
/// implying they know the plaintext.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlobHash([u8; 32]);

impl BlobHash {
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

pub struct EncryptedBlob {
    pub hash: BlobHash,
    pub ciphertext: Vec<u8>,
}

/// Encrypts `plaintext` under a fresh key, returning both the ciphertext
/// (ready to publish) and the key (to embed in the message envelope).
pub fn encrypt_attachment(plaintext: &[u8]) -> Result<(EncryptedBlob, AttachmentKey), CryptoError> {
    let key = AttachmentKey::generate();
    let mut nonce_bytes = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let sealed = key
        .cipher()
        .encrypt(nonce, plaintext)
        .map_err(|_| CryptoError::DecryptionFailed)?;

    let mut framed = Vec::with_capacity(NONCE_LEN + sealed.len());
    framed.extend_from_slice(&nonce_bytes);
    framed.extend_from_slice(&sealed);

    let hash = BlobHash(*blake3::hash(&framed).as_bytes());

    Ok((
        EncryptedBlob {
            hash,
            ciphertext: framed,
        },
        key,
    ))
}

/// Decrypts a blob fetched from the network. Verifies the content hash
/// first (plan.md §73: check/verify before trusting untrusted bytes) —
/// AEAD decryption would also catch tampering, but hash-checking first
/// means a corrupted transfer is rejected without even attempting to run
/// the cipher over attacker-controlled bytes.
pub fn decrypt_attachment(
    ciphertext: &[u8],
    expected_hash: BlobHash,
    key: &AttachmentKey,
) -> Result<Vec<u8>, CryptoError> {
    let actual_hash = BlobHash(*blake3::hash(ciphertext).as_bytes());
    if actual_hash != expected_hash {
        return Err(CryptoError::DecryptionFailed);
    }
    if ciphertext.len() < NONCE_LEN {
        return Err(CryptoError::DecryptionFailed);
    }
    let (nonce_bytes, sealed) = ciphertext.split_at(NONCE_LEN);
    let nonce = Nonce::from_slice(nonce_bytes);
    key.cipher()
        .decrypt(nonce, sealed)
        .map_err(|_| CryptoError::DecryptionFailed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips() {
        let plaintext = b"a whole file's worth of bytes, pretend";
        let (blob, key) = encrypt_attachment(plaintext).unwrap();
        let recovered = decrypt_attachment(&blob.ciphertext, blob.hash, &key).unwrap();
        assert_eq!(recovered, plaintext);
    }

    #[test]
    fn same_plaintext_twice_yields_different_ciphertext_and_hash() {
        // plan.md §24: privacy over dedup — verify that's actually true,
        // not just asserted in a comment.
        let plaintext = b"identical content";
        let (blob_a, _) = encrypt_attachment(plaintext).unwrap();
        let (blob_b, _) = encrypt_attachment(plaintext).unwrap();
        assert_ne!(blob_a.ciphertext, blob_b.ciphertext);
        assert_ne!(blob_a.hash, blob_b.hash);
    }

    #[test]
    fn tampered_ciphertext_fails_the_hash_check() {
        let (mut blob, key) = encrypt_attachment(b"data").unwrap();
        let last = blob.ciphertext.len() - 1;
        blob.ciphertext[last] ^= 0xFF;
        assert!(decrypt_attachment(&blob.ciphertext, blob.hash, &key).is_err());
    }

    #[test]
    fn wrong_key_fails_even_with_correct_hash() {
        let (blob, _) = encrypt_attachment(b"data").unwrap();
        let wrong_key = AttachmentKey::generate();
        assert!(decrypt_attachment(&blob.ciphertext, blob.hash, &wrong_key).is_err());
    }
}
