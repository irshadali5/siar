//! next.md §81: a dedicated relay node — Raspberry Pi, old laptop, or
//! small Linux box, running BLE/Wi-Fi/Ethernet/Iroh with a large DTN
//! store. Phase 7 of `next.md`'s roadmap.
//!
//! What this binary actually does right now: generates or loads a
//! *persistent* identity (unlike `apps/cli`'s throwaway
//! `DeviceIdentity::generate()` every run — a relay node needs the same
//! identity across restarts so peers can keep trusting it, hence this
//! phase's addition of `DeviceIdentity::save_to_file`/`load_from_file`
//! to `siar-crypto`), binds a [`SiarEndpoint`] (reusing every transport
//! already built — Iroh direct/relay, LAN mDNS from Phase 1), prints
//! its connection ticket the same way `apps/cli`'s `listen` does, and
//! constructs the DTN machinery Phases 4–5 built (`BundleStore`,
//! `SeenBundles`, `PathTable`, `PriorityScheduler`), sized from CLI
//! flags.
//!
//! What it does now, this pass: real destination-aware forwarding,
//! replacing the naive flood this file's own doc comment used to
//! describe as the whole story, plus authenticating the mailbox
//! check-ins that forwarding now partly depends on. Three real pieces
//! closed these gaps:
//!
//! - **`siar_connectivity::TransportManager`** existed (built in an
//!   earlier session closing `siar-routing`'s own flagged gap) but was
//!   never actually constructed or used anywhere in this binary —
//!   confirmed by grepping this file before this pass touched it: the
//!   name only appeared in doc-comment prose, never in an `use` or a
//!   `let`. Now it's wired: `sync_local_peers` runs on a timer,
//!   keeping `PathTable` current with whichever peers Phase 1's mDNS
//!   discovery can currently see on the LAN.
//! - **`siar_routing::device_routes::DeviceRoutes`**, new this pass —
//!   see that module's own doc comment for the full reasoning. In
//!   short: `PathTable` is keyed on `EndpointId`, but a `MeshBundle`'s
//!   `destination` is a `DeviceId`; nothing anywhere mapped one to the
//!   other. `MailboxCheckIn`'s self-disclosure is the one real signal
//!   this relay has for that mapping, so it's recorded here and used to
//!   push a bundle proactively to its destination's last-known endpoint
//!   the moment it arrives, instead of only reactively flooding it to
//!   whoever happens to make contact next.
//! - **`siar_protocol::mailbox::DeviceKeyDirectory`**, new this pass —
//!   next.md §32's mailbox-authentication gap, which `mailbox.rs`'s own
//!   doc comment used to flag outright ("this type's `device` field is
//!   a bare, unauthenticated claim"). A `MailboxCheckIn` is now
//!   verified (Ed25519 signature + freshness window) and its claimed
//!   device's key trust-on-first-use pinned before this relay trusts it
//!   for anything — including the `DeviceRoutes` recording above, which
//!   would otherwise have been trusting the exact same unauthenticated
//!   claim it was built to act on.
//!
//! The remaining naive-flood fallback (for a bundle whose destination
//! has no known endpoint hint yet) now at least orders its candidates
//! by `SchedulePriority` via `PriorityScheduler`, instead of whatever
//! order `BundleStore::iter()` happened to yield.
//!
//! What's still NOT real destination-aware routing, flagged rather than
//! oversold: `DeviceRoutes` only ever learns a mapping from a device's
//! own voluntary check-in — there's still no way to learn "device X is
//! reachable via peer Y" from ordinary traffic, since `MeshEnvelope`
//! deliberately carries no sender identity (next.md §73–74's mesh-
//! privacy design). A destination that's never checked in with this
//! relay is still only reachable via the naive flood. Multi-hop
//! path computation, BLE→Wi-Fi upgrades, and gateway bridging remain
//! exactly as unbuilt as `siar-routing`'s own crate doc comment already
//! states. And mailbox authentication is still only half of next.md
//! §32 — see `mailbox.rs`'s own doc comment for the unlinkability half
//! that remains open on purpose.
//!
//! Congestion detection (also flagged unbuilt in earlier passes) is now
//! real for the queue-occupancy half: the candidate-forwarding loop
//! below derives its `dequeue_next` ceiling from
//! `PriorityScheduler::congestion_ceiling` instead of a hardcoded
//! `None`. The RTT/reliability half (`siar_routing::link_health::
//! LinkHealth`, wired into `siar_connectivity::TransportManager::
//! record_send_outcome`) now has a real caller: every `endpoint.send`
//! in this file goes through the new `send_and_record` helper, which
//! times the attempt and folds the outcome back into `TransportManager`
//! — see that function's own doc comment for the one honest
//! approximation it makes (classifying every send as
//! `TransportLink::InternetDirect` without actually checking whether
//! iroh negotiated a direct connection or fell back to relay).
//!
//! Multi-hop route computation also moved from "nothing" to "a real,
//! bounded primitive with no real caller": `siar_routing::path::
//! PathTable::compose_via_relay` can now derive a genuine 2-hop
//! candidate route once given a `RelayAdvertisement`, but nothing in
//! this binary (or anywhere in this workspace) produces one — this
//! relay still has no routing-advertisement exchange, so `DeviceRoutes`
//! remains this binary's only actual multi-hop-relevant signal, and
//! only for the one-hop "device checked in with me directly" case. See
//! `siar-routing`'s own crate doc comment for the full accounting.
//!
//! This relay now also adopts `siar_protocol::mailbox::
//! TokenMailboxStore` — the token-keyed counterpart to `bundle_store`
//! below, filled by `WireMessage::TokenMailboxDeposit` and drained by
//! `WireMessage::AnonymousMailboxCheckIn`. Unlike the `MailboxCheckIn`
//! arm, there's no signature check here — presenting a token *is* the
//! authorization (see `siar_crypto::mailbox_token`'s own doc comment
//! for that bearer-capability tradeoff). `apps/cli` now has a real
//! sender/receiver for this path (`send-anon`/`check-mailbox-anon`,
//! via `MessageService::send_text_anon`/`build_anonymous_check_in`/
//! `decrypt_token_mailbox_envelope`) — `apps/desktop` still only
//! builds/sends the `DeviceId`-addressed `MailboxCheckIn`; wiring the
//! same choice into its UI is separate, real follow-up work.
//! real follow-up work.
//!
//! Finally, this relay now sends and receives real
//! `WireMessage::RouteAdvertisement`s — `siar_routing::path::
//! PathTable::compose_via_relay`'s own doc comment named this exchange
//! as not existing anywhere in the workspace; it now does. See
//! `route_advertisement.rs`'s doc comment for the message shape and,
//! importantly, its deliberately unauthenticated trust model. The raw-
//! bytes `EndpointId` round-trip below (`PublicKey::as_bytes`/
//! `from_bytes`) was verified directly against iroh 1.0.3's real
//! published docs this pass, not guessed — the one iroh API surface
//! this pass could confirm without a compiler, since docs.rs is
//! reachable even though building iroh itself still isn't (see this
//! workspace's own memory of the edition2024 wall for why).
//!
//! `MeshBundle` (`siar-dtn::bundle`) gained `destination`/`payload_hash`
//! fields alongside an earlier pass's forwarding logic — without them, a
//! bundle converted from a received `MeshEnvelope` for storage had no
//! way to be converted back into a valid one to forward. Every existing
//! construction site was updated to match.
//!
//! `siar_messaging::MessageService::handle_incoming` requires a
//! `&PeerTicket` — the sender's public keys, known *in advance* — to
//! decrypt anything (see `apps/cli`'s own `listen` mode, which takes
//! exactly one peer ticket up front). `WireMessage::V1` frames from an
//! unpaired stranger still can't be processed by this relay for that
//! reason; a stranger's traffic has to arrive as a `MeshEnvelope`
//! instead, whose whole design point (see `siar-protocol::mesh`'s own
//! doc comment) is not needing a session at all.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use siar_connectivity::TransportManager;
use siar_crypto::DeviceIdentity;
use siar_domain::DeviceId;
use siar_dtn::dedup::SeenBundles;
use siar_dtn::store::BundleStore;
use siar_protocol::{DeviceKeyDirectory, RouteAdvertisement, TokenMailboxEnvelope, TokenMailboxStore, WireMessage};
use siar_routing::device_routes::DeviceRoutes;
use siar_routing::path::RelayAdvertisement;
use siar_routing::scheduler::{PriorityScheduler, SchedulePriority};
use siar_transport::{PeerTransport, SiarEndpoint};
use tokio::sync::mpsc;

