//! §14 "Messaging Recovery", §15 "Ambiguous Send Result", §16
//! "Delivery Receipt Recovery", §17 "Inbox Recovery".
//!
//! §15/§16/§17 all name the same underlying mechanism under three
//! different words — "recipient deduplicates," "sender applies
//! idempotently," "recipient deduplicates EventId/MessageId" — so
//! this module builds it once, as [`Deduplicator`], and reuses it for
//! all three rather than three near-identical hand-rolled `HashSet`
//! checks that could quietly drift apart. Uses this workspace's real
//! `siar_domain::MessageId` and `siar_event_log::ids::EventId` rather
//! than local stand-in newtypes, since both already exist as concrete
//! types elsewhere in this workspace and §17 names both by name.

use siar_domain::MessageId;
use siar_event_log::ids::EventId;
use std::collections::HashSet;
use std::hash::Hash;

/// A generic "have I already processed this id" ledger — the one
/// mechanism behind §15's send-retry dedup, §16's receipt-apply
/// idempotency, and §17's inbound `EventId`/`MessageId` dedup. A real
/// caller backs the underlying set with durable storage (the id must
/// survive the very crash this module exists to recover from); this
/// type only defines the admit-once semantics.
#[derive(Debug, Clone)]
pub struct Deduplicator<Id: Eq + Hash> {
    seen: HashSet<Id>,
}

impl<Id: Eq + Hash> Default for Deduplicator<Id> {
    fn default() -> Self {
        Self {
            seen: HashSet::new(),
        }
    }
}

impl<Id: Eq + Hash + Copy> Deduplicator<Id> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns `true` and records `id` the first time it's seen;
    /// returns `false` (without changing anything) every time after —
    /// exactly the "recipient deduplicates" / "sender applies
    /// idempotently" behavior §15-17 all require.
    pub fn try_admit(&mut self, id: Id) -> bool {
        self.seen.insert(id)
    }

    pub fn has_seen(&self, id: Id) -> bool {
        self.seen.contains(&id)
    }
}

/// §14's own state list, plus `Sent` — §14 lists only
/// `MessageCreated`/`MessageQueued`/`OutboxPending`, but §15's own text
/// ("crashes before local Sent marker") presupposes a `Sent` state
/// exists; included here so the chain has somewhere to land rather
/// than leaving §15's own scenario impossible to represent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OutboundMessageState {
    Created,
    Queued,
    OutboxPending,
    Sent,
}

/// One pending-at-crash-time outbound message, enough to drive §14's
/// restart logic.
#[derive(Debug, Clone, Copy)]
pub struct PendingOutboundMessage {
    pub id: MessageId,
    pub state: OutboundMessageState,
    pub expires_at_millis: Option<u64>,
    pub recipient_revoked: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiscardReason {
    Expired,
    Revoked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OutboundRecoveryAction {
    /// §14: "retry same MessageId. Do not create new MessageId" —
    /// this variant only ever carries the original id back, never a
    /// freshly generated one, which is what actually enforces that
    /// rule structurally rather than just documenting it.
    RetrySend(MessageId),
    Discard(MessageId, DiscardReason),
}

/// §14's own restart sequence: "reload pending outbox / validate
/// expiry/revocation / retry same MessageId." Every message not yet
/// `Sent` gets exactly one of the two actions above — nothing is ever
/// silently dropped without a reason, and nothing is ever retried with
/// a new id.
pub fn reload_outbox(
    pending: &[PendingOutboundMessage],
    now_millis: u64,
) -> Vec<OutboundRecoveryAction> {
    pending
        .iter()
        .filter(|m| m.state != OutboundMessageState::Sent)
        .map(|m| {
            if m.recipient_revoked {
                OutboundRecoveryAction::Discard(m.id, DiscardReason::Revoked)
            } else if m.expires_at_millis.is_some_and(|exp| exp <= now_millis) {
                OutboundRecoveryAction::Discard(m.id, DiscardReason::Expired)
            } else {
                OutboundRecoveryAction::RetrySend(m.id)
            }
        })
        .collect()
}

/// §16: "recipient may resend receipt, sender applies idempotently."
/// Built directly on [`Deduplicator`] — a receipt for a `MessageId`
/// already marked delivered is a safe no-op, not an error and not a
/// double-apply.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReceiptApplyOutcome {
    Applied,
    AlreadyApplied,
}

pub fn apply_receipt(
    applied: &mut Deduplicator<MessageId>,
    message_id: MessageId,
) -> ReceiptApplyOutcome {
    if applied.try_admit(message_id) {
        ReceiptApplyOutcome::Applied
    } else {
        ReceiptApplyOutcome::AlreadyApplied
    }
}

/// §17's own durable-receive sequence, verbatim stage list (`validate
/// → persist → commit → then ACK`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InboundReceiveState {
    Validated,
    Persisted,
    Committed,
    Acked,
}

