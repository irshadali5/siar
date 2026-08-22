//! The DTN bundle envelope — next.md §29–30, §38.
//!
//! Reuses `siar_domain::MessageId` as the bundle's identity rather than
//! inventing a separate `EnvelopeId` — next.md §79's idempotency
//! requirement ("Database idempotency ensures... one logical message")
//! means the DTN layer and the ordinary direct-delivery layer need to
//! agree on what "the same message" means; a second ID type would just
//! be one more thing to keep in sync with `MessageId` instead of simply
//! being it.
//!
//! `created_at`/`expires_at` are opaque `u64` ticks in whatever
//! monotonic unit the caller chooses, not a wall-clock timestamp —
//! next.md §96: "Don't assume accurate Internet time... use device
//! monotonic time... be resilient to incorrect wall clocks." Nothing in
//! this crate reads a real clock, so nothing here can be fooled by one;
//! `is_expired` takes `now` as a parameter, same pattern
//! `siar_calls::jitter::JitterBuffer` and
//! `siar_transport_ble::reassembly::ReassemblyBuffer` already use for
//! the same reason.

use siar_domain::{DeviceId, MessageId};

/// Re-exported from `siar_domain` — see that module's doc comment for
/// why this moved out of this crate in the pass that added
/// `siar-protocol`'s `MeshEnvelope`. Still resolves at
/// `siar_dtn::bundle::MessagePriority`, unchanged for every existing
/// caller.
pub use siar_domain::MessagePriority;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeshBundle {
    pub id: MessageId,
    /// Added alongside `apps/emergency-node`'s forwarding logic — a
    /// relay deciding whether/where to forward a stored bundle needs
    /// to know who it's ultimately for, same as `MeshEnvelope`'s own
    /// `destination` field on the wire (`siar-protocol::mesh`). Every
    /// existing construction site (this crate's own tests,
    /// `siar-testkit::mesh_sim`) predates this field and needed a
    /// value added — see each site's own comment for what it uses.
    pub destination: DeviceId,
    /// Carried through unchanged from the originating `MeshEnvelope` —
    /// added alongside `destination` for the same reason: a relay
    /// reconstructing a `MeshEnvelope` to forward needs every field
    /// that struct requires, and `siar-protocol::mesh`'s own doc
    /// comment is explicit that a receiver is "responsible for
    /// actually checking this against `ciphertext`" — silently
    /// zeroing or dropping it on forward would break that contract for
    /// whoever receives the forwarded copy, not just leave a field
    /// blank.
    pub payload_hash: [u8; 32],
    /// Already-encrypted bytes — next.md §28/§76: intermediates forward
    /// ciphertext, never see plaintext. This crate has no decrypt path
    /// and no reason to; it never needs to look inside this.
    pub ciphertext: Vec<u8>,
    pub priority: MessagePriority,
    pub hop_limit: u8,
    pub replication_budget: u8,
    pub created_at: u64,
    pub expires_at: u64,
}

impl MeshBundle {
    pub fn is_expired(&self, now: u64) -> bool {
        now >= self.expires_at
    }

    /// next.md §30: "Every forward decrements: TTL = TTL - 1. At zero:
    /// drop." Returns the bundle with `hop_limit` reduced by one if
    /// it's still forwardable, or `None` if this was the last hop —
    /// callers should treat `None` exactly like next.md's "drop", not
    /// forward a bundle with an already-zero hop limit.
    pub fn forwarded(mut self) -> Option<Self> {
        if self.hop_limit == 0 {
            return None;
        }
        self.hop_limit -= 1;
        Some(self)
    }

    /// next.md §38: handing a copy of this bundle to a new carrier
    /// consumes one unit of replication budget. Returns `true` (and
    /// consumes one unit) if a copy may still be handed out; `false` if
    /// the budget is already exhausted, in which case the caller must
    /// not replicate further — direct delivery to a known destination
    /// is a separate concern this budget doesn't gate (next.md §37's
    /// "direct delivery" class).
    pub fn try_consume_replication(&mut self) -> bool {
        if self.replication_budget == 0 {
            return false;
        }
        self.replication_budget -= 1;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bundle(hop_limit: u8, replication_budget: u8) -> MeshBundle {
        MeshBundle {
            id: MessageId::new(),
            destination: DeviceId::new(),
            payload_hash: [0u8; 32],
            ciphertext: vec![1, 2, 3],
            priority: MessagePriority::Normal,
            hop_limit,
            replication_budget,
            created_at: 0,
            expires_at: 100,
        }
    }

    #[test]
    fn is_expired_true_once_now_reaches_expires_at() {
        let bundle = bundle(4, 2);
        assert!(!bundle.is_expired(99));
        assert!(bundle.is_expired(100));
        assert!(bundle.is_expired(101));
    }

    #[test]
    fn forwarded_decrements_hop_limit_until_it_hits_zero() {
        let bundle = bundle(2, 2);
        let bundle = bundle.forwarded().expect("hop_limit 2 -> 1 should still forward");
        assert_eq!(bundle.hop_limit, 1);
        let bundle = bundle.forwarded().expect("hop_limit 1 -> 0 should still forward");
        assert_eq!(bundle.hop_limit, 0);
        assert!(bundle.forwarded().is_none(), "hop_limit already 0 must not forward further");
    }

    #[test]
    fn try_consume_replication_stops_at_zero() {
        let mut bundle = bundle(4, 1);
        assert!(bundle.try_consume_replication());
        assert_eq!(bundle.replication_budget, 0);
        assert!(!bundle.try_consume_replication());
    }

    #[test]
    fn emergency_gets_the_highest_hop_limit_and_replication_budget() {
        assert_eq!(MessagePriority::Emergency.default_hop_limit(), 8);
        assert_eq!(MessagePriority::Emergency.default_replication_budget(), 8);
        assert!(MessagePriority::Emergency.default_hop_limit() >= MessagePriority::Normal.default_hop_limit());
        assert!(MessagePriority::Emergency.default_replication_budget() >= MessagePriority::Normal.default_replication_budget());
    }
}