/// next.md §68: "Emergency: configurable 500 MB – several GB" — this
/// binary's own default, not a hard limit; override with
/// `--quota-bytes`.
const DEFAULT_QUOTA_BYTES: u64 = 1024 * 1024 * 1024; // 1 GiB
const DEFAULT_SEEN_CAPACITY: usize = 100_000;
const DEFAULT_SCHEDULER_CAPACITY_PER_QUEUE: usize = 1024;
/// Fraction of a throttled tier's capacity that counts as "backed up"
/// for `PriorityScheduler::congestion_ceiling` (see that method's doc
/// comment). Chosen conservatively — react to backlog before a queue
/// is anywhere near actually full and starting to reject new items —
/// not tuned against real traffic, same status every other constant in
/// this file carries.
const CONGESTION_OCCUPANCY_THRESHOLD: f32 = 0.5;
/// How long a `DeviceRoutes`/`PathTable` entry is trusted before
/// `remove_stale` drops it — next.md §92's "mobile topology changes too
/// quickly" reasoning, same one `PathTable::remove_stale`'s own doc
/// comment already gives. Ten minutes, not a value next.md specifies
/// anywhere: a relay node is meant to be relatively stationary
/// (Raspberry Pi / small box, per this file's own top doc comment), so
/// this favors "stale entries get cleaned up eventually" over guessing
/// at a tighter mobile-handset-appropriate number this binary doesn't
/// need.
const ROUTE_STALE_AFTER_MILLIS: u64 = 10 * 60 * 1000;
/// How often the periodic sync/cleanup task runs.
const SYNC_INTERVAL: std::time::Duration = std::time::Duration::from_secs(5);
/// Freshness window for a `MailboxCheckIn`'s signature — see
/// `siar_protocol::mailbox::MailboxCheckIn::verify`'s own doc comment
/// for why this doubles as clock-skew tolerance on both sides. Five
/// minutes: generous enough that ordinary clock drift between a phone
/// and this relay won't cause spurious rejections, tight enough that a
/// captured check-in can't be replayed hours or days later.
const MAILBOX_CHECKIN_MAX_AGE_MILLIS: u64 = 5 * 60 * 1000;

