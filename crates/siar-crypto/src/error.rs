use thiserror::Error;

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("signature verification failed")]
    InvalidSignature,
    #[error("AEAD decryption failed (wrong key, or ciphertext was tampered with)")]
    DecryptionFailed,
    #[error("malformed key material")]
    MalformedKey,
    #[error("identity file I/O error: {0}")]
    Io(String),
}
