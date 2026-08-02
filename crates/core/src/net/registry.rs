//! Global username registry, backed by `iroh-docs`.
//!
//! See `ARCHITECTURE.md` §3 for the full design and its honest limits —
//! short version: this gives you eventually-consistent, optimistic
//! uniqueness (first claim to propagate wins), not consensus-grade
//! guaranteed uniqueness. That's a real trade-off, not an oversight.
//!
//! ## Author identity, simplified
//! Earlier drafts of this file tried to derive a *deterministic*
//! iroh-docs `AuthorId` from the user's seed phrase, so the same author
//! signed your registry claim on every device. That needs an
//! import-a-specific-author-secret API this version of iroh-docs doesn't
//! clearly expose, so this version takes a simpler and equally correct
//! approach: create a random author once per install
//! (`docs.author_create()`), persist the returned `AuthorId` in local
//! settings, and reuse it across relaunches on *this* device. Recovering
//! your seed on a *new* device gets a fresh author there too — that's
//! fine, because the thing that actually needs to be stable is the
//! **value** you claim (`endpoint_id`, which *is* deterministic from the
//! seed — see `identity::seed`), not who signed the claim. `claim()`
//! below already treats "an existing entry whose `endpoint_id` matches
//! mine" as already-claimed regardless of which author wrote it.
//!
//! ## API accuracy note
//! `iroh-docs`'s client surface is one of the fastest-moving parts of the
//! iroh stack. Everything below follows the patterns confirmed against
//! the current docs.iroh.computer walkthrough and iroh-docs' own
//! rustdoc — `Docs::persistent(path).spawn(endpoint, blobs, gossip)`,
//! `docs.author_create()`, `docs.import(ticket)`, `doc.set_bytes(author,
//! key, value)`, `doc.get_one(Query::single_latest_per_key().key_exact(k))`
//! — but a couple of narrower specifics (the exact `Capability` enum
//! variant names, `NamespaceSecret`'s constructor name, and whether
//! `key_prefix` exists alongside `key_exact` on the query builder) are
//! marked `// VERIFY:` since I couldn't find a confirmed example of them.

use crate::protocol::message::MAX_MESSAGE_BYTES;
use anyhow::{Context, Result};
use iroh::{EndpointAddr, EndpointId};
use iroh_docs::protocol::Docs;
use iroh_docs::store::Query;
use iroh_docs::{AuthorId, Capability, DocTicket, NamespaceSecret};
use n0_future::StreamExt;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Every install joins the same namespace — this is the "phonebook"
/// everyone reads from and writes their own claim into. The 32 bytes here
/// are a fixed, public constant baked into the binary (regenerate once at
/// project genesis and hardcode the result — do NOT regenerate this per
/// build, or every install ends up in its own disconnected phonebook).
///
/// // VERIFY: replace with a real, generated-once secret before shipping.
/// Placeholder zeroes so the module compiles standalone.
const REGISTRY_NAMESPACE_SECRET: [u8; 32] = [0u8; 32];

fn registry_key(username: &str) -> Vec<u8> {
    format!("user/{}", username.trim().to_lowercase()).into_bytes()
}

