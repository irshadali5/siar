//! §16 "Device Linking Invitation"'s `ephemeral_link_key`, §17 "QR
//! Linking".

use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret};

/// A fresh, single-use X25519 keypair generated for one linking
/// attempt — never the account root key or a device's own long-term
/// key. Mirrors `siar_crypto::DeviceIdentity`'s own `x25519_dalek`
/// usage pattern (`StaticSecret::random_from_rng`/`PublicKey::from`),
/// duplicated here rather than imported since this crate is
/// deliberately independent of `siar-crypto` (see this crate's own top
/// doc comment).
pub struct EphemeralLinkKeyPair {
    secret: StaticSecret,
}

impl EphemeralLinkKeyPair {
    pub fn generate() -> Self {
        Self {
            secret: StaticSecret::random_from_rng(OsRng),
        }
    }

    pub fn public_key(&self) -> EphemeralLinkPublicKey {
        EphemeralLinkPublicKey(X25519PublicKey::from(&self.secret).to_bytes())
    }

    /// The shared secret both sides of a linking handshake derive
    /// independently — never transmitted, never serialized. Feeds
    /// [`crate::verification_code::derive_verification_code`] (§19).
    pub fn diffie_hellman(&self, their_public: &EphemeralLinkPublicKey) -> [u8; 32] {
        let their_public = X25519PublicKey::from(their_public.0);
        self.secret.diffie_hellman(&their_public).to_bytes()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct EphemeralLinkPublicKey(pub [u8; 32]);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_sides_of_a_handshake_derive_the_same_shared_secret() {
        let inviter = EphemeralLinkKeyPair::generate();
        let new_device = EphemeralLinkKeyPair::generate();

        let shared_a = inviter.diffie_hellman(&new_device.public_key());
        let shared_b = new_device.diffie_hellman(&inviter.public_key());
        assert_eq!(shared_a, shared_b);
    }

    #[test]
    fn different_keypairs_never_derive_the_same_shared_secret() {
        let inviter = EphemeralLinkKeyPair::generate();
        let new_device = EphemeralLinkKeyPair::generate();
        let impostor = EphemeralLinkKeyPair::generate();

        let real_shared = inviter.diffie_hellman(&new_device.public_key());
        let impostor_shared = inviter.diffie_hellman(&impostor.public_key());
        assert_ne!(real_shared, impostor_shared);
    }
}
