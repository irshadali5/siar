//! Conversation *metadata* — synced via `iroh-docs`, deliberately holding
//! nothing that belongs in `store.rs`.
//!
//! ## Division of labor (important, don't blur this)
//! - `store.rs` (sqlite/rusqlite) is the single source of truth for
//!   **message and file content** — chat history, file-transfer records,
//!   everything the conversation view renders as a scrollback. It is local,
//!   fast, queryable, and per-device; it is never synced peer-to-peer on
//!   its own.
//! - This module (`iroh-docs`) is the single source of truth for
//!   **conversation-level metadata that needs to survive being offline and
//!   converge without a server**: a room's title and member list, a DM's
//!   shared nickname/pin/archive/disappearing-message settings. None of it
//!   is chat content. If a field would show up as a message bubble, it
//!   belongs in `store.rs`, not here.
//!
//! This split is what the user asked for explicitly and is also the
//! more defensible design: `iroh-docs`' CRDT merge is what "durable,
//! conflict-free, syncs while offline" actually buys you (spec §6.2,
//! §7's data-model note), and that property matters far more for "who's in
//! this room" (silently losing a membership change is a correctness bug)
//! than it does for message ordering (already handled by `(sent_at_ms,
//! sender, seq)` sort per spec §7, and delivered directly over the DM/
//! gossip transport rather than replayed through a doc). Putting the full
//! message log in `iroh-docs` too — the option the spec's §6.2 sketches as
//! one *possible* shape — would mean every message write pays doc-merge
//! overhead and every device holds the entire CRDT history forever with no
//! natural pruning story; sqlite has neither problem and is what v2 was
//! already built on (see `ARCHITECTURE.md` §6).
//!
//! ## Namespace derivation (no ticket exchange needed)
//! Both a room's and a DM's metadata namespace are *deterministically*
//! derived (see `crate::ticket::namespace_secret_for_room` /
//! `namespace_secret_for_dm`) rather than created by one side and shipped
//! to the other as a `DocTicket`. This sidesteps a real chicken-and-egg
//! problem: for a brand new room there's no established channel to send a
//! ticket over before the room "exists," and for a DM the contact-request
//! handshake (`net::contacts`) already establishes mutual consent — a
//! second round-trip just to exchange a doc ticket would be pure overhead.
//! Anyone who knows the room name (or, for a DM, both endpoint IDs — which
//! by definition both parties already know) can derive the same namespace
//! and open it directly.
//!
//! ## API accuracy note
//! Same caveat as `net::registry`: `iroh-docs`' client surface moves fast.
//! The call shapes below follow the confirmed patterns already used there
//! (`docs.import(ticket)`, `doc.set_bytes`, `doc.get_one`/`get_many` with
//! `Query::single_latest_per_key()`); narrower specifics are marked
//! `// VERIFY:` where I couldn't confirm them directly.

use crate::protocol::message::MAX_MESSAGE_BYTES;
use anyhow::{Context, Result};
use iroh::{EndpointAddr, EndpointId};
use iroh_docs::protocol::Docs;
use iroh_docs::store::Query;
use iroh_docs::{AuthorId, Capability, DocTicket, NamespaceSecret};
use n0_future::StreamExt;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn hex32(bytes: &[u8; 32]) -> String {
    data_encoding::HEXLOWER.encode(bytes)
}

async fn open_doc(
    docs: &Docs,
    namespace_secret: [u8; 32],
    bootstrap: Vec<EndpointAddr>,
) -> Result<iroh_docs::api::Doc> {
    // VERIFY: same `NamespaceSecret::from_bytes` / `Capability::Write(..)`
    // shape as `net::registry::Registry::new` — kept identical on purpose
    // so a future fix to one is a fix to both.
    let namespace_secret = NamespaceSecret::from_bytes(&namespace_secret);
    let ticket = DocTicket::new(Capability::Write(namespace_secret), bootstrap);
    docs.import(ticket)
        .await
        .context("opening conversation metadata doc")
}

