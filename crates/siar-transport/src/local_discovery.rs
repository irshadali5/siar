//! next.md §12–14: Iroh already supports local-network discovery via
//! `iroh-mdns-address-lookup` — a separate crate iroh 1.x moved this
//! into, not a Cargo feature flag on `iroh` itself. (The widely-mirrored
//! `discovery-local-network` feature-flag docs found by a general search
//! are for iroh 0.x and don't apply to the `iroh = "1.0.3"` this
//! workspace pins — confirmed against iroh's own current docs,
//! <https://docs.iroh.computer/connecting/local-discovery>, not assumed.)
//!
//! [`LocalPeerDirectory`] tracks what mDNS has found on the LAN right
//! now — this is next.md §60's "3 nearby relay devices" UI status, and
//! also what §12 means by "no special message format required": once a
//! peer's `EndpointAddr` shows up here, `SiarEndpoint::send`/
//! `fetch_blob` already know how to reach it — same iroh direct-QUIC
//! path used for an Internet peer, just resolved from a LAN broadcast
//! instead of DNS.
//!
//! Deliberately doesn't hold or check anything about identity or trust —
//! next.md §14 is explicit that LAN presence is never itself
//! authorization. This is purely "here's an address iroh could dial,"
//! the same shape as a manually-typed `PeerTicket`; E2EE and identity
//! verification happen entirely above this crate, unchanged by whether a
//! peer's address came from mDNS or was typed in by hand.
//!
//! Also deliberately advertises nothing beyond what iroh's discovery
//! already publishes by default (the endpoint's public key and network
//! addresses) — next.md §13's "do not advertise phone number/username/
//! real name in plaintext LAN discovery" is satisfied by *omission*
//! here: nothing in this module ever calls `EndpointInfo::with_user_data`,
//! so there's no application-level identity in the broadcast to leak in
//! the first place.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use iroh::{EndpointAddr, EndpointId};
use iroh_mdns_address_lookup::{DiscoveryEvent, MdnsAddressLookup};

#[derive(Default)]
pub struct LocalPeerDirectory {
    peers: Mutex<HashMap<EndpointId, EndpointAddr>>,
}

impl LocalPeerDirectory {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// next.md §60's "N nearby relay devices" count-and-address-list —
    /// also exactly what a caller would hand to `SiarEndpoint::send` to
    /// reach one of these peers without going through DNS discovery at
    /// all.
    pub fn snapshot(&self) -> Vec<EndpointAddr> {
        self.peers
            .lock()
            .expect("LocalPeerDirectory poisoned")
            .values()
            .cloned()
            .collect()
    }

    pub fn len(&self) -> usize {
        self.peers
            .lock()
            .expect("LocalPeerDirectory poisoned")
            .len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn apply(&self, event: DiscoveryEvent) {
        let mut peers = self.peers.lock().expect("LocalPeerDirectory poisoned");
        match event {
            DiscoveryEvent::Discovered { endpoint_info, .. } => {
                peers.insert(endpoint_info.endpoint_id, endpoint_info.to_endpoint_addr());
            }
            DiscoveryEvent::Expired { endpoint_id } => {
                peers.remove(&endpoint_id);
            }
            // `DiscoveryEvent` is `#[non_exhaustive]` (confirmed against
            // its docs.rs page) — a future variant this crate doesn't
            // know about yet isn't this directory's business to guess
            // at; ignore rather than fail to compile against iroh's next
            // minor version.
            _ => {}
        }
    }
}

/// Spawns the background task draining `mdns`'s event stream into
/// `directory`. Returns nothing to hold onto deliberately: this task's
/// lifetime is tied to `mdns` and `directory` staying alive (both are
/// held by `SiarEndpoint` — `directory` via its own field, `mdns`
/// implicitly by whatever `endpoint.address_lookup()...add()` retains
/// internally), not to a handle a caller needs to manage. It exits on
/// its own once the event stream ends.
pub(crate) fn spawn_local_discovery_task(
    mdns: MdnsAddressLookup,
    directory: Arc<LocalPeerDirectory>,
) {
    tokio::spawn(async move {
        let mut events = mdns.subscribe().await;
        while let Some(event) = n0_future::StreamExt::next(&mut events).await {
            directory.apply(event);
        }
    });
}
