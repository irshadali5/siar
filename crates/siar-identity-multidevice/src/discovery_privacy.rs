//! §105 "Privacy", §106 "Public vs Private Device Metadata", §107
//! "Rotating Discovery Tokens", §108 "Device Tracking Resistance",
//! §109 "Address Book Mapping".
//!
//! §109 gets no new type here, by design: it says the identity SDK
//! should "map AccountId → Contact" nowhere in itself, and should NOT
//! own a phone-number address book, social graph, or contact labels —
//! those are application concerns. There is no `Contact` type, no
//! address-book trait, and no phone-number field anywhere in this
//! crate; §109 is satisfied by the absence, not by a boundary type
//! that would itself be a small step toward owning what it's supposed
//! to stay out of.

use serde::{Deserialize, Serialize};
use std::time::Duration;

use siar_domain::{AccountId, DeviceId};

/// §105/§106's split, made structural rather than a convention callers
/// have to remember: [`DeviceMetadata`] only ever holds the three
/// public fields §106 names, and its private counterpart is a
/// genuinely different type ([`PrivateDeviceMetadata`]) rather than
/// the same struct with fields a caller is trusted not to read before
/// authentication. There is no single struct here with "sensitive"
/// fields left un-enforced by type — a caller who only has a
/// `DeviceMetadata` cannot accidentally reach `friendly_name` at all,
/// because it doesn't exist on that type.
///
/// §106's public tier: "device id / device key / required certificate
/// fields." The certificate itself
/// ([`crate::certificate::DeviceCertificate`]) already carries the key
/// and required fields, so this only adds the `device_id` label on
/// top of a certificate reference — it does not re-list or duplicate
/// the certificate's own fields.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceMetadata {
    pub device_id: DeviceId,
}

/// §106's private/authenticated tier, verbatim four fields. Only ever
/// constructed and handed out after the authenticated
/// contact/session-establishment §105 requires — nothing in this
/// crate exposes a `PrivateDeviceMetadata` value from an unauthenticated
/// code path, since it doesn't exist until a caller who has actually
/// authenticated builds one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivateDeviceMetadata {
    pub friendly_name: String,
    pub platform_detail: String,
    pub last_seen: Option<siar_event_log::ids::Timestamp>,
    pub capabilities: crate::capability::DeviceCapabilitySet,
}

/// §107: "nearby discovery must not advertise permanent AccountId /
/// DeviceId in plaintext... use rotating ephemeral discovery tokens...
/// authenticated handshake resolves them to actual device identity."
///
/// Deliberately opaque bytes, not a struct with an embedded
/// `AccountId`/`DeviceId` field — a token that structurally *could*
/// carry either of those in cleartext would make it possible for a
/// careless caller to serialize and broadcast one, which is exactly
/// what §107 forbids. Resolving a token back to a real device requires
/// [`RotatingDiscoveryToken::resolve`], which needs the account's
/// resolution secret to do the correlation — nothing about the token's
/// own bytes leaks that mapping.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RotatingDiscoveryToken(Vec<u8>);

/// How long a single discovery token is valid for before a new one
/// must be derived — the actual rotation, not just the capability to
/// have one. §107 requires tokens to rotate; a `RotatingDiscoveryToken`
/// type that never expired would only be "rotating" in name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiscoveryTokenEpoch(pub u64);

impl DiscoveryTokenEpoch {
    /// A caller picks the rotation period (§107 doesn't mandate one);
    /// this only defines how an instant maps to an epoch number, so
    /// two devices with synchronized clocks and the same period derive
    /// the same epoch independently, without exchanging one.
    pub fn for_instant(unix_millis: u64, rotation_period: Duration) -> Self {
        let period_millis = rotation_period.as_millis().max(1) as u64;
        Self(unix_millis / period_millis)
    }
}

impl RotatingDiscoveryToken {
    /// Derives an ephemeral token for one (account, device, epoch)
    /// triple from a resolution secret the account controls — a plain
    /// HMAC-shaped derivation (BLAKE3 keyed hash), matching this
    /// crate's existing preference for BLAKE3 over inventing an HMAC
    /// construction (see [`crate::verification_code`]'s own use of
    /// it). Two different epochs for the same device produce
    /// unlinkable tokens, satisfying §108's "avoid stable identifiers"
    /// as a consequence of how the token is derived, not as a separate
    /// rule layered on top.
    pub fn derive(
        resolution_secret: &[u8; 32],
        account_id: AccountId,
        device_id: DeviceId,
        epoch: DiscoveryTokenEpoch,
    ) -> Self {
        let mut input = Vec::new();
        input.extend_from_slice(account_id.to_string().as_bytes());
        input.extend_from_slice(device_id.to_string().as_bytes());
        input.extend_from_slice(&epoch.0.to_le_bytes());
        let hash = blake3::keyed_hash(resolution_secret, &input);
        Self(hash.as_bytes().to_vec())
    }