// ---------------------------------------------------------------------
// Room metadata: title + membership (add via self-write, remove via
// admin-written tombstone).
// ---------------------------------------------------------------------

const ROOM_META_KEY: &[u8] = b"meta";
const ROOM_MEMBER_PREFIX: &str = "member/";
const ROOM_REMOVED_PREFIX: &str = "removed/";

/// Room-level settings that only make sense to have one current value —
/// title and who's allowed to remove members. Stored as a single
/// last-writer-wins record at a fixed key, same resolution policy
/// `net::registry` already uses (`single_latest_per_key`, latest
/// `claimed_at_unix_ms`-equivalent wins).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RoomMeta {
    pub name: String,
    pub title: String,
    /// The endpoint that first wrote this record — treated as the room's
    /// admin for the purpose of "who can remove a member" in the UI. This
    /// is a convention the UI enforces (see module doc), not a namespace
    /// permission: the namespace has no concept of write ACLs beyond "you
    /// need the secret," and everyone joining the same room name has it.
    pub admin: [u8; 32],
    pub created_at_ms: u64,
}

/// One member's self-attested presence in the room. Written under the
/// member's *own* key so concurrent joins from different people never
/// conflict with each other — each person only ever writes their own row.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemberRecord {
    pub endpoint_id: [u8; 32],
    pub display_name: String,
    pub joined_at_ms: u64,
}

/// A tombstone marking a member as removed. Kept separate from
/// `MemberRecord` (rather than mutating it) so the removed member's own
/// device can still see *that* it was removed next time it syncs, instead
/// of its row just vanishing with no explanation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemovalRecord {
    pub removed_at_ms: u64,
    pub removed_by: [u8; 32],
}

pub struct RoomDoc {
    blobs: iroh_blobs::api::Store,
    doc: iroh_docs::api::Doc,
    author: AuthorId,
}

impl RoomDoc {
    /// Open (joining if this is the first local instance to do so, or
    /// resuming an already-synced local copy otherwise) the metadata doc
    /// for room `name`. Never fails just because nobody's written a
    /// `RoomMeta` record yet — that only happens on `ensure_meta`.
    pub async fn open(
        docs: &Docs,
        blobs: iroh_blobs::api::Store,
        author: AuthorId,
        name: &str,
        bootstrap: Vec<EndpointAddr>,
    ) -> Result<Self> {
        let doc = open_doc(
            docs,
            crate::ticket::namespace_secret_for_room(name),
            bootstrap,
        )
        .await?;
        Ok(Self { blobs, doc, author })
    }

    /// Make sure a `RoomMeta` record exists, writing one (with `self_id` as
    /// admin) if this is the first time anyone locally has seen this room.
    /// Idempotent — if a record already synced in (from us on another
    /// occasion, or from someone else), it's left alone rather than
    /// overwritten, so re-joining a room you didn't create never steals
    /// the `admin` slot from whoever actually made it.
    pub async fn ensure_meta(&self, name: &str, self_id: EndpointId) -> Result<RoomMeta> {
        if let Some(existing) = self.meta().await? {
            return Ok(existing);
        }
        let meta = RoomMeta {
            name: name.to_string(),
            title: name.to_string(),
            admin: *self_id.as_bytes(),
            created_at_ms: now_ms(),
        };
        self.doc
            .set_bytes(
                self.author,
                ROOM_META_KEY.to_vec(),
                postcard::to_stdvec(&meta)?,
            )
            .await
            .context("writing initial room metadata")?;
        Ok(meta)
    }

    pub async fn meta(&self) -> Result<Option<RoomMeta>> {
        let Some(entry) = self
            .doc
            .get_one(Query::single_latest_per_key().key_exact(ROOM_META_KEY))
            .await
            .context("reading room metadata")?
        else {
            return Ok(None);
        };
        let bytes = self.blobs.blobs().get_bytes(entry.content_hash()).await?;
        Ok(Some(decode(&bytes)?))
    }

