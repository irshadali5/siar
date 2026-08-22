//! Message repository (plan.md §87: a domain trait + a concrete
//! implementation, so nothing above this crate needs to know SQL exists).

use crate::{blob_codec::{decode_blob, encode_blob}, StorageError};
use siar_domain::{ConversationId, DeliveryState, DeviceId, MessageId};
use stoolap::Database;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct StoredMessage {
    pub message_id: MessageId,
    pub conversation_id: ConversationId,
    pub sender_device: DeviceId,
    pub sequence: u64,
    pub timestamp_millis: u64,
    pub delivery_state: DeliveryState,
    /// Application-E2EE ciphertext (siar-crypto output) — storage never
    /// sees plaintext.
    pub payload: Vec<u8>,
}

pub trait MessageRepository {
    fn insert(&self, message: &StoredMessage) -> Result<(), StorageError>;

    /// Idempotent insert for the receive path (plan.md §70: receiving the
    /// same envelope 5 times must produce exactly one persisted message).
    /// Returns `Ok(true)` if this call actually inserted a new row,
    /// `Ok(false)` if `message.message_id` already existed and this was a
    /// no-op. Callers on the receive path (e.g. deciding whether to send
    /// a fresh delivery ACK) branch on that return value.
    fn insert_if_new(&self, message: &StoredMessage) -> Result<bool, StorageError>;

    /// Single-message lookup — the retry scheduler uses this to
    /// reconstruct a `WireMessage` from what's already in `messages`
    /// rather than keeping a second copy of the ciphertext in `outbox`.
    fn get(&self, message_id: MessageId) -> Result<Option<StoredMessage>, StorageError>;

    /// Newest-first page of a conversation's timeline, walking backward
    /// from `before` (plan.md §64 — never load the full history).
    fn timeline(
        &self,
        conversation: ConversationId,
        before: Option<u64>,
        limit: usize,
    ) -> Result<Vec<StoredMessage>, StorageError>;

    fn update_delivery_state(
        &self,
        message_id: MessageId,
        state: DeliveryState,
    ) -> Result<(), StorageError>;
}

pub struct StoolapMessageRepository {
    db: Arc<Database>,
}

impl StoolapMessageRepository {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }
}

impl MessageRepository for StoolapMessageRepository {
    fn insert(&self, message: &StoredMessage) -> Result<(), StorageError> {
        self.db
            .execute(
                "INSERT INTO messages
                    (message_id, conversation_id, sender_device, sequence,
                     timestamp_millis, delivery_state, payload)
                 VALUES ($1, $2, $3, $4, $5, $6, $7)",
                (
                    message.message_id.as_uuid().to_string(),
                    message.conversation_id.as_uuid().to_string(),
                    message.sender_device.as_uuid().to_string(),
                    message.sequence as i64,
                    message.timestamp_millis as i64,
                    delivery_state_to_str(message.delivery_state),
                    encode_blob(&message.payload),
                ),
            )
            .map_err(StorageError::from_stoolap)?;
        Ok(())
    }

    fn insert_if_new(&self, message: &StoredMessage) -> Result<bool, StorageError> {
        // BEGIN/COMMIT/ROLLBACK as literal SQL, matching the pattern
        // already used in outbox_repo.rs (stoolap's documented
        // transaction form) — the existence check and the insert must
        // commit together so two concurrent deliveries of the same
        // envelope can't both see "absent" and both insert.
        self.db.execute("BEGIN", ()).map_err(StorageError::from_stoolap)?;

        let already_exists = {
            let mut rows = self
                .db
                .query(
                    "SELECT 1 FROM messages WHERE message_id = $1",
                    (message.message_id.as_uuid().to_string(),),
                )
                .map_err(StorageError::from_stoolap)?;
            rows.next().is_some()
        };

        if already_exists {
            self.db.execute("COMMIT", ()).map_err(StorageError::from_stoolap)?;
            return Ok(false);
        }

        let insert_result = self.db.execute(
            "INSERT INTO messages
                (message_id, conversation_id, sender_device, sequence,
                 timestamp_millis, delivery_state, payload)
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
            (
                message.message_id.as_uuid().to_string(),
                message.conversation_id.as_uuid().to_string(),
                message.sender_device.as_uuid().to_string(),
                message.sequence as i64,
                message.timestamp_millis as i64,
                delivery_state_to_str(message.delivery_state),
                encode_blob(&message.payload),
            ),
        );
        if let Err(e) = insert_result {
            let _ = self.db.execute("ROLLBACK", ());
            return Err(StorageError::from_stoolap(e));
        }

        self.db.execute("COMMIT", ()).map_err(StorageError::from_stoolap)?;
        Ok(true)
    }

