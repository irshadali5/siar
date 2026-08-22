//! The `siar_messaging::MessageService` FFI surface
//! `apps/android/README.md` named explicitly as missing: "there is no
//! `siar-messaging::MessageService` FFI surface here at all... This is
//! separate, much larger follow-up work." This crate is that follow-up
//! — deliberately still a *slice* of it, not the whole thing (see
//! "What's still NOT here" below).
//!
//! ## Shape: one global instance, a background pump, a poll queue
//!
//! Same "exactly one, not per-connection" reasoning
//! `siar-android-connectivity`'s own doc comment gives for
//! `ConnectivityState`: there's one `MessageService` for the whole
//! running app, so this is a `static` behind a lock, not a
//! `createBridge`/handle pattern like the four transport bridges use.
//!
//! JNI is call-in only in this whole app (every `jni_bridge.rs`'s own
//! doc comment states this rule) — Rust has no way to push an incoming
//! message to Kotlin the instant it arrives. So a background task
//! (spawned once, in [`bootstrap`]) drains `MessageService`'s incoming
//! frames continuously and appends a plain-text summary line to an
//! in-memory queue; [`poll_next_event`] is what Kotlin calls
//! (repeatedly, e.g. on its own timer — same tradeoff
//! [`com.siar.ble.BleGattManager`]'s fragment pump already documents
//! for the identical shape) to drain it one line at a time.
//!
//! ## Event line format
//!
//! Deliberately plain tab-separated text, not JSON — this workspace has
//! no `serde_json` dependency anywhere (checked: `siar-crypto-mls`'s own
//! Cargo.toml has a comment explaining why it deliberately avoided
//! adding one), and inventing a second serialization format across the
//! JNI boundary for a handful of fixed-shape lines is real, avoidable
//! overhead for both sides — Kotlin can `split("\t")` a fixed number of
//! fields without a JSON parser dependency of its own. Two event kinds
//! today:
//! - `text\t<sender endpoint id hex>\t<message text>`
//! - `mailbox\t<count>` — a `MailboxCheckIn` response batch finished
//!   arriving (matches `apps/cli`'s own `check_mailbox`'s "count and
//!   move on" shape; individual mailbox item contents aren't
//!   surfaced by this crate yet — see below).
//! - `anon_text\t<matched peer's endpoint id hex>\t<message text>` — a
//!   `TokenMailboxDeposit` that decrypted successfully against one of
//!   this process's registered peers. The "sender" here is *inferred*
//!   (which candidate's session key actually opened it), not read off
//!   the wire — an anonymous check-in response has no sender field at
//!   all, the entire point (see `siar_crypto::mailbox_token`'s doc
//!   comment).
//!
//! ## What's here
//!
//! - [`bootstrap`] — generates a fresh identity (no persistence across
//!   app restarts yet, same Phase-1 stand-in `apps/cli`'s own
//!   `bootstrap()` doc comment already carries), binds a
//!   `SiarEndpoint`, starts the incoming-event pump, and reports
//!   `TransportLink::InternetDirect` up into `siar-android-
//!   connectivity`'s shared `ConnectivityState` once the bind
//!   succeeds (no corresponding "mark it down" anywhere in this crate
//!   yet — see [`bootstrap`]'s own inline comment).
//! - [`my_ticket`] — this device's own `PeerTicket`, encoded, to
//!   display/share (e.g. as a QR code — not built here, just the
//!   string it would encode).
//! - [`add_peer`] — registers a peer's ticket so incoming `V1`
//!   envelopes from them can actually be decrypted (`MessageService::
//!   handle_incoming` needs the sender's `PeerTicket` up front — see
//!   that method's own doc comment: "Phase 1's CLI trusts that `peer`
//!   really is who the transport says it is... paired by hand," the
//!   exact same precondition this crate inherits unchanged). Also
//!   what lets an incoming `TokenMailboxDeposit` be matched to a
//!   sender — see the `anon_text` event kind above.
//! - [`send_text`]/[`send_text_anon`] — the `DeviceId`-addressed and
//!   unlinkable-token-addressed paths respectively
//!   (`MessageService::send_text`/`send_text_anon`), mirroring
//!   `apps/cli`'s `send`/`send-anon`.
//! - [`check_mailbox`]/[`check_mailbox_anon`] — fire a
//!   `MailboxCheckIn`/`AnonymousMailboxCheckIn` at a relay; responses
//!   land in the poll queue as `mailbox\t<count>`/`anon_text\t...`
//!   respectively, mirroring `apps/cli`'s `check-mailbox`/
//!   `check-mailbox-anon`.
//!
//! ## What's still NOT here
//!
//! - **No groups/MLS.** `GroupService` isn't wired in at all — no
//!   `apps/android` equivalent of `apps/cli`'s `group-create`/
//!   `group-send`/`join-group`.
//! - **No attachments.** `MessageService::send_attachment`/
//!   `fetch_attachment` aren't exposed.
//! - **No delivery-ack visibility.** Incoming `DeliveryAck`/
//!   `ReadReceipt` frames are silently absorbed by `handle_incoming`
//!   (returns `None` for them, same as `apps/cli`) — nothing surfaces
//!   "your message was delivered" to Kotlin.
//! - **Individual mailbox item contents aren't surfaced.** `check_mailbox`
//!   reports a count, not each `Mesh` envelope's ciphertext/metadata —
//!   `apps/cli`'s `check_mailbox` shows byte length/hop_limit/priority
//!   per item; this crate's poll-queue line format doesn't have room
//!   for that yet without designing a richer event line (or, more
//!   likely, finally justifying a real serialization format — noted,
//!   not decided here).
//! - **Identity/database now persist across restarts.** `bootstrap`
//!   takes a `base_dir` (Kotlin passes `Context.filesDir`) and
//!   load-or-creates `DeviceIdentity`/`DeviceId`/`AccountId` there via
//!   `DeviceIdentity::save_to_file`/`load_from_file` (already-existing
//!   primitives, previously with no Android caller) plus two bare-UUID
//!   text files, and opens a real on-disk database (`siar_storage::open`)
//!   instead of `open_in_memory()` — the exact pattern `apps/desktop`'s
//!   own `resolve_data_paths`/`load_or_create_id` already used,
//!   applied here for the first time, and later mirrored into
//!   `apps/cli` too (see that binary's own `resolve_data_paths`/
//!   `load_or_create_id`) — all three of this workspace's client
//!   entry points now persist identity the same way.
//! - **`mark_link_up` now has a real `mark_link_down` counterpart** —
//!   [`shutdown_inner`], called (via JNI) from `MainActivity.onDestroy`.
//!   `APP` moved from a `OnceLock` to `Mutex<Option<Arc<AppMessaging>>>`
//!   specifically so this could stop the pump task
//!   (`JoinHandle::abort`) and let the bound endpoint drop for real,
//!   instead of only updating connectivity state — see
//!   [`shutdown_inner`]'s own doc comment for the one remaining
//!   caveat (abort isn't synchronous).

