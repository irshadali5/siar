//! §176 "Call Integration", §177 "Call Ring Arbitration": "identity
//! provides the device list; call protocol handles arbitration" —
//! [`crate::routing_integration::resolve_account_endpoints`] is the
//! device list half; [`CallRingState`] below is the arbitration half,
//! kept in this crate only because §177 asks for the specific
//! deterministic states, not because call signaling itself belongs
//! here (it doesn't — see [`crate::device_classes`]/[`crate::destination`]
//! for the same "identity provides inputs, another layer does the
//! actual work" split elsewhere in this crate).

use siar_domain::DeviceId;

/// §177's own four named states, verbatim.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallRingState {
    Ringing,
    AcceptedBy(DeviceId),
    Cancelled,
    Expired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("cannot advance call ring state from {from:?} to {to:?}")]
pub struct InvalidCallRingTransition {
    pub from: CallRingState,
    pub to: CallRingState,
}

impl CallRingState {
    pub fn new() -> Self {
        CallRingState::Ringing
    }

    /// §177's own point, made structural: only `Ringing` can transition
    /// anywhere — `AcceptedBy`/`Cancelled`/`Expired` are all terminal.
    /// "First device to accept can establish call ownership" is why
    /// there is no `AcceptedBy(a) → AcceptedBy(b)` transition at all:
    /// once one device has answered, the state machine itself refuses
    /// a second acceptance, which is exactly "avoids simultaneous
    /// multi-device answers" — not a race a caller has to resolve with
    /// its own locking, but something this type cannot represent in
    /// the first place.
    pub fn advance(self, to: CallRingState) -> Result<CallRingState, InvalidCallRingTransition> {
        match self {
            CallRingState::Ringing => Ok(to),
            _ => Err(InvalidCallRingTransition { from: self, to }),
        }
    }

    /// §176: "other devices receive: call answered elsewhere." A
    /// device checks this against its own id to decide which message
    /// to show — `false` for the accepting device's own id means it
    /// gets no such notification about itself.
    pub fn answered_elsewhere_for(&self, this_device: DeviceId) -> bool {
        matches!(self, CallRingState::AcceptedBy(accepted) if *accepted != this_device)
    }

    pub fn is_terminal(&self) -> bool {
        !matches!(self, CallRingState::Ringing)
    }
}

impl Default for CallRingState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_one_device_can_accept_a_ringing_call() {
        let device_a = DeviceId::new();
        let device_b = DeviceId::new();
        let accepted = CallRingState::new()
            .advance(CallRingState::AcceptedBy(device_a))
            .unwrap();

        // A second device's acceptance, arriving after the first, is
        // rejected — the state machine itself, not a lock a caller has
        // to remember to take.
        let second_attempt = accepted.advance(CallRingState::AcceptedBy(device_b));
        assert!(second_attempt.is_err());
    }

    #[test]
    fn other_devices_see_answered_elsewhere_but_the_accepting_device_does_not() {
        let device_a = DeviceId::new();
        let device_b = DeviceId::new();
        let accepted = CallRingState::new()
            .advance(CallRingState::AcceptedBy(device_a))
            .unwrap();

        assert!(accepted.answered_elsewhere_for(device_b));
        assert!(!accepted.answered_elsewhere_for(device_a));
    }

    #[test]
    fn cancelled_and_expired_are_both_terminal() {
        let cancelled = CallRingState::new()
            .advance(CallRingState::Cancelled)
            .unwrap();
        let expired = CallRingState::new()
            .advance(CallRingState::Expired)
            .unwrap();
        assert!(cancelled.is_terminal());
        assert!(expired.is_terminal());
        assert!(cancelled.advance(CallRingState::Ringing).is_err());
        assert!(expired.advance(CallRingState::Ringing).is_err());
    }
}
