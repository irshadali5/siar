//! §25 "Two-Phase Confirmation" and §26 "Negotiation Transcript Hash".
//!
//! §13 sketches `CapabilityAdvertisement` as a full authenticated,
//! session-bound wrapper around a peer's capabilities (device id,
//! account generation, session nonce, ...) — that type needs Part 02
//! session/identity wiring this crate doesn't have yet (documented as
//! a deferred gap in `lib.rs`). What this module builds instead is
//! the transcript-hashing primitive §26 actually describes
//! ("canonical encode: local advertisement, remote advertisement,
//! selected capabilities, session nonce... derive NegotiationHash"),
//! applied directly to the [`CapabilitySet`]s a caller already has —
//! a future `CapabilityAdvertisement` can supply its inner set to
//! this same function rather than this module needing to be rewritten
//! once that type exists.

use crate::hash::CapabilitySetHash;
use crate::set::CapabilitySet;
use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};
use std::fmt;

/// §13's `nonce: HandshakeNonce` field, and §15's freshness-binding
/// requirement ("Capability advertisements must be freshness-bound...
/// session nonce... short lifetime"). 16 bytes matches this
/// workspace's existing nonce sizing convention
/// (`siar_identity_multidevice::invite`'s `OsRng.fill_bytes` call
/// uses the same width for its own handshake nonce).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HandshakeNonce([u8; 16]);

impl HandshakeNonce {
    /// §15: the nonce must actually be fresh per session, not a fixed
    /// or caller-guessable value — generated from `OsRng`, matching
    /// how every other nonce/key in this workspace is sourced
    /// (`rand_core::OsRng`, never a seeded/deterministic RNG).
    pub fn generate() -> Self {
        let mut bytes = [0u8; 16];
        OsRng.fill_bytes(&mut bytes);
        Self(bytes)
    }

    pub fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }
}

/// §26: "derive: NegotiationHash. Both peers verify equality." A
/// distinct type from [`CapabilitySetHash`] even though both wrap a
/// 32-byte blake3 digest — a `NegotiationHash` commits to a whole
/// negotiation (both advertisements, the selection, and the session
/// nonce), while a `CapabilitySetHash` commits to one set alone; §94
/// lists the plain set hash's own uses (cache validation, delta
/// detection) as distinct from the transcript hash's use (session
/// confirmation), so collapsing them into one type would make it easy
/// to compare a value computed for one purpose against a value
/// computed for the other.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NegotiationHash([u8; 32]);

impl NegotiationHash {
    /// §26's canonical encode, computed so that it does not matter
    /// which peer calls it "local" and which calls it "remote": the
    /// two advertised sets' own [`CapabilitySetHash`]es are sorted
    /// before being folded in, the same way §72's negotiation
    /// determinism requires `negotiate()` to be symmetric under
    /// swapped inputs (see `negotiate.rs`'s own symmetry test) — a
    /// transcript hash that differed depending on which side computed
    /// it first would make "both peers verify equality" (§26)
    /// impossible to satisfy in the ordinary case where nothing has
    /// actually gone wrong.
    pub fn compute(
        peer_a_offered: &CapabilitySet,
        peer_b_offered: &CapabilitySet,
        selected: &CapabilitySet,
        nonce: HandshakeNonce,
    ) -> Self {
        let hash_a = CapabilitySetHash::of(peer_a_offered);
        let hash_b = CapabilitySetHash::of(peer_b_offered);
        let (first, second) = if hash_a.as_bytes() <= hash_b.as_bytes() {
            (hash_a, hash_b)
        } else {
            (hash_b, hash_a)
        };
        let selected_hash = CapabilitySetHash::of(selected);

        let mut hasher = blake3::Hasher::new();
        hasher.update(first.as_bytes());
        hasher.update(second.as_bytes());
        hasher.update(selected_hash.as_bytes());
        hasher.update(nonce.as_bytes());
        Self(*hasher.finalize().as_bytes())
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl fmt::Display for NegotiationHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

/// §25: "Both peers confirm the same negotiated capability set." —
/// the confirmation step itself, once each side has independently
/// computed its own [`NegotiationHash`] via [`NegotiationHash::compute`]
/// and exchanged it with the other.
pub fn confirm(
    local: NegotiationHash,
    remote_reported: NegotiationHash,
) -> Result<(), crate::error::CapabilityNegotiationError> {
    if local == remote_reported {
        Ok(())
    } else {
        Err(crate::error::CapabilityNegotiationError::TranscriptMismatch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::descriptor::{CapabilityDescriptor, CapabilityParameters, CapabilityRequirement};
    use crate::id::{CapabilityId, CapabilityNamespace};
    use crate::version::CapabilityVersion;

    fn set_with(code: u32) -> CapabilitySet {
        let mut set = CapabilitySet::new();
        set.insert(CapabilityDescriptor::new(
            CapabilityId::new(CapabilityNamespace::Core, code),
            CapabilityVersion::new(1, 0),
            CapabilityRequirement::Optional,
            CapabilityParameters::None,
        ))
        .unwrap();
        set
    }

    #[test]
    fn generate_produces_distinct_nonces() {
        let a = HandshakeNonce::generate();
        let b = HandshakeNonce::generate();
        assert_ne!(a, b);
    }

    #[test]
    fn transcript_hash_is_symmetric_regardless_of_which_side_is_local() {
        let a = set_with(1);
        let b = set_with(2);
        let selected = set_with(1);
        let nonce = HandshakeNonce::generate();

        let from_a = NegotiationHash::compute(&a, &b, &selected, nonce);
        let from_b = NegotiationHash::compute(&b, &a, &selected, nonce);
        assert_eq!(from_a, from_b);
    }

    #[test]
    fn transcript_hash_changes_with_nonce() {
        let a = set_with(1);
        let b = set_with(2);
        let selected = set_with(1);

        let h1 = NegotiationHash::compute(&a, &b, &selected, HandshakeNonce::from_bytes([1; 16]));
        let h2 = NegotiationHash::compute(&a, &b, &selected, HandshakeNonce::from_bytes([2; 16]));
        assert_ne!(h1, h2);
    }

    #[test]
    fn transcript_hash_changes_with_selected_set() {
        let a = set_with(1);
        let b = set_with(2);
        let nonce = HandshakeNonce::from_bytes([9; 16]);

        let h1 = NegotiationHash::compute(&a, &b, &set_with(1), nonce);
        let h2 = NegotiationHash::compute(&a, &b, &set_with(2), nonce);
        assert_ne!(h1, h2);
    }

    #[test]
    fn confirm_accepts_matching_and_rejects_mismatched_hashes() {
        let a = set_with(1);
        let b = set_with(2);
        let selected = set_with(1);
        let nonce = HandshakeNonce::generate();

        let mine = NegotiationHash::compute(&a, &b, &selected, nonce);
        let theirs_matching = NegotiationHash::compute(&b, &a, &selected, nonce);
        assert!(confirm(mine, theirs_matching).is_ok());

        let theirs_wrong = NegotiationHash::compute(&a, &b, &set_with(2), nonce);
        assert_eq!(
            confirm(mine, theirs_wrong),
            Err(crate::error::CapabilityNegotiationError::TranscriptMismatch)
        );
    }
}