/// Real transition validation for §17's sequence — same linear,
/// no-skipping discipline as this crate's other state machines.
pub fn can_transition(from: InboundReceiveState, to: InboundReceiveState) -> bool {
    use InboundReceiveState::*;
    matches!(
        (from, to),
        (Validated, Persisted) | (Persisted, Committed) | (Committed, Acked)
    )
}

/// §17: "Crash before ACK: sender retries, recipient deduplicates
/// EventId/MessageId." The recipient side of that, built on the same
/// [`Deduplicator`] used for §15/§16 — a re-delivery of an
/// already-committed message (identified by its `EventId`, per §17's
/// own naming) is recognized and skipped rather than persisted twice,
/// regardless of whether the ACK for the first delivery ever made it
/// back to the sender.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InboundReceiveOutcome {
    New,
    Duplicate,
}

pub fn receive_inbound(
    seen: &mut Deduplicator<EventId>,
    event_id: EventId,
) -> InboundReceiveOutcome {
    if seen.try_admit(event_id) {
        InboundReceiveOutcome::New
    } else {
        InboundReceiveOutcome::Duplicate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pending(
        state: OutboundMessageState,
        expires_at_millis: Option<u64>,
        revoked: bool,
    ) -> PendingOutboundMessage {
        PendingOutboundMessage {
            id: MessageId::new(),
            state,
            expires_at_millis,
            recipient_revoked: revoked,
        }
    }

    #[test]
    fn already_sent_messages_are_not_reloaded_at_all() {
        let msgs = [pending(OutboundMessageState::Sent, None, false)];
        assert!(reload_outbox(&msgs, 0).is_empty());
    }

    #[test]
    fn pending_message_retries_with_its_original_message_id() {
        // §14's exact rule: "Do not create new MessageId."
        let msg = pending(OutboundMessageState::OutboxPending, None, false);
        let actions = reload_outbox(&[msg], 0);
        assert_eq!(actions, vec![OutboundRecoveryAction::RetrySend(msg.id)]);
    }

    #[test]
    fn expired_pending_message_is_discarded_not_retried() {
        let msg = pending(OutboundMessageState::Queued, Some(100), false);
        let actions = reload_outbox(&[msg], 200); // now_millis past expiry
        assert_eq!(
            actions,
            vec![OutboundRecoveryAction::Discard(
                msg.id,
                DiscardReason::Expired
            )]
        );
    }

    #[test]
    fn revoked_recipient_message_is_discarded_even_if_not_expired() {
        let msg = pending(OutboundMessageState::Created, None, true);
        let actions = reload_outbox(&[msg], 0);
        assert_eq!(
            actions,
            vec![OutboundRecoveryAction::Discard(
                msg.id,
                DiscardReason::Revoked
            )]
        );
    }

    #[test]
    fn not_yet_expired_pending_message_still_retries() {
        let msg = pending(OutboundMessageState::OutboxPending, Some(1_000), false);
        let actions = reload_outbox(&[msg], 500); // now_millis before expiry
        assert_eq!(actions, vec![OutboundRecoveryAction::RetrySend(msg.id)]);
    }

    #[test]
    fn resent_message_id_is_deduplicated_on_the_recipient_side() {
        // §15's exact scenario.
        let id = MessageId::new();
        let mut seen: Deduplicator<MessageId> = Deduplicator::new();
        assert!(seen.try_admit(id)); // first delivery
        assert!(!seen.try_admit(id)); // resend after ambiguous send result
    }

    #[test]
    fn receipt_is_applied_exactly_once_even_if_resent() {
        let id = MessageId::new();
        let mut applied = Deduplicator::new();
        assert_eq!(
            apply_receipt(&mut applied, id),
            ReceiptApplyOutcome::Applied
        );
        assert_eq!(
            apply_receipt(&mut applied, id),
            ReceiptApplyOutcome::AlreadyApplied
        );
        assert_eq!(
            apply_receipt(&mut applied, id),
            ReceiptApplyOutcome::AlreadyApplied
        );
    }

    #[test]
    fn inbound_receive_sequence_is_linear_no_skipping_to_acked() {
        use InboundReceiveState::*;
        assert!(can_transition(Validated, Persisted));
        assert!(can_transition(Persisted, Committed));
        assert!(can_transition(Committed, Acked));
        assert!(!can_transition(Validated, Acked));
        assert!(!can_transition(Persisted, Validated));
    }

    #[test]
    fn redelivered_event_after_missed_ack_is_recognized_as_duplicate() {
        // §17's exact scenario: sender retries after never seeing the
        // ACK, recipient must recognize it already committed this
        // EventId rather than persisting it twice.
        let event_id = EventId::new();
        let mut seen: Deduplicator<EventId> = Deduplicator::new();
        assert_eq!(
            receive_inbound(&mut seen, event_id),
            InboundReceiveOutcome::New
        );
        assert_eq!(
            receive_inbound(&mut seen, event_id),
            InboundReceiveOutcome::Duplicate
        );
    }
}
