//! §135 "State Machine for Linking": spec's own eight-state success
//! chain plus four named failure states, verbatim.
//!
//! Same shape as [`crate::device_state::DeviceLifecycle::advance`] —
//! an enum plus a guarded `advance` method returning a typed error on
//! an invalid transition, rather than a type-state machine (unlike
//! [`crate::transaction`]'s device-addition sequence): linking has
//! real failure branches that can occur from several different
//! points, which a straight linear type-state chain doesn't represent
//! well, while `DeviceLifecycle`'s guarded-enum pattern already does.

use serde::{Deserialize, Serialize};

/// Spec's eight success states plus four failure states, in the order
/// spec lists them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinkingState {
    Created,
    InviteShared,
    PeerConnected,
    Verified,
    Approved,
    CertificateIssued,
    Committed,
    Completed,
    Expired,
    Rejected,
    VerificationFailed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("cannot advance linking state from {from:?} to {to:?}")]
pub struct InvalidLinkingTransition {
    pub from: LinkingState,
    pub to: LinkingState,
}

impl LinkingState {
    /// The starting state for a freshly created linking attempt.
    pub fn new() -> Self {
        LinkingState::Created
    }

    /// Transition table: the eight-step success chain in order, plus
    /// each failure state reachable only from the states where it
    /// plausibly occurs — `Expired` before a peer ever connects (an
    /// unused invite going stale, matching
    /// [`crate::invite::DeviceLinkInvite::is_expired`]'s own concern),
    /// `VerificationFailed` right after a peer connects (verification
    /// is the very next step), `Rejected` only once there's something
    /// to reject (after `Verified`), and `Cancelled` from any state up
    /// through `Approved` but never after — once a certificate is
    /// actually issued, undoing it is [`crate::revocation::revoke_device`]'s
    /// job, not a linking-flow cancellation.
    pub fn advance(self, to: LinkingState) -> Result<LinkingState, InvalidLinkingTransition> {
        use LinkingState::*;
        let valid = matches!(
            (self, to),
            (Created, InviteShared)
                | (InviteShared, PeerConnected)
                | (PeerConnected, Verified)
                | (Verified, Approved)
                | (Approved, CertificateIssued)
                | (CertificateIssued, Committed)
                | (Committed, Completed)
                | (Created, Expired)
                | (InviteShared, Expired)
                | (PeerConnected, VerificationFailed)
                | (Verified, Rejected)
                | (Created, Cancelled)
                | (InviteShared, Cancelled)
                | (PeerConnected, Cancelled)
                | (Verified, Cancelled)
                | (Approved, Cancelled)
        );
        if valid {
            Ok(to)
        } else {
            Err(InvalidLinkingTransition { from: self, to })
        }
    }

    /// Whether this state is one of §135's four named failure
    /// outcomes.
    pub fn is_failure(&self) -> bool {
        matches!(
            self,
            LinkingState::Expired
                | LinkingState::Rejected
                | LinkingState::VerificationFailed
                | LinkingState::Cancelled
        )
    }

    /// Terminal states — no `advance` call from any of these ever
    /// succeeds, success or failure alike.
    pub fn is_terminal(&self) -> bool {
        self.is_failure() || matches!(self, LinkingState::Completed)
    }
}

impl Default for LinkingState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn happy_path_walks_every_state_in_order() {
        let state = LinkingState::new()
            .advance(LinkingState::InviteShared)
            .unwrap()
            .advance(LinkingState::PeerConnected)
            .unwrap()
            .advance(LinkingState::Verified)
            .unwrap()
            .advance(LinkingState::Approved)
            .unwrap()
            .advance(LinkingState::CertificateIssued)
            .unwrap()
            .advance(LinkingState::Committed)
            .unwrap()
            .advance(LinkingState::Completed)
            .unwrap();
        assert_eq!(state, LinkingState::Completed);
        assert!(state.is_terminal());
        assert!(!state.is_failure());
    }

    #[test]
    fn cannot_skip_a_state() {
        let result = LinkingState::Created.advance(LinkingState::Verified);
        assert!(result.is_err());
    }

    #[test]
    fn cannot_advance_out_of_a_terminal_state() {
        let completed = LinkingState::Committed
            .advance(LinkingState::Completed)
            .unwrap();
        assert!(completed.advance(LinkingState::Cancelled).is_err());

        let expired = LinkingState::Created
            .advance(LinkingState::Expired)
            .unwrap();
        assert!(expired.advance(LinkingState::InviteShared).is_err());
    }

    #[test]
    fn cancellation_is_not_reachable_once_certificate_is_issued() {
        let issued = LinkingState::Created
            .advance(LinkingState::InviteShared)
            .unwrap()
            .advance(LinkingState::PeerConnected)
            .unwrap()
            .advance(LinkingState::Verified)
            .unwrap()
            .advance(LinkingState::Approved)
            .unwrap()
            .advance(LinkingState::CertificateIssued)
            .unwrap();
        assert!(issued.advance(LinkingState::Cancelled).is_err());
    }

    #[test]
    fn verification_failure_is_only_reachable_right_after_peer_connects() {
        assert!(LinkingState::PeerConnected
            .advance(LinkingState::VerificationFailed)
            .is_ok());
        assert!(LinkingState::Verified
            .advance(LinkingState::VerificationFailed)
            .is_err());
    }

    #[test]
    fn rejection_requires_having_reached_verified_first() {
        assert!(LinkingState::PeerConnected
            .advance(LinkingState::Rejected)
            .is_err());
        assert!(LinkingState::Verified
            .advance(LinkingState::Rejected)
            .is_ok());
    }
}
