//! Local persistence. One sqlite file per identity, under the data dir.
//! Nothing here ever leaves the machine — there's no server, so "storage"
//! just means "don't lose chat history, contacts, or settings across
//! restarts."
//!
//! v2 changes from the original single-table version:
//!   - `contacts` gains a `username` and a `state` state-machine column
//!     (`pending_out` / `pending_in` / `accepted` / `blocked`) to back the
//!     request/accept flow in `net::contacts`.
//!   - `messages` gains file-transfer columns, used when `kind = 'file'`.
//!   - a small `settings` key/value table replaces one-off files for things
//!     like "has the user confirmed writing down their seed phrase" and
//!     the locally cached claimed username.

use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
use std::sync::Mutex;

/// A single sqlite connection isn't `Sync` on its own (rusqlite uses
/// interior mutability that assumes single-threaded access). `Store` is
/// shared as `Arc<Store>` across concurrent tokio tasks — most visibly,
/// `net::contacts::ContactProtocol` needs `Send + Sync` to satisfy iroh's
/// `ProtocolHandler` bound, since `accept()` can run concurrently for
/// simultaneous incoming connections. Wrapping the connection in a
/// `Mutex` makes `Store` (and therefore `Arc<Store>`) `Sync` regardless.
/// sqlite itself serializes writes anyway, so this isn't giving up real
/// concurrency — it's just making the compiler's guarantee match what was
/// already true.
pub struct Store {
    conn: Mutex<Connection>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContactState {
    /// We sent a contact request and are waiting on them to accept.
    PendingOut,
    /// They sent us a request; sitting in our requests inbox.
    PendingIn,
    /// Mutually accepted — DMs are allowed.
    Accepted,
    /// We've blocked them; their traffic is accepted-then-discarded at the
    /// protocol layer, no notification leak.
    Blocked,
}

impl ContactState {
    fn as_str(self) -> &'static str {
        match self {
            ContactState::PendingOut => "pending_out",
            ContactState::PendingIn => "pending_in",
            ContactState::Accepted => "accepted",
            ContactState::Blocked => "blocked",
        }
    }

    fn parse(s: &str) -> Result<Self> {
        Ok(match s {
            "pending_out" => ContactState::PendingOut,
            "pending_in" => ContactState::PendingIn,
            "accepted" => ContactState::Accepted,
            "blocked" => ContactState::Blocked,
            other => anyhow::bail!("unknown contact state {other:?} in database"),
        })
    }
}

#[derive(Debug, Clone)]
pub struct Contact {
    pub endpoint_id: String, // hex-encoded EndpointId, stored as text
    pub username: Option<String>,
    pub alias: String,
    pub state: ContactState,
    pub requested_at: i64,
    /// Out-of-band identity confirmation (QR scan or manual `EndpointId`
    /// compare) — the actual defense against impersonation, since there's
    /// no CA and a username claim alone is only optimistically-unique
    /// (§3). Purely a local, manually-set flag: nothing about accepting a
    /// contact request implies verification, and this never travels over
    /// the wire — it's not something the other side can set for you.
    pub verified: bool,
    /// BLAKE3 hex hash of this contact's current display picture in the
    /// local `iroh-blobs` store, if we've received a `Body::AvatarUpdate`
    /// from them — `None` until then. The PNG bytes themselves aren't
    /// fetched eagerly when this is set; see
    /// `app::fetch_contact_avatar`/`download_history` for the
    /// download-on-demand + local-cache path.
    pub avatar_hash: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Conversation {
    Dm(String),   // hex EndpointId of the peer
    Room(String), // room name
}

/// Persisted in `settings` under `"theme_mode"` — see `Store::theme_mode`.
/// `System` is the default and, per `ui::css`'s module doc, is handled
/// almost entirely in CSS (`prefers-color-scheme`) rather than by this
/// app polling the OS — `Light`/`Dark` are the explicit-override cases
/// that actually need this value read back on the Rust side.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    System,
    Light,
    Dark,
}

impl ThemeMode {
    pub fn as_str(self) -> &'static str {
        match self {
            ThemeMode::System => "system",
            ThemeMode::Light => "light",
            ThemeMode::Dark => "dark",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "system" => Some(ThemeMode::System),
            "light" => Some(ThemeMode::Light),
            "dark" => Some(ThemeMode::Dark),
            _ => None,
        }
    }
}

/// A second, independent theming axis from `ThemeMode`: not light-vs-
/// dark, but the app's whole visual language. `Regular` is this app's
/// normal Signal/WhatsApp-influenced look (bubbles, sans-serif, and
/// whatever `ThemeMode` says about light/dark). `HackerGreen`/`HackerRed`
/// are fixed-palette monospace/terminal styles — greenscreen and
/// redscreen respectively — that don't have a light/dark variant of
/// their own; picking either one overrides `ThemeMode` rather than
/// combining with it, the same way picking a specific `ThemeMode::Light`/
/// `Dark` already overrides `System`. Persisted separately
/// (`"theme_style"`, see `Store::theme_style`) so switching back to
/// `Regular` restores whatever `ThemeMode` was set to, instead of losing
/// it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeStyle {
    Regular,
    HackerGreen,
    HackerRed,
}

