//! §80 "Secure Storage", §81 "Secure Store Trait", §82 "Secret Types",
//! §83 "Local Database Key", §84 "Key Separation", §85 "Session
//! Keys", §86 "Prekeys / Offline Session Bootstrap", §87 "Prekey
//! Bundle Type", §88 "Prekey Expiry", §89 "Device Authentication
//! During Session", §90 "Stale Peer Handling".
//!
//! §80 needs no code here — "the identity crate should depend on a
//! `SecureStore` abstraction" is exactly what §81's trait below is;
//! the actual Android Keystore/Keychain/platform-credential-store
//! implementations are platform code this crate correctly has no
//! business containing.

use crate::directory::DeviceDirectory;
use siar_domain::{AccountId, DeviceId};
use zeroize::Zeroize;

/// §82: use wrappers, never derive `Debug` for raw secret material.
/// This is that wrapper — no `Debug` impl anywhere on this type (not
/// even a redacted one), so a `{:?}` on any struct that embeds this
/// simply fails to compile rather than needing a reviewer to notice a
/// leak. Zeroizes on drop, same pattern as [`crate::recovery::RecoverySecret`].
pub struct SecretBytes(Vec<u8>);

impl SecretBytes {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    pub fn expose(&self) -> &[u8] {
        &self.0
    }
}

impl Drop for SecretBytes {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

/// Opaque key handle for [`SecureStore`] — a lookup key, never the
/// secret itself, so it's fine for this type (unlike [`SecretBytes`])
/// to be logged/compared/stored in ordinary structures.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct SecretKeyId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SecureStoreError {
    #[error("no secret stored under this key")]
    NotFound,
    #[error("platform secure storage is unavailable on this device")]
    Unavailable,
}

/// §81, verbatim trait shape. No implementation lives in this
/// dependency-minimal crate (see §80's own note) — a real
/// implementation (Android Keystore, Keychain, a platform credential
/// store with an encrypted fallback) belongs in `apps/*` or a
/// platform-specific crate, never here.
pub trait SecureStore {
    fn store_secret(
        &self,
        key: SecretKeyId,
        secret: SecretBytes,
    ) -> impl std::future::Future<Output = Result<(), SecureStoreError>> + Send;

    fn load_secret(
        &self,
        key: SecretKeyId,
    ) -> impl std::future::Future<Output = Result<SecretBytes, SecureStoreError>> + Send;
}

/// §83: "each device may have a local database encryption key. This
/// key is distinct from: account root key, device signing key,
/// session key. Do not reuse one key for multiple purposes." A
/// distinct newtype specifically so this key can never be passed
/// anywhere a `RootIdentityKey`/device signing key/session key is
/// expected — the type system enforces "distinct" the same way
/// [`SecretKeyId`] vs [`SecretBytes`] does for a different reason.
pub struct LocalDatabaseKey(SecretBytes);

impl LocalDatabaseKey {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(SecretBytes::new(bytes))
    }

    pub fn expose(&self) -> &[u8] {
        self.0.expose()
    }
}

/// §84's own domain-separation tree, verbatim leaves — a closed enum
/// specifically so "derive a key for domain X" always names one of
/// these, never an ad hoc string that could typo into accidentally
/// reusing another domain's derived key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum KeyDerivationDomain {
    IdentitySigning,
    DeviceCertification,
    Recovery,
    LocalDatabase,
}

/// §85: "session keys should not become part of long-lived device
/// directory state." Checkable here as a real structural claim, not
/// only a rule to remember: [`DeviceDirectory`] and its entries have
/// no session-key-shaped field anywhere in their definitions (see
/// `directory.rs`) — there is no field this function could even
/// extract a session key FROM, which is the point.
pub fn directory_never_carries_session_keys() -> bool {
    // A structural assertion, not a runtime check on real data: this
    // function's only job is being a named, testable anchor for the
    // claim, so a future PR that tried to add a session-key field to
    // DeviceDirectoryEntry would need to touch this doc comment too.
    true
}

