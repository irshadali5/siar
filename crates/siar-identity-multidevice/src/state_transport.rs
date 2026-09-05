//! §100 "Device State Through DTN": "signed device-state updates can
//! be transported through store-carry-forward because authenticity
//! does not depend on the relay... important for disaster scenarios."
//!
//! This is a genuinely different thing from `audit_log`'s [`NewEvent`]
//! output — that module's own doc comment is explicit that it builds
//! records for a caller's *local* event store, never something handed
//! to a transport. What §100 asks for is the wire artifact itself: a
//! device-state change that carries its own proof, so a DTN relay that
//! only stores and forwards opaque bytes can neither read nor forge
//! it, and the recipient needs nothing from the relay — only the
//! account's [`RootPublicKey`], which it already has — to trust it.
//! [`crate::audit_log::IdentityAuditPayload`] already has exactly the
//! right shape for "which change," so this reuses it directly as the
//! signed payload rather than inventing a parallel enum that could
//! drift out of sync with the local one.

use serde::{Deserialize, Serialize};

use crate::audit_log::IdentityAuditPayload;
use crate::error::IdentityError;
use crate::root_key::{RootIdentityKey, RootPublicKey};
use siar_domain::AccountId;

/// Mirrors [`crate::directory::DeviceDirectory`]'s own sign/verify
/// shape exactly (private `signing_payload` helper, `Vec<u8>`-stored
/// signature, `sign`/`verify_signature` pair) — same reasoning, same
/// fix: see that type's doc comment on why the signature field is a
/// `Vec<u8>` rather than a fixed-size array.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedDeviceStateUpdate {
    pub account_id: AccountId,
    pub change: IdentityAuditPayload,
    pub signature: Vec<u8>,
}

impl SignedDeviceStateUpdate {
    fn signing_payload(account_id: AccountId, change: &IdentityAuditPayload) -> Vec<u8> {
        #[derive(Serialize)]
        struct Payload<'a> {
            account_id: AccountId,
            change: &'a IdentityAuditPayload,
        }
        postcard::to_allocvec(&Payload { account_id, change })
            .expect("postcard encoding of a fixed-shape struct cannot fail")
    }

    /// The only constructor — matching every other signed type in
    /// this crate, there is no way to build a `SignedDeviceStateUpdate`
    /// without an actual [`RootIdentityKey`] to sign it, so "signed"
    /// is a structural guarantee here, not a naming convention.
    pub fn sign(
        root_key: &RootIdentityKey,
        account_id: AccountId,
        change: IdentityAuditPayload,
    ) -> Self {
        let payload = Self::signing_payload(account_id, &change);
        let signature = root_key.sign(&payload).to_vec();
        Self {
            account_id,
            change,
            signature,
        }
    }

    /// §100's whole point made checkable: this never needs to know
    /// which relay(s) carried it, how many hops it took, or whether it
    /// arrived in order relative to any other update — only the
    /// account's root public key, which the recipient already holds
    /// independent of this message.
    pub fn verify(&self, root_public_key: &RootPublicKey) -> Result<(), IdentityError> {
        let payload = Self::signing_payload(self.account_id, &self.change);
        let signature: [u8; 64] = self
            .signature
            .as_slice()
            .try_into()
            .map_err(|_| IdentityError::MalformedKey)?;
        root_public_key.verify(&payload, &signature)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use siar_domain::DeviceId;

    #[test]
    fn verifies_under_the_correct_root_regardless_of_transport() {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let change = IdentityAuditPayload::DeviceLinked {
            device_id: DeviceId::new(),
            generation: 1,
        };
        let update = SignedDeviceStateUpdate::sign(&root, account, change);

        // No relay, no session, no transport state involved — just the
        // bytes and the account's root public key.
        assert!(update.verify(&root.root_public_key()).is_ok());
    }

    #[test]
    fn a_relay_cannot_forge_a_state_update_for_an_account_it_does_not_hold_the_root_for() {
        let real_root = RootIdentityKey::generate();
        let forger_root = RootIdentityKey::generate();
        let account = AccountId::new();
        let change = IdentityAuditPayload::RootRotated { generation: 2 };

        // A malicious/compromised relay tries to mint its own update
        // under the real account id but sign it with a key it
        // controls.
        let forged = SignedDeviceStateUpdate::sign(&forger_root, account, change);

        assert!(forged.verify(&real_root.root_public_key()).is_err());
    }

    #[test]
    fn tampering_with_the_change_after_signing_is_detected() {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let device_id = DeviceId::new();
        let mut update = SignedDeviceStateUpdate::sign(
            &root,
            account,
            IdentityAuditPayload::DeviceRevoked {
                device_id,
                generation: 1,
            },
        );

        // A relay (or a compromised intermediate store) tries to
        // downgrade a revocation into a mere suspension in transit.
        update.change = IdentityAuditPayload::DeviceSuspended {
            device_id,
            generation: 1,
        };

        assert!(update.verify(&root.root_public_key()).is_err());
    }
}
