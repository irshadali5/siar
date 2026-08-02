//! Application core. Owns the `Endpoint`, the `Router` that dispatches
//! incoming connections by ALPN, `Gossip` (rooms + registry sync),
//! `FsStore` (blobs — files), the username `Registry`, and the sqlite
//! `Store` (contacts/messages/settings) that the UI layer drives.
//!
//! v2 adds three ALPN handlers to the router alongside the original DM
//! handler: the contact request/accept protocol, iroh-gossip (needed by
//! both rooms and the registry's sync), and iroh-docs itself. Same
//! background-task design as v1: slow network operations run on
//! `tokio::spawn`ed tasks against cheap cloned handles and report back
//! through a channel, so the UI render loop never awaits a network call
//! directly.

use crate::identity;
use crate::net::contacts::{self, ContactEvent, ContactProtocol, ALPN as CONTACT_ALPN};
use crate::net::conv_docs::{DmDoc, RoomDoc};
use crate::net::registry::Registry;
use crate::protocol::dm::{DmEvent, DmProtocol, DmSession, ALPN as DM_ALPN};
use crate::protocol::message::Envelope;
use crate::store::{ContactState, Conversation, MessageKind, Store};
use crate::{
    gossip::room::{Room, RoomEvent},
    ticket,
};
use anyhow::{Context, Result};
use iroh::endpoint::presets;
use iroh::protocol::Router;
use iroh::{Endpoint, EndpointAddr, EndpointId, SecretKey};
use iroh_blobs::store::fs::FsStore;
use iroh_blobs::BlobsProtocol;
use iroh_docs::protocol::Docs;
use iroh_docs::AuthorId;
use iroh_gossip::Gossip;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tracing::warn;

pub enum AppEvent {
    Dm(DmEvent),
    Room(RoomEvent),
    Contact(ContactEvent),
    Call(crate::net::calls::CallEvent),
}

pub struct App {
    pub my_name: String,
    pub my_id: EndpointId,
    pub my_username: Option<String>,
    relay_ok: bool,
    endpoint: Endpoint,
    gossip: Gossip,
    blobs: FsStore,
    blobs_store: iroh_blobs::api::Store,
    registry: Arc<Registry>,
    // Shared handle onto the same `iroh-docs` engine the registry uses, plus
    // the persisted local author, so `RoomDoc`/`DmDoc` (net::conv_docs) can
    // open per-conversation metadata namespaces on demand rather than every
    // conversation paying the cost of one at startup. See ARCHITECTURE.md
    // §11 (ADR: conversation metadata over iroh-docs).
    docs: Docs,
    docs_author: AuthorId,
    room_tx: UnboundedSender<RoomEvent>,
    _router: Router, // keep alive: dropping it stops accepting connections
    store: Arc<Store>,
    dm_sessions: HashMap<EndpointId, DmSession>,
    rooms: HashMap<String, Room>,
    // Opened lazily on first use (join/create for rooms, first DM screen
    // view for DMs) and cached thereafter — see `room_doc`/`dm_doc` below.
    room_docs: HashMap<String, RoomDoc>,
    dm_docs: HashMap<EndpointId, DmDoc>,
    call_tx: UnboundedSender<crate::net::calls::CallEvent>,
    // Plain std `Mutex`, not routed through the Dioxus `Signal` this whole
    // `App` lives behind: this only ever needs a quick, synchronous
    // take-or-set, never held across an `.await` — using `&self` (not
    // `&mut self`) here specifically avoids the class of bug fixed
    // elsewhere in this codebase (see the `AlreadyBorrowedMut` panic note
    // on the conversation-info load path), rather than reintroducing it
    // for calls.
    active_call_hangup: std::sync::Mutex<Option<tokio::sync::oneshot::Sender<()>>>,
    mesh: Arc<crate::net::mesh::MeshManager>,
}

/// Bound for the two setup steps in `App::start` that can plausibly hang
/// when there's no network — endpoint bind and the local `iroh-docs`
/// engine spin-up — same rationale as every other `NET_TIMEOUT` in this
/// codebase (`protocol::dm`, `net::contacts`, `gossip::room`): name which
/// step got stuck instead of leaving it to a much coarser outer timeout
/// (`ui::spawn_boot::BOOT_TIMEOUT`) to eventually notice with no detail.
const STARTUP_STEP_TIMEOUT: Duration = Duration::from_secs(15);

/// Shorter bound specifically for the username-registry sync
/// (`net::registry::Registry::new`) — see the comment at its call site in
/// `App::start` for why this one gets a retry-without-cached-author
/// fallback instead of failing immediately.
const REGISTRY_STEP_TIMEOUT: Duration = Duration::from_secs(10);

