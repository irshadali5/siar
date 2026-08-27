//! Bounded priority scheduling — next.md §93–94.
//!
//! Seven fixed queues, each individually bounded — next.md §94: "Never
//! permit unlimited async channels." [`PriorityScheduler`] doesn't send
//! or receive anything itself (no I/O, matching this whole crate); it's
//! the ordering/admission policy a real send loop consults, same
//! "decision logic over caller-supplied data" shape as `path.rs` and
//! `score.rs`.

use std::collections::VecDeque;

/// next.md §93's seven tiers, in priority order (declaration order is
/// significant: it's what `as usize` and every ordering comparison in
/// this file relies on — P0 is index 0, most urgent).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SchedulePriority {
    /// SOS / authority alert.
    P0Emergency,
    /// Delivery ACK / routing control.
    P1Control,
    P2Text,
    P3Voice,
    P4Thumbnail,
    P5Files,
    P6BackgroundSync,
}

const PRIORITY_COUNT: usize = 7;

impl SchedulePriority {
    /// Maps a `MeshBundle`'s `siar_domain::MessagePriority` (5 levels)
    /// onto this scheduler's tiers (7 levels) — added for
    /// `apps/emergency-node`'s forward-on-contact step, which needed
    /// *some* way to order candidate bundles by priority instead of
    /// `BundleStore::iter()`'s unordered iteration.
    ///
    /// Deliberately doesn't use `P3Voice`/`P4Thumbnail`/`P5Files` — an
    /// opaque `MeshBundle`'s ciphertext carries no content-type
    /// signal (see `MeshBundle`'s own fields: `priority`, not a
    /// separate content-kind), so mapping onto those content-specific
    /// tiers would be a guess this crate has no basis for, not a
    /// principled decision. `Interactive`/`Normal` collapse onto the
    /// same tier (`P2Text`) for the same reason — there's no third
    /// content-agnostic tier between "urgent" and "background" to
    /// spread them across without inventing a distinction the data
    /// doesn't support. The one invariant this mapping actually
    /// guarantees, and the only one `apps/emergency-node`'s scheduling
    /// use needs, is relative order: `Emergency > Critical >
    /// Interactive == Normal > Background`.
    pub fn from_message_priority(priority: siar_domain::MessagePriority) -> Self {
        use siar_domain::MessagePriority;
        match priority {
            MessagePriority::Emergency => SchedulePriority::P0Emergency,
            MessagePriority::Critical => SchedulePriority::P1Control,
            MessagePriority::Interactive | MessagePriority::Normal => SchedulePriority::P2Text,
            MessagePriority::Background => SchedulePriority::P6BackgroundSync,
        }
    }
}

pub struct PriorityScheduler<T> {
    queues: [VecDeque<T>; PRIORITY_COUNT],
    capacity_per_queue: usize,
}

impl<T> PriorityScheduler<T> {
    pub fn new(capacity_per_queue: usize) -> Self {
        assert!(
            capacity_per_queue >= 1,
            "a zero-capacity scheduler could never admit anything"
        );
        Self {
            queues: std::array::from_fn(|_| VecDeque::new()),
            capacity_per_queue,
        }
    }

    /// Enqueues `item` at `priority`. Returns `false` (and drops `item`
    /// without enqueueing it) if that priority's queue is already at
    /// capacity — next.md §94 again. Every priority gets the same fixed
    /// capacity here; a caller that wants Emergency's queue larger than
    /// Background's should construct a scheduler sized for its own
    /// worst case, or run two `PriorityScheduler`s with different
    /// capacities, rather than this type silently favoring one tier.
    pub fn enqueue(&mut self, priority: SchedulePriority, item: T) -> bool {
        let queue = &mut self.queues[priority as usize];
        if queue.len() >= self.capacity_per_queue {
            return false;
        }
        queue.push_back(item);
        true
    }

