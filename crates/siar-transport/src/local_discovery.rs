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

#[cfg(test)]
mod tests {
    //! `LocalPeerDirectory::apply` is where all of this module's real
    //! logic lives (`snapshot`/`len`/`is_empty` are trivial reads) — it's
    //! never been exercised by a test before. `apply` is private, but
    //! this `mod tests` is a child of the module that defines it, so it
    //! can be called directly without spinning up any real mDNS traffic
    //! or an iroh endpoint: `DiscoveryEvent`/`EndpointInfo` are plain,
    //! network-free data to construct.

    use super::*;
    use iroh::address_lookup::EndpointInfo;
    use iroh::SecretKey;
    use std::net::{Ipv4Addr, SocketAddr};

    fn fresh_endpoint_id() -> EndpointId {
        SecretKey::generate().public()
    }

    fn addr(port: u16) -> SocketAddr {
        SocketAddr::from((Ipv4Addr::LOCALHOST, port))
    }

    #[test]
    fn fresh_directory_is_empty() {
        let dir = LocalPeerDirectory::new();
        assert!(dir.is_empty());
        assert_eq!(dir.len(), 0);
        assert!(dir.snapshot().is_empty());
    }

    #[test]
    fn discovered_event_adds_the_peer() {
        let dir = LocalPeerDirectory::new();
        let id = fresh_endpoint_id();
        let info = EndpointInfo::new(id).with_ip_addrs(vec![addr(4433)]);

        dir.apply(DiscoveryEvent::Discovered {
            endpoint_info: info,
            last_updated: None,
        });

        assert!(!dir.is_empty());
        assert_eq!(dir.len(), 1);
        let snapshot = dir.snapshot();
        assert_eq!(snapshot.len(), 1);
        assert_eq!(snapshot[0].id, id);
    }

    #[test]
    fn expired_event_removes_a_known_peer() {
        let dir = LocalPeerDirectory::new();
        let id = fresh_endpoint_id();
        let info = EndpointInfo::new(id).with_ip_addrs(vec![addr(4433)]);
        dir.apply(DiscoveryEvent::Discovered {
            endpoint_info: info,
            last_updated: None,
        });
        assert_eq!(dir.len(), 1);

        dir.apply(DiscoveryEvent::Expired { endpoint_id: id });

        assert!(dir.is_empty());
        assert!(dir.snapshot().is_empty());
    }

    #[test]
    fn expiring_an_unknown_peer_is_a_no_op() {
        let dir = LocalPeerDirectory::new();
        let unknown = fresh_endpoint_id();

        dir.apply(DiscoveryEvent::Expired {
            endpoint_id: unknown,
        });

        assert!(dir.is_empty());
    }

    #[test]
    fn rediscovering_the_same_id_updates_rather_than_duplicates() {
        let dir = LocalPeerDirectory::new();
        let id = fresh_endpoint_id();

        dir.apply(DiscoveryEvent::Discovered {
            endpoint_info: EndpointInfo::new(id).with_ip_addrs(vec![addr(4433)]),
            last_updated: None,
        });
        dir.apply(DiscoveryEvent::Discovered {
            endpoint_info: EndpointInfo::new(id).with_ip_addrs(vec![addr(5544)]),
            last_updated: None,
        });

        // Same id seen twice must still be one entry, not two — this is
        // exactly what `next.md §60`'s "N nearby relay devices" count
        // depends on being accurate.
        assert_eq!(dir.len(), 1);
        let snapshot = dir.snapshot();
        assert_eq!(snapshot.len(), 1);
        assert!(snapshot[0]
            .addrs
            .iter()
            .any(|a| matches!(a, iroh::TransportAddr::Ip(sock) if sock.port() == 5544)));
    }

    #[test]
    fn discovering_two_different_peers_keeps_both() {
        let dir = LocalPeerDirectory::new();
        let a = fresh_endpoint_id();
        let b = fresh_endpoint_id();

        dir.apply(DiscoveryEvent::Discovered {
            endpoint_info: EndpointInfo::new(a).with_ip_addrs(vec![addr(4433)]),
            last_updated: None,
        });
        dir.apply(DiscoveryEvent::Discovered {
            endpoint_info: EndpointInfo::new(b).with_ip_addrs(vec![addr(4434)]),
            last_updated: None,
        });

        assert_eq!(dir.len(), 2);
        let ids: std::collections::HashSet<_> = dir.snapshot().into_iter().map(|p| p.id).collect();
        assert!(ids.contains(&a));
        assert!(ids.contains(&b));
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
