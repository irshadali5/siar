//! §8 "Recovery State Machine".
//!
//! §8 lists the happy-path chain and the failure branches as two flat
//! lists, without wiring which failure state is reachable from which
//! stage. [`can_transition`]'s edges for the failure branches are this
//! module's own reasoned mapping (documented per-edge below), not a
//! transcription — the happy-path chain itself is verbatim.

use serde::{Deserialize, Serialize};

/// §8, verbatim happy-path chain plus verbatim failure-branch variant
/// list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RecoveryState {
    Starting,
    StorageOpened,
    IntegrityChecked,
    Reconciling,
    Recovered,
    RuntimeReady,
    // Failure branches (§8's own list).
    ReadOnlyRecovery,
    ManualRepairRequired,
    FatalCorruption,
    MigrationFailed,
    StorageUnavailable,
}

impl RecoveryState {
    /// The four states with no automatic further transition — reaching
    /// one means recovery has stopped and needs an external actor
    /// (an operator, a repair tool, a reinstall) rather than more
    /// automatic recovery code. [`RecoveryState::ReadOnlyRecovery`] is
    /// deliberately not terminal — see [`can_transition`]'s own note on
    /// that edge.
    pub fn is_terminal_failure(self) -> bool {
        matches!(
            self,
            RecoveryState::ManualRepairRequired
                | RecoveryState::FatalCorruption
                | RecoveryState::MigrationFailed
                | RecoveryState::StorageUnavailable
        )
    }
}

/// Whether `to` is a valid next state from `from`.
pub fn can_transition(from: RecoveryState, to: RecoveryState) -> bool {
    use RecoveryState::*;
    matches!(
        (from, to),
        // §8's own happy-path chain, verbatim.
        (Starting, StorageOpened)
            | (StorageOpened, IntegrityChecked)
            | (IntegrityChecked, Reconciling)
            | (Reconciling, Recovered)
            | (Recovered, RuntimeReady)
            // Opening storage itself can fail outright — nothing to
            // check integrity of yet.
            | (StorageOpened, StorageUnavailable)
            // Integrity checking is where corruption or a schema
            // mismatch is actually discovered, so all three
            // integrity-shaped failure branches originate here.
            | (IntegrityChecked, FatalCorruption)
            | (IntegrityChecked, ManualRepairRequired)
            | (IntegrityChecked, MigrationFailed)
            // Reconciling (replaying/resolving ambiguous operations)
            // can determine it isn't safe to reconcile automatically —
            // the degraded fallback is read-only access rather than a
            // hard failure, since the data itself passed integrity
            // checking.
            | (Reconciling, ReadOnlyRecovery)
            // §8 doesn't say a read-only recovery is stuck forever —
            // "read-only" describes a capability restriction, not a
            // terminal state, so this module allows it to still reach
            // a running (if degraded) runtime rather than treating it
            // as equivalent to the four true terminal failures.
            | (ReadOnlyRecovery, RuntimeReady)
    )
}

/// A small stateful wrapper around [`can_transition`] — tracks the
/// current state and rejects any attempted transition
/// [`can_transition`] doesn't allow, so a caller can't accidentally
/// skip a stage (e.g. `Starting` straight to `Recovered`) by forgetting
/// to call the free functions in order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecoveryStateMachine {
    current: RecoveryState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("invalid recovery transition: {from:?} -> {to:?}")]
pub struct InvalidTransition {
    pub from: RecoveryState,
    pub to: RecoveryState,
}

impl RecoveryStateMachine {
    pub fn new() -> Self {
        Self {
            current: RecoveryState::Starting,
        }
    }

    pub fn current(&self) -> RecoveryState {
        self.current
    }

    pub fn transition(&mut self, to: RecoveryState) -> Result<(), InvalidTransition> {
        if can_transition(self.current, to) {
            self.current = to;
            Ok(())
        } else {
            Err(InvalidTransition {
                from: self.current,
                to,
            })
        }
    }
}

impl Default for RecoveryStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use RecoveryState::*;

    #[test]
    fn happy_path_runs_start_to_finish() {
        let mut m = RecoveryStateMachine::new();
        for next in [
            StorageOpened,
            IntegrityChecked,
            Reconciling,
            Recovered,
            RuntimeReady,
        ] {
            m.transition(next).unwrap();
        }
        assert_eq!(m.current(), RuntimeReady);
    }

    #[test]
    fn skipping_a_stage_is_rejected() {
        let mut m = RecoveryStateMachine::new();
        let err = m.transition(Reconciling).unwrap_err();
        assert_eq!(
            err,
            InvalidTransition {
                from: Starting,
                to: Reconciling
            }
        );
        // Rejected transition must not have moved the state.
        assert_eq!(m.current(), Starting);
    }

    #[test]
    fn integrity_failures_are_reachable_only_from_integrity_checked() {
        for failure in [FatalCorruption, ManualRepairRequired, MigrationFailed] {
            let mut m = RecoveryStateMachine::new();
            assert!(
                m.transition(failure).is_err(),
                "should not be reachable from Starting"
            );

            let mut m = RecoveryStateMachine::new();
            m.transition(StorageOpened).unwrap();
            m.transition(IntegrityChecked).unwrap();
            assert!(
                m.transition(failure).is_ok(),
                "should be reachable from IntegrityChecked"
            );
        }
    }

    #[test]
    fn terminal_failures_have_no_further_transition() {
        for failure in [
            ManualRepairRequired,
            FatalCorruption,
            MigrationFailed,
            StorageUnavailable,
        ] {
            assert!(failure.is_terminal_failure());
            for candidate in [
                Starting,
                StorageOpened,
                IntegrityChecked,
                Reconciling,
                Recovered,
                RuntimeReady,
            ] {
                assert!(
                    !can_transition(failure, candidate),
                    "{failure:?} should be terminal"
                );
            }
        }
    }

    #[test]
    fn read_only_recovery_can_still_reach_runtime_ready() {
        let mut m = RecoveryStateMachine::new();
        m.transition(StorageOpened).unwrap();
        m.transition(IntegrityChecked).unwrap();
        m.transition(Reconciling).unwrap();
        m.transition(ReadOnlyRecovery).unwrap();
        assert!(m.transition(RuntimeReady).is_ok());
        assert!(!ReadOnlyRecovery.is_terminal_failure());
    }
}