impl App {
    pub async fn start(
        data_dir: PathBuf,
        secret_key: SecretKey,
        my_name: String,
        relay_timeout: Duration,
    ) -> Result<(Self, UnboundedReceiver<AppEvent>)> {
        let store = Arc::new(Store::open(
            &data_dir,
            &crate::identity::storage_key(&data_dir)?,
        )?);
        let my_username = store.get_setting("claimed_username")?;

        tracing::info!("App::start: binding endpoint");
        // Same rationale as every other network call in this codebase
        // (`protocol::dm::NET_TIMEOUT`, `net::contacts::NET_TIMEOUT`,
        // `gossip::room::NET_TIMEOUT`): bound it, name which step failed.
        // This one matters more than most — a hang here previously wasn't
        // caught by anything narrower than the outer 45s
        // `ui::spawn_boot::BOOT_TIMEOUT`, which just said "didn't finish"
        // with no indication which of bind/docs/registry was actually
        // stuck. `.bind()` can plausibly hang offline if it does any
        // network-dependent setup (e.g. resolving relay/discovery
        // endpoints) before returning, rather than deferring that to the
        // relay-wait step below, which is already explicitly bounded.
        let endpoint = tokio::time::timeout(
            STARTUP_STEP_TIMEOUT,
            Endpoint::builder(presets::N0).secret_key(secret_key).bind(),
        )
        .await
        .map_err(|_| {
            anyhow::anyhow!(
                "binding the network endpoint timed out after {STARTUP_STEP_TIMEOUT:?} — \
                 check your network connection and try again"
            )
        })??;
        tracing::info!("App::start: endpoint bound, opening blob store");

        let blobs_path = data_dir.join("blobs");
        std::fs::create_dir_all(&blobs_path)?;
        // VERIFY: `FsStore::load`/`FsStore::open` exact constructor name
        // for the pinned iroh-blobs version.
        let blobs = FsStore::load(&blobs_path).await?;
        let blobs_store: iroh_blobs::api::Store = blobs.clone().into();
        tracing::info!("App::start: blob store ready");

        let (dm_tx, mut dm_rx) = unbounded_channel();
        let (room_tx, mut room_rx) = unbounded_channel::<RoomEvent>();
        let (contact_tx, mut contact_rx) = unbounded_channel::<ContactEvent>();
        let (call_tx, mut call_rx) = unbounded_channel::<crate::net::calls::CallEvent>();
        let (app_tx, app_rx) = unbounded_channel();

        // Fan all four event sources into one channel the UI loop selects on.
        {
            let app_tx = app_tx.clone();
            tokio::spawn(async move {
                while let Some(ev) = dm_rx.recv().await {
                    if app_tx.send(AppEvent::Dm(ev)).is_err() {
                        break;
                    }
                }
            });
        }
        {
            let app_tx = app_tx.clone();
            tokio::spawn(async move {
                while let Some(ev) = room_rx.recv().await {
                    if app_tx.send(AppEvent::Room(ev)).is_err() {
                        break;
                    }
                }
            });
        }
        {
            let app_tx = app_tx.clone();
            tokio::spawn(async move {
                while let Some(ev) = contact_rx.recv().await {
                    if app_tx.send(AppEvent::Contact(ev)).is_err() {
                        break;
                    }
                }
            });
        }
        {
            tokio::spawn(async move {
                while let Some(ev) = call_rx.recv().await {
                    if app_tx.send(AppEvent::Call(ev)).is_err() {
                        break;
                    }
                }
            });
        }

        let dm_protocol = DmProtocol::new(dm_tx.clone());
        let contact_protocol = ContactProtocol::new(store.clone(), contact_tx);
        let video_handoff: crate::net::calls::VideoHandoff = Arc::new(std::sync::Mutex::new(None));
        let call_protocol =
            crate::net::calls::CallProtocol::new(call_tx.clone(), video_handoff.clone());
        let video_call_protocol = crate::net::calls::video::VideoCallProtocol::new(video_handoff);
        let gossip = Gossip::builder().spawn(endpoint.clone());
        tracing::info!("App::start: gossip spawned, opening iroh-docs engine");

        let docs_path = data_dir.join("docs");
        std::fs::create_dir_all(&docs_path)?;
        let docs = tokio::time::timeout(
            STARTUP_STEP_TIMEOUT,
            Docs::persistent(docs_path).spawn(
                endpoint.clone(),
                blobs_store.clone(),
                gossip.clone(),
            ),
        )
        .await
        .map_err(|_| {
            anyhow::anyhow!(
                "starting the local sync engine timed out after {STARTUP_STEP_TIMEOUT:?}"
            )
        })??;
        tracing::info!("App::start: iroh-docs engine ready, resolving username registry");

        // VERIFY: assumes `iroh_docs::AuthorId` implements Serialize/Deserialize
        // (very likely, since it round-trips through DocTicket/entries
        // elsewhere in the crate, but not directly confirmed here).
        let existing_author = store
            .get_setting("docs_author_id")?
            .and_then(|hex_str| data_encoding::HEXLOWER.decode(hex_str.as_bytes()).ok())
            .and_then(|bytes| postcard::from_bytes(&bytes).ok());

        // Unlike the two steps above, a stuck registry sync shouldn't be
        // able to keep the whole app from opening — the registry is only
        // needed for claiming/searching *usernames*; sending messages to
        // already-known contacts, and even opening room/DM metadata via
        // `net::conv_docs`, don't depend on it. So this one gets a
        // shorter timeout and, on expiry, a warning instead of a hard
        // failure: `Registry::new` is retried with no cached author (safe
        // — it's `Option<AuthorId>`, not required), and if that *also*
        // times out, the whole thing is genuinely offline enough that
        // nothing past this point would work either, so it does still
        // propagate at that point.
        // Every endpoint we've ever directly connected to — see
        // `Store::remember_registry_peer` and the `bootstrap_peers` doc
        // comment on `Registry::new`. Deliberately *not* passed into the
        // `Registry::new` call below: an earlier version did, and it's a
        // likely culprit behind a real "startup didn't finish within
        // 150s" report — `docs.import()` with a peer in its list may try
        // to actually reach that peer before this resolves, and a stale
        // entry (a peer that's since gone offline) turns that into a
        // hang neither `REGISTRY_STEP_TIMEOUT` retry below fully protects
        // against, since both attempts would carry the exact same stale
        // list. Startup should never be able to hang on a peer that isn't
        // there anymore. Instead: import with an empty list here (fast,
        // local, same as the original design), then hint these peers in
        // afterward on a detached task, using the exact same best-effort
        // `hint_peer` the rest of the app already uses for this at
        // runtime — a few seconds' extra delay before search converges
        // is a fine trade for startup that can never block on it.
        let known_peers: Vec<EndpointId> = store
            .known_registry_peers()
            .unwrap_or_default()
            .iter()
            .filter_map(|hex_str| parse_hex(hex_str).ok())
            .collect();

        let (registry, author) = match tokio::time::timeout(
            REGISTRY_STEP_TIMEOUT,
            Registry::new(
                docs.clone(),
                blobs_store.clone(),
                existing_author,
                Vec::new(),
            ),
        )
        .await
        {
            Ok(result) => result?,
            Err(_) => {
                tracing::warn!(
                    "username registry sync timed out after {REGISTRY_STEP_TIMEOUT:?} — \
                     retrying once without a cached author before giving up"
                );
                tokio::time::timeout(
                    REGISTRY_STEP_TIMEOUT,
                    Registry::new(docs.clone(), blobs_store.clone(), None, Vec::new()),
                )
                .await
                .map_err(|_| {
                    anyhow::anyhow!(
                        "username registry sync timed out twice ({REGISTRY_STEP_TIMEOUT:?} each) — \
                         this needs at least one relay-reachable peer on first launch; \
                         check your connection and try again"
                    )
                })??
            }
        };
        tracing::info!("App::start: registry ready, spawning protocol router");
        let registry = Arc::new(registry);
        let bootstrap_peers: Vec<EndpointId> = crate::net::registry::BOOTSTRAP_REGISTRY_PEERS
            .iter()
            .filter_map(|hex_str| parse_hex(hex_str).ok())
            .collect();
        if !known_peers.is_empty() || !bootstrap_peers.is_empty() {
            let registry_for_hints = registry.clone();
            tokio::spawn(async move {
                for peer in known_peers.into_iter().chain(bootstrap_peers) {
                    registry_for_hints.hint_peer(peer).await;
                }
            });
        }
        if let Ok(bytes) = postcard::to_stdvec(&author) {
            let _ = store.set_setting("docs_author_id", &data_encoding::HEXLOWER.encode(&bytes));
        }
        // `registry` is already `Arc<Registry>` at this point (wrapped
        // just above, before the hint-peers task was spawned off it).
        // Kept around (cheap clone — see `net::registry`'s own use of
        // `docs.clone()` above) so `net::conv_docs::{RoomDoc, DmDoc}` can
        // open per-conversation metadata namespaces later; the same
        // `author` persisted for the registry is reused here rather than
        // minting a second one, since both are just "this device's local
        // iroh-docs writer identity," not anything conversation-specific.
        let docs_for_router = docs.clone();

        let router = Router::builder(endpoint.clone())
            .accept(DM_ALPN, dm_protocol)
            .accept(CONTACT_ALPN, contact_protocol)
            .accept(crate::net::calls::ALPN, call_protocol)
            .accept(crate::net::calls::video::ALPN, video_call_protocol)
            .accept(iroh_gossip::ALPN, gossip.clone())
            // VERIFY: blobs ALPN constant name/path for the pinned version
            // (`iroh_blobs::ALPN`), and `BlobsProtocol::new(&blobs, None)`
            // signature.
            .accept(iroh_blobs::ALPN, BlobsProtocol::new(&blobs, None))
            .accept(iroh_docs::ALPN, docs_for_router)
            .spawn();
        tracing::info!("App::start: router spawned, waiting up to {relay_timeout:?} for relay");

        let relay_ok = match tokio::time::timeout(relay_timeout, endpoint.online()).await {
            Ok(()) => true,
            Err(_) => {
                warn!(
                    "no relay after {relay_timeout:?} — continuing anyway (direct/LAN peers may \
                     still work; DMs and registry sync needing relay assistance will fail until \
                     connectivity recovers)."
                );
                false
            }
        };
        tracing::info!(relay_ok, "App::start: done");
        let my_id = endpoint.id();

        // Offline mesh (net::mesh): constructed unconditionally — cheap,
        // does nothing until `start()` is called — and started now only
        // if the user already had it enabled from a previous session.
        // Mesh-delivered payloads are decoded and re-injected through
        // the *same* `dm_tx` the QUIC DM protocol handler uses, so a
        // message that arrived over BLE/LAN goes through the identical
        // `AppEvent::Dm` → `record_incoming_dm` path as one that arrived
        // over QUIC — see net/mesh/mod.rs's module doc for why.
        let (mesh, mut mesh_inbound_rx) = crate::net::mesh::MeshManager::new(my_id);
        {
            let dm_tx = dm_tx.clone();
            tokio::spawn(async move {
                while let Some((from, payload)) = mesh_inbound_rx.recv().await {
                    match Envelope::decode(&payload) {
                        Ok(envelope) => {
                            if dm_tx.send(DmEvent::Received { from, envelope }).is_err() {
                                break;
                            }
                        }
                        Err(err) => tracing::debug!(?err, "mesh: dropped undecodable payload"),
                    }
                }
            });
        }
        if store.offline_mesh_enabled() {
            mesh.clone().start().await;
        }

        let app = Self {
            my_name,
            my_id,
            my_username,
            relay_ok,
            endpoint,
            gossip,
            blobs,
            blobs_store,
            registry,
            docs,
            docs_author: author,
            room_tx,
            _router: router,
            store,
            dm_sessions: HashMap::new(),
            rooms: HashMap::new(),
            room_docs: HashMap::new(),
            dm_docs: HashMap::new(),
            call_tx,
            active_call_hangup: std::sync::Mutex::new(None),
            mesh,
        };
        Ok((app, app_rx))
    }