// Everything below `AppMessaging` through `poll_next_event_inner` is
// only ever called from `jni_bridge`, which is `#[cfg(target_os =
// "android")]` — on a host `cargo check`/`cargo test` (this crate's
// `rlib` output, not its `cdylib`), that whole module drops out and
// every one of these becomes unreachable dead code by the compiler's
// reckoning, despite being real, exercised logic on the platform this
// crate actually ships on. Same reasoning, same fix shape,
// `siar-android-connectivity::link_from_ordinal`'s own `cfg_attr`
// already uses — applied once here at module scope instead of item by
// item, since nearly everything in this file shares the exact same
// android-only-caller situation.
#![cfg_attr(not(target_os = "android"), allow(dead_code))]

use siar_domain::{AccountId, ConversationId, DeviceId, MessageContent, MessageText};
use siar_messaging::{IncomingEvent, MessageService, PeerTicket};
use siar_transport::{IncomingFrame, PeerTransport, SiarEndpoint};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, OnceLock};
use tokio::sync::mpsc;

struct AppMessaging {
    endpoint: Arc<SiarEndpoint>,
    service: MessageService,
    my_ticket: PeerTicket,
    /// Registered via [`add_peer`] — see this module's top doc comment
    /// on why a sender's `PeerTicket` has to already be known before an
    /// incoming `V1` envelope from them can be decrypted.
    known_peers: Mutex<HashMap<iroh::EndpointId, PeerTicket>>,
    events: Mutex<VecDeque<String>>,
    /// The incoming-frame pump's own handle, so [`shutdown_inner`] can
    /// actually stop it via [`tokio::task::JoinHandle::abort`] instead
    /// of leaving it running until process exit — the specific
    /// limitation this field exists to close. `Mutex<Option<_>>`
    /// rather than a bare field since it's populated right after
    /// `tokio::spawn` returns, one step after this struct itself is
    /// constructed (see [`bootstrap_inner`]).
    pump_handle: Mutex<Option<tokio::task::JoinHandle<()>>>,
}

// `RUNTIME` stays a `OnceLock`: a Tokio multi-thread runtime is a real
// thread pool, and tearing it down and rebuilding it on every
// bootstrap/shutdown cycle is neither how any real Android app treats
// its executor nor something a `shutdown()` call driven from
// `onDestroy` needs — the process is going away regardless once that
// fires for real. `APP` is the piece that actually needed to become
// resettable: it holds the per-session `SiarEndpoint`/`MessageService`/
// pump task, which a real `shutdown()` must be able to tear down and
// let a later `bootstrap()` recreate from scratch, which a `OnceLock`
// can never do once set. `Arc` (not a bare `AppMessaging`) so the pump
// task can hold its own strong reference independent of the slot in
// `APP`, which [`shutdown_inner`] can empty out from under it.
static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
static APP: Mutex<Option<Arc<AppMessaging>>> = Mutex::new(None);

