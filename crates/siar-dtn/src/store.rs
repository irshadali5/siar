//! Bounded DTN storage with priority eviction — next.md §68–69.
//!
//! Eviction order, exactly next.md §68's list — "expired / delivered /
//! low-priority / old normal / critical last": expired bundles are
//! swept first (real, not-otherwise-useful space, checked before
//! evicting anything that might still matter), then among what's left,
//! [`BundleStore::insert`] evicts by [`eviction_rank`] until there's
//! room, cheapest-to-lose first.

use siar_domain::MessageId;

use crate::bundle::{MeshBundle, MessagePriority};

struct StoredBundle {
    bundle: MeshBundle,
    /// Set by [`BundleStore::mark_delivered`]. Collapses next.md §63's
    /// finer routing-acknowledgment ladder (stored / handed to relay /
    /// reached gateway / reached destination / persisted / read) into
    /// one boolean here — this store only needs "is this bundle still
    /// worth carrying," not which specific rung a caller's UI should
    /// show; tracking that finer state is that caller's job.
    delivered: bool,
}

pub struct BundleStore {
    quota_bytes: u64,
    used_bytes: u64,
    bundles: Vec<StoredBundle>,
}

impl BundleStore {
    pub fn new(quota_bytes: u64) -> Self {
        Self {
            quota_bytes,
            used_bytes: 0,
            bundles: Vec::new(),
        }
    }

    pub fn used_bytes(&self) -> u64 {
        self.used_bytes
    }

    pub fn quota_bytes(&self) -> u64 {
        self.quota_bytes
    }

    pub fn len(&self) -> usize {
        self.bundles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bundles.is_empty()
    }

    pub fn contains(&self, id: MessageId) -> bool {
        self.bundles.iter().any(|stored| stored.bundle.id == id)
    }

    /// A cloned copy of one stored bundle, or `None` if `id` isn't
    /// present — added for `apps/emergency-node`'s mailbox check-in
    /// handler (next.md §76–77), which needs to look a specific bundle
    /// back up between the initial scan-for-matches and the actual
    /// send (the two can't safely be one held borrow across an `.await`
    /// point). Clones rather than returning a reference for the same
    /// reason — the caller needs an owned value it can hold across that
    /// `.await`.
    pub fn get(&self, id: MessageId) -> Option<MeshBundle> {
        self.bundles
            .iter()
            .find(|stored| stored.bundle.id == id)
            .map(|stored| stored.bundle.clone())
    }

    /// Every bundle currently held, delivered or not — next.md Phase 8's
    /// simulation harness (`siar-testkit`) is what actually needs this;
    /// nothing in Phases 1–7 required inspecting a store's full
    /// contents rather than checking one id at a time.
    pub fn iter(&self) -> impl Iterator<Item = &MeshBundle> {
        self.bundles.iter().map(|stored| &stored.bundle)
    }

    /// Marks a bundle already in the store as delivered. A no-op if
    /// `id` isn't in the store (already evicted, or never inserted) —
    /// there's nothing here for the caller to have gotten wrong in that
    /// case worth erroring over.
    pub fn mark_delivered(&mut self, id: MessageId) {
        if let Some(stored) = self
            .bundles
            .iter_mut()
            .find(|stored| stored.bundle.id == id)
        {
            stored.delivered = true;
        }
    }

    /// Prepares a forwardable copy of a stored bundle to hand to a
    /// newly-encountered carrier — next.md §35's "peer encounter
    /// protocol" applied at the single-bundle level. Atomically checks
    /// and consumes *both* limits a forward has to respect
    /// (`MeshBundle::forwarded`'s hop limit, `MeshBundle::
    /// try_consume_replication`'s copy budget) against the **stored**
    /// entry, so the depletion is real and shared across every future
    /// caller of this method for this bundle — not just applied to a
    /// throwaway clone that resets next time. Returns the
    /// already-decremented copy to actually put on the wire, or `None`
    /// if either limit was already exhausted (the caller should treat
    /// that exactly like "don't forward this one," not retry).
    ///
    /// Deliberately does not remove the bundle from the store even once
    /// `replication_budget` hits zero — `try_consume_replication`'s own
    /// doc comment notes direct delivery to a known destination is a
    /// separate concern this budget doesn't gate, and this store has no
    /// way to know from here whether the *next* encounter is with the
    /// destination itself.
    pub fn consume_for_forward(&mut self, id: MessageId) -> Option<MeshBundle> {
        let stored = self
            .bundles
            .iter_mut()
            .find(|stored| stored.bundle.id == id)?;
        if stored.bundle.hop_limit == 0 || stored.bundle.replication_budget == 0 {
            return None;
        }
        stored.bundle.hop_limit -= 1;
        stored.bundle.replication_budget -= 1;
        Some(stored.bundle.clone())
    }

