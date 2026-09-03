//! §31 "Device Rotation", §32 "Rotation Reasons".
//!
//! Mirrors [`crate::revocation::revoke_device`]'s exact shape: same
//! generation-advances-by-exactly-one discipline (§24), same
//! synchronous/local function a caller invokes directly on rotation
//! intent, same real-error-not-silent-noop handling for a device
//! that's gone or already revoked.
//!
//! One naming note worth being explicit about: §31's own text talks
//! about "device key generation N -> generation N+1" — this is NOT
//! [`crate::directory::DeviceDirectory::generation`] (the account's
//! directory-wide counter, §24) reused under a different name. A
//! rotation is modeled here as exactly what it structurally is: the
//! existing device keeps its `DeviceId`, gets issued a fresh
//! [`crate::certificate::DeviceCertificate`] at the directory's *next*
//! generation (advancing the directory the same way a revocation
//! does), and the old certificate is superseded the moment any caller
//! consults the returned (newer-generation) directory instead of the
//! old one — there is no second, per-device generation counter
//! anywhere in this crate, and this module doesn't invent one.

use crate::certificate::DeviceCertificate;
use crate::directory::{DeviceDirectory, DeviceDirectoryEntry, DeviceStatus};
use crate::root_key::RootIdentityKey;
use siar_domain::DeviceId;

/// §32's own five reasons, verbatim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RotationReason {
    RoutinePolicy,
    SuspectedLocalCompromise,
    OsOrPlatformKeystoreUpgrade,
    AppReinstall,
    HardwareChange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum RotationError {
    #[error("device {0:?} is not in the current directory — nothing to rotate")]
    DeviceNotFound(DeviceId),
    #[error("device {0:?} is revoked — re-link it instead of rotating a dead key")]
    DeviceRevoked(DeviceId),
}

/// §31's flow. "Old keys become invalid for future authentication" is
/// true the instant a verifier is checking against the *returned*
/// (newer) directory rather than `current` — the same structural
/// argument [`crate::revocation::revoke_device`]'s own doc comment
/// makes for "future sessions reject Device B," applied here to "old
/// key stops authenticating" instead.
///
/// The new certificate inherits the old one's `capabilities` and
/// `expires_at_millis` unchanged — a key rotation is not a capability
/// change (that's a separate, unrelated operation this function
/// doesn't perform) and not automatically an expiry extension.
pub fn rotate_device_key(
    root_key: &RootIdentityKey,
    current: &DeviceDirectory,
    device_id: DeviceId,
    new_device_public_key: [u8; 32],
    reason: RotationReason,
    now_millis: u64,
) -> Result<(DeviceDirectory, RotationReason), RotationError> {
    let entry = current
        .devices
        .iter()
        .find(|d| d.device_id == device_id)
        .ok_or(RotationError::DeviceNotFound(device_id))?;
    if entry.status == DeviceStatus::Revoked {
        return Err(RotationError::DeviceRevoked(device_id));
    }

    let new_generation = current.generation + 1;
    let new_certificate = DeviceCertificate::issue(
        root_key,
        current.account_id,
        device_id,
        new_device_public_key,
        now_millis,
        entry.certificate.expires_at_millis,
        entry.certificate.capabilities,
        new_generation,
    );

    let updated_devices: Vec<DeviceDirectoryEntry> = current
        .devices
        .iter()
        .map(|d| {
            if d.device_id == device_id {
                DeviceDirectoryEntry {
                    device_id: d.device_id,
                    certificate: new_certificate.clone(),
                    status: DeviceStatus::Active,
                }
            } else {
                d.clone()
            }
        })
        .collect();

    let rotated = DeviceDirectory::sign(
        root_key,
        current.account_id,
        new_generation,
        updated_devices,
    );
    Ok((rotated, reason))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::DeviceCapabilitySet;
    use siar_domain::AccountId;

    fn one_device_directory(
        root: &RootIdentityKey,
        account: AccountId,
    ) -> (DeviceDirectory, DeviceId) {
        let device = DeviceId::new();
        let cert = DeviceCertificate::issue(
            root,
            account,
            device,
            [1u8; 32],
            0,
            Some(1_000_000),
            DeviceCapabilitySet::SEND_MESSAGE,
            1,
        );
        let directory = DeviceDirectory::sign(
            root,
            account,
            1,
            vec![DeviceDirectoryEntry {
                device_id: device,
                certificate: cert,
                status: DeviceStatus::Active,
            }],
        );
        (directory, device)
    }

    #[test]
    fn rotating_produces_a_correctly_signed_next_generation_with_the_same_device_id() {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let (directory, device) = one_device_directory(&root, account);

        let (rotated, reason) = rotate_device_key(
            &root,
            &directory,
            device,
            [9u8; 32],
            RotationReason::RoutinePolicy,
            1_000,
        )
        .unwrap();

        assert_eq!(rotated.generation, 2);
        assert_eq!(reason, RotationReason::RoutinePolicy);
        assert!(rotated.verify_signature(&root.root_public_key()).is_ok());
        assert!(rotated.is_device_trusted(device)); // same DeviceId, still trusted
        let entry = rotated
            .devices
            .iter()
            .find(|d| d.device_id == device)
            .unwrap();
        assert_eq!(entry.certificate.device_public_key, [9u8; 32]);
    }

    #[test]
    fn rotation_preserves_capabilities_and_expiry_unchanged() {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let (directory, device) = one_device_directory(&root, account);
        let original = directory.devices[0].certificate.clone();

        let (rotated, _) = rotate_device_key(
            &root,
            &directory,
            device,
            [9u8; 32],
            RotationReason::HardwareChange,
            1_000,
        )
        .unwrap();
        let new_entry = rotated
            .devices
            .iter()
            .find(|d| d.device_id == device)
            .unwrap();

        assert_eq!(new_entry.certificate.capabilities, original.capabilities);
        assert_eq!(
            new_entry.certificate.expires_at_millis,
            original.expires_at_millis
        );
        assert_ne!(
            new_entry.certificate.device_public_key,
            original.device_public_key
        );
    }

    #[test]
    fn rotating_an_unknown_device_is_a_real_error() {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let (directory, _) = one_device_directory(&root, account);
        let result = rotate_device_key(
            &root,
            &directory,
            DeviceId::new(),
            [9u8; 32],
            RotationReason::AppReinstall,
            1_000,
        );
        assert!(matches!(result, Err(RotationError::DeviceNotFound(_))));
    }

    #[test]
    fn cannot_rotate_a_revoked_device_key() {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let (directory, device) = one_device_directory(&root, account);
        let revoked = crate::revocation::revoke_device(&root, &directory, device).unwrap();

        let result = rotate_device_key(
            &root,
            &revoked,
            device,
            [9u8; 32],
            RotationReason::SuspectedLocalCompromise,
            1_000,
        );
        assert!(matches!(result, Err(RotationError::DeviceRevoked(_))));
    }
}