    pub fn my_ticket(&self) -> String {
        // `encode` is now fallible (postcard-serializes the full address,
        // not just the raw id — see `ticket.rs`'s module doc). It can only
        // realistically fail on a serialization bug, not anything the user
        // did, so degrade to the old bare-id ticket rather than making
        // "open Settings" a place that can show an error.
        ticket::encode(self.my_addr())
            .unwrap_or_else(|_| ticket::encode(self.my_id.into()).unwrap_or_default())
    }

    pub fn relay_ok(&self) -> bool {
        self.relay_ok
    }

    /// Live counters for Settings' Network tab (peers seen, envelopes
    /// relayed, which transports are currently active).
    pub fn mesh_status(&self) -> Arc<crate::net::mesh::MeshStatus> {
        self.mesh.status()
    }

    /// Cheap `Arc` clone — lets callers (notably the Settings toggle in
    /// `siar-ui`) start/stop the mesh from inside a `spawn`ed task
    /// without holding a `Signal` read guard across an `.await` (the
    /// same `AlreadyBorrowedMut` hazard already documented elsewhere in
    /// this codebase).
    pub fn mesh(&self) -> Arc<crate::net::mesh::MeshManager> {
        self.mesh.clone()
    }

    pub fn offline_mesh_enabled(&self) -> bool {
        self.store.offline_mesh_enabled()
    }

    /// Persists the setting and starts/stops `MeshManager` to match —
    /// the single call Settings' toggle makes; nothing else needs to
    /// know mesh even exists to flip it on or off.
    pub async fn set_offline_mesh_enabled(&self, value: bool) -> Result<()> {
        self.store.set_offline_mesh_enabled(value)?;
        if value {
            self.mesh.clone().start().await;
        } else {
            self.mesh.stop();
        }
        Ok(())
    }

    /// Best-effort fallback send: floods `envelope` over whichever mesh
    /// transports are active. Returns `false` (nothing sent) when the
    /// feature is off or no transport is currently running. There's no
    /// delivery acknowledgement over mesh, unlike the QUIC DM path's
    /// `Body::Ack` — flooding doesn't have a return path baked in, so
    /// this is intentionally "at least tried," not "confirmed
    /// delivered."
    ///
    /// A convenience wrapper around `mesh()`/`offline_mesh_enabled()`
    /// for callers that can afford to hold `&self` across the
    /// `.await` — `siar-ui`'s `spawn_dm_send` can't (a `Signal` read
    /// guard held across an await is the `AlreadyBorrowed` hazard
    /// documented on `mesh()`), so it calls those two directly via its
    /// own `try_mesh_send` instead of this method.
    pub async fn mesh_send(&self, envelope: &Envelope) -> bool {
        if !self.store.offline_mesh_enabled() {
            return false;
        }
        let Ok(bytes) = envelope.encode() else {
            return false;
        };
        self.mesh.send(bytes).await;
        true
    }

    // ---- Cheap handles for background tasks ----

    pub fn endpoint(&self) -> Endpoint {
        self.endpoint.clone()
    }

    pub fn gossip(&self) -> Gossip {
        self.gossip.clone()
    }

    pub fn blobs(&self) -> FsStore {
        self.blobs.clone()
    }

    pub fn registry(&self) -> Arc<Registry> {
        self.registry.clone()
    }

    pub fn room_tx(&self) -> UnboundedSender<RoomEvent> {
        self.room_tx.clone()
    }

    pub fn existing_dm_session(&self, peer_id: EndpointId) -> Option<DmSession> {
        self.dm_sessions.get(&peer_id).cloned()
    }

    /// A cloneable snapshot of every currently-open DM session, for the
    /// periodic keepalive sweep (`ui::spawn_dm_keepalive`). Returned as an
    /// owned `Vec` rather than iterated in place so the caller isn't
    /// holding a lock on `App` across a batch of `.await`s — same reason
    /// most other network-touching UI code snapshots what it needs first,
    /// then drops the read guard before the actual network calls.
    pub fn dm_sessions_snapshot(&self) -> Vec<(EndpointId, DmSession)> {
        self.dm_sessions
            .iter()
            .map(|(id, s)| (*id, s.clone()))
            .collect()
    }

    pub fn existing_room(&self, name: &str) -> Option<Room> {
        self.rooms.get(name).cloned()
    }

    // ---- Username registry ----

    /// Claim a username at onboarding time (or later, from settings). On
    /// success, caches it locally so we don't have to re-resolve our own
    /// name from the registry on every launch.
    /// Whether this device's claimed username still resolves back to this
    /// device, per the registry's own conflict-resolution rule (see
    /// `Registry::resolve_raw`'s doc). `Ok(true)` covers both "nothing
    /// claimed yet" and "still mine" — `Ok(false)` is the one case that
    /// actually means something: another device's claim for the same
    /// name is now the one the registry resolves to. Checked periodically
    /// (see `ui::spawn_disappearing_sweep`, which already runs every 30s
    /// for an unrelated reason and was the natural place to piggyback
    /// this on) rather than once at claim time, since the whole reason
    /// this check can ever return `false` is that the conflicting claim
    /// wasn't visible yet when this device made its own.
    pub async fn username_still_valid(&self) -> Result<bool> {
        let Some(username) = &self.my_username else {
            return Ok(true);
        };
        match self.registry.resolve(username).await? {
            Some(record) if record.endpoint_id != *self.my_id.as_bytes() => Ok(false),
            _ => Ok(true),
        }
    }

