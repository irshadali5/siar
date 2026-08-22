//! Application-level E2EE session (plan.md §12).
//!
//! See the module-level doc on `lib.rs`: this is a static-key AEAD
//! channel, not a ratchet. It is correct and safe to ship for Phase 1
//! (confidentiality + integrity over an already-untrusted relay/mailbox
//! path), but forward secrecy is a Phase-2 requirement this type does
//! not yet meet.

use crate::{identity::DeviceIdentity, CryptoError};
use chacha20poly1305::{
    aead::{Aead, KeyInit, OsRng as AeadOsRng},
    ChaCha20Poly1305, Nonce,
};
use rand_core::RngCore;
use x25519_dalek::PublicKey as X25519PublicKey;

const NONCE_LEN: usize = 12;

pub struct Session {
    cipher: ChaCha20Poly1305,
}

impl Session {
    /// Derive a session from our identity and the peer's X25519 public
    /// key. BLAKE3 (not the raw ECDH output) is used as the key precisely
    /// because raw DH output should never be used directly as a
    /// symmetric key — it needs a KDF step first.
    pub fn establish(us: &DeviceIdentity, peer_x25519_public: &X25519PublicKey) -> Self {
        let shared = us.diffie_hellman(peer_x25519_public);
        let key_bytes = blake3::hash(&shared);
        Self {
            cipher: ChaCha20Poly1305::new(key_bytes.as_bytes().into()),
        }
    }

    /// Encrypts `plaintext`, returning `nonce || ciphertext`. The nonce is
    /// random per-message (safe under ChaCha20Poly1305 as long as the same
    /// key isn't reused often enough to hit birthday-bound collisions,
    /// which a Phase-2 ratchet removes as a concern entirely).
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let mut nonce_bytes = [0u8; NONCE_LEN];
        AeadOsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext)
            .map_err(|_| CryptoError::DecryptionFailed)?;
        let mut out = Vec::with_capacity(NONCE_LEN + ciphertext.len());
        out.extend_from_slice(&nonce_bytes);
        out.extend_from_slice(&ciphertext);
        Ok(out)
    }

    pub fn decrypt(&self, framed: &[u8]) -> Result<Vec<u8>, CryptoError> {
        if framed.len() < NONCE_LEN {
            return Err(CryptoError::DecryptionFailed);
        }
        let (nonce_bytes, ciphertext) = framed.split_at(NONCE_LEN);
        let nonce = Nonce::from_slice(nonce_bytes);
        self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| CryptoError::DecryptionFailed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_sides_derive_a_working_session() {
        let alice = DeviceIdentity::generate();
        let bob = DeviceIdentity::generate();

        let alice_session = Session::establish(&alice, &bob.x25519_public());
        let bob_session = Session::establish(&bob, &alice.x25519_public());

        let ciphertext = alice_session.encrypt(b"hey bob").unwrap();
        let plaintext = bob_session.decrypt(&ciphertext).unwrap();
        assert_eq!(plaintext, b"hey bob");
    }

    #[test]
    fn wrong_session_cannot_decrypt() {
        let alice = DeviceIdentity::generate();
        let bob = DeviceIdentity::generate();
        let mallory = DeviceIdentity::generate();

        let alice_session = Session::establish(&alice, &bob.x25519_public());
        let mallory_session = Session::establish(&mallory, &bob.x25519_public());

        let ciphertext = alice_session.encrypt(b"secret").unwrap();
        assert!(mallory_session.decrypt(&ciphertext).is_err());
    }

    #[test]
    fn tampered_ciphertext_fails_to_decrypt() {
        let alice = DeviceIdentity::generate();
        let bob = DeviceIdentity::generate();
        let alice_session = Session::establish(&alice, &bob.x25519_public());
        let bob_session = Session::establish(&bob, &alice.x25519_public());

        let mut ciphertext = alice_session.encrypt(b"hey bob").unwrap();
        let last = ciphertext.len() - 1;
        ciphertext[last] ^= 0xFF;

        assert!(bob_session.decrypt(&ciphertext).is_err());
    }
}
