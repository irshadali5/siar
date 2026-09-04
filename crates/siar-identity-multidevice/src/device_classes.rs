//! §45 "Device Trust Classes", §46 "Headless Devices", §47 "Service
//! Identities", §48 "Organization Identity".

/// §45, verbatim enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DeviceTrustClass {
    Full,
    Limited,
    Temporary,
    Headless,
}

/// §45's own four worked examples, verbatim mapping.
pub fn spec_45_example_classification(device_description: &str) -> Option<DeviceTrustClass> {
    match device_description {
        "primary phone" | "laptop" => Some(DeviceTrustClass::Full),
        "public kiosk" => Some(DeviceTrustClass::Limited),
        "emergency relay" => Some(DeviceTrustClass::Headless),
        _ => None,
    }
}

/// §46's own three possible owner kinds for a headless device
/// ("NAS/Raspberry Pi/home relay/enterprise gateway/emergency node...
/// may belong to: user account, organization account, service
/// identity") — "do not assume every device has a screen" doesn't
/// need its own function; it's enforced by this crate never having a
/// UI-presence field on [`crate::directory::DeviceDirectoryEntry`] or
/// [`crate::certificate::DeviceCertificate`] in the first place.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum HeadlessDeviceOwner {
    UserAccount,
    OrganizationAccount,
    ServiceIdentity,
}

/// A headless device is always [`DeviceTrustClass::Headless`],
/// regardless of who owns it — the owner answers a different question
/// (whose principal is this) than the trust class does (how much can
/// this device's own compromise cost).
pub fn headless_device_trust_class(_owner: HeadlessDeviceOwner) -> DeviceTrustClass {
    DeviceTrustClass::Headless
}

/// §47's own four worked examples of a non-human principal — kept as a
/// label only, never a distinct authentication model: §47's actual
/// rule is "use the same identity primitives... do not create a
/// completely separate authentication model unless necessary," which
/// is why there is no `ServiceRootIdentityKey`/`ServiceDeviceDirectory`
/// anywhere in this crate — a service identity is
/// [`crate::root_key::RootIdentityKey`] and
/// [`crate::directory::DeviceDirectory`] like any other account, this
/// enum exists purely so an application can label which kind of
/// principal an ordinary [`siar_domain::AccountId`] represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ServiceIdentityKind {
    SchoolServer,
    EmergencyAuthority,
    AutomationService,
    DocumentRelay,
}

/// §48's own four worked device roles within an organization account —
/// again a label, not a new authorization mechanism: §48's actual
/// point is "identity proves this device belongs to this
/// organization principal; authorization decides what it may do," and
/// this crate already keeps those separate as a matter of workspace
/// architecture — identity (this crate) has no dependency on and no
/// knowledge of `siar-protocol-ext::ExtensionAuthorization` (the real
/// authorization-decision trait, spec 01 §33), which is what actually
/// decides "what it may do."
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum OrganizationDeviceRole {
    ServerDevice,
    AdminDevice,
    Relay,
    AutomationNode,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_45_four_worked_examples_classify_as_given() {
        assert_eq!(
            spec_45_example_classification("primary phone"),
            Some(DeviceTrustClass::Full)
        );
        assert_eq!(
            spec_45_example_classification("laptop"),
            Some(DeviceTrustClass::Full)
        );
        assert_eq!(
            spec_45_example_classification("public kiosk"),
            Some(DeviceTrustClass::Limited)
        );
        assert_eq!(
            spec_45_example_classification("emergency relay"),
            Some(DeviceTrustClass::Headless)
        );
    }

    #[test]
    fn spec_46_a_headless_device_is_always_headless_trust_class_regardless_of_owner() {
        assert_eq!(
            headless_device_trust_class(HeadlessDeviceOwner::UserAccount),
            DeviceTrustClass::Headless
        );
        assert_eq!(
            headless_device_trust_class(HeadlessDeviceOwner::OrganizationAccount),
            DeviceTrustClass::Headless
        );
        assert_eq!(
            headless_device_trust_class(HeadlessDeviceOwner::ServiceIdentity),
            DeviceTrustClass::Headless
        );
    }

    #[test]
    fn spec_47_a_service_identity_uses_the_real_root_key_and_directory_types() {
        // The actual claim §47 makes, proven structurally: this
        // compiles and runs using exactly the same RootIdentityKey and
        // DeviceDirectory a normal user account uses — no
        // service-specific identity type exists to reach for instead.
        use crate::capability::DeviceCapabilitySet;
        use crate::certificate::DeviceCertificate;
        use crate::directory::{DeviceDirectory, DeviceDirectoryEntry, DeviceStatus};
        use crate::root_key::RootIdentityKey;
        use siar_domain::{AccountId, DeviceId};

        let root = RootIdentityKey::generate(); // the "school server"'s own root key
        let account = AccountId::new();
        let device = DeviceId::new();
        let cert = DeviceCertificate::issue(
            &root,
            account,
            device,
            [1u8; 32],
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
                device_id: device,
                certificate: cert,
                status: DeviceStatus::Active,
                transport_endpoints: vec![],
            }],
        );

        assert!(directory.verify_signature(&root.root_public_key()).is_ok());
        let _kind = ServiceIdentityKind::SchoolServer; // purely a label alongside the above
    }
}
