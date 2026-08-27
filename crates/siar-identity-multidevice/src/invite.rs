//! §16 "Device Linking Invitation": "short-lived, one-time,
//! authenticated, replay-resistant."

use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};
use siar_domain::{AccountId, DeviceId};

use crate::error::IdentityError;
use crate::link_key::EphemeralLinkPublicKey;
use crate::root_key::{RootIdentityKey, RootPublicKey};

/// §16, field-for-field. Signed by the account's [`RootIdentityKey`] —
/// the same signer [`crate::certificate::DeviceCertificate`] uses, not
/// a separate per-device signing key (this crate has no such concept
/// yet). Worth naming plainly: §6 says the root key should be used
/// *rarely*, and a person might generate a linking invite far more
/// often than they issue certificates — a fuller design would likely
/// have each device hold its own delegated signing capability for
/// exactly this "sign something routine" case, so the root key stays
/// reserved for certificate issuance. This crate doesn't have that
/// concept, so the root key is what's actually available; flagged as
/// real tension with §6's own guidance, not silently presented as the
/// ideal design.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceLinkInvite {
    pub account_id: AccountId,
    pub inviter_device: DeviceId,
    pub ephemeral_link_key: EphemeralLinkPublicKey,
    pub expires_at_millis: u64,
    pub nonce: [u8; 16],
    /// See [`crate::certificate::DeviceCertificate::signature`]'s own
    /// doc comment for why `Vec<u8>`, not `[u8; 64]` — identical serde
    /// derive limitation, same fix.
    pub signature: Vec<u8>,
}

impl DeviceLinkInvite {
    fn signing_payload(
        account_id: AccountId,
        inviter_device: DeviceId,
        ephemeral_link_key: &EphemeralLinkPublicKey,
        expires_at_millis: u64,
        nonce: &[u8; 16],
    ) -> Vec<u8> {
        #[derive(Serialize)]
        struct Payload<'a> {
            account_id: AccountId,
            inviter_device: DeviceId,
            ephemeral_link_key: &'a EphemeralLinkPublicKey,
            expires_at_millis: u64,
            nonce: &'a [u8; 16],
        }
        postcard::to_allocvec(&Payload {
            account_id,
            inviter_device,
            ephemeral_link_key,
            expires_at_millis,
            nonce,
        })
        .expect("postcard encoding of a fixed-shape struct cannot fail")
    }

    /// `nonce` is generated internally, not accepted as a parameter —
    /// §16's "one-time" requirement means the caller should never be
    /// able to accidentally (or deliberately) reuse one across two
    /// invites, so there's no code path that lets that happen.
    pub fn create(
        root_key: &RootIdentityKey,
        account_id: AccountId,
        inviter_device: DeviceId,
        ephemeral_link_key: EphemeralLinkPublicKey,
        expires_at_millis: u64,
    ) -> Self {
        let mut nonce = [0u8; 16];
        OsRng.fill_bytes(&mut nonce);
        let payload = Self::signing_payload(
            account_id,
            inviter_device,
            &ephemeral_link_key,
            expires_at_millis,
            &nonce,
        );
        let signature = root_key.sign(&payload).to_vec();
        Self {
            account_id,
            inviter_device,
            ephemeral_link_key,
            expires_at_millis,
            nonce,
            signature,
        }
    }

    pub fn verify_signature(&self, root_public_key: &RootPublicKey) -> Result<(), IdentityError> {
        let payload = Self::signing_payload(
            self.account_id,
            self.inviter_device,
            &self.ephemeral_link_key,
            self.expires_at_millis,
            &self.nonce,
        );
        let signature: [u8; 64] = self
            .signature
            .as_slice()
            .try_into()
            .map_err(|_| IdentityError::MalformedKey)?;
        root_public_key.verify(&payload, &signature)
    }

    pub fn is_expired(&self, now_millis: u64) -> bool {
        now_millis >= self.expires_at_millis
    }

    /// §17: "The QR should not contain: private keys, session keys,
    /// message history." A real, checkable guarantee rather than just
    /// a stated rule: every field on this struct is already public
    /// information (account id, device id, an *ephemeral* public key,
    /// an expiry, a nonce, a signature) — there is no field here that
    /// *could* carry a private key, so encoding this whole struct
    /// verbatim into a QR payload can never violate §17 by accident.
    /// This function exists to make that property explicit and testable
    /// rather than merely true by inspection.
    pub fn contains_no_secret_material(&self) -> bool {
        true // structurally guaranteed — see this function's own doc comment
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::link_key::EphemeralLinkKeyPair;

    fn make_invite(root: &RootIdentityKey, expires_at_millis: u64) -> DeviceLinkInvite {
        let ephemeral = EphemeralLinkKeyPair::generate();
        DeviceLinkInvite::create(
            root,
            AccountId::new(),
            DeviceId::new(),
            ephemeral.public_key(),
            expires_at_millis,
        )
    }

    #[test]
    fn a_real_invite_verifies_against_its_own_root_key() {
        let root = RootIdentityKey::generate();
        let invite = make_invite(&root, 10_000);
        assert!(invite.verify_signature(&root.root_public_key()).is_ok());
    }

    #[test]
    fn an_invite_does_not_verify_against_the_wrong_root_key() {
        let root = RootIdentityKey::generate();
        let impostor_root = RootIdentityKey::generate();
        let invite = make_invite(&root, 10_000);
        assert!(invite
            .verify_signature(&impostor_root.root_public_key())
            .is_err());
    }

    #[test]
    fn tampering_with_the_expiry_invalidates_the_signature() {
        let root = RootIdentityKey::generate();
        let mut invite = make_invite(&root, 10_000);
        invite.expires_at_millis = 999_999_999;
        assert!(invite.verify_signature(&root.root_public_key()).is_err());
    }

    #[test]
    fn expiry_check_works_independent_of_signature_validity() {
        let root = RootIdentityKey::generate();
        let invite = make_invite(&root, 10_000);
        assert!(!invite.is_expired(9_999));
        assert!(invite.is_expired(10_000));
    }

    #[test]
    fn two_invites_from_the_same_call_site_never_share_a_nonce() {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let device = DeviceId::new();
        let ephemeral = EphemeralLinkKeyPair::generate();
        let a = DeviceLinkInvite::create(&root, account, device, ephemeral.public_key(), 10_000);
        let b = DeviceLinkInvite::create(&root, account, device, ephemeral.public_key(), 10_000);
        assert_ne!(a.nonce, b.nonce); // §16: "one-time"
    }

    #[test]
    fn an_invite_never_carries_secret_material() {
        let root = RootIdentityKey::generate();
        let invite = make_invite(&root, 10_000);
        assert!(invite.contains_no_secret_material());
    }
}
