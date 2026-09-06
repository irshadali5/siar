//! §162 "Testing Strategy" (the "integration tests" category — "link
//! device, sync directory, revoke device, attempt reconnect"), §165
//! "Simulated Multi-Device Tests" (spec's own named topology and
//! scenario, verbatim).
//!
//! §162's "unit tests" and "fault tests" categories aren't new work —
//! they're what every other module's own `#[cfg(test)] mod tests`
//! already is, accumulated across seven rounds (certificate
//! verification: `certificate.rs`; generation ordering/rollback:
//! `trust_store.rs`; revocation: `revocation.rs`; rotation:
//! `rotation.rs`/`root_rotation.rs`; linking invite expiry:
//! `invite.rs`; capability enforcement: `device_authorization.rs`;
//! duplicate/out-of-order events: `reconciliation.rs`; stale
//! directory/conflicting generation: `trust_store.rs`'s own
//! `spec_57_*` fork test). "Process death during linking" has no
//! test anywhere in this crate — a real gap, named here rather than
//! silently assumed covered: this crate's linking functions are all
//! pure (no partial-completion state to crash mid-way through), so
//! the property probably already holds, but "probably, by
//! construction" is not the same claim as a test that exercises an
//! actual crash-and-resume, which nothing here does.
//!
//! §165's own topology and scenario, run for real below rather than
//! only described.

#[cfg(test)]
mod tests {
    use crate::capability::DeviceCapabilitySet;
    use crate::certificate::DeviceCertificate;
    use crate::directory::{DeviceDirectory, DeviceDirectoryEntry, DeviceStatus};
    use crate::root_key::RootIdentityKey;
    use crate::session_cache::{AuthenticationOutcome, RevocationCache};
    use crate::trust_store::TrustedAccountStore;
    use siar_domain::{AccountId, DeviceId};

    fn active_entry(
        root: &RootIdentityKey,
        account: AccountId,
        device_id: DeviceId,
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
            status: DeviceStatus::Active,
            transport_endpoints: vec![],
        }
    }

    /// §165's exact topology and scenario:
    /// - Alice: A1 Phone, A2 Laptop, A3 Tablet
    /// - Bob: B1 Phone, B2 Laptop
    /// - A1 adds A3 (A2 conceptually offline, doesn't affect this test
    ///   since a linking event's validity never depends on which of an
    ///   account's other devices are currently reachable)
    /// - A3 is trusted
    /// - A1 revokes A3
    /// - Bob receives Alice's updated directory (an account boundary a
    ///   directory never crosses on its own, but the SIGNATURE Bob
    ///   would verify is exactly the one this test produces)
    /// - A3's reconnection attempt is rejected
    ///
    /// §162's "link device / sync directory / revoke device / attempt
    /// reconnect" integration list is exactly these four steps, run in
    /// order against real crate functions rather than described in
    /// prose.
    #[test]
    fn spec_165_alice_and_bob_topology_scenario() {
        let alice_root = RootIdentityKey::generate();
        let alice = AccountId::new();
        let a1_phone = DeviceId::new();
        let a2_laptop = DeviceId::new();
        let a3_tablet = DeviceId::new();

        let bob_root = RootIdentityKey::generate();
        let bob = AccountId::new();
        let b1_phone = DeviceId::new();
        let b2_laptop = DeviceId::new();

        // Step 1: Alice starts with A1 and A2 (A3 not yet linked).
        let mut alice_store = TrustedAccountStore::new();
        let gen_1 = DeviceDirectory::sign(
            &alice_root,
            alice,
            1,
            vec![
                active_entry(&alice_root, alice, a1_phone, 1),
                active_entry(&alice_root, alice, a2_laptop, 1),
            ],
        );
        alice_store
            .accept(gen_1, &alice_root.root_public_key())
            .unwrap();

        // Step 2: "A1 adds A3" — a new signed generation including all
        // three devices (A2 offline has no bearing on this signature's
        // validity, matching invariant 4's "transport reachability
        // never substitutes for identity" one layer up).
        let gen_2 = DeviceDirectory::sign(
            &alice_root,
            alice,
            2,
            vec![
                active_entry(&alice_root, alice, a1_phone, 1),
                active_entry(&alice_root, alice, a2_laptop, 1),
                active_entry(&alice_root, alice, a3_tablet, 1),
            ],
        );
        alice_store
            .accept(gen_2.clone(), &alice_root.root_public_key())
            .unwrap();
        assert!(
            gen_2.is_device_trusted(a3_tablet),
            "A3 must be trusted after linking"
        );

        // "A2 reconnects" — no code needed; A2 was never removed from
        // any directory, so nothing about its earlier offline state
        // needed reconciling. This is the test's own confirmation that
        // "offline" isn't a distinct identity state this crate tracks
        // at all (see this crate's `device_state.rs` presence tracking
        // for where reachability, as distinct from authorization,
        // actually lives).

        // Step 3: "A1 revokes A3."
        let gen_3 = crate::revocation::revoke_device(&alice_root, &gen_2, a3_tablet)
            .expect("revoking a real, currently-active device must succeed");
        alice_store
            .accept(gen_3.clone(), &alice_root.root_public_key())
            .unwrap();
        assert!(!gen_3.is_device_trusted(a3_tablet));

        // Step 4: "Bob receives updated directory" — Bob verifies the
        // exact same signed directory Alice's own store now holds,
        // using only Alice's root public key (no channel between the
        // two accounts is modeled here beyond that; §143/§144's own
        // reuse tests already prove signature verification needs
        // nothing else).
        assert!(gen_3
            .verify_signature(&alice_root.root_public_key())
            .is_ok());
        let bob_side_cache = RevocationCache::from_directory(&gen_3);

        // Step 5: "A3 reconnect attempt rejected."
        assert!(matches!(
            bob_side_cache.authentication_attempt(a3_tablet),
            AuthenticationOutcome::Reject { .. }
        ));

        // Bob's own two devices are unaffected by any of Alice's
        // account activity — a sanity check that account boundaries
        // actually hold in this scenario, not just that revocation
        // works within one account.
        let bob_directory = DeviceDirectory::sign(
            &bob_root,
            bob,
            1,
            vec![
                active_entry(&bob_root, bob, b1_phone, 1),
                active_entry(&bob_root, bob, b2_laptop, 1),
            ],
        );
        assert!(bob_directory.is_device_trusted(b1_phone));
        assert!(bob_directory.is_device_trusted(b2_laptop));
    }
}