/// Clones the current app handle out of the lock and releases it
/// immediately — every caller below needs the `Arc` for the duration
/// of a `block_on`/lock elsewhere, not the `APP` mutex itself, so
/// nothing holds `APP`'s lock across an `.await` or another lock
/// acquisition.
fn app_handle() -> Option<Arc<AppMessaging>> {
    APP.lock().expect("APP lock poisoned").clone()
}

fn runtime() -> &'static tokio::runtime::Runtime {
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("failed to build the Tokio runtime this crate drives every async call through")
    })
}

/// Where this device's identity/account id/database live on disk —
/// same fix `apps/desktop`'s own `DataPaths`/`resolve_data_paths`
/// already carries, applied here for the first time on Android: before
/// this, `bootstrap_inner` regenerated a fresh `DeviceIdentity`/
/// `AccountId`/`DeviceId` and opened an in-memory database on *every*
/// launch, so nothing survived a process restart — the specific gap
/// this crate's own doc comment and `apps/android/README.md` both
/// named. `base_dir` is supplied by Kotlin (`Context.filesDir`,
/// app-private storage the OS already sandboxes per-app — no new
/// permission needed), since this crate has no `Context` of its own to
/// resolve a directory from.
struct AppDataPaths {
    identity: std::path::PathBuf,
    account_id: std::path::PathBuf,
    device_id: std::path::PathBuf,
    database: std::path::PathBuf,
}

impl AppDataPaths {
    fn under(base_dir: &str) -> Self {
        let base = std::path::PathBuf::from(base_dir);
        Self {
            identity: base.join("identity.bin"),
            account_id: base.join("account_id.txt"),
            device_id: base.join("device_id.txt"),
            database: base.join("siar.db"),
        }
    }
}

/// Same "bare UUID text file, not postcard/JSON" reasoning
/// `apps/desktop`'s own `load_or_create_id` doc comment gives —
/// duplicated here rather than shared, since sharing it would mean
/// either `apps/desktop` depending on this crate or a new shared crate
/// for one ~10-line function, neither of which is worth the coupling.
fn load_or_create_id<T>(
    path: &std::path::Path,
    from_uuid: impl Fn(uuid::Uuid) -> T,
    to_uuid: impl Fn(&T) -> uuid::Uuid,
    generate: impl Fn() -> T,
) -> Result<T, String> {
    if path.exists() {
        let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        let id = uuid::Uuid::parse_str(text.trim()).map_err(|e| e.to_string())?;
        Ok(from_uuid(id))
    } else {
        let id = generate();
        std::fs::write(path, to_uuid(&id).to_string()).map_err(|e| e.to_string())?;
        Ok(id)
    }
}

