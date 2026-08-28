//! §9 "Storage Transaction Boundaries".
//!
//! §9's own example — "append event + update projection + enqueue
//! outbox, one transaction where consistency requires" — needs a real
//! database transaction to implement for real; this sandbox has no
//! such database available (the same rustc-floor constraint that
//! blocks a real SQLite backend everywhere else in this workspace, per
//! [[resilient-mesh]] project memory). [`TransactionGroup`] is
//! therefore a genuine in-memory *analog* of that all-or-nothing
//! property, not a substitute for a real database transaction: it
//! demonstrates and tests the actual behavior §9 requires (every step
//! in the group applies, or none do) against an in-memory state, so
//! the property itself is verified even though no bytes are durably
//! written. A real implementation swaps this for actual database
//! transactions per §10's own instruction not to reinvent what the
//! engine already provides.

pub type TransactionStep<S> = Box<dyn FnMut(&mut S) -> Result<(), String>>;

/// A batch of state-mutating steps that either all apply or none do —
/// §9's "one transaction where consistency requires," and its closing
/// warning against "splitting logically atomic state unnecessarily"
/// made structural: a caller that wants §9's own three-part example
/// atomic has to add all three steps to one [`TransactionGroup`]
/// before calling [`TransactionGroup::apply`], not three separate
/// calls that could each partially succeed.
pub struct TransactionGroup<S> {
    steps: Vec<TransactionStep<S>>,
}

impl<S: Clone> TransactionGroup<S> {
    pub fn new() -> Self {
        Self { steps: Vec::new() }
    }

    pub fn add_step(&mut self, step: impl FnMut(&mut S) -> Result<(), String> + 'static) {
        self.steps.push(Box::new(step));
    }

    pub fn len(&self) -> usize {
        self.steps.len()
    }

    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    /// Applies every step against a clone of `target`, and only writes
    /// the result back to `target` if every step succeeded — the
    /// all-or-nothing property §9 requires. On failure, `target` is
    /// left exactly as it was before this call, with no trace of any
    /// step that ran before the one that failed.
    pub fn apply(&mut self, target: &mut S) -> Result<(), String> {
        let mut staged = target.clone();
        for step in self.steps.iter_mut() {
            step(&mut staged)?;
        }
        *target = staged;
        Ok(())
    }
}

impl<S: Clone> Default for TransactionGroup<S> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// §9's own three-part example, modeled directly: an event log
    /// count, a projection version, and an outbox length.
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct ExampleState {
        event_count: u64,
        projection_version: u64,
        outbox_len: u64,
    }

    #[test]
    fn all_steps_apply_together_on_success() {
        let mut group: TransactionGroup<ExampleState> = TransactionGroup::new();
        group.add_step(|s: &mut ExampleState| {
            s.event_count += 1;
            Ok(())
        });
        group.add_step(|s: &mut ExampleState| {
            s.projection_version += 1;
            Ok(())
        });
        group.add_step(|s: &mut ExampleState| {
            s.outbox_len += 1;
            Ok(())
        });

        let mut state = ExampleState {
            event_count: 0,
            projection_version: 0,
            outbox_len: 0,
        };
        group.apply(&mut state).unwrap();
        assert_eq!(
            state,
            ExampleState {
                event_count: 1,
                projection_version: 1,
                outbox_len: 1
            }
        );
    }

    #[test]
    fn a_failing_step_rolls_back_every_earlier_step_in_the_same_group() {
        // §9's exact requirement, checked directly: if the outbox
        // enqueue fails, the event append and projection update from
        // the same group must not be left half-applied.
        let mut group: TransactionGroup<ExampleState> = TransactionGroup::new();
        group.add_step(|s: &mut ExampleState| {
            s.event_count += 1;
            Ok(())
        });
        group.add_step(|s: &mut ExampleState| {
            s.projection_version += 1;
            Ok(())
        });
        group.add_step(|_s: &mut ExampleState| Err("outbox full".to_string()));

        let mut state = ExampleState {
            event_count: 0,
            projection_version: 0,
            outbox_len: 0,
        };
        let err = group.apply(&mut state).unwrap_err();
        assert_eq!(err, "outbox full");
        // Completely untouched — not partially applied.
        assert_eq!(
            state,
            ExampleState {
                event_count: 0,
                projection_version: 0,
                outbox_len: 0
            }
        );
    }

    #[test]
    fn independent_groups_do_not_roll_back_each_other() {
        // A failure in one TransactionGroup must not affect state
        // already committed by an earlier, separate, successful group
        // — atomicity is per-group, not global.
        let mut first: TransactionGroup<ExampleState> = TransactionGroup::new();
        first.add_step(|s: &mut ExampleState| {
            s.event_count += 1;
            Ok(())
        });

        let mut state = ExampleState {
            event_count: 0,
            projection_version: 0,
            outbox_len: 0,
        };
        first.apply(&mut state).unwrap();
        assert_eq!(state.event_count, 1);

        let mut second: TransactionGroup<ExampleState> = TransactionGroup::new();
        second.add_step(|_s: &mut ExampleState| Err("boom".to_string()));
        assert!(second.apply(&mut state).is_err());

        // The first group's committed change survives the second
        // group's failure.
        assert_eq!(state.event_count, 1);
    }
}
