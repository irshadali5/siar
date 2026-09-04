//! §25 "Device Revocation", §26 "Revocation Semantics" (the
//! prevention-check half — see [`crate::directory::DeviceDirectory::is_device_trusted`]
//! for the other half), §27 "Immediate Local Revocation", §28
//! "Offline Revocation Propagation".
//!
//! Before this module, [`crate::directory::DeviceStatus::Revoked`] was
//! a value a directory *could* hold, and
//! [`crate::trust_store::TrustedAccountStore`] already rejected a stale
//! directory trying to un-revoke a device (§29, tested since the
//! original session) — but nothing actually *produced* a revocation.
//! [`revoke_device`] is that missing middle piece: §25's whole flow
//! ("Trusted device → Revoke Device B → signed revocation event →
//! generation increments") as one real function.

use crate::directory::{DeviceDirectory, DeviceDirectoryEntry, DeviceStatus};
use crate::error::IdentityError;
use crate::root_key::RootIdentityKey;
use siar_domain::DeviceId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum RevocationError {
    #[error("device {0:?} is not in the current directory — nothing to revoke")]
    DeviceNotFound(DeviceId),
    #[error("device {0:?} is already revoked")]
    AlreadyRevoked(DeviceId),
}

/// §25's flow, minus the "sync to all devices" step (that's §28
/// propagation — transport this crate doesn't own, see its own top doc
/// comment) and "future sessions reject Device B" (that's
/// [`crate::directory::DeviceDirectory::is_device_trusted`], called by
/// whatever does session establishment).
///
/// §27 "Immediate Local Revocation": "Do not wait for a server round
/// trip." This function is synchronous and local — it needs nothing
/// but `current` and `root_key`, both already in the caller's
/// possession, so a caller invoking this directly on revocation intent
/// (not after some network round trip) already satisfies §27's own
/// requirement by construction. What §27 actually asks *beyond* that —
/// stop sending new data to B, remove B from future fan-out, update
/// local authorization — happens the instant a caller starts consulting
/// the *returned* directory (via `is_device_trusted`/`active_devices`)
/// instead of `current` for its next decision; this function doesn't
/// (and structurally can't) force that swap, but it's the only thing
/// standing between "revoked" and "still trusted."
///
/// §24: the returned directory's generation is `current.generation +
/// 1` — never supplied by the caller, so there's no way to call this
/// and accidentally produce a directory the account's own history
/// would consider stale.
pub fn revoke_device(
    root_key: &RootIdentityKey,
    current: &DeviceDirectory,
    device_to_revoke: DeviceId,
) -> Result<DeviceDirectory, RevocationError> {
    let entry = current
        .devices
        .iter()
        .find(|d| d.device_id == device_to_revoke)
        .ok_or(RevocationError::DeviceNotFound(device_to_revoke))?;
    if entry.status == DeviceStatus::Revoked {
        return Err(RevocationError::AlreadyRevoked(device_to_revoke));
    }

    let updated_devices: Vec<DeviceDirectoryEntry> = current
        .devices
        .iter()
        .map(|d| {
            if d.device_id == device_to_revoke {
                DeviceDirectoryEntry {
                    device_id: d.device_id,
                    certificate: d.certificate.clone(),
                    status: DeviceStatus::Revoked,
                    transport_endpoints: vec![],
                }
            } else {
                d.clone()
            }
        })
        .collect();

    Ok(DeviceDirectory::sign(
        root_key,
        current.account_id,
        current.generation + 1,
        updated_devices,
    ))
}

