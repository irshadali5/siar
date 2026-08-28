//! §7 "Crash Recovery Must Be Idempotent".
//!
//! §7's own test case, made real rather than left as prose: "run
//! recovery, crash halfway, run again — must be safe." This module
//! provides the mechanism that makes that safe for *any* sequence of
//! steps — a durable-in-spirit ledger of which step ids have already
//! completed, checked before each step runs, so a second pass after a
//! simulated crash resumes rather than re-executing already-finished
//! work.

use std::collections::HashSet;

/// A stable identifier for one recovery step — §7 doesn't name a
/// concrete id type, so a `&'static str` (the step's own name) is this
/// module's own choice, matching how steps are already named in this
/// spec's prose (e.g. "reconcile inbox", "resolve ambiguous sends").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RecoveryStepId(pub &'static str);

/// §7: "Every recovery step should be: idempotent, transactional, or
/// resumable." This trait models the *resumable* strategy directly —
/// [`run_recovery`] never calls [`RecoveryStep::execute`] twice for the
/// same id once [`RecoveryLedger::mark_completed`] has recorded it. A
/// step that's naturally idempotent or wrapped in its own transaction
/// (the other two strategies §7 names) can still implement this same
/// trait — the ledger skip is a no-op for a step that would have been
/// safe to re-run anyway, so this one mechanism covers all three
/// strategies without needing to know which one a given step uses.
pub trait RecoveryStep {
    fn id(&self) -> RecoveryStepId;
    fn execute(&mut self) -> Result<(), String>;
}

/// The record of which steps have already completed — a real
/// caller backs this with the durable store recovery itself is
/// running against (so the ledger survives the very crash it's meant
/// to protect against); this module only defines the in-memory shape
/// and the skip logic, the same seam/stand-in split
/// `shutdown_marker::ShutdownMarkerStore` already uses.
#[derive(Debug, Clone, Default)]
pub struct RecoveryLedger {
    completed: HashSet<&'static str>,
}

impl RecoveryLedger {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_completed(&self, id: RecoveryStepId) -> bool {
        self.completed.contains(id.0)
    }

    pub fn mark_completed(&mut self, id: RecoveryStepId) {
        self.completed.insert(id.0);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("recovery step {step:?} failed: {message}")]
pub struct RecoveryStepError {
    pub step: RecoveryStepId,
    pub message: String,
}

/// §7's own scenario, implemented directly: iterate `steps` in order,
/// skipping any whose id [`RecoveryLedger::is_completed`] already
/// reports done, executing and then marking complete every step that
/// isn't. If `steps` itself represents only part of the full recovery
/// pipeline (because the process crashed mid-run last time and this
/// call is a fresh process picking up from a partially-populated
/// `ledger`), already-completed steps are silently skipped rather than
/// re-executed — exactly §7's required property.
pub fn run_recovery(
    steps: &mut [Box<dyn RecoveryStep>],
    ledger: &mut RecoveryLedger,
) -> Result<(), RecoveryStepError> {
    for step in steps.iter_mut() {
        let id = step.id();
        if ledger.is_completed(id) {
            continue;
        }
        step.execute()
            .map_err(|message| RecoveryStepError { step: id, message })?;
        ledger.mark_completed(id);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    struct RecordingStep {
        id: RecoveryStepId,
        run_count: Rc<RefCell<Vec<&'static str>>>,
        fail: bool,
    }

    impl RecoveryStep for RecordingStep {
        fn id(&self) -> RecoveryStepId {
            self.id
        }

        fn execute(&mut self) -> Result<(), String> {
            self.run_count.borrow_mut().push(self.id.0);
            if self.fail {
                Err("simulated failure".to_string())
            } else {
                Ok(())
            }
        }
    }

    fn step(name: &'static str, log: &Rc<RefCell<Vec<&'static str>>>) -> Box<dyn RecoveryStep> {
        Box::new(RecordingStep {
            id: RecoveryStepId(name),
            run_count: Rc::clone(log),
            fail: false,
        })
    }

    #[test]
    fn a_full_clean_run_executes_every_step_once() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let mut steps = vec![step("a", &log), step("b", &log), step("c", &log)];
        let mut ledger = RecoveryLedger::new();

        run_recovery(&mut steps, &mut ledger).unwrap();
        assert_eq!(*log.borrow(), vec!["a", "b", "c"]);
    }

    #[test]
    fn resuming_after_a_simulated_crash_does_not_re_execute_completed_steps() {
        // §7's exact scenario: run recovery, "crash" partway (here,
        // simulated by only handing the pipeline its first two steps
        // on the first call), then run again with the full pipeline
        // and the *same* ledger a real crash-surviving store would
        // have preserved.
        let log = Rc::new(RefCell::new(Vec::new()));
        let mut ledger = RecoveryLedger::new();

        let mut first_run = vec![step("a", &log), step("b", &log)];
        run_recovery(&mut first_run, &mut ledger).unwrap();
        assert_eq!(*log.borrow(), vec!["a", "b"]);

        // Fresh process, full step list, surviving ledger.
        let mut second_run = vec![step("a", &log), step("b", &log), step("c", &log)];
        run_recovery(&mut second_run, &mut ledger).unwrap();

        // "a" and "b" must not appear twice — only "c" actually ran
        // this time.
        assert_eq!(*log.borrow(), vec!["a", "b", "c"]);
    }

    #[test]
    fn a_failing_step_stops_the_pipeline_without_marking_itself_complete() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let mut steps: Vec<Box<dyn RecoveryStep>> = vec![
            step("a", &log),
            Box::new(RecordingStep {
                id: RecoveryStepId("b"),
                run_count: Rc::clone(&log),
                fail: true,
            }),
            step("c", &log),
        ];
        let mut ledger = RecoveryLedger::new();

        let err = run_recovery(&mut steps, &mut ledger).unwrap_err();
        assert_eq!(err.step, RecoveryStepId("b"));
        // "c" never ran — the pipeline stopped at the failure.
        assert_eq!(*log.borrow(), vec!["a", "b"]);
        // "b" is not marked complete, so a retry will attempt it again
        // rather than silently skipping a step that actually failed.
        assert!(!ledger.is_completed(RecoveryStepId("b")));
        assert!(ledger.is_completed(RecoveryStepId("a")));
    }
}
