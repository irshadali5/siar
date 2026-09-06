//! §147 "Device History Retention", §149 "Root Trust Cache".

use siar_domain::DeviceId;
use siar_event_log::ids::Timestamp;

use crate::certificate::DeviceCertificate;
use crate::client_api::RevocationReason;
use crate::root_key::RootPublicKey;

/// §147's own four named fields, verbatim — no more. "Without
/// retaining unnecessary sensitive metadata forever" is why this type
/// does NOT hold a friendly name, a transport endpoint, or any of
/// [`crate::discovery_privacy::PrivateDeviceMetadata`]'s other
/// fields — a history record answers "which device, when, what
/// fingerprint, why," nothing about who it belonged to or how to
/// reach it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceHistoryRecord {
    pub device_id: DeviceId,
    pub revoked_at: Timestamp,
    pub certificate_fingerprint: [u8; 32],
    pub reason: Option<RevocationReason>,
}

/// A stable fingerprint for one certificate — BLAKE3 over the two
/// fields that actually change between certificates for the same
/// device (`device_public_key`, `signature`), so a rotated
/// certificate for the same [`DeviceId`] produces a different
/// fingerprint, matching §147's own implicit need to tell "this
/// specific certificate was compromised" apart from "this device was
/// compromised at some point."
pub fn certificate_fingerprint(certificate: &DeviceCertificate) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(&certificate.device_public_key);
    hasher.update(&certificate.signature);
    *hasher.finalize().as_bytes()
}

/// §147: "keep historical records... without retaining unnecessary
/// sensitive metadata forever." [`DeviceHistoryLog::retain_since`] is
/// the "not forever" half made real — a caller decides the retention
/// window (spec names none), but the mechanism to actually enforce
/// one exists and is tested, rather than left as a comment for
/// whoever persists this type to remember on their own.
#[derive(Debug, Clone, Default)]
pub struct DeviceHistoryLog {
    records: Vec<DeviceHistoryRecord>,
}

impl DeviceHistoryLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record(&mut self, entry: DeviceHistoryRecord) {
        self.records.push(entry);
    }

    pub fn records(&self) -> &[DeviceHistoryRecord] {
        &self.records
    }

    /// Drops every record older than `cutoff` — "without retaining
    /// unnecessary sensitive metadata forever" as an actual operation,
    /// not a policy statement a caller has to remember to implement
    /// separately.
    pub fn retain_since(&mut self, cutoff: Timestamp) {
        self.records.retain(|r| r.revoked_at.0 >= cutoff.0);
    }
}

/// §149's own three named fields, verbatim. Deliberately a SEPARATE
/// type from [`crate::contact_verification::VerifiedContact`], not a
/// new field bolted onto it — `VerifiedContact` is an already-shipped,
/// already-tested public type from an earlier round, and adding a
/// `verified_at: Timestamp` field to it now would be exactly the kind
/// of breaking change to a shipped type this crate names rather than
/// makes silently elsewhere (see §125's schema-versioning gap note in
/// [`crate::wire_limits`]). The overlap between the two is real and
/// named here, not hidden: a future consolidation pass could fold
/// `RootTrustCacheEntry`'s `verified_at` into `VerifiedContact` once
/// that's an intentional, reviewed migration — not an incidental
/// side effect of this round.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootTrustCacheEntry {
    trusted_root: RootPublicKey,
    pub verified_at: Timestamp,
    pub verification_method: crate::contact_verification::OfflineVerificationMethod,
}

impl RootTrustCacheEntry {
    pub fn new(
        trusted_root: RootPublicKey,
        verified_at: Timestamp,
        verification_method: crate::contact_verification::OfflineVerificationMethod,
    ) -> Self {
        Self {
            trusted_root,
            verified_at,
            verification_method,
        }
    }

    pub fn trusted_root(&self) -> RootPublicKey {
        self.trusted_root
    }

    /// §149: "root changes require explicit policy." There is
    /// deliberately no `set_trusted_root`/`&mut trusted_root` — the
    /// only way to change which root this entry trusts is to build a
    /// whole new entry via [`RootTrustCacheEntry::new`], the same
    /// explicit-reconstruction pattern
    /// [`crate::contact_verification::VerifiedContact::re_anchor`]
    /// already uses for the identical concern.
    pub fn re_anchor(
        self,
        new_trusted_root: RootPublicKey,
        verified_at: Timestamp,
        verification_method: crate::contact_verification::OfflineVerificationMethod,
    ) -> Self {
        Self::new(new_trusted_root, verified_at, verification_method)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::DeviceCapabilitySet;
    use crate::contact_verification::OfflineVerificationMethod;
    use crate::root_key::RootIdentityKey;
    use siar_domain::AccountId;

    #[test]
    fn certificate_fingerprint_changes_when_the_certificate_is_rotated() {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let device_id = DeviceId::new();
        let original = DeviceCertificate::issue(
            &root,
            account,
            device_id,
            [1u8; 32],
            0,
            None,
            DeviceCapabilitySet(0),
            1,
        );
        let rotated = DeviceCertificate::issue(
            &root,
            account,
            device_id,
            [2u8; 32],
            0,
            None,
            DeviceCapabilitySet(0),
            2,
        );

        assert_ne!(
            certificate_fingerprint(&original),
            certificate_fingerprint(&rotated)
        );
    }

    #[test]
    fn history_log_retains_only_records_at_or_after_the_cutoff() {
        let mut log = DeviceHistoryLog::new();
        log.record(DeviceHistoryRecord {
            device_id: DeviceId::new(),
            revoked_at: Timestamp(10),
            certificate_fingerprint: [0u8; 32],
            reason: Some(RevocationReason::Lost),
        });
        log.record(DeviceHistoryRecord {
            device_id: DeviceId::new(),
            revoked_at: Timestamp(100),
            certificate_fingerprint: [1u8; 32],
            reason: Some(RevocationReason::Compromised),
        });

        log.retain_since(Timestamp(50));
        assert_eq!(log.records().len(), 1);
        assert_eq!(log.records()[0].revoked_at, Timestamp(100));
    }

    #[test]
    fn root_trust_cache_entry_only_changes_root_via_explicit_re_anchor() {
        let root = RootIdentityKey::generate();
        let new_root = RootIdentityKey::generate();
        let entry = RootTrustCacheEntry::new(
            root.root_public_key(),
            Timestamp(1),
            OfflineVerificationMethod::Qr,
        );
        assert_eq!(entry.trusted_root(), root.root_public_key());

        let re_anchored = entry.re_anchor(
            new_root.root_public_key(),
            Timestamp(2),
            OfflineVerificationMethod::ManualFingerprint,
        );
        assert_eq!(re_anchored.trusted_root(), new_root.root_public_key());
        assert_eq!(re_anchored.verified_at, Timestamp(2));
    }
}
