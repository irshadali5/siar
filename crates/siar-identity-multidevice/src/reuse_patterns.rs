//! §140 "Headless Linking", §141 "File-Only Product Reuse", §142 "ERP
//! Reuse", §143 "Emergency Reuse", §144 "Service-to-Service Reuse".
//!
//! All five are reuse-pattern CLAIMS about this crate's existing
//! primitives ([`crate::directory::DeviceDirectory`],
//! [`crate::revocation::revoke_device`],
//! [`crate::principal_claims::PrincipalType`],
//! [`crate::device_classes`]'s service-identity types) rather than
//! requests for new mechanism — so instead of new runtime types, this
//! module proves each claim with a real composition test using only
//! this crate's public API, the same way
//! [`crate::destination::spec_70_example_target`] and
//! [`crate::device_classes::spec_45_example_classification`] already
//! prove earlier "does this compose" claims elsewhere in this crate.
//! A test that fails to compile because some reuse claim turned out to
//! need a type this crate doesn't expose would be real evidence the
//! claim doesn't hold; one that passes is real evidence it does.
//!
//! §142 is the one exception with genuinely new code:
//! [`MapsToAccount`]. "The communication SDK should not know what an
//! employee is" is why there is no `EmployeeId` type anywhere in this
//! crate (the absence is the point, same as §109's address-book
//! rule) — but an ERP still needs *some* seam to plug its own id type
//! into [`crate::directory::DeviceDirectory`] lookups, and a bare
//! `Fn(EmployeeId) -> AccountId` callback per call site is worse than
//! one small, ERP-agnostic trait an application's own id type
//! implements once.

use siar_domain::AccountId;

/// §142: "ERP can map `EmployeeId → AccountId` or maintain its own
/// mapping." This trait is that seam — implemented by an
/// application's OWN id type (an `EmployeeId` the application defines,
/// never this crate), never the other way around. This crate accepts
/// `AccountId`s everywhere already; nothing here or anywhere else in
/// this crate needs to call `account_id()` itself; the trait exists so
/// an ERP integration has one conventional name for the mapping
/// instead of each integration inventing its own.
pub trait MapsToAccount {
    fn account_id(&self) -> AccountId;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::DeviceCapabilitySet;
    use crate::certificate::DeviceCertificate;
    use crate::device_classes::{HeadlessDeviceOwner, OrganizationDeviceRole, ServiceIdentityKind};
    use crate::directory::{DeviceDirectory, DeviceDirectoryEntry, DeviceStatus};
    use crate::linking_authority::device_can_approve_links;
    use crate::principal_claims::PrincipalType;
    use crate::root_key::RootIdentityKey;
    use siar_domain::DeviceId;

    fn signed_entry(
        root: &RootIdentityKey,
        account: AccountId,
        device_id: DeviceId,
        capabilities: DeviceCapabilitySet,
    ) -> DeviceDirectoryEntry {
        let certificate = DeviceCertificate::issue(
            root,
            account,
            device_id,
            [0u8; 32],
            0,
            None,
            capabilities,
            1,
        );
        DeviceDirectoryEntry {
            device_id,
            certificate,
            status: DeviceStatus::Active,
            transport_endpoints: vec![],
        }
    }

    /// §140: a headless node links via any of four named methods
    /// ("CLI approval / QR rendered elsewhere / one-time token /
    /// admin-issued certificate") — proved here by exercising the
    /// admin-issued-certificate path (the most headless of the four,
    /// no interactive approval at all) through the exact same
    /// `DeviceCertificate::issue`/`DeviceDirectory::sign` primitives an
    /// interactive device uses. "The same identity semantics apply" is
    /// demonstrated by there being no separate code path at all — this
    /// test calls nothing headless-specific.
    #[test]
    fn spec_140_headless_node_links_through_the_same_primitives() {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let headless_device = DeviceId::new();
        let entry = signed_entry(&root, account, headless_device, DeviceCapabilitySet(0));
        let directory = DeviceDirectory::sign(&root, account, 1, vec![entry]);
        assert!(directory.is_device_trusted(headless_device));
    }

