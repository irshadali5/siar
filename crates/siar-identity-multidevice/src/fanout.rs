//! §40 "Multi-Device Messaging Fan-Out", §41 "Sender Attribution", §42
//! "Account-Level Presentation", §43 "Device-Level Receipts", §44
//! "Sync Between User's Own Devices".

use crate::directory::DeviceDirectory;
use siar_domain::{AccountId, DeviceId};

/// §40: "the exact cryptographic fan-out mechanism can evolve, but the
/// identity model must expose device membership." This is that
/// exposure, made concrete rather than left as a principle: every
/// currently-active device that "may need the event" per §40's own
/// worked example — every active device of the recipient, plus every
/// *other* active device of the sender (never the originating device
/// itself, which already has the message it just sent).
pub fn fan_out_targets(
    sender_directory: &DeviceDirectory,
    sender_originating_device: DeviceId,
    recipient_directory: &DeviceDirectory,
) -> Vec<DeviceId> {
    let mut targets: Vec<DeviceId> = recipient_directory
        .active_devices()
        .map(|entry| entry.device_id)
        .collect();
    targets.extend(
        sender_directory
            .active_devices()
            .map(|entry| entry.device_id)
            .filter(|&id| id != sender_originating_device),
    );
    targets
}

/// §41, verbatim struct.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SenderIdentity {
    pub account_id: AccountId,
    pub device_id: DeviceId,
}

/// §42's own four contexts where device attribution "remains
/// available" even though ordinary UI shouldn't show it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PresentationContext {
    Normal,
    SecurityDetails,
    Diagnostics,
    EnterpriseAudit,
    IdentityChangeWarning,
}

/// §42's actual rule as a function, not just a UI convention: given a
/// [`SenderIdentity`] and the context it's being shown in, returns
/// only what that context is allowed to reveal. `Normal` gets the
/// account only — never silently the device too.
pub fn account_level_display(
    sender: SenderIdentity,
    context: PresentationContext,
) -> (AccountId, Option<DeviceId>) {
    match context {
        PresentationContext::Normal => (sender.account_id, None),
        PresentationContext::SecurityDetails
        | PresentationContext::Diagnostics
        | PresentationContext::EnterpriseAudit
        | PresentationContext::IdentityChangeWarning => (sender.account_id, Some(sender.device_id)),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DeviceReceiptStatus {
    Delivered,
    Offline,
    Pending,
}

/// §43's own worked example, verbatim shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DeviceReceipt {
    pub device_id: DeviceId,
    pub status: DeviceReceiptStatus,
}

/// §43: "the core should retain device-level truth" — enforced by
/// construction, not just convention: there is no way to construct an
/// account-level "delivered" summary except by calling
/// [`aggregate_delivered_to_account`] over real [`DeviceReceipt`]s;
/// nothing in this module lets an aggregate be stored or produced on
/// its own, so device-level truth can never silently become the only
/// copy of the data.
pub fn aggregate_delivered_to_account(receipts: &[DeviceReceipt]) -> bool {
    // §43's own example aggregation policy: "Delivered to Bob" as soon
    // as any one of Bob's devices has it — a product could reasonably
    // choose a stricter "all devices" policy instead; this function
    // implements the one example the spec actually gives.
    receipts
        .iter()
        .any(|r| r.status == DeviceReceiptStatus::Delivered)
}

/// §44's own six data classes, verbatim (`Drafts` marked optional in
/// the spec text itself, not a distinction this enum encodes — it's
/// still one more class among equals here, the "optionally" is product
/// policy about whether to sync it at all, not a different kind of
/// class).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum SyncDataClass {
    SentMessages,
    ReadState,
    Contacts,
    GroupState,
    Settings,
    Drafts,
}

/// §44: "do not treat own devices as automatically fully trusted for
/// every application secret. Per-data-class policy is useful." A
/// per-class map is the structural form of that rule — there is no
/// single "trust my other devices: yes/no" flag anywhere in this type
/// for a caller to reach for as a shortcut.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct OwnDeviceSyncPolicy {
    allowed: std::collections::BTreeMap<SyncDataClassKey, bool>,
}

/// `SyncDataClass` isn't `Ord`/`Hash`-friendly as a `BTreeMap` key
/// without deriving more than the type otherwise needs — this newtype
/// keeps the public enum minimal and gives the map a real key type.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub struct SyncDataClassKey(pub SyncDataClass);

impl PartialOrd for SyncDataClass {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for SyncDataClass {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (*self as u8).cmp(&(*other as u8))
    }
}

