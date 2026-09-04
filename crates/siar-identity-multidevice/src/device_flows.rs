//! §76 "Lost Device Flow", §77 "Compromised Device Flow", §78 "Device
//! Reinstallation", §79 "Device Migration".

use crate::capability::DeviceCapabilitySet;
use crate::directory::DeviceDirectory;
use crate::error::IdentityError;
use crate::revocation::revoke_device;
use crate::root_key::RootIdentityKey;
use siar_domain::DeviceId;

/// §76's own four-step list, verbatim. This crate can only actually
/// perform the first step (`revoke_device` is real, right here); the
/// other three are `siar-messaging`/`siar-crypto-mls`'s job
/// (conversation/group key rotation, session invalidation, fan-out
/// membership) — [`LostDeviceOutcome`] names all four so a caller
/// sees the complete checklist, not just the part this crate did.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LostDeviceStep {
    RevokeLostDevice,
    RotateAffectedConversationOrGroupState,
    InvalidateDeviceSessions,
    RemoveDeviceFromFanOut,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LostDeviceOutcome {
    pub new_directory: DeviceDirectory,
    /// §76: "the exact data-key rotation can occur asynchronously" —
    /// these are the steps this call did NOT perform, named so a
    /// caller can actually schedule them rather than assume revocation
    /// alone was sufficient.
    pub remaining_steps: Vec<LostDeviceStep>,
}

/// §76's flow: revoke the lost device now (synchronous, this crate's
/// real job), and hand back exactly which of spec's other three steps
/// still need doing elsewhere.
pub fn handle_lost_device(
    root_key: &RootIdentityKey,
    current: &DeviceDirectory,
    lost_device: DeviceId,
) -> Result<LostDeviceOutcome, IdentityError> {
    let new_directory = revoke_device(root_key, current, lost_device)
        .map_err(|_| IdentityError::AccountMismatch)?;
    Ok(LostDeviceOutcome {
        new_directory,
        remaining_steps: vec![
            LostDeviceStep::RotateAffectedConversationOrGroupState,
            LostDeviceStep::InvalidateDeviceSessions,
            LostDeviceStep::RemoveDeviceFromFanOut,
        ],
    })
}

