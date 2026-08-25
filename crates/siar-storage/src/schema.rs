//! Schema (plan.md §20). Phase 1 subset: just `messages` and `outbox` —
//! receipts/sessions arrive with the crates that actually need them
//! (Phase 2+) rather than being pre-declared unused. `contacts` (see
//! this file's own section below) landed with the desktop app's
//! persistent contact book.

use crate::StorageError;
use stoolap::Database;

pub(crate) fn apply(db: &Database) -> Result<(), StorageError> {
    db.execute(
        "CREATE TABLE IF NOT EXISTS messages (
            message_id      TEXT PRIMARY KEY,
            conversation_id TEXT NOT NULL,
            sender_device   TEXT NOT NULL,
            sequence        BIGINT NOT NULL,
            timestamp_millis BIGINT NOT NULL,
            delivery_state  TEXT NOT NULL,
            payload         BLOB NOT NULL
        )",
        (),
    )
    .map_err(StorageError::from_stoolap)?;

    // Plan.md §20: messages(conversation_id, sequence) is the timeline
    // query's index — every conversation-open call hits this.
    db.execute(
        "CREATE INDEX IF NOT EXISTS idx_messages_conversation_sequence
         ON messages (conversation_id, sequence)",
        (),
    )
    .map_err(StorageError::from_stoolap)?;

    // Plan.md §17: the outbox is the transactional-delivery mechanism.
    // `next_attempt_at` drives the retry scheduler's polling query.
    db.execute(
        "CREATE TABLE IF NOT EXISTS outbox (
            message_id      TEXT PRIMARY KEY,
            next_attempt_at BIGINT NOT NULL,
            attempts        INTEGER NOT NULL DEFAULT 0,
            -- Denormalized recipient info (Phase-2 stand-in — plan.md §20's
            -- `contacts`/`conversation_members` tables don't exist yet, so
            -- the retry scheduler needs *something* to resend to. Once
            -- those tables land, this becomes a conversation_id lookup
            -- instead and this column goes away).
            peer_ticket_hex TEXT NOT NULL
        )",
        (),
    )
    .map_err(StorageError::from_stoolap)?;

    db.execute(
        "CREATE INDEX IF NOT EXISTS idx_outbox_next_attempt
         ON outbox (next_attempt_at)",
        (),
    )
    .map_err(StorageError::from_stoolap)?;

    // plan.md §81: blob garbage collection is reference-counted — a blob
    // is only deleted once nothing (a message, a thumbnail, a draft)
    // references it anymore. Phase 4 scope: the count exists and is
    // incremented/decremented; the actual GC sweep that acts on
    // `ref_count = 0` is future work once there's more than one
    // reference source to make GC worth running.
    db.execute(
        "CREATE TABLE IF NOT EXISTS blobs (
            blob_hash TEXT PRIMARY KEY,
            ciphertext BLOB NOT NULL,
            ref_count INTEGER NOT NULL DEFAULT 1
        )",
        (),
    )
    .map_err(StorageError::from_stoolap)?;

    // plan.md §20's group tables — see group_repo.rs. `group_state` is
    // one row per conversation (just the epoch counter); membership is
    // normalized out into `group_members` rather than a serialized blob
    // column so a single member's role/join-epoch can be queried without
    // deserializing the whole group.
    db.execute(
        "CREATE TABLE IF NOT EXISTS group_state (
            conversation_id TEXT PRIMARY KEY,
            epoch           BIGINT NOT NULL
        )",
        (),
    )
    .map_err(StorageError::from_stoolap)?;

    db.execute(
        "CREATE TABLE IF NOT EXISTS group_members (
            conversation_id  TEXT NOT NULL,
            account_id       TEXT NOT NULL,
            role             TEXT NOT NULL,
            joined_at_epoch  BIGINT NOT NULL,
            PRIMARY KEY (conversation_id, account_id)
        )",
        (),
    )
    .map_err(StorageError::from_stoolap)?;

    db.execute(
        "CREATE INDEX IF NOT EXISTS idx_group_members_conversation
         ON group_members (conversation_id)",
        (),
    )
    .map_err(StorageError::from_stoolap)?;

    // plan.md §20's "conversations/contacts" tables, added for the
    // desktop app's persistent contact book (see contact_repo.rs). One
    // row per known peer *device* — `account_id` is duplicated across a
    // peer's multiple devices on purpose, same denormalization
    // `group_members` already uses, rather than a separate `accounts`
    // table this pass doesn't need yet. `key_package_b64` is nullable:
    // a 1:1-only contact (never added to a group) may never have had
    // one recorded.
    db.execute(
        "CREATE TABLE IF NOT EXISTS contacts (
            device_id        TEXT PRIMARY KEY,
            account_id       TEXT NOT NULL,
            display_name     TEXT NOT NULL,
            ticket_text      TEXT NOT NULL,
            key_package_b64  TEXT,
            added_at_millis  BIGINT NOT NULL
        )",
        (),
    )
    .map_err(StorageError::from_stoolap)?;

    db.execute(
        "CREATE INDEX IF NOT EXISTS idx_contacts_account
         ON contacts (account_id)",
        (),
    )
    .map_err(StorageError::from_stoolap)?;

    Ok(())
}
