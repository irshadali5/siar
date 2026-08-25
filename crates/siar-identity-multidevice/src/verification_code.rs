//! §19 "Numeric Verification": "The code is derived from the
//! authenticated handshake transcript. Do not generate an unrelated
//! random code."

use crate::invite::DeviceLinkInvite;
use crate::link_key::EphemeralLinkPublicKey;

/// The real transcript §19 asks for: the invite's own signed content
/// (binding the code to *this specific* linking attempt, not a generic
/// handshake) plus both sides' ephemeral public keys — mirroring the
/// same "bind a derived value to every field that must not be
/// substitutable" reasoning
/// [`crate::certificate::DeviceCertificate::signing_payload`] already
/// applies to signatures, applied here to a derived *display* value
/// instead. Including the shared secret itself (not just the public
/// keys) means an attacker who only observes the public handshake
/// traffic (both public keys, the invite) cannot precompute the
/// verification code without actually completing the Diffie-Hellman
/// exchange.
fn transcript(invite: &DeviceLinkInvite, responder_public: &EphemeralLinkPublicKey, shared_secret: &[u8; 32]) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&postcard::to_allocvec(invite).expect("postcard encoding of a fixed-shape struct cannot fail"));
    bytes.extend_from_slice(&responder_public.0);
    bytes.extend_from_slice(shared_secret);
    bytes
}

/// §19: "6–12 digit code." Six digits — enough entropy (10^6, ~20
/// bits) for a person to catch a mismatch during a short-range,
/// low-latency comparison (this isn't a password; it's a
/// human-in-the-loop check against an active on-path attacker during
/// one linking attempt), matching the low end of the spec's own stated
/// range rather than the high end, since a 12-digit code is
/// meaningfully harder for a person to actually compare correctly.
pub fn derive_verification_code(invite: &DeviceLinkInvite, responder_public: &EphemeralLinkPublicKey, shared_secret: &[u8; 32]) -> String {
    let transcript_bytes = transcript(invite, responder_public, shared_secret);
    let hash = blake3::hash(&transcript_bytes);
    let hash_bytes = hash.as_bytes();
    // First 4 bytes of the hash, reduced mod 1_000_000 — a real,
    // deterministic reduction from uniformly-distributed hash output
    // to a 6-digit decimal display value, not a truncation that would
    // bias toward smaller numbers.
    let value = u32::from_be_bytes([hash_bytes[0], hash_bytes[1], hash_bytes[2], hash_bytes[3]]);
    format!("{:06}", value % 1_000_000)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::link_key::EphemeralLinkKeyPair;
    use crate::root_key::RootIdentityKey;
    use siar_domain::{AccountId, DeviceId};

    fn setup() -> (DeviceLinkInvite, EphemeralLinkPublicKey, [u8; 32]) {
        let root = RootIdentityKey::generate();
        let inviter = EphemeralLinkKeyPair::generate();
        let new_device = EphemeralLinkKeyPair::generate();
        let invite = DeviceLinkInvite::create(&root, AccountId::new(), DeviceId::new(), inviter.public_key(), 10_000);
        let shared_secret = inviter.diffie_hellman(&new_device.public_key());
        (invite, new_device.public_key(), shared_secret)
    }

    #[test]
    fn the_code_is_always_exactly_six_digits() {
        let (invite, responder_public, shared_secret) = setup();
        let code = derive_verification_code(&invite, &responder_public, &shared_secret);
        assert_eq!(code.len(), 6);
        assert!(code.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn both_sides_of_a_real_handshake_derive_the_same_code() {
        let root = RootIdentityKey::generate();
        let inviter = EphemeralLinkKeyPair::generate();
        let new_device = EphemeralLinkKeyPair::generate();
        let invite = DeviceLinkInvite::create(&root, AccountId::new(), DeviceId::new(), inviter.public_key(), 10_000);

        let shared_a = inviter.diffie_hellman(&new_device.public_key());
        let shared_b = new_device.diffie_hellman(&inviter.public_key());

        let code_inviter_side = derive_verification_code(&invite, &new_device.public_key(), &shared_a);
        let code_new_device_side = derive_verification_code(&invite, &new_device.public_key(), &shared_b);
        assert_eq!(code_inviter_side, code_new_device_side);
    }

    #[test]
    fn a_different_shared_secret_produces_a_different_code_with_overwhelming_probability() {
        let (invite, responder_public, shared_secret) = setup();
        let mut tampered_secret = shared_secret;
        tampered_secret[0] ^= 0xFF;
        let code_a = derive_verification_code(&invite, &responder_public, &shared_secret);
        let code_b = derive_verification_code(&invite, &responder_public, &tampered_secret);
        assert_ne!(code_a, code_b);
    }

    #[test]
    fn a_different_invite_produces_a_different_code_with_overwhelming_probability() {
        let (invite_a, responder_public, shared_secret) = setup();
        let (invite_b, _, _) = setup(); // a fresh, unrelated invite
        let code_a = derive_verification_code(&invite_a, &responder_public, &shared_secret);
        let code_b = derive_verification_code(&invite_b, &responder_public, &shared_secret);
        assert_ne!(code_a, code_b);
    }
}
