//! §91 "Offline Identity Verification", §92 "Contact Verification", §93
//! "New Device Notification", §94 "Identity Change Notification", §95
//! "Verification Modes", §96 "Device Transparency Log", §97 "Self-Hosted
//! Transparency", §99 "Directory Service Role".
//!
//! §98 "No Mandatory Central Directory" gets no new code here, same
//! posture this crate has already taken for other "this is already
//! true structurally" sections (see `directory.rs`'s own §54 note):
//! every function in this crate — [`DeviceDirectory::verify_signature`],
//! [`crate::trust_store::TrustedAccountStore::accept`],
//! [`DirectoryServiceResponse::verify_and_accept`] below — operates on
//! a signed [`DeviceDirectory`] plus a [`RootPublicKey`] the caller
//! already has, never on a live connection to any server. §98's actual
//! requirement ("must work P2P/offline/LAN-only/DTN... a directory
//! server may improve discovery, but it is not the source of
//! cryptographic truth") is exactly what §99's own function below
//! enforces for the one optional-server case this crate does model.

use serde::{Deserialize, Serialize};

use crate::directory::{DeviceDirectory, DeviceEndpoint};
use crate::error::IdentityError;
use crate::root_key::RootPublicKey;
use crate::secure_storage::DevicePrekeyBundle;
use siar_domain::{AccountId, DeviceId};

/// §91's four listed methods. Deliberately no "matched via transport
/// endpoint" variant of any kind — §91's actual point is that
/// whichever of these is used, what gets bound is the account's
/// [`RootPublicKey`], never "the address we happened to see them at."
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OfflineVerificationMethod {
    Qr,
    NumericCode,
    Nfc,
    ManualFingerprint,
}

/// §91: the record that "verification bound the account root identity,
/// not just current transport endpoint" produces. The only way to get
/// one is [`OfflineVerification::bind_root_identity`] — there's no
/// public constructor that takes a certificate, device id, or
/// transport address instead, matching this crate's existing pattern
/// (`approval::LinkingApprovalPrompt`, `revocation::verify_revocation`)
/// of making a spec rule about *how* a value came to exist a real
/// constructor shape, not just a comment someone could bypass.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OfflineVerification {
    pub method: OfflineVerificationMethod,
    pub root_identity: RootPublicKey,
}

impl OfflineVerification {
    /// §91's own payoff — "this prevents re-verifying every transport
    /// change" — only holds if what's recorded is the root key, not a
    /// device/session/endpoint value that changes on its own.
    pub fn bind_root_identity(
        method: OfflineVerificationMethod,
        root_identity: RootPublicKey,
    ) -> Self {
        Self {
            method,
            root_identity,
        }
    }
}

/// §95, verbatim three variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationPolicy {
    TrustAccountRoot,
    VerifyEveryDevice,
    HighSecurity,
}

impl Default for VerificationPolicy {
    /// §95: "Default messenger: verify account root."
    fn default() -> Self {
        VerificationPolicy::TrustAccountRoot
    }
}

/// §92: a verified contact's state is tied to `AccountId`/root
/// identity, "with awareness of root rotation." `verified_root` is the
/// root the actual offline verification was performed against;
/// `current_root` is whatever this account's root is now, updated
/// independently via [`VerifiedContact::observe_current_root`]. The two
/// only start out equal — nothing here silently keeps them in sync,
/// so a real rotation is always visible as a state change, not
/// something this type quietly absorbs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifiedContact {
    pub account_id: AccountId,
    pub verified_root: RootPublicKey,
    pub current_root: RootPublicKey,
    pub method: OfflineVerificationMethod,
    pub policy: VerificationPolicy,
}

impl VerifiedContact {
    pub fn new(
        account_id: AccountId,
        verification: OfflineVerification,
        policy: VerificationPolicy,
    ) -> Self {
        Self {
            account_id,
            verified_root: verification.root_identity,
            current_root: verification.root_identity,
            method: verification.method,
            policy,
        }
    }