    /// Rename the room. Only meaningful when called by the room's admin —
    /// the UI should gate the "rename" action on `meta().admin == my_id`,
    /// same convention as `remove_member` below.
    pub async fn set_title(&self, title: &str) -> Result<()> {
        let Some(mut meta) = self.meta().await? else {
            anyhow::bail!("room metadata not initialized yet — call ensure_meta first");
        };
        meta.title = title.to_string();
        self.doc
            .set_bytes(
                self.author,
                ROOM_META_KEY.to_vec(),
                postcard::to_stdvec(&meta)?,
            )
            .await
            .context("updating room title")?;
        Ok(())
    }

    /// Announce that `self_id` has joined — writes under `self_id`'s own
    /// key, so this never races against anyone else's join.
    pub async fn announce_self(&self, self_id: EndpointId, display_name: &str) -> Result<()> {
        let record = MemberRecord {
            endpoint_id: *self_id.as_bytes(),
            display_name: display_name.to_string(),
            joined_at_ms: now_ms(),
        };
        let key = format!("{ROOM_MEMBER_PREFIX}{}", hex32(&record.endpoint_id));
        self.doc
            .set_bytes(self.author, key.into_bytes(), postcard::to_stdvec(&record)?)
            .await
            .context("announcing room membership")?;
        Ok(())
    }

    /// Write a removal tombstone for `target`. The UI is responsible for
    /// only exposing this action to the room's recorded admin — see the
    /// module doc's note that this is an application-level convention, not
    /// an enforced namespace ACL (nothing stops a non-admin device from
    /// calling this directly; the honest limitation is the same shape as
    /// `net::registry`'s optimistic uniqueness).
    pub async fn remove_member(&self, target: EndpointId, removed_by: EndpointId) -> Result<()> {
        let key = format!("{ROOM_REMOVED_PREFIX}{}", hex32(target.as_bytes()));
        let record = RemovalRecord {
            removed_at_ms: now_ms(),
            removed_by: *removed_by.as_bytes(),
        };
        self.doc
            .set_bytes(self.author, key.into_bytes(), postcard::to_stdvec(&record)?)
            .await
            .context("writing removal tombstone")?;
        Ok(())
    }

    /// Current membership: everyone with a `MemberRecord` that has no
    /// matching `RemovalRecord`. Local-only read of the currently-synced
    /// view, same as `Registry::resolve` — no network round trip.
    pub async fn list_members(&self) -> Result<Vec<MemberRecord>> {
        let removed = self.removed_set().await?;

        let entries = self
            .doc
            .get_many(Query::single_latest_per_key().key_prefix(ROOM_MEMBER_PREFIX.as_bytes()))
            .await
            .context("listing room members")?;
        let mut entries = Box::pin(entries);

        let mut members = Vec::new();
        while let Some(entry) = entries.next().await {
            let entry = entry?;
            let bytes = self.blobs.blobs().get_bytes(entry.content_hash()).await?;
            let record: MemberRecord = decode(&bytes)?;
            if !removed.contains(&record.endpoint_id) {
                members.push(record);
            }
        }
        Ok(members)
    }

    async fn removed_set(&self) -> Result<std::collections::HashSet<[u8; 32]>> {
        let entries = self
            .doc
            .get_many(Query::single_latest_per_key().key_prefix(ROOM_REMOVED_PREFIX.as_bytes()))
            .await
            .context("listing room removal tombstones")?;
        let mut entries = Box::pin(entries);

        let mut removed = std::collections::HashSet::new();
        while let Some(entry) = entries.next().await {
            let entry = entry?;
            if let Some(hex) = std::str::from_utf8(entry.key())
                .ok()
                .and_then(|k| k.strip_prefix(ROOM_REMOVED_PREFIX))
            {
                if let Ok(bytes) = data_encoding::HEXLOWER.decode(hex.as_bytes()) {
                    if let Ok(arr) = <[u8; 32]>::try_from(bytes.as_slice()) {
                        removed.insert(arr);
                    }
                }
            }
        }
        Ok(removed)
    }
}

