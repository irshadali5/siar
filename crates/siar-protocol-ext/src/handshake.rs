//! spec §78 "Core Handshake State Machine", §79 "Reconnection".

/// spec §78's own eight-stage diagram, plus a `Failed` terminal state
/// for "authentication failure stops application-level use" —
/// reachable from any pre-`SessionEstablished` stage, since an auth
/// failure can in principle occur at identity binding or later, and
/// once reached nothing else in this enum is a valid next state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum HandshakeStage {
    TransportConnected,
    CoreHello,
    IdentityBinding,
    CoreVersionAgreement,
    ExtensionAdvertisement,
    CapabilityNegotiation,
    SessionEstablished,
    LazyExtensionOpens,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("cannot advance handshake from {from:?} to {to:?}")]
pub struct InvalidHandshakeTransition {
    pub from: HandshakeStage,
    pub to: HandshakeStage,
}

impl HandshakeStage {
    /// spec §78's diagram, strictly linear for the success path, plus
    /// `Failed` reachable from every stage before `SessionEstablished`
    /// — matching "authentication failure stops application-level
    /// use," which can only meaningfully happen before a session
    /// actually exists.
    pub fn advance(self, to: HandshakeStage) -> Result<HandshakeStage, InvalidHandshakeTransition> {
        use HandshakeStage::*;
        let valid = matches!(
            (self, to),
            (TransportConnected, CoreHello)
                | (CoreHello, IdentityBinding)
                | (IdentityBinding, CoreVersionAgreement)
                | (CoreVersionAgreement, ExtensionAdvertisement)
                | (ExtensionAdvertisement, CapabilityNegotiation)
                | (CapabilityNegotiation, SessionEstablished)
                | (SessionEstablished, LazyExtensionOpens)
        ) || (
            to == Failed
                && !matches!(self, SessionEstablished | LazyExtensionOpens | Failed)
        );
        if valid {
            Ok(to)
        } else {
            Err(InvalidHandshakeTransition { from: self, to })
        }
    }
}

/// spec §79's own three-item revalidation list, verbatim — all three
/// required, not a majority or a best-effort subset, matching §79's
/// own "never blindly trust stale session metadata."
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct ReconnectionRevalidation {
    pub identity_reverified: bool,
    pub protocol_compatibility_reverified: bool,
    pub capability_validity_reverified: bool,
}

impl ReconnectionRevalidation {
    /// The one thing this type exists to enforce: a cached negotiation
    /// from a prior session is only safe to reuse once ALL THREE checks
    /// have actually been redone — not assumed from the fact that a
    /// cache entry exists at all (that would be exactly the "blindly
    /// trust stale session metadata" spec §79 forbids).
    pub fn safe_to_reuse_cached_negotiation(&self) -> bool {
        self.identity_reverified
            && self.protocol_compatibility_reverified
            && self.capability_validity_reverified
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use HandshakeStage::*;

    #[test]
    fn spec_78_happy_path_advances_through_every_stage_in_order() {
        let stages = [
            TransportConnected,
            CoreHello,
            IdentityBinding,
            CoreVersionAgreement,
            ExtensionAdvertisement,
            CapabilityNegotiation,
            SessionEstablished,
            LazyExtensionOpens,
        ];
        let mut current = stages[0];
        for &next in &stages[1..] {
            current = current.advance(next).unwrap();
        }
        assert_eq!(current, LazyExtensionOpens);
    }

    #[test]
    fn spec_78_cannot_skip_stages() {
        assert!(TransportConnected.advance(SessionEstablished).is_err());
    }

    #[test]
    fn spec_78_authentication_failure_reachable_before_session_established() {
        assert!(IdentityBinding.advance(Failed).is_ok());
        assert!(CoreVersionAgreement.advance(Failed).is_ok());
    }

    #[test]
    fn spec_78_a_session_already_established_cannot_retroactively_fail_authentication() {
        // "Authentication failure stops application-level use" is
        // about gating entry into a session, not tearing one down —
        // an already-established session has its own separate teardown
        // path (see lifecycle.rs), not this one.
        assert!(SessionEstablished.advance(Failed).is_err());
        assert!(LazyExtensionOpens.advance(Failed).is_err());
    }

    #[test]
    fn spec_79_all_three_checks_are_required_not_a_majority() {
        let mut revalidation = ReconnectionRevalidation::default();
        assert!(!revalidation.safe_to_reuse_cached_negotiation());

        revalidation.identity_reverified = true;
        revalidation.protocol_compatibility_reverified = true;
        assert!(
            !revalidation.safe_to_reuse_cached_negotiation(),
            "two of three must not be enough"
        );

        revalidation.capability_validity_reverified = true;
        assert!(revalidation.safe_to_reuse_cached_negotiation());
    }
}
