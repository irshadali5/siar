//! §66 "Multi-Device File Transfer", §67 "Account Address vs Device
//! Address", §68 "Device Resolution", §69 "Fan-Out Policy", §70
//! "Own-Device Synchronization Policy".

use crate::capability::DeviceCapabilitySet;
use crate::directory::{DeviceDirectory, DeviceEndpoint};
use siar_domain::{AccountId, DeviceId};

/// §67, verbatim enum — §66's own three worked examples ("send to
/// Bob's account," "send only to Bob's laptop," "sync file to all my
/// devices") are exactly `Account`/`Device`/`Devices` respectively.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Destination {
    Account(AccountId),
    Device(DeviceId),
    Devices(Vec<DeviceId>),
}

/// One resolved target: enough for a transport layer to actually
/// reach the device, per §68's flow ending in "transport endpoints."
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedDevice {
    pub device_id: DeviceId,
    pub transport_endpoints: Vec<DeviceEndpoint>,
}

/// §68's flow, implemented exactly: `Destination` → device directory →
/// active AND authorized devices → transport endpoints. "Authorized"
/// here means holding `required_capability`, if the caller supplies
/// one (e.g. `RECEIVE_MESSAGE` for messaging) — §68 doesn't spell out
/// what "authorized" checks against, but this crate already has a real
/// capability system (§64) to check it with, so this uses that rather
/// than only "active."
///
/// §68's own explicit point — "do not make the application manually
/// maintain endpoint lists" — is why this function exists at all: a
/// caller passes a [`Destination`] and gets real, current
/// [`ResolvedDevice`]s back, never asked to track endpoints itself.
pub fn resolve_destination(
    destination: &Destination,
    account_directory: &DeviceDirectory,
    required_capability: Option<DeviceCapabilitySet>,
) -> Vec<ResolvedDevice> {
    let authorized = |device_id: DeviceId| {
        account_directory
            .active_devices()
            .find(|d| d.device_id == device_id)
            .filter(|d| {
                required_capability
                    .map(|cap| d.certificate.capabilities.contains(cap))
                    .unwrap_or(true)
            })
    };

    match destination {
        Destination::Account(_) => account_directory
            .active_devices()
            .filter(|d| {
                required_capability
                    .map(|cap| d.certificate.capabilities.contains(cap))
                    .unwrap_or(true)
            })
            .map(|d| ResolvedDevice {
                device_id: d.device_id,
                transport_endpoints: d.transport_endpoints.clone(),
            })
            .collect(),
        Destination::Device(device_id) => authorized(*device_id)
            .map(|d| ResolvedDevice {
                device_id: d.device_id,
                transport_endpoints: d.transport_endpoints.clone(),
            })
            .into_iter()
            .collect(),
        Destination::Devices(device_ids) => device_ids
            .iter()
            .filter_map(|&id| authorized(id))
            .map(|d| ResolvedDevice {
                device_id: d.device_id,
                transport_endpoints: d.transport_endpoints.clone(),
            })
            .collect(),
    }
}

/// §69's own five policies, verbatim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FanOutPolicy {
    AllActiveDevices,
    PrimaryDeviceOnly,
    BestReachableDevice,
    AtLeastOneDevice,
    ApplicationDefinedSubset,
}

/// §69's own two worked examples, as real defaults rather than only
/// prose — messaging and large files each get the policy spec's text
/// actually names for them.
pub fn messaging_default_fan_out_policy() -> FanOutPolicy {
    FanOutPolicy::AllActiveDevices
}

pub fn large_file_default_fan_out_policy() -> FanOutPolicy {
    FanOutPolicy::ApplicationDefinedSubset // "specific device" — an application choice, not a fixed rule
}

/// §70's own five data-type-to-target examples — a distinct axis from
/// [`crate::fanout::OwnDeviceSyncPolicy`] (§44): that type answers
/// "should this data class be trusted/synced to other devices at
/// all," this one answers "which devices, once trust says yes."
/// Reuses [`crate::fanout::SyncDataClass`] rather than a second,
/// competing enum for the same five-or-six data kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SyncTarget {
    AllFullDevices,
    SelectedDevices,
    OnDemand,
    Optional,
}