    /// §107's "authenticated handshake resolves them to actual device
    /// identity": given the same resolution secret and a candidate
    /// (account, device) pair for the same epoch, confirms whether
    /// this token actually names that device. A real handshake would
    /// call this once per candidate it already suspects (e.g. from an
    /// existing contact list) rather than reversing the hash — the
    /// derivation is one-way by construction, matching §108's demand
    /// that passive observers gain nothing from the token alone.
    pub fn resolve(
        &self,
        resolution_secret: &[u8; 32],
        account_id: AccountId,
        device_id: DeviceId,
        epoch: DiscoveryTokenEpoch,
    ) -> bool {
        let candidate = Self::derive(resolution_secret, account_id, device_id, epoch);
        candidate.0 == self.0
    }
}

/// §108: "transport discovery identity and cryptographic account
/// identity should be separated." A marker trait, not a data type —
/// its only job is to give transport-layer discovery identifiers (a
/// BLE advertisement id, a Wi-Fi service name, whatever a real
/// transport crate defines) a shared bound that is explicitly *not*
/// [`AccountId`] or [`DeviceId`], so a transport crate implementing
/// this can't accidentally satisfy it by reusing the cryptographic
/// identity type directly. This crate defines no implementors — that
/// is `siar-transport`'s/the platform transport crates' job; this is
/// only the seam between the two layers §108 asks for.
pub trait TransportDiscoveryIdentity: Send + Sync {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_inputs_same_epoch_produce_the_same_token() {
        let secret = [7u8; 32];
        let account = AccountId::new();
        let device = DeviceId::new();
        let epoch = DiscoveryTokenEpoch(42);

        let a = RotatingDiscoveryToken::derive(&secret, account, device, epoch);
        let b = RotatingDiscoveryToken::derive(&secret, account, device, epoch);
        assert_eq!(a, b);
    }

    #[test]
    fn different_epochs_are_unlinkable() {
        let secret = [7u8; 32];
        let account = AccountId::new();
        let device = DeviceId::new();

        let epoch_1 =
            RotatingDiscoveryToken::derive(&secret, account, device, DiscoveryTokenEpoch(1));
        let epoch_2 =
            RotatingDiscoveryToken::derive(&secret, account, device, DiscoveryTokenEpoch(2));

        assert_ne!(
            epoch_1, epoch_2,
            "a passive observer must not be able to link two epochs' tokens to the same device"
        );
    }

    #[test]
    fn resolve_confirms_only_the_true_owning_device() {
        let secret = [7u8; 32];
        let real_account = AccountId::new();
        let real_device = DeviceId::new();
        let other_device = DeviceId::new();
        let epoch = DiscoveryTokenEpoch(9);

        let token = RotatingDiscoveryToken::derive(&secret, real_account, real_device, epoch);

        assert!(token.resolve(&secret, real_account, real_device, epoch));
        assert!(!token.resolve(&secret, real_account, other_device, epoch));
    }

    #[test]
    fn wrong_resolution_secret_never_resolves() {
        let secret = [7u8; 32];
        let wrong_secret = [9u8; 32];
        let account = AccountId::new();
        let device = DeviceId::new();
        let epoch = DiscoveryTokenEpoch(1);

        let token = RotatingDiscoveryToken::derive(&secret, account, device, epoch);
        assert!(!token.resolve(&wrong_secret, account, device, epoch));
    }

    #[test]
    fn epoch_for_instant_groups_nearby_times_and_separates_distant_ones() {
        let period = Duration::from_secs(60);
        let a = DiscoveryTokenEpoch::for_instant(1_000, period);
        let b = DiscoveryTokenEpoch::for_instant(1_500, period);
        let c = DiscoveryTokenEpoch::for_instant(120_000, period);

        assert_eq!(
            a, b,
            "two instants within the same rotation period share an epoch"
        );
        assert_ne!(
            a, c,
            "an instant two rotation periods later must land in a different epoch"
        );
    }
}