/// Real setup, real errors — every fallible step returns a `String`
/// description rather than panicking, since a JNI function panicking
/// unwinds into Kotlin as undefined behavior (documented Rust/JNI
/// interop hazard, not specific to this crate) rather than a catchable
/// exception.
fn bootstrap_inner(base_dir: &str) -> Result<String, String> {
    if let Some(app) = app_handle() {
        return Ok(app.my_ticket.encode());
    }
    let app_arc: Arc<AppMessaging> = runtime().block_on(async {
        std::fs::create_dir_all(base_dir).map_err(|e| e.to_string())?;
        let paths = AppDataPaths::under(base_dir);

        let identity = if paths.identity.exists() {
            siar_crypto::DeviceIdentity::load_from_file(&paths.identity).map_err(|e| e.to_string())?
        } else {
            let identity = siar_crypto::DeviceIdentity::generate();
            identity.save_to_file(&paths.identity).map_err(|e| e.to_string())?;
            identity
        };
        let device_id = load_or_create_id(&paths.device_id, DeviceId::from_uuid, DeviceId::as_uuid, DeviceId::new)?;
        let _local_account = load_or_create_id(&paths.account_id, AccountId::from_uuid, AccountId::as_uuid, AccountId::new)?; // kept alive by MessageService's own device_id, not used directly here yet

        // Real on-disk database, not `open_in_memory()` — the other
        // half of the persistence fix: an in-memory DB would have
        // discarded every message/outbox row the moment the process
        // exited regardless of how stable the identity above now is.
        let db = siar_storage::open(&paths.database.display().to_string()).map_err(|e| e.to_string())?;
        let messages = Arc::new(siar_storage::StoolapMessageRepository::new(db.clone()));
        let outbox = Arc::new(siar_storage::StoolapOutboxRepository::new(db.clone()));
        let blobs: Arc<dyn siar_storage::BlobRepository + Send + Sync> =
            Arc::new(siar_storage::StoolapBlobRepository::new(db));
        let blob_store: Arc<dyn siar_transport::BlobStore> =
            Arc::new(siar_messaging::StorageBlobStore(blobs.clone()));

        let (tx, mut rx) = mpsc::channel::<IncomingFrame>(64);
        let iroh_secret = iroh::SecretKey::generate();
        let endpoint = Arc::new(SiarEndpoint::bind(iroh_secret, tx, blob_store).await.map_err(|e| e.to_string())?);

        let my_ticket = PeerTicket {
            endpoint_addr: endpoint.addr(),
            x25519_public: identity.x25519_public().to_bytes(),
            ed25519_verifying: identity.verifying_key().to_bytes(),
        };

        // Closes the gap this crate's own doc comment (and
        // `apps/android/README.md`) used to name explicitly: a
        // successfully bound `SiarEndpoint` is real evidence this
        // device has Internet/LAN connectivity via iroh, reported into
        // the same shared `ConnectivityState` the four transport
        // bridges already feed. [`shutdown_inner`] now has a real
        // `mark_link_down` counterpart to this.
        siar_android_connectivity::mark_link_up(siar_domain::TransportLink::InternetDirect);

        let service = MessageService::new(device_id, identity, endpoint.clone(), messages, outbox, blobs);

        let app = AppMessaging {
            endpoint,
            service,
            my_ticket: my_ticket.clone(),
            known_peers: Mutex::new(HashMap::new()),
            events: Mutex::new(VecDeque::new()),
            pump_handle: Mutex::new(None),
        };
        let app = Arc::new(app);

        // The incoming-frame pump — see this module's top doc comment
        // for why this drains continuously into a poll queue instead
        // of pushing into Kotlin directly. Captures its own `Arc`
        // clone of `app` directly rather than looking it up via a
        // global on every frame — the pump no longer depends on `APP`
        // being populated at all, which is what makes it possible for
        // [`shutdown_inner`] to empty `APP` out from under it and stop
        // it independently via the handle stored just below.
        let pump_app = app.clone();
        let pump_handle = tokio::spawn(async move {
            let app = pump_app;
            let mut mailbox_batch_count: u32 = 0;
            loop {
                let Some(frame) = rx.recv().await else { break };
                match frame.message {
                    siar_protocol::WireMessage::V1(envelope) => {
                        let peer_ticket = {
                            let known = app.known_peers.lock().expect("known_peers lock poisoned");
                            known.get(&frame.from).cloned()
                        };
                        let Some(peer_ticket) = peer_ticket else {
                            tracing::debug!(from = ?frame.from, "V1 envelope from an unregistered peer, dropping — call add_peer first");
                            continue;
                        };
                        match app.service.handle_incoming(&peer_ticket, envelope).await {
                            Ok(Some(IncomingEvent::Content(MessageContent::Text(text)))) => {
                                // `{:?}` (Debug), not `{}` (Display) —
                                // matching this workspace's own existing
                                // precedent for formatting an
                                // `iroh::EndpointId` (every `tracing::
                                // debug!(from = ?frame.from, ...)` in
                                // `apps/emergency-node`, for instance):
                                // whether `PublicKey` implements
                                // `Display` wasn't confirmed against
                                // real docs this pass, so this doesn't
                                // assume it does.
                                let line = format!("text\t{:?}\t{}", frame.from, text.as_str());
                                app.events.lock().expect("events lock poisoned").push_back(line);
                            }
                            Ok(_) => {} // attachment/call-signal/ack — not surfaced by this crate yet, see top doc comment
                            Err(e) => tracing::debug!(error = %e, from = ?frame.from, "failed to handle an incoming V1 envelope"),
                        }
                    }
                    siar_protocol::WireMessage::Mesh(_) => {
                        mailbox_batch_count += 1;
                    }
                    siar_protocol::WireMessage::TokenMailboxDeposit(envelope) => {
                        // An answer to `check_mailbox_anon`. The wire
                        // message itself carries no sender identity —
                        // that's the entire unlinkability point (see
                        // `siar_crypto::mailbox_token`'s doc comment) —
                        // so the only way to find out who it's from is
                        // to try decrypting against every peer this
                        // process currently knows about (the same
                        // `known_peers` registered via `add_peer`) and
                        // see which one's session actually opens it.
                        // AEAD decryption rejecting a wrong key is a
                        // real, correct signal here, not a workaround —
                        // the same principle every "try each candidate
                        // key" pattern in real E2EE clients relies on.
                        let candidates: Vec<PeerTicket> = {
                            let known = app.known_peers.lock().expect("known_peers lock poisoned");
                            known.values().cloned().collect()
                        };
                        let mut matched = false;
                        for candidate in candidates {
                            if let Ok(MessageContent::Text(text)) =
                                app.service.decrypt_token_mailbox_envelope(&candidate, &envelope)
                            {
                                let line = format!("anon_text\t{:?}\t{}", candidate.endpoint_addr.id, text.as_str());
                                app.events.lock().expect("events lock poisoned").push_back(line);
                                matched = true;
                                break;
                            }
                        }
                        if !matched {
                            tracing::debug!("received a TokenMailboxDeposit that didn't decrypt against any known peer");
                        }
                    }
                    _ => {} // MailboxCheckIn/AnonymousMailboxCheckIn/RouteAdvertisement — not this client's traffic (it only ever sends the first two, never receives them back; RouteAdvertisement is relay-to-relay)
                }
                // A crude batch boundary: report whatever mailbox items
                // have accumulated once the channel briefly empties,
                // rather than trying to detect "the relay is done
                // sending" any more precisely — `try_recv` draining
                // what's immediately available is a real, if approximate,
                // stand-in for `apps/cli::check_mailbox`'s fixed 5-second
                // deadline (this crate has no equivalent timer here).
                if mailbox_batch_count > 0 {
                    while rx.try_recv().is_ok() {
                        mailbox_batch_count += 1;
                    }
                    let line = format!("mailbox\t{mailbox_batch_count}");
                    app.events.lock().expect("events lock poisoned").push_back(line);
                    mailbox_batch_count = 0;
                }
            }
        });
        *app.pump_handle.lock().expect("pump_handle lock poisoned") = Some(pump_handle);

        Ok::<Arc<AppMessaging>, String>(app)
    })?;

    // Installed only now that everything above succeeded — `APP` never
    // sees a partially-built `AppMessaging`. If another call already
    // won the race and installed first, this call's own endpoint/pump
    // are real but redundant: stop its pump immediately and let its
    // `Arc` drop (see `shutdown_inner`'s own comment on why `abort`
    // doesn't guarantee the drop lands synchronously — same caveat
    // here) rather than erroring the caller out, since the winner's
    // ticket is just as valid an answer to "what's my ticket".
    let mut guard = APP.lock().expect("APP lock poisoned");
    if let Some(existing) = guard.as_ref() {
        let ticket = existing.my_ticket.encode();
        drop(guard);
        if let Some(handle) = app_arc.pump_handle.lock().expect("pump_handle lock poisoned").take() {
            handle.abort();
        }
        return Ok(ticket);
    }
    let ticket = app_arc.my_ticket.encode();
    *guard = Some(app_arc);
    Ok(ticket)
}