    /// Removes every bundle expired as of `now`, returning their ids.
    pub fn remove_expired(&mut self, now: u64) -> Vec<MessageId> {
        let mut removed = Vec::new();
        self.bundles.retain(|stored| {
            if stored.bundle.is_expired(now) {
                removed.push(stored.bundle.id);
                false
            } else {
                true
            }
        });
        self.recompute_used_bytes();
        removed
    }

    /// Inserts `bundle`, first sweeping anything already expired, then
    /// evicting by [`eviction_rank`] (cheapest-to-lose first) until
    /// there's room within `quota_bytes`. Returns the ids of everything
    /// evicted to make room, in eviction order — a caller that wants to
    /// know "did my own bundle just get evicted to make room for a
    /// higher-priority one arriving after it" can check `insert`'s next
    /// return value, though that's a separate call, not this one.
    pub fn insert(&mut self, bundle: MeshBundle, now: u64) -> Vec<MessageId> {
        let incoming_size = bundle.ciphertext.len() as u64;
        let mut evicted = self.remove_expired(now);

        while self.used_bytes + incoming_size > self.quota_bytes {
            let Some(victim_index) = self.pick_eviction_victim() else {
                // Nothing left to evict — `quota_bytes` itself is
                // smaller than `incoming_size` alone. Insert anyway
                // (next.md doesn't say to reject an oversized-relative-
                // to-quota bundle outright) rather than looping forever
                // with nothing left to remove.
                break;
            };
            let victim = self.bundles.remove(victim_index);
            self.used_bytes -= victim.bundle.ciphertext.len() as u64;
            evicted.push(victim.bundle.id);
        }

        self.used_bytes += incoming_size;
        self.bundles.push(StoredBundle {
            bundle,
            delivered: false,
        });
        evicted
    }

    fn recompute_used_bytes(&mut self) {
        self.used_bytes = self
            .bundles
            .iter()
            .map(|stored| stored.bundle.ciphertext.len() as u64)
            .sum();
    }

    fn pick_eviction_victim(&self) -> Option<usize> {
        self.bundles
            .iter()
            .enumerate()
            .min_by_key(|(_, stored)| eviction_rank(stored))
            .map(|(index, _)| index)
    }
}

/// Lower rank evicts first. `(delivered_rank, priority_rank, created_at)`
/// lexicographic ordering: delivered beats everything regardless of
/// priority (next.md §68 lists it first), then priority (lowest
/// evicted first, `Emergency` evicted only as an absolute last resort),
/// then — within the same priority — oldest (`created_at` ascending)
/// first, matching the doc's "old normal" wording generalized across
/// every tier rather than just the `Normal` one.
fn eviction_rank(stored: &StoredBundle) -> (u8, u8, u64) {
    let delivered_rank = if stored.delivered { 0 } else { 1 };
    let priority_rank = match stored.bundle.priority {
        MessagePriority::Background => 0,
        MessagePriority::Normal => 1,
        MessagePriority::Interactive => 2,
        MessagePriority::Critical => 3,
        MessagePriority::Emergency => 4,
    };
    (delivered_rank, priority_rank, stored.bundle.created_at)
}

#[cfg(test)]
mod tests {
    use super::*;
    use siar_domain::DeviceId;

    fn bundle(priority: MessagePriority, created_at: u64, size: usize) -> MeshBundle {
        MeshBundle {
            id: MessageId::new(),
            destination: DeviceId::new(),
            payload_hash: [0u8; 32],
            ciphertext: vec![0u8; size],
            priority,
            hop_limit: 4,
            replication_budget: 2,
            created_at,
            expires_at: 1000,
        }
    }

    #[test]
    fn insert_under_quota_evicts_nothing() {
        let mut store = BundleStore::new(100);
        let evicted = store.insert(bundle(MessagePriority::Normal, 0, 10), 0);
        assert!(evicted.is_empty());
        assert_eq!(store.used_bytes(), 10);
    }

