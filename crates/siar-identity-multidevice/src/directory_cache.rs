//! §167 "Performance Goals", §168 "Cache Strategy", §169 "Device
//! Directory Size".
//!
//! §167 gets no new type: "avoid repeatedly verifying full account
//! history for every message... use validated cached state" is
//! exactly what [`DirectoryCache`] below and
//! [`crate::session_cache::RevocationCache`] (an earlier round) both
//! already are — a validated snapshot checked once, then consulted
//! cheaply, rather than re-walking a whole directory or event chain
//! per operation.

use std::collections::{HashMap, HashSet};

use siar_domain::DeviceId;

use crate::certificate::DeviceCertificate;
use crate::directory::{DeviceDirectory, DeviceEndpoint};

/// §168's own five named cache categories, built together from ONE
/// [`DeviceDirectory`] in [`DirectoryCache::from_directory`] — the
/// only constructor, matching
/// [`crate::session_cache::RevocationCache::from_directory`]'s exact
/// precedent for the identical reason: "invalidate on signed state
/// update" is enforced by there being no incremental mutator at all —
/// a new directory means building a whole new `DirectoryCache`, never
/// patching the old one in place.
#[derive(Debug, Clone, Default)]
pub struct DirectoryCache {
    verified_certificates: HashMap<DeviceId, DeviceCertificate>,
    active_devices: HashSet<DeviceId>,
    revoked_devices: HashSet<DeviceId>,
    highest_generation: u64,
    transport_endpoints: HashMap<DeviceId, Vec<DeviceEndpoint>>,
}

impl DirectoryCache {
    pub fn from_directory(directory: &DeviceDirectory) -> Self {
        let mut cache = DirectoryCache {
            highest_generation: directory.generation,
            ..Default::default()
        };
        for entry in &directory.devices {
            cache
                .verified_certificates
                .insert(entry.device_id, entry.certificate.clone());
            if entry.status == crate::directory::DeviceStatus::Active {
                cache.active_devices.insert(entry.device_id);
            }
            if entry.status == crate::directory::DeviceStatus::Revoked {
                cache.revoked_devices.insert(entry.device_id);
            }
            cache
                .transport_endpoints
                .insert(entry.device_id, entry.transport_endpoints.clone());
        }
        cache
    }

    pub fn verified_certificate(&self, device_id: DeviceId) -> Option<&DeviceCertificate> {
        self.verified_certificates.get(&device_id)
    }

    pub fn is_active(&self, device_id: DeviceId) -> bool {
        self.active_devices.contains(&device_id)
    }

    pub fn is_revoked(&self, device_id: DeviceId) -> bool {
        self.revoked_devices.contains(&device_id)
    }

    pub fn highest_generation(&self) -> u64 {
        self.highest_generation
    }

    pub fn transport_endpoints(&self, device_id: DeviceId) -> Option<&[DeviceEndpoint]> {
        self.transport_endpoints.get(&device_id).map(Vec::as_slice)
    }
}

/// §169: "normal handshake can send: account id, device certificate,
/// generation, state hash. Then request: missing state, only if
/// needed." The four fields, verbatim — deliberately NOT the whole
/// [`DeviceDirectory`]; a caller builds one of these per handshake
/// instead of sending `directory.devices` in full, and hands the
/// result to [`crate::reconciliation::ConvergenceStatus::compare`] to
/// decide whether a follow-up request is needed at all — the "only if
/// needed" half is that existing function, not new code here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandshakeSummary {
    pub account_id: siar_domain::AccountId,
    pub device_certificate_fingerprint: [u8; 32],
    pub generation: u64,
    pub state_hash: crate::state_chain::StateHash,
}

impl HandshakeSummary {
    pub fn from_directory(
        directory: &DeviceDirectory,
        presenting_device: DeviceId,
    ) -> Option<Self> {
        let certificate = directory
            .devices
            .iter()
            .find(|entry| entry.device_id == presenting_device)?
            .certificate
            .clone();
        Some(Self {
            account_id: directory.account_id,
            device_certificate_fingerprint: crate::local_records::certificate_fingerprint(
                &certificate,
            ),
            generation: directory.generation,
            state_hash: crate::reconciliation::state_hash_of(directory),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::DeviceCapabilitySet;
    use crate::certificate::DeviceCertificate;
    use crate::directory::{DeviceDirectory, DeviceDirectoryEntry, DeviceStatus};
    use crate::root_key::RootIdentityKey;
    use siar_domain::AccountId;

    fn entry(
        root: &RootIdentityKey,
        account: AccountId,
        device_id: DeviceId,
        status: DeviceStatus,
    ) -> DeviceDirectoryEntry {
        let certificate = DeviceCertificate::issue(
            root,
            account,
            device_id,
            [0u8; 32],
            0,
            None,
            DeviceCapabilitySet(0),
            1,
        );
        DeviceDirectoryEntry {
            device_id,
            certificate,
            status,
            transport_endpoints: vec![DeviceEndpoint(vec![1, 2, 3])],
        }
    }

    #[test]
    fn directory_cache_reflects_all_five_categories_from_one_directory() {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let active = DeviceId::new();
        let revoked = DeviceId::new();
        let directory = DeviceDirectory::sign(
            &root,
            account,
            7,
            vec![
                entry(&root, account, active, DeviceStatus::Active),
                entry(&root, account, revoked, DeviceStatus::Revoked),
            ],
        );

        let cache = DirectoryCache::from_directory(&directory);
        assert!(cache.verified_certificate(active).is_some());
        assert!(cache.is_active(active));
        assert!(!cache.is_active(revoked));
        assert!(cache.is_revoked(revoked));
        assert_eq!(cache.highest_generation(), 7);
        assert_eq!(cache.transport_endpoints(active).map(<[_]>::len), Some(1));
    }

    #[test]
    fn a_new_directory_requires_building_a_whole_new_cache() {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let device = DeviceId::new();
        let gen_1 = DeviceDirectory::sign(
            &root,
            account,
            1,
            vec![entry(&root, account, device, DeviceStatus::Active)],
        );
        let gen_2 = DeviceDirectory::sign(
            &root,
            account,
            2,
            vec![entry(&root, account, device, DeviceStatus::Revoked)],
        );

        let cache_1 = DirectoryCache::from_directory(&gen_1);
        let cache_2 = DirectoryCache::from_directory(&gen_2);
        assert!(cache_1.is_active(device));
        assert!(cache_2.is_revoked(device));
        // No mutator exists that could have turned `cache_1` into
        // `cache_2` in place — proving this by what the type doesn't
        // offer, not just by not calling one.
    }

    #[test]
    fn handshake_summary_carries_exactly_the_four_named_fields() {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let device = DeviceId::new();
        let directory = DeviceDirectory::sign(
            &root,
            account,
            3,
            vec![entry(&root, account, device, DeviceStatus::Active)],
        );

        let summary = HandshakeSummary::from_directory(&directory, device).unwrap();
        assert_eq!(summary.account_id, account);
        assert_eq!(summary.generation, 3);
        assert_eq!(
            summary.state_hash,
            crate::reconciliation::state_hash_of(&directory)
        );
    }

    #[test]
    fn handshake_summary_is_none_for_a_device_not_in_the_directory() {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let directory = DeviceDirectory::sign(&root, account, 1, vec![]);
        assert!(HandshakeSummary::from_directory(&directory, DeviceId::new()).is_none());
    }
}