    pub async fn claim_username(
        &mut self,
        username: &str,
    ) -> Result<crate::net::registry::ClaimOutcome> {
        let outcome = self.registry.claim(username, self.my_addr()).await?;
        if matches!(outcome, crate::net::registry::ClaimOutcome::Claimed) {
            self.store.set_setting("claimed_username", username)?;
            self.my_username = Some(username.to_string());
        }
        Ok(outcome)
    }

    // ---- Contacts: request/accept state machine ----

    /// Send a contact request to a resolved username (registry search
    /// result). Marks the local row `PendingOut` immediately so the UI
    /// reflects it before the network call even completes. For a pasted
    /// ticket, use `request_contact_via_addr` instead — it carries the
    /// peer's actual address, not just their id.
    /// Clone of the sender that feeds `AppEvent::Call` — needed by the UI
    /// layer to pass into `net::calls::place_call`/the incoming-call
    /// decision flow, both of which report their progress on this same
    /// channel regardless of who's calling whom.
    pub fn call_events_sender(&self) -> UnboundedSender<crate::net::calls::CallEvent> {
        self.call_tx.clone()
    }

    /// Record the hangup trigger for whichever call is now active, so a
    /// later `hang_up_active_call` can fire it. Overwrites any previous
    /// one — this app only supports one call at a time.
    pub fn set_active_call_hangup(&self, tx: tokio::sync::oneshot::Sender<()>) {
        *self.active_call_hangup.lock().unwrap() = Some(tx);
    }

    /// Ends whatever call is currently active, if any. Safe to call even
    /// if there isn't one (e.g. a stale UI click after the call already
    /// ended on its own) — just a no-op.
    pub fn hang_up_active_call(&self) {
        if let Some(tx) = self.active_call_hangup.lock().unwrap().take() {
            let _ = tx.send(());
        }
    }

    pub async fn request_contact(&self, to: EndpointId, note: &str) -> Result<()> {
        self.store
            .upsert_contact(&hex(to), None, &short_id(to), ContactState::PendingOut)?;
        let result = request_contact_with_retry(
            &self.endpoint,
            to,
            None,
            self.my_id,
            self.my_username.clone(),
            &self.my_name,
            note,
        )
        .await;
        if result.is_ok() {
            self.remember_and_hint_peer(to).await;
        }
        result
    }

    /// Same as `request_contact`, but for a ticket-initiated connect,
    /// where we have the peer's actual relay/direct addresses on hand
    /// (decoded straight from the ticket) instead of only their bare id.
    /// This is what actually fixes "contact request connect timed out"
    /// for freshly pasted tickets — see `ticket.rs`'s module doc and
    /// `net::contacts::send_request`'s `addr_hint` for the full story.
    pub async fn request_contact_via_addr(&self, addr: EndpointAddr, note: &str) -> Result<()> {
        let to = addr.id; // `EndpointAddr::id` — confirmed via rustc, not just guessed (see ticket.rs).
        self.store
            .upsert_contact(&hex(to), None, &short_id(to), ContactState::PendingOut)?;
        let result = request_contact_with_retry(
            &self.endpoint,
            to,
            Some(addr),
            self.my_id,
            self.my_username.clone(),
            &self.my_name,
            note,
        )
        .await;
        if result.is_ok() {
            self.remember_and_hint_peer(to).await;
        }
        result
    }

    /// This device's current shareable address — relay URL and any known
    /// direct addresses, not just the bare id. Used to generate the "your
    /// ticket" display in Settings, so a freshly pasted ticket can skip
    /// discovery entirely on the receiving end. (Onboarding's own ticket
    /// preview is computed separately, straight from the seed, before this
    /// method's `Endpoint` even exists — see `ui::onboarding`.) See
    /// `ticket.rs`'s module doc.
    pub fn my_addr(&self) -> EndpointAddr {
        // VERIFY: `Endpoint::addr()` — renamed from `node_addr()`, and per
        // iroh's own 0.93 changelog this is "infallible and instantaneous",
        // so no need to await anything here even right after `bind()`.
        self.endpoint.addr()
    }

    pub async fn accept_contact(&self, from: EndpointId) -> Result<()> {
        self.store
            .set_contact_state(&hex(from), ContactState::Accepted)?;
        let result = accept_contact_with_retry(
            &self.endpoint,
            from,
            self.my_id,
            self.my_username.clone(),
            &self.my_name,
        )
        .await;
        if result.is_ok() {
            self.remember_and_hint_peer(from).await;
        }
        result
    }

    /// Someone we've now directly reached — worth remembering as a
    /// registry bootstrap peer for future launches, and worth telling the
    /// *already-open* registry replica about right away so a search that
    /// just failed might succeed a few seconds later without a restart.
    /// See `Store::remember_registry_peer` and `Registry::hint_peer`.
    pub async fn remember_and_hint_peer(&self, peer: EndpointId) {
        let _ = self.store.remember_registry_peer(&hex(peer));
        self.registry.hint_peer(peer).await;
    }

    pub async fn reject_contact(&self, from: EndpointId) -> Result<()> {
        self.store.remove_contact(&hex(from))?;
        contacts::send_reject(&self.endpoint, from, self.my_id).await
    }

    pub fn block_contact(&self, peer: EndpointId) -> Result<()> {
        self.store
            .set_contact_state(&hex(peer), ContactState::Blocked)
    }

    pub fn pending_incoming_requests(&self) -> Result<Vec<crate::store::Contact>> {
        self.store.pending_incoming()
    }

    pub fn accepted_contacts(&self) -> Result<Vec<crate::store::Contact>> {
        self.store.accepted_contacts()
    }

    pub fn recent_calls(&self, limit: u32) -> Result<Vec<crate::store::CallLogEntry>> {
        self.store.recent_calls(limit)
    }

    pub fn log_call(
        &self,
        peer_id: EndpointId,
        peer_name: &str,
        direction: crate::store::CallDirection,
        outcome: crate::store::CallOutcome,
        started_at_ms: i64,
        duration_secs: i64,
    ) -> Result<()> {
        self.store.log_call(
            &hex(peer_id),
            peer_name,
            direction,
            outcome,
            started_at_ms,
            duration_secs,
        )
    }

    pub fn active_statuses(&self) -> Result<Vec<crate::store::StatusEntry>> {
        self.store.active_statuses(now_unix_ms())
    }

    pub fn prune_expired_statuses(&self) -> Result<usize> {
        self.store.prune_expired_statuses(now_unix_ms())
    }

    pub fn is_accepted(&self, peer: EndpointId) -> bool {
        self.store.is_accepted(&hex(peer))
    }