/// Hex-encoded `EndpointId`s of long-running, always-on peers whose sole
/// relevant job is staying part of the registry sync mesh — the fix for
/// a real cold-start gap: a brand-new install has an empty
/// `Store::known_registry_peers` (it's never directly contacted anyone
/// yet), so with nothing here its local registry replica has *no* way to
/// learn about anyone else's username claims, ever, no matter how long
/// it runs — a fresh username check would see an already-taken name as
/// "available" simply because the replica that would say otherwise was
/// never synced. `EndpointId`s deliberately, not full tickets: `hint_peer`
/// (see `App::start`'s deferred-hint task, where this list is fed in
/// alongside `known_registry_peers`) already resolves a bare id via
/// on-demand discovery the same way every other peer hint does, so
/// there's no need to also carry rich address hints here — same
/// resolution path, not a special case.
///
/// Empty by default: this needs at least one real, long-running instance
/// of this app actually deployed and its endpoint id pasted in below to
/// do anything. Until then, fresh installs remain in the same
/// only-syncs-after-a-real-contact state described above — this constant
/// is the hook, not a fix that ships itself; add an entry once such a
/// node exists.
pub const BOOTSTRAP_REGISTRY_PEERS: &[&str] = &[
    // "abcdef0123...", // hex EndpointId of a long-running bootstrap instance
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsernameRecord {
    pub endpoint_id: [u8; 32],
    pub ticket: String,
    pub claimed_at_unix_ms: u64,
}

impl UsernameRecord {
    /// `addr` should be the claiming device's current `EndpointAddr` (see
    /// `App::my_addr`), not just its bare id — the whole point of storing
    /// `ticket` here is so someone who finds this record via username
    /// search gets the same instant, discovery-free connect a pasted
    /// ticket gives them (see `ticket.rs`'s module doc). Falls back to an
    /// id-only ticket if encoding the rich one somehow fails, rather than
    /// blocking the claim itself over it.
    fn new(addr: EndpointAddr) -> Self {
        let endpoint_id = addr.id; // confirmed via rustc: `EndpointAddr` has `id`/`addrs` fields.
        Self {
            endpoint_id: *endpoint_id.as_bytes(),
            ticket: crate::ticket::encode(addr)
                .unwrap_or_else(|_| crate::ticket::encode(endpoint_id.into()).unwrap_or_default()),
            claimed_at_unix_ms: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        }
    }

    pub fn encode(&self) -> Result<Vec<u8>> {
        Ok(postcard::to_stdvec(self)?)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() > MAX_MESSAGE_BYTES {
            anyhow::bail!("registry record implausibly large");
        }
        Ok(postcard::from_bytes(bytes)?)
    }
}

pub enum ClaimOutcome {
    /// Nobody else holds this name (as far as we can currently see) — our
    /// claim has been written.
    Claimed,
    /// Someone else already holds it, with a different endpoint. Pick
    /// another name.
    TakenBy(UsernameRecord),
}

/// Handle to the registry document, layered on an already-spawned `Docs`
/// engine (spawned in `app.rs` alongside the DM/contact/gossip/blobs
/// protocols, since it also needs registering on the same shared
/// `Router`). Owned by `App` alongside the DM/room state.
pub struct Registry {
    docs: Docs,
    blobs: iroh_blobs::api::Store,
    doc: iroh_docs::api::Doc, // VERIFY: exact handle type name for an open document in your pinned version
    author: AuthorId,
}

impl Registry {
    /// Join the well-known registry namespace on an already-running `Docs`
    /// engine.
    ///
    /// `existing_author` is an `AuthorId` persisted from a previous run of
    /// *this app on this device* (see module doc for why it doesn't need
    /// to come from the seed phrase); pass `None` on first run and persist
    /// the one returned so subsequent launches reuse it instead of
    /// accumulating a new author every time.
    ///
    /// `bootstrap_peers` seeds the `DocTicket`'s peer list. This matters a
    /// lot more than it looks: `iroh-docs` only syncs with peers it's
    /// actually told about (a `DocTicket` "contains both a key ... and a
    /// list of peers to join" — there's no global "find everyone in this
    /// namespace" discovery). Importing with an empty peer list is *not*
    /// harmless — it means this replica can never learn about anyone
    /// else's username claims, only ever see its own, no matter how long
    /// it runs. `App::start` passes the endpoint IDs of everyone we've
    /// ever directly connected to (`Store::known_registry_peers`) as a
    /// practical, infra-free stand-in for a bootstrap server: if any of
    /// them are themselves registry members (likely, since most people
    /// running this app will have claimed a username), we sync through
    /// them and transitively pick up everyone *they've* synced with too.
    pub async fn new(
        docs: Docs,
        blobs: iroh_blobs::api::Store,
        existing_author: Option<AuthorId>,
        bootstrap_peers: Vec<EndpointId>,
    ) -> Result<(Self, AuthorId)> {
        let author = match existing_author {
            Some(a) => a,
            None => docs.author_create().await.context("creating docs author")?,
        };

        // VERIFY: `NamespaceSecret::from_bytes` and `Capability::Write(..)`
        // exact names — see module doc.
        let namespace_secret = NamespaceSecret::from_bytes(&REGISTRY_NAMESPACE_SECRET);
        // Bare `EndpointId`s convert to `EndpointAddr`s with no known relay/
        // direct addresses yet — that's fine, the same on-demand DNS/Pkarr
        // discovery `ticket::decode` relies on for direct connects resolves
        // them (see that module's doc comment).
        let peers: Vec<EndpointAddr> = bootstrap_peers
            .into_iter()
            .map(EndpointAddr::from)
            .collect();
        let ticket = DocTicket::new(Capability::Write(namespace_secret), peers);
        let doc = docs
            .import(ticket)
            .await
            .context("joining registry namespace")?;

        Ok((
            Self {
                docs,
                blobs,
                doc,
                author,
            },
            author,
        ))
    }

    /// Best-effort: tell the already-open registry replica about one more
    /// peer worth syncing with, without waiting for the next app restart
    /// to pick up `Store::known_registry_peers`. Called opportunistically
    /// whenever we directly connect to someone new (see `app.rs`).
    ///
    /// Re-importing a ticket for a namespace we've already joined, just
    /// with an extra peer in its list, is the only peer-hinting handle
    /// this version of `iroh-docs`'s client surface clearly exposes (see
    /// module doc's "API accuracy note") — if a more direct "add sync
    /// peer" method exists in the pinned version, prefer it, but this is
    /// safe either way: errors here are logged and swallowed rather than
    /// surfaced, since the peer is already persisted for next launch
    /// regardless (`Store::remember_registry_peer`) and this is purely an
    /// optimization to shorten the wait.
    pub async fn hint_peer(&self, peer: EndpointId) {
        let namespace_secret = NamespaceSecret::from_bytes(&REGISTRY_NAMESPACE_SECRET);
        let ticket = DocTicket::new(
            Capability::Write(namespace_secret),
            vec![EndpointAddr::from(peer)],
        );
        if let Err(e) = self.docs.import(ticket).await {
            tracing::debug!(%peer, error = %e, "registry: live peer hint failed (will retry next launch)");
        }
    }

    /// Attempt to claim `username` for `my_addr`'s endpoint. Returns
    /// `Claimed` if no *other* endpoint currently holds it in our locally
    /// synced view, or `TakenBy` with the record that wins the conflict.
    pub async fn claim(&self, username: &str, my_addr: EndpointAddr) -> Result<ClaimOutcome> {
        let key = registry_key(username);
        let my_id = my_addr.id; // confirmed via rustc: `EndpointAddr` has `id`/`addrs` fields.

        if let Some(existing) = self.resolve_raw(&key).await? {
            if existing.endpoint_id != *my_id.as_bytes() {
                return Ok(ClaimOutcome::TakenBy(existing));
            }
            // Already our own prior claim (possibly under a different
            // author, e.g. after a crash — see module doc) — nothing to do.
            return Ok(ClaimOutcome::Claimed);
        }

        let record = UsernameRecord::new(my_addr);
        self.doc
            .set_bytes(self.author, key, record.encode()?)
            .await
            .context("writing username claim")?;

        Ok(ClaimOutcome::Claimed)
    }

    /// Resolve a username to its claimed record, if any is currently
    /// visible in the locally synced view of the registry. This is a local
    /// read — no network round trip — since sync happens continuously in
    /// the background via iroh-gossip as peers connect.
    pub async fn resolve(&self, username: &str) -> Result<Option<UsernameRecord>> {
        self.resolve_raw(&registry_key(username)).await
    }

    async fn resolve_raw(&self, key: &[u8]) -> Result<Option<UsernameRecord>> {
        // `single_latest_per_key` collapses conflicting claims from
        // multiple authors on the same key down to the one with the
        // latest entry timestamp — docs' own built-in resolution, which is
        // exactly the "earliest/latest wins" policy described in
        // ARCHITECTURE.md §3, so there's no need to hand-roll it here.
        let Some(entry) = self
            .doc
            .get_one(Query::single_latest_per_key().key_exact(key))
            .await
            .context("querying registry")?
        else {
            return Ok(None);
        };
        let bytes = self.blobs.blobs().get_bytes(entry.content_hash()).await?;
        Ok(Some(UsernameRecord::decode(&bytes)?))
    }

    /// Username autocomplete/search as the user types in the sidebar's
    /// "find someone" box. Prefix-scans the locally synced view.
    pub async fn search_prefix(&self, prefix: &str) -> Result<Vec<String>> {
        let prefix_key = format!("user/{}", prefix.trim().to_lowercase()).into_bytes();
        // VERIFY: `key_prefix` alongside the confirmed `key_exact` — not
        // directly confirmed in available docs, but follows the same
        // builder pattern.
        let entries = self
            .doc
            .get_many(Query::single_latest_per_key().key_prefix(prefix_key))
            .await
            .context("prefix search on registry")?;
        let mut entries = Box::pin(entries);

        let mut names = Vec::new();
        while let Some(entry) = entries.next().await {
            let entry = entry?;
            if let Ok(key_str) = std::str::from_utf8(entry.key()) {
                if let Some(name) = key_str.strip_prefix("user/") {
                    names.push(name.to_string());
                }
            }
        }
        Ok(names)
    }
}
