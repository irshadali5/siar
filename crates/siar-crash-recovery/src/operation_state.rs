//! §13 "Durable Operation State".

use serde::{Deserialize, Serialize};

/// §13's own worked example, verbatim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FileOperationState {
    Created,
    Preparing,
    Transferring,
    Finalizing,
    Completed,
}

/// Real transition validation for §13's example chain — linear, no
/// skipping, same discipline as this crate's other state machines.
pub fn can_transition(from: FileOperationState, to: FileOperationState) -> bool {
    use FileOperationState::*;
    matches!(
        (from, to),
        (Created, Preparing)
            | (Preparing, Transferring)
            | (Transferring, Finalizing)
            | (Finalizing, Completed)
    )
}

/// §13's own explicit warning, made structural rather than left as
/// prose a caller could still get wrong: "Never rely on: absence of
/// error to infer completion." A caller checking "did this operation
/// finish?" against a store that might not have any record at all
/// (crash before the very first state was ever persisted, or a lookup
/// that simply found nothing) must treat that absence as "not
/// complete," never as "must have succeeded" — this function is that
/// one-line rule made into code a caller can call instead of
/// re-deriving the same reasoning ad hoc at every call site.
pub fn is_definitely_complete(state: Option<FileOperationState>) -> bool {
    matches!(state, Some(FileOperationState::Completed))
}

#[cfg(test)]
mod tests {
    use super::*;
    use FileOperationState::*;

    #[test]
    fn chain_transitions_are_linear() {
        for (from, to) in [
            (Created, Preparing),
            (Preparing, Transferring),
            (Transferring, Finalizing),
            (Finalizing, Completed),
        ] {
            assert!(can_transition(from, to));
        }
        assert!(!can_transition(Created, Completed));
        assert!(!can_transition(Completed, Created));
    }

    #[test]
    fn no_recorded_state_is_never_treated_as_complete() {
        // §13's exact warning: absence of a record (or of an error)
        // must never be read as "it must have finished."
        assert!(!is_definitely_complete(None));
    }

    #[test]
    fn an_in_progress_state_is_not_complete() {
        assert!(!is_definitely_complete(Some(Transferring)));
        assert!(!is_definitely_complete(Some(Finalizing)));
    }

    #[test]
    fn only_the_explicit_completed_state_counts_as_complete() {
        assert!(is_definitely_complete(Some(Completed)));
    }
}
