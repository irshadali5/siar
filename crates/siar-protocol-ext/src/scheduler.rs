//! §22 "Fair Scheduling": "Do not permanently starve bulk traffic. Use
//! weighted fair scheduling + strict bounded emergency override,
//! rather than naive perpetual highest-priority-first scheduling."

use std::collections::HashMap;

use crate::backpressure::BoundedQueue;
use crate::lifecycle::TrafficPriority;

const TIERS: [TrafficPriority; 6] = [
    TrafficPriority::Critical,
    TrafficPriority::Control,
    TrafficPriority::Interactive,
    TrafficPriority::Normal,
    TrafficPriority::Bulk,
    TrafficPriority::Background,
];

/// §22's own two-part rule, both parts real: weighted round-robin
/// across tiers (the "fair" half — every tier gets picked sometimes,
/// in proportion to its weight, never literally zero), plus
/// `max_consecutive_critical` as the "strict bounded"
/// half of "emergency override" — `Critical` traffic is allowed to
/// preempt everything else, but only for a bounded run of consecutive
/// picks before the scheduler is forced to yield to the next tier
/// regardless, which is what keeps "highest-priority-first" from
/// degrading into "permanently starve everything else" under
/// sustained `Critical` load (the spec's own named failure mode).
pub struct FairScheduler<T> {
    queues: HashMap<TrafficPriority, BoundedQueue<T>>,
    weights: HashMap<TrafficPriority, u32>,
    max_consecutive_critical: u32,
    consecutive_critical_picks: u32,
    /// Round-robin cursor for the non-Critical weighted rotation,
    /// tracked as (tier index, picks remaining at this tier's current
    /// weight) so a tier with weight 3 gets three consecutive turns
    /// before rotating — standard weighted round-robin, not randomized
    /// selection (§123-style determinism, same reasoning
    /// `siar-routing-policy`'s own tie-breaking already documents,
    /// applied here).
    cursor_tier_index: usize,
    cursor_picks_remaining: u32,
}

impl<T> FairScheduler<T> {
    /// `per_tier_capacity` bounds every tier's own [`BoundedQueue`]
    /// (§20: every queue in this crate is bounded, no exceptions —
    /// including a `Critical` one, since even emergency traffic must
    /// not be able to grow a queue without limit).
    pub fn new(
        weights: HashMap<TrafficPriority, u32>,
        per_tier_capacity: usize,
        max_consecutive_critical: u32,
    ) -> Self {
        let mut queues = HashMap::new();
        for tier in TIERS {
            queues.insert(tier, BoundedQueue::new(per_tier_capacity));
        }
        Self {
            queues,
            weights,
            max_consecutive_critical,
            consecutive_critical_picks: 0,
            cursor_tier_index: 0,
            cursor_picks_remaining: 0,
        }
    }

    /// §21's own worked weights — SOS/Critical weighted far above
    /// everything else but still bounded (see
    /// `max_consecutive_critical`'s own doc comment),
    /// Background lowest but nonzero (never literally starved).
    pub fn with_default_weights(per_tier_capacity: usize) -> Self {
        let weights = HashMap::from([
            (TrafficPriority::Critical, 20),
            (TrafficPriority::Control, 10),
            (TrafficPriority::Interactive, 8),
            (TrafficPriority::Normal, 4),
            (TrafficPriority::Bulk, 2),
            (TrafficPriority::Background, 1),
        ]);
        Self::new(weights, per_tier_capacity, 8)
    }

    pub fn enqueue(
        &mut self,
        priority: TrafficPriority,
        item: T,
    ) -> Result<(), (T, crate::backpressure::QueueFull)> {
        self.queues
            .get_mut(&priority)
            .expect("every TrafficPriority tier has a queue")
            .try_push(item)
    }

    /// Picks the next item to dispatch, or `None` if every queue is
    /// empty. `Critical` is checked first every call (true preemption
    /// for genuinely urgent traffic — an SOS message shouldn't wait
    /// out a round-robin cycle), but only up to
    /// `max_consecutive_critical` picks in a row; once that cap is
    /// hit, this function is forced to fall through to the weighted
    /// rotation even if more `Critical` items are still queued, which
    /// is what actually bounds the "override."
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<T> {
        if self.consecutive_critical_picks < self.max_consecutive_critical {
            if let Some(item) = self
                .queues
                .get_mut(&TrafficPriority::Critical)
                .unwrap()
                .pop()
            {
                self.consecutive_critical_picks += 1;
                return Some(item);
            }
        }
        self.consecutive_critical_picks = 0;
        self.next_from_weighted_rotation()
    }

