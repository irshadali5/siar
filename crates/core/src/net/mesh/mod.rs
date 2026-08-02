//! Offline mesh: store-and-forward message relay over Bluetooth LE
//! (`ble`) and the local Wi-Fi/hotspot network (`lan`), used when the
//! public relay/discovery path (`App::relay_ok`) is down or there's no
//! internet uplink at all. Off by default (§ store::Store::
//! offline_mesh_enabled) — this only ever runs when the user has
//! explicitly opted in, same as `background_wake_enabled`.
//!
//! ## Why one mesh module for two transports
//!
//! Bluetooth LE and local Wi-Fi solve the same problem (reach a peer
//! with no internet) at different ranges — BLE: metres, low bandwidth,
//! very low power; LAN: same building/hotspot, full IP bandwidth — so a
//! phone with both on effectively has two overlapping radios for the
//! same mesh. Rather than build two independent delivery paths, both
//! transports share one format (`Envelope`, below) and one dedup/flood
//! policy (`SeenCache`); only the actual bytes-on-the-wire differ,
//! which is what `MeshTransport` isolates.
//!
//! ## Delivery model: flood with TTL, not routing
//!
//! There's no routing table and no peer graph — every node that hears
//! an `Envelope` it hasn't seen before re-broadcasts it (on whichever
//! transports are enabled) and decrements `ttl`. This is deliberately
//! the simplest correct thing that works for a phone-scale mesh (tens
//! of nodes, not thousands): flooding wastes bandwidth relative to real
//! routing, but needs no topology discovery, tolerates nodes joining/
//! leaving/moving mid-conversation for free, and the LAN/BLE ranges
//! this is meant for are small enough that flood amplification stays
//! bounded. `ttl` starts at `DEFAULT_TTL` hops and existing message
//! sizes (chat text, not file transfers — those still need the QUIC
//! path) keep per-hop cost small.
//!
//! ## What rides on top
//!
//! `Envelope.payload` is an already-encoded `protocol::message::
//! Envelope` (the same wire format DMs/room messages use over QUIC) —
//! the mesh doesn't know or care what's inside, it just gets those
//! bytes to every reachable node. `App` decodes `payload` the same way
//! it decodes an incoming QUIC stream, so a message delivered via mesh
//! shows up through the exact same `record_incoming_dm`/
//! `record_incoming_room` path as one delivered via relay — no
//! parallel message pipeline to keep in sync.
//!
//! ## Verified vs. open per platform
//!
//! - **Desktop (Linux/Windows):** both `lan` (UDP broadcast, pure
//!   std/tokio) and `ble` (btleplug, BlueZ/WinRT backends) are real
//!   dependency-verified code paths here.
//! - **Android — LAN:** UDP *broadcast* (what `lan` actually uses,
//!   deliberately — see `Cargo.toml`) needs no special Android
//!   permission or lock, unlike multicast/mDNS. This path is expected
//!   to work as written, same as desktop.
//! - **Android — BLE:** `net::mesh::ble` is cfg'd out entirely on
//!   Android (see `Cargo.toml`) — btleplug's Android backend needs a
//!   JNIEnv/Context handle this crate has no verified way to obtain
//!   yet. Same class of bootstrap-glue gap as the native/Blitz
//!   renderer's JNI entry point already documented in `siar-android`.

mod envelope;
mod lan;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
mod ble;

pub use envelope::{Envelope, DEFAULT_TTL};

use iroh::EndpointId;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;

/// A transport `MeshManager` can drive: broadcast an envelope out, and
/// hand received ones back in over `inbound`. `lan::LanTransport` and
/// (desktop-only) `ble::BleTransport` are the two implementations.
#[async_trait::async_trait]
pub trait MeshTransport: Send + Sync {
    async fn broadcast(&self, envelope: &Envelope) -> anyhow::Result<()>;
    fn name(&self) -> &'static str;
}