    fn get(&self, message_id: MessageId) -> Result<Option<StoredMessage>, StorageError> {
        let rows = self
            .db
            .query(
                "SELECT message_id, conversation_id, sender_device, sequence,
                        timestamp_millis, delivery_state, payload
                 FROM messages
                 WHERE message_id = $1",
                (message_id.as_uuid().to_string(),),
            )
            .map_err(StorageError::from_stoolap)?;

        for row in rows {
            let row = row.map_err(StorageError::from_stoolap)?;
            return Ok(Some(row_to_stored_message(&row)?));
        }
        Ok(None)
    }

    fn timeline(
        &self,
        conversation: ConversationId,
        before: Option<u64>,
        limit: usize,
    ) -> Result<Vec<StoredMessage>, StorageError> {
        let before_sequence = before.unwrap_or(u64::MAX) as i64;
        let rows = self
            .db
            .query(
                "SELECT message_id, conversation_id, sender_device, sequence,
                        timestamp_millis, delivery_state, payload
                 FROM messages
                 WHERE conversation_id = $1 AND sequence < $2
                 ORDER BY sequence DESC
                 LIMIT $3",
                (
                    conversation.as_uuid().to_string(),
                    before_sequence,
                    limit as i64,
                ),
            )
            .map_err(StorageError::from_stoolap)?;

        let mut out = Vec::with_capacity(limit);
        for row in rows {
            let row = row.map_err(StorageError::from_stoolap)?;
            out.push(row_to_stored_message(&row)?);
        }
        Ok(out)
    }

    fn update_delivery_state(
        &self,
        message_id: MessageId,
        state: DeliveryState,
    ) -> Result<(), StorageError> {
        self.db
            .execute(
                "UPDATE messages SET delivery_state = $1 WHERE message_id = $2",
                (
                    delivery_state_to_str(state),
                    message_id.as_uuid().to_string(),
                ),
            )
            .map_err(StorageError::from_stoolap)?;
        Ok(())
    }
}

fn row_to_stored_message(row: &stoolap::ResultRow) -> Result<StoredMessage, StorageError> {
    let message_id: String = row.get(0).map_err(StorageError::from_stoolap)?;
    let conversation_id: String = row.get(1).map_err(StorageError::from_stoolap)?;
    let sender_device: String = row.get(2).map_err(StorageError::from_stoolap)?;
    let sequence: i64 = row.get(3).map_err(StorageError::from_stoolap)?;
    let timestamp_millis: i64 = row.get(4).map_err(StorageError::from_stoolap)?;
    let delivery_state: String = row.get(5).map_err(StorageError::from_stoolap)?;
    let payload_b64: String = row.get(6).map_err(StorageError::from_stoolap)?;

    Ok(StoredMessage {
        message_id: MessageId::from_uuid(parse_uuid(&message_id)?),
        conversation_id: ConversationId::from_uuid(parse_uuid(&conversation_id)?),
        sender_device: DeviceId::from_uuid(parse_uuid(&sender_device)?),
        sequence: sequence as u64,
        timestamp_millis: timestamp_millis as u64,
        delivery_state: str_to_delivery_state(&delivery_state)?,
        payload: decode_blob(&payload_b64)?,
    })
}

fn parse_uuid(s: &str) -> Result<Uuid, StorageError> {
    Uuid::parse_str(s).map_err(|e| StorageError::MalformedId(e.to_string()))
}

// Returns an owned `String` rather than `&'static str`: `CarriedByPeers`
// carries a `copies: u8` payload (next.md §61) that has to be serialized
// into the column, not just tagged, and `Expired` (next.md §95) is its own
// distinct terminal state from `Failed` (see delivery.rs's doc comment on
// why the two are not merged). All existing call sites already called
// `.to_string()` on the old `&'static str` result, so this is a
// zero-behavior-change signature widening at every call site.
fn delivery_state_to_str(state: DeliveryState) -> String {
    match state {
        DeliveryState::Local => "local".to_string(),
        DeliveryState::Queued => "queued".to_string(),
        DeliveryState::Sending => "sending".to_string(),
        DeliveryState::Sent => "sent".to_string(),
        DeliveryState::CarriedByPeers { copies } => format!("carried_by_peers:{copies}"),
        DeliveryState::Delivered => "delivered".to_string(),
        DeliveryState::Read => "read".to_string(),
        DeliveryState::Failed => "failed".to_string(),
        DeliveryState::Expired => "expired".to_string(),
    }
}

