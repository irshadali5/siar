//! Bounded seen-bundle deduplication — next.md §31: "Without this, mesh
//! forwarding creates storms."
//!
//! A node needs to remember which `MessageId`s it's already processed
//! or forwarded so it doesn't re-process the same bundle arriving via
//! multiple paths — next.md §42's whole point is one message crossing
//! BLE → Wi-Fi Direct → LAN → Internet without re-encryption, which
//! means it can plausibly arrive at any given hop more than once.
//!
//! Bounded by entry count, not wall-clock age — same reasoning as
//! `siar_calls::jitter::JitterBuffer` and
//! `siar_transport_ble::reassembly::ReassemblyBuffer`: this stays pure
//! and testable without mocking time, and next.md §69's DoS-resistance
//! concern (unbounded memory growth from a flood of distinct ids) is
//! what actually needs bounding here, not elapsed time specifically.

use std::collections::{HashSet, VecDeque};

use siar_domain::MessageId;

pub struct SeenBundles {
    capacity: usize,
    /// Oldest-first insertion order, so eviction always drops the
    /// least-recently-seen id — a `HashSet` alone can't offer that
    /// ordering.
    order: VecDeque<MessageId>,
    set: HashSet<MessageId>,
}

impl SeenBundles {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity >= 1, "a zero-capacity seen-set can never remember anything, defeating deduplication entirely");
        Self {
            capacity,
            order: VecDeque::with_capacity(capacity),
            set: HashSet::with_capacity(capacity),
        }
    }

    /// Returns `true` if `id` had already been seen — the caller should
    /// drop this bundle rather than process or forward it again.
    /// Returns `false` on first sighting, in which case `id` is now
    /// recorded (evicting the oldest entry first if already at
    /// capacity).
    pub fn check_and_record(&mut self, id: MessageId) -> bool {
        if self.set.contains(&id) {
            return true;
        }
        if self.order.len() >= self.capacity {
            if let Some(oldest) = self.order.pop_front() {
                self.set.remove(&oldest);
            }
        }
        self.order.push_back(id);
        self.set.insert(id);
        false
    }

    /// Read-only peek — `true` if `id` has already been recorded, same
    /// answer [`Self::check_and_record`] would give, without recording
    /// anything or evicting on `id`'s account. `siar-testkit`'s mesh
    /// simulation needs this to decide whether a bundle is worth
    /// forwarding *before* committing to forwarding it — `check_and_
    /// record` would have already marked it seen by the time the answer
    /// came back, which is the wrong order for "should I bother
    /// preparing this forward at all."
    pub fn contains(&self, id: MessageId) -> bool {
        self.set.contains(&id)
    }

    pub fn len(&self) -> usize {
        self.order.len()
    }

    pub fn is_empty(&self) -> bool {
        self.order.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_sighting_returns_false_and_is_then_remembered() {
        let mut seen = SeenBundles::new(4);
        let id = MessageId::new();
        assert!(!seen.check_and_record(id));
        assert_eq!(seen.len(), 1);
    }

    #[test]
    fn repeated_sighting_returns_true_and_does_not_grow() {
        let mut seen = SeenBundles::new(4);
        let id = MessageId::new();
        assert!(!seen.check_and_record(id));
        assert!(seen.check_and_record(id));
        assert!(seen.check_and_record(id));
        assert_eq!(seen.len(), 1);
    }

    #[test]
    fn exceeding_capacity_evicts_the_oldest_and_forgets_it() {
        let mut seen = SeenBundles::new(2);
        let (a, b, c) = (MessageId::new(), MessageId::new(), MessageId::new());
        seen.check_and_record(a);
        seen.check_and_record(b);
        seen.check_and_record(c); // evicts `a`
        assert_eq!(seen.len(), 2);
        // `a` was forgotten, so it now looks like a fresh sighting again
        // — a bounded seen-set trading perfect memory for bounded
        // memory is the whole point (next.md §69), not a bug.
        assert!(!seen.check_and_record(a));
    }
}
