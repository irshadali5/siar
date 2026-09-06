//! §136 "State Machine for Recovery": spec's own six-state chain,
//! verbatim, with no failure states — unlike §135's linking machine,
//! spec names none here. That's not an oversight this module papers
//! over: [`crate::recovery::add_device_via_recovery`]'s own internal
//! `recovery_evidence_satisfies_policy` check already rejects bad
//! evidence via [`crate::recovery::RecoveryError`] BEFORE this state
//! machine would ever be entered (see
//! [`crate::client_api::RecoveryClient::begin_recovery`]'s own doc
//! comment) — so there is no state within this six-step chain where
//! failure is spec's concern, and adding one here would invent detail
//! spec doesn't ask for.

use serde::{Deserialize, Serialize};

/// Spec's six states, in order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryState {
    Started,
    RecoveryMaterialVerified,
    AuthorityRestored,
    NewDeviceCreated,
    OldDevicesReviewed,
    RevocationsCommitted,
    Completed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("cannot advance recovery state from {from:?} to {to:?}")]
pub struct InvalidRecoveryTransition {
    pub from: RecoveryState,
    pub to: RecoveryState,
}

impl RecoveryState {
    pub fn new() -> Self {
        RecoveryState::Started
    }

    /// Strictly linear — each state advances to exactly the next one
    /// in spec's own list, nothing more. §136: "persist progress
    /// safely where appropriate" is why this returns a plain value a
    /// caller can persist after each step, rather than this module
    /// doing any persistence itself (same "decide, don't dial" split
    /// as everywhere else in this crate).
    pub fn advance(self, to: RecoveryState) -> Result<RecoveryState, InvalidRecoveryTransition> {
        use RecoveryState::*;
        let valid = matches!(
            (self, to),
            (Started, RecoveryMaterialVerified)
                | (RecoveryMaterialVerified, AuthorityRestored)
                | (AuthorityRestored, NewDeviceCreated)
                | (NewDeviceCreated, OldDevicesReviewed)
                | (OldDevicesReviewed, RevocationsCommitted)
                | (RevocationsCommitted, Completed)
        );
        if valid {
            Ok(to)
        } else {
            Err(InvalidRecoveryTransition { from: self, to })
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, RecoveryState::Completed)
    }
}

impl Default for RecoveryState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn walks_all_six_transitions_in_order() {
        let state = RecoveryState::new()
            .advance(RecoveryState::RecoveryMaterialVerified)
            .unwrap()
            .advance(RecoveryState::AuthorityRestored)
            .unwrap()
            .advance(RecoveryState::NewDeviceCreated)
            .unwrap()
            .advance(RecoveryState::OldDevicesReviewed)
            .unwrap()
            .advance(RecoveryState::RevocationsCommitted)
            .unwrap()
            .advance(RecoveryState::Completed)
            .unwrap();
        assert_eq!(state, RecoveryState::Completed);
        assert!(state.is_terminal());
    }

    #[test]
    fn cannot_skip_a_step() {
        assert!(RecoveryState::Started
            .advance(RecoveryState::AuthorityRestored)
            .is_err());
    }

    #[test]
    fn cannot_go_backwards() {
        let restored = RecoveryState::Started
            .advance(RecoveryState::RecoveryMaterialVerified)
            .unwrap()
            .advance(RecoveryState::AuthorityRestored)
            .unwrap();
        assert!(restored
            .advance(RecoveryState::RecoveryMaterialVerified)
            .is_err());
    }

    #[test]
    fn cannot_advance_out_of_completed() {
        let mut state = RecoveryState::new();
        for next in [
            RecoveryState::RecoveryMaterialVerified,
            RecoveryState::AuthorityRestored,
            RecoveryState::NewDeviceCreated,
            RecoveryState::OldDevicesReviewed,
            RecoveryState::RevocationsCommitted,
            RecoveryState::Completed,
        ] {
            state = state.advance(next).unwrap();
        }
        assert!(state.advance(RecoveryState::Started).is_err());
    }
}