    fn next_from_weighted_rotation(&mut self) -> Option<T> {
        // At most one full lap over all tiers — if every tier is
        // empty, this terminates instead of spinning forever.
        for _ in 0..TIERS.len() {
            if self.cursor_picks_remaining == 0 {
                self.cursor_tier_index = (self.cursor_tier_index + 1) % TIERS.len();
                let tier = TIERS[self.cursor_tier_index];
                self.cursor_picks_remaining = *self.weights.get(&tier).unwrap_or(&1);
            }
            let tier = TIERS[self.cursor_tier_index];
            if let Some(item) = self.queues.get_mut(&tier).unwrap().pop() {
                self.cursor_picks_remaining = self.cursor_picks_remaining.saturating_sub(1);
                return Some(item);
            }
            // This tier is empty — move on immediately rather than
            // burning its remaining weighted turns on nothing.
            self.cursor_picks_remaining = 0;
        }
        None
    }

    pub fn is_empty(&self) -> bool {
        self.queues.values().all(|q| q.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn critical_traffic_preempts_everything_else_up_to_the_bound() {
        let mut scheduler: FairScheduler<String> = FairScheduler::with_default_weights(100);
        scheduler
            .enqueue(TrafficPriority::Bulk, "bulk-1".to_string())
            .unwrap();
        for i in 0..8 {
            scheduler
                .enqueue(TrafficPriority::Critical, format!("sos-{i}"))
                .unwrap();
        }
        // First 8 picks are all Critical — the bound is exactly 8.
        for i in 0..8 {
            assert_eq!(scheduler.next(), Some(format!("sos-{i}")));
        }
    }

    #[test]
    fn bulk_traffic_is_never_permanently_starved_by_sustained_critical_load() {
        let mut scheduler: FairScheduler<String> = FairScheduler::with_default_weights(1000);
        scheduler
            .enqueue(TrafficPriority::Bulk, "bulk-1".to_string())
            .unwrap();
        // Flood Critical far past the consecutive-pick bound.
        for i in 0..100 {
            scheduler
                .enqueue(TrafficPriority::Critical, format!("sos-{i}"))
                .unwrap();
        }
        let mut picks = Vec::new();
        for _ in 0..20 {
            if let Some(item) = scheduler.next() {
                picks.push(item);
            }
        }
        // §22's own named failure mode this test guards against:
        // naive highest-priority-first scheduling would never let
        // "bulk-1" surface at all while Critical items remain queued.
        assert!(
            picks.contains(&"bulk-1".to_string()),
            "bulk traffic must surface even under sustained Critical flooding"
        );
    }

    #[test]
    fn an_empty_scheduler_returns_none_without_spinning() {
        let mut scheduler: FairScheduler<&str> = FairScheduler::with_default_weights(10);
        assert_eq!(scheduler.next(), None);
    }

    #[test]
    fn every_tier_queue_is_bounded_including_critical() {
        let mut scheduler = FairScheduler::with_default_weights(1);
        scheduler.enqueue(TrafficPriority::Critical, "a").unwrap();
        let result = scheduler.enqueue(TrafficPriority::Critical, "b");
        assert!(result.is_err()); // §20: no queue is ever unbounded, not even Critical's
    }

    #[test]
    fn a_tier_with_higher_weight_is_picked_more_often_in_the_rotation() {
        let weights = HashMap::from([
            (TrafficPriority::Critical, 0),
            (TrafficPriority::Control, 0),
            (TrafficPriority::Interactive, 0),
            (TrafficPriority::Normal, 5),
            (TrafficPriority::Bulk, 1),
            (TrafficPriority::Background, 0),
        ]);
        let mut scheduler = FairScheduler::new(weights, 100, 0);
        for i in 0..10 {
            scheduler
                .enqueue(TrafficPriority::Normal, format!("n{i}"))
                .unwrap();
            scheduler
                .enqueue(TrafficPriority::Bulk, format!("b{i}"))
                .unwrap();
        }
        let mut normal_count = 0;
        let mut bulk_count = 0;
        for _ in 0..12 {
            match scheduler.next() {
                Some(item) if item.starts_with('n') => normal_count += 1,
                Some(item) if item.starts_with('b') => bulk_count += 1,
                _ => {}
            }
        }
        assert!(
            normal_count > bulk_count,
            "Normal (weight 5) should be picked more often than Bulk (weight 1)"
        );
    }
}
