//! Violation classification and forward-compatibility policy — spec
//! §28 "Protocol Violation Classification", §29 "Unknown Extensions",
//! §30 "Unknown Capabilities", §31 "Operation-Level Required
//! Capabilities".

use crate::capability::{CapabilityId, CapabilitySet};
use crate::framing::FramingError;
use crate::lifecycle::ExtensionError;

/// spec §28, verbatim four-way classification, plus its own worked
/// examples (kept as this module's tests rather than restated only in
/// prose): "unknown optional frame → Recoverable," "oversized
/// malicious frame → Peer-abusive," "authentication failure →
/// Session-fatal," "invalid file state → Extension-fatal."
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ViolationClass {
    Recoverable,
    ExtensionFatal,
    SessionFatal,
    PeerAbusive,
}

/// Classifies a [`FramingError`] per spec §28's own two framing-level
/// examples. `FrameTooLarge` and `FrameLengthImpossiblySmall` are both
/// internally-inconsistent claims an honest, correctly-implemented
/// peer cannot produce (a declared length either exceeds the
/// negotiated `max_frame_size` or is smaller than the header format
/// itself) — spec §28's "oversized malicious frame → Peer-abusive"
/// example, generalized to both. `HeaderTooShort` is treated
/// differently on purpose: an honest peer can produce a short read
/// under ordinary conditions (a slow/fragmenting transport, a stream
/// cut off mid-header), so this one is `Recoverable` — the read can
/// simply be retried — rather than assumed malicious.
pub fn classify_framing_error(error: &FramingError) -> ViolationClass {
    match error {
        FramingError::FrameTooLarge { .. } | FramingError::FrameLengthImpossiblySmall { .. } => {
            ViolationClass::PeerAbusive
        }
        FramingError::HeaderTooShort { .. } => ViolationClass::Recoverable,
    }
}

/// Classifies an [`ExtensionError`] per spec §28's other two examples:
/// "authentication failure → Session-fatal" maps directly onto
/// `Unauthorized`. "invalid file state → Extension-fatal" is a
/// correctness problem scoped to one extension's own state machine —
/// `ProtocolViolation`, `CapabilityMismatch`, `ResourceLimit`,
/// `VersionMismatch`, `Unsupported`, and `StorageFailure` all share
/// that shape (the extension itself cannot continue, but nothing about
/// the rest of the session is implicated), so all classify as
/// `ExtensionFatal`. `Internal` is deliberately the one exception:
/// spec §27's own "internal debug strings must not become protocol
/// semantics" already treats it as not a peer-facing classification at
/// all — kept `ExtensionFatal` here too (the safe, non-escalating
/// default) rather than invented a fifth class this spec never named.
pub fn classify_extension_error(error: &ExtensionError) -> ViolationClass {
    match error {
        ExtensionError::Unauthorized => ViolationClass::SessionFatal,
        ExtensionError::ProtocolViolation
        | ExtensionError::CapabilityMismatch
        | ExtensionError::ResourceLimit
        | ExtensionError::VersionMismatch
        | ExtensionError::Unsupported
        | ExtensionError::StorageFailure
        | ExtensionError::Internal => ViolationClass::ExtensionFatal,
    }
}

/// spec §29: "If a peer advertises an unknown optional extension:
/// ignore or mark unsupported. Do not fail the entire connection. This
/// is fundamental for forward compatibility." There is exactly one
/// correct classification for this case — always `Recoverable` — which
/// is what makes it worth a named function rather than leaving callers
/// to reason it out themselves at each call site: an unknown extension
/// is not this crate's [`crate::negotiation::negotiate`] failing to
/// find a match (that's already handled — see
/// [`crate::negotiation::NegotiationError`]), it's a peer advertising
/// something *this* side has never heard of, which by definition
/// cannot be a protocol violation on either side.
pub fn unknown_extension_policy() -> ViolationClass {
    ViolationClass::Recoverable
}

/// spec §31 "Operation-Level Required Capabilities": "An operation may
/// require a specific capability... If the peer lacks it: do not send
/// unsupported wire operation." Checked against what
/// [`crate::negotiation::negotiate`] actually produced for this
/// session — not the locally-*advertised* capability set, since only
/// the negotiated intersection reflects what the peer actually shares.
pub fn operation_supported(negotiated: &CapabilitySet, required: CapabilityId) -> bool {
    negotiated.contains(required)
}

/// spec §31's own three listed application responses to an
/// unsupported operation — kept as a real enum (not just prose) so a
/// caller of [`operation_supported`] has a documented, closed set of
/// valid reactions to choose from rather than inventing a fourth.
/// Which one applies is an application/UI decision this crate doesn't
/// make on the caller's behalf.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnsupportedOperationResponse {
    DisableAction,
    SendAlternative,
    ShowUnsupported,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_28_oversized_malicious_frame_is_peer_abusive() {
        let error = FramingError::FrameTooLarge {
            declared: 999_999,
            max: 1024,
        };
        assert_eq!(classify_framing_error(&error), ViolationClass::PeerAbusive);
    }

    #[test]
    fn spec_28_impossible_frame_length_is_also_peer_abusive() {
        let error = FramingError::FrameLengthImpossiblySmall { declared: 2 };
        assert_eq!(classify_framing_error(&error), ViolationClass::PeerAbusive);
    }

    #[test]
    fn spec_28_short_header_read_is_recoverable_not_abusive() {
        let error = FramingError::HeaderTooShort { actual: 3 };
        assert_eq!(classify_framing_error(&error), ViolationClass::Recoverable);
    }

    #[test]
    fn spec_28_authentication_failure_is_session_fatal() {
        assert_eq!(
            classify_extension_error(&ExtensionError::Unauthorized),
            ViolationClass::SessionFatal
        );
    }

    #[test]
    fn spec_28_invalid_extension_state_is_extension_fatal() {
        // "invalid file state" in spec §28's own example — the closest
        // ExtensionError variant to a violated invariant within one
        // extension's own state machine.
        assert_eq!(
            classify_extension_error(&ExtensionError::ProtocolViolation),
            ViolationClass::ExtensionFatal
        );
    }

    #[test]
    fn spec_29_unknown_extension_is_always_recoverable() {
        assert_eq!(unknown_extension_policy(), ViolationClass::Recoverable);
    }

    #[test]
    fn spec_31_operation_requiring_a_negotiated_capability_is_supported() {
        let negotiated = CapabilitySet::new([CapabilityId(1), CapabilityId(2)]);
        assert!(operation_supported(&negotiated, CapabilityId(2)));
    }

    #[test]
    fn spec_31_operation_requiring_a_capability_the_peer_lacks_is_unsupported() {
        // messaging.edit example from spec §31, id 3 stands in for it —
        // negotiated only has [1, 2], so EditMessage must not be sent.
        let negotiated = CapabilitySet::new([CapabilityId(1), CapabilityId(2)]);
        assert!(!operation_supported(&negotiated, CapabilityId(3)));
    }
}