    /// Manually mark/unmark a contact as verified — see the doc on
    /// `store::Contact::verified`. UI entry point: the "Mark verified"
    /// toggle in the DM tab of `ui::ConvInfoPanel`.
    pub fn set_contact_verified(&self, peer: EndpointId, verified: bool) -> Result<()> {
        self.store.set_verified(&hex(peer), verified)
    }

    // ---- Fast, non-network mutations applied once a background task
    // ---- reports a result back to the UI loop ----

    pub fn commit_dm_session(&mut self, peer_id: EndpointId, session: DmSession) {
        self.dm_sessions.insert(peer_id, session);
    }

    pub fn drop_dm_session(&mut self, peer_id: EndpointId) {
        self.dm_sessions.remove(&peer_id);
    }

    pub fn commit_room(&mut self, name: &str, room: Room) {
        self.rooms.insert(name.to_string(), room);
    }

    pub fn log_outgoing_dm(
        &self,
        peer_id: EndpointId,
        text: &str,
        sent_unix_ms: i64,
        expires_at_unix_ms: Option<i64>,
        envelope_id: u64,
        reply_to_envelope_id: Option<u64>,
    ) -> Result<()> {
        self.store.log_message(
            &Conversation::Dm(hex(peer_id)),
            &hex(self.my_id),
            &self.my_name,
            text,
            &MessageKind::Text,
            sent_unix_ms,
            true,
            expires_at_unix_ms,
            envelope_id,
            reply_to_envelope_id,
        )
    }

    pub fn log_outgoing_room(
        &self,
        name: &str,
        text: &str,
        sent_unix_ms: i64,
        envelope_id: u64,
        reply_to_envelope_id: Option<u64>,
    ) -> Result<()> {
        self.store.log_message(
            &Conversation::Room(name.to_string()),
            &hex(self.my_id),
            &self.my_name,
            text,
            &MessageKind::Text,
            sent_unix_ms,
            true,
            None, // no disappearing-message policy for rooms yet — see ARCHITECTURE.md
            envelope_id,
            reply_to_envelope_id,
        )
    }

    pub fn log_outgoing_file(
        &self,
        conversation: &Conversation,
        prepared: &crate::net::transfer::PreparedFile,
        sent_unix_ms: i64,
        envelope_id: u64,
        reply_to_envelope_id: Option<u64>,
    ) -> Result<()> {
        self.store.log_message(
            conversation,
            &hex(self.my_id),
            &self.my_name,
            &prepared.name,
            &MessageKind::File {
                name: prepared.name.clone(),
                hash: prepared.hash.to_string(),
                size_bytes: prepared.size_bytes,
                compressed: prepared.compressed,
            },
            sent_unix_ms,
            true,
            None, // file-attachment expiry isn't wired up yet, text-only for now
            envelope_id,
            reply_to_envelope_id,
        )
    }

    // ---- Startup / retry helpers ----

    /// Fast, non-network insert of an already-opened `RoomDoc` into the
    /// cache — the commit half of the connect-then-commit split used
    /// throughout `App` (see the "Fast, non-network mutations" section
    /// below), so a caller holding a UI lock (e.g. a Dioxus `Signal` write
    /// guard) is never stuck holding it across the network I/O that
    /// opening/syncing a doc involves. Pair with
    /// `ensure_room_metadata_standalone` — see its doc for why this is a
    /// free function rather than an `&mut self` method.
    pub fn commit_room_doc(&mut self, name: &str, doc: RoomDoc) {
        self.room_docs.insert(name.to_string(), doc);
    }

    /// Same as `commit_room_doc`, for DMs.
    pub fn commit_dm_doc(&mut self, peer_id: EndpointId, doc: DmDoc) {
        self.dm_docs.insert(peer_id, doc);
    }

    pub fn docs(&self) -> Docs {
        self.docs.clone()
    }

    pub fn blobs_store(&self) -> iroh_blobs::api::Store {
        self.blobs_store.clone()
    }

    pub fn docs_author(&self) -> AuthorId {
        self.docs_author
    }

    pub fn known_room_names(&self) -> Result<Vec<String>> {
        self.store.distinct_rooms()
    }

    // ---- Conversation metadata (`net::conv_docs`) — title/membership for
    // ---- rooms, shared settings for DMs. Never message content: see that
    // ---- module's doc comment for the split with `store.rs`.
    //
    // There used to be `&mut self` accessors here (`room_meta`,
    // `room_members`, `set_room_title`, `remove_room_member`,
    // `dm_settings`, `set_dm_*`) that opened/cached a doc and awaited the
    // read/write internally. They're gone: every caller held `ui.core`'s
    // Signal write-lock across that `.await` to get the `&mut self`, on
    // the (reasonable-sounding, but wrong) theory that the doc was always
    // already cached by then so the await would be "fast." Fast doesn't
    // mean zero-yield-points, and a background task (e.g. the DM
    // keepalive sweep) touching the same `Signal` during that window is
    // enough to panic with `AlreadyBorrowedMut` — which is exactly what
    // happened when this file still had those methods.
    //
    // The fix: every caller in `ui::mod` now opens its own throwaway
    // `RoomDoc`/`DmDoc` via the standalone `RoomDoc::open`/`DmDoc::open`
    // (same pattern `ensure_room_metadata_standalone` above already used
    // correctly for the very first open) and calls straight through to
    // it — no `App`, no `Signal`, nothing to hold a lock on during the
    // await at all. `commit_room_doc`/`commit_dm_doc` above still exist
    // purely so the message-send fast path can reuse an already-open
    // handle instead of reopening one on every send.

    /// Persists someone's status update — a separate table
    /// (`Store::upsert_status`) from chat history, since a status isn't
    /// part of any one conversation and always expires (see
    /// `protocol::message::Body::Status`'s doc). `expires_at_ms` should
    /// always be `Some` for a real status; falls back to a 24h default
    /// from receipt time on the off chance it isn't, rather than storing
    /// something that never expires.
    // The wire Status body is already the natural aggregate; keeping its
    // three optional media descriptors explicit here avoids inventing a
    // second near-identical domain type only to satisfy an argument count.
    #[allow(clippy::too_many_arguments)]
    pub fn record_incoming_status(
        &self,
        from: EndpointId,
        peer_name: &str,
        text: &str,
        image: Option<crate::protocol::message::StatusImage>,
        video: Option<crate::protocol::message::StatusVideo>,
        audio: Option<crate::protocol::message::StatusAudio>,
        expires_at_ms: Option<u64>,
    ) -> Result<()> {
        let now = now_unix_ms();
        let expires = expires_at_ms
            .map(|v| v as i64)
            .unwrap_or(now + 24 * 3600 * 1000);
        let (image_hash, image_size_bytes) = match &image {
            Some(img) => (Some(img.blake3_hash.as_str()), Some(img.size_bytes)),
            None => (None, None),
        };
        let (video_hash, video_size_bytes) = match &video {
            Some(v) => (Some(v.blake3_hash.as_str()), Some(v.size_bytes)),
            None => (None, None),
        };
        let (audio_hash, audio_size_bytes) = match &audio {
            Some(a) => (Some(a.blake3_hash.as_str()), Some(a.size_bytes)),
            None => (None, None),
        };
        self.store.upsert_status(
            &hex(from),
            peer_name,
            text,
            now,
            expires,
            image_hash,
            image_size_bytes,
            video_hash,
            video_size_bytes,
            audio_hash,
            audio_size_bytes,
        )
    }