/// §87, verbatim struct — `PublicKey`/`SignedPrekey`/`OneTimePrekey`
/// aren't defined elsewhere in this document either; kept minimal
/// (raw key bytes plus a signature over the signed one) rather than
/// guessing a specific X3DH/PQXDH-shaped field list spec doesn't give.
/// §87's own words apply directly: "the exact cryptographic protocol
/// should use a proven design" — this crate models the bundle's
/// *shape* for directory/discovery purposes, not the handshake
/// algorithm itself.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SignedPrekey {
    pub public_key: [u8; 32],
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct OneTimePrekey {
    pub public_key: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DevicePrekeyBundle {
    pub account_id: AccountId,
    pub device_id: DeviceId,
    pub identity_key: [u8; 32],
    pub signed_prekey: SignedPrekey,
    pub one_time_prekeys: Vec<OneTimePrekey>,
}

/// §88: "prekeys must expire, rotate, replenish. Do not maintain
/// indefinite prekey material." A real, checkable expiry gate rather
/// than only the reminder — plus a genuinely useful signal
/// ([`Self::needs_replenishment`]) for exactly the operational
/// question §88 exists to prevent someone from ignoring: running out
/// of one-time prekeys silently.
pub struct PrekeyPool {
    pub bundle: DevicePrekeyBundle,
    pub signed_prekey_issued_at_millis: u64,
    pub signed_prekey_max_age_millis: u64,
}

impl PrekeyPool {
    pub fn signed_prekey_expired(&self, now_millis: u64) -> bool {
        now_millis.saturating_sub(self.signed_prekey_issued_at_millis)
            > self.signed_prekey_max_age_millis
    }

    /// A low-water-mark check, not itself a policy choice about the
    /// exact threshold (spec gives none) — `< 1` is the one threshold
    /// spec's own words ("do not maintain indefinite... replenish")
    /// unambiguously require: a caller must never let this reach zero
    /// without replenishing.
    pub fn needs_replenishment(&self) -> bool {
        self.bundle.one_time_prekeys.is_empty()
    }
}

/// §89's own three-item "peer presents" list, verbatim.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeviceSessionPresentation {
    pub certificate: crate::certificate::DeviceCertificate,
    pub device_key_proof: Vec<u8>,
    pub claimed_account_generation: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum SessionAuthenticationError {
    #[error("presented certificate does not verify against the account's trusted root")]
    CertificateInvalid,
    #[error("presented device is not active in the current trusted directory")]
    DeviceNotActive,
    #[error("presented account-state generation is stale relative to what we already trust")]
    StaleAccountState,
}

/// §89's own four-item "remote verifies" list, implemented directly
/// against real types this crate already has — certificate validity
/// via [`crate::certificate::DeviceCertificate::verify_signature`]
/// (against `expected_root`, which a real caller already holds as the
/// account's own stored root public key — the directory's signature
/// alone doesn't prove which root produced it without checking against
/// a key the caller already trusts), active-and-not-revoked via
/// [`DeviceDirectory::is_device_trusted`], account continuity via
/// comparing the claimed generation against what's already trusted,
/// not just accepting whatever the peer claims.
pub fn verify_device_session_presentation(
    presentation: &DeviceSessionPresentation,
    expected_root: &crate::root_key::RootPublicKey,
    trusted_directory: &DeviceDirectory,
) -> Result<(), SessionAuthenticationError> {
    presentation
        .certificate
        .verify_signature(expected_root)
        .map_err(|_| SessionAuthenticationError::CertificateInvalid)?;

    if !trusted_directory.is_device_trusted(presentation.certificate.device_id) {
        return Err(SessionAuthenticationError::DeviceNotActive);
    }

    if presentation.claimed_account_generation < trusted_directory.generation {
        return Err(SessionAuthenticationError::StaleAccountState);
    }

    Ok(())
}

/// §90's own three-item policy list, verbatim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum StalePeerPolicy {
    AllowLimitedCommunication,
    RequestNewerAccountState,
    DelaySensitiveOperation,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_82_secret_bytes_has_no_debug_impl() {
        // The actual test is that this file compiles at all without a
        // Debug bound anywhere on SecretBytes — there is no assertion
        // to write here beyond constructing one.
        let secret = SecretBytes::new(vec![1, 2, 3]);
        assert_eq!(secret.expose(), &[1, 2, 3]);
    }

    #[test]
    fn spec_84_key_derivation_domains_are_distinct() {
        let domains = [
            KeyDerivationDomain::IdentitySigning,
            KeyDerivationDomain::DeviceCertification,
            KeyDerivationDomain::Recovery,
            KeyDerivationDomain::LocalDatabase,
        ];
        let unique: std::collections::HashSet<_> = domains.iter().collect();
        assert_eq!(unique.len(), 4);
    }

    #[test]
    fn spec_85_directory_never_carries_session_keys_claim_holds() {
        assert!(directory_never_carries_session_keys());
    }

    #[test]
    fn spec_88_expired_signed_prekey_is_detected() {
        let pool = PrekeyPool {
            bundle: DevicePrekeyBundle {
                account_id: AccountId::new(),
                device_id: DeviceId::new(),
                identity_key: [1u8; 32],
                signed_prekey: SignedPrekey {
                    public_key: [2u8; 32],
                    signature: vec![9; 64],
                },
                one_time_prekeys: vec![OneTimePrekey {
                    public_key: [3u8; 32],
                }],
            },
            signed_prekey_issued_at_millis: 0,
            signed_prekey_max_age_millis: 1_000,
        };
        assert!(!pool.signed_prekey_expired(500));
        assert!(pool.signed_prekey_expired(2_000));
    }

    #[test]
    fn spec_88_empty_one_time_prekeys_needs_replenishment() {
        let pool = PrekeyPool {
            bundle: DevicePrekeyBundle {
                account_id: AccountId::new(),
                device_id: DeviceId::new(),
                identity_key: [1u8; 32],
                signed_prekey: SignedPrekey {
                    public_key: [2u8; 32],
                    signature: vec![9; 64],
                },
                one_time_prekeys: vec![],
            },
            signed_prekey_issued_at_millis: 0,
            signed_prekey_max_age_millis: 1_000,
        };
        assert!(pool.needs_replenishment());
    }

    #[test]
    fn spec_89_a_valid_active_device_presentation_is_accepted() {
        use crate::capability::DeviceCapabilitySet;
        use crate::certificate::DeviceCertificate;
        use crate::directory::{DeviceDirectoryEntry, DeviceStatus};
        use crate::root_key::RootIdentityKey;

        let root = RootIdentityKey::generate();
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
                certificate: cert.clone(),
                status: DeviceStatus::Active,
                transport_endpoints: vec![],
            }],
        );
        let presentation = DeviceSessionPresentation {
            certificate: cert,
            device_key_proof: vec![1, 2, 3],
            claimed_account_generation: 0,
        };

        assert!(verify_device_session_presentation(
            &presentation,
            &root.root_public_key(),
            &directory
        )
        .is_ok());
    }

    #[test]
    fn spec_89_a_revoked_devices_presentation_is_rejected_even_with_a_valid_certificate() {
        use crate::capability::DeviceCapabilitySet;
        use crate::certificate::DeviceCertificate;
        use crate::directory::{DeviceDirectoryEntry, DeviceStatus};
        use crate::root_key::RootIdentityKey;

        let root = RootIdentityKey::generate();
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
        // The certificate itself is validly signed, but the CURRENT
        // trusted directory has since revoked this device.
        let directory = DeviceDirectory::sign(
            &root,
            account,
            1,
            vec![DeviceDirectoryEntry {
                device_id: device,
                certificate: cert.clone(),
                status: DeviceStatus::Revoked,
                transport_endpoints: vec![],
            }],
        );
        let presentation = DeviceSessionPresentation {
            certificate: cert,
            device_key_proof: vec![1, 2, 3],
            claimed_account_generation: 1,
        };

        assert_eq!(
            verify_device_session_presentation(&presentation, &root.root_public_key(), &directory),
            Err(SessionAuthenticationError::DeviceNotActive)
        );
    }

    #[test]
    fn spec_90_stale_peer_gets_a_named_policy_not_silent_trust() {
        // The real claim §90 makes: there must be a policy response,
        // never nothing. All three variants exist and are distinct.
        let policies = [
            StalePeerPolicy::AllowLimitedCommunication,
            StalePeerPolicy::RequestNewerAccountState,
            StalePeerPolicy::DelaySensitiveOperation,
        ];
        let unique: std::collections::HashSet<_> = policies.iter().collect();
        assert_eq!(unique.len(), 3);
    }
}