    /// next.md §93: "BLE scheduler may send only P0–P3 under
    /// congestion." Pops the oldest item from the highest-priority
    /// non-empty queue at or above `congestion_ceiling` — `None`
    /// ceiling means uncongested operation, eligible to pull from any
    /// tier including P6.
    pub fn dequeue_next(&mut self, congestion_ceiling: Option<SchedulePriority>) -> Option<T> {
        let ceiling = congestion_ceiling.unwrap_or(SchedulePriority::P6BackgroundSync);
        for priority_index in 0..=(ceiling as usize) {
            if let Some(item) = self.queues[priority_index].pop_front() {
                return Some(item);
            }
        }
        None
    }

    pub fn len_at(&self, priority: SchedulePriority) -> usize {
        self.queues[priority as usize].len()
    }

    pub fn is_empty(&self) -> bool {
        self.queues.iter().all(VecDeque::is_empty)
    }

    /// Derives a congestion ceiling from this scheduler's own queue
    /// occupancy — closing the scheduler-facing half of the gap
    /// `siar-routing`'s crate doc comment has flagged since Phase 5
    /// ("detecting real congestion to pick a `congestion_ceiling`").
    /// Backlog in a bounded queue is a legitimate, self-contained
    /// congestion signal on its own (the same active-queue-management
    /// reading networking's own RED/CoDel algorithms use: the sender is
    /// producing work faster than the link is draining it) — not a
    /// placeholder standing in for a real network-level RTT/loss
    /// measurement, which is [`crate::link_health::LinkHealth`]'s
    /// separate, still-unwired half of this same gap (see that module's
    /// doc comment for why it can't be computed here).
    ///
    /// Looks only at `P4Thumbnail`/`P5Files`/`P6BackgroundSync` —
    /// next.md §93's own dividing line for what a congested BLE link
    /// should stop sending. Backlog specifically in those tiers means
    /// non-urgent traffic is piling up, which throttling should
    /// address; backlog in `P0Emergency..P3Voice` instead means urgent
    /// traffic itself is the bottleneck, and lowering the ceiling
    /// further wouldn't help — there's nothing less urgent left to
    /// shed, so this never returns a ceiling below `P3Voice`.
    ///
    /// `occupancy_threshold` is the fraction of a queue's capacity
    /// (`0.0..=1.0`) that counts as "backed up" — left as a caller-
    /// supplied parameter rather than a hardcoded constant since the
    /// right threshold depends on `capacity_per_queue`, which this type
    /// doesn't dictate. Returns `None` (uncongested) when every
    /// throttled tier is below the threshold.
    pub fn congestion_ceiling(&self, occupancy_threshold: f32) -> Option<SchedulePriority> {
        use SchedulePriority::*;
        let throttled_tiers = [P4Thumbnail, P5Files, P6BackgroundSync];
        let congested = throttled_tiers.iter().any(|&tier| {
            let fill = self.queues[tier as usize].len() as f32 / self.capacity_per_queue as f32;
            fill >= occupancy_threshold
        });
        if congested {
            Some(P3Voice)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use SchedulePriority::*;

    #[test]
    fn dequeue_always_prefers_the_highest_priority_non_empty_queue() {
        let mut scheduler = PriorityScheduler::new(4);
        scheduler.enqueue(P5Files, "file");
        scheduler.enqueue(P2Text, "text");
        scheduler.enqueue(P0Emergency, "sos");
        assert_eq!(scheduler.dequeue_next(None), Some("sos"));
        assert_eq!(scheduler.dequeue_next(None), Some("text"));
        assert_eq!(scheduler.dequeue_next(None), Some("file"));
        assert_eq!(scheduler.dequeue_next(None), None);
    }

    #[test]
    fn same_priority_is_fifo() {
        let mut scheduler = PriorityScheduler::new(4);
        scheduler.enqueue(P2Text, "first");
        scheduler.enqueue(P2Text, "second");
        assert_eq!(scheduler.dequeue_next(None), Some("first"));
        assert_eq!(scheduler.dequeue_next(None), Some("second"));
    }

    #[test]
    fn congestion_ceiling_skips_lower_priority_items_entirely() {
        let mut scheduler = PriorityScheduler::new(4);
        scheduler.enqueue(P5Files, "file");
        scheduler.enqueue(P2Text, "text");
        // Under BLE congestion (§93: only P0-P3), P5 must not be
        // reachable no matter how long it's waited.
        assert_eq!(scheduler.dequeue_next(Some(P3Voice)), Some("text"));
        assert_eq!(scheduler.dequeue_next(Some(P3Voice)), None);
        // Lifting the ceiling makes the still-queued P5 item reachable.
        assert_eq!(scheduler.dequeue_next(None), Some("file"));
    }

    #[test]
    fn enqueue_rejects_once_that_priority_is_at_capacity() {
        let mut scheduler = PriorityScheduler::new(2);
        assert!(scheduler.enqueue(P2Text, "a"));
        assert!(scheduler.enqueue(P2Text, "b"));
        assert!(!scheduler.enqueue(P2Text, "c"));
        assert_eq!(scheduler.len_at(P2Text), 2);
    }

    #[test]
    fn capacity_is_tracked_independently_per_priority() {
        let mut scheduler = PriorityScheduler::new(1);
        assert!(scheduler.enqueue(P0Emergency, "sos"));
        // P0 is full, but that must not affect P6's own separate quota.
        assert!(scheduler.enqueue(P6BackgroundSync, "sync"));
        assert!(!scheduler.is_empty());
    }

    #[test]
    fn from_message_priority_preserves_relative_order() {
        use siar_domain::MessagePriority;
        assert!(
            SchedulePriority::from_message_priority(MessagePriority::Emergency)
                < SchedulePriority::from_message_priority(MessagePriority::Critical)
        );
        assert!(
            SchedulePriority::from_message_priority(MessagePriority::Critical)
                < SchedulePriority::from_message_priority(MessagePriority::Normal)
        );
        assert!(
            SchedulePriority::from_message_priority(MessagePriority::Normal)
                < SchedulePriority::from_message_priority(MessagePriority::Background)
        );
    }

    #[test]
    fn from_message_priority_collapses_interactive_and_normal_onto_the_same_tier() {
        use siar_domain::MessagePriority;
        assert_eq!(
            SchedulePriority::from_message_priority(MessagePriority::Interactive),
            SchedulePriority::from_message_priority(MessagePriority::Normal)
        );
    }

    #[test]
    fn congestion_ceiling_is_none_when_throttled_tiers_are_below_threshold() {
        let mut scheduler = PriorityScheduler::new(10);
        scheduler.enqueue(P5Files, "file");
        assert_eq!(scheduler.congestion_ceiling(0.5), None);
    }

    #[test]
    fn congestion_ceiling_triggers_once_a_throttled_tier_crosses_the_threshold() {
        let mut scheduler = PriorityScheduler::new(4);
        scheduler.enqueue(P6BackgroundSync, "a");
        scheduler.enqueue(P6BackgroundSync, "b");
        // 2/4 == the 0.5 threshold exactly — "at or above" per the
        // doc comment, not "strictly above".
        assert_eq!(scheduler.congestion_ceiling(0.5), Some(P3Voice));
    }

    #[test]
    fn congestion_ceiling_ignores_backlog_in_urgent_tiers() {
        let mut scheduler = PriorityScheduler::new(2);
        // P2Text completely full — real backlog, but not one throttling
        // could ever relieve (nothing lower-priority to shed here).
        scheduler.enqueue(P2Text, "a");
        scheduler.enqueue(P2Text, "b");
        assert_eq!(scheduler.congestion_ceiling(0.5), None);
    }

    #[test]
    fn congestion_ceiling_checks_every_throttled_tier_not_just_the_first() {
        let mut scheduler = PriorityScheduler::new(4);
        // P4/P5 stay under threshold; only P6 is backed up.
        scheduler.enqueue(P4Thumbnail, "thumb");
        scheduler.enqueue(P6BackgroundSync, "a");
        scheduler.enqueue(P6BackgroundSync, "b");
        scheduler.enqueue(P6BackgroundSync, "c");
        assert_eq!(scheduler.congestion_ceiling(0.5), Some(P3Voice));
    }
}