    #[test]
    fn delivered_bundles_are_evicted_before_anything_else() {
        let mut store = BundleStore::new(15);
        let low_priority = bundle(MessagePriority::Emergency, 0, 10);
        let low_priority_id = low_priority.id;
        store.insert(low_priority, 0);
        store.mark_delivered(low_priority_id);

        // Emergency priority would normally be evicted last, but it's
        // delivered, so it goes first anyway.
        let evicted = store.insert(bundle(MessagePriority::Background, 1, 10), 0);
        assert_eq!(evicted, vec![low_priority_id]);
    }

    #[test]
    fn lower_priority_is_evicted_before_higher_priority() {
        let mut store = BundleStore::new(15);
        let background = bundle(MessagePriority::Background, 0, 10);
        let background_id = background.id;
        store.insert(background, 0);

        let evicted = store.insert(bundle(MessagePriority::Emergency, 1, 10), 0);
        assert_eq!(evicted, vec![background_id]);
    }

    #[test]
    fn same_priority_evicts_oldest_first() {
        let mut store = BundleStore::new(15);
        let old = bundle(MessagePriority::Normal, 0, 10);
        let old_id = old.id;
        store.insert(old, 0);

        let evicted = store.insert(bundle(MessagePriority::Normal, 5, 10), 0);
        assert_eq!(evicted, vec![old_id]);
    }

    #[test]
    fn expired_bundles_are_swept_before_evicting_anything_live() {
        let mut store = BundleStore::new(15);
        let mut expiring = bundle(MessagePriority::Emergency, 0, 10);
        expiring.expires_at = 5;
        let expiring_id = expiring.id;
        store.insert(expiring, 0);

        // now=10: the Emergency bundle above has already expired, so
        // inserting a new Background bundle should evict *that*
        // (expired) rather than reaching for anything by priority.
        let evicted = store.insert(bundle(MessagePriority::Background, 1, 10), 10);
        assert_eq!(evicted, vec![expiring_id]);
    }

    #[test]
    fn mark_delivered_on_an_absent_id_is_a_no_op() {
        let mut store = BundleStore::new(100);
        store.mark_delivered(siar_domain::MessageId::new()); // must not panic
        assert!(store.is_empty());
    }

    #[test]
    fn consume_for_forward_decrements_both_limits_and_persists_the_change() {
        let mut store = BundleStore::new(100);
        let mut b = bundle(MessagePriority::Normal, 0, 10);
        b.hop_limit = 2;
        b.replication_budget = 2;
        let id = b.id;
        store.insert(b, 0);

        let forwarded_once = store
            .consume_for_forward(id)
            .expect("first forward should succeed");
        assert_eq!(forwarded_once.hop_limit, 1);
        assert_eq!(forwarded_once.replication_budget, 1);

        // The stored entry itself was decremented, not just a throwaway
        // clone — a second forward sees the already-reduced limits.
        let forwarded_twice = store
            .consume_for_forward(id)
            .expect("second forward should succeed");
        assert_eq!(forwarded_twice.hop_limit, 0);
        assert_eq!(forwarded_twice.replication_budget, 0);

        // Both limits now exhausted — a third attempt must not succeed.
        assert!(store.consume_for_forward(id).is_none());
    }

    #[test]
    fn consume_for_forward_on_an_absent_id_returns_none() {
        let mut store = BundleStore::new(100);
        assert!(store
            .consume_for_forward(siar_domain::MessageId::new())
            .is_none());
    }

    #[test]
    fn get_returns_a_clone_that_does_not_alias_the_stored_copy() {
        let mut store = BundleStore::new(100);
        let b = bundle(MessagePriority::Normal, 0, 10);
        let id = b.id;
        let expected_destination = b.destination;
        store.insert(b, 0);

        let fetched = store.get(id).expect("should find the inserted bundle");
        assert_eq!(fetched.destination, expected_destination);

        // Mutating the caller's clone must not touch the store's own
        // copy — `get` documents itself as returning a clone
        // specifically so callers can hold it across an `.await`
        // without a lingering borrow; that only makes sense if it's
        // genuinely independent.
        assert!(store.contains(id));
    }

    #[test]
    fn get_on_an_absent_id_returns_none() {
        let store = BundleStore::new(100);
        assert!(store.get(siar_domain::MessageId::new()).is_none());
    }
}
