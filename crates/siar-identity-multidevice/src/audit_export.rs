//! §186 "Audit Export": "support exporting: device list, security
//! fingerprints, revocation history, identity generation... without
//! exporting private keys."

use crate::directory::{DeviceDirectory, DeviceStatus};
use crate::local_records::DeviceHistoryRecord;
use siar_domain::DeviceId;

/// One row of the exported device list — deliberately a narrower view
/// than [`crate::platform_boundary::DeviceRowVm`] (no "role," no
/// "last active"): an audit export is for enterprise/security review
/// of WHAT devices exist and their trust status, not a UI convenience,
/// so it carries only what §186 actually asks an export to contain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditDeviceRow {
    pub device_id: DeviceId,
    pub status: DeviceStatus,
}

/// §186's own four named contents, verbatim, and structurally nothing
/// else: no [`crate::root_key::RootIdentityKey`],
/// [`crate::secure_storage::SecretBytes`], or any other secret-holding
/// type is imported into this file — the same "enforced by absence"
/// pattern [`crate::identity_backup::IdentityBackup`] and
/// [`crate::platform_boundary`]'s view models already use for the
/// identical concern. `security_fingerprints` holds already-rendered
/// display strings
/// ([`crate::safety_fingerprint::SafetyFingerprint::display_string`]'s
/// own output), never a fingerprint's underlying key material.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditExport {
    pub device_list: Vec<AuditDeviceRow>,
    pub security_fingerprints: Vec<String>,
    pub revocation_history: Vec<DeviceHistoryRecord>,
    pub identity_generation: u64,
}

impl AuditExport {
    /// Builds the device-list and generation fields from a real,
    /// currently-trusted directory; `security_fingerprints` and
    /// `revocation_history` are supplied separately since neither
    /// lives inside a [`DeviceDirectory`] itself (fingerprints are
    /// pairwise, computed per contact; history is
    /// [`crate::local_records::DeviceHistoryLog`]'s own accumulated
    /// record, not part of the current directory snapshot at all).
    pub fn from_directory(
        directory: &DeviceDirectory,
        security_fingerprints: Vec<String>,
        revocation_history: Vec<DeviceHistoryRecord>,
    ) -> Self {
        Self {
            device_list: directory
                .devices
                .iter()
                .map(|entry| AuditDeviceRow {
                    device_id: entry.device_id,
                    status: entry.status,
                })
                .collect(),
            security_fingerprints,
            revocation_history,
            identity_generation: directory.generation,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::DeviceCapabilitySet;
    use crate::certificate::DeviceCertificate;
    use crate::directory::DeviceDirectoryEntry;
    use crate::root_key::RootIdentityKey;
    use siar_domain::AccountId;

    #[test]
    fn export_reflects_the_real_directorys_devices_and_generation() {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let device = DeviceId::new();
        let certificate = DeviceCertificate::issue(
            &root,
            account,
            device,
            [0u8; 32],
            0,
            None,
            DeviceCapabilitySet(0),
            1,
        );
        let directory = DeviceDirectory::sign(
            &root,
            account,
            9,
            vec![DeviceDirectoryEntry {
                device_id: device,
                certificate,
                status: DeviceStatus::Active,
                transport_endpoints: vec![],
            }],
        );

        let export = AuditExport::from_directory(&directory, vec!["1234-5678".to_string()], vec![]);
        assert_eq!(export.device_list.len(), 1);
        assert_eq!(export.device_list[0].device_id, device);
        assert_eq!(export.identity_generation, 9);
        assert_eq!(export.security_fingerprints, vec!["1234-5678".to_string()]);
    }

    // There is deliberately no test attempting to add a private-key
    // field to `AuditExport` — that would be a compile error, since no
    // secret-holding type is even imported into this file, matching
    // `identity_backup.rs`'s own tests module reasoning for the
    // identical kind of structural guarantee.
}
