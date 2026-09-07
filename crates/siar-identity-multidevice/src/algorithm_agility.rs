//! §192 "Algorithm Agility", §193 "Algorithm Downgrade Protection".
//!
//! §192's own caution — "avoid needless generic abstraction in the
//! first implementation" — is why [`AlgorithmId`] names only the three
//! algorithms this crate actually uses today (Ed25519 signing,
//! X25519 key agreement, BLAKE3 hashing), not a speculative catalog of
//! algorithms nothing here implements. This crate currently hard-codes
//! exactly one algorithm per purpose everywhere — there is no
//! negotiation happening anywhere in this codebase today, so §193's
//! downgrade attack has no actual surface to exploit yet.
//! [`negotiate_algorithm`] exists as the seam a future real
//! negotiation would use, tested on its own terms, not wired into any
//! live handshake — matching this crate's established pattern of
//! providing a real boundary function ahead of the mechanism that will
//! eventually call it (same posture as
//! [`crate::storage::IdentityStore`]/[`crate::client_api`]'s traits).

/// §192's own three named examples. Paired with
/// [`crate::wire_limits::SchemaVersion`] for the same "future migration
/// matters" reasoning — a `SchemaVersion` says which wire shape a
/// message uses; an `AlgorithmId` would say which cryptographic
/// primitive backs a specific key/signature/hash value within it. Not
/// used anywhere in this crate's actual signing/verification code
/// today — those functions still call `ed25519_dalek`/`x25519_dalek`/
/// `blake3` directly, per §192's own "avoid needless generic
/// abstraction" — this is a label for describing an algorithm choice
/// out-of-band (e.g. in a future backup format version, or a future
/// negotiated handshake), not a trait object or dynamic dispatch layer
/// this crate doesn't need yet.
///
/// Deliberately no [`Ord`] — Ed25519 (signing), X25519 (key
/// agreement), and Blake3 (hashing) solve different problems, so
/// "stronger than" is meaningless across them; a derived declaration-
/// order `Ord` would silently imply a comparison that doesn't exist.
/// [`negotiate_algorithm`] instead trusts the CALLER's own
/// `locally_allowed` ordering (strongest-preference-first, by
/// convention) rather than inventing a ranking here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AlgorithmId {
    Ed25519,
    X25519,
    Blake3,
}

/// §193's own formula, verbatim: "mutually supported ∩ locally
/// allowed," with an additional floor — nothing before
/// `minimum_required`'s position in `locally_allowed` is ever
/// returned, so a peer cannot force a downgrade by simply not listing
/// anything stronger. Returns the FIRST entry in `locally_allowed`
/// (read as caller-supplied, strongest-preference-first, per this
/// module's own note on why [`AlgorithmId`] has no built-in
/// [`Ord`]) that both `peer_supported` contains and that appears at or
/// before `minimum_required` in that same preference list — never the
/// first match overall, since a peer offering both a weak and a
/// strong mutually-allowed option must not get the weak one picked by
/// accident.
pub fn negotiate_algorithm(
    locally_allowed: &[AlgorithmId],
    peer_supported: &[AlgorithmId],
    minimum_required: AlgorithmId,
) -> Option<AlgorithmId> {
    let minimum_position = locally_allowed
        .iter()
        .position(|a| *a == minimum_required)?;
    locally_allowed[..=minimum_position]
        .iter()
        .find(|candidate| peer_supported.contains(candidate))
        .copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn negotiation_never_returns_anything_weaker_than_the_minimum() {
        // locally_allowed lists X25519 as strictly preferred over
        // Ed25519 for this (hypothetical) purpose; the peer only
        // supports the weaker, later-listed option.
        let result = negotiate_algorithm(
            &[AlgorithmId::X25519, AlgorithmId::Ed25519],
            &[AlgorithmId::Ed25519],
            AlgorithmId::X25519,
        );
        assert_eq!(
            result, None,
            "the only mutually supported option is weaker than the required minimum"
        );
    }

    #[test]
    fn negotiation_picks_the_most_preferred_mutually_supported_option() {
        let result = negotiate_algorithm(
            &[
                AlgorithmId::X25519,
                AlgorithmId::Ed25519,
                AlgorithmId::Blake3,
            ],
            &[AlgorithmId::Ed25519, AlgorithmId::Blake3],
            AlgorithmId::Blake3,
        );
        assert_eq!(
            result,
            Some(AlgorithmId::Ed25519),
            "Ed25519 is more preferred than Blake3 in the locally_allowed order, and both are mutually supported and at/above the minimum"
        );
    }

    #[test]
    fn a_peer_offering_nothing_locally_allowed_gets_no_algorithm() {
        let result = negotiate_algorithm(
            &[AlgorithmId::Ed25519],
            &[AlgorithmId::X25519],
            AlgorithmId::Ed25519,
        );
        assert_eq!(result, None);
    }

    #[test]
    fn minimum_required_not_present_in_locally_allowed_yields_no_algorithm() {
        let result = negotiate_algorithm(
            &[AlgorithmId::Ed25519],
            &[AlgorithmId::Ed25519],
            AlgorithmId::X25519,
        );
        assert_eq!(result, None);
    }
}
