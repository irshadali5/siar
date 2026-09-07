//! §184 "Contact Pairing vs Device Linking": "these are distinct...
//! must use different protocol states and UI language."
//!
//! No new type — a traceability/proof module, same approach as
//! [`crate::reuse_patterns`]. The claim is that two type families never
//! overlap; the way to actually check that in Rust is to try to use
//! one where the other is expected and confirm it doesn't compile —
//! which isn't something a runtime `#[test]` can directly assert, so
//! instead these tests construct one real example of each side by
//! side and confirm the two families' values are never comparable to
//! each other. Contact pairing lives in
//! [`crate::contact_verification`]; device linking lives in
//! [`crate::linking_state_machine`]/[`crate::platform_boundary::NewDeviceUxStep`]
//! — two disjoint modules, not two variants of one enum.

#[cfg(test)]
mod tests {
    use crate::contact_verification::{
        OfflineVerification, OfflineVerificationMethod, VerificationPolicy, VerifiedContact,
    };
    use crate::linking_state_machine::LinkingState;
    use crate::platform_boundary::NewDeviceUxStep;
    use crate::root_key::RootIdentityKey;
    use siar_domain::AccountId;

    /// §184's own worked example: Alice↔Bob is contact pairing; Alice
    /// Phone↔Alice Laptop is device linking. Built side by side here to
    /// make the point concrete rather than abstract.
    #[test]
    fn alice_bob_pairing_and_alice_phone_laptop_linking_use_disjoint_types() {
        // Contact pairing: Alice verifies Bob's account root identity.
        let bob_root = RootIdentityKey::generate();
        let bob_account = AccountId::new();
        let pairing = OfflineVerification::bind_root_identity(
            OfflineVerificationMethod::Qr,
            bob_root.root_public_key(),
        );
        let verified_bob =
            VerifiedContact::new(bob_account, pairing, VerificationPolicy::default());
        assert!(verified_bob.is_continuous());

        // Device linking: Alice's phone links her laptop as a new
        // device on the SAME account — a completely different state
        // machine, never touching `VerifiedContact` or
        // `OfflineVerification` at any point.
        let linking = LinkingState::new()
            .advance(LinkingState::InviteShared)
            .unwrap()
            .advance(LinkingState::PeerConnected)
            .unwrap();
        assert_eq!(linking, LinkingState::PeerConnected);

        // The two flows' "in progress" states are not the same type
        // and cannot be compared to each other at all — this line
        // would be a compile error if uncommented:
        // assert_eq!(linking, verified_bob);
    }

    /// §184's "different UI language" made checkable at the type
    /// level: [`NewDeviceUxStep`] (linking) and
    /// [`OfflineVerificationMethod`] (pairing) are separate enums with
    /// no shared type connecting them.
    #[test]
    fn linking_ux_steps_and_pairing_methods_are_separate_enums() {
        let linking_step = NewDeviceUxStep::Verify;
        let pairing_method = OfflineVerificationMethod::ManualFingerprint;
        assert_eq!(linking_step, NewDeviceUxStep::Verify);
        assert_eq!(pairing_method, OfflineVerificationMethod::ManualFingerprint);
        // No `PartialEq` exists between these two types — there is no
        // `linking_step == pairing_method` this test could even write.
    }
}