/// Live status, read by the UI (Settings' Network tab) — deliberately
/// just counts, not a peer list: at flood-mesh scale "how many nodes
/// have I seen recently" is the useful signal, and per-peer identity
/// isn't reliable anyway (BLE advertisements and mDNS records don't
/// carry a verified `EndpointId` the way a QUIC handshake does).
#[derive(Default)]
pub struct MeshStatus {
    pub lan_active: AtomicBool,
    pub ble_active: AtomicBool,
    pub peers_seen_recently: AtomicUsize,
    pub envelopes_relayed: AtomicUsize,
    /// Backs `peers_seen_recently` with an actual decaying set instead
    /// of a counter that only ever climbs — see `note_peer_seen`.
    recent_peer_ids: std::sync::Mutex<std::collections::HashMap<Vec<u8>, std::time::Instant>>,
}

impl MeshStatus {
    /// How long a peer identifier counts as "recently seen" before it
    /// ages out. Long enough that a normal chat-message or BLE-
    /// advertisement cadence keeps an active peer visible continuously;
    /// short enough that someone who walked out of range or turned
    /// Bluetooth/Wi-Fi off actually disappears from the Network tab
    /// instead of staying "seen" forever.
    const PEER_WINDOW: std::time::Duration = std::time::Duration::from_secs(120);

    /// Records `id` (a LAN sender's raw `EndpointId` bytes, or a BLE
    /// peripheral's address/identifier bytes) as seen just now, prunes
    /// anything older than `PEER_WINDOW`, and republishes the resulting
    /// distinct-peer count to `peers_seen_recently`. Both transports
    /// share one tracker — the Network tab's "nearby signals" number is
    /// "distinct identifiers seen recently across BLE and LAN
    /// combined," not two separate counts, since that's the number
    /// someone actually wants when checking whether the mesh has
    /// anyone to talk to at all.
    pub(crate) fn note_peer_seen(&self, id: Vec<u8>) {
        let mut seen = self.recent_peer_ids.lock().unwrap();
        let now = std::time::Instant::now();
        seen.retain(|_, t| now.duration_since(*t) < Self::PEER_WINDOW);
        seen.insert(id, now);
        self.peers_seen_recently
            .store(seen.len(), Ordering::Relaxed);
    }
}

/// Owns whichever transports are enabled, the seen-envelope dedup
/// cache, and the task that forwards decoded payloads into `App`.
/// Constructed once per identity (same lifetime as `App`); `start`/
/// `stop` are cheap and safe to call repeatedly — they're what the
/// Settings toggle calls directly.
pub struct MeshManager {
    my_id: EndpointId,
    status: Arc<MeshStatus>,
    seen: envelope::SeenCache,
    transports: std::sync::Mutex<Vec<Arc<dyn MeshTransport>>>,
    /// Decoded mesh-delivered payloads, ready for `App` to feed through
    /// the same path an incoming QUIC stream would use.
    inbound_tx: mpsc::UnboundedSender<(EndpointId, Vec<u8>)>,
}

impl MeshManager {
    pub fn new(my_id: EndpointId) -> (Arc<Self>, mpsc::UnboundedReceiver<(EndpointId, Vec<u8>)>) {
        let (inbound_tx, inbound_rx) = mpsc::unbounded_channel();
        let mgr = Arc::new(Self {
            my_id,
            status: Arc::new(MeshStatus::default()),
            seen: envelope::SeenCache::new(),
            transports: std::sync::Mutex::new(Vec::new()),
            inbound_tx,
        });
        (mgr, inbound_rx)
    }

    pub fn status(&self) -> Arc<MeshStatus> {
        self.status.clone()
    }