// ---------------------------------------------------------------------
// DM metadata: shared conversation settings (nickname/pin/archive/TTL).
// Never the message log itself — see module doc.
// ---------------------------------------------------------------------

const DM_SETTINGS_KEY: &[u8] = b"settings";

/// Small, user-visible conversation settings that both sides of a DM
/// benefit from seeing synced (e.g. a shared nickname you set once rather
/// than each device tracking its own). Deliberately just a handful of
/// fields — this is not where chat history or delivery status lives.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct DmSettings {
    pub conversation_title: Option<String>,
    pub pinned: bool,
    pub archived: bool,
    /// Disappearing-message TTL, seconds. `None` = off. Enforcement is
    /// local (each device deletes its own expired rows from `store.rs`);
    /// this field is just the synced *setting*, per spec §11's "TTL
    /// metadata + local enforcement."
    pub disappearing_ttl_secs: Option<u64>,
    pub updated_at_ms: u64,
}

pub struct DmDoc {
    blobs: iroh_blobs::api::Store,
    doc: iroh_docs::api::Doc,
    author: AuthorId,
}

impl DmDoc {
    pub async fn open(
        docs: &Docs,
        blobs: iroh_blobs::api::Store,
        author: AuthorId,
        my_id: EndpointId,
        peer_id: EndpointId,
    ) -> Result<Self> {
        let doc = open_doc(
            docs,
            crate::ticket::namespace_secret_for_dm(my_id, peer_id),
            vec![],
        )
        .await?;
        Ok(Self { blobs, doc, author })
    }

    /// Current settings, resolved across whichever side last wrote —
    /// `single_latest_per_key` again does the "latest timestamp wins"
    /// merge for us, same as the registry and room title above. Returns
    /// the all-default record if neither side has ever set anything.
    pub async fn settings(&self) -> Result<DmSettings> {
        let Some(entry) = self
            .doc
            .get_one(Query::single_latest_per_key().key_exact(DM_SETTINGS_KEY))
            .await
            .context("reading DM settings")?
        else {
            return Ok(DmSettings::default());
        };
        let bytes = self.blobs.blobs().get_bytes(entry.content_hash()).await?;
        decode(&bytes)
    }

    async fn write(&self, settings: &DmSettings) -> Result<()> {
        self.doc
            .set_bytes(
                self.author,
                DM_SETTINGS_KEY.to_vec(),
                postcard::to_stdvec(settings)?,
            )
            .await
            .context("writing DM settings")?;
        Ok(())
    }

    pub async fn set_title(&self, title: Option<&str>) -> Result<()> {
        let mut s = self.settings().await?;
        s.conversation_title = title.map(str::to_string);
        s.updated_at_ms = now_ms();
        self.write(&s).await
    }

    pub async fn set_pinned(&self, pinned: bool) -> Result<()> {
        let mut s = self.settings().await?;
        s.pinned = pinned;
        s.updated_at_ms = now_ms();
        self.write(&s).await
    }

    pub async fn set_archived(&self, archived: bool) -> Result<()> {
        let mut s = self.settings().await?;
        s.archived = archived;
        s.updated_at_ms = now_ms();
        self.write(&s).await
    }

    pub async fn set_disappearing_ttl(&self, ttl_secs: Option<u64>) -> Result<()> {
        let mut s = self.settings().await?;
        s.disappearing_ttl_secs = ttl_secs;
        s.updated_at_ms = now_ms();
        self.write(&s).await
    }
}

fn decode<T: for<'de> Deserialize<'de>>(bytes: &[u8]) -> Result<T> {
    if bytes.len() > MAX_MESSAGE_BYTES {
        anyhow::bail!("conversation metadata record implausibly large");
    }
    Ok(postcard::from_bytes(bytes)?)
}
