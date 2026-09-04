//! §36 "Recovery Architecture", §37 "Recovery Policy Type", §38
//! "Recovery Secret", §39 "Recovery Device Addition".

use crate::capability::DeviceCapabilitySet;
use crate::certificate::DeviceCertificate;
use crate::directory::{DeviceDirectory, DeviceDirectoryEntry, DeviceStatus};
use crate::root_key::RootIdentityKey;
use siar_domain::DeviceId;
use zeroize::Zeroize;

/// §37, verbatim four-variant policy. `RecoverySecret`'s `commitment`
/// is `blake3(derived_key)`, never the derived key itself — a stolen
/// directory (which is not secret; §5 says the directory is public
/// account state) must not hand an attacker anything closer to the
/// real recovery secret than a one-way commitment to check a guess
/// against.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RecoveryPolicy {
    None,
    RecoverySecret { commitment: [u8; 32] },
    TrustedDeviceQuorum { threshold: u8 },
    OrganizationManaged,
}

/// §38: "the secret itself never touches the network. Only a derived
/// recovery key does." Deliberately NOT `Serialize`/`Deserialize` —
/// unlike almost every other type in this crate, which exists
/// specifically to be signed, stored, or sent — so there is no
/// `postcard`/`serde` path anywhere that could accidentally put this
/// on the wire or in a persisted blob. Zeroizes on drop, matching
/// [`RootIdentityKey`]'s own defense-in-depth pattern for the same
/// reason: this is exactly as sensitive as the root key it can
/// reconstruct access to.
pub struct RecoverySecret(String);

impl RecoverySecret {
    pub fn new(secret: String) -> Self {
        Self(secret)
    }

    pub fn expose_for_derivation(&self) -> &str {
        &self.0
    }
}