    /// Broadcast a status update to every accepted contact, over each
    /// one's existing DM session where possible, connecting fresh
    /// (`connect_with_retry`) for anyone we don't currently have one open
    /// with — same reasoning as `spawn_send_ack`'s fix: a status people
    /// are meant to actually see is worth the cost of a real connection
    /// attempt rather than silently skipping contacts with no warm
    /// session. Also records it as our own current status locally so it
    /// shows in our own Status tab. Best-effort per contact — one
    /// contact being unreachable doesn't stop the others from getting it.
    pub async fn broadcast_status(
        &self,
        text: &str,
        image_raw: Option<&[u8]>,
        video_frames: Option<Vec<image::RgbImage>>,
        audio_clip: Option<Vec<u8>>,
        ttl_hours: u64,
    ) -> Result<(
        Option<(String, Vec<u8>)>,
        Option<(String, Vec<u8>)>,
        Option<(String, Vec<u8>)>,
    )> {
        let now = now_unix_ms();
        let expires = now + (ttl_hours as i64) * 3600 * 1000;

        let mut cached_image = None;
        let image = match image_raw {
            Some(raw) => {
                let decoded = crate::media::decode_status_image(raw)?;
                let tag_info = self
                    .blobs
                    .blobs()
                    .add_bytes(decoded.png_bytes.clone())
                    .await
                    .context("adding status image to local blob store")?;
                let hash = tag_info.hash.to_string();
                cached_image = Some((hash.clone(), decoded.png_bytes.clone()));
                Some(crate::protocol::message::StatusImage {
                    blake3_hash: hash,
                    size_bytes: decoded.png_bytes.len() as u64,
                })
            }
            None => None,
        };
        let (image_hash, image_size_bytes) = match &image {
            Some(img) => (Some(img.blake3_hash.as_str()), Some(img.size_bytes)),
            None => (None, None),
        };

        let mut cached_video = None;
        let video = match video_frames {
            Some(frames) => {
                // Real CPU work (AV1 encode, even at this resolution/
                // duration) — spawn_blocking rather than running it
                // inline on the async runtime, same reasoning as every
                // other CPU-bound step in this codebase.
                let clip_bytes = tokio::task::spawn_blocking(move || {
                    crate::net::calls::video::encode_clip(&frames)
                })
                .await
                .context("AV1 encode task panicked")??;
                let tag_info = self
                    .blobs
                    .blobs()
                    .add_bytes(clip_bytes.clone())
                    .await
                    .context("adding status video to local blob store")?;
                let hash = tag_info.hash.to_string();
                cached_video = Some((hash.clone(), clip_bytes.clone()));
                Some(crate::protocol::message::StatusVideo {
                    blake3_hash: hash,
                    size_bytes: clip_bytes.len() as u64,
                })
            }
            None => None,
        };
        let (video_hash, video_size_bytes) = match &video {
            Some(v) => (Some(v.blake3_hash.as_str()), Some(v.size_bytes)),
            None => (None, None),
        };

        let mut cached_audio = None;
        let audio = match audio_clip {
            Some(clip_bytes) => {
                let tag_info = self
                    .blobs
                    .blobs()
                    .add_bytes(clip_bytes.clone())
                    .await
                    .context("adding status voice clip to local blob store")?;
                let hash = tag_info.hash.to_string();
                cached_audio = Some((hash.clone(), clip_bytes.clone()));
                Some(crate::protocol::message::StatusAudio {
                    blake3_hash: hash,
                    size_bytes: clip_bytes.len() as u64,
                })
            }
            None => None,
        };
        let (audio_hash, audio_size_bytes) = match &audio {
            Some(a) => (Some(a.blake3_hash.as_str()), Some(a.size_bytes)),
            None => (None, None),
        };

        self.store.upsert_status(
            &hex(self.my_id),
            &self.my_name,
            text,
            now,
            expires,
            image_hash,
            image_size_bytes,
            video_hash,
            video_size_bytes,
            audio_hash,
            audio_size_bytes,
        )?;

        let contacts = self.store.accepted_contacts().unwrap_or_default();
        for contact in contacts {
            let Ok(peer) = parse_hex(&contact.endpoint_id) else {
                continue;
            };
            let envelope = crate::protocol::message::Envelope::status(
                &self.my_name,
                text,
                image.clone(),
                video.clone(),
                audio.clone(),
                expires as u64,
            );
            let session = match self.dm_sessions.get(&peer) {
                Some(s) => s.clone(),
                None => match connect_with_retry(&self.endpoint, peer).await {
                    Ok(s) => s,
                    Err(e) => {
                        tracing::debug!(peer = %hex(peer), error = %e, "couldn't reach contact for status broadcast");
                        continue;
                    }
                },
            };
            if let Err(e) = session.send(&envelope).await {
                tracing::debug!(peer = %hex(peer), error = %e, "status send failed for one contact");
            }
        }
        Ok((cached_image, cached_video, cached_audio))
    }

    /// Decode, canonicalize (see `media::decode_avatar`), and add `raw`
    /// as this identity's display picture: stores it in the local
    /// `iroh-blobs` store, persists the resulting hash under
    /// `settings.my_avatar_hash`, records it in `download_history` (so
    /// `Store::cached_download_path` finds it on the next launch the same
    /// uniform way it would for a fetched contact avatar — nothing
    /// downloaded it, but it's on disk under its own hash all the same),
    /// and broadcasts a `Body::AvatarUpdate` to every accepted contact —
    /// same best-effort, connect-if-needed pattern as `broadcast_status`.
    /// Returns the canonical PNG bytes so the caller can update its own
    /// UI immediately without a round trip back through storage.
    pub async fn set_my_avatar(&self, raw: &[u8], cache_dir: &std::path::Path) -> Result<Vec<u8>> {
        let decoded = crate::media::decode_avatar(raw)?;
        let tag_info = self
            .blobs
            .blobs()
            .add_bytes(decoded.png_bytes.clone())
            .await
            .context("adding avatar to local blob store")?;
        let hash = tag_info.hash.to_string();
        let size_bytes = decoded.png_bytes.len() as u64;
        self.store.set_setting("my_avatar_hash", &hash)?;

        tokio::fs::create_dir_all(cache_dir).await.ok();
        let dest_path = cache_dir.join(format!("{hash}.png"));
        if tokio::fs::write(&dest_path, &decoded.png_bytes)
            .await
            .is_ok()
        {
            let _ = self
                .store
                .record_download(&hash, &dest_path.to_string_lossy(), size_bytes);
        }

        let contacts = self.store.accepted_contacts().unwrap_or_default();
        for contact in contacts {
            let Ok(peer) = parse_hex(&contact.endpoint_id) else {
                continue;
            };
            let envelope =
                crate::protocol::message::Envelope::avatar_update(&self.my_name, &hash, size_bytes);
            let session = match self.dm_sessions.get(&peer) {
                Some(s) => s.clone(),
                None => match connect_with_retry(&self.endpoint, peer).await {
                    Ok(s) => s,
                    Err(e) => {
                        tracing::debug!(peer = %hex(peer), error = %e, "couldn't reach contact for avatar broadcast");
                        continue;
                    }
                },
            };
            if let Err(e) = session.send(&envelope).await {
                tracing::debug!(peer = %hex(peer), error = %e, "avatar update send failed for one contact");
            }
        }
        Ok(decoded.png_bytes)
    }