struct Config {
    identity_path: PathBuf,
    quota_bytes: u64,
    seen_capacity: usize,
}

impl Config {
    fn from_args() -> Self {
        let mut config =
            Self { identity_path: default_identity_path(), quota_bytes: DEFAULT_QUOTA_BYTES, seen_capacity: DEFAULT_SEEN_CAPACITY };

        let mut args = std::env::args().skip(1);
        while let Some(flag) = args.next() {
            match flag.as_str() {
                "--identity" => {
                    if let Some(value) = args.next() {
                        config.identity_path = PathBuf::from(value);
                    }
                }
                "--quota-bytes" => {
                    if let Some(value) = args.next().and_then(|v| v.parse().ok()) {
                        config.quota_bytes = value;
                    }
                }
                "--seen-capacity" => {
                    if let Some(value) = args.next().and_then(|v| v.parse().ok()) {
                        config.seen_capacity = value;
                    }
                }
                other => {
                    tracing::warn!(flag = other, "unrecognized flag, ignoring");
                }
            }
        }

        config
    }
}

fn default_identity_path() -> PathBuf {
    // next.md §81's target platforms (Raspberry Pi / small Linux box)
    // are exactly where $HOME is reliably set; falling back to the
    // current directory rather than panicking if it somehow isn't —
    // a relay node refusing to start over a missing env var is a worse
    // failure mode than writing its identity file next to the binary.
    let base = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
    base.join(".siar-emergency-node").join("identity.bin")
}

fn load_or_create_identity(path: &std::path::Path) -> Result<DeviceIdentity> {
    if path.exists() {
        return DeviceIdentity::load_from_file(path).context("loading existing identity");
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).context("creating identity directory")?;
    }
    let identity = DeviceIdentity::generate();
    identity.save_to_file(path).context("saving new identity")?;
    tracing::info!(path = %path.display(), "generated new relay node identity");
    Ok(identity)
}

/// A stored bundle carries everything a `MeshEnvelope` needs (see
/// `siar-dtn::bundle::MeshBundle`'s own doc comment on why
/// `destination`/`payload_hash` were added specifically so this
/// round-trip is possible) — shared by both this file's forward-on-
/// contact step and its mailbox check-in handler, which both need to
/// turn a stored bundle back into wire bytes.
fn bundle_to_envelope(bundle: siar_dtn::bundle::MeshBundle) -> siar_protocol::MeshEnvelope {
    siar_protocol::MeshEnvelope {
        id: bundle.id,
        destination: bundle.destination,
        created_at: bundle.created_at,
        expires_at: bundle.expires_at,
        hop_limit: bundle.hop_limit,
        priority: bundle.priority,
        payload_hash: bundle.payload_hash,
        ciphertext: bundle.ciphertext,
    }
}