fn str_to_delivery_state(s: &str) -> Result<DeliveryState, StorageError> {
    if let Some(copies_str) = s.strip_prefix("carried_by_peers:") {
        let copies: u8 = copies_str
            .parse()
            .map_err(|_| StorageError::MalformedId(format!("bad carried_by_peers copies '{s}'")))?;
        return Ok(DeliveryState::CarriedByPeers { copies });
    }
    Ok(match s {
        "local" => DeliveryState::Local,
        "queued" => DeliveryState::Queued,
        "sending" => DeliveryState::Sending,
        "sent" => DeliveryState::Sent,
        "delivered" => DeliveryState::Delivered,
        "read" => DeliveryState::Read,
        "failed" => DeliveryState::Failed,
        "expired" => DeliveryState::Expired,
        other => return Err(StorageError::MalformedId(format!("unknown delivery_state '{other}'"))),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_then_timeline_round_trips() {
        let db = crate::open_in_memory().unwrap();
        let repo = StoolapMessageRepository::new(db);

        let conversation = ConversationId::new();
        let msg = StoredMessage {
            message_id: MessageId::new(),
            conversation_id: conversation,
            sender_device: DeviceId::new(),
            sequence: 1,
            timestamp_millis: 1_700_000_000_000,
            delivery_state: DeliveryState::Local,
            payload: vec![1, 2, 3],
        };
        repo.insert(&msg).unwrap();

        let page = repo.timeline(conversation, None, 50).unwrap();
        assert_eq!(page.len(), 1);
        assert_eq!(page[0].message_id, msg.message_id);
        assert_eq!(page[0].payload, vec![1, 2, 3]);
    }

    #[test]
    fn get_finds_an_inserted_message_and_none_for_unknown_id() {
        let db = crate::open_in_memory().unwrap();
        let repo = StoolapMessageRepository::new(db);

        let msg = StoredMessage {
            message_id: MessageId::new(),
            conversation_id: ConversationId::new(),
            sender_device: DeviceId::new(),
            sequence: 1,
            timestamp_millis: 42,
            delivery_state: DeliveryState::Local,
            payload: vec![7],
        };
        repo.insert(&msg).unwrap();

        let found = repo.get(msg.message_id).unwrap().unwrap();
        assert_eq!(found.payload, vec![7]);
        assert!(repo.get(MessageId::new()).unwrap().is_none());
    }

    #[test]
    fn insert_if_new_is_idempotent_under_duplicate_delivery() {
        let db = crate::open_in_memory().unwrap();
        let repo = StoolapMessageRepository::new(db);

        let msg = StoredMessage {
            message_id: MessageId::new(),
            conversation_id: ConversationId::new(),
            sender_device: DeviceId::new(),
            sequence: 1,
            timestamp_millis: 0,
            delivery_state: DeliveryState::Delivered,
            payload: vec![4, 5, 6],
        };

        // plan.md §70: same envelope delivered 5 times -> one row.
        let mut inserted_count = 0;
        for _ in 0..5 {
            if repo.insert_if_new(&msg).unwrap() {
                inserted_count += 1;
            }
        }
        assert_eq!(inserted_count, 1);

        let page = repo.timeline(msg.conversation_id, None, 10).unwrap();
        assert_eq!(page.len(), 1);
    }

    #[test]
    fn update_delivery_state_persists() {
        let db = crate::open_in_memory().unwrap();
        let repo = StoolapMessageRepository::new(db);

        let conversation = ConversationId::new();
        let msg = StoredMessage {
            message_id: MessageId::new(),
            conversation_id: conversation,
            sender_device: DeviceId::new(),
            sequence: 1,
            timestamp_millis: 0,
            delivery_state: DeliveryState::Local,
            payload: vec![],
        };
        repo.insert(&msg).unwrap();
        repo.update_delivery_state(msg.message_id, DeliveryState::Sent)
            .unwrap();

        let page = repo.timeline(conversation, None, 1).unwrap();
        assert_eq!(page[0].delivery_state, DeliveryState::Sent);
    }
}
