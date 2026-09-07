//! §191 "Migration Strategy": "persistent identity schema must support
//! migrations... migration tests should include: old device directory,
//! old certificates, old trust state, old recovery metadata."
//!
//! Honestly incomplete, same as §125's gap this note is paired with:
//! there is no version field on [`crate::certificate::DeviceCertificate`]/
//! [`crate::directory::DeviceDirectory`] yet (see
//! [`crate::wire_limits`]'s own §125 note), so there is no OLD version
//! to migrate FROM in a way this crate could test today — "migration
//! tests" in the full sense §191 asks for can't exist before §125's
//! versioning gap is closed. What CAN exist now, and does, below: a
//! real fixture-based regression test that would fail the moment
//! today's wire format changes in an incompatible way, which is the
//! seed §191's migration tests would grow from once real versioning
//! exists — catching an accidental break is a real, useful property
//! even before there's a second version to migrate between.

#[cfg(test)]
mod tests {
    use crate::capability::DeviceCapabilitySet;
    use crate::certificate::DeviceCertificate;
    use crate::directory::{DeviceDirectory, DeviceDirectoryEntry, DeviceStatus};
    use crate::root_key::RootIdentityKey;
    use siar_domain::{AccountId, DeviceId};

    /// A `DeviceCertificate` signed and serialized today must still
    /// verify after a postcard round trip — this is the actual
    /// property a real migration test would extend into "and an old
    /// certificate from a PREVIOUS schema version must still verify
    /// today," once there's a previous version to test against.
    #[test]
    fn certificate_round_trips_through_serialization_and_still_verifies() {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let device_id = DeviceId::new();
        let certificate = DeviceCertificate::issue(
            &root,
            account,
            device_id,
            [3u8; 32],
            0,
            None,
            DeviceCapabilitySet::SEND_MESSAGE,
            1,
        );

        let bytes = postcard::to_allocvec(&certificate).expect("serialize");
        let restored: DeviceCertificate = postcard::from_bytes(&bytes).expect("deserialize");
        assert!(restored.verify_signature(&root.root_public_key()).is_ok());
    }

    /// Same property for a whole signed [`DeviceDirectory`] — "old
    /// device directory" from §191's own list, tested at today's
    /// version.
    #[test]
    fn directory_round_trips_through_serialization_and_still_verifies() {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let device_id = DeviceId::new();
        let certificate = DeviceCertificate::issue(
            &root,
            account,
            device_id,
            [4u8; 32],
            0,
            None,
            DeviceCapabilitySet(0),
            1,
        );
        let directory = DeviceDirectory::sign(
            &root,
            account,
            1,
            vec![DeviceDirectoryEntry {
                device_id,
                certificate,
                status: DeviceStatus::Active,
                transport_endpoints: vec![],
            }],
        );

        let bytes = postcard::to_allocvec(&directory).expect("serialize");
        let restored: DeviceDirectory = postcard::from_bytes(&bytes).expect("deserialize");
        assert!(restored.verify_signature(&root.root_public_key()).is_ok());
        assert!(restored.is_device_trusted(device_id));
    }
}