/// Times a real `SiarEndpoint::send` attempt and folds the outcome into
/// `TransportManager`'s `LinkHealth`/`PathTable` via
/// `record_send_outcome` — the real caller
/// `siar_routing::link_health::LinkHealth::record_outcome`'s own doc
/// comment named as missing from this workspace ever since it was
/// built, and `TransportManager::record_send_outcome`'s own doc comment
/// echoed the same thing. Every real outbound send in this file now
/// goes through this instead of calling `endpoint.send` directly, so
/// `PathTable`'s `rtt_millis`/`reliability` fields stop being permanent
/// `None`/`1.0` placeholders the moment this relay actually talks to
/// anyone.
///
/// Classifies every send this function makes as
/// `TransportLink::InternetDirect` — an honest approximation, not a
/// verified fact: `PeerTransport::send`'s current signature doesn't
/// expose whether iroh actually negotiated a direct connection or fell
/// back to one of its relay servers for this particular send (that
/// distinction needs inspecting the underlying `iroh::endpoint::
/// Connection`'s `remote_info()`, which isn't reachable from here).
/// `InternetDirect` is the more common case for this workspace's Phase
/// 1 (Iroh Internet+LAN) plane and the honest default until a real
/// direct-vs-relayed distinction is wired in — narrower than
/// `TransportLink::InternetRelay` sometimes being the true answer, but
/// not fabricated as if this function actually checked.
async fn send_and_record(
    endpoint: &SiarEndpoint,
    transport_manager: &TransportManager,
    destination: iroh::EndpointId,
    message: &WireMessage,
) -> Result<(), siar_transport::TransportError> {
    let started = std::time::Instant::now();
    let result = endpoint.send(iroh::EndpointAddr::new(destination), message).await;
    let elapsed_millis = u32::try_from(started.elapsed().as_millis()).unwrap_or(u32::MAX);
    let outcome = match &result {
        Ok(()) => siar_routing::link_health::SendOutcome::success(elapsed_millis),
        Err(_) => siar_routing::link_health::SendOutcome::failure(),
    };
    transport_manager.record_send_outcome(
        destination,
        siar_domain::TransportLink::InternetDirect,
        siar_routing::path::NextHop::Direct,
        siar_domain::now_millis(),
        outcome,
    );
    result
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let config = Config::from_args();

    let identity = load_or_create_identity(&config.identity_path)?;
    let device_id = DeviceId::new();

    // Same bootstrap shape as `apps/cli`'s `bootstrap()` — in-memory
    // storage for now, same as that CLI; a relay node's own message/
    // outbox/blob persistence (as opposed to the DTN bundle store below,
    // which is what actually needs to survive a restart for this
    // binary's purpose) is real work this phase didn't need to touch.
    let db = siar_storage::open_in_memory().context("opening local database")?;
    let messages = Arc::new(siar_storage::StoolapMessageRepository::new(db.clone()));
    let outbox = Arc::new(siar_storage::StoolapOutboxRepository::new(db.clone()));
    let blobs: Arc<dyn siar_storage::BlobRepository + Send + Sync> = Arc::new(siar_storage::StoolapBlobRepository::new(db));
    let blob_store: Arc<dyn siar_transport::BlobStore> = Arc::new(siar_messaging::StorageBlobStore(blobs.clone()));

    let (tx, mut rx) = mpsc::channel::<siar_transport::IncomingFrame>(256);
    let iroh_secret = iroh::SecretKey::generate();
    let endpoint = Arc::new(SiarEndpoint::bind(iroh_secret, tx, blob_store).await.context("binding endpoint")?);

    let ticket = siar_messaging::PeerTicket {
        endpoint_addr: endpoint.addr(),
        x25519_public: identity.x25519_public().to_bytes(),
        ed25519_verifying: identity.verifying_key().to_bytes(),
    };
    // next.md §111: a relay node's own contact card, the same concept
    // as a phone's QR pairing — just printed as text on a terminal
    // instead of rendered as a QR image.
    tracing::info!(ticket = %ticket.encode(), "emergency relay node ready");

    let _service = Arc::new(siar_messaging::MessageService::new(device_id, identity, endpoint.clone(), messages, outbox, blobs));

    // next.md §68's DTN storage, §31's dedup, §91's path table, §93's
    // scheduler, plus this pass's `DeviceRoutes` — see this file's top
    // doc comment for how they're actually wired together now.
    // `Mutex`, not the async-aware channel types elsewhere in this
    // workspace: everything below is short synchronous critical
    // sections (lock, read/mutate, drop before any `.await` — see the
    // receive loop's own comments on why that ordering matters), never
    // held across a suspend point.
    let bundle_store: Mutex<BundleStore> = Mutex::new(BundleStore::new(config.quota_bytes));
    let seen: Mutex<SeenBundles> = Mutex::new(SeenBundles::new(config.seen_capacity));
    let transport_manager = Arc::new(TransportManager::new(endpoint.clone()));
    let device_routes: Mutex<DeviceRoutes> = Mutex::new(DeviceRoutes::new());
    let device_keys: Mutex<DeviceKeyDirectory> = Mutex::new(DeviceKeyDirectory::new());
    // The unlinkable counterpart to `bundle_store` — see
    // `siar_protocol::mailbox::TokenMailboxStore`'s own doc comment for
    // why this needs to be a structurally separate store rather than a
    // second index into `bundle_store`. Reuses the same `seen`
    // dedup set as `bundle_store`'s `Mesh` path below (a `MessageId` is
    // globally unique regardless of which addressing scheme a given
    // message used, so one dedup set correctly covers both).
    let token_mailbox: Mutex<TokenMailboxStore<TokenMailboxEnvelope>> = Mutex::new(TokenMailboxStore::new());
    let scheduler: Mutex<PriorityScheduler<siar_domain::MessageId>> =
        Mutex::new(PriorityScheduler::new(DEFAULT_SCHEDULER_CAPACITY_PER_QUEUE));

    // Keeps `PathTable` (via `TransportManager`) and `DeviceRoutes` both
    // current on a timer — next.md §92's "mobile topology changes too
    // quickly" applies to both: a LAN peer that's walked out of mDNS
    // range, or a device whose `MailboxCheckIn` endpoint hint is stale,
    // should both stop being trusted eventually rather than lingering
    // forever.
    //
    // The same tick also *sends* `RouteAdvertisement`s — the other half
    // of the exchange `WireMessage::RouteAdvertisement`'s receive-side
    // arm above consumes. Deliberately narrow, matching
    // `route_advertisement.rs`'s own "no propagation policy beyond
    // this" doc comment: advertises only this relay's own *direct*
    // routes (`NextHop::Direct` — never re-advertising something heard
    // from someone else's advertisement, which is exactly the
    // unbounded-flooding case that doc comment flags as unhandled), to
    // every currently-known local peer, once per tick — no fan-out
    // beyond that.
    {
        let transport_manager = transport_manager.clone();
        let endpoint = endpoint.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(SYNC_INTERVAL);
            loop {
                interval.tick().await;
                let now = siar_domain::now_millis();
                transport_manager.sync_local_peers(now);
                transport_manager.path_table().remove_stale(now, ROUTE_STALE_AFTER_MILLIS);

                // Snapshot of (destination, best direct entry) pairs,
                // resolved and the lock dropped before any `.await`
                // below — same "resolve then act" split this file's
                // other `Mutex`-guarded sections already use for the
                // same reason (holding a `MutexGuard` across
                // `endpoint.send(...).await` would block every other
                // task waiting on this same lock for the duration of a
                // network send).
                let direct_routes: Vec<(iroh::EndpointId, siar_routing::path::PathEntry)> = {
                    let table = transport_manager.path_table();
                    table
                        .destinations()
                        .filter_map(|destination| {
                            table
                                .routes_for(destination)
                                .iter()
                                .find(|entry| entry.next_hop == siar_routing::path::NextHop::Direct)
                                .map(|entry| (destination, *entry))
                        })
                        .collect()
                };
                if direct_routes.is_empty() {
                    continue;
                }

                let peers = endpoint.local_peers();
                for peer in &peers {
                    for (destination, entry) in &direct_routes {
                        // Advertising a peer's own route back to itself
                        // is a pure no-op for the receiver (`compose_via_relay`
                        // would need a route *to* the peer, not *from*
                        // it) and would be the simplest possible loop —
                        // skipped, not relied on `compose_via_relay`'s
                        // own no-chaining-through-`Via` guard to catch.
                        if *destination == peer.id {
                            continue;
                        }
                        let advertisement = WireMessage::RouteAdvertisement(RouteAdvertisement {
                            destination_endpoint: *destination.as_bytes(),
                            rtt_millis: entry.rtt_millis,
                            reliability: entry.reliability,
                            advertised_at: now,
                        });
                        if let Err(e) = send_and_record(&endpoint, &transport_manager, peer.id, &advertisement).await {
                            tracing::debug!(error = %e, peer = ?peer.id, "route advertisement send failed");
                        }
                    }
                }
            }
        });
    }

    tracing::info!(
        quota_bytes = config.quota_bytes,
        seen_capacity = config.seen_capacity,
        "DTN store/dedup ready — destination-aware push via DeviceRoutes for devices that have checked in, priority-ordered flood fallback otherwise"
    );

    while let Some(frame) = rx.recv().await {
        // Set inside the `Mesh` arm below when this iteration stored a
        // bundle AND already pushed it directly to a known destination
        // endpoint (see `DeviceRoutes` below) — skipped from the
        // fallback flood step so it isn't sent twice and doesn't
        // double-consume its `replication_budget`.
        let mut already_pushed: Option<siar_domain::MessageId> = None;
        // Set when this iteration's frame came from a peer we just
        // received a *new* bundle from — kept from the fallback flood
        // for the same "don't hand it straight back to where it came
        // from" reason the naive version of this loop already had.
        let mut just_received: Option<siar_domain::MessageId> = None;

        match frame.message {
            WireMessage::V1(_envelope) => {
                // `MessageService::handle_incoming` needs a `&PeerTicket`
                // known in advance to decrypt anything — this node can't
                // do that for an arbitrary, not-pre-paired sender, which
                // is exactly the gap this file's top doc comment
                // describes. A stranger's traffic needs to arrive as a
                // `MeshEnvelope` instead (see below) — this branch is
                // for the "someone sent this relay an ordinary paired
                // message" case, which just isn't useful for a relay
                // with no conversation partner of its own yet.
                tracing::info!(from = ?frame.from, "received a V1 frame from a peer with no pre-shared ticket — dropping");
            }
            WireMessage::MailboxCheckIn(check_in) => {
                // Verified and pinned before this check-in is trusted
                // for anything — see `siar_protocol::mailbox`'s doc
                // comment for exactly what a passing `verify_and_pin`
                // does and doesn't prove. A rejected check-in gets
                // neither an answer nor a `DeviceRoutes` recording:
                // treating it as silently absent is the correct
                // response to an unverifiable identity claim, not an
                // error worth this relay's own logging budget beyond a
                // debug line.
                let verified = device_keys.lock().expect("DeviceKeyDirectory lock poisoned").verify_and_pin(
                    &check_in,
                    siar_domain::now_millis(),
                    MAILBOX_CHECKIN_MAX_AGE_MILLIS,
                );
                if let Err(e) = verified {
                    tracing::debug!(error = %e, device = ?check_in.device, from = ?frame.from, "rejected an unverifiable mailbox check-in");
                    // `continue` skips this iteration's flood-fallback
                    // step too, not just the check-in response — a
                    // deliberate, slightly stricter choice than the
                    // "flood to any contact regardless of frame
                    // validity" reasoning the flood step's own comment
                    // gives for a `V1` frame: an invalid signature is a
                    // stronger signal of a hostile or broken peer than
                    // "we simply can't decrypt this," so this relay
                    // spends no forwarding effort on that contact
                    // either.
                    continue;
                }

                // The self-disclosure moment `DeviceRoutes` exists for
                // (see that module's own doc comment) — recorded before
                // answering, so a bundle for this exact device that
                // arrives later in this same process's lifetime can be
                // pushed to it directly instead of waiting for another
                // check-in. Only reached once `verify_and_pin` above has
                // actually confirmed this device controls the key it
                // claims — `DeviceRoutes` no longer trusts a bare,
                // unauthenticated assertion the way it would have before
                // this pass.
                device_routes.lock().expect("DeviceRoutes lock poisoned").record(
                    check_in.device,
                    frame.from,
                    siar_domain::now_millis(),
                );

                // next.md §76–77's mailbox check-in — see
                // `siar-protocol::mailbox`'s doc comment for what this
                // does and doesn't authenticate. Unlike the naive
                // forward-on-contact step below, this is a *destination*
                // explicitly asking, so a match here is a real, direct
                // delivery — `mark_delivered` afterward, not gated by
                // `consume_for_forward`'s replication-budget check
                // (`MeshBundle::try_consume_replication`'s own doc
                // comment already draws this same distinction: direct
                // delivery to a known destination is separate from the
                // budget that gates opportunistic replication).
                let matching_ids: Vec<siar_domain::MessageId> = bundle_store
                    .lock()
                    .expect("BundleStore lock poisoned")
                    .iter()
                    .filter(|bundle| bundle.destination == check_in.device)
                    .map(|bundle| bundle.id)
                    .collect();

                let mut delivered_count = 0usize;
                for id in matching_ids {
                    let Some(bundle) = bundle_store.lock().expect("BundleStore lock poisoned").get(id) else {
                        continue; // evicted between the scan above and now
                    };
                    let envelope = bundle_to_envelope(bundle);
                    match send_and_record(&endpoint, &transport_manager, frame.from, &WireMessage::Mesh(envelope)).await {
                        Ok(()) => {
                            bundle_store.lock().expect("BundleStore lock poisoned").mark_delivered(id);
                            delivered_count += 1;
                        }
                        Err(e) => tracing::debug!(error = %e, id = ?id, "mailbox delivery attempt failed"),
                    }
                }
                tracing::info!(
                    device = ?check_in.device,
                    from = ?frame.from,
                    delivered_count,
                    "answered a mailbox check-in"
                );
            }
            WireMessage::TokenMailboxDeposit(envelope) => {
                // The unlinkable counterpart to the `Mesh` arm below —
                // same dedup/expiry shape, filed into `token_mailbox`
                // (keyed by the sender's chosen `MailboxToken`) instead
                // of `bundle_store` (keyed by `DeviceId`). See
                // `siar_protocol::mailbox`'s doc comments for why these
                // stay two structurally separate stores.
                let now = siar_domain::now_millis();
                let already_seen = seen.lock().expect("SeenBundles lock poisoned").check_and_record(envelope.id);
                if already_seen {
                    tracing::debug!(id = ?envelope.id, from = ?frame.from, "duplicate TokenMailboxEnvelope, dropping");
                    continue;
                }
                if envelope.is_expired(now) {
                    tracing::debug!(id = ?envelope.id, from = ?frame.from, "expired TokenMailboxEnvelope, dropping");
                    continue;
                }
                let token = envelope.destination_token;
                token_mailbox.lock().expect("TokenMailboxStore lock poisoned").deposit(token, envelope);
                tracing::info!(from = ?frame.from, "stored a TokenMailboxEnvelope for later collection");
            }
            WireMessage::AnonymousMailboxCheckIn(check_in) => {
                // Bearer-capability model (`siar_crypto::mailbox_token`'s
                // own doc comment): presenting the token *is* the
                // authorization — no signature to verify here, unlike
                // the `MailboxCheckIn` arm above. This relay hands back
                // whatever is filed under it, no questions asked, the
                // same way any bearer-token API would.
                let deposits = token_mailbox.lock().expect("TokenMailboxStore lock poisoned").collect(check_in.token);

                let mut delivered_count = 0usize;
                // Failed sends are re-deposited rather than dropped —
                // `collect` above already removed them from the store,
                // and a delivery attempt that fails shouldn't cost the
                // sender their message the way it would if this arm
                // just let a failed `envelope` fall out of scope.
                let mut redeposit = Vec::new();
                for envelope in deposits {
                    let message = WireMessage::TokenMailboxDeposit(envelope.clone());
                    match send_and_record(&endpoint, &transport_manager, frame.from, &message).await {
                        Ok(()) => delivered_count += 1,
                        Err(e) => {
                            tracing::debug!(error = %e, "anonymous mailbox delivery attempt failed, re-depositing");
                            redeposit.push(envelope);
                        }
                    }
                }
                if !redeposit.is_empty() {
                    let mut store = token_mailbox.lock().expect("TokenMailboxStore lock poisoned");
                    for envelope in redeposit {
                        store.deposit(check_in.token, envelope);
                    }
                }
                tracing::info!(from = ?frame.from, delivered_count, "answered an anonymous mailbox check-in");
            }
            WireMessage::RouteAdvertisement(advertisement) => {
                // The real routing-advertisement exchange
                // `siar_routing::path::PathTable::compose_via_relay`'s
                // own doc comment flagged as not existing anywhere in
                // this workspace — see `route_advertisement.rs`'s doc
                // comment for what this does and, importantly, does NOT
                // verify (no signature; trusting a direct transport
                // peer the same amount the existing naive-flood forward
                // already does, not a new or stronger trust boundary).
                let Ok(destination) = iroh::EndpointId::from_bytes(&advertisement.destination_endpoint) else {
                    tracing::debug!(from = ?frame.from, "route advertisement had a malformed destination endpoint, dropping");
                    continue;
                };
                let relay_advertisement = RelayAdvertisement {
                    via: frame.from,
                    destination,
                    rtt_millis: advertisement.rtt_millis,
                    reliability: advertisement.reliability,
                    last_seen: advertisement.advertised_at,
                };
                // One lock, both calls: `compose_via_relay` takes `&self`
                // and `upsert_route` takes `&mut self`, but nothing here
                // suspends between them (no `.await`), so there's no
                // reason to drop and re-acquire — same "hold across a
                // synchronous critical section, never across an await"
                // rule this file's own top block comment already states
                // for every other `Mutex` here.
                let mut table = transport_manager.path_table();
                match table.compose_via_relay(&relay_advertisement) {
                    Some(entry) => {
                        table.upsert_route(destination, entry);
                        tracing::debug!(from = ?frame.from, destination = ?destination, "composed and stored a 2-hop route from an advertisement");
                    }
                    None => {
                        // We have no *direct* route to `frame.from`
                        // ourselves (the precondition `compose_via_relay`
                        // requires) — nothing to compose yet. Not an
                        // error: `TransportManager::sync_local_peers`'s
                        // own periodic tick will supply that direct
                        // route once/if it exists, and a later
                        // advertisement will compose successfully then.
                        tracing::debug!(from = ?frame.from, destination = ?destination, "no direct route to the advertiser yet, dropping advertisement");
                    }
                }
            }
            WireMessage::Mesh(mesh_envelope) => {
                let now = siar_domain::now_millis();

                // next.md §31 dedup: a bundle that's already been seen
                // (forwarded here before, or looped back around) is
                // dropped without touching the store — `check_and_record`
                // both checks and marks in one call, same pattern
                // `siar-dtn`'s own tests exercise.
                let already_seen = seen.lock().expect("SeenBundles lock poisoned").check_and_record(mesh_envelope.id);
                if already_seen {
                    tracing::debug!(id = ?mesh_envelope.id, from = ?frame.from, "duplicate MeshEnvelope, dropping");
                    continue;
                }

                if mesh_envelope.is_expired(now) {
                    tracing::debug!(id = ?mesh_envelope.id, from = ?frame.from, "expired MeshEnvelope, dropping");
                    continue;
                }

                // `MeshEnvelope`'s own doc comment on `replication_budget`:
                // the wire format doesn't carry one (a relay's own
                // outgoing-copy policy, not the original sender's, per
                // next.md §38's "ordinary DM = 2 copies... SOS = 8
                // copies" framing) — `MessagePriority::
                // default_replication_budget` is exactly that policy,
                // already built in `siar-domain` for this.
                let destination = mesh_envelope.destination;
                let bundle = siar_dtn::bundle::MeshBundle {
                    id: mesh_envelope.id,
                    destination,
                    payload_hash: mesh_envelope.payload_hash,
                    ciphertext: mesh_envelope.ciphertext,
                    priority: mesh_envelope.priority,
                    hop_limit: mesh_envelope.hop_limit,
                    replication_budget: mesh_envelope.priority.default_replication_budget(),
                    created_at: mesh_envelope.created_at,
                    expires_at: mesh_envelope.expires_at,
                };
                just_received = Some(bundle.id);

                let evicted = bundle_store.lock().expect("BundleStore lock poisoned").insert(bundle, now);
                if evicted.is_empty() {
                    tracing::info!(id = ?mesh_envelope.id, from = ?frame.from, destination = ?destination, "stored MeshEnvelope for later carriage");
                } else {
                    tracing::info!(
                        id = ?mesh_envelope.id,
                        from = ?frame.from,
                        destination = ?destination,
                        evicted_count = evicted.len(),
                        "stored MeshEnvelope, evicting lower-priority bundles to make room"
                    );
                }

                // The real destination-aware improvement this pass adds:
                // if this device has checked in with us before, we
                // already know its endpoint — push the freshly-stored
                // bundle to it right now rather than waiting for it to
                // either check in again or happen to be the next peer
                // this relay hears from.
                let known_endpoint = device_routes.lock().expect("DeviceRoutes lock poisoned").get(destination);
                if let Some(target_endpoint) = known_endpoint {
                    if target_endpoint != frame.from {
                        // Resolved to a plain owned `Option<MeshBundle>`
                        // in its own statement first, not matched on
                        // directly — under this workspace's edition
                        // 2021, an `if let`'s scrutinee temporaries (the
                        // `MutexGuard` `.lock()` would produce) live for
                        // the whole `if let` block, which would hold
                        // this lock across the `endpoint.send(...).await`
                        // below. Matching on the already-resolved,
                        // lock-free `consumed` avoids that; the flood
                        // step later in this loop already relies on the
                        // same "resolve then match" split via `let-else`
                        // for the same reason.
                        let consumed =
                            bundle_store.lock().expect("BundleStore lock poisoned").consume_for_forward(mesh_envelope.id);
                        if let Some(bundle) = consumed {
                            let envelope = bundle_to_envelope(bundle);
                            let message = WireMessage::Mesh(envelope);
                            match send_and_record(&endpoint, &transport_manager, target_endpoint, &message).await {
                                Ok(()) => {
                                    already_pushed = Some(mesh_envelope.id);
                                    tracing::info!(
                                        id = ?mesh_envelope.id,
                                        destination = ?destination,
                                        to = ?target_endpoint,
                                        "pushed directly to destination's known endpoint (DeviceRoutes hit)"
                                    );
                                }
                                Err(e) => tracing::debug!(
                                    error = %e,
                                    id = ?mesh_envelope.id,
                                    to = ?target_endpoint,
                                    "destination-aware push failed — falling back to the flood step"
                                ),
                            }
                        }
                        // `consumed` being `None` (hop_limit/
                        // replication_budget already exhausted) just
                        // means nothing to push — falls through to the
                        // flood step below like any other bundle would.
                    }
                }
            }
        }

        // next.md §35's "peer encounter protocol." Any frame from a
        // peer — including a `V1` one this relay couldn't decrypt —
        // means that peer is reachable right now, so it's offered every
        // currently-forwardable stored bundle that wasn't already
        // handled above. This is still a flood for any bundle whose
        // destination `DeviceRoutes` doesn't know yet (next.md §39's
        // full route-scoring needs a live multi-hop path view this
        // binary doesn't have — see this file's top doc comment) — but
        // it's no longer an *unordered* one: candidates are queued
        // through `PriorityScheduler` so Emergency-tier bundles are
        // offered before Background-tier ones whenever both are
        // waiting.
        let candidates: Vec<(siar_domain::MessageId, SchedulePriority)> = bundle_store
            .lock()
            .expect("BundleStore lock poisoned")
            .iter()
            .filter(|bundle| Some(bundle.id) != just_received && Some(bundle.id) != already_pushed)
            .map(|bundle| (bundle.id, SchedulePriority::from_message_priority(bundle.priority)))
            .collect();

        // Dequeued into a plain `Vec` first, with the `PriorityScheduler`
        // lock released before any `.await` — same "resolve under the
        // lock, then act without it" split used above for `DeviceRoutes`/
        // `BundleStore`; holding a `std::sync::MutexGuard` across
        // `endpoint.send(...).await` would be a real bug, not just a
        // style nit.
        let ordered_ids: Vec<siar_domain::MessageId> = {
            let mut scheduler = scheduler.lock().expect("PriorityScheduler lock poisoned");
            for (id, priority) in candidates {
                // A full queue at this priority just means this
                // iteration offers fewer candidates to this peer than
                // it otherwise would — not a reason to fail the whole
                // receive loop over a bounded-capacity admission
                // policy doing exactly what next.md §94 asks of it.
                let _ = scheduler.enqueue(priority, id);
            }
            // Was `dequeue_next(None)` — always uncongested — until
            // this pass. Now derives the ceiling from this same
            // scheduler's own queue occupancy (next.md §93's own
            // congestion behavior, self-produced from data this loop
            // already has, no external RTT/loss measurement needed —
            // see `PriorityScheduler::congestion_ceiling`'s doc comment
            // for why that's a real, separate half of the same gap this
            // relay isn't closing here).
            let ceiling = scheduler.congestion_ceiling(CONGESTION_OCCUPANCY_THRESHOLD);
            std::iter::from_fn(|| scheduler.dequeue_next(ceiling)).collect()
        };

        for id in ordered_ids {
            let Some(bundle) = bundle_store.lock().expect("BundleStore lock poisoned").consume_for_forward(id) else {
                continue; // hop_limit or replication_budget already exhausted
            };
            let envelope = bundle_to_envelope(bundle);
            let message = WireMessage::Mesh(envelope);
            if let Err(e) = send_and_record(&endpoint, &transport_manager, frame.from, &message).await {
                tracing::debug!(error = %e, id = ?id, to = ?frame.from, "forward attempt failed");
            } else {
                tracing::debug!(id = ?id, to = ?frame.from, "forwarded a stored bundle on contact");
            }
        }
    }

    Ok(())
}
