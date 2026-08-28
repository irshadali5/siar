//! §11 "WAL Is Not Enough", §12 "Cross-Store Atomicity".
//!
//! §11's exact scenario — "database row says file exists but file
//! rename failed" — is a *disagreement* between persisted state and
//! physical reality that no database transaction can prevent, because
//! the filesystem (or the network, in other cross-store cases) isn't
//! part of that transaction. §12's own answer is a staged state
//! machine (`Prepare → Persist Intent → Perform Filesystem Action →
//! Persist Completion`) plus "recovery checks intermediate state" —
//! this module makes both halves real: [`StagedOperationState`] is
//! that staged machine (with real transition validation, the same
//! discipline `state_machine.rs` already established for the
//! top-level recovery flow), and [`reconcile`] is the actual
//! "recovery checks intermediate state" logic — given what's
//! persisted and what a caller-supplied check finds is *actually*
//! true on disk/network, it returns what recovery should do about the
//! disagreement, rather than leaving that as unstated prose.

use serde::{Deserialize, Serialize};

/// §12's own four-stage pipeline, minus "Prepare" itself — planning an
/// operation before anything is durably recorded has no state to be in
/// yet, so the first *persisted* stage is [`StagedOperationState::IntentPersisted`]
/// (§12's "Persist Intent"). The remaining two variants are §12's
/// "Perform Filesystem Action" and "Persist Completion" stages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StagedOperationState {
    IntentPersisted,
    ActionPerformed,
    Completed,
}

/// Real transition validation for §12's pipeline — linear, no skipping
/// a stage, the same discipline `state_machine::can_transition`
/// already applies to the top-level recovery flow.
pub fn can_transition(from: StagedOperationState, to: StagedOperationState) -> bool {
    use StagedOperationState::*;
    matches!(
        (from, to),
        (IntentPersisted, ActionPerformed) | (ActionPerformed, Completed)
    )
}

/// §12: "Recovery checks intermediate state" — turned into an actual
/// decision. `persisted` is what the durable record last said;
/// `action_confirmed` is what a caller-supplied real-world check
/// (e.g. "does the renamed file actually exist on disk?") just found.
/// The two can disagree in either direction after a crash, and each
/// disagreement needs a different response — this function is the
/// full 3×2 truth table, not just the two "obviously fine" cases.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReconciliationAction {
    /// Nothing was actually performed yet — safe to run the action
    /// from scratch.
    RerunAction,
    /// The action clearly did happen (crash occurred between
    /// performing it and persisting that fact) — just catch the
    /// record up, no need to redo the action itself.
    RecordActionPerformed,
    /// The action is confirmed done and there's nothing further to
    /// persist or do.
    NothingToDo,
    /// §11's exact named scenario: the persisted record and physical
    /// reality disagree in a way this function cannot safely resolve
    /// on its own (e.g. `Completed` was persisted but the action
    /// verifiably did *not* happen). Whether it's safe to just rerun
    /// the action depends on whether that specific action is
    /// idempotent — information this generic function doesn't have —
    /// so it reports the disagreement rather than guessing; the caller
    /// (who knows the action) decides between re-running and flagging
    /// for manual repair.
    Inconsistent,
}

pub fn reconcile(persisted: StagedOperationState, action_confirmed: bool) -> ReconciliationAction {
    use ReconciliationAction::*;
    use StagedOperationState::*;
    match (persisted, action_confirmed) {
        (IntentPersisted, false) => RerunAction,
        (IntentPersisted, true) => RecordActionPerformed,
        (ActionPerformed, true) => RecordActionPerformed, // idempotent no-op if already recorded
        (ActionPerformed, false) => Inconsistent,         // claimed done, reality disagrees
        (Completed, true) => NothingToDo,
        (Completed, false) => Inconsistent, // §11's exact scenario
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use StagedOperationState::*;

    #[test]
    fn pipeline_transitions_are_linear_no_skipping() {
        assert!(can_transition(IntentPersisted, ActionPerformed));
        assert!(can_transition(ActionPerformed, Completed));
        assert!(!can_transition(IntentPersisted, Completed));
        assert!(!can_transition(Completed, IntentPersisted));
    }

    #[test]
    fn intent_only_with_action_not_confirmed_reruns_safely() {
        assert_eq!(
            reconcile(IntentPersisted, false),
            ReconciliationAction::RerunAction
        );
    }

    #[test]
    fn intent_only_but_action_actually_happened_catches_up_the_record() {
        // Crash occurred between performing the action and persisting
        // that fact — the action itself must not run twice.
        assert_eq!(
            reconcile(IntentPersisted, true),
            ReconciliationAction::RecordActionPerformed
        );
    }

    #[test]
    fn action_performed_confirmed_true_is_a_safe_no_op_catchup() {
        assert_eq!(
            reconcile(ActionPerformed, true),
            ReconciliationAction::RecordActionPerformed
        );
    }

    #[test]
    fn completed_and_confirmed_needs_nothing_further() {
        assert_eq!(
            reconcile(Completed, true),
            ReconciliationAction::NothingToDo
        );
    }

    #[test]
    fn completed_but_action_not_confirmed_is_11s_exact_named_scenario() {
        // "database row says file exists but file rename failed."
        assert_eq!(
            reconcile(Completed, false),
            ReconciliationAction::Inconsistent
        );
    }

    #[test]
    fn action_performed_but_not_confirmed_is_also_flagged_inconsistent() {
        assert_eq!(
            reconcile(ActionPerformed, false),
            ReconciliationAction::Inconsistent
        );
    }
}
