//! §129 "Session Cache", §130 "Revocation Cache".

use std::collections::HashSet;

use siar_domain::{AccountId, DeviceId};

use crate::directory::{DeviceDirectory, DeviceStatus};
use siar_event_log::ids::Timestamp;

/// An opaque session identifier — this crate doesn't define how a
/// session is actually established (that's `siar-messaging`'s/a
/// transport's job), only the cache entry shape §129 names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SessionId(pub u64);

/// §129's own five named fields, verbatim.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionCacheEntry {
    pub account_id: AccountId,
    pub device_id: DeviceId,
    pub account_generation: u64,
    pub session_id: SessionId,
    pub last_authenticated: Timestamp,
}

/// §129's three named invalidation triggers — checked against the
/// account's CURRENT [`DeviceDirectory`], never against whatever
/// generation the session itself claims, since a session that's stale
/// precisely because of one of these three events is exactly the
/// thing this function has to catch even though the session entry's
/// own fields still look internally consistent.
impl SessionCacheEntry {
    pub fn is_invalidated_by(&self, current_directory: &DeviceDirectory) -> bool {
        // "root changed": current_directory not even being for a
        // higher-or-equal generation trust chain than this session
        // knew about isn't checkable from the directory alone (root
        // rotation is verified elsewhere — see
        // `crate::root_rotation::verify_root_rotation`); what IS
        // checkable here is the generation moving at all versus what
        // this session was authenticated under.
        if current_directory.generation != self.account_generation {
            return true;
        }

        // "device revoked" / "certificate rotated": look up this
        // session's device in the current directory. Not found, or
        // found but not `Active`, both invalidate — a session for a
        // device that no longer appears at all is at least as stale as
        // one for a device now marked `Revoked`.
        match current_directory
            .devices
            .iter()
            .find(|entry| entry.device_id == self.device_id)
        {
            Some(entry) => entry.status != DeviceStatus::Active,
            None => true,
        }
    }
}

/// §130: "must be derived from durable authenticated state." The only
/// constructor is [`RevocationCache::from_directory`] — there is no
/// way to insert a [`DeviceId`] into a `RevocationCache` that didn't
/// come from an actual signed [`DeviceDirectory`], so this cache
/// cannot itself become a second, independent source of truth about
/// which devices are revoked; it can only ever restate what a real
/// directory already says, in a shape that's fast to check.
#[derive(Debug, Clone, Default)]
pub struct RevocationCache {
    revoked: HashSet<DeviceId>,
}

impl RevocationCache {
    pub fn from_directory(directory: &DeviceDirectory) -> Self {
        let revoked = directory
            .devices
            .iter()
            .filter(|entry| entry.status == DeviceStatus::Revoked)
            .map(|entry| entry.device_id)
            .collect();
        Self { revoked }
    }

    /// §130's actual payoff: an O(1) fast-path check for session
    /// validation, instead of scanning the whole directory on every
    /// authenticated request.
    pub fn is_revoked(&self, device_id: DeviceId) -> bool {
        self.revoked.contains(&device_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::DeviceCapabilitySet;
    use crate::certificate::DeviceCertificate;
    use crate::directory::{DeviceDirectoryEntry, DeviceEndpoint};
    use crate::root_key::RootIdentityKey;

    fn entry(
        root: &RootIdentityKey,
        account: AccountId,
        device_id: DeviceId,
        status: DeviceStatus,
        generation: u64,
    ) -> DeviceDirectoryEntry {
        let certificate = DeviceCertificate::issue(
            root,
            account,
            device_id,
            [0u8; 32],
            0,
            None,
            DeviceCapabilitySet(0),
            generation,
        );
        DeviceDirectoryEntry {
            device_id,
            certificate,
            status,
            transport_endpoints: vec![DeviceEndpoint(vec![1])],
        }
    }

    #[test]
    fn session_is_not_invalidated_while_device_stays_active_at_the_same_generation() {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let device_id = DeviceId::new();
        let directory = DeviceDirectory::sign(
            &root,
            account,
            1,
            vec![entry(&root, account, device_id, DeviceStatus::Active, 1)],
        );
        let session = SessionCacheEntry {
            account_id: account,
            device_id,
            account_generation: 1,
            session_id: SessionId(42),
            last_authenticated: Timestamp(0),
        };
        assert!(!session.is_invalidated_by(&directory));
    }

    #[test]
    fn session_is_invalidated_when_its_device_is_revoked() {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let device_id = DeviceId::new();
        let directory = DeviceDirectory::sign(
            &root,
            account,
            2,
            vec![entry(&root, account, device_id, DeviceStatus::Revoked, 1)],
        );
        let session = SessionCacheEntry {
            account_id: account,
            device_id,
            account_generation: 1,
            session_id: SessionId(1),
            last_authenticated: Timestamp(0),
        };
        assert!(session.is_invalidated_by(&directory));
    }

    #[test]
    fn session_is_invalidated_when_the_account_generation_has_moved() {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let device_id = DeviceId::new();
        let directory = DeviceDirectory::sign(
            &root,
            account,
            5,
            vec![entry(&root, account, device_id, DeviceStatus::Active, 1)],
        );
        let session = SessionCacheEntry {
            account_id: account,
            device_id,
            account_generation: 1,
            session_id: SessionId(1),
            last_authenticated: Timestamp(0),
        };
        assert!(session.is_invalidated_by(&directory));
    }

    #[test]
    fn session_is_invalidated_when_its_device_no_longer_appears_at_all() {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let device_id = DeviceId::new();
        let other_device = DeviceId::new();
        let directory = DeviceDirectory::sign(
            &root,
            account,
            1,
            vec![entry(&root, account, other_device, DeviceStatus::Active, 1)],
        );
        let session = SessionCacheEntry {
            account_id: account,
            device_id,
            account_generation: 1,
            session_id: SessionId(1),
            last_authenticated: Timestamp(0),
        };
        assert!(session.is_invalidated_by(&directory));
    }

    #[test]
    fn revocation_cache_only_ever_reflects_what_the_directory_actually_says() {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let active_device = DeviceId::new();
        let revoked_device = DeviceId::new();
        let directory = DeviceDirectory::sign(
            &root,
            account,
            1,
            vec![
                entry(&root, account, active_device, DeviceStatus::Active, 1),
                entry(&root, account, revoked_device, DeviceStatus::Revoked, 1),
            ],
        );

        let cache = RevocationCache::from_directory(&directory);
        assert!(!cache.is_revoked(active_device));
        assert!(cache.is_revoked(revoked_device));
        assert!(
            !cache.is_revoked(DeviceId::new()),
            "an unrelated device id must never read as revoked"
        );
    }
}
