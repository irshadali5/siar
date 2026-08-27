//! §16 "Destination Resolution", §17 "Account-Level Routing": "Part 02
//! provides device membership." A real dependency edge, not a
//! placeholder — this module calls into the actual
//! `siar-identity-multidevice` crate this same workspace already has.

use siar_domain::DeviceId;
use siar_identity_multidevice::TrustedAccountStore;

use crate::error::RoutingError;
use crate::types::Destination;

/// §16's flow: `AccountId → Device Directory → active devices → known
/// transport endpoints → candidate paths`. This function covers the
/// first three steps; turning a [`DeviceId`] into actual candidate
/// paths (the last step) is [`crate::candidate::PathCandidate`]
/// construction, which needs live discovery data this crate doesn't
/// have (see this crate's own top doc comment on scope) — that's the
/// caller's job, done once per device this function returns.
///
/// `Destination::Group` is deliberately NOT resolved here: group
/// membership is `siar-messaging::GroupService`'s concern (its own
/// `GroupState`/MLS member list), not `TrustedAccountStore`'s — adding
/// a dependency on `siar-messaging` from this crate for that would be
/// a real, separate integration decision, not a mechanical extension of
/// this function. Returns [`RoutingError::NoActiveDevicesForAccount`]
/// for a group destination rather than silently returning an empty
/// list, so the gap is visible at the call site instead of looking like
/// "this account just has no devices."
pub fn resolve_destination_devices(
    destination: Destination,
    trust_store: &TrustedAccountStore,
) -> Result<Vec<DeviceId>, RoutingError> {
    match destination {
        Destination::Device(device_id) => Ok(vec![device_id]),
        Destination::Account(account_id) => {
            let directory = trust_store
                .directory_for(account_id)
                .ok_or(RoutingError::NoActiveDevicesForAccount)?;
            let devices: Vec<DeviceId> = directory
                .active_devices()
                .map(|entry| entry.device_id)
                .collect();
            if devices.is_empty() {
                return Err(RoutingError::NoActiveDevicesForAccount);
            }
            Ok(devices)
        }
        Destination::Group(_) => Err(RoutingError::NoActiveDevicesForAccount),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use siar_domain::AccountId;
    use siar_identity_multidevice::{
        DeviceCapabilitySet, DeviceCertificate, DeviceDirectory, DeviceDirectoryEntry,
        DeviceStatus, RootIdentityKey,
    };

    #[test]
    fn an_account_destination_resolves_to_its_active_devices_only() {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let active_device = DeviceId::new();
        let revoked_device = DeviceId::new();

        let active_cert = DeviceCertificate::issue(
            &root,
            account,
            active_device,
            [1u8; 32],
            0,
            None,
            DeviceCapabilitySet::SEND_MESSAGE,
            1,
        );
        let revoked_cert = DeviceCertificate::issue(
            &root,
            account,
            revoked_device,
            [2u8; 32],
            0,
            None,
            DeviceCapabilitySet::SEND_MESSAGE,
            1,
        );
        let directory = DeviceDirectory::sign(
            &root,
            account,
            1,
            vec![
                DeviceDirectoryEntry {
                    device_id: active_device,
                    certificate: active_cert,
                    status: DeviceStatus::Active,
                },
                DeviceDirectoryEntry {
                    device_id: revoked_device,
                    certificate: revoked_cert,
                    status: DeviceStatus::Revoked,
                },
            ],
        );

        let mut trust_store = TrustedAccountStore::new();
        trust_store
            .accept(directory, &root.root_public_key())
            .unwrap();

        let devices =
            resolve_destination_devices(Destination::Account(account), &trust_store).unwrap();
        assert_eq!(devices, vec![active_device]);
    }

    #[test]
    fn an_account_with_no_trusted_directory_is_a_real_error_not_an_empty_list() {
        let trust_store = TrustedAccountStore::new();
        let result =
            resolve_destination_devices(Destination::Account(AccountId::new()), &trust_store);
        assert_eq!(result, Err(RoutingError::NoActiveDevicesForAccount));
    }

    #[test]
    fn a_device_destination_resolves_to_itself_without_touching_the_trust_store() {
        let trust_store = TrustedAccountStore::new();
        let device = DeviceId::new();
        let devices =
            resolve_destination_devices(Destination::Device(device), &trust_store).unwrap();
        assert_eq!(devices, vec![device]);
    }
}
