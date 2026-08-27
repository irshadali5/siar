//! Device linking (plan.md §42): an already-trusted device vouches for a
//! new one by signing its public keys. This is composition of the
//! Ed25519 sign/verify primitives `identity.rs` already implements and
//! tests — not a new protocol — so unlike group-epoch crypto, this is
//! safe to implement and test now rather than defer.

use crate::{CryptoError, DeviceIdentity};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};

/// What actually gets signed: the new device's two public keys bound
/// together, so a certificate can't be replayed to vouch for a
/// mismatched (verifying_key, x25519_public) pair.
fn signing_payload(
    new_device_verifying_key: &[u8; 32],
    new_device_x25519_public: &[u8; 32],
) -> Vec<u8> {
    let mut payload = Vec::with_capacity(64);
    payload.extend_from_slice(new_device_verifying_key);
    payload.extend_from_slice(new_device_x25519_public);
    payload
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCertificate {
    pub new_device_verifying_key: [u8; 32],
    pub new_device_x25519_public: [u8; 32],
    pub signer_verifying_key: [u8; 32],
    /// 64 raw signature bytes. `Vec<u8>` rather than `[u8; 64]` purely
    /// because serde's derive doesn't support arrays longer than 32
    /// elements without an extra crate (confirmed by a real compile
    /// error, not a guess) — `issue_device_certificate`/
    /// `verify_device_certificate` are what actually enforce it's a
    /// valid 64-byte Ed25519 signature.
    pub signature: Vec<u8>,
}

/// Called on the already-trusted device during linking (plan.md §42):
/// signs the new device's keys with our own `DeviceIdentity`.
pub fn issue_device_certificate(
    signer: &DeviceIdentity,
    new_device_verifying_key: [u8; 32],
    new_device_x25519_public: [u8; 32],
) -> DeviceCertificate {
    let payload = signing_payload(&new_device_verifying_key, &new_device_x25519_public);
    let signature: Signature = signer.sign(&payload);
    DeviceCertificate {
        new_device_verifying_key,
        new_device_x25519_public,
        signer_verifying_key: signer.verifying_key().to_bytes(),
        signature: signature.to_bytes().to_vec(),
    }
}

/// Called by anyone who already trusts `expected_signer` and wants to
/// know whether they should now also trust the device this certificate
/// names (plan.md §68: verify remote-influenced values before trusting
/// them — a `DeviceCertificate` arriving over the network is exactly
/// that).
pub fn verify_device_certificate(
    cert: &DeviceCertificate,
    expected_signer: &VerifyingKey,
) -> Result<(), CryptoError> {
    if cert.signer_verifying_key != expected_signer.to_bytes() {
        return Err(CryptoError::InvalidSignature);
    }
    let payload = signing_payload(
        &cert.new_device_verifying_key,
        &cert.new_device_x25519_public,
    );
    let signature_bytes: [u8; 64] = cert
        .signature
        .as_slice()
        .try_into()
        .map_err(|_| CryptoError::MalformedKey)?;
    let signature = Signature::from_bytes(&signature_bytes);
    expected_signer
        .verify(&payload, &signature)
        .map_err(|_| CryptoError::InvalidSignature)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_valid_certificate_verifies_against_its_real_signer() {
        let existing_device = DeviceIdentity::generate();
        let new_device = DeviceIdentity::generate();

        let cert = issue_device_certificate(
            &existing_device,
            new_device.verifying_key().to_bytes(),
            new_device.x25519_public().to_bytes(),
        );

        assert!(verify_device_certificate(&cert, &existing_device.verifying_key()).is_ok());
    }

    #[test]
    fn a_certificate_does_not_verify_against_the_wrong_signer() {
        let existing_device = DeviceIdentity::generate();
        let impostor = DeviceIdentity::generate();
        let new_device = DeviceIdentity::generate();

        let cert = issue_device_certificate(
            &existing_device,
            new_device.verifying_key().to_bytes(),
            new_device.x25519_public().to_bytes(),
        );

        assert!(verify_device_certificate(&cert, &impostor.verifying_key()).is_err());
    }

    #[test]
    fn tampering_with_the_certified_key_invalidates_the_signature() {
        let existing_device = DeviceIdentity::generate();
        let new_device = DeviceIdentity::generate();
        let attacker_device = DeviceIdentity::generate();

        let mut cert = issue_device_certificate(
            &existing_device,
            new_device.verifying_key().to_bytes(),
            new_device.x25519_public().to_bytes(),
        );
        // Try to splice in a different device's key while keeping the
        // original signature — must not verify.
        cert.new_device_verifying_key = attacker_device.verifying_key().to_bytes();

        assert!(verify_device_certificate(&cert, &existing_device.verifying_key()).is_err());
    }

    #[test]
    fn cannot_forge_a_signer_field_without_the_real_signature() {
        let existing_device = DeviceIdentity::generate();
        let impostor = DeviceIdentity::generate();
        let new_device = DeviceIdentity::generate();

        let mut cert = issue_device_certificate(
            &impostor,
            new_device.verifying_key().to_bytes(),
            new_device.x25519_public().to_bytes(),
        );
        // Claim it was issued by `existing_device` instead of `impostor`
        // — the signature itself is still impostor's, so this must fail.
        cert.signer_verifying_key = existing_device.verifying_key().to_bytes();

        assert!(verify_device_certificate(&cert, &existing_device.verifying_key()).is_err());
    }
}
