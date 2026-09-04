//! §56 "Rollback Protection", §55 "Stale Device Directory", §29
//! "Revocation Conflict Rules": the receiving side of a
//! [`DeviceDirectory`] — tracks the highest trusted generation seen per
//! account and refuses to move backward.

use std::collections::HashMap;

use crate::directory::DeviceDirectory;
use crate::error::IdentityError;
use crate::root_key::RootPublicKey;
use siar_domain::AccountId;

/// One peer's local, durable view of every account's device directory
/// it has ever trusted — §56's "highest trusted generation per
/// account" plus the actual accepted snapshot, since a caller needs the
/// directory itself (to know who's currently active), not just its
/// generation number. `HashMap` in memory here; a real deployment would
/// back this with durable storage (§56: "or stronger state continuity")
/// — not attempted in this crate, see its own top-level doc comment.
#[derive(Debug, Default)]
pub struct TrustedAccountStore {
    trusted: HashMap<AccountId, DeviceDirectory>,
}

impl TrustedAccountStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// §55/§56: accepts `directory` only if its signature verifies
    /// against `root_public_key` AND its generation is strictly greater
    /// than the highest generation already trusted for this account —
    /// equal or lower is rejected, matching §56's own explicit rule
    /// ("never accept generation 17 after already trusting generation
    /// 22"). §57 "Fork Detection" governs the equal-generation case
    /// specifically: a resend of the EXACT SAME signed directory
    /// (identical signature bytes — Ed25519 signing is deterministic,
    /// so identical content always produces identical signature bytes)
    /// is a harmless no-op, but two DIFFERENT signed directories
    /// claiming the same generation is exactly §57's "two conflicting
    /// signed account states at the same generation" — this used to be
    /// silently treated as a no-op regardless of content (a real bug,
    /// found while implementing §57 properly, not merely a docs gap);
    /// now it returns [`IdentityError::IdentityForkDetected`], per
    /// §57's own explicit instruction not to silently choose one.
    pub fn accept(
        &mut self,
        directory: DeviceDirectory,
        root_public_key: &RootPublicKey,
    ) -> Result<(), IdentityError> {
        directory
            .verify_signature(root_public_key)
            .map_err(|_| IdentityError::DirectorySignatureInvalid)?;

        if let Some(existing) = self.trusted.get(&directory.account_id) {
            if directory.generation < existing.generation {
                return Err(IdentityError::RollbackRejected {
                    given: directory.generation,
                    highest: existing.generation,
                });
            }
            if directory.generation == existing.generation {
                if directory.signature == existing.signature {
                    return Ok(()); // genuinely the same directory, resent
                }
                return Err(IdentityError::IdentityForkDetected {
                    generation: directory.generation,
                });
            }
        }

        self.trusted.insert(directory.account_id, directory);
        Ok(())
    }

    pub fn directory_for(&self, account_id: AccountId) -> Option<&DeviceDirectory> {
        self.trusted.get(&account_id)
    }

    pub fn highest_generation_for(&self, account_id: AccountId) -> Option<u64> {
        self.trusted.get(&account_id).map(|d| d.generation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::DeviceCapabilitySet;
    use crate::certificate::DeviceCertificate;
    use crate::directory::{DeviceDirectoryEntry, DeviceStatus};
    use crate::root_key::RootIdentityKey;
    use siar_domain::DeviceId;

    fn entry(
        root: &RootIdentityKey,
        account: AccountId,
        generation: u64,
        status: DeviceStatus,
    ) -> DeviceDirectoryEntry {
        let device = DeviceId::new();
        let cert = DeviceCertificate::issue(
            root,
            account,
            device,
            [3u8; 32],
            0,
            None,
            DeviceCapabilitySet::SEND_MESSAGE,
            generation,
        );
        DeviceDirectoryEntry {
            device_id: device,
            certificate: cert,
            status,
            transport_endpoints: vec![],
        }
    }

    #[test]
    fn a_newer_generation_is_accepted_and_replaces_the_trusted_directory() {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let mut store = TrustedAccountStore::new();

        let gen1 = DeviceDirectory::sign(
            &root,
            account,
            1,
            vec![entry(&root, account, 1, DeviceStatus::Active)],
        );
        store.accept(gen1, &root.root_public_key()).unwrap();
        assert_eq!(store.highest_generation_for(account), Some(1));

        let gen2 = DeviceDirectory::sign(
            &root,
            account,
            2,
            vec![entry(&root, account, 2, DeviceStatus::Active)],
        );
        store.accept(gen2, &root.root_public_key()).unwrap();
        assert_eq!(store.highest_generation_for(account), Some(2));
    }

    #[test]
    fn an_older_generation_is_rejected_after_a_newer_one_is_trusted() {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let mut store = TrustedAccountStore::new();

        let gen1 = DeviceDirectory::sign(
            &root,
            account,
            1,
            vec![entry(&root, account, 1, DeviceStatus::Active)],
        );
        let gen2 = DeviceDirectory::sign(
            &root,
            account,
            2,
            vec![entry(&root, account, 2, DeviceStatus::Active)],
        );
        store.accept(gen2, &root.root_public_key()).unwrap();

        let result = store.accept(gen1, &root.root_public_key());
        assert_eq!(
            result,
            Err(IdentityError::RollbackRejected {
                given: 1,
                highest: 2
            })
        );
        // still at generation 2 — the rejected rollback attempt had no effect
        assert_eq!(store.highest_generation_for(account), Some(2));
    }

    #[test]
    fn a_revoked_device_cannot_regain_authority_via_a_stale_directory() {
        // §29's own scenario: device A revokes device B, but an offline
        // peer still holds an older directory where B is Active. That
        // older directory must not be accepted once the newer
        // (revoking) one has been trusted — same mechanism as the
        // generation-rollback test above, exercised with the actual
        // revocation scenario the spec names.
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let device = DeviceId::new();
        let mut store = TrustedAccountStore::new();

        let cert1 = DeviceCertificate::issue(
            &root,
            account,
            device,
            [9u8; 32],
            0,
            None,
            DeviceCapabilitySet::SEND_MESSAGE,
            1,
        );
        let stale_active = DeviceDirectory::sign(
            &root,
            account,
            1,
            vec![DeviceDirectoryEntry {
                device_id: device,
                certificate: cert1.clone(),
                status: DeviceStatus::Active,
                transport_endpoints: vec![],
            }],
        );
        let revoking = DeviceDirectory::sign(
            &root,
            account,
            2,
            vec![DeviceDirectoryEntry {
                device_id: device,
                certificate: cert1,
                status: DeviceStatus::Revoked,
                transport_endpoints: vec![],
            }],
        );

        store.accept(revoking, &root.root_public_key()).unwrap();
        assert!(store.accept(stale_active, &root.root_public_key()).is_err());

        let trusted = store.directory_for(account).unwrap();
        assert!(trusted.active_devices().next().is_none());
    }

    #[test]
    fn resending_the_same_generation_is_a_harmless_no_op() {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let mut store = TrustedAccountStore::new();

        let gen1 = DeviceDirectory::sign(
            &root,
            account,
            1,
            vec![entry(&root, account, 1, DeviceStatus::Active)],
        );
        store.accept(gen1.clone(), &root.root_public_key()).unwrap();
        assert!(store.accept(gen1, &root.root_public_key()).is_ok());
        assert_eq!(store.highest_generation_for(account), Some(1));
    }

    #[test]
    fn spec_57_two_different_signed_directories_at_the_same_generation_is_a_fork_not_a_silent_pick()
    {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let mut store = TrustedAccountStore::new();

        let gen1_a = DeviceDirectory::sign(
            &root,
            account,
            1,
            vec![entry(&root, account, 1, DeviceStatus::Active)],
        );
        // A DIFFERENT device at the SAME generation 1 — same root key,
        // genuinely different content, e.g. two devices concurrently
        // believing they're issuing "the" generation-1 directory.
        let gen1_b = DeviceDirectory::sign(
            &root,
            account,
            1,
            vec![entry(&root, account, 2, DeviceStatus::Active)],
        );
        assert_ne!(gen1_a.signature, gen1_b.signature);

        store.accept(gen1_a, &root.root_public_key()).unwrap();
        let result = store.accept(gen1_b, &root.root_public_key());
        assert_eq!(
            result,
            Err(IdentityError::IdentityForkDetected { generation: 1 })
        );
        // The store must keep the FIRST directory it trusted — a fork
        // must not silently overwrite what's already trusted either.
        assert_eq!(store.highest_generation_for(account), Some(1));
    }

    #[test]
    fn a_directory_with_a_bad_signature_is_rejected_outright() {
        let root = RootIdentityKey::generate();
        let impostor = RootIdentityKey::generate();
        let account = AccountId::new();
        let mut store = TrustedAccountStore::new();

        let forged = DeviceDirectory::sign(
            &impostor,
            account,
            1,
            vec![entry(&impostor, account, 1, DeviceStatus::Active)],
        );
        assert_eq!(
            store.accept(forged, &root.root_public_key()),
            Err(IdentityError::DirectorySignatureInvalid)
        );
    }
}
