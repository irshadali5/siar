//! Bounded fragment reassembly — next.md §25: "Never hold arbitrary
//! unfinished fragments indefinitely."
//!
//! One [`ReassemblyBuffer`] is meant to belong to one BLE connection
//! (one peer link), not to be shared across multiple peers — nothing
//! here disambiguates *which peer* a `transfer_id` came from, only
//! which in-flight transfer on this link it belongs to. Mixing
//! transfers from different peers into one buffer would let one peer's
//! `transfer_id` collide with another's; that's a connection-management
//! decision for whatever owns the GATT link (the not-yet-built Android
//! boundary this crate's types are meant for), not something this pure
//! data structure can enforce on its own.
//!
//! Boundedness, deliberately without a clock: like
//! `siar-calls::jitter::JitterBuffer`, this counts *concurrent
//! in-flight transfers*, not elapsed time, so it stays pure and
//! testable without mocking time. When a new transfer would exceed
//! `capacity`, the oldest still-incomplete transfer is evicted and
//! lost — next.md §69's DoS-resistance concern (a peer opening many
//! transfers and never finishing any of them) is what bounds this, not
//! a timeout; a caller that also wants a wall-clock timeout can layer
//! one on top by calling [`ReassemblyBuffer::expire`] on its own timer,
//! the same "caller decides what real time maps to" split
//! `JitterBuffer::flush` uses.

use std::collections::VecDeque;

use crate::fragment::BleFragment;

pub struct ReassemblyBuffer {
    capacity: usize,
    /// Ordered oldest-first purely by *first-fragment-arrival* order,
    /// so eviction (`capacity` exceeded) and `expire` both have a
    /// well-defined "oldest" to act on — a `HashMap` alone can't offer
    /// that ordering.
    transfers: VecDeque<PartialTransfer>,
}