impl Drop for RecoverySecret {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

/// §38: "must use a strong KDF: Argon2id or similar." This crate is
/// deliberately dependency-minimal (see `Cargo.toml`'s own comment —
/// no `siar-crypto`, identity *policy* only) and has no memory-hard
/// KDF of its own to add without pulling in real cryptographic
/// primitives that belong in `siar-crypto` instead. This trait is the
/// boundary: an implementation living wherever the real KDF is
/// (Argon2id via `siar-crypto`, most likely) derives a
/// [`DerivedRecoveryKey`] from a [`RecoverySecret`]; this crate only
/// defines what a derivation looks like and how the result gets used
/// afterward, never the KDF math itself.
pub trait RecoveryKeyDerivation {
    fn derive(&self, secret: &RecoverySecret) -> DerivedRecoveryKey;
}

/// §38's other half — the ONE thing allowed to leave the device: a
/// fixed-size derived key, `Serialize`/`Deserialize` on purpose (this
/// is what's meant to travel).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DerivedRecoveryKey(pub [u8; 32]);

impl DerivedRecoveryKey {
    /// A commitment for [`RecoveryPolicy::RecoverySecret`] to store —
    /// never the key itself.
    pub fn commitment(&self) -> [u8; 32] {
        blake3::hash(&self.0).into()
    }
}

/// §37's four policies each need a different shape of proof —
/// verified against a specific [`RecoveryPolicy`] by
/// [`recovery_evidence_satisfies_policy`].
pub enum RecoveryEvidence {
    /// Checked against [`RecoveryPolicy::RecoverySecret`]'s stored
    /// commitment.
    DerivedRecoveryKey(DerivedRecoveryKey),
    /// Checked against [`RecoveryPolicy::TrustedDeviceQuorum`]: one
    /// signature per participating device, each verified against that
    /// device's OWN certificate in the current directory (an
    /// already-revoked or unknown device's "signature" counts for
    /// nothing).
    TrustedDeviceSignatures(Vec<(DeviceId, [u8; 64])>),
    /// `OrganizationManaged`'s trust is established entirely outside
    /// this crate (an external directory/SSO system) — there is no
    /// evidence shape this crate itself can check, only a marker that
    /// the caller has already confirmed it out of band.
    OrganizationConfirmedExternally,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum RecoveryError {
    #[error("this account has no recovery policy configured (RecoveryPolicy::None)")]
    NoRecoveryPolicyConfigured,
    #[error("recovery evidence does not match the account's configured recovery policy")]
    EvidenceDoesNotMatchPolicy,
    #[error("derived recovery key does not match the stored commitment")]
    WrongRecoverySecret,
    #[error("trusted-device quorum not met: needed {needed}, got {got} valid signatures from currently-active devices")]
    QuorumNotMet { needed: u8, got: u8 },
}

fn verify_device_signature(
    device_public_key: &[u8; 32],
    payload: &[u8],
    signature: &[u8; 64],
) -> bool {
    use ed25519_dalek::{Signature, Verifier, VerifyingKey};
    let Ok(verifying_key) = VerifyingKey::from_bytes(device_public_key) else {
        return false;
    };
    let signature = Signature::from_bytes(signature);
    verifying_key.verify(payload, &signature).is_ok()
}

fn recovery_evidence_satisfies_policy(
    policy: &RecoveryPolicy,
    evidence: &RecoveryEvidence,
    current_directory: &DeviceDirectory,
    request_payload: &[u8],
) -> Result<(), RecoveryError> {
    match (policy, evidence) {
        (RecoveryPolicy::None, _) => Err(RecoveryError::NoRecoveryPolicyConfigured),
        (
            RecoveryPolicy::RecoverySecret { commitment },
            RecoveryEvidence::DerivedRecoveryKey(key),
        ) => {
            if &key.commitment() == commitment {
                Ok(())
            } else {
                Err(RecoveryError::WrongRecoverySecret)
            }
        }
        (
            RecoveryPolicy::TrustedDeviceQuorum { threshold },
            RecoveryEvidence::TrustedDeviceSignatures(signatures),
        ) => {
            let valid_count = signatures
                .iter()
                .filter(|(device_id, signature)| {
                    current_directory
                        .devices
                        .iter()
                        .find(|d| d.device_id == *device_id && d.status == DeviceStatus::Active)
                        .map(|d| {
                            verify_device_signature(
                                &d.certificate.device_public_key,
                                request_payload,
                                signature,
                            )
                        })
                        .unwrap_or(false)
                })
                .count() as u8;
            if valid_count >= *threshold {
                Ok(())
            } else {
                Err(RecoveryError::QuorumNotMet {
                    needed: *threshold,
                    got: valid_count,
                })
            }
        }
        (
            RecoveryPolicy::OrganizationManaged,
            RecoveryEvidence::OrganizationConfirmedExternally,
        ) => Ok(()),
        _ => Err(RecoveryError::EvidenceDoesNotMatchPolicy),
    }
}

/// §39: "adds a new device via the SAME certificate-issuance process
/// as normal device linking, using recovery evidence instead of an
/// existing device's approval. Not a distinct wire protocol." Enforced
/// structurally, not just claimed: this function's body issues a
/// [`DeviceCertificate`] via the exact same
/// [`DeviceCertificate::issue`] call and the exact same
/// generation-advances-by-one directory update
/// [`crate::rotation::rotate_device_key`]/[`crate::revocation::revoke_device`]
/// already use — the only thing specific to recovery is the
/// `recovery_evidence_satisfies_policy` check gating entry into that
/// shared path, in place of an existing device's sign-off.
#[allow(clippy::too_many_arguments)]
pub fn add_device_via_recovery(
    root_key: &RootIdentityKey,
    current: &DeviceDirectory,
    policy: &RecoveryPolicy,
    evidence: &RecoveryEvidence,
    new_device_id: DeviceId,
    new_device_public_key: [u8; 32],
    capabilities: DeviceCapabilitySet,
    expires_at_millis: Option<u64>,
    now_millis: u64,
) -> Result<DeviceDirectory, RecoveryError> {
    let request_payload = new_device_public_key; // what a quorum is actually attesting to
    recovery_evidence_satisfies_policy(policy, evidence, current, &request_payload)?;

    let new_generation = current.generation + 1;
    let new_certificate = DeviceCertificate::issue(
        root_key,
        current.account_id,
        new_device_id,
        new_device_public_key,
        now_millis,
        expires_at_millis,
        capabilities,
        new_generation,
    );

    let mut devices: Vec<DeviceDirectoryEntry> = current.devices.clone();
    devices.push(DeviceDirectoryEntry {
        device_id: new_device_id,
        certificate: new_certificate,
        status: DeviceStatus::Active,
        transport_endpoints: vec![],
    });

    Ok(DeviceDirectory::sign(
        root_key,
        current.account_id,
        new_generation,
        devices,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use siar_domain::AccountId;

    fn empty_directory(root: &RootIdentityKey, account: AccountId) -> DeviceDirectory {
        DeviceDirectory::sign(root, account, 0, vec![])
    }

    #[test]
    fn spec_38_recovery_secret_never_serializes() {
        // Structural claim: RecoverySecret has no Serialize impl at
        // all, so `postcard::to_allocvec(&secret)` wouldn't even
        // compile — this test's only job is documenting that fact is
        // load-bearing, not incidental.
        let secret = RecoverySecret::new("correct horse battery staple".to_string());
        assert_eq!(
            secret.expose_for_derivation(),
            "correct horse battery staple"
        );
    }

    #[test]
    fn spec_38_commitment_never_equals_the_key_it_commits_to() {
        let key = DerivedRecoveryKey([7u8; 32]);
        assert_ne!(key.commitment(), key.0);
    }

    #[test]
    fn spec_39_correct_recovery_secret_adds_the_device() {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let directory = empty_directory(&root, account);

        let correct_key = DerivedRecoveryKey([3u8; 32]);
        let policy = RecoveryPolicy::RecoverySecret {
            commitment: correct_key.commitment(),
        };
        let evidence = RecoveryEvidence::DerivedRecoveryKey(correct_key);
        let new_device = DeviceId::new();

        let recovered = add_device_via_recovery(
            &root,
            &directory,
            &policy,
            &evidence,
            new_device,
            [5u8; 32],
            DeviceCapabilitySet::SEND_MESSAGE,
            None,
            1_000,
        )
        .unwrap();

        assert_eq!(recovered.generation, 1);
        assert!(recovered.is_device_trusted(new_device));
    }

    #[test]
    fn spec_38_wrong_recovery_secret_is_rejected() {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let directory = empty_directory(&root, account);

        let correct_key = DerivedRecoveryKey([3u8; 32]);
        let policy = RecoveryPolicy::RecoverySecret {
            commitment: correct_key.commitment(),
        };
        let wrong_evidence = RecoveryEvidence::DerivedRecoveryKey(DerivedRecoveryKey([9u8; 32]));

        let result = add_device_via_recovery(
            &root,
            &directory,
            &policy,
            &wrong_evidence,
            DeviceId::new(),
            [5u8; 32],
            DeviceCapabilitySet::SEND_MESSAGE,
            None,
            1_000,
        );
        assert_eq!(result.unwrap_err(), RecoveryError::WrongRecoverySecret);
    }

    #[test]
    fn spec_37_no_recovery_policy_configured_always_rejects() {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let directory = empty_directory(&root, account);

        let result = add_device_via_recovery(
            &root,
            &directory,
            &RecoveryPolicy::None,
            &RecoveryEvidence::OrganizationConfirmedExternally,
            DeviceId::new(),
            [5u8; 32],
            DeviceCapabilitySet::SEND_MESSAGE,
            None,
            1_000,
        );
        assert_eq!(
            result.unwrap_err(),
            RecoveryError::NoRecoveryPolicyConfigured
        );
    }

    #[test]
    fn spec_37_trusted_device_quorum_requires_enough_valid_signatures_from_active_devices() {
        use ed25519_dalek::{Signer, SigningKey};
        use rand_core::OsRng;

        let root = RootIdentityKey::generate();
        let account = AccountId::new();

        // Two existing trusted devices, each with a real signing key.
        let device_a_key = SigningKey::generate(&mut OsRng);
        let device_a_id = DeviceId::new();
        let device_a_cert = DeviceCertificate::issue(
            &root,
            account,
            device_a_id,
            device_a_key.verifying_key().to_bytes(),
            0,
            None,
            DeviceCapabilitySet::SEND_MESSAGE,
            0,
        );
        let device_b_key = SigningKey::generate(&mut OsRng);
        let device_b_id = DeviceId::new();
        let device_b_cert = DeviceCertificate::issue(
            &root,
            account,
            device_b_id,
            device_b_key.verifying_key().to_bytes(),
            0,
            None,
            DeviceCapabilitySet::SEND_MESSAGE,
            0,
        );
        let directory = DeviceDirectory::sign(
            &root,
            account,
            0,
            vec![
                DeviceDirectoryEntry {
                    device_id: device_a_id,
                    certificate: device_a_cert,
                    status: DeviceStatus::Active,
                    transport_endpoints: vec![],
                },
                DeviceDirectoryEntry {
                    device_id: device_b_id,
                    certificate: device_b_cert,
                    status: DeviceStatus::Active,
                    transport_endpoints: vec![],
                },
            ],
        );

        let policy = RecoveryPolicy::TrustedDeviceQuorum { threshold: 2 };
        let new_device_public_key = [8u8; 32];

        // Only one of the two signs — quorum of 2 not met.
        let one_signature = RecoveryEvidence::TrustedDeviceSignatures(vec![(
            device_a_id,
            device_a_key.sign(&new_device_public_key).to_bytes(),
        )]);
        let result = add_device_via_recovery(
            &root,
            &directory,
            &policy,
            &one_signature,
            DeviceId::new(),
            new_device_public_key,
            DeviceCapabilitySet::SEND_MESSAGE,
            None,
            1_000,
        );
        assert_eq!(
            result.unwrap_err(),
            RecoveryError::QuorumNotMet { needed: 2, got: 1 }
        );

        // Both sign — quorum met, device added.
        let both_signatures = RecoveryEvidence::TrustedDeviceSignatures(vec![
            (
                device_a_id,
                device_a_key.sign(&new_device_public_key).to_bytes(),
            ),
            (
                device_b_id,
                device_b_key.sign(&new_device_public_key).to_bytes(),
            ),
        ]);
        let recovered = add_device_via_recovery(
            &root,
            &directory,
            &policy,
            &both_signatures,
            DeviceId::new(),
            new_device_public_key,
            DeviceCapabilitySet::SEND_MESSAGE,
            None,
            1_000,
        )
        .unwrap();
        assert_eq!(recovered.generation, 1);
    }

    #[test]
    fn spec_37_a_revoked_devices_signature_does_not_count_toward_quorum() {
        use ed25519_dalek::{Signer, SigningKey};
        use rand_core::OsRng;

        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let revoked_key = SigningKey::generate(&mut OsRng);
        let revoked_id = DeviceId::new();
        let revoked_cert = DeviceCertificate::issue(
            &root,
            account,
            revoked_id,
            revoked_key.verifying_key().to_bytes(),
            0,
            None,
            DeviceCapabilitySet::SEND_MESSAGE,
            0,
        );
        let directory = DeviceDirectory::sign(
            &root,
            account,
            0,
            vec![DeviceDirectoryEntry {
                device_id: revoked_id,
                certificate: revoked_cert,
                status: DeviceStatus::Revoked, // already revoked
                transport_endpoints: vec![],
            }],
        );

        let policy = RecoveryPolicy::TrustedDeviceQuorum { threshold: 1 };
        let new_device_public_key = [8u8; 32];
        let evidence = RecoveryEvidence::TrustedDeviceSignatures(vec![(
            revoked_id,
            revoked_key.sign(&new_device_public_key).to_bytes(),
        )]);

        let result = add_device_via_recovery(
            &root,
            &directory,
            &policy,
            &evidence,
            DeviceId::new(),
            new_device_public_key,
            DeviceCapabilitySet::SEND_MESSAGE,
            None,
            1_000,
        );
        assert_eq!(
            result.unwrap_err(),
            RecoveryError::QuorumNotMet { needed: 1, got: 0 }
        );
    }
}