fn my_ticket_inner() -> String {
    app_handle().expect("bootstrap must run before my_ticket").my_ticket.encode()
}

fn add_peer_inner(ticket: &str) -> Result<(), String> {
    let app = app_handle().ok_or("bootstrap must run before add_peer")?;
    let peer = PeerTicket::decode(ticket).map_err(|e| e.to_string())?;
    app.known_peers.lock().expect("known_peers lock poisoned").insert(peer.endpoint_addr.id, peer);
    Ok(())
}

fn send_text_inner(peer_ticket: &str, text: &str) -> Result<String, String> {
    let app = app_handle().ok_or("bootstrap must run before send_text")?;
    let peer = PeerTicket::decode(peer_ticket).map_err(|e| e.to_string())?;
    let text = MessageText::parse(text.to_string()).map_err(|e| e.to_string())?;
    // Real evidence-based link classification, replacing the bootstrap-
    // time blanket `InternetDirect` — see `siar_routing::path::
    // classify_endpoint_addr`'s own doc comment for exactly what this
    // is (advertised reachability) and isn't (a measured path).
    siar_android_connectivity::mark_link_up(siar_routing::path::classify_endpoint_addr(&peer.endpoint_addr));
    runtime().block_on(async {
        // Phase-1 stand-in, same as `apps/cli`'s own `send`: a real
        // client looks up (or creates) the conversation with this peer
        // rather than minting a fresh one per call.
        let conversation = ConversationId::new();
        app.service.send_text(conversation, &peer, text).await.map(|id| id.to_string()).map_err(|e| e.to_string())
    })
}

fn check_mailbox_inner(relay_ticket: &str) -> Result<(), String> {
    let app = app_handle().ok_or("bootstrap must run before check_mailbox")?;
    let relay = PeerTicket::decode(relay_ticket).map_err(|e| e.to_string())?;
    let check_in = app.service.sign_mailbox_check_in(siar_domain::now_millis());
    runtime().block_on(async {
        app.endpoint
            .send(relay.endpoint_addr.clone(), &siar_protocol::WireMessage::MailboxCheckIn(check_in))
            .await
            .map_err(|e| e.to_string())
    })
}