impl ThemeStyle {
    pub fn as_str(self) -> &'static str {
        match self {
            ThemeStyle::Regular => "regular",
            ThemeStyle::HackerGreen => "hacker-green",
            ThemeStyle::HackerRed => "hacker-red",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "regular" => Some(ThemeStyle::Regular),
            "hacker-green" => Some(ThemeStyle::HackerGreen),
            "hacker-red" => Some(ThemeStyle::HackerRed),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum MessageKind {
    Text,
    File {
        name: String,
        hash: String,
        size_bytes: u64,
        compressed: bool,
    },
}

#[derive(Debug, Clone)]
pub struct StoredMessage {
    pub sender_name: String,
    pub body: String, // text content, or the file name for File kind
    pub sent_unix_ms: i64,
    pub outgoing: bool,
    pub kind: MessageKind,
    /// Absolute expiry (disappearing messages) — see
    /// `net::conv_docs::DmSettings::disappearing_ttl_secs` (the synced
    /// per-conversation policy) and `Store::sweep_expired_messages` (the
    /// thing that actually deletes rows once past this).
    pub expires_at_unix_ms: Option<i64>,
    /// The wire `Envelope::id` this row came from — `None` for rows
    /// written before this column existed (see the migration comment in
    /// `Store::open`). Needed to target a `Reaction`/`Edit`/`Delete` at
    /// this specific message; a row with `None` here simply can't be
    /// reacted to/edited/deleted from the UI.
    pub envelope_id: Option<u64>,
    /// `Some(edited_at_ms)` if `body` reflects a later `Body::Edit`
    /// rather than the original text — UI shows an "edited" marker when
    /// this is `Some`.
    pub edited_at_unix_ms: Option<i64>,
    /// Tombstoned via `Body::Delete` — `body` at that point is replaced
    /// with a placeholder by `Store::mark_message_deleted` rather than
    /// physically dropping the row, so the conversation's shape/ordering
    /// doesn't shift under everyone else's already-rendered view.
    pub deleted: bool,
    /// `(sender_id hex, emoji)` pairs, one per person who's reacted —
    /// loaded alongside the message list by `recent_messages` (a
    /// separate batched query, not a SQL join/GROUP_CONCAT — simpler to
    /// get right than parsing a concatenated string back apart, and this
    /// runs once per conversation load, not once per message).
    pub reactions: Vec<(String, String)>,
    /// `Envelope::id` of the message this one replies to, if any — set at
    /// send time from whatever the composer's "replying to" state was
    /// pointing at (see `ui::chat`'s reply-preview bar), or on receipt
    /// straight from the wire envelope. `None` for the ordinary case.
    /// The reply target's own sender/snippet is resolved locally by
    /// looking up that envelope id in the same conversation's already-
    /// loaded rows — not re-fetched, not carried a second time on the
    /// wire.
    pub reply_to_envelope_id: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CallDirection {
    Incoming,
    Outgoing,
}

impl CallDirection {
    fn as_str(self) -> &'static str {
        match self {
            Self::Incoming => "incoming",
            Self::Outgoing => "outgoing",
        }
    }
    fn from_str(s: &str) -> Self {
        if s == "outgoing" {
            Self::Outgoing
        } else {
            Self::Incoming
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CallOutcome {
    Completed,
    Declined,
    Missed,
    Failed,
}

impl CallOutcome {
    fn as_str(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::Declined => "declined",
            Self::Missed => "missed",
            Self::Failed => "failed",
        }
    }
    fn from_str(s: &str) -> Self {
        match s {
            "declined" => Self::Declined,
            "missed" => Self::Missed,
            "failed" => Self::Failed,
            _ => Self::Completed,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CallLogEntry {
    pub peer_id: String,
    pub peer_name: String,
    pub direction: CallDirection,
    pub outcome: CallOutcome,
    pub started_at_ms: i64,
    pub duration_secs: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StatusEntry {
    pub peer_id: String,
    pub peer_name: String,
    pub text: String,
    pub posted_at_ms: i64,
    pub expires_at_ms: i64,
    /// blake3 hex hash of the attached image's canonical PNG bytes in the
    /// local iroh-blobs store, if this status has one — see
    /// `media::decode_status_image`. `None` for a text-only status.
    pub image_hash: Option<String>,
    pub image_size_bytes: Option<u64>,
    /// blake3 hex hash of the attached codec-tagged video clip blob (see
    /// `net::calls::video::encode_clip`), if this status has one.
    pub video_hash: Option<String>,
    pub video_size_bytes: Option<u64>,
    /// blake3 hex hash of the attached voice clip's Opus blob (see
    /// `net::calls::audio::record_and_encode_voice_clip`), if this
    /// status has one.
    pub audio_hash: Option<String>,
    pub audio_size_bytes: Option<u64>,
}

/// `ALTER TABLE ... ADD COLUMN`, tolerating "it's already there" — the
/// only realistic error on a table that already has the column, since a
/// missing table or a genuinely malformed statement would fail in ways
/// this doesn't try to swallow (the error message check is deliberately
/// narrow).
fn add_column_if_missing(conn: &Connection, table: &str, column: &str, decl: &str) -> Result<()> {
    match conn.execute(
        &format!("ALTER TABLE {table} ADD COLUMN {column} {decl}"),
        [],
    ) {
        Ok(_) => Ok(()),
        Err(rusqlite::Error::SqliteFailure(_, Some(msg)))
            if msg.contains("duplicate column name") =>
        {
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}

fn now_unix_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

impl Store {
    /// `key` encrypts `messenger.db` at rest via SQLCipher (see
    /// `Cargo.toml`'s `rusqlite` feature comment) — pass
    /// `identity::storage_key(data_dir)`. The `PRAGMA key` below must be
    /// the very first thing run on a fresh connection; SQLCipher treats
    /// every statement before it as operating on the *plaintext* page
    /// format and everything after as ciphertext, so it can't be folded
    /// into the batch of `CREATE TABLE`s below or reordered after them.
    pub fn open(data_dir: &Path, key: &[u8; 32]) -> Result<Self> {
        std::fs::create_dir_all(data_dir)?;
        let conn = Connection::open(data_dir.join("messenger.db"))?;
        let hex_key = data_encoding::HEXLOWER.encode(key);
        // Raw-key form (`x'...'`) rather than a passphrase string: skips
        // SQLCipher's own PBKDF2 key-stretching step, which would be pure
        // overhead here since `key` is already a uniformly random 256-bit
        // HKDF output, not a low-entropy human passphrase that needs
        // stretching.
        conn.execute_batch(&format!("PRAGMA key = \"x'{hex_key}'\";"))
            .map_err(|e| {
                anyhow::anyhow!(
                    "opening encrypted messenger.db failed ({e}) — if this is an \
                     existing plaintext database from before encryption was added, \
                     it needs a one-time `sqlcipher_export`-based migration rather \
                     than opening directly"
                )
            })?;

        conn.execute_batch(
            r#"
            PRAGMA journal_mode = WAL;
            PRAGMA foreign_keys = ON;

            CREATE TABLE IF NOT EXISTS contacts (
                endpoint_id   TEXT PRIMARY KEY,
                username      TEXT,
                alias         TEXT NOT NULL,
                state         TEXT NOT NULL DEFAULT 'accepted',
                requested_at  INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS messages (
                id                INTEGER PRIMARY KEY AUTOINCREMENT,
                conversation_kind TEXT NOT NULL,   -- 'dm' | 'room'
                conversation_key  TEXT NOT NULL,   -- endpoint_id hex, or room name
                sender_id         TEXT NOT NULL,   -- hex EndpointId of the actual sender
                sender_name       TEXT NOT NULL,
                body              TEXT NOT NULL,   -- text content, or file display name
                kind              TEXT NOT NULL DEFAULT 'text', -- 'text' | 'file'
                file_hash         TEXT,
                file_size_bytes   INTEGER,
                file_compressed   INTEGER,
                sent_unix_ms      INTEGER NOT NULL,
                outgoing          INTEGER NOT NULL   -- 1 if we sent it, 0 if received
            );

            CREATE INDEX IF NOT EXISTS idx_messages_conversation
                ON messages (conversation_kind, conversation_key, sent_unix_ms);

            CREATE TABLE IF NOT EXISTS settings (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS call_log (
                id             INTEGER PRIMARY KEY AUTOINCREMENT,
                peer_id        TEXT NOT NULL,
                peer_name      TEXT NOT NULL,
                direction      TEXT NOT NULL,   -- 'incoming' | 'outgoing'
                outcome        TEXT NOT NULL,   -- 'completed' | 'declined' | 'missed' | 'failed'
                started_at_ms  INTEGER NOT NULL,
                duration_secs  INTEGER NOT NULL DEFAULT 0
            );

            CREATE INDEX IF NOT EXISTS idx_call_log_started
                ON call_log (started_at_ms);

            -- Statuses (WhatsApp/Signal "story"-style updates): text only
            -- for now, always expiring — see `net::calls`... no, see
            -- `Store::prune_expired_statuses` and the 24h default this
            -- app posts with. One row per status *we've received* from
            -- someone, plus our own last-posted one (peer_id = our own
            -- hex id for that row).
            CREATE TABLE IF NOT EXISTS statuses (
                peer_id        TEXT PRIMARY KEY,
                peer_name      TEXT NOT NULL,
                text           TEXT NOT NULL,
                posted_at_ms   INTEGER NOT NULL,
                expires_at_ms  INTEGER NOT NULL
            );

            -- ---- Local-only tables added alongside the storage-layer
            -- ---- review below (ARCHITECTURE.md's storage-layers ADR).
            -- None of these are ever synced peer-to-peer; they're purely
            -- this device's own bookkeeping, same as `settings` above.

            -- One row per (device-local) notification we owe the user —
            -- currently just tracks what's already been shown so a
            -- restart doesn't re-notify for old messages. Nothing writes
            -- to this yet; reserved for the notification-center feature.
            CREATE TABLE IF NOT EXISTS notification_state (
                conversation_kind TEXT NOT NULL,
                conversation_key  TEXT NOT NULL,
                last_notified_ms  INTEGER NOT NULL,
                PRIMARY KEY (conversation_kind, conversation_key)
            );

            -- Cheap key/value scratch space for UI-only state that
            -- shouldn't live in `settings` (durable, user-facing prefs) —
            -- things like "which sidebar tab was last open." Safe to wipe
            -- entirely with no data loss, unlike every other table here.
            CREATE TABLE IF NOT EXISTS ui_cache (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            -- One row per file we've fetched via `net::transfer`, keyed
            -- by blob hash — lets a future "downloads" panel list what's
            -- been pulled without re-scanning the blob store, and lets a
            -- re-download of the same hash short-circuit to the existing
            -- path instead of fetching again.
            CREATE TABLE IF NOT EXISTS download_history (
                blob_hash      TEXT PRIMARY KEY,
                dest_path      TEXT NOT NULL,
                size_bytes     INTEGER NOT NULL,
                downloaded_at  INTEGER NOT NULL
            );

            -- Files handed to `net::transfer::prepare_outgoing` but not
            -- yet confirmed announced/delivered — a restart mid-send
            -- currently just silently drops the in-flight attempt; this
            -- is where a future retry-on-relaunch sweep would read from.
            CREATE TABLE IF NOT EXISTS upload_queue (
                id           INTEGER PRIMARY KEY AUTOINCREMENT,
                conversation_kind TEXT NOT NULL,
                conversation_key  TEXT NOT NULL,
                source_path  TEXT NOT NULL,
                queued_at_ms INTEGER NOT NULL,
                status       TEXT NOT NULL DEFAULT 'pending' -- 'pending' | 'sent' | 'failed'
            );

            -- Full-text index over message bodies, kept in sync with
            -- `messages` by the triggers below rather than maintained by
            -- hand at every call site. `content=''` (a "contentless" FTS5
            -- table) stores only the index, not a second copy of `body` —
            -- `messages` itself is still the source of truth; a search
            -- hit's `rowid` is looked up back against `messages.id`.
            CREATE VIRTUAL TABLE IF NOT EXISTS messages_fts USING fts5(
                body, sender_name, content=''
            );

            CREATE TRIGGER IF NOT EXISTS messages_fts_ai AFTER INSERT ON messages BEGIN
                INSERT INTO messages_fts(rowid, body, sender_name)
                VALUES (new.id, new.body, new.sender_name);
            END;

            CREATE TRIGGER IF NOT EXISTS messages_fts_ad AFTER DELETE ON messages BEGIN
                INSERT INTO messages_fts(messages_fts, rowid, body, sender_name)
                VALUES ('delete', old.id, old.body, old.sender_name);
            END;

            -- One row per (message, reactor): a second reaction from the
            -- same sender on the same message replaces (not adds to) the
            -- first via INSERT OR REPLACE — "change your reaction" reuses
            -- the same slot rather than stacking, matching every chat app
            -- that only shows one emoji per person per message.
            CREATE TABLE IF NOT EXISTS reactions (
                conversation_kind TEXT NOT NULL,
                conversation_key  TEXT NOT NULL,
                envelope_id       INTEGER NOT NULL,
                sender_id         TEXT NOT NULL,
                emoji             TEXT NOT NULL,
                PRIMARY KEY (conversation_kind, conversation_key, envelope_id, sender_id)
            );

            -- DM-only for now (see `Body::Read`'s doc for why rooms don't
            -- get this yet). `peer_id` is whoever's watermark this is —
            -- i.e. the *reader*, not the message sender — so a DM has (at
            -- most) one row per direction.
            CREATE TABLE IF NOT EXISTS read_watermarks (
                conversation_kind  TEXT NOT NULL,
                conversation_key   TEXT NOT NULL,
                peer_id            TEXT NOT NULL,
                up_to_sent_unix_ms INTEGER NOT NULL,
                PRIMARY KEY (conversation_kind, conversation_key, peer_id)
            );
            "#,
        )?;
        // `CREATE TABLE IF NOT EXISTS` above doesn't add columns to a table
        // that already existed on disk from an earlier version, and for
        // simplicity the CREATE TABLE text above was left as-is rather than
        // edited — so both a brand new `messenger.db` and an upgraded one
        // get these columns the same way: try to add, and ignore only
        // the specific "already there" error.
        for (table, column, decl) in [
            ("contacts", "verified", "INTEGER NOT NULL DEFAULT 0"),
            ("contacts", "avatar_hash", "TEXT"),
            ("messages", "expires_at", "INTEGER"),
            // BLAKE3 hex digest of `body` — see `log_message`. Purely a
            // local integrity/dedup-detection aid (content-addressed the
            // same way `iroh-blobs` addresses files), computed
            // synchronously with the existing `blake3` dependency rather
            // than routed through `iroh-blobs`'s async blob store: every
            // `log_message` call site here is a plain, synchronous `&self`
            // method called from inside code that (in several places)
            // already holds a Dioxus `Signal` write-lock — see the
            // `AlreadyBorrowedMut` history in `ARCHITECTURE.md` and
            // `ui::mod`'s doc comments on `commit_room_doc`/`commit_dm_doc`
            // for exactly the bug class that turning this into an `.await`
            // point would risk reintroducing. If real dedup/backup-via-
            // blob-store for text is wanted later, it should go through a
            // background task that reads already-committed rows, not
            // inline in the send/receive path.
            ("messages", "content_hash", "TEXT"),
            // The wire `Envelope::id` this row came from — not persisted
            // before this round, since nothing needed to address an
            // individual already-sent message from the network side
            // until reactions/edit/delete/read-receipts did. `NULL` for
            // any row written before this migration (there's no way to
            // retroactively know their original envelope id) — those
            // just can't be reacted to/edited/deleted, which is fine,
            // they predate the feature existing at all.
            ("messages", "envelope_id", "INTEGER"),
            ("messages", "edited_body", "TEXT"),
            ("messages", "edited_at", "INTEGER"),
            ("messages", "deleted", "INTEGER NOT NULL DEFAULT 0"),
            // Envelope::id of the message this one quotes, if any — see
            // `Body::Text`/`Body::File`'s doc for why this rides along on
            // the envelope itself rather than needing a lookup: the
            // reply target's own snippet (sender + truncated body) is
            // resolved locally at render time in `bubbles_for`, not
            // carried over the wire a second time. `NULL` for ordinary
            // messages and for any row written before this migration —
            // same "predates the feature" story as `envelope_id` above.
            ("messages", "reply_to_envelope_id", "INTEGER"),
            // Optional image attachment on a status update — announce-not-
            // payload shape, same as contacts.avatar_hash: the actual PNG
            // bytes live in the local iroh-blobs store (see
            // `media::decode_status_image` and `App::broadcast_status`),
            // this just remembers which one and how big it claimed to be.
            ("statuses", "image_hash", "TEXT"),
            ("statuses", "image_size_bytes", "INTEGER"),
            ("statuses", "video_hash", "TEXT"),
            ("statuses", "video_size_bytes", "INTEGER"),
            ("statuses", "audio_hash", "TEXT"),
            ("statuses", "audio_size_bytes", "INTEGER"),
        ] {
            add_column_if_missing(&conn, table, column, decl)?;
        }

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Lock the connection for the duration of one call. `unwrap()` on a
    /// poisoned lock is acceptable here: a panic while holding this lock
    /// would mean a logic bug in one of the methods below, and continuing
    /// to use a possibly-inconsistent connection afterwards would be worse
    /// than crashing loudly.
    fn conn(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
    // ---- contacts / request-accept state machine ----

    /// Insert-or-update a contact row from a *network-triggered* state
    /// transition (an incoming `ContactMsg::Request`/`Accept` — see
    /// `net::contacts::ContactProtocol::accept`). Deliberately triggered by
    /// untrusted peer input, so the `state` column here is guarded against
    /// ever regressing a settled contact:
    ///
    /// - `blocked` never moves, regardless of what `state` this call asks
    ///   for — a request-message replay can't be used to un-block someone.
    /// - `accepted` can't be knocked back down to `pending_in` — this is
    ///   the fix for the exact bug where an already-accepted contact kept
    ///   reappearing in the Requests inbox: `send_one` (see that fn's own
    ///   doc) only *schedules* delivery and doesn't guarantee exactly-once
    ///   arrival, so a `Request` that both sides raced to send before
    ///   either had accepted, or a retried one, can legitimately still
    ///   land here *after* `set_contact_state(Accepted)` already ran.
    ///   Previously this `ON CONFLICT` unconditionally applied
    ///   `excluded.state`, silently downgrading the row back to
    ///   `pending_in` the moment that late message arrived — which is
    ///   precisely what put an already-chatting contact back in Requests.
    ///
    /// Legitimate local transitions (the user clicking Accept, or a future
    /// Block action) go through `set_contact_state` instead, which has no
    /// such guard — it's driven by this device's own user action, not
    /// untrusted wire input, so it's allowed to set any state directly.
    pub fn upsert_contact(
        &self,
        endpoint_id: &str,
        username: Option<&str>,
        alias: &str,
        state: ContactState,
    ) -> Result<()> {
        self.conn().execute(
            "INSERT INTO contacts (endpoint_id, username, alias, state, requested_at)
             VALUES (?1, ?2, ?3, ?4, strftime('%s','now'))
             ON CONFLICT(endpoint_id) DO UPDATE SET
                username = COALESCE(excluded.username, contacts.username),
                alias = excluded.alias,
                state = CASE
                    WHEN contacts.state = 'blocked' THEN contacts.state
                    WHEN contacts.state = 'accepted' AND excluded.state = 'pending_in' THEN contacts.state
                    ELSE excluded.state
                END",
            params![endpoint_id, username, alias, state.as_str()],
        )?;
        Ok(())
    }

    pub fn set_contact_state(&self, endpoint_id: &str, state: ContactState) -> Result<()> {
        self.conn().execute(
            "UPDATE contacts SET state = ?2 WHERE endpoint_id = ?1",
            params![endpoint_id, state.as_str()],
        )?;
        Ok(())
    }

    pub fn remove_contact(&self, endpoint_id: &str) -> Result<()> {
        self.conn().execute(
            "DELETE FROM contacts WHERE endpoint_id = ?1",
            params![endpoint_id],
        )?;
        Ok(())
    }

    /// All contacts regardless of state (pending/accepted/blocked) — a
    /// superset of `pending_incoming`/`accepted_contacts`. Reserved for a
    /// future "all contacts" directory view; nothing in `ui::` calls this
    /// yet, so it's marked accordingly rather than left as a bare warning.
    #[allow(dead_code)]
    pub fn list_contacts(&self) -> Result<Vec<Contact>> {
        self.list_contacts_where("1=1", [])
    }

    /// Incoming requests waiting on us — drives the requests-inbox badge.
    pub fn pending_incoming(&self) -> Result<Vec<Contact>> {
        self.list_contacts_where(
            "state = 'pending_in' AND NOT EXISTS (
                SELECT 1 FROM messages
                WHERE messages.conversation_kind = 'dm'
                AND messages.conversation_key = contacts.endpoint_id
            )",
            [],
        )
    }

    pub fn accepted_contacts(&self) -> Result<Vec<Contact>> {
        self.list_contacts_where("state = 'accepted'", [])
    }

    fn list_contacts_where(
        &self,
        predicate: &str,
        params: impl rusqlite::Params,
    ) -> Result<Vec<Contact>> {
        let sql = format!(
            "SELECT endpoint_id, username, alias, state, requested_at, verified, avatar_hash
             FROM contacts WHERE {predicate} ORDER BY requested_at DESC"
        );
        let conn = self.conn();
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt
            .query_map(params, |row| {
                let state_str: String = row.get(3)?;
                Ok((
                    Contact {
                        endpoint_id: row.get(0)?,
                        username: row.get(1)?,
                        alias: row.get(2)?,
                        state: ContactState::Accepted, // placeholder, fixed below
                        requested_at: row.get(4)?,
                        verified: row.get::<_, i64>(5)? != 0,
                        avatar_hash: row.get(6)?,
                    },
                    state_str,
                ))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        rows.into_iter()
            .map(|(mut c, state_str)| {
                c.state = ContactState::parse(&state_str)?;
                Ok(c)
            })
            .collect()
    }

    /// Manually mark (or unmark) a contact as verified — see the doc on
    /// `Contact::verified`. UI entry point is the "Mark verified" toggle
    /// in `ui::ConvInfoPanel`'s DM tab.
    pub fn set_verified(&self, endpoint_id: &str, verified: bool) -> Result<()> {
        self.conn().execute(
            "UPDATE contacts SET verified = ?2 WHERE endpoint_id = ?1",
            params![endpoint_id, verified as i64],
        )?;
        Ok(())
    }

    /// Records a contact's current avatar hash — called from
    /// `App::record_incoming_avatar_hash` on receipt of a
    /// `Body::AvatarUpdate`. Doesn't touch `download_history`; that only
    /// gets a row once the bytes are actually fetched (see
    /// `cached_avatar_path`/`record_avatar_download` below), which may
    /// happen later or never if the picture's never opened.
    pub fn set_contact_avatar_hash(&self, endpoint_id: &str, hash: &str) -> Result<()> {
        self.conn().execute(
            "UPDATE contacts SET avatar_hash = ?2 WHERE endpoint_id = ?1",
            params![endpoint_id, hash],
        )?;
        Ok(())
    }

    /// Local disk path for a previously-downloaded avatar/status image, if
    /// this exact content hash has already been fetched — lets a repeat
    /// view (or a second contact who happens to share the same picture,
    /// content-addressing dedups this for free) skip the network
    /// entirely. `None` means fetch it via
    /// `net::transfer`/`app::fetch_contact_avatar` and then call
    /// `record_download` to populate this for next time.
    pub fn cached_download_path(&self, blob_hash: &str) -> Result<Option<String>> {
        self.conn()
            .query_row(
                "SELECT dest_path FROM download_history WHERE blob_hash = ?1",
                params![blob_hash],
                |row| row.get(0),
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn record_download(&self, blob_hash: &str, dest_path: &str, size_bytes: u64) -> Result<()> {
        self.conn().execute(
            "INSERT INTO download_history (blob_hash, dest_path, size_bytes, downloaded_at)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(blob_hash) DO UPDATE SET dest_path = excluded.dest_path",
            params![blob_hash, dest_path, size_bytes as i64, now_unix_ms()],
        )?;
        Ok(())
    }

    pub fn is_blocked(&self, endpoint_id: &str) -> bool {
        self.conn()
            .query_row(
                "SELECT 1 FROM contacts WHERE endpoint_id = ?1 AND state = 'blocked'",
                params![endpoint_id],
                |_| Ok(()),
            )
            .is_ok()
    }

    pub fn is_accepted(&self, endpoint_id: &str) -> bool {
        self.conn()
            .query_row(
                "SELECT 1 FROM contacts WHERE endpoint_id = ?1 AND state = 'accepted'",
                params![endpoint_id],
                |_| Ok(()),
            )
            .is_ok()
    }

    /// Look up a contact's alias by raw hex endpoint id, for call sites
    /// that only have the id (not a whole `Contact`) in hand — e.g. a
    /// future "who's this file/message from" resolver working off an
    /// `EndpointId` alone. `ui::` currently resolves aliases by scanning
    /// its own already-loaded `Vec<Contact>` instead (see
    /// `ui::short_peer_label`), so this isn't called yet.
    #[allow(dead_code)]
    pub fn alias_for(&self, endpoint_id: &str) -> Result<Option<String>> {
        Ok(self
            .conn()
            .query_row(
                "SELECT alias FROM contacts WHERE endpoint_id = ?1",
                params![endpoint_id],
                |row| row.get(0),
            )
            .ok())
    }

    // ---- settings key/value ----

    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        self.conn().execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        Ok(self
            .conn()
            .query_row(
                "SELECT value FROM settings WHERE key = ?1",
                params![key],
                |row| row.get(0),
            )
            .ok())
    }

    // ---- typed convenience wrappers over the same key/value table ----
    //
    // Everything below is just `get_setting`/`set_setting` with a fixed
    // key and a parsed/defaulted type on top — no new table, no new
    // migration. `theme_mode` defaults to `System` and the four toggles
    // default to `true` (matches what the app already did before any of
    // these were user-configurable: notifications and sound were always
    // on, and read receipts/typing indicators were always sent) — a
    // brand new install, or an upgrade from a version that predates
    // these settings existing, behaves exactly as before until the user
    // actually opens Settings and changes something.

    pub fn theme_mode(&self) -> ThemeMode {
        self.get_setting("theme_mode")
            .ok()
            .flatten()
            .and_then(|v| ThemeMode::parse(&v))
            .unwrap_or(ThemeMode::System)
    }

    pub fn set_theme_mode(&self, mode: ThemeMode) -> Result<()> {
        self.set_setting("theme_mode", mode.as_str())
    }

    pub fn theme_style(&self) -> ThemeStyle {
        self.get_setting("theme_style")
            .ok()
            .flatten()
            .and_then(|v| ThemeStyle::parse(&v))
            .unwrap_or(ThemeStyle::Regular)
    }

    pub fn set_theme_style(&self, style: ThemeStyle) -> Result<()> {
        self.set_setting("theme_style", style.as_str())
    }

    fn get_bool_setting(&self, key: &str, default: bool) -> bool {
        match self.get_setting(key) {
            Ok(Some(v)) => v == "1",
            _ => default,
        }
    }

    fn set_bool_setting(&self, key: &str, value: bool) -> Result<()> {
        self.set_setting(key, if value { "1" } else { "0" })
    }

    pub fn notifications_enabled(&self) -> bool {
        self.get_bool_setting("notifications_enabled", true)
    }
    pub fn set_notifications_enabled(&self, value: bool) -> Result<()> {
        self.set_bool_setting("notifications_enabled", value)
    }

    pub fn notification_sound_enabled(&self) -> bool {
        self.get_bool_setting("notification_sound_enabled", true)
    }
    pub fn set_notification_sound_enabled(&self, value: bool) -> Result<()> {
        self.set_bool_setting("notification_sound_enabled", value)
    }

    /// Whether to send `Body::Read` at all — see that variant's doc for
    /// why it's a distinct, privacy-sensitive signal from `Body::Ack`.
    /// Turning this off means *your own* read state is never announced;
    /// it doesn't affect whether you can see peers' read receipts to you.
    pub fn read_receipts_enabled(&self) -> bool {
        self.get_bool_setting("read_receipts_enabled", true)
    }
    pub fn set_read_receipts_enabled(&self, value: bool) -> Result<()> {
        self.set_bool_setting("read_receipts_enabled", value)
    }

    /// Whether to send `Body::Typing` at all. Same privacy shape as
    /// `read_receipts_enabled`: only gates *sending* your own signal.
    pub fn typing_indicator_enabled(&self) -> bool {
        self.get_bool_setting("typing_indicator_enabled", true)
    }
    pub fn set_typing_indicator_enabled(&self, value: bool) -> Result<()> {
        self.set_bool_setting("typing_indicator_enabled", value)
    }

    /// Android only: keep a low-priority foreground service (`dev.
    /// irshad.siar.RelayForegroundService`) alive so the app's iroh
    /// endpoint stays connected — and can wake the device for an
    /// incoming DM, room message, or call — while backgrounded. Off by
    /// default: it's a real battery/always-on-notification trade-off
    /// (a persistent low-priority notification is unavoidable — Android
    /// requires one for any foreground service), so this only runs once
    /// the user opts in from Settings' Network tab. No effect on
    /// desktop, which doesn't get backgrounded/Doze'd the same way.
    pub fn background_wake_enabled(&self) -> bool {
        self.get_bool_setting("background_wake_enabled", false)
    }
    pub fn set_background_wake_enabled(&self, value: bool) -> Result<()> {
        self.set_bool_setting("background_wake_enabled", value)
    }

    /// Whether `net::mesh::MeshManager` should run: flood messages over
    /// local Wi-Fi broadcast and (desktop) Bluetooth LE scanning when
    /// the relay/internet path isn't reachable. Off by default — same
    /// reasoning as `background_wake_enabled`: this is an explicit
    /// trade-off (broadcast/scan chatter, extra battery use) the user
    /// opts into, not a silent background behavior.
    pub fn offline_mesh_enabled(&self) -> bool {
        self.get_bool_setting("offline_mesh_enabled", false)
    }
    pub fn set_offline_mesh_enabled(&self, value: bool) -> Result<()> {
        self.set_bool_setting("offline_mesh_enabled", value)
    }

    // ---- known registry peers ----
    //
    // The username registry (`net::registry`) is an `iroh-docs` replica
    // that only syncs with peers it's told about (see that module's doc
    // comment on `Registry::new` — a `DocTicket`'s peer list, not magic
    // global discovery). A brand new install has nobody to sync with, so
    // its local view only ever contains claims it wrote itself. To fix
    // that without any central bootstrap server, every endpoint we ever
    // directly connect to for *any* reason (accepted contact, incoming
    // request, successful ticket connect) is remembered here and fed back
    // into `Registry::new` as bootstrap peers on the *next* launch —
    // people you've actually talked to are a reasonable bet to also be
    // registry members, and this lets the phonebook propagate transitively
    // across the social graph over time instead of staying siloed per
    // install. Capped and de-duplicated so it can't grow unbounded.
    const MAX_KNOWN_REGISTRY_PEERS: usize = 64;

    pub fn remember_registry_peer(&self, endpoint_id: &str) -> Result<()> {
        let mut peers = self.known_registry_peers()?;
        peers.retain(|p| p != endpoint_id);
        peers.insert(0, endpoint_id.to_string());
        peers.truncate(Self::MAX_KNOWN_REGISTRY_PEERS);
        self.set_setting("known_registry_peers", &peers.join(","))
    }

    pub fn known_registry_peers(&self) -> Result<Vec<String>> {
        Ok(self
            .get_setting("known_registry_peers")?
            .map(|s| {
                s.split(',')
                    .filter(|p| !p.is_empty())
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default())
    }

    // ---- messages ----

    // ---- call log ----

    pub fn log_call(
        &self,
        peer_id: &str,
        peer_name: &str,
        direction: CallDirection,
        outcome: CallOutcome,
        started_at_ms: i64,
        duration_secs: i64,
    ) -> Result<()> {
        self.conn().execute(
            "INSERT INTO call_log (peer_id, peer_name, direction, outcome, started_at_ms, duration_secs)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![peer_id, peer_name, direction.as_str(), outcome.as_str(), started_at_ms, duration_secs],
        )?;
        Ok(())
    }

    /// Most recent calls first, capped at `limit` — a call log is a
    /// history view, not something that needs to load every call ever
    /// made.
    pub fn recent_calls(&self, limit: u32) -> Result<Vec<CallLogEntry>> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT peer_id, peer_name, direction, outcome, started_at_ms, duration_secs
             FROM call_log ORDER BY started_at_ms DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit], |row| {
            Ok(CallLogEntry {
                peer_id: row.get(0)?,
                peer_name: row.get(1)?,
                direction: CallDirection::from_str(&row.get::<_, String>(2)?),
                outcome: CallOutcome::from_str(&row.get::<_, String>(3)?),
                started_at_ms: row.get(4)?,
                duration_secs: row.get(5)?,
            })
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    // ---- statuses ----
    //
    // One row per peer (including ourselves, keyed by our own hex id) —
    // posting a new status replaces the previous one rather than
    // accumulating history, matching how WhatsApp/Signal statuses work
    // (a feed of *current* statuses, not a log of past ones).

    // Mirrors one normalized SQL row. A parameter object would merely
    // duplicate StatusEntry with borrowed fields and make call sites less
    // transparent, so keep the database boundary explicit.
    #[allow(clippy::too_many_arguments)]
    pub fn upsert_status(
        &self,
        peer_id: &str,
        peer_name: &str,
        text: &str,
        posted_at_ms: i64,
        expires_at_ms: i64,
        image_hash: Option<&str>,
        image_size_bytes: Option<u64>,
        video_hash: Option<&str>,
        video_size_bytes: Option<u64>,
        audio_hash: Option<&str>,
        audio_size_bytes: Option<u64>,
    ) -> Result<()> {
        self.conn().execute(
            "INSERT INTO statuses (peer_id, peer_name, text, posted_at_ms, expires_at_ms, image_hash, image_size_bytes, video_hash, video_size_bytes, audio_hash, audio_size_bytes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
             ON CONFLICT(peer_id) DO UPDATE SET
                peer_name = excluded.peer_name, text = excluded.text,
                posted_at_ms = excluded.posted_at_ms, expires_at_ms = excluded.expires_at_ms,
                image_hash = excluded.image_hash, image_size_bytes = excluded.image_size_bytes,
                video_hash = excluded.video_hash, video_size_bytes = excluded.video_size_bytes,
                audio_hash = excluded.audio_hash, audio_size_bytes = excluded.audio_size_bytes",
            params![
                peer_id, peer_name, text, posted_at_ms, expires_at_ms,
                image_hash, image_size_bytes.map(|v| v as i64),
                video_hash, video_size_bytes.map(|v| v as i64),
                audio_hash, audio_size_bytes.map(|v| v as i64),
            ],
        )?;
        Ok(())
    }

    /// Currently-live statuses (not yet expired), most recent first.
    pub fn active_statuses(&self, now_ms: i64) -> Result<Vec<StatusEntry>> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT peer_id, peer_name, text, posted_at_ms, expires_at_ms, image_hash, image_size_bytes, video_hash, video_size_bytes, audio_hash, audio_size_bytes
             FROM statuses WHERE expires_at_ms > ?1 ORDER BY posted_at_ms DESC",
        )?;
        let rows = stmt.query_map(params![now_ms], |row| {
            Ok(StatusEntry {
                peer_id: row.get(0)?,
                peer_name: row.get(1)?,
                text: row.get(2)?,
                posted_at_ms: row.get(3)?,
                expires_at_ms: row.get(4)?,
                image_hash: row.get(5)?,
                image_size_bytes: row.get::<_, Option<i64>>(6)?.map(|v| v as u64),
                video_hash: row.get(7)?,
                video_size_bytes: row.get::<_, Option<i64>>(8)?.map(|v| v as u64),
                audio_hash: row.get(9)?,
                audio_size_bytes: row.get::<_, Option<i64>>(10)?.map(|v| v as u64),
            })
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    /// Sweep out anything past its `expires_at_ms` — called alongside the
    /// existing disappearing-messages sweep (`ui::spawn_disappearing_sweep`).
    pub fn prune_expired_statuses(&self, now_ms: i64) -> Result<usize> {
        Ok(self.conn().execute(
            "DELETE FROM statuses WHERE expires_at_ms <= ?1",
            params![now_ms],
        )?)
    }

    // This is the single persistence boundary for the complete wire
    // envelope metadata. Keeping the columns visible here makes migrations
    // and INSERT ordering auditable together.
    #[allow(clippy::too_many_arguments)]
    pub fn log_message(
        &self,
        conversation: &Conversation,
        sender_id: &str,
        sender_name: &str,
        body: &str,
        kind: &MessageKind,
        sent_unix_ms: i64,
        outgoing: bool,
        expires_at_unix_ms: Option<i64>,
        envelope_id: u64,
        reply_to_envelope_id: Option<u64>,
    ) -> Result<()> {
        let (conv_kind, conv_key) = match conversation {
            Conversation::Dm(id) => ("dm", id.as_str()),
            Conversation::Room(name) => ("room", name.as_str()),
        };
        let (kind_str, hash, size, compressed) = match kind {
            MessageKind::Text => ("text", None, None, None),
            MessageKind::File {
                hash,
                size_bytes,
                compressed,
                ..
            } => (
                "file",
                Some(hash.as_str()),
                Some(*size_bytes as i64),
                Some(*compressed as i64),
            ),
        };
        // Content-addressed the same way a file's `file_hash` already is —
        // see the `content_hash` migration comment in `open` for why this
        // is a plain synchronous BLAKE3 hash rather than a write into
        // `iroh-blobs`. Hashing `body` covers both kinds: for `File`,
        // `body` is the display name (matches what's actually stored
        // in-row), and the real file content is already integrity-checked
        // separately by `iroh-blobs`' own BLAKE3 streaming.
        let content_hash = blake3::hash(body.as_bytes()).to_hex().to_string();
        // `envelope_id` (a `u64`) stored bit-for-bit into a signed 64-bit
        // SQLite column via `as i64` — this is never used for numeric
        // comparison/ordering, only equality lookups against another
        // `u64` cast the same way, so the sign bit doesn't matter. Same
        // deal for `reply_to_envelope_id`.
        self.conn().execute(
            "INSERT INTO messages
                (conversation_kind, conversation_key, sender_id, sender_name, body,
                 kind, file_hash, file_size_bytes, file_compressed, sent_unix_ms, outgoing,
                 expires_at, content_hash, envelope_id, reply_to_envelope_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            params![
                conv_kind,
                conv_key,
                sender_id,
                sender_name,
                body,
                kind_str,
                hash,
                size,
                compressed,
                sent_unix_ms,
                outgoing as i64,
                expires_at_unix_ms,
                content_hash,
                envelope_id as i64,
                reply_to_envelope_id.map(|id| id as i64),
            ],
        )?;
        Ok(())
    }

    /// Applies a `Body::Reaction` — `remove: false` upserts (replacing any
    /// previous emoji from the same `sender_id` on this message, see the
    /// `reactions` table's doc), `remove: true` deletes that sender's row
    /// entirely (not just this emoji — tapping *any* reaction off removes
    /// whatever you had, matching the "toggle" UI this is meant to back).
    pub fn apply_reaction(
        &self,
        conversation: &Conversation,
        envelope_id: u64,
        sender_id: &str,
        emoji: &str,
        remove: bool,
    ) -> Result<()> {
        let (conv_kind, conv_key) = match conversation {
            Conversation::Dm(id) => ("dm", id.as_str()),
            Conversation::Room(name) => ("room", name.as_str()),
        };
        if remove {
            self.conn().execute(
                "DELETE FROM reactions
                 WHERE conversation_kind = ?1 AND conversation_key = ?2
                   AND envelope_id = ?3 AND sender_id = ?4",
                params![conv_kind, conv_key, envelope_id as i64, sender_id],
            )?;
        } else {
            self.conn().execute(
                "INSERT OR REPLACE INTO reactions
                    (conversation_kind, conversation_key, envelope_id, sender_id, emoji)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![conv_kind, conv_key, envelope_id as i64, sender_id, emoji],
            )?;
        }
        Ok(())
    }

    /// `(sender_id, emoji)` pairs for every envelope_id in `ids`, in one
    /// query — used by `recent_messages` to attach reactions to the
    /// message list it just loaded, without a query per message.
    fn reactions_for_envelopes(
        &self,
        conversation: &Conversation,
        ids: &[i64],
    ) -> Result<std::collections::HashMap<i64, Vec<(String, String)>>> {
        let mut out = std::collections::HashMap::new();
        if ids.is_empty() {
            return Ok(out);
        }
        let (conv_kind, conv_key) = match conversation {
            Conversation::Dm(id) => ("dm", id.as_str()),
            Conversation::Room(name) => ("room", name.as_str()),
        };
        let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let sql = format!(
            "SELECT envelope_id, sender_id, emoji FROM reactions
             WHERE conversation_kind = ? AND conversation_key = ?
               AND envelope_id IN ({placeholders})"
        );
        let conn = self.conn();
        let mut stmt = conn.prepare(&sql)?;
        let mut params_vec: Vec<&dyn rusqlite::ToSql> = vec![&conv_kind, &conv_key];
        for id in ids {
            params_vec.push(id);
        }
        let rows = stmt.query_map(params_vec.as_slice(), |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?;
        for row in rows {
            let (envelope_id, sender_id, emoji) = row?;
            out.entry(envelope_id)
                .or_insert_with(Vec::new)
                .push((sender_id, emoji));
        }
        Ok(out)
    }

    /// Applies a `Body::Edit` — `claimed_sender_id` is whoever the
    /// network connection this arrived on is authenticated as; it's
    /// folded straight into the `WHERE` clause rather than checked with
    /// a separate lookup query, so an edit claiming to own a message it
    /// didn't originally send just matches zero rows and silently no-ops
    /// — there's no separate window between "check" and "write" to race.
    pub fn apply_edit(
        &self,
        conversation: &Conversation,
        envelope_id: u64,
        claimed_sender_id: &str,
        new_text: &str,
        edited_at_unix_ms: i64,
    ) -> Result<()> {
        let (conv_kind, conv_key) = match conversation {
            Conversation::Dm(id) => ("dm", id.as_str()),
            Conversation::Room(name) => ("room", name.as_str()),
        };
        self.conn().execute(
            "UPDATE messages SET body = ?1, edited_body = ?1, edited_at = ?2
             WHERE conversation_kind = ?3 AND conversation_key = ?4
               AND envelope_id = ?5 AND sender_id = ?6",
            params![
                new_text,
                edited_at_unix_ms,
                conv_kind,
                conv_key,
                envelope_id as i64,
                claimed_sender_id
            ],
        )?;
        Ok(())
    }

    /// Applies a `Body::Delete` — same ownership note as `apply_edit`.
    pub fn apply_delete(
        &self,
        conversation: &Conversation,
        envelope_id: u64,
        claimed_sender_id: &str,
    ) -> Result<()> {
        let (conv_kind, conv_key) = match conversation {
            Conversation::Dm(id) => ("dm", id.as_str()),
            Conversation::Room(name) => ("room", name.as_str()),
        };
        self.conn().execute(
            "UPDATE messages SET deleted = 1
             WHERE conversation_kind = ?1 AND conversation_key = ?2
               AND envelope_id = ?3 AND sender_id = ?4",
            params![conv_kind, conv_key, envelope_id as i64, claimed_sender_id],
        )?;
        Ok(())
    }

    /// Records that `peer_id` (the reader) has now seen everything up to
    /// `up_to_sent_unix_ms` in this conversation. DM-only, per `Body::
    /// Read`'s doc — room read receipts would need per-member tracking
    /// against an interleaved multi-sender timeline, a genuinely
    /// different (and harder) feature than a single watermark, left for
    /// later rather than approximated badly here.
    pub fn set_read_watermark(
        &self,
        conversation: &Conversation,
        peer_id: &str,
        up_to_sent_unix_ms: i64,
    ) -> Result<()> {
        let (conv_kind, conv_key) = match conversation {
            Conversation::Dm(id) => ("dm", id.as_str()),
            Conversation::Room(name) => ("room", name.as_str()),
        };
        self.conn().execute(
            "INSERT INTO read_watermarks (conversation_kind, conversation_key, peer_id, up_to_sent_unix_ms)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT (conversation_kind, conversation_key, peer_id)
             DO UPDATE SET up_to_sent_unix_ms = MAX(up_to_sent_unix_ms, excluded.up_to_sent_unix_ms)",
            params![conv_kind, conv_key, peer_id, up_to_sent_unix_ms],
        )?;
        Ok(())
    }

    /// The other side's read watermark for this DM — `None` if they've
    /// never sent one (older peer, or nothing read yet). Compare a sent
    /// message's `sent_unix_ms` against this to decide "read" vs
    /// "delivered" in the UI.
    pub fn read_watermark(
        &self,
        conversation: &Conversation,
        peer_id: &str,
    ) -> Result<Option<i64>> {
        let (conv_kind, conv_key) = match conversation {
            Conversation::Dm(id) => ("dm", id.as_str()),
            Conversation::Room(name) => ("room", name.as_str()),
        };
        Ok(self
            .conn()
            .query_row(
                "SELECT up_to_sent_unix_ms FROM read_watermarks
                 WHERE conversation_kind = ?1 AND conversation_key = ?2 AND peer_id = ?3",
                params![conv_kind, conv_key, peer_id],
                |row| row.get(0),
            )
            .optional()?)
    }

    /// Full-text search over message bodies/sender names, most recent
    /// first — backed by the `messages_fts` FTS5 index kept in sync by
    /// triggers in `open` (see there for why it's contentless). `query`
    /// is passed straight through as an FTS5 match expression (so the
    /// caller gets FTS5's prefix (`term*`), phrase (`"a b"`), and
    /// boolean-operator syntax for free); a caller taking raw user input
    /// should decide whether to expose that or pre-sanitize it.
    pub fn search_messages(&self, query: &str, limit: u32) -> Result<Vec<StoredMessage>> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT m.sender_name, m.body, m.kind, m.file_hash, m.file_size_bytes,
                    m.file_compressed, m.sent_unix_ms, m.outgoing, m.expires_at,
                    m.envelope_id, m.edited_at, m.deleted, m.reply_to_envelope_id
             FROM messages_fts f
             JOIN messages m ON m.id = f.rowid
             WHERE messages_fts MATCH ?1
             ORDER BY m.sent_unix_ms DESC LIMIT ?2",
        )?;
        let rows = stmt
            .query_map(params![query, limit], |row| {
                let kind_str: String = row.get(2)?;
                let kind = if kind_str == "file" {
                    MessageKind::File {
                        name: row.get::<_, String>(1)?,
                        hash: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                        size_bytes: row.get::<_, Option<i64>>(4)?.unwrap_or(0) as u64,
                        compressed: row.get::<_, Option<i64>>(5)?.unwrap_or(0) != 0,
                    }
                } else {
                    MessageKind::Text
                };
                Ok(StoredMessage {
                    sender_name: row.get(0)?,
                    body: row.get(1)?,
                    sent_unix_ms: row.get(6)?,
                    outgoing: row.get::<_, i64>(7)? != 0,
                    kind,
                    expires_at_unix_ms: row.get(8)?,
                    envelope_id: row.get::<_, Option<i64>>(9)?.map(|v| v as u64),
                    edited_at_unix_ms: row.get(10)?,
                    deleted: row.get::<_, i64>(11)? != 0,
                    // Search results are a lightweight preview list, not
                    // the full conversation view — not worth a second
                    // batched reactions query for a list the user is
                    // about to click through to the real message view
                    // from anyway.
                    reactions: Vec::new(),
                    reply_to_envelope_id: row.get::<_, Option<i64>>(12)?.map(|v| v as u64),
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn recent_messages(
        &self,
        conversation: &Conversation,
        limit: u32,
    ) -> Result<Vec<StoredMessage>> {
        let (conv_kind, conv_key) = match conversation {
            Conversation::Dm(id) => ("dm", id.as_str()),
            Conversation::Room(name) => ("room", name.as_str()),
        };
        let now_ms = now_unix_ms();
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT sender_name, body, kind, file_hash, file_size_bytes, file_compressed,
                    sent_unix_ms, outgoing, expires_at, envelope_id, edited_at, deleted,
                    reply_to_envelope_id
             FROM messages
             WHERE conversation_kind = ?1 AND conversation_key = ?2
               AND (expires_at IS NULL OR expires_at > ?4)
             ORDER BY sent_unix_ms DESC LIMIT ?3",
        )?;
        let mut rows = stmt
            .query_map(params![conv_kind, conv_key, limit, now_ms], |row| {
                let kind_str: String = row.get(2)?;
                let kind = if kind_str == "file" {
                    MessageKind::File {
                        name: row.get::<_, String>(1)?,
                        hash: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                        size_bytes: row.get::<_, Option<i64>>(4)?.unwrap_or(0) as u64,
                        compressed: row.get::<_, Option<i64>>(5)?.unwrap_or(0) != 0,
                    }
                } else {
                    MessageKind::Text
                };
                Ok(StoredMessage {
                    sender_name: row.get(0)?,
                    body: row.get(1)?,
                    sent_unix_ms: row.get(6)?,
                    outgoing: row.get::<_, i64>(7)? != 0,
                    kind,
                    expires_at_unix_ms: row.get(8)?,
                    envelope_id: row.get::<_, Option<i64>>(9)?.map(|v| v as u64),
                    edited_at_unix_ms: row.get(10)?,
                    deleted: row.get::<_, i64>(11)? != 0,
                    reactions: Vec::new(), // filled in below, once, for the whole batch
                    reply_to_envelope_id: row.get::<_, Option<i64>>(12)?.map(|v| v as u64),
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        drop(stmt);
        drop(conn);
        let ids: Vec<i64> = rows
            .iter()
            .filter_map(|m| m.envelope_id)
            .map(|id| id as i64)
            .collect();
        let mut reactions_by_id = self.reactions_for_envelopes(conversation, &ids)?;
        for m in &mut rows {
            if let Some(id) = m.envelope_id {
                m.reactions = reactions_by_id.remove(&(id as i64)).unwrap_or_default();
            }
        }
        rows.reverse(); // chronological order
        Ok(rows)
    }

    /// Physically delete every message past its `expires_at`. Called
    /// periodically (see `ui::spawn_disappearing_sweep`) rather than
    /// relying solely on the `recent_messages` read-side filter above —
    /// belt-and-suspenders per the worked example this follows: a missed
    /// sweep never leaks an expired message (the read filter guarantees
    /// that), but without an actual sweep the rows would just accumulate
    /// forever, never freeing space. Returns how many rows were removed,
    /// purely for logging.
    pub fn sweep_expired_messages(&self) -> Result<usize> {
        let now_ms = now_unix_ms();
        Ok(self.conn().execute(
            "DELETE FROM messages WHERE expires_at IS NOT NULL AND expires_at <= ?1",
            params![now_ms],
        )?)
    }

    /// Room names you've ever sent or received a message in, most recently
    /// active first. Used to auto-rejoin rooms across restarts.
    pub fn distinct_rooms(&self) -> Result<Vec<String>> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT conversation_key, MAX(sent_unix_ms) as last_active
             FROM messages WHERE conversation_kind = 'room'
             GROUP BY conversation_key ORDER BY last_active DESC",
        )?;
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }
}