    /// Records that `from` has a new avatar available — called on receipt
    /// of `Body::AvatarUpdate`. Just the hash; doesn't fetch the bytes
    /// (see `fetch_contact_avatar` for the download-on-demand path that
    /// actually pulls them, whenever the UI first needs to display it).
    pub fn record_incoming_avatar_hash(&self, from: EndpointId, hash: &str) -> Result<()> {
        self.store.set_contact_avatar_hash(&hex(from), hash)
    }

    pub fn my_avatar_hash(&self) -> Option<String> {
        self.store.get_setting("my_avatar_hash").ok().flatten()
    }

    pub fn store(&self) -> std::sync::Arc<crate::store::Store> {
        self.store.clone()
    }

    /// Get `peer`'s avatar PNG bytes for `hash`, fetching over the
    /// network only if it isn't already in the local cache
    /// (`Store::cached_download_path`/`download_history` — content-hash
    /// keyed, so this is also a free dedup: two contacts who happen to
    /// share the same picture only ever cost one download between them).
    /// `cache_dir` is wherever the caller wants avatar files kept.
    ///
    /// A free function, not an `App` method: `App` doesn't implement
    /// `Clone` (it holds a `Mutex<Option<oneshot::Sender<()>>>` for
    /// `active_call_hangup`, among other non-`Clone` pieces), so a caller
    /// inside a `spawn(async move { ... })` block can't pull an owned
    /// `App` out of `ui.core`'s `Signal` to await against — the
    /// established pattern here (see `ui::mod`'s file-download call site)
    /// is to extract the specific cheap-to-clone handles a call needs
    /// (`store()`/`blobs()`/`endpoint()`, all real `Arc`/cheap-handle
    /// clones) inside a scoped block that drops the `Signal` read guard
    /// *before* the `.await`, then call a plain function with those. This
    /// is exactly that shape, not an `App` method.
    pub async fn fetch_contact_avatar(
        store: &crate::store::Store,
        blobs: &FsStore,
        endpoint: &Endpoint,
        peer: EndpointId,
        hash: &str,
        cache_dir: &std::path::Path,
    ) -> Result<Vec<u8>> {
        if let Some(path) = store.cached_download_path(hash)? {
            if let Ok(bytes) = tokio::fs::read(&path).await {
                return Ok(bytes);
            }
            // Cached path recorded but the file's gone (cache dir wiped,
            // etc.) — fall through and re-fetch rather than erroring.
        }

        let parsed_hash: iroh_blobs::Hash = hash.parse().context("parsing avatar blob hash")?;
        let bytes =
            crate::net::transfer::fetch_avatar_bytes(blobs, endpoint, peer, parsed_hash).await?;

        tokio::fs::create_dir_all(cache_dir).await.ok();
        let dest_path = cache_dir.join(format!("{hash}.png"));
        if tokio::fs::write(&dest_path, &bytes).await.is_ok() {
            let _ = store.record_download(hash, &dest_path.to_string_lossy(), bytes.len() as u64);
        }
        Ok(bytes)
    }

    pub fn record_incoming_dm(&self, from: EndpointId, envelope: &Envelope) -> Result<()> {
        let expires_at = envelope.expires_at_unix_ms.map(|v| v as i64);
        match &envelope.body {
            crate::protocol::message::Body::Text { text, reply_to } => self.store.log_message(
                &Conversation::Dm(hex(from)),
                &hex(from),
                &envelope.from_name,
                text,
                &MessageKind::Text,
                envelope.sent_unix_ms as i64,
                false,
                expires_at,
                envelope.id,
                *reply_to,
            ),
            crate::protocol::message::Body::File {
                name,
                size_bytes,
                compressed,
                blake3_hash,
                reply_to,
                ..
            } => self.store.log_message(
                &Conversation::Dm(hex(from)),
                &hex(from),
                &envelope.from_name,
                name,
                &MessageKind::File {
                    name: name.clone(),
                    hash: blake3_hash.clone(),
                    size_bytes: *size_bytes,
                    compressed: *compressed,
                },
                envelope.sent_unix_ms as i64,
                false,
                expires_at,
                envelope.id,
                *reply_to,
            ),
            crate::protocol::message::Body::Reaction {
                target_id,
                emoji,
                remove,
            } => self.store.apply_reaction(
                &Conversation::Dm(hex(from)),
                *target_id,
                &hex(from),
                emoji,
                *remove,
            ),
            crate::protocol::message::Body::Edit {
                target_id,
                new_text,
            } => self.store.apply_edit(
                &Conversation::Dm(hex(from)),
                *target_id,
                &hex(from),
                new_text,
                envelope.sent_unix_ms as i64,
            ),
            crate::protocol::message::Body::Delete { target_id } => {
                self.store
                    .apply_delete(&Conversation::Dm(hex(from)), *target_id, &hex(from))
            }
            crate::protocol::message::Body::Read { up_to_sent_unix_ms } => {
                self.store.set_read_watermark(
                    &Conversation::Dm(hex(from)),
                    &hex(from),
                    *up_to_sent_unix_ms as i64,
                )
            }
            _ => Ok(()),
        }
    }

    pub fn record_incoming_room(
        &self,
        room: &str,
        from: EndpointId,
        envelope: &Envelope,
    ) -> Result<()> {
        if from == self.my_id {
            return Ok(()); // gossip echoes our own broadcasts back to us
        }
        match &envelope.body {
            crate::protocol::message::Body::Text { text, reply_to } => {
                self.store.log_message(
                    &Conversation::Room(room.to_string()),
                    &hex(from),
                    &envelope.from_name,
                    text,
                    &MessageKind::Text,
                    envelope.sent_unix_ms as i64,
                    false,
                    None, // no disappearing-message policy for rooms yet — see ARCHITECTURE.md
                    envelope.id,
                    *reply_to,
                )?;
            }
            crate::protocol::message::Body::Reaction {
                target_id,
                emoji,
                remove,
            } => {
                self.store.apply_reaction(
                    &Conversation::Room(room.to_string()),
                    *target_id,
                    &hex(from),
                    emoji,
                    *remove,
                )?;
            }
            crate::protocol::message::Body::Edit {
                target_id,
                new_text,
            } => {
                self.store.apply_edit(
                    &Conversation::Room(room.to_string()),
                    *target_id,
                    &hex(from),
                    new_text,
                    envelope.sent_unix_ms as i64,
                )?;
            }
            crate::protocol::message::Body::Delete { target_id } => {
                self.store.apply_delete(
                    &Conversation::Room(room.to_string()),
                    *target_id,
                    &hex(from),
                )?;
            }
            // Read receipts are DM-only for now — see `Body::Read`'s doc.
            _ => {}
        }
        Ok(())
    }

    pub fn history(
        &self,
        conversation: &Conversation,
        limit: u32,
    ) -> Result<Vec<crate::store::StoredMessage>> {
        self.store.recent_messages(conversation, limit)
    }