/// The unlinkable counterpart to [`send_text_inner`] — see
/// `MessageService::send_text_anon`'s own doc comment for exactly what
/// this does and doesn't guarantee (no delivery ack/retry on this
/// path, in particular). `peer_ticket` is who the message is *for*;
/// `relay_ticket` is who it's handed to for pickup.
fn send_text_anon_inner(peer_ticket: &str, relay_ticket: &str, text: &str) -> Result<String, String> {
    let app = app_handle().ok_or("bootstrap must run before send_text_anon")?;
    let peer = PeerTicket::decode(peer_ticket).map_err(|e| e.to_string())?;
    let relay = PeerTicket::decode(relay_ticket).map_err(|e| e.to_string())?;
    let text = MessageText::parse(text.to_string()).map_err(|e| e.to_string())?;
    // Classified against the relay's address, not the peer's — this
    // path is delivered *through* the relay (see `send_text_anon`'s
    // own doc comment), so the relay is the actual link this device
    // talks over, same reasoning `classify_endpoint_addr` applies
    // anywhere else in this workspace: classify what's actually
    // reached, not the final recipient the traffic is addressed to.
    siar_android_connectivity::mark_link_up(siar_routing::path::classify_endpoint_addr(&relay.endpoint_addr));
    runtime().block_on(async {
        app.service.send_text_anon(&peer, &relay, text).await.map(|id| id.to_string()).map_err(|e| e.to_string())
    })
}

/// The unlinkable counterpart to [`check_mailbox_inner`] — presents a
/// rotating token derived against `peer_ticket` instead of this
/// device's own `DeviceId`. Responses land in the poll queue as
/// `anon_text\t...` lines (see the incoming-frame pump's own comment
/// on `WireMessage::TokenMailboxDeposit` for how the sender is
/// recovered without a sender field on the wire). `peer_ticket` must
/// already be registered via [`add_peer_inner`] — the pump can't try
/// decrypting against a peer it doesn't know about.
fn check_mailbox_anon_inner(peer_ticket: &str, relay_ticket: &str) -> Result<(), String> {
    let app = app_handle().ok_or("bootstrap must run before check_mailbox_anon")?;
    let peer = PeerTicket::decode(peer_ticket).map_err(|e| e.to_string())?;
    let relay = PeerTicket::decode(relay_ticket).map_err(|e| e.to_string())?;
    let check_in = app.service.build_anonymous_check_in(&peer, siar_domain::now_millis());
    runtime().block_on(async {
        app.endpoint
            .send(relay.endpoint_addr.clone(), &siar_protocol::WireMessage::AnonymousMailboxCheckIn(check_in))
            .await
            .map_err(|e| e.to_string())
    })
}

/// Real, previously-missing piece the chat UI needs: Kotlin has no way
/// to decode a `PeerTicket` string itself (that's real Rust-side
/// decoding logic, not duplicated here), so without this there was no
/// way for a contact added via [`add_peer_inner`] to be matched back
/// against [`poll_next_event_inner`]'s `text\t<sender endpoint id
/// hex>\t...` lines — those two strings (a `PeerTicket` and a
/// `{:?}`-formatted `iroh::EndpointId`) look nothing alike even though
/// they can identify the same peer. This decodes a ticket and returns
/// the exact same `{:?}` (Debug) formatting the incoming-frame pump
/// already uses for `frame.from`, so a Kotlin-side contact list can
/// key its threads by this value and actually match incoming messages
/// to the contact that sent them. Pure — doesn't require [`bootstrap`]
/// to have run first, since it only decodes the ticket string itself.
fn ticket_endpoint_debug_inner(ticket: &str) -> Result<String, String> {
    let peer = PeerTicket::decode(ticket).map_err(|e| e.to_string())?;
    Ok(format!("{:?}", peer.endpoint_addr.id))
}

fn poll_next_event_inner() -> Option<String> {
    app_handle()?.events.lock().expect("events lock poisoned").pop_front()
}