struct PartialTransfer {
    transfer_id: u32,
    fragment_count: u16,
    received: Vec<Option<Vec<u8>>>,
    received_len: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReassemblyOutcome {
    /// Still waiting on more fragments for this transfer.
    Incomplete,
    /// This was the last missing fragment — payloads concatenated in
    /// `fragment_index` order into the original (still-encrypted)
    /// envelope bytes. The completed transfer is removed from the
    /// buffer; a repeated fragment for the same `transfer_id` arriving
    /// after this starts a fresh transfer rather than erroring — BLE
    /// links are lossy enough that "the sender retransmitted a fragment
    /// we'd already used to complete the message" is a plausible
    /// duplicate, not a protocol violation worth failing on.
    Complete(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ReassemblyError {
    #[error("fragment claims fragment_count {claimed}, but transfer {transfer_id} is already tracking fragment_count {tracked}")]
    FragmentCountMismatch { transfer_id: u32, tracked: u16, claimed: u16 },
}

impl ReassemblyBuffer {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity >= 1, "a zero-capacity reassembly buffer can never complete a multi-fragment transfer");
        Self { capacity, transfers: VecDeque::with_capacity(capacity) }
    }

    pub fn len(&self) -> usize {
        self.transfers.len()
    }

    pub fn is_empty(&self) -> bool {
        self.transfers.is_empty()
    }

    /// Inserts one fragment. A duplicate fragment (same `transfer_id`
    /// and `fragment_index` as one already received) is accepted and
    /// ignored rather than erroring — same lossy-link reasoning as
    /// `ReassemblyOutcome::Complete`'s doc comment.
    pub fn insert(&mut self, fragment: BleFragment) -> Result<ReassemblyOutcome, ReassemblyError> {
        let position = self.transfers.iter().position(|t| t.transfer_id == fragment.transfer_id);

        let index = match position {
            Some(index) => {
                let tracked = self.transfers[index].fragment_count;
                if tracked != fragment.fragment_count {
                    return Err(ReassemblyError::FragmentCountMismatch {
                        transfer_id: fragment.transfer_id,
                        tracked,
                        claimed: fragment.fragment_count,
                    });
                }
                index
            }
            None => {
                if self.transfers.len() >= self.capacity {
                    // Oldest still-incomplete transfer loses its spot —
                    // see this file's top doc comment.
                    self.transfers.pop_front();
                }
                self.transfers.push_back(PartialTransfer {
                    transfer_id: fragment.transfer_id,
                    fragment_count: fragment.fragment_count,
                    received: vec![None; fragment.fragment_count as usize],
                    received_len: 0,
                });
                self.transfers.len() - 1
            }
        };

        let transfer = &mut self.transfers[index];
        let slot = &mut transfer.received[fragment.fragment_index as usize];
        if slot.is_none() {
            *slot = Some(fragment.payload);
            transfer.received_len += 1;
        }

        if transfer.received_len == transfer.received.len() {
            let transfer = self.transfers.remove(index).expect("index came from this same deque");
            let mut assembled = Vec::new();
            for piece in transfer.received {
                assembled.extend(piece.expect("received_len == len means every slot is Some"));
            }
            Ok(ReassemblyOutcome::Complete(assembled))
        } else {
            Ok(ReassemblyOutcome::Incomplete)
        }
    }

    /// Drops every still-incomplete transfer, discarding whatever
    /// fragments they'd collected. For a caller layering a wall-clock
    /// timeout on top (see this file's top doc comment) — calling this
    /// unconditionally on a timer is coarser than expiring individual
    /// transfers by age, but doesn't need this buffer to know about
    /// real time to offer.
    pub fn expire_all_incomplete(&mut self) {
        self.transfers.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fragment(transfer_id: u32, fragment_index: u16, fragment_count: u16, byte: u8) -> BleFragment {
        BleFragment { protocol: 1, transfer_id, fragment_index, fragment_count, payload: vec![byte] }
    }

    #[test]
    fn single_fragment_transfer_completes_immediately() {
        let mut buffer = ReassemblyBuffer::new(4);
        let outcome = buffer.insert(fragment(1, 0, 1, b'a')).unwrap();
        assert_eq!(outcome, ReassemblyOutcome::Complete(vec![b'a']));
        assert!(buffer.is_empty());
    }

    #[test]
    fn multi_fragment_transfer_completes_in_index_order_regardless_of_arrival_order() {
        let mut buffer = ReassemblyBuffer::new(4);
        assert_eq!(buffer.insert(fragment(1, 2, 3, b'c')).unwrap(), ReassemblyOutcome::Incomplete);
        assert_eq!(buffer.insert(fragment(1, 0, 3, b'a')).unwrap(), ReassemblyOutcome::Incomplete);
        let outcome = buffer.insert(fragment(1, 1, 3, b'b')).unwrap();
        assert_eq!(outcome, ReassemblyOutcome::Complete(vec![b'a', b'b', b'c']));
    }

    #[test]
    fn duplicate_fragment_is_ignored_not_an_error() {
        let mut buffer = ReassemblyBuffer::new(4);
        buffer.insert(fragment(1, 0, 2, b'a')).unwrap();
        // Same fragment arrives again (retransmit) before the transfer
        // completes — accepted, doesn't advance received_len twice.
        assert_eq!(buffer.insert(fragment(1, 0, 2, b'a')).unwrap(), ReassemblyOutcome::Incomplete);
        assert_eq!(buffer.insert(fragment(1, 1, 2, b'b')).unwrap(), ReassemblyOutcome::Complete(vec![b'a', b'b']));
    }

    #[test]
    fn mismatched_fragment_count_for_a_known_transfer_is_an_error() {
        let mut buffer = ReassemblyBuffer::new(4);
        buffer.insert(fragment(1, 0, 3, b'a')).unwrap();
        let err = buffer.insert(fragment(1, 1, 5, b'b')).unwrap_err();
        assert_eq!(err, ReassemblyError::FragmentCountMismatch { transfer_id: 1, tracked: 3, claimed: 5 });
    }

    #[test]
    fn exceeding_capacity_evicts_the_oldest_incomplete_transfer() {
        let mut buffer = ReassemblyBuffer::new(2);
        buffer.insert(fragment(1, 0, 2, b'a')).unwrap(); // transfer 1: incomplete, oldest
        buffer.insert(fragment(2, 0, 2, b'a')).unwrap(); // transfer 2: incomplete
        buffer.insert(fragment(3, 0, 2, b'a')).unwrap(); // transfer 3: pushes out transfer 1
        assert_eq!(buffer.len(), 2);

        // Transfer 1's remaining fragment now starts a *new* transfer 1
        // from scratch, since its old partial state was evicted.
        let outcome = buffer.insert(fragment(1, 1, 2, b'b')).unwrap();
        assert_eq!(outcome, ReassemblyOutcome::Incomplete);
    }

    #[test]
    fn expire_all_incomplete_clears_everything_pending() {
        let mut buffer = ReassemblyBuffer::new(4);
        buffer.insert(fragment(1, 0, 2, b'a')).unwrap();
        buffer.insert(fragment(2, 0, 2, b'a')).unwrap();
        assert_eq!(buffer.len(), 2);
        buffer.expire_all_incomplete();
        assert!(buffer.is_empty());
    }
}