/// Verifies a revocation actually happened correctly — real,
/// independent double-checking rather than trusting
/// [`revoke_device`]'s own return value blindly: confirms the
/// generation really advanced, the target device is really `Revoked`
/// in the new directory, and every other device's status is
/// unchanged (a real, checkable form of §13-style "only what should
/// change, changed" — borrowed from Part 06's own bundle-immutability
/// reasoning, applied here to a directory update instead of a bundle
/// hop).
pub fn verify_revocation(
    before: &DeviceDirectory,
    after: &DeviceDirectory,
    revoked_device: DeviceId,
) -> Result<(), IdentityError> {
    if after.generation <= before.generation {
        return Err(IdentityError::RollbackRejected {
            given: after.generation,
            highest: before.generation,
        });
    }
    let now_revoked = after.devices.iter().find(|d| d.device_id == revoked_device);
    match now_revoked {
        Some(entry) if entry.status == DeviceStatus::Revoked => {}
        _ => return Err(IdentityError::RevocationMismatch),
    }
    for before_entry in &before.devices {
        if before_entry.device_id == revoked_device {
            continue;
        }
        let still_present = after
            .devices
            .iter()
            .any(|d| d.device_id == before_entry.device_id && d.status == before_entry.status);
        if !still_present {
            return Err(IdentityError::RevocationMismatch);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::DeviceCapabilitySet;
    use crate::certificate::DeviceCertificate;
    use crate::trust_store::TrustedAccountStore;
    use siar_domain::AccountId;

    fn two_device_directory(
        root: &RootIdentityKey,
        account: siar_domain::AccountId,
    ) -> (DeviceDirectory, DeviceId, DeviceId) {
        let device_a = DeviceId::new();
        let device_b = DeviceId::new();
        let cert_a = DeviceCertificate::issue(
            root,
            account,
            device_a,
            [1u8; 32],
            0,
            None,
            DeviceCapabilitySet::SEND_MESSAGE,
            1,
        );
        let cert_b = DeviceCertificate::issue(
            root,
            account,
            device_b,
            [2u8; 32],
            0,
            None,
            DeviceCapabilitySet::SEND_MESSAGE,
            1,
        );
        let directory = DeviceDirectory::sign(
            root,
            account,
            1,
            vec![
                DeviceDirectoryEntry {
                    device_id: device_a,
                    certificate: cert_a,
                    status: DeviceStatus::Active,
                    transport_endpoints: vec![],
                },
                DeviceDirectoryEntry {
                    device_id: device_b,
                    certificate: cert_b,
                    status: DeviceStatus::Active,
                    transport_endpoints: vec![],
                },
            ],
        );
        (directory, device_a, device_b)
    }

    #[test]
    fn revoking_a_device_produces_a_correctly_signed_next_generation() {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let (directory, device_a, device_b) = two_device_directory(&root, account);

        let revoked = revoke_device(&root, &directory, device_b).unwrap();
        assert_eq!(revoked.generation, 2); // §24: 42 -> 43, here 1 -> 2
        assert!(revoked.verify_signature(&root.root_public_key()).is_ok());
        assert!(revoked.is_device_trusted(device_a)); // untouched
        assert!(!revoked.is_device_trusted(device_b)); // §26
        verify_revocation(&directory, &revoked, device_b).unwrap();
    }

    #[test]
    fn revoking_an_unknown_device_is_a_real_error() {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let (directory, _, _) = two_device_directory(&root, account);
        let result = revoke_device(&root, &directory, DeviceId::new());
        assert!(matches!(result, Err(RevocationError::DeviceNotFound(_))));
    }

    #[test]
    fn revoking_an_already_revoked_device_is_a_real_error_not_a_silent_no_op() {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let (directory, _, device_b) = two_device_directory(&root, account);
        let once_revoked = revoke_device(&root, &directory, device_b).unwrap();
        let result = revoke_device(&root, &once_revoked, device_b);
        assert!(matches!(result, Err(RevocationError::AlreadyRevoked(_))));
    }

    /// The full realistic flow end to end: revoke, accept the result
    /// into a trust store, and confirm §29's own already-tested
    /// rollback protection (from the original session) now actually
    /// engages against a *real* revocation this module produced,
    /// not a hand-built test fixture standing in for one.
    #[test]
    fn a_real_revocation_is_accepted_and_a_stale_pre_revocation_directory_is_then_rejected() {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let (directory, _, device_b) = two_device_directory(&root, account);
        let mut store = TrustedAccountStore::new();
        store
            .accept(directory.clone(), &root.root_public_key())
            .unwrap();

        let revoked = revoke_device(&root, &directory, device_b).unwrap();
        store.accept(revoked, &root.root_public_key()).unwrap();

        // Device C, having been offline, tries to sync the stale
        // pre-revocation directory where B is still Active — §29's
        // own scenario, now exercised against this module's real
        // output instead of a fixture.
        let result = store.accept(directory, &root.root_public_key());
        assert!(result.is_err());
        assert!(!store
            .directory_for(account)
            .unwrap()
            .is_device_trusted(device_b));
    }
}