/// The `mark_link_up` counterpart this crate's own doc comment named
/// as missing — reports `TransportLink::InternetDirect` down when
/// Kotlin calls it (intended for `MainActivity.onDestroy`/`onStop`).
///
/// Now a genuine teardown, not just a connectivity-state update: `APP`
/// holding `Mutex<Option<Arc<AppMessaging>>>` instead of a `OnceLock`
/// (see that static's own comment for why) means this can actually
/// take the app out of the slot, stop its pump task, and let its
/// `Arc<SiarEndpoint>` drop instead of leaking until process exit.
/// Concretely: `.take()` empties `APP` immediately, so every other
/// function in this file (`app_handle`) sees "not bootstrapped" from
/// this point on and a later `bootstrap` call genuinely builds a fresh
/// endpoint/service/pump rather than silently handing back the old
/// ticket. The pump is stopped via `JoinHandle::abort`, which
/// schedules cancellation rather than guaranteeing it lands
/// synchronously — Tokio drops an aborted task's captured state
/// (including its own `Arc<AppMessaging>` clone, the last thing
/// keeping the endpoint alive once this function's local `app` binding
/// also drops at the end of this function) at its own next scheduling
/// point, not necessarily before this function returns. That's a real
/// limitation worth naming, not a gap papered over: this function's
/// synchronous, verifiable guarantee is that `APP` is empty and the
/// pump has been told to stop the instant it returns; full resource
/// release (the endpoint's own `Drop`, whatever that does) follows
/// shortly after on the runtime's own schedule.
fn shutdown_inner() {
    let Some(app) = APP.lock().expect("APP lock poisoned").take() else { return };
    if let Some(handle) = app.pump_handle.lock().expect("pump_handle lock poisoned").take() {
        handle.abort();
    }
    siar_android_connectivity::mark_link_down(siar_domain::TransportLink::InternetDirect);
    // `app` drops here — the slot's own strong reference to
    // `AppMessaging` (and, through it, `Arc<SiarEndpoint>`) is gone;
    // only the aborted pump task's clone remains until Tokio finishes
    // dropping that task, per the comment above.
}

#[cfg(target_os = "android")]
mod jni_bridge {
    use super::*;
    use jni::objects::{JClass, JString};
    use jni::sys::jstring;
    use jni::JNIEnv;

