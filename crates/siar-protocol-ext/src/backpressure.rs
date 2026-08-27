//! §20 "Backpressure": "Backpressure is mandatory, not optional
//! optimization." "A slow Bluetooth or DTN route must never produce
//! unlimited queued file chunks → out-of-memory."

use std::collections::VecDeque;

/// §20's own pipeline (`Extension producer → bounded queue → session
/// scheduler → transport`) — this is the "bounded queue" stage: a
/// plain `VecDeque` with a hard capacity, real rejection on overflow
/// (not silent unbounded growth), so a slow downstream transport
/// applies real backpressure to its producer instead of this crate
/// buffering forever on its behalf.
pub struct BoundedQueue<T> {
    items: VecDeque<T>,
    capacity: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("queue is full at capacity {capacity}")]
pub struct QueueFull {
    pub capacity: usize,
}

impl<T> BoundedQueue<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            items: VecDeque::with_capacity(capacity.min(1024)),
            capacity,
        }
    }

    /// Real rejection, not silent unbounded growth — the producer gets
    /// its item back on failure so nothing is silently dropped either;
    /// what the producer does with a rejected item (retry later, drop
    /// it, apply its own backpressure upstream) is the producer's
    /// decision, not this queue's.
    pub fn try_push(&mut self, item: T) -> Result<(), (T, QueueFull)> {
        if self.items.len() >= self.capacity {
            return Err((
                item,
                QueueFull {
                    capacity: self.capacity,
                },
            ));
        }
        self.items.push_back(item);
        Ok(())
    }

    pub fn pop(&mut self) -> Option<T> {
        self.items.pop_front()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn is_full(&self) -> bool {
        self.items.len() >= self.capacity
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pushing_within_capacity_succeeds() {
        let mut queue = BoundedQueue::new(3);
        assert!(queue.try_push(1).is_ok());
        assert!(queue.try_push(2).is_ok());
        assert_eq!(queue.len(), 2);
    }

    #[test]
    fn pushing_past_capacity_is_rejected_and_returns_the_item() {
        let mut queue = BoundedQueue::new(1);
        queue.try_push(1).unwrap();
        let result = queue.try_push(2);
        assert_eq!(result, Err((2, QueueFull { capacity: 1 })));
        assert_eq!(queue.len(), 1); // the rejected item never entered the queue
    }

    #[test]
    fn popping_makes_room_for_another_push() {
        let mut queue = BoundedQueue::new(1);
        queue.try_push(1).unwrap();
        assert!(queue.is_full());
        assert_eq!(queue.pop(), Some(1));
        assert!(!queue.is_full());
        assert!(queue.try_push(2).is_ok());
    }

    #[test]
    fn fifo_order_is_preserved() {
        let mut queue = BoundedQueue::new(3);
        queue.try_push('a').unwrap();
        queue.try_push('b').unwrap();
        queue.try_push('c').unwrap();
        assert_eq!(queue.pop(), Some('a'));
        assert_eq!(queue.pop(), Some('b'));
        assert_eq!(queue.pop(), Some('c'));
    }
}
