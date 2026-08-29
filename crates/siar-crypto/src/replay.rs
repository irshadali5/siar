//! Replay protection (Part 28 §16).
//!
//! §16's own requirement is specific: "the same ciphertext arriving
//! through direct, relay, DTN, and multipath must remain one logical
//! message." That rules out a naive strictly-increasing-counter check
//! (`reject if counter <= highest_seen`) — this workspace's own
//! multipath/DTN design means a duplicate of an *older* message can
//! legitimately arrive after a newer one (a DTN carrier delivering a
//! bundle that raced a faster direct path, or a multipath duplicate
//! arriving on a slower link after the primary copy already landed).
//! `ReplayGuard` instead keeps a bounded sliding window of counters
//! already accepted and only rejects exact duplicates within it, or
//! anything so far behind the window that it can no longer be verified
//! either way — matching §16's own listed inputs (message IDs, ratchet
//! counters, bounded replay windows, security epochs).

use std::collections::{HashMap, HashSet};

use siar_domain::{ConversationId, DeviceId};

use crate::envelope::SecureMessageEnvelope;
use crate::epoch::SecurityEpoch;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ReplayError {
    #[error("counter already seen for this conversation/sender/epoch")]
    Duplicate,
    #[error("counter is older than this guard's replay window and can no longer be verified")]
    TooOld,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct StreamKey {
    conversation: ConversationId,
    sender_device: DeviceId,
    epoch: SecurityEpoch,
}

#[derive(Default)]
struct StreamState {
    highest_seen: u64,
    // Only counters within `[highest_seen - window, highest_seen]` are
    // ever kept — see `prune`. Bounded by `window_size`, not by message
    // volume, so this can't grow without limit for a long-lived stream.
    seen_in_window: HashSet<u64>,
}

/// One guard instance is meant to be scoped to a single local device's
/// receive path (it has no notion of "which device is receiving" itself
/// — that's implicit in which `ReplayGuard` instance a caller uses).
/// Tracks state per `(conversation, sender_device, epoch)` so that a
/// new epoch — e.g. after a device revocation — naturally starts that
/// stream's counter space fresh, since a genuinely new epoch's
/// `StreamKey` has never been seen before.
pub struct ReplayGuard {
    window_size: u64,
    streams: HashMap<StreamKey, StreamState>,
}

impl ReplayGuard {
    /// `window_size` is how far behind the highest counter seen so far
    /// a counter may still legitimately arrive (e.g. via a slow DTN
    /// path) and be accepted. Too small and legitimate delayed/
    /// multipath deliveries get rejected as `TooOld`; too large and the
    /// per-stream memory/dedup-set cost grows. The spec gives no
    /// concrete number — this is a caller-tunable policy, not a fixed
    /// constant this crate bakes in.
    pub fn new(window_size: u64) -> Self {
        Self {
            window_size,
            streams: HashMap::new(),
        }
    }

    /// Checks whether `envelope` is new (not a replay within this
    /// guard's window) and, if so, records it as seen. Does **not**
    /// verify authenticity — call this alongside (either before or
    /// after) `decrypt_envelope`, not as a substitute for it. Checking
    /// before decrypt avoids doing AEAD work for a ciphertext that
    /// would be rejected as a replay anyway; checking after avoids
    /// letting an attacker who can't forge a valid envelope still
    /// influence this guard's state. Either order is safe from a
    /// correctness standpoint since this guard makes no confidentiality
    /// claim of its own — this crate checks before, since it's cheaper.
    pub fn check_and_record(&mut self, envelope: &SecureMessageEnvelope) -> Result<(), ReplayError> {
        let key = StreamKey {
            conversation: envelope.conversation,
            sender_device: envelope.sender_device,
            epoch: envelope.epoch,
        };
        let state = self.streams.entry(key).or_default();
        let counter = envelope.counter;

        if counter + self.window_size < state.highest_seen {
            return Err(ReplayError::TooOld);
        }

        if !state.seen_in_window.insert(counter) {
            return Err(ReplayError::Duplicate);
        }

        if counter > state.highest_seen {
            state.highest_seen = counter;
            let window_size = self.window_size;
            state
                .seen_in_window
                .retain(|&c| c + window_size >= state.highest_seen);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::epoch::SecurityEpoch;
    use bytes::Bytes;
    use siar_domain::MessageId;

    fn envelope_with_counter(
        conversation: ConversationId,
        sender: DeviceId,
        epoch: SecurityEpoch,
        counter: u64,
    ) -> SecureMessageEnvelope {
        SecureMessageEnvelope {
            conversation,
            sender_device: sender,
            message_id: MessageId::new(),
            epoch,
            counter,
            ciphertext: Bytes::new(),
            authentication: crate::envelope::AuthenticationTag([0u8; 16]),
        }
    }

    #[test]
    fn first_message_is_accepted() {
        let mut guard = ReplayGuard::new(16);
        let e = envelope_with_counter(ConversationId::new(), DeviceId::new(), SecurityEpoch::zero(), 0);
        assert!(guard.check_and_record(&e).is_ok());
    }

    #[test]
    fn exact_duplicate_is_rejected() {
        let mut guard = ReplayGuard::new(16);
        let conversation = ConversationId::new();
        let sender = DeviceId::new();
        let epoch = SecurityEpoch::zero();

        let e = envelope_with_counter(conversation, sender, epoch, 5);
        assert!(guard.check_and_record(&e).is_ok());
        // Same conversation/sender/epoch/counter arriving a second time
        // (e.g. delivered via both a direct path and a DTN bundle).
        let e_again = envelope_with_counter(conversation, sender, epoch, 5);
        assert_eq!(guard.check_and_record(&e_again), Err(ReplayError::Duplicate));
    }

    #[test]
    fn reordered_delivery_within_window_is_accepted_once() {
        let mut guard = ReplayGuard::new(16);
        let conversation = ConversationId::new();
        let sender = DeviceId::new();
        let epoch = SecurityEpoch::zero();

        // Message 10 arrives first (e.g. over a fast direct link),
        // then message 3 arrives late (e.g. over a slower DTN path).
        // Both are legitimate, distinct messages and must both be
        // accepted despite arriving out of counter order.
        let ten = envelope_with_counter(conversation, sender, epoch, 10);
        let three = envelope_with_counter(conversation, sender, epoch, 3);
        assert!(guard.check_and_record(&ten).is_ok());
        assert!(guard.check_and_record(&three).is_ok());
    }

    #[test]
    fn counter_far_behind_the_window_is_too_old() {
        let mut guard = ReplayGuard::new(4);
        let conversation = ConversationId::new();
        let sender = DeviceId::new();
        let epoch = SecurityEpoch::zero();

        let recent = envelope_with_counter(conversation, sender, epoch, 100);
        assert!(guard.check_and_record(&recent).is_ok());

        // 100 - 4 = 96 is the oldest counter still inside the window;
        // 90 is well outside it.
        let stale = envelope_with_counter(conversation, sender, epoch, 90);
        assert_eq!(guard.check_and_record(&stale), Err(ReplayError::TooOld));
    }

    #[test]
    fn different_epochs_are_independent_streams() {
        let mut guard = ReplayGuard::new(16);
        let conversation = ConversationId::new();
        let sender = DeviceId::new();

        let e_epoch0 = envelope_with_counter(conversation, sender, SecurityEpoch(0), 5);
        let e_epoch1 = envelope_with_counter(conversation, sender, SecurityEpoch(1), 5);
        // Same counter, but a new epoch (e.g. after a device
        // revocation advanced it) starts a fresh counter space —
        // counter 5 under epoch 1 is not a replay of counter 5 under
        // epoch 0.
        assert!(guard.check_and_record(&e_epoch0).is_ok());
        assert!(guard.check_and_record(&e_epoch1).is_ok());
    }

    #[test]
    fn different_senders_are_independent_streams() {
        let mut guard = ReplayGuard::new(16);
        let conversation = ConversationId::new();
        let epoch = SecurityEpoch::zero();

        let alice = envelope_with_counter(conversation, DeviceId::new(), epoch, 0);
        let bob = envelope_with_counter(conversation, DeviceId::new(), epoch, 0);
        assert!(guard.check_and_record(&alice).is_ok());
        assert!(guard.check_and_record(&bob).is_ok());
    }
}