    /// Turns a `Result<String, String>`/`Option<String>` into a
    /// `jstring`, prefixing an `Err` with `"error:"` — the simplest
    /// possible error channel across this boundary (Kotlin checks
    /// `result.startsWith("error:")`) rather than a second JNI calling
    /// convention (exceptions, out-parameters) for the rare failure
    /// case. Consistent within this crate; not necessarily how a larger
    /// FFI surface should scale this pattern.
    fn to_jstring<'local>(env: &mut JNIEnv<'local>, value: Result<String, String>) -> jstring {
        let text = match value {
            Ok(s) => s,
            Err(e) => format!("error:{e}"),
        };
        env.new_string(text).expect("failed to allocate a JNI string").into_raw()
    }

    fn jstring_to_string<'local>(env: &mut JNIEnv<'local>, s: &JString<'local>) -> String {
        env.get_string(s).expect("invalid UTF-8 from Kotlin").into()
    }

    /// Real, previously-missing prerequisite for `bootstrap`/`SiarEndpoint::
    /// bind` to work reliably on Android — confirmed directly against
    /// iroh 1.0.3's real published docs (`docs.rs/iroh/1.0.3/iroh/
    /// endpoint/struct.Endpoint.html`'s own "Usage on Android" section,
    /// and `docs.rs/iroh-dns/1.0.3/iroh_dns/fn.install_android_jni_context.html`
    /// directly — not guessed): the endpoint's default `DnsResolver`
    /// reads Android's system DNS configuration through JNI, which
    /// needs a `JavaVM`/Application `Context` published to
    /// `ndk_context` *before* the endpoint is constructed. Without
    /// this, iroh falls back to Google's public DNS servers — and if
    /// this app's compilation profile ever sets `panic = "abort"`
    /// (not currently the case, but worth flagging), the fallback
    /// detection itself can't work and the app would panic instead.
    ///
    /// `JNI_OnLoad` is the standard, un-namespaced JNI entry point the
    /// JVM calls automatically the moment `System.loadLibrary
    /// ("siar_android_messaging")` finishes loading this `.so` —
    /// before any `Java_com_siar_messaging_...` function is ever
    /// called, which is exactly the ordering this needs. No explicit
    /// Kotlin-side call required.
    ///
    /// One genuine unresolved uncertainty, flagged rather than
    /// papered over: the real docs.rs example this is copied from
    /// passes `JNI_OnLoad`'s own `res: *mut c_void` parameter (the
    /// JNI spec's "reserved" argument, conventionally null) straight
    /// through as `context_jobject` — the same pattern shown on the
    /// real docs page verbatim. Whether that's actually a valid
    /// Application `Context` in practice, or whether this needs a
    /// separate real Context passed in from Kotlin some other way,
    /// isn't something this pass could verify by running it — flagged
    /// so a real device/emulator run's DNS-resolution behavior is what
    /// confirms or corrects this, not blind trust in one docs example.
    #[no_mangle]
    pub extern "C" fn JNI_OnLoad(vm: jni::JavaVM, reserved: *mut std::ffi::c_void) -> jni::sys::jint {
        let java_vm = match vm.get_java_vm_pointer() as *mut std::ffi::c_void {
            ptr if !ptr.is_null() => ptr,
            // JNI spec's well-known `JNI_ERR` value (-1) — used as a
            // literal rather than a `jni::sys::JNI_ERR` constant this
            // pass couldn't independently confirm exists under that
            // exact name in the `jni` 0.21 crate.
            _ => return -1,
        };
        unsafe {
            iroh::dns::install_android_jni_context(java_vm, reserved);
        }
        jni::JNIVersion::V6.into()
    }

    #[no_mangle]
    pub extern "system" fn Java_com_siar_messaging_NativeMessagingBridge_bootstrap<'local>(
        mut env: JNIEnv<'local>,
        _class: JClass<'local>,
        base_dir: JString<'local>,
    ) -> jstring {
        let base_dir = jstring_to_string(&mut env, &base_dir);
        let result = bootstrap_inner(&base_dir);
        to_jstring(&mut env, result)
    }

    #[no_mangle]
    pub extern "system" fn Java_com_siar_messaging_NativeMessagingBridge_ticketEndpointDebug<'local>(
        mut env: JNIEnv<'local>,
        _class: JClass<'local>,
        ticket: JString<'local>,
    ) -> jstring {
        let ticket = jstring_to_string(&mut env, &ticket);
        let result = ticket_endpoint_debug_inner(&ticket);
        to_jstring(&mut env, result)
    }

    #[no_mangle]
    pub extern "system" fn Java_com_siar_messaging_NativeMessagingBridge_addPeer<'local>(
        mut env: JNIEnv<'local>,
        _class: JClass<'local>,
        ticket: JString<'local>,
    ) -> jstring {
        let ticket = jstring_to_string(&mut env, &ticket);
        let result = add_peer_inner(&ticket).map(|()| "ok".to_string());
        to_jstring(&mut env, result)
    }

    #[no_mangle]
    pub extern "system" fn Java_com_siar_messaging_NativeMessagingBridge_sendText<'local>(
        mut env: JNIEnv<'local>,
        _class: JClass<'local>,
        peer_ticket: JString<'local>,
        text: JString<'local>,
    ) -> jstring {
        let peer_ticket = jstring_to_string(&mut env, &peer_ticket);
        let text = jstring_to_string(&mut env, &text);
        let result = send_text_inner(&peer_ticket, &text);
        to_jstring(&mut env, result)
    }

    #[no_mangle]
    pub extern "system" fn Java_com_siar_messaging_NativeMessagingBridge_checkMailbox<'local>(
        mut env: JNIEnv<'local>,
        _class: JClass<'local>,
        relay_ticket: JString<'local>,
    ) -> jstring {
        let relay_ticket = jstring_to_string(&mut env, &relay_ticket);
        let result = check_mailbox_inner(&relay_ticket).map(|()| "ok".to_string());
        to_jstring(&mut env, result)
    }

    #[no_mangle]
    pub extern "system" fn Java_com_siar_messaging_NativeMessagingBridge_sendTextAnon<'local>(
        mut env: JNIEnv<'local>,
        _class: JClass<'local>,
        peer_ticket: JString<'local>,
        relay_ticket: JString<'local>,
        text: JString<'local>,
    ) -> jstring {
        let peer_ticket = jstring_to_string(&mut env, &peer_ticket);
        let relay_ticket = jstring_to_string(&mut env, &relay_ticket);
        let text = jstring_to_string(&mut env, &text);
        let result = send_text_anon_inner(&peer_ticket, &relay_ticket, &text);
        to_jstring(&mut env, result)
    }

    #[no_mangle]
    pub extern "system" fn Java_com_siar_messaging_NativeMessagingBridge_checkMailboxAnon<'local>(
        mut env: JNIEnv<'local>,
        _class: JClass<'local>,
        peer_ticket: JString<'local>,
        relay_ticket: JString<'local>,
    ) -> jstring {
        let peer_ticket = jstring_to_string(&mut env, &peer_ticket);
        let relay_ticket = jstring_to_string(&mut env, &relay_ticket);
        let result = check_mailbox_anon_inner(&peer_ticket, &relay_ticket).map(|()| "ok".to_string());
        to_jstring(&mut env, result)
    }

    /// Returns `null` (not an empty string — Kotlin's `external fun`
    /// declares this nullable) when nothing is queued, so a caller
    /// polling on a timer can distinguish "nothing yet" from a real
    /// empty-string event, which none of today's two event kinds would
    /// ever produce anyway but a future one might.
    #[no_mangle]
    pub extern "system" fn Java_com_siar_messaging_NativeMessagingBridge_pollNextEvent<'local>(
        env: JNIEnv<'local>,
        _class: JClass<'local>,
    ) -> jstring {
        match poll_next_event_inner() {
            Some(line) => env.new_string(line).expect("failed to allocate a JNI string").into_raw(),
            None => std::ptr::null_mut(),
        }
    }

    #[no_mangle]
    pub extern "system" fn Java_com_siar_messaging_NativeMessagingBridge_shutdown<'local>(
        _env: JNIEnv<'local>,
        _class: JClass<'local>,
    ) {
        shutdown_inner();
    }
}
