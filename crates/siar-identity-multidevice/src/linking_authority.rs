//! §60 "Device Linking Authority", §61 "Default Consumer Policy", §62
//! "Link Approval Certificate", §63 "Device Roles", §64 "Security
//! Capabilities", §65 "Principle of Least Authority".

use crate::capability::DeviceCapabilitySet;
use crate::directory::DeviceDirectoryEntry;

/// §60's own four policies, verbatim, plus §60's own instruction:
/// "make this configurable" — a closed enum a caller picks one of,
/// not a hardcoded rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LinkingAuthorityPolicy {
    RootAuthorityOnly,
    AnyFullTrustDevice,
    ThresholdOfTrustedDevices { threshold: u8 },
    OrganizationAdmin,
}

/// §61's own two worked defaults.
pub fn default_consumer_policy() -> LinkingAuthorityPolicy {
    LinkingAuthorityPolicy::AnyFullTrustDevice
}

pub fn default_enterprise_policy() -> LinkingAuthorityPolicy {
    LinkingAuthorityPolicy::OrganizationAdmin
}

/// §62: "do not let any device arbitrarily add peers" — the actual
/// gate, checked against §64's real `LINK_NEW_DEVICE` capability bit
/// rather than merely "this device is Active." An active device with
/// no delegated link authority still can't approve a new device;
/// that's precisely §62's "root certificate delegates device-link
/// authority" made checkable.
pub fn device_can_approve_links(entry: &DeviceDirectoryEntry) -> bool {
    entry.status == crate::directory::DeviceStatus::Active
        && entry
            .certificate
            .capabilities
            .contains(DeviceCapabilitySet::LINK_NEW_DEVICE)
}

/// §63's own six roles, verbatim. "These are account-level
/// capabilities, not UI labels only" — [`DeviceRole::default_capabilities`]
/// is that claim made real: each role maps to an actual
/// [`DeviceCapabilitySet`], not just a display string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DeviceRole {
    Primary,
    Standard,
    Limited,
    Relay,
    Recovery,
    Admin,
}

impl DeviceRole {
    /// A reasonable, real default capability set per role — not given
    /// verbatim by §63 itself (it names the roles, not their exact
    /// bits), so this mapping is this crate's own considered choice,
    /// built from §64's real capability bits and §65's least-authority
    /// principle: `Relay` gets exactly the minimum §65's own worked
    /// example describes, `Limited` gets messaging only, `Primary`/
    /// `Admin` get the most because those roles' whole point is
    /// broad authority.
    pub fn default_capabilities(self) -> DeviceCapabilitySet {
        use DeviceCapabilitySet as Caps;
        match self {
            DeviceRole::Primary => Caps::ALL,
            DeviceRole::Standard => Caps::SEND_MESSAGE
                .union(Caps::RECEIVE_MESSAGE)
                .union(Caps::SYNC_HISTORY),
            DeviceRole::Limited => Caps::SEND_MESSAGE.union(Caps::RECEIVE_MESSAGE),
            DeviceRole::Relay => Caps::RELAY,
            DeviceRole::Recovery => Caps::ROTATE_ACCOUNT_STATE,
            DeviceRole::Admin => Caps::ALL,
        }
    }
}

/// §65's own worked example, verbatim: exactly what a headless relay
/// should NOT need, made checkable by construction rather than only
/// documented. There is no `LINK_NEW_DEVICE`/`REVOKE_DEVICE`/
/// `ROTATE_ACCOUNT_STATE`/`SEND_MESSAGE` (message *decryption* implies
/// the ability to originate/read messages this role has no business
/// with either) anywhere in this return value.
pub fn headless_relay_minimum_capabilities() -> DeviceCapabilitySet {
    DeviceCapabilitySet::RELAY
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::certificate::DeviceCertificate;
    use crate::directory::DeviceStatus;
    use crate::root_key::RootIdentityKey;
    use siar_domain::{AccountId, DeviceId};

    fn entry_with_capabilities(
        caps: DeviceCapabilitySet,
        status: DeviceStatus,
    ) -> DeviceDirectoryEntry {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let device = DeviceId::new();
        let cert = DeviceCertificate::issue(&root, account, device, [1u8; 32], 0, None, caps, 0);
        DeviceDirectoryEntry {
            device_id: device,
            certificate: cert,
            status,
            transport_endpoints: vec![],
        }
    }

    #[test]
    fn spec_62_a_device_without_link_capability_cannot_approve_links() {
        let entry =
            entry_with_capabilities(DeviceCapabilitySet::SEND_MESSAGE, DeviceStatus::Active);
        assert!(!device_can_approve_links(&entry));
    }

    #[test]
    fn spec_62_an_active_device_with_link_capability_can_approve_links() {
        let entry =
            entry_with_capabilities(DeviceCapabilitySet::LINK_NEW_DEVICE, DeviceStatus::Active);
        assert!(device_can_approve_links(&entry));
    }

    #[test]
    fn spec_62_a_revoked_device_cannot_approve_links_even_with_the_capability_bit_set() {
        let entry =
            entry_with_capabilities(DeviceCapabilitySet::LINK_NEW_DEVICE, DeviceStatus::Revoked);
        assert!(!device_can_approve_links(&entry));
    }

    #[test]
    fn spec_65_a_headless_relay_has_no_root_authority_or_message_or_linking_capability() {
        let relay_caps = headless_relay_minimum_capabilities();
        assert!(!relay_caps.contains(DeviceCapabilitySet::LINK_NEW_DEVICE));
        assert!(!relay_caps.contains(DeviceCapabilitySet::REVOKE_DEVICE));
        assert!(!relay_caps.contains(DeviceCapabilitySet::ROTATE_ACCOUNT_STATE));
        assert!(!relay_caps.contains(DeviceCapabilitySet::SEND_MESSAGE));
        assert!(relay_caps.contains(DeviceCapabilitySet::RELAY));
    }

    #[test]
    fn spec_63_each_role_maps_to_a_real_capability_set_not_just_a_label() {
        // The actual claim: at least two roles must differ in their
        // real capabilities, proving this isn't a decorative label.
        assert_ne!(
            DeviceRole::Limited.default_capabilities(),
            DeviceRole::Admin.default_capabilities()
        );
    }
}