    /// §92's "awareness of root rotation" made checkable: still
    /// anchored to the exact root that was actually verified.
    pub fn is_continuous(&self) -> bool {
        self.verified_root == self.current_root
    }

    /// §92's second half: "new devices signed under the verified
    /// account can inherit account-level trust according to policy."
    /// Only true under `TrustAccountRoot`, and only while continuous —
    /// `VerifyEveryDevice`/`HighSecurity` never inherit trust
    /// automatically regardless of continuity, matching §95's own
    /// distinction between "verify account root" and "require every
    /// device approval."
    pub fn new_device_inherits_trust(&self) -> bool {
        self.is_continuous() && matches!(self.policy, VerificationPolicy::TrustAccountRoot)
    }

    /// Records the account's root as it stands now, independent of
    /// what was verified. Call this whenever a fresh, signed
    /// [`DeviceDirectory`] for the account is seen — it does not
    /// itself decide whether the new root is legitimate (that's
    /// [`crate::root_rotation::verify_root_rotation`]'s job); it only
    /// records the fact for [`VerifiedContact::is_continuous`] to
    /// report on.
    pub fn observe_current_root(&mut self, current_root: RootPublicKey) {
        self.current_root = current_root;
    }

    /// Re-anchors verification to the account's new root after a
    /// rotation. Deliberately a separate, explicitly-named action
    /// rather than something `observe_current_root` does on its own —
    /// re-verifying trust is a decision a caller makes (having already
    /// checked a real `RootRotation` via
    /// `crate::root_rotation::verify_root_rotation`, or performed a
    /// fresh offline verification), never something this type applies
    /// silently just because a new root was observed.
    pub fn re_anchor(&mut self, new_verification: OfflineVerification) {
        self.verified_root = new_verification.root_identity;
        self.current_root = new_verification.root_identity;
        self.method = new_verification.method;
    }
}

/// §93 "New Device Notification", §94 "Identity Change Notification":
/// deliberately two separate variants, not one generic "account
/// changed" event with a severity field. §94's own words — "this is
/// more serious than adding a device... the distinction must be
/// explicit" — rule out a single type where a caller could default or
/// misconfigure the severity; the variant itself is the distinction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IdentityNotification {
    /// §93: "Alice added a new device."
    NewDevice {
        account_id: AccountId,
        device_id: DeviceId,
    },
    /// §94: "Security identity changed." Carries both roots so a UI
    /// can show the actual before/after, not just the fact that
    /// something changed.
    IdentityChanged {
        account_id: AccountId,
        previous_root: RootPublicKey,
        new_root: RootPublicKey,
    },
}

impl IdentityNotification {
    /// §93: "may show this as informational or security-significant
    /// depending on verification policy... do not silently hide device
    /// changes in high-security mode." §94: unlike §93, spec never
    /// offers a policy under which an identity change is merely
    /// informational — it is always security-significant here,
    /// regardless of `policy`.
    pub fn is_security_significant(&self, policy: VerificationPolicy) -> bool {
        match self {
            // Informational only under the default TrustAccountRoot
            // policy; both VerifyEveryDevice and HighSecurity treat a
            // new device as security-significant, satisfying §93's
            // "do not silently hide... in high-security mode."
            IdentityNotification::NewDevice { .. } => {
                !matches!(policy, VerificationPolicy::TrustAccountRoot)
            }
            IdentityNotification::IdentityChanged { .. } => true,
        }
    }
}

/// §96 "Device Transparency Log": spec's own words call this "a future
/// enhancement," so this is a boundary trait this crate leaves room
/// for — matching `recovery::RecoveryKeyDerivation`'s existing
/// precedent for a real external capability this dependency-minimal
/// crate doesn't implement itself (an actual append-only transparency
/// log needs real storage and, for the multi-writer case, consensus —
/// the same category of dependency this crate has avoided everywhere
/// else). Nothing in this crate calls `append` anywhere; the trait
/// exists so a future implementation has a concrete shape to target
/// without this crate needing to change.
pub trait DeviceTransparencyLog {
    fn append(&mut self, entry: DeviceTransparencyEntry);
}