    /// Physically delete expired (disappearing) messages from sqlite.
    /// Called periodically by `ui::spawn_disappearing_sweep`. See
    /// `store::Store::sweep_expired_messages`'s doc for why this exists
    /// alongside (not instead of) the read-path filter in `history`.
    pub fn sweep_expired_messages(&self) -> Result<usize> {
        self.store.sweep_expired_messages()
    }

    pub async fn shutdown(self) {
        self._router.shutdown().await.ok();
        self.endpoint.close().await;
    }
}

pub fn hex(id: EndpointId) -> String {
    data_encoding::HEXLOWER.encode(id.as_bytes())
}

pub fn short_id(id: EndpointId) -> String {
    let h = hex(id);
    format!("{}…{}", &h[..6], &h[h.len() - 4..])
}

fn now_unix_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

fn parse_hex(s: &str) -> Result<EndpointId> {
    let bytes = data_encoding::HEXLOWER.decode(s.as_bytes())?;
    let arr: [u8; 32] = bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("wrong-length hex endpoint id"))?;
    EndpointId::from_bytes(&arr).map_err(|e| anyhow::anyhow!("bad endpoint id: {e}"))
}

/// Load or create this device's identity. First run: caller must have
/// already obtained a `Seed` (generated fresh or typed in for recovery) via
/// the onboarding UI; every run after that just loads the persisted
/// derived keys.
pub fn load_identity(data_dir: &Path) -> Result<Option<SecretKey>> {
    if !identity::exists(data_dir) {
        return Ok(None);
    }
    identity::load(data_dir).map(Some)
}

pub fn create_identity(data_dir: &Path, seed: &identity::seed::Seed) -> Result<SecretKey> {
    identity::create_from_seed(data_dir, seed)
}

/// Same retry/backoff shape as `connect_with_retry` below, for the same
/// underlying reason: `net::contacts::send_request`'s single connect
/// attempt is already bounded (`NET_TIMEOUT`, 8s — see its own "contact
/// request connect timed out" error), but iroh's discovery has real
/// propagation lag — the *other* side's current address often hasn't
/// finished publishing to the discovery service yet, especially right
/// after they've just started their own app. That's not a "your network
/// is broken" failure, and a relay being reachable (`relay_ok`) doesn't
/// mean anything about *discovery* specifically — they're different
/// services. One 8s attempt routinely isn't enough; four attempts with
/// backoff (500ms → 1s → 2s → 4s, ~40s worst case including the connect
/// timeouts themselves) gives discovery realistic time to catch up
/// without the UI looking like it's hung (each failed attempt is a
/// `tracing::warn!`, visible with `RUST_LOG=info`).
#[allow(clippy::too_many_arguments)]
pub(crate) async fn request_contact_with_retry(
    endpoint: &Endpoint,
    to: EndpointId,
    addr_hint: Option<EndpointAddr>,
    my_id: EndpointId,
    my_username: Option<String>,
    my_name: &str,
    note: &str,
) -> Result<()> {
    const ATTEMPTS: u32 = 4;
    let mut delay = Duration::from_millis(500);
    let mut last_err = None;

    for attempt in 1..=ATTEMPTS {
        match contacts::send_request(
            endpoint,
            to,
            addr_hint.clone(),
            my_id,
            my_username.clone(),
            my_name,
            note,
        )
        .await
        {
            Ok(()) => return Ok(()),
            Err(e) => {
                warn!(%to, attempt, %e, "contact request attempt failed");
                last_err = Some(e);
                if attempt < ATTEMPTS {
                    tokio::time::sleep(delay).await;
                    delay *= 2;
                }
            }
        }
    }
    Err(last_err.expect("loop runs at least once"))
}

/// Same as `request_contact_with_retry`, for accepting an incoming
/// request — the dial-back that tells the requester "yes, go ahead" is
/// just as exposed to the same discovery-lag failure mode.
pub(crate) async fn accept_contact_with_retry(
    endpoint: &Endpoint,
    to: EndpointId,
    my_id: EndpointId,
    my_username: Option<String>,
    my_name: &str,
) -> Result<()> {
    const ATTEMPTS: u32 = 4;
    let mut delay = Duration::from_millis(500);
    let mut last_err = None;

    for attempt in 1..=ATTEMPTS {
        match contacts::send_accept(endpoint, to, my_id, my_username.clone(), my_name).await {
            Ok(()) => return Ok(()),
            Err(e) => {
                warn!(%to, attempt, %e, "contact accept attempt failed");
                last_err = Some(e);
                if attempt < ATTEMPTS {
                    tokio::time::sleep(delay).await;
                    delay *= 2;
                }
            }
        }
    }
    Err(last_err.expect("loop runs at least once"))
}

pub async fn connect_with_retry(endpoint: &Endpoint, peer_id: EndpointId) -> Result<DmSession> {
    const ATTEMPTS: u32 = 4;
    let mut delay = Duration::from_millis(500);
    let mut last_err = None;

    for attempt in 1..=ATTEMPTS {
        match DmSession::connect(endpoint, peer_id).await {
            Ok(session) => return Ok(session),
            Err(e) => {
                warn!(%peer_id, attempt, %e, "connect attempt failed");
                last_err = Some(e);
                if attempt < ATTEMPTS {
                    tokio::time::sleep(delay).await;
                    delay *= 2;
                }
            }
        }
    }

    Err(last_err.expect("loop runs at least once"))
}

/// Open a room's `iroh-docs` metadata doc, ensure a `RoomMeta` record
/// exists, and announce our own membership — then hand the opened
/// `RoomDoc` back to the caller to commit (`App::commit_room_doc`) once
/// it's no longer holding any UI lock across the await. A free function
/// taking cheap owned/cloned handles rather than `&mut App`, for the same
/// reason `connect_with_retry`/`join_room_with_retry` above are free
/// functions: so a Dioxus UI call site can extract these handles under a
/// short-lived read-lock, do the actual network-bound work with no lock
/// held at all, and only take a write-lock again for the fast, synchronous
/// commit at the end. See the caveat note above `App::room_doc`.
pub async fn ensure_room_metadata_standalone(
    docs: Docs,
    blobs: iroh_blobs::api::Store,
    author: AuthorId,
    my_id: EndpointId,
    my_name: String,
    name: String,
    bootstrap: Vec<EndpointAddr>,
) -> Result<RoomDoc> {
    let room_doc = RoomDoc::open(&docs, blobs, author, &name, bootstrap).await?;
    room_doc.ensure_meta(&name, my_id).await?;
    room_doc.announce_self(my_id, &my_name).await?;
    Ok(room_doc)
}

pub async fn join_room_with_retry(
    gossip: &Gossip,
    name: &str,
    bootstrap: Vec<EndpointId>,
    room_tx: UnboundedSender<RoomEvent>,
) -> Result<Room> {
    const ATTEMPTS: u32 = 4;
    let mut delay = Duration::from_millis(500);
    let mut last_err = None;

    for attempt in 1..=ATTEMPTS {
        match Room::join(gossip, name, bootstrap.clone(), room_tx.clone()).await {
            Ok(room) => return Ok(room),
            Err(e) => {
                warn!(room = %name, attempt, %e, "join attempt failed");
                last_err = Some(e);
                if attempt < ATTEMPTS {
                    tokio::time::sleep(delay).await;
                    delay *= 2;
                }
            }
        }
    }
    Err(last_err.expect("loop runs at least once"))
}
