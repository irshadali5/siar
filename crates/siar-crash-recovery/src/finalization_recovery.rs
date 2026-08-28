//! §21 "Finalization Recovery".
//!
//! §21 names three specific crash scenarios by prose rather than as a
//! decision table — this module turns that prose into an exhaustive
//! function, [`reconcile_finalization`], covering all five reachable
//! `(persisted state, does the final object actually exist)`
//! combinations, not only the three the spec calls out by name.

use serde::{Deserialize, Serialize};

/// §21's own pipeline, restricted to its durably-persisted stages
/// ("verify full blob/root" is a check performed *before* entering
/// [`FinalizationState::Renamed`], not a state of its own — nothing
/// about that check needs to survive a crash independently of the
/// `Finalizing` marker that precedes it).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FinalizationState {
    Finalizing,
    Renamed,
    Completed,
}

pub fn can_transition(from: FinalizationState, to: FinalizationState) -> bool {
    use FinalizationState::*;
    matches!((from, to), (Finalizing, Renamed) | (Renamed, Completed))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FinalizationRecoveryAction {
    /// §21's own first named scenario: "crash before rename." The
    /// atomic rename itself never started (or can't be assumed to
    /// have), so recovery re-runs finalization from the top.
    ResumeFinalization,
    /// §21's own second named scenario: "crash after rename, before
    /// Completed row: recovery detects final object exists, verifies
    /// it." The rename is durable at the filesystem level once it
    /// happens — recovery's job is only to confirm the object is
    /// intact and then persist `Completed`, not to redo the rename.
    VerifyThenComplete,
    /// §21's own third named scenario: "crash after Completed but
    /// object missing" — plus its one unnamed sibling, `Renamed`
    /// persisted but the object is missing. Both describe the same
    /// underlying problem (the persisted record claims a state that
    /// physical reality contradicts) that [`crate::staged_intent::reconcile`]
    /// already names `Inconsistent` for the general cross-store case —
    /// this variant is that same judgment, specific to finalization.
    FlagCorruption,
    NothingToDo,
}

/// §21's own "recovery checks intermediate state," made concrete:
/// given the last persisted [`FinalizationState`] and whether the
/// final object is actually present, decide what recovery does next.
pub fn reconcile_finalization(
    persisted: FinalizationState,
    final_object_exists: bool,
) -> FinalizationRecoveryAction {
    use FinalizationRecoveryAction::*;
    use FinalizationState::*;
    match (persisted, final_object_exists) {
        // Whether or not the object happens to exist here doesn't
        // change the answer: `Finalizing` only means finalization was
        // *attempted*, not that the rename definitely did or didn't
        // happen, so the safe response is always to resume/re-run
        // finalization from the top rather than trust either reading.
        (Finalizing, _) => ResumeFinalization,
        (Renamed, true) => VerifyThenComplete,
        (Renamed, false) => FlagCorruption,
        (Completed, true) => NothingToDo,
        (Completed, false) => FlagCorruption, // §21's exact named scenario
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use FinalizationRecoveryAction::*;
    use FinalizationState::*;

    #[test]
    fn pipeline_is_linear() {
        assert!(can_transition(Finalizing, Renamed));
        assert!(can_transition(Renamed, Completed));
        assert!(!can_transition(Finalizing, Completed));
    }

    #[test]
    fn crash_before_rename_resumes_finalization_regardless_of_object_presence() {
        // §21's first named scenario, both physical-reality readings.
        assert_eq!(
            reconcile_finalization(Finalizing, false),
            ResumeFinalization
        );
        assert_eq!(reconcile_finalization(Finalizing, true), ResumeFinalization);
    }

    #[test]
    fn crash_after_rename_before_completed_row_verifies_the_object() {
        // §21's second named scenario.
        assert_eq!(reconcile_finalization(Renamed, true), VerifyThenComplete);
    }

    #[test]
    fn renamed_but_object_missing_is_flagged_not_silently_resumed() {
        assert_eq!(reconcile_finalization(Renamed, false), FlagCorruption);
    }

    #[test]
    fn completed_and_object_present_needs_nothing_further() {
        assert_eq!(reconcile_finalization(Completed, true), NothingToDo);
    }

    #[test]
    fn completed_but_object_missing_is_21s_exact_third_named_scenario() {
        assert_eq!(reconcile_finalization(Completed, false), FlagCorruption);
    }
}
