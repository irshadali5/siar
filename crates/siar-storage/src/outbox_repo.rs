//! Transactional outbox (plan.md §17, §33): persist-before-send, with
//! exponential-backoff retry bookkeeping.

use crate::{blob_codec::encode_blob, message_repo::StoredMessage, StorageError};
use siar_domain::MessageId;
use std::sync::Arc;
use stoolap::Database;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct OutboxOperation {
    pub message_id: MessageId,
    pub next_attempt_at_millis: i64,
    pub attempts: u32,
    /// See schema.rs's note on this column: a Phase-2 stand-in for a real
    /// `conversation_members` lookup. Opaque to this crate — it's
    /// `siar-messaging`'s `PeerTicket::encode()` output, round-tripped
    /// verbatim so the retry scheduler knows who to resend to.
    pub peer_ticket_hex: String,
}

pub trait OutboxRepository {
    /// Inserts the message row and its outbox row in a single transaction
    /// (plan.md §16: "persist before sending" — this is the primitive
    /// that guarantees it; there is no code path that writes `messages`
    /// without also writing `outbox` in the same commit).
    fn enqueue(&self, message: &StoredMessage, peer_ticket_hex: &str) -> Result<(), StorageError>;

    /// Operations due for a send attempt, ordered by `next_attempt_at`
    /// (plan.md §33's retry scheduler polls this).
    fn due(&self, now_millis: i64, limit: usize) -> Result<Vec<OutboxOperation>, StorageError>;

    /// Send succeeded and was durably persisted by the recipient (or, for
    /// local bookkeeping, the transport accepted it) — remove from the
    /// outbox.
    fn complete(&self, message_id: MessageId) -> Result<(), StorageError>;

    /// Transport accepted the bytes but no `DeliveryAck` has come back
    /// yet (plan.md §46) — push the next retry out without counting this
    /// as a failed attempt (`attempts` is unchanged; a real send failure
    /// goes through `record_failure` instead, which does bump it).
    fn reschedule(
        &self,
        message_id: MessageId,
        next_attempt_at_millis: i64,
    ) -> Result<(), StorageError>;

    /// Send attempt failed — bump `attempts` and schedule the next try.
    fn record_failure(
        &self,
        message_id: MessageId,
        next_attempt_at_millis: i64,
    ) -> Result<(), StorageError>;
}

pub struct StoolapOutboxRepository {
    db: Arc<Database>,
}

impl StoolapOutboxRepository {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }
}

impl OutboxRepository for StoolapOutboxRepository {
    fn enqueue(&self, message: &StoredMessage, peer_ticket_hex: &str) -> Result<(), StorageError> {
        // BEGIN/COMMIT as literal SQL (stoolap's documented transaction
        // pattern) rather than a bespoke Rust Transaction type, since that
        // literal form is what's confirmed in stoolap's own docs.
        self.db
            .execute("BEGIN", ())
            .map_err(StorageError::from_stoolap)?;

        let insert_result = self.db.execute(
            "INSERT INTO messages
                (message_id, conversation_id, sender_device, seq_num,
                 timestamp_millis, delivery_state, payload)
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
            (
                message.message_id.as_uuid().to_string(),
                message.conversation_id.as_uuid().to_string(),
                message.sender_device.as_uuid().to_string(),
                message.sequence as i64,
                message.timestamp_millis as i64,
                "queued".to_string(),
                encode_blob(&message.payload),
            ),
        );
        if let Err(e) = insert_result {
            let _ = self.db.execute("ROLLBACK", ());
            return Err(StorageError::from_stoolap(e));
        }

        let outbox_result = self.db.execute(
            "INSERT INTO outbox (message_id, next_attempt_at, attempts, peer_ticket_hex)
             VALUES ($1, $2, 0, $3)",
            (
                message.message_id.as_uuid().to_string(),
                message.timestamp_millis as i64,
                peer_ticket_hex.to_string(),
            ),
        );
        if let Err(e) = outbox_result {
            let _ = self.db.execute("ROLLBACK", ());
            return Err(StorageError::from_stoolap(e));
        }

        self.db
            .execute("COMMIT", ())
            .map_err(StorageError::from_stoolap)?;
        Ok(())
    }

