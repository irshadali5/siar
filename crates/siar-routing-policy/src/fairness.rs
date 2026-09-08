//! §67 "Per-Peer Fairness", §68 "Per-Extension Fairness".
//!
//! Both sections ask for the same shape: "one `{peer, extension}`
//! must not starve the others" — a bounded round-robin-by-key queue,
//! generic over what `key` actually is. [`RoundRobinFairQueue`]
//! implements that once, and this module's own tests instantiate it
//! both ways: keyed by `DeviceId` for §67's own "one peer transferring
//! a huge file must not starve messages to others," and keyed by
//! [`crate::descriptor::ContentClass`] for §68's own "files should not
//! starve messaging/receipts/presence" (`ContentClass` already
//! distinguishes `File` from `Text`/`Control`/etc, so it doubles as
//! §68's "extension" without inventing a second enum for the same
//! idea).
//!
//! Deliberately **not** wired into [`crate::dispatch::RouteDispatchQueue`]:
//! that type's per-tier storage is `siar-protocol-ext`'s own
//! `BoundedQueue` — a plain FIFO, fixed inside `FairScheduler` and out
//! of this crate's control (see this crate's own top doc comment on
//! not depending on `siar-transport`/wire concerns beyond what it
//! already does). A caller that wants peer- or extension-fair
//! delivery *within* one priority tier can use this type as that
//! tier's own producer-side buffer, feeding fairly-interleaved items
//! into [`crate::dispatch::RouteDispatchQueue::enqueue`] one at a time
//! — composable, the same posture as [`crate::security`]/[`crate::privacy`]'s
//! own elimination functions.

use std::collections::{HashMap, VecDeque};
use std::hash::Hash;

use siar_protocol_ext::backpressure::QueueFull;

/// A bounded queue-of-queues, one inner queue per distinct `key`,
/// popped in round-robin order across whichever keys currently hold
/// at least one item. `capacity_per_key` bounds each key's own inner
/// queue independently (§67's "must not starve messages to others" is
/// about *fairness*, not a shared bound — a global bound would let one
/// key's burst still crowd out the others' capacity even under
/// round-robin popping, whereas a per-key bound guarantees every key
/// always has room for its own next item regardless of how full
/// everyone else's queue is).
pub struct RoundRobinFairQueue<K, T> {
    capacity_per_key: usize,
    queues: HashMap<K, VecDeque<T>>,
    /// Rotation order. A key is appended here the first time it's
    /// pushed and stays in this ring for the queue's whole lifetime,
    /// even while its own inner queue is temporarily empty — so a
    /// peer that drains and later resumes sending falls back into its
    /// existing turn order rather than jumping to the front (or back)
    /// relative to peers who never stopped.
    rotation: VecDeque<K>,
}

impl<K: Eq + Hash + Clone, T> RoundRobinFairQueue<K, T> {
    pub fn new(capacity_per_key: usize) -> Self {
        Self {
            capacity_per_key,
            queues: HashMap::new(),
            rotation: VecDeque::new(),
        }
    }

    /// Rejects with the same `QueueFull` shape
    /// [`siar_protocol_ext::backpressure::BoundedQueue`] already uses,
    /// so a caller juggling both never has to match two different
    /// full-queue error types for the same underlying policy (§20:
    /// bounded, no exceptions, item handed back rather than dropped).
    pub fn push(&mut self, key: K, item: T) -> Result<(), (T, QueueFull)> {
        let queue = self.queues.entry(key.clone()).or_default();
        if queue.len() >= self.capacity_per_key {
            return Err((
                item,
                QueueFull {
                    capacity: self.capacity_per_key,
                },
            ));
        }
        if !self.rotation.contains(&key) {
            self.rotation.push_back(key);
        }
        queue.push_back(item);
        Ok(())
    }

    /// One full lap over the current rotation, returning the first
    /// item found and rotating that key to the back — so the *next*
    /// call resumes with whichever key comes after it, giving every
    /// key with pending work exactly one turn per lap regardless of
    /// how many items it's holding (this is what actually stops one
    /// peer's large backlog from getting picked more than once per
    /// lap just because it has more queued).
    pub fn pop(&mut self) -> Option<T> {
        for _ in 0..self.rotation.len() {
            let key = self.rotation.pop_front()?;
            let item = self.queues.get_mut(&key).and_then(VecDeque::pop_front);
            self.rotation.push_back(key);
            if item.is_some() {
                return item;
            }
        }
        None
    }

    pub fn is_empty(&self) -> bool {
        self.queues.values().all(VecDeque::is_empty)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::descriptor::ContentClass;
    use siar_domain::DeviceId;

    #[test]
    fn one_busy_peer_does_not_starve_a_quieter_one() {
        // §67's own example: a peer pushing a large file (many items)
        // alongside a peer sending occasional messages (few items).
        let mut q: RoundRobinFairQueue<DeviceId, &str> = RoundRobinFairQueue::new(100);
        let file_sender = DeviceId::new();
        let chatty_peer = DeviceId::new();

        for _ in 0..10 {
            q.push(file_sender, "chunk").unwrap();
        }
        q.push(chatty_peer, "hello").unwrap();

        // The quiet peer's one message comes out within the first two
        // pops, not stuck behind all ten of the busy peer's chunks.
        let first_two = [q.pop().unwrap(), q.pop().unwrap()];
        assert!(first_two.contains(&"hello"));
    }

    #[test]
    fn a_key_with_no_items_is_skipped_without_breaking_the_rotation() {
        let mut q: RoundRobinFairQueue<DeviceId, &str> = RoundRobinFairQueue::new(10);
        let a = DeviceId::new();
        let b = DeviceId::new();
        q.push(a, "a1").unwrap();
        q.push(b, "b1").unwrap();
        assert_eq!(q.pop(), Some("a1"));
        assert_eq!(q.pop(), Some("b1"));
        // `a`'s queue is empty now but still in rotation.
        q.push(b, "b2").unwrap();
        assert_eq!(q.pop(), Some("b2"));
        assert_eq!(q.pop(), None);
    }

    #[test]
    fn pushing_past_a_keys_own_capacity_rejects_without_touching_other_keys() {
        let mut q: RoundRobinFairQueue<DeviceId, u32> = RoundRobinFairQueue::new(1);
        let peer = DeviceId::new();
        let other = DeviceId::new();
        q.push(peer, 1).unwrap();
        let err = q.push(peer, 2).unwrap_err();
        assert_eq!(err.1.capacity, 1);
        // `other`'s own per-key capacity is untouched by `peer`'s.
        assert!(q.push(other, 99).is_ok());
    }

    #[test]
    fn extension_fairness_files_do_not_starve_messaging() {
        // §68's own example, using `ContentClass` as the "extension"
        // key.
        let mut q: RoundRobinFairQueue<ContentClass, &str> = RoundRobinFairQueue::new(50);
        for _ in 0..20 {
            q.push(ContentClass::File, "chunk").unwrap();
        }
        q.push(ContentClass::Text, "message").unwrap();

        let first_two = [q.pop().unwrap(), q.pop().unwrap()];
        assert!(first_two.contains(&"message"));
    }

    #[test]
    fn an_empty_queue_pops_none() {
        let mut q: RoundRobinFairQueue<DeviceId, u32> = RoundRobinFairQueue::new(10);
        assert!(q.is_empty());
        assert_eq!(q.pop(), None);
    }
}