/// §77's own five-step list, verbatim — one step longer than §76's,
/// matching spec's own framing ("stronger than simple loss").
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CompromisedDeviceStep {
    RevokeDevice,
    RotateAccountSensitiveSessionMaterial,
    RotateGroupMembershipKeys,
    InvalidateDeviceTokens,
    TriggerSecurityWarning,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CompromisedDeviceOutcome {
    pub new_directory: DeviceDirectory,
    /// §77's own closing line, made unavoidable rather than only
    /// documented: this list can NEVER contain a claim that keys are
    /// erased — "do not assume revocation alone erases any keys
    /// already obtained" has no corresponding step anywhere in §77's
    /// own five, and none is invented here either.
    pub remaining_steps: Vec<CompromisedDeviceStep>,
}

pub fn handle_compromised_device(
    root_key: &RootIdentityKey,
    current: &DeviceDirectory,
    compromised_device: DeviceId,
) -> Result<CompromisedDeviceOutcome, IdentityError> {
    let new_directory = revoke_device(root_key, current, compromised_device)
        .map_err(|_| IdentityError::AccountMismatch)?;
    Ok(CompromisedDeviceOutcome {
        new_directory,
        remaining_steps: vec![
            CompromisedDeviceStep::RotateAccountSensitiveSessionMaterial,
            CompromisedDeviceStep::RotateGroupMembershipKeys,
            CompromisedDeviceStep::InvalidateDeviceTokens,
            CompromisedDeviceStep::TriggerSecurityWarning,
        ],
    })
}

/// §79's flow, implemented directly: link a new device identity into
/// the directory, then OPTIONALLY revoke the old one in the same
/// call — matching spec's own diagram ("old device optionally
/// revoked" as the last, conditional step). §79's closing line,
/// "private device keys should still not be copied unless the
/// architecture explicitly defines secure key migration," is enforced
/// by this function's own signature: `new_device_public_key` is a
/// PUBLIC key ([u8; 32]) — there is no parameter anywhere on this
/// function through which a private key could flow, so this crate
/// structurally cannot be the thing that copies one. §78's "reinstall
/// should normally create a new DeviceId, new device key, rather than
/// silently resurrecting the previous identity" is the same claim
/// this function makes for migration too: `new_device_id` must be a
/// genuinely fresh [`DeviceId`], never `old_device_id` reused.
#[allow(clippy::too_many_arguments)]
pub fn migrate_device(
    root_key: &RootIdentityKey,
    current: &DeviceDirectory,
    old_device_id: DeviceId,
    new_device_id: DeviceId,
    new_device_public_key: [u8; 32],
    capabilities: DeviceCapabilitySet,
    expires_at_millis: Option<u64>,
    now_millis: u64,
    revoke_old_device: bool,
) -> Result<DeviceDirectory, IdentityError> {
    if new_device_id == old_device_id {
        // §78/§79 both explicitly forbid resurrecting the old identity
        // under its own id — the same DeviceId is not a "new device
        // identity."
        return Err(IdentityError::AccountMismatch);
    }

    let new_generation = current.generation + 1;
    let new_certificate = crate::certificate::DeviceCertificate::issue(
        root_key,
        current.account_id,
        new_device_id,
        new_device_public_key,
        now_millis,
        expires_at_millis,
        capabilities,
        new_generation,
    );

    let mut devices = current.devices.clone();
    devices.push(crate::directory::DeviceDirectoryEntry {
        device_id: new_device_id,
        certificate: new_certificate,
        status: crate::directory::DeviceStatus::Active,
        transport_endpoints: vec![],
    });

    let with_new_device =
        DeviceDirectory::sign(root_key, current.account_id, new_generation, devices);

    if revoke_old_device {
        revoke_device(root_key, &with_new_device, old_device_id)
            .map_err(|_| IdentityError::AccountMismatch)
    } else {
        Ok(with_new_device)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::certificate::DeviceCertificate;
    use crate::directory::{DeviceDirectoryEntry, DeviceStatus};
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
            None,
            DeviceCapabilitySet::SEND_MESSAGE,
            0,
        );
        let directory = DeviceDirectory::sign(
            root,
            account,
            0,
            vec![DeviceDirectoryEntry {
                device_id: device,
                certificate: cert,
                status: DeviceStatus::Active,
                transport_endpoints: vec![],
            }],
        );
        (directory, device)
    }

    #[test]
    fn spec_76_lost_device_is_revoked_now_with_three_steps_named_remaining() {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let (directory, device) = one_device_directory(&root, account);

        let outcome = handle_lost_device(&root, &directory, device).unwrap();
        assert!(!outcome.new_directory.is_device_trusted(device));
        assert_eq!(outcome.remaining_steps.len(), 3);
        assert!(outcome
            .remaining_steps
            .contains(&LostDeviceStep::InvalidateDeviceSessions));
    }

    #[test]
    fn spec_77_compromised_device_gets_four_remaining_steps_never_claiming_key_erasure() {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let (directory, device) = one_device_directory(&root, account);

        let outcome = handle_compromised_device(&root, &directory, device).unwrap();
        assert!(!outcome.new_directory.is_device_trusted(device));
        assert_eq!(outcome.remaining_steps.len(), 4);
        assert!(outcome
            .remaining_steps
            .contains(&CompromisedDeviceStep::TriggerSecurityWarning));
    }

    #[test]
    fn spec_78_79_reinstallation_or_migration_to_the_same_device_id_is_rejected() {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let (directory, device) = one_device_directory(&root, account);

        let result = migrate_device(
            &root,
            &directory,
            device,
            device,
            [9u8; 32],
            DeviceCapabilitySet::SEND_MESSAGE,
            None,
            1_000,
            false,
        );
        assert!(
            result.is_err(),
            "must not resurrect the previous identity under its own id"
        );
    }

    #[test]
    fn spec_79_migration_links_the_new_device_and_optionally_revokes_the_old_one() {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let (directory, old_device) = one_device_directory(&root, account);
        let new_device = DeviceId::new();

        let migrated = migrate_device(
            &root,
            &directory,
            old_device,
            new_device,
            [9u8; 32],
            DeviceCapabilitySet::SEND_MESSAGE,
            None,
            1_000,
            true,
        )
        .unwrap();

        assert!(migrated.is_device_trusted(new_device));
        assert!(
            !migrated.is_device_trusted(old_device),
            "old device optionally revoked, and this call opted in"
        );
    }

    #[test]
    fn spec_79_migration_can_keep_the_old_device_active_if_the_caller_declines_revocation() {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let (directory, old_device) = one_device_directory(&root, account);
        let new_device = DeviceId::new();

        let migrated = migrate_device(
            &root,
            &directory,
            old_device,
            new_device,
            [9u8; 32],
            DeviceCapabilitySet::SEND_MESSAGE,
            None,
            1_000,
            false,
        )
        .unwrap();

        assert!(migrated.is_device_trusted(new_device));
        assert!(
            migrated.is_device_trusted(old_device),
            "revocation was declined, old device stays active"
        );
    }
}