/// The shape a real [`DeviceTransparencyLog`] would append — device
/// additions/revocations only, per §96's own two named examples;
/// nothing about rotation or suspension, which §96 doesn't mention.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceTransparencyEntry {
    pub account_id: AccountId,
    pub device_id: DeviceId,
    pub change: DeviceTransparencyChange,
    pub generation: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceTransparencyChange {
    Added,
    Revoked,
}

/// §97: whether an organization hosts its own transparency service.
/// Deliberately a pure label with no behavior of its own — hosting one
/// changes nothing about how directories or certificates are verified
/// (§97's own "without changing basic identity semantics"); any real
/// [`DeviceTransparencyLog`] implementation, self-hosted or not, plugs
/// into the same trait above without this crate caring which mode
/// produced it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransparencyDeploymentMode {
    None,
    SelfHosted,
}

/// §99: exactly what spec lists an optional directory service may
/// provide — nothing more. In particular, no field here for a "trust
/// level" or "verification status" a directory service might claim on
/// a device's behalf; §99's own next sentence ("it must not be trusted
/// to forge identities") is why those don't exist as inputs at all,
/// not just values this crate ignores.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryServiceResponse {
    pub directory: DeviceDirectory,
    pub prekey_bundles: Vec<DevicePrekeyBundle>,
    pub transport_hints: Vec<DeviceEndpoint>,
}

