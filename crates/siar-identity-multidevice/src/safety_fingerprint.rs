//! Safety fingerprint (Part 28 §43).
//!
//! Deliberately distinct from `verification_code.rs`'s SAS: that code
//! is short (6 digits), tied to one ephemeral linking *session*
//! (derived from `DeviceLinkInvite` + ephemeral handshake keys + a
//! fresh shared secret), and meant to be compared once, in person,
//! during that one linking attempt. §43 asks for something else — "an
//! advanced verification fingerprint/safety number for high-assurance
//! contacts" — which needs to be: long-lived (comparable weeks or
//! months after pairing, not just during a handshake), stable (the
//! same two accounts get the same fingerprint every time either side
//! recomputes it, not a fresh value per session), and tied to the
//! parties' actual long-term identity (`RootPublicKey`), not to an
//! ephemeral, single-use handshake.
//!
//! Modeled on the same shape Signal's own "safety number" uses: a
//! value derived from *both* participants' long-term public identity
//! keys, combined in a canonical (sort-then-concatenate) order so both
//! sides compute the identical fingerprint regardless of who's "local"
//! and who's "remote" from their own point of view.

use serde::{Deserialize, Serialize};

use crate::root_key::RootPublicKey;

const DIGIT_GROUPS: usize = 12;
const DIGITS_PER_GROUP: usize = 5;

/// A stable, long-form, human-comparable fingerprint for one pair of
/// accounts' root identities. `SafetyFingerprint::derive`'s symmetric
/// ordering means `derive(a, b) == derive(b, a)` — either party
/// computes the same value, matching what §43 means by "provide a
/// fingerprint for high-assurance contacts" (one shared value both
/// sides read out and compare, not two different per-side values).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SafetyFingerprint {
    /// `DIGIT_GROUPS` groups of `DIGITS_PER_GROUP` decimal digits each
    /// (60 digits total) — long enough that an active attacker who
    /// doesn't hold both real root keys cannot feasibly find a second
    /// key pair producing a matching fingerprint (a 60-digit decimal
    /// fingerprint carries far more entropy than the 6-digit SAS is
    /// designed to, matching this type's long-lived-verification role
    /// rather than the SAS's single-session one), while still being
    /// something two people can read aloud and compare a few digits at
    /// a time rather than a raw hex blob.
    groups: [[u8; DIGITS_PER_GROUP]; DIGIT_GROUPS],
}

impl SafetyFingerprint {
    /// Derives the fingerprint for the pair `(a, b)`. Order of
    /// arguments doesn't matter — the two 32-byte public keys are
    /// sorted before hashing specifically so that whichever side calls
    /// this with (their own key, the contact's key) or (the contact's
    /// key, their own key) always gets the same result.
    pub fn derive(a: &RootPublicKey, b: &RootPublicKey) -> Self {
        let (first, second) = if a.0 <= b.0 { (a, b) } else { (b, a) };

        let mut hasher = blake3::Hasher::new();
        hasher.update(b"siar-crypto/safety-fingerprint/v1");
        hasher.update(&first.0);
        hasher.update(&second.0);
        let digest = hasher.finalize();
        let digest_bytes = digest.as_bytes();

        // Expand the 32-byte digest into DIGIT_GROUPS*DIGITS_PER_GROUP
        // (60) decimal digits via a second, distinctly-tagged hash per
        // group rather than trying to squeeze 60 digits out of 32
        // bytes directly (32 bytes is only ~77 bits under a naive
        // byte-to-digit mapping, well short of 60 honest decimal
        // digits' worth of entropy — but the actual security bound is
        // still the original 32-byte/256-bit digest; each group hash
        // is a deterministic *expansion* of that one value, not a
        // fresh independent source of randomness).
        let mut groups = [[0u8; DIGITS_PER_GROUP]; DIGIT_GROUPS];
        for (i, group) in groups.iter_mut().enumerate() {
            let mut group_hasher = blake3::Hasher::new();
            group_hasher.update(digest_bytes);
            group_hasher.update(&(i as u32).to_be_bytes());
            let group_digest = group_hasher.finalize();
            let value = u32::from_be_bytes([
                group_digest.as_bytes()[0],
                group_digest.as_bytes()[1],
                group_digest.as_bytes()[2],
                group_digest.as_bytes()[3],
            ]) % 100_000; // DIGITS_PER_GROUP = 5 decimal digits

            let mut v = value;
            for d in group.iter_mut().rev() {
                *d = (v % 10) as u8;
                v /= 10;
            }
        }

        Self { groups }
    }

    /// Formats as space-separated 5-digit groups (`"12345 67890 ..."`,
    /// 12 groups) — a display shape a person can read a chunk at a
    /// time and compare against a contact's screen, matching how
    /// Signal/Matrix-style safety numbers are conventionally shown.
    pub fn display_string(&self) -> String {
        self.groups
            .iter()
            .map(|group| {
                group
                    .iter()
                    .map(|d| char::from(b'0' + d))
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::root_key::RootIdentityKey;

    #[test]
    fn is_symmetric_regardless_of_argument_order() {
        let alice = RootIdentityKey::generate();
        let bob = RootIdentityKey::generate();
        let fp_ab = SafetyFingerprint::derive(&alice.root_public_key(), &bob.root_public_key());
        let fp_ba = SafetyFingerprint::derive(&bob.root_public_key(), &alice.root_public_key());
        assert_eq!(fp_ab, fp_ba);
    }

    #[test]
    fn is_deterministic() {
        let alice = RootIdentityKey::generate();
        let bob = RootIdentityKey::generate();
        let first = SafetyFingerprint::derive(&alice.root_public_key(), &bob.root_public_key());
        let second = SafetyFingerprint::derive(&alice.root_public_key(), &bob.root_public_key());
        assert_eq!(first, second);
    }

    #[test]
    fn different_pairs_get_different_fingerprints() {
        let alice = RootIdentityKey::generate();
        let bob = RootIdentityKey::generate();
        let carol = RootIdentityKey::generate();
        let fp_alice_bob =
            SafetyFingerprint::derive(&alice.root_public_key(), &bob.root_public_key());
        let fp_alice_carol =
            SafetyFingerprint::derive(&alice.root_public_key(), &carol.root_public_key());
        assert_ne!(fp_alice_bob, fp_alice_carol);
    }

    #[test]
    fn display_string_is_twelve_groups_of_five_digits() {
        let alice = RootIdentityKey::generate();
        let bob = RootIdentityKey::generate();
        let fp = SafetyFingerprint::derive(&alice.root_public_key(), &bob.root_public_key());
        let display = fp.display_string();
        let groups: Vec<&str> = display.split(' ').collect();
        assert_eq!(groups.len(), DIGIT_GROUPS);
        for group in groups {
            assert_eq!(group.len(), DIGITS_PER_GROUP);
            assert!(group.chars().all(|c| c.is_ascii_digit()));
        }
    }

    #[test]
    fn a_changed_root_key_changes_the_fingerprint() {
        // Directly exercises §42's own premise: "a root/account identity
        // change should trigger a strong warning" — this confirms the
        // fingerprint a UI would display actually changes when the
        // underlying root key does, which is what makes that warning
        // meaningful rather than cosmetic.
        let alice = RootIdentityKey::generate();
        let bob = RootIdentityKey::generate();
        let bob_new = RootIdentityKey::generate();
        let fp_before = SafetyFingerprint::derive(&alice.root_public_key(), &bob.root_public_key());
        let fp_after =
            SafetyFingerprint::derive(&alice.root_public_key(), &bob_new.root_public_key());
        assert_ne!(fp_before, fp_after);
    }
}
