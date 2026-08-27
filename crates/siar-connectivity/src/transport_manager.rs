//! next.md §90's `TransportManager` — the piece `siar-routing`'s own
//! doc comment flagged as needing "live transport sessions this crate
//! deliberately has zero dependency on." `siar-connectivity` already
//! bridges `SiarEndpoint` into higher-level state
//! ([`crate::ConnectivityMonitor`]); this does the same for
//! `siar-routing`'s `PathTable`.
//!
//! [`TransportManager::sync_local_peers`] is the fix for the identity
//! mismatch that blocked this: `PathTable` now keys on `EndpointId`
//! (see `siar_routing::path`'s doc comment for the full story), which
//! is exactly what [`siar_transport::SiarEndpoint::local_peers`]
//! already gives out via each [`iroh::EndpointAddr`]'s `id` field — no
//! more guessing needed.
//!
//! `rtt_millis`/`reliability` for a freshly-synced LAN peer are best
//! guesses (`None`/`1.0`), not measurements — this crate doesn't
//! measure either yet. [`TransportManager::record_send_outcome`]
//! (added this pass, closing the wiring half of `siar-routing::
//! link_health`'s doc comment) is where a real measurement would land
//! once something supplies one: nothing in this workspace currently
//! times a real `SiarEndpoint::send` attempt and calls it — that's
//! `siar-messaging`'s send path, still separate follow-up work — but
//! the method itself is real, tested logic, not a stub.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

use siar_domain::TransportLink;
use siar_routing::link_health::{LinkHealth, SendOutcome};
use siar_routing::path::{NextHop, PathEntry, PathTable};
use siar_transport::SiarEndpoint;

/// Default rolling-window size for each `(EndpointId, TransportLink)`'s
/// [`LinkHealth`] — small enough that a link's reliability reading
/// reacts to a recent run of failures within a handful of attempts,
/// large enough that one flaky send doesn't swing the reading on its
/// own. Not tuned against real usage data (no real caller exists yet to
/// generate any) — a starting point, same status next.md §90's other
/// "chosen conservatively, not measured" constants carry throughout
/// this workspace.
const LINK_HEALTH_WINDOW: usize = 20;

pub struct TransportManager {
    endpoint: Arc<SiarEndpoint>,
    path_table: Mutex<PathTable>,
    /// Keyed by `(destination, link)` rather than nested inside
    /// `PathTable` itself — `PathTable`'s own `PathEntry` only ever
    /// holds the *current* `rtt_millis`/`reliability` snapshot next.md
    /// §91 wants exposed; the rolling history behind that snapshot is
    /// this type's own concern, same separation `path.rs`'s doc comment
    /// draws between "what a caller needs to make a routing decision"
    /// and "how that number was computed."
    link_health: Mutex<HashMap<(iroh::EndpointId, TransportLink), LinkHealth>>,
}

impl TransportManager {
    pub fn new(endpoint: Arc<SiarEndpoint>) -> Self {
        Self {
            endpoint,
            path_table: Mutex::new(PathTable::new()),
            link_health: Mutex::new(HashMap::new()),
        }
    }

    /// Refreshes `LocalLan` candidates in the path table from
    /// `SiarEndpoint::local_peers()` — the piece that was missing.
    /// Cheap enough to call on a timer, same "caller supplies the real-
    /// world timing" split [`crate::ConnectivityMonitor::snapshot`]
    /// already uses; `now` is this call's own opaque tick (next.md
    /// §96), stamped onto every entry it touches so a later
    /// `PathTable::remove_stale` call can tell fresh entries from ones
    /// that stopped being refreshed (their LAN peer walked out of mDNS
    /// range) without this method needing to know anything about real
    /// time itself.
    pub fn sync_local_peers(&self, now: u64) {
        let mut table = self.path_table.lock().expect("PathTable lock poisoned");
        for addr in self.endpoint.local_peers() {
            table.upsert_route(
                addr.id,
                PathEntry {
                    link: TransportLink::LocalLan,
                    next_hop: NextHop::Direct,
                    last_seen: now,
                    rtt_millis: None,
                    // 1.0 (not measured, not 0.0/"unknown") on purpose:
                    // this is "iroh's own mDNS just told us this peer
                    // is on the LAN right now" — a reasonable prior for
                    // a link that's currently confirmed up, not a
                    // placeholder that should read as "no data."
                    reliability: 1.0,
                },
            );
        }
    }

    pub fn path_table(&self) -> MutexGuard<'_, PathTable> {
        self.path_table.lock().expect("PathTable lock poisoned")
    }

    /// Folds one real send attempt into this link's rolling
    /// [`LinkHealth`] window, then re-upserts `destination`'s
    /// `PathEntry` for `link` with the recomputed reliability/RTT —
    /// closing next.md §90's "actually measuring `PathEntry.
    /// rtt_millis`/`reliability`" gap on the computation side. `now`/
    /// `next_hop` are caller-supplied for the same reasons every other
    /// timestamped/route-shaped value in this workspace is (next.md
    /// §96; `PathTable::upsert_route`'s own signature) — this method
    /// has no clock and no opinion on whether the send went direct or
    /// via a relay.
    ///
    /// No real caller exists yet (see this file's top doc comment) —
    /// this is the landing point for one, not evidence one is wired in.
    pub fn record_send_outcome(
        &self,
        destination: iroh::EndpointId,
        link: TransportLink,
        next_hop: NextHop,
        now: u64,
        outcome: SendOutcome,
    ) {
        let (reliability, rtt_millis) = {
            let mut health = self
                .link_health
                .lock()
                .expect("LinkHealth map lock poisoned");
            let entry = health
                .entry((destination, link))
                .or_insert_with(|| LinkHealth::new(LINK_HEALTH_WINDOW));
            entry.record_outcome(outcome);
            (entry.reliability(), entry.average_rtt_millis())
        };
        self.path_table
            .lock()
            .expect("PathTable lock poisoned")
            .upsert_route(
                destination,
                PathEntry {
                    link,
                    next_hop,
                    last_seen: now,
                    rtt_millis,
                    reliability,
                },
            );
    }
}