impl DirectoryServiceResponse {
    /// §99's own rule made structural: "it must not be trusted to
    /// forge identities... all state is verified cryptographically."
    /// This is the ONLY way to accept a `DirectoryServiceResponse` —
    /// it does exactly what every other path into this crate's trust
    /// state already requires
    /// ([`DeviceDirectory::verify_signature`]), so a directory service
    /// gets no special, weaker acceptance path just because it's "the
    /// directory service." A response whose directory signature
    /// doesn't verify is rejected outright, and its prekey bundles and
    /// transport hints are discarded along with it — both were only
    /// ever claims from the same unauthenticated source as the
    /// directory itself, so there is no reading in which they're safe
    /// to keep while the directory they arrived with is rejected.
    pub fn verify_and_accept(
        self,
        root_public_key: &RootPublicKey,
    ) -> Result<
        (
            DeviceDirectory,
            Vec<DevicePrekeyBundle>,
            Vec<DeviceEndpoint>,
        ),
        IdentityError,
    > {
        self.directory.verify_signature(root_public_key)?;
        Ok((self.directory, self.prekey_bundles, self.transport_hints))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::root_key::RootIdentityKey;
    use siar_domain::AccountId;

    fn account() -> AccountId {
        AccountId::new()
    }

    #[test]
    fn offline_verification_binds_root_not_transport() {
        let root = RootIdentityKey::generate();
        let v = OfflineVerification::bind_root_identity(
            OfflineVerificationMethod::NumericCode,
            root.root_public_key(),
        );
        assert_eq!(v.root_identity, root.root_public_key());
    }

    #[test]
    fn default_policy_is_trust_account_root() {
        assert_eq!(
            VerificationPolicy::default(),
            VerificationPolicy::TrustAccountRoot
        );
    }

    #[test]
    fn verified_contact_starts_continuous() {
        let root = RootIdentityKey::generate();
        let v = OfflineVerification::bind_root_identity(
            OfflineVerificationMethod::Qr,
            root.root_public_key(),
        );
        let contact = VerifiedContact::new(account(), v, VerificationPolicy::default());
        assert!(contact.is_continuous());
        assert!(contact.new_device_inherits_trust());
    }

    #[test]
    fn root_rotation_breaks_continuity_and_trust_inheritance() {
        let root = RootIdentityKey::generate();
        let new_root = RootIdentityKey::generate();
        let v = OfflineVerification::bind_root_identity(
            OfflineVerificationMethod::Qr,
            root.root_public_key(),
        );
        let mut contact = VerifiedContact::new(account(), v, VerificationPolicy::default());

        contact.observe_current_root(new_root.root_public_key());

        assert!(!contact.is_continuous());
        assert!(
            !contact.new_device_inherits_trust(),
            "trust must not be inherited once the account's root has diverged from what was verified"
        );
    }

    #[test]
    fn re_anchor_restores_continuity_under_the_new_root() {
        let root = RootIdentityKey::generate();
        let new_root = RootIdentityKey::generate();
        let v = OfflineVerification::bind_root_identity(
            OfflineVerificationMethod::Qr,
            root.root_public_key(),
        );
        let mut contact = VerifiedContact::new(account(), v, VerificationPolicy::default());
        contact.observe_current_root(new_root.root_public_key());
        assert!(!contact.is_continuous());

        let re_verification = OfflineVerification::bind_root_identity(
            OfflineVerificationMethod::ManualFingerprint,
            new_root.root_public_key(),
        );
        contact.re_anchor(re_verification);

        assert!(contact.is_continuous());
        assert_eq!(contact.method, OfflineVerificationMethod::ManualFingerprint);
    }

    #[test]
    fn verify_every_device_policy_never_inherits_trust_even_when_continuous() {
        let root = RootIdentityKey::generate();
        let v = OfflineVerification::bind_root_identity(
            OfflineVerificationMethod::Qr,
            root.root_public_key(),
        );
        let contact = VerifiedContact::new(account(), v, VerificationPolicy::VerifyEveryDevice);
        assert!(contact.is_continuous());
        assert!(!contact.new_device_inherits_trust());
    }

    #[test]
    fn new_device_notification_is_informational_only_under_trust_account_root() {
        let notification = IdentityNotification::NewDevice {
            account_id: account(),
            device_id: DeviceId::new(),
        };
        assert!(!notification.is_security_significant(VerificationPolicy::TrustAccountRoot));
        assert!(notification.is_security_significant(VerificationPolicy::VerifyEveryDevice));
        assert!(notification.is_security_significant(VerificationPolicy::HighSecurity));
    }

    #[test]
    fn identity_changed_notification_is_always_security_significant() {
        let root = RootIdentityKey::generate();
        let new_root = RootIdentityKey::generate();
        let notification = IdentityNotification::IdentityChanged {
            account_id: account(),
            previous_root: root.root_public_key(),
            new_root: new_root.root_public_key(),
        };
        for policy in [
            VerificationPolicy::TrustAccountRoot,
            VerificationPolicy::VerifyEveryDevice,
            VerificationPolicy::HighSecurity,
        ] {
            assert!(notification.is_security_significant(policy));
        }
    }

    #[test]
    fn directory_service_response_rejects_bad_signature() {
        let real_root = RootIdentityKey::generate();
        let attacker_root = RootIdentityKey::generate();
        let directory = DeviceDirectory::sign(&real_root, account(), 1, vec![]);

        let response = DirectoryServiceResponse {
            directory,
            prekey_bundles: vec![],
            transport_hints: vec![],
        };

        // Verifying against the WRONG root (as if a directory service
        // tried to pass off a directory for an account it doesn't
        // actually hold the root key for) must fail, not just warn.
        let result = response.verify_and_accept(&attacker_root.root_public_key());
        assert!(result.is_err());
    }

    #[test]
    fn directory_service_response_accepts_a_correctly_signed_directory() {
        let root = RootIdentityKey::generate();
        let directory = DeviceDirectory::sign(&root, account(), 1, vec![]);

        let response = DirectoryServiceResponse {
            directory,
            prekey_bundles: vec![],
            transport_hints: vec![],
        };

        let result = response.verify_and_accept(&root.root_public_key());
        assert!(result.is_ok());
    }
}