impl OwnDeviceSyncPolicy {
    pub fn set(&mut self, class: SyncDataClass, allowed: bool) {
        self.allowed.insert(SyncDataClassKey(class), allowed);
    }

    /// Defaults to `false` — an unconfigured data class is NOT
    /// synced, matching §44's "do not treat own devices as
    /// automatically fully trusted" for anything not explicitly
    /// opted in.
    pub fn is_allowed(&self, class: SyncDataClass) -> bool {
        self.allowed
            .get(&SyncDataClassKey(class))
            .copied()
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::DeviceCapabilitySet;
    use crate::certificate::DeviceCertificate;
    use crate::directory::{DeviceDirectoryEntry, DeviceStatus};
    use crate::root_key::RootIdentityKey;

    fn directory_with_devices(
        root: &RootIdentityKey,
        account: AccountId,
        count: usize,
    ) -> (DeviceDirectory, Vec<DeviceId>) {
        let mut ids = Vec::new();
        let mut entries = Vec::new();
        for i in 0..count {
            let device = DeviceId::new();
            ids.push(device);
            let cert = DeviceCertificate::issue(
                root,
                account,
                device,
                [i as u8; 32],
                0,
                None,
                DeviceCapabilitySet::SEND_MESSAGE,
                0,
            );
            entries.push(DeviceDirectoryEntry {
                device_id: device,
                certificate: cert,
                status: DeviceStatus::Active,
            });
        }
        (DeviceDirectory::sign(root, account, 0, entries), ids)
    }

    #[test]
    fn spec_40_fan_out_includes_all_recipient_devices_and_senders_other_devices() {
        let root = RootIdentityKey::generate();
        let alice = AccountId::new();
        let bob = AccountId::new();
        let (alice_directory, alice_devices) = directory_with_devices(&root, alice, 3); // phone, laptop, tablet
        let (bob_directory, bob_devices) = directory_with_devices(&root, bob, 2); // phone, laptop

        let originating = alice_devices[0];
        let targets = fan_out_targets(&alice_directory, originating, &bob_directory);

        assert!(targets.contains(&bob_devices[0]));
        assert!(targets.contains(&bob_devices[1]));
        assert!(targets.contains(&alice_devices[1]));
        assert!(targets.contains(&alice_devices[2]));
        assert!(
            !targets.contains(&originating),
            "the originating device already has the message it just sent"
        );
    }

    #[test]
    fn spec_42_normal_context_never_reveals_the_device() {
        let sender = SenderIdentity {
            account_id: AccountId::new(),
            device_id: DeviceId::new(),
        };
        let (_, device) = account_level_display(sender, PresentationContext::Normal);
        assert_eq!(device, None);
    }

    #[test]
    fn spec_42_security_details_reveals_the_device() {
        let sender = SenderIdentity {
            account_id: AccountId::new(),
            device_id: DeviceId::new(),
        };
        let (_, device) = account_level_display(sender, PresentationContext::SecurityDetails);
        assert_eq!(device, Some(sender.device_id));
    }

    #[test]
    fn spec_43_delivered_to_account_if_any_device_delivered() {
        let phone = DeviceId::new();
        let laptop = DeviceId::new();
        let receipts = vec![
            DeviceReceipt {
                device_id: phone,
                status: DeviceReceiptStatus::Delivered,
            },
            DeviceReceipt {
                device_id: laptop,
                status: DeviceReceiptStatus::Offline,
            },
        ];
        assert!(aggregate_delivered_to_account(&receipts));
    }

    #[test]
    fn spec_43_not_delivered_if_no_device_has_it() {
        let receipts = vec![DeviceReceipt {
            device_id: DeviceId::new(),
            status: DeviceReceiptStatus::Offline,
        }];
        assert!(!aggregate_delivered_to_account(&receipts));
    }

    #[test]
    fn spec_44_unconfigured_data_class_is_not_synced_by_default() {
        let policy = OwnDeviceSyncPolicy::default();
        assert!(!policy.is_allowed(SyncDataClass::Contacts));
    }

    #[test]
    fn spec_44_policy_is_genuinely_per_data_class() {
        let mut policy = OwnDeviceSyncPolicy::default();
        policy.set(SyncDataClass::SentMessages, true);
        assert!(policy.is_allowed(SyncDataClass::SentMessages));
        assert!(
            !policy.is_allowed(SyncDataClass::Contacts),
            "must not leak to unrelated classes"
        );
    }
}
