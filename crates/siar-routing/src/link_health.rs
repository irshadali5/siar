//! Measures `PathEntry.rtt_millis`/`reliability` from real send
//! outcomes — the missing half this crate's own top-level doc comment
//! has flagged since Phase 5: `siar-connectivity::TransportManager::
//! sync_local_peers`'s `None`/`1.0` defaults are "best guesses, not
//! measurements," because nothing anywhere records what actually
//! happens when a message is sent over a given link.
//!
//! [`LinkHealth`] is that missing recorder — pure logic, no I/O, same
//! shape as everything else in this crate: a caller with a real
//! connection attempt (timed, success/failure known) calls
//! [`LinkHealth::record_outcome`]; this type turns a bounded history of
//! those into the two numbers `PathEntry` has always had fields for.
//! Nothing in this workspace calls `record_outcome` with a real
//! observation yet — that wiring needs an actual `SiarEndpoint::send`
//! attempt to time, which is real transport-touching work belonging to
//! whatever owns that connection (`siar-messaging`/`siar-connectivity`),
//! not this infra-free crate. This type is the computation a future
//! caller needs, built now so that wiring doesn't also have to invent
//! "how do I turn a pile of send attempts into a reliability number"
//! from scratch later — same "decision ready, real inputs still owed"
//! shape as `path.rs`'s `recommend_upgrade` before `TransportLink::
//! preference_rank` existed to drive it.

use std::collections::VecDeque;

/// One measured send attempt. A failed attempt has no meaningful RTT —
/// `rtt_millis` is only ever read for attempts where `success` is
/// `true`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SendOutcome {
    pub success: bool,
    pub rtt_millis: Option<u32>,
}

impl SendOutcome {
    pub fn success(rtt_millis: u32) -> Self {
        Self { success: true, rtt_millis: Some(rtt_millis) }
    }

    pub fn failure() -> Self {
        Self { success: false, rtt_millis: None }
    }
}

/// A bounded rolling window of [`SendOutcome`]s for one link — next.md
/// §94's "never permit unlimited" discipline applied to measurement
/// history too, not just the scheduler's send queues: a link that's
/// been sent over thousands of times shouldn't grow this without
/// bound. Older observations are dropped first, so `reliability`/
/// `average_rtt_millis` always reflect *recent* behavior — matching
/// `PathTable::remove_stale`'s own reasoning that a stale signal is
/// worse than no signal on a topology that "changes too quickly"
/// (next.md §92).
pub struct LinkHealth {
    window: VecDeque<SendOutcome>,
    capacity: usize,
}

impl LinkHealth {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity >= 1, "a zero-capacity window could never track anything");
        Self { window: VecDeque::new(), capacity }
    }

    pub fn record_outcome(&mut self, outcome: SendOutcome) {
        if self.window.len() >= self.capacity {
            self.window.pop_front();
        }
        self.window.push_back(outcome);
    }

    /// Fraction of recorded attempts that succeeded, `0.0..=1.0`. `1.0`
    /// — not `0.0` or some other "unknown" stand-in — when nothing has
    /// been recorded yet, matching `TransportManager::sync_local_peers`'s
    /// own existing placeholder: an unmeasured link defaults to "assume
    /// it works" rather than reading as artificially unreliable before
    /// any real data exists.
    pub fn reliability(&self) -> f32 {
        if self.window.is_empty() {
            return 1.0;
        }
        let successes = self.window.iter().filter(|o| o.success).count();
        successes as f32 / self.window.len() as f32
    }

    /// Mean RTT across successful attempts with a recorded RTT —
    /// `None` if there are none yet, matching `PathEntry.rtt_millis`'s
    /// own `Option<u32>` shape rather than inventing a fake zero.
    pub fn average_rtt_millis(&self) -> Option<u32> {
        let samples: Vec<u64> =
            self.window.iter().filter(|o| o.success).filter_map(|o| o.rtt_millis).map(u32::into).collect();
        if samples.is_empty() {
            return None;
        }
        let sum: u64 = samples.iter().sum();
        Some((sum / samples.len() as u64) as u32)
    }

    /// How many observations this window currently holds — lets a
    /// caller distinguish "genuinely 100% reliable" from "no data yet,
    /// defaulting to 100%" if that distinction ever matters to it.
    pub fn observation_count(&self) -> usize {
        self.window.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unmeasured_link_defaults_to_fully_reliable_not_zero() {
        let health = LinkHealth::new(4);
        assert_eq!(health.reliability(), 1.0);
        assert_eq!(health.average_rtt_millis(), None);
        assert_eq!(health.observation_count(), 0);
    }

    #[test]
    fn reliability_reflects_the_mix_of_recorded_outcomes() {
        let mut health = LinkHealth::new(4);
        health.record_outcome(SendOutcome::success(50));
        health.record_outcome(SendOutcome::success(60));
        health.record_outcome(SendOutcome::failure());
        health.record_outcome(SendOutcome::failure());
        assert_eq!(health.reliability(), 0.5);
    }

    #[test]
    fn average_rtt_only_counts_successful_attempts() {
        let mut health = LinkHealth::new(4);
        health.record_outcome(SendOutcome::success(100));
        health.record_outcome(SendOutcome::success(200));
        health.record_outcome(SendOutcome::failure());
        assert_eq!(health.average_rtt_millis(), Some(150));
    }

    #[test]
    fn window_drops_oldest_observation_once_at_capacity() {
        let mut health = LinkHealth::new(2);
        health.record_outcome(SendOutcome::failure());
        health.record_outcome(SendOutcome::failure());
        assert_eq!(health.reliability(), 0.0);

        // A capacity-2 window should have forgotten one of the two
        // earlier failures by the time these two successes land.
        health.record_outcome(SendOutcome::success(10));
        health.record_outcome(SendOutcome::success(10));
        assert_eq!(health.reliability(), 1.0);
        assert_eq!(health.observation_count(), 2);
    }

    #[test]
    #[should_panic(expected = "zero-capacity")]
    fn zero_capacity_window_is_rejected_up_front() {
        LinkHealth::new(0);
    }
}
