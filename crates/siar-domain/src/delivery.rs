//! Message delivery lifecycle (plan.md §15).
//!
//! Modeled as an enum rather than three independent booleans (plan.md
//! §122) so "sent AND failed" simply cannot be constructed — the compiler
//! rules it out instead of a runtime assertion having to catch it.
//!
//! `CarriedByPeers` is next.md §61's addition, for the DTN path
//! (Phase 4): a bundle handed to a relay peer over the mesh has left
//! this device, but next.md §62 is explicit that this "does NOT mean
//! delivered to recipient — the UI must not show a misleading
//! double-check mark." It sits where `Sent` used to be the only option
//! after `Sending`, not replacing `Sent` — a direct transport send still
//! goes `Sending -> Sent` unchanged; only a DTN handoff goes through
//! `CarriedByPeers` first.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeliveryState {
    /// Created locally, not yet handed to the outbox.
    Local,
    /// Persisted in the outbox, waiting for a send attempt.
    Queued,
    /// A send attempt is in flight.
    Sending,
    /// Transport accepted the bytes (does not imply persistence on the
    /// recipient — see plan.md §46 on the ACK/delivery/read distinction).
    Sent,
    /// next.md §61: handed to at least one DTN relay peer, `copies`
    /// tracking how many currently carry it (next.md §38's replication
    /// budget bounds how high this can go). Not delivery — see this
    /// module's top doc comment.
    CarriedByPeers { copies: u8 },
    /// Recipient persisted the message locally.
    Delivered,
    /// Recipient has read the message.
    Read,
    /// Every retry attempt failed; sits in the outbox for manual/backoff retry.
    Failed,
    /// next.md §95: this bundle's DTN expiry was reached before it
    /// could be delivered — distinct from `Failed`, which is a transport
    /// send attempt failing, not a store-carry-forward bundle timing out
    /// while still technically "in flight" on the mesh.
    Expired,
}

impl DeliveryState {
    /// Legal forward transitions. Deliberately conservative: no jumping
    /// straight from `Queued` to `Read`, no un-reading a message, etc.
    pub fn can_transition_to(self, next: DeliveryState) -> bool {
        use DeliveryState::*;
        matches!(
            (self, next),
            (Local, Queued)
                | (Queued, Sending)
                | (Sending, Sent)
                | (Sending, Failed)
                | (Sending, CarriedByPeers { .. })
                | (Failed, Sending) // retry
                | (Sent, Delivered)
                | (Delivered, Read)
                // Copy count changing (a further peer picks up a copy,
                // next.md §38) is still logically "the same state," not
                // a transition into a different one — but it needs to
                // be a legal `(CarriedByPeers{..}, CarriedByPeers{..})`
                // pair here since the struct-like variant's data differs.
                | (CarriedByPeers { .. }, CarriedByPeers { .. })
                // A carrying peer reaches the recipient directly, or
                // becomes an Internet gateway that completes delivery —
                // either way the mesh path's own concept of "Sent"
                // never applied, so this goes straight to `Delivered`.
                | (CarriedByPeers { .. }, Delivered)
                | (CarriedByPeers { .. }, Expired)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use DeliveryState::*;

    #[test]
    fn happy_path_is_allowed() {
        assert!(Local.can_transition_to(Queued));
        assert!(Queued.can_transition_to(Sending));
        assert!(Sending.can_transition_to(Sent));
        assert!(Sent.can_transition_to(Delivered));
        assert!(Delivered.can_transition_to(Read));
    }

    #[test]
    fn cannot_unread_or_skip_states() {
        assert!(!Read.can_transition_to(Delivered));
        assert!(!Local.can_transition_to(Read));
        assert!(!Queued.can_transition_to(Delivered));
    }

    #[test]
    fn failed_messages_can_retry() {
        assert!(Sending.can_transition_to(Failed));
        assert!(Failed.can_transition_to(Sending));
    }

    #[test]
    fn mesh_carry_path_reaches_delivered_without_going_through_sent() {
        assert!(Sending.can_transition_to(CarriedByPeers { copies: 1 }));
        assert!(CarriedByPeers { copies: 1 }.can_transition_to(CarriedByPeers { copies: 2 }));
        assert!(CarriedByPeers { copies: 2 }.can_transition_to(Delivered));
        // Still not allowed to skip straight past CarriedByPeers into
        // Read, same "no skipping states" discipline as the direct path.
        assert!(!CarriedByPeers { copies: 1 }.can_transition_to(Read));
    }

    #[test]
    fn mesh_carry_can_expire_instead_of_delivering() {
        assert!(CarriedByPeers { copies: 3 }.can_transition_to(Expired));
        assert!(
            !Sent.can_transition_to(Expired),
            "a directly-sent message has no DTN expiry concept"
        );
    }
}
