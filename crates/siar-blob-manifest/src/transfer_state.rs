//! §26 "Transfer State Machine".

/// §26's own named states — the spec's section shows the states
/// conceptually rather than an exhaustive transition table, so
/// [`TransferState::transition`]'s specific edges below are this
/// crate's own reasonable reading of the obvious lifecycle (offer →
/// accept → transfer → complete, with pause/cancel/fail available from
/// the right states), not a transcription of an explicit table in the
/// source document.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferState {
    Offered,
    Accepted,
    InProgress,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferEvent {
    Accept,
    Decline,
    Start,
    Pause,
    Resume,
    ChunksComplete,
    Fail,
    Cancel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("{event:?} is not a valid transition from {state:?}")]
pub struct InvalidTransition {
    pub state: TransferState,
    pub event: TransferEvent,
}

impl TransferState {
    /// A real state machine — most `(state, event)` pairs are rejected,
    /// not silently accepted. `Failed`/`Cancelled` are terminal:
    /// nothing transitions out of them here, matching an immutable
    /// event log's own append-only spirit (Part 04 §10) applied to
    /// transfer lifecycle instead of event storage.
    pub fn transition(self, event: TransferEvent) -> Result<TransferState, InvalidTransition> {
        use TransferEvent as E;
        use TransferState as S;
        let next = match (self, event) {
            (S::Offered, E::Accept) => S::Accepted,
            (S::Offered, E::Decline) => S::Cancelled,
            (S::Offered, E::Cancel) => S::Cancelled,
            (S::Accepted, E::Start) => S::InProgress,
            (S::Accepted, E::Cancel) => S::Cancelled,
            (S::InProgress, E::Pause) => S::Paused,
            (S::InProgress, E::ChunksComplete) => S::Completed,
            (S::InProgress, E::Fail) => S::Failed,
            (S::InProgress, E::Cancel) => S::Cancelled,
            (S::Paused, E::Resume) => S::InProgress,
            (S::Paused, E::Cancel) => S::Cancelled,
            (S::Paused, E::Fail) => S::Failed,
            _ => return Err(InvalidTransition { state: self, event }),
        };
        Ok(next)
    }

    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_normal_transfer_walks_offer_through_completion() {
        let s = TransferState::Offered;
        let s = s.transition(TransferEvent::Accept).unwrap();
        assert_eq!(s, TransferState::Accepted);
        let s = s.transition(TransferEvent::Start).unwrap();
        assert_eq!(s, TransferState::InProgress);
        let s = s.transition(TransferEvent::ChunksComplete).unwrap();
        assert_eq!(s, TransferState::Completed);
        assert!(s.is_terminal());
    }

    #[test]
    fn pause_and_resume_returns_to_in_progress() {
        let s = TransferState::InProgress;
        let s = s.transition(TransferEvent::Pause).unwrap();
        assert_eq!(s, TransferState::Paused);
        let s = s.transition(TransferEvent::Resume).unwrap();
        assert_eq!(s, TransferState::InProgress);
    }

    #[test]
    fn completed_is_terminal_and_rejects_further_events() {
        let s = TransferState::Completed;
        assert!(s.is_terminal());
        assert!(s.transition(TransferEvent::Cancel).is_err());
        assert!(s.transition(TransferEvent::Pause).is_err());
    }

    #[test]
    fn starting_a_transfer_that_was_never_accepted_is_rejected() {
        let s = TransferState::Offered;
        let result = s.transition(TransferEvent::Start);
        assert_eq!(
            result,
            Err(InvalidTransition {
                state: TransferState::Offered,
                event: TransferEvent::Start
            })
        );
    }
}