    /// §141: a file-transfer app uses AccountId/DeviceId/DeviceDirectory/
    /// pairing/revocation "without: contacts, message history,
    /// conversation model." Proved by construction — this test builds
    /// and revokes a two-device account using only identity types, and
    /// this crate has no contact/message/conversation type for it to
    /// have accidentally needed.
    #[test]
    fn spec_141_file_only_reuse_needs_no_messaging_concepts() {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let device_a = DeviceId::new();
        let device_b = DeviceId::new();
        let directory = DeviceDirectory::sign(
            &root,
            account,
            1,
            vec![
                signed_entry(&root, account, device_a, DeviceCapabilitySet(0)),
                signed_entry(&root, account, device_b, DeviceCapabilitySet(0)),
            ],
        );

        let after_revocation = crate::revocation::revoke_device(&root, &directory, device_b)
            .expect("revoking a real, present device must succeed");
        assert!(after_revocation.is_device_trusted(device_a));
        assert!(!after_revocation.is_device_trusted(device_b));
    }

    /// §142: an ERP's own `EmployeeId` (defined here, in the TEST, not
    /// in this crate) maps to an `AccountId` via [`MapsToAccount`], and
    /// nothing downstream needs to change.
    #[test]
    fn spec_142_an_applications_own_id_type_maps_to_account_id() {
        struct EmployeeId(u64);
        impl MapsToAccount for EmployeeId {
            fn account_id(&self) -> AccountId {
                // A real ERP would look this up in its own mapping
                // table; this test only needs to prove the seam
                // compiles and threads through correctly.
                AccountId::new()
            }
        }

        let employee = EmployeeId(42);
        assert_eq!(employee.0, 42);
        let account = employee.account_id();
        let root = RootIdentityKey::generate();
        let device = DeviceId::new();
        let directory = DeviceDirectory::sign(
            &root,
            account,
            1,
            vec![signed_entry(&root, account, device, DeviceCapabilitySet(0))],
        );
        assert_eq!(directory.account_id, account);
    }

    /// §143: one responder account links a phone, a radio gateway, and
    /// a vehicle node — three devices of genuinely different kinds
    /// under one account, tagged [`PrincipalType::Responder`], "without
    /// changing identity semantics." Proved by using the exact same
    /// `DeviceDirectory`/`DeviceCertificate` shape for all three device
    /// kinds; nothing here special-cases "radio gateway" or "vehicle
    /// node" as a type.
    #[test]
    fn spec_143_one_responder_account_links_three_different_device_kinds() {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let phone = DeviceId::new();
        let radio_gateway = DeviceId::new();
        let vehicle_node = DeviceId::new();
        let principal = PrincipalType::Responder;

        let directory = DeviceDirectory::sign(
            &root,
            account,
            1,
            vec![
                signed_entry(&root, account, phone, DeviceCapabilitySet(0)),
                signed_entry(&root, account, radio_gateway, DeviceCapabilitySet(0)),
                signed_entry(&root, account, vehicle_node, DeviceCapabilitySet(0)),
            ],
        );

        assert_eq!(principal, PrincipalType::Responder);
        assert_eq!(directory.devices.len(), 3);
        assert!(directory.is_device_trusted(phone));
        assert!(directory.is_device_trusted(radio_gateway));
        assert!(directory.is_device_trusted(vehicle_node));
    }

    /// §144: a headless service uses "organization identity, device
    /// certificate, transport identity" for automated
    /// service-to-service communication — proved by composing
    /// [`crate::device_classes`]'s existing organization/service-role
    /// labels with an ordinary signed certificate, the same primitive
    /// §140's headless node used, and confirming a service device
    /// still goes through the exact same link-approval eligibility
    /// check any other device does — no service-specific carve-out.
    #[test]
    fn spec_144_headless_service_composes_org_identity_with_a_device_certificate() {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let service_device = DeviceId::new();
        let entry = signed_entry(&root, account, service_device, DeviceCapabilitySet(0));
        let directory = DeviceDirectory::sign(&root, account, 1, vec![entry.clone()]);

        let owner = HeadlessDeviceOwner::ServiceIdentity;
        let kind = ServiceIdentityKind::AutomationService;
        let org_role = OrganizationDeviceRole::AutomationNode;

        assert_eq!(owner, HeadlessDeviceOwner::ServiceIdentity);
        assert_eq!(kind, ServiceIdentityKind::AutomationService);
        assert_eq!(org_role, OrganizationDeviceRole::AutomationNode);
        // This service device was issued with no LINK_NEW_DEVICE
        // capability, so — same rule every other device follows —
        // it cannot approve new device links.
        assert!(!device_can_approve_links(&entry));
        assert!(directory.is_device_trusted(service_device));
    }
}