/// §70's own worked example mapping, verbatim.
pub fn spec_70_example_target(class: crate::fanout::SyncDataClass) -> SyncTarget {
    use crate::fanout::SyncDataClass;
    match class {
        SyncDataClass::SentMessages => SyncTarget::AllFullDevices, // "messages"
        SyncDataClass::Settings => SyncTarget::SelectedDevices,
        SyncDataClass::Drafts => SyncTarget::Optional,
        // "large files" isn't itself a SyncDataClass variant (files
        // are their own extension, §73/§10 of Part 01) — ReadState and
        // Contacts/GroupState aren't named in §70's own example either,
        // so they default to the same on-demand posture large files
        // gets, rather than guessing a specific answer §70 never gives.
        SyncDataClass::ReadState | SyncDataClass::Contacts | SyncDataClass::GroupState => {
            SyncTarget::OnDemand
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::certificate::DeviceCertificate;
    use crate::directory::{DeviceDirectoryEntry, DeviceStatus};
    use crate::root_key::RootIdentityKey;

    fn directory_with(
        root: &RootIdentityKey,
        account: AccountId,
        devices: &[(DeviceId, DeviceCapabilitySet, DeviceStatus)],
    ) -> DeviceDirectory {
        let entries = devices
            .iter()
            .map(|&(id, caps, status)| DeviceDirectoryEntry {
                device_id: id,
                certificate: DeviceCertificate::issue(
                    root, account, id, [1u8; 32], 0, None, caps, 0,
                ),
                status,
                transport_endpoints: vec![DeviceEndpoint(vec![9, 9])],
            })
            .collect();
        DeviceDirectory::sign(root, account, 0, entries)
    }

    #[test]
    fn spec_67_68_account_destination_resolves_to_every_active_device() {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let phone = DeviceId::new();
        let laptop = DeviceId::new();
        let directory = directory_with(
            &root,
            account,
            &[
                (
                    phone,
                    DeviceCapabilitySet::RECEIVE_MESSAGE,
                    DeviceStatus::Active,
                ),
                (
                    laptop,
                    DeviceCapabilitySet::RECEIVE_MESSAGE,
                    DeviceStatus::Active,
                ),
            ],
        );

        let resolved = resolve_destination(&Destination::Account(account), &directory, None);
        assert_eq!(resolved.len(), 2);
        assert!(
            !resolved[0].transport_endpoints.is_empty(),
            "endpoints must come along, not be re-derived by the caller"
        );
    }

    #[test]
    fn spec_68_account_destination_filters_by_required_capability() {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let full_device = DeviceId::new();
        let relay_only_device = DeviceId::new();
        let directory = directory_with(
            &root,
            account,
            &[
                (
                    full_device,
                    DeviceCapabilitySet::RECEIVE_MESSAGE,
                    DeviceStatus::Active,
                ),
                (
                    relay_only_device,
                    DeviceCapabilitySet::RELAY,
                    DeviceStatus::Active,
                ),
            ],
        );

        let resolved = resolve_destination(
            &Destination::Account(account),
            &directory,
            Some(DeviceCapabilitySet::RECEIVE_MESSAGE),
        );
        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0].device_id, full_device);
    }

    #[test]
    fn spec_67_a_revoked_device_destination_resolves_to_nothing() {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let revoked = DeviceId::new();
        let directory = directory_with(
            &root,
            account,
            &[(
                revoked,
                DeviceCapabilitySet::RECEIVE_MESSAGE,
                DeviceStatus::Revoked,
            )],
        );

        let resolved = resolve_destination(&Destination::Device(revoked), &directory, None);
        assert!(resolved.is_empty());
    }

    #[test]
    fn spec_69_messaging_and_files_get_their_own_named_default_policies() {
        assert_eq!(
            messaging_default_fan_out_policy(),
            FanOutPolicy::AllActiveDevices
        );
        assert_eq!(
            large_file_default_fan_out_policy(),
            FanOutPolicy::ApplicationDefinedSubset
        );
    }

    #[test]
    fn spec_70_worked_examples_match_the_spec() {
        use crate::fanout::SyncDataClass;
        assert_eq!(
            spec_70_example_target(SyncDataClass::SentMessages),
            SyncTarget::AllFullDevices
        );
        assert_eq!(
            spec_70_example_target(SyncDataClass::Settings),
            SyncTarget::SelectedDevices
        );
        assert_eq!(
            spec_70_example_target(SyncDataClass::Drafts),
            SyncTarget::Optional
        );
    }
}