    fn due(&self, now_millis: i64, limit: usize) -> Result<Vec<OutboxOperation>, StorageError> {
        let rows = self
            .db
            .query(
                "SELECT message_id, next_attempt_at, attempts, peer_ticket_hex
                 FROM outbox
                 WHERE next_attempt_at <= $1
                 ORDER BY next_attempt_at
                 LIMIT $2",
                (now_millis, limit as i64),
            )
            .map_err(StorageError::from_stoolap)?;

        let mut out = Vec::with_capacity(limit);
        for row in rows {
            let row = row.map_err(StorageError::from_stoolap)?;
            let message_id: String = row.get(0).map_err(StorageError::from_stoolap)?;
            let next_attempt_at: i64 = row.get(1).map_err(StorageError::from_stoolap)?;
            let attempts: i64 = row.get(2).map_err(StorageError::from_stoolap)?;
            let peer_ticket_hex: String = row.get(3).map_err(StorageError::from_stoolap)?;
            out.push(OutboxOperation {
                message_id: MessageId::from_uuid(
                    Uuid::parse_str(&message_id)
                        .map_err(|e| StorageError::MalformedId(e.to_string()))?,
                ),
                next_attempt_at_millis: next_attempt_at,
                attempts: attempts as u32,
                peer_ticket_hex,
            });
        }
        Ok(out)
    }

    fn complete(&self, message_id: MessageId) -> Result<(), StorageError> {
        self.db
            .execute(
                "DELETE FROM outbox WHERE message_id = $1",
                (message_id.as_uuid().to_string(),),
            )
            .map_err(StorageError::from_stoolap)?;
        Ok(())
    }

    fn reschedule(
        &self,
        message_id: MessageId,
        next_attempt_at_millis: i64,
    ) -> Result<(), StorageError> {
        self.db
            .execute(
                "UPDATE outbox SET next_attempt_at = $1 WHERE message_id = $2",
                (next_attempt_at_millis, message_id.as_uuid().to_string()),
            )
            .map_err(StorageError::from_stoolap)?;
        Ok(())
    }

    fn record_failure(
        &self,
        message_id: MessageId,
        next_attempt_at_millis: i64,
    ) -> Result<(), StorageError> {
        self.db
            .execute(
                "UPDATE outbox
                 SET attempts = attempts + 1, next_attempt_at = $1
                 WHERE message_id = $2",
                (next_attempt_at_millis, message_id.as_uuid().to_string()),
            )
            .map_err(StorageError::from_stoolap)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message_repo::StoredMessage;
    use siar_domain::{ConversationId, DeliveryState, DeviceId};

    fn sample() -> StoredMessage {
        StoredMessage {
            message_id: MessageId::new(),
            conversation_id: ConversationId::new(),
            sender_device: DeviceId::new(),
            sequence: 1,
            timestamp_millis: 1000,
            delivery_state: DeliveryState::Queued,
            payload: vec![9, 9, 9],
        }
    }

    #[test]
    fn enqueue_writes_both_tables_atomically() {
        let db = crate::open_in_memory().unwrap();
        let outbox = StoolapOutboxRepository::new(db.clone());
        let msg = sample();
        outbox.enqueue(&msg, "deadbeef").unwrap();

        let due = outbox.due(i64::MAX, 10).unwrap();
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].message_id, msg.message_id);
        assert_eq!(due[0].attempts, 0);
        assert_eq!(due[0].peer_ticket_hex, "deadbeef");
    }

    #[test]
    fn complete_removes_from_outbox() {
        let db = crate::open_in_memory().unwrap();
        let outbox = StoolapOutboxRepository::new(db.clone());
        let msg = sample();
        outbox.enqueue(&msg, "deadbeef").unwrap();
        outbox.complete(msg.message_id).unwrap();

        let due = outbox.due(i64::MAX, 10).unwrap();
        assert!(due.is_empty());
    }

    #[test]
    fn reschedule_moves_next_attempt_without_bumping_attempts() {
        let db = crate::open_in_memory().unwrap();
        let outbox = StoolapOutboxRepository::new(db.clone());
        let msg = sample();
        outbox.enqueue(&msg, "deadbeef").unwrap();
        outbox.reschedule(msg.message_id, 9_000).unwrap();

        let due = outbox.due(i64::MAX, 10).unwrap();
        assert_eq!(due[0].attempts, 0);
        assert_eq!(due[0].next_attempt_at_millis, 9_000);
    }

    #[test]
    fn record_failure_bumps_attempts_and_reschedules() {
        let db = crate::open_in_memory().unwrap();
        let outbox = StoolapOutboxRepository::new(db.clone());
        let msg = sample();
        outbox.enqueue(&msg, "deadbeef").unwrap();
        outbox.record_failure(msg.message_id, 5_000).unwrap();

        let due = outbox.due(i64::MAX, 10).unwrap();
        assert_eq!(due[0].attempts, 1);
        assert_eq!(due[0].next_attempt_at_millis, 5_000);
    }
}