    /// Starts both transports. Safe to call when already running — it
    /// just no-ops on top of whatever's already active, since `start`/
    /// `stop` mirror a single on/off setting rather than independent
    /// per-transport toggles (see `store::offline_mesh_enabled`).
    ///
    /// Takes `self: Arc<Self>` (not `&self`) — the only one of these
    /// stable "smart-pointer receiver" forms Rust actually supports
    /// without the (still unstable) `arbitrary_self_types` feature is
    /// `Arc<Self>` by value, not `&Arc<Self>`; callers pass an owned
    /// clone (`mesh.clone().start().await` where `mesh: Arc<MeshManager>`
    /// needs to stay usable afterward, or plain `mesh.start().await`
    /// where it doesn't — see call sites in `app.rs` and `siar-ui`).
    pub async fn start(self: Arc<Self>) {
        if !self.transports.lock().unwrap().is_empty() {
            return; // already running
        }

        let mut running: Vec<Arc<dyn MeshTransport>> = Vec::new();
        let handle = MeshInboundHandle { mgr: self.clone() };

        match lan::LanTransport::start(self.my_id, self.status.clone(), handle.clone()).await {
            Ok(t) => {
                self.status.lan_active.store(true, Ordering::Relaxed);
                running.push(Arc::new(t));
            }
            Err(err) => tracing::warn!(?err, "mesh: LAN transport failed to start"),
        }

        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        match ble::BleTransport::start(self.my_id, self.status.clone(), handle.clone()).await {
            Ok(t) => {
                self.status.ble_active.store(true, Ordering::Relaxed);
                running.push(Arc::new(t));
            }
            Err(err) => tracing::warn!(?err, "mesh: BLE transport failed to start"),
        }

        *self.transports.lock().unwrap() = running;
    }

    pub fn stop(&self) {
        self.transports.lock().unwrap().clear();
        self.status.lan_active.store(false, Ordering::Relaxed);
        self.status.ble_active.store(false, Ordering::Relaxed);
    }

    /// Encodes `payload` (an already-`protocol::message::Envelope::
    /// encode`d message) and floods it on every running transport.
    pub async fn send(&self, payload: Vec<u8>) {
        let env = Envelope::new(self.my_id, payload);
        self.seen.mark(env.id);
        // Collected into an owned `Vec` and bound to its own `let`
        // *before* the loop — not `for t in ....lock().unwrap()....
        // collect() { ... await ... }` directly. That looks equivalent
        // but isn't: a temporary created in a `for` loop's head
        // expression lives until the end of the whole `for` statement
        // (not just until the iterator is produced), so the
        // `MutexGuard` temporary would otherwise stay alive across
        // every `.await` in the loop body, making this future `!Send`
        // (`tokio::spawn` requires `Send`) and failing to compile in
        // exactly the way it originally did.
        let active: Vec<Arc<dyn MeshTransport>> =
            self.transports.lock().unwrap().iter().cloned().collect();
        for t in active {
            if let Err(err) = t.broadcast(&env).await {
                tracing::debug!(transport = t.name(), ?err, "mesh: broadcast failed");
            }
        }
    }

    /// Called by each transport when it hears an `Envelope`, whether
    /// from the wire (BLE GATT write / LAN UDP datagram) or handed to
    /// it directly. Applies the dedup+TTL flood policy once, in one
    /// place, regardless of which transport it arrived on. Plain
    /// `&self` — unlike `start`, this never needs to hand out a fresh
    /// `Arc<Self>` clone of its own, only read/lock existing fields.
    async fn on_received(&self, env: Envelope) {
        if !self.seen.mark_and_check_new(env.id) {
            return; // already relayed this one — flood control
        }
        self.status
            .envelopes_relayed
            .fetch_add(1, Ordering::Relaxed);

        // Hand the decoded payload to App regardless of hop count.
        if let Some(sender) = env.sender_id() {
            let _ = self.inbound_tx.send((sender, env.payload.clone()));
        } else {
            tracing::debug!("mesh: dropped envelope with unparseable sender id");
        }

        // Re-flood to every other transport if there's TTL budget left.
        // Same `let`-then-`for` shape as `send()` above, same reason.
        if let Some(next) = env.decremented() {
            let active: Vec<Arc<dyn MeshTransport>> =
                self.transports.lock().unwrap().iter().cloned().collect();
            for t in active {
                if let Err(err) = t.broadcast(&next).await {
                    tracing::debug!(transport = t.name(), ?err, "mesh: re-flood failed");
                }
            }
        }
    }
}

/// Cheap `Arc`-backed callback handle passed to each transport's
/// `start()` so it can report received envelopes back without a
/// circular `&mut` borrow on `MeshManager` itself.
#[derive(Clone)]
pub(crate) struct MeshInboundHandle {
    mgr: Arc<MeshManager>,
}

impl MeshInboundHandle {
    pub(crate) async fn received(&self, env: Envelope) {
        self.mgr.on_received(env).await;
    }
}
