//! Transport capabilities and the per-destination path table — next.md
//! §7, §91.
//!
//! [`capabilities_for`] is a static baseline per [`TransportLink`] kind
//! — next.md §7's own examples ("Iroh/direct: high bandwidth, low
//! latency, files=yes, video=yes... BLE: very low bandwidth, higher
//! latency, files=restricted, video=no"). Deliberately doesn't include
//! `metered`/`power_cost` from the doc's `TransportCapabilities`
//! sketch: those are properties of one specific active connection (is
//! *this* Internet link on cellular data right now?), not of the
//! transport kind in the abstract, so they belong on a live
//! [`PathEntry`] once something is actually measuring them — a later
//! pass, not this static table.
//!
//! **Correction from the first version of this file**: [`PathTable`]
//! now keys on `iroh::EndpointId`, not `siar_domain::DeviceId`.
//! Building the actual `TransportManager` on top of this ran into a
//! real identity mismatch in this workspace: `SiarEndpoint::local_peers`
//! (Phase 1) and every discovery mechanism built since only ever
//! produce an iroh `EndpointId`/`EndpointAddr` — nothing anywhere maps
//! one of those to a `DeviceId`, including in the pre-`next.md`
//! codebase (`PeerTicket` doesn't carry a `DeviceId` either). Keying by
//! `DeviceId` would have meant this table could never actually be
//! populated from anything Phases 1–3 built. `EndpointId` is also
//! arguably the more architecturally honest key for a *network* routing
//! table in the first place — next.md §91 is about "how do I reach this
//! endpoint," which is what `EndpointId` represents; `DeviceId` reads
//! more like an application/contact-identity concept. If a caller
//! genuinely needs a `DeviceId`-keyed view (e.g. "which of my known
//! contacts are currently reachable"), that's a join against whatever
//! maps `DeviceId -> EndpointId` for paired contacts — this table
//! doesn't attempt that mapping itself.

use iroh::EndpointId;
use siar_domain::TransportLink;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BandwidthClass {
    VeryLow,
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LatencyClass {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransportCapabilities {
    pub bandwidth: BandwidthClass,
    pub latency: LatencyClass,
    pub supports_streaming: bool,
    pub supports_large_files: bool,
    /// next.md §57: BLE mesh never; Bluetooth Classic "audio perhaps,
    /// not preferred" collapses to `false` here too — a boolean can't
    /// represent "usable but not preferred," and treating it as
    /// unsupported is the conservative reading of the doc's own
    /// hedging on that one.
    pub supports_realtime_media: bool,
}

/// Classifies a peer's *advertised* addressing info into the closest
/// [`TransportLink`] this crate can infer without a live connection to
/// measure — closing (partially) the "`LocalLan`/`InternetRelay`
/// unreported, everything classified `InternetDirect`" gap flagged
/// across several earlier passes (`apps/emergency-node`'s
/// `send_and_record`, `apps/android/messaging-jni`'s `bootstrap`).
///
/// Real, evidence-based heuristic — not a guess: an [`iroh::EndpointAddr`]
/// advertising any private/link-local/loopback direct IP (RFC 1918,
/// link-local, or IPv6 unique-local `fc00::/7`, checked via
/// `Ipv4Addr::is_private`/`is_link_local`/a manual IPv6 range check)
/// classifies as [`TransportLink::LocalLan`]; one advertising any
/// public direct IP classifies as [`TransportLink::InternetDirect`];
/// one with only a relay URL and no direct IPs at all classifies as
/// [`TransportLink::InternetRelay`].
///
/// **What this is NOT**: the connection's actually-measured path.
/// iroh's `Endpoint::conn_type` (which reported the real, currently-in-
/// use path) was removed in iroh 0.96 in favor of a per-path,
/// multipath-aware `Connection::paths()` stream — a real, separate
/// integration this function doesn't attempt (it would need a live
/// `Connection` handle, not just the `EndpointAddr` this function
/// takes). A peer that advertises a LAN IP but was actually reached
/// over relay (NAT traversal failed) still classifies as `LocalLan`
/// here. Better than every call site in this workspace previously
/// defaulting to `InternetDirect` unconditionally, not a full
/// measurement-based fix.
pub fn classify_endpoint_addr(addr: &iroh::EndpointAddr) -> TransportLink {
    let mut saw_any_ip = false;
    for socket_addr in addr.ip_addrs() {
        saw_any_ip = true;
        if is_local_or_private_ip(socket_addr) {
            return TransportLink::LocalLan;
        }
    }
    if saw_any_ip {
        TransportLink::InternetDirect
    } else {
        TransportLink::InternetRelay
    }
}

fn is_local_or_private_ip(socket_addr: &std::net::SocketAddr) -> bool {
    match socket_addr.ip() {
        std::net::IpAddr::V4(v4) => v4.is_private() || v4.is_link_local() || v4.is_loopback(),
        std::net::IpAddr::V6(v6) => {
            // fc00::/7 (unique local) checked by bit pattern rather
            // than `Ipv6Addr::is_unique_local()` — that method's
            // stabilization history wasn't something this pass could
            // confirm against this workspace's `rust-version = "1.91"`
            // floor without a real compile, so this uses the
            // unambiguous, always-stable bit check instead: the top 7
            // bits of a unique-local address are `1111110`.
            v6.is_loopback() || (v6.segments()[0] & 0xfe00) == 0xfc00
        }
    }
}

pub fn capabilities_for(link: TransportLink) -> TransportCapabilities {
    use TransportLink::*;
    match link {
        InternetDirect | InternetRelay | LocalLan => TransportCapabilities {
            bandwidth: BandwidthClass::High,
            latency: LatencyClass::Low,
            supports_streaming: true,
            supports_large_files: true,
            supports_realtime_media: true,
        },
        WifiDirect | WifiAware => TransportCapabilities {
            bandwidth: BandwidthClass::High,
            latency: LatencyClass::Low,
            supports_streaming: true,
            supports_large_files: true,
            supports_realtime_media: true,
        },
        BluetoothClassic => TransportCapabilities {
            bandwidth: BandwidthClass::Medium,
            latency: LatencyClass::Medium,
            supports_streaming: false,
            supports_large_files: false,
            supports_realtime_media: false,
        },
        Ble => TransportCapabilities {
            bandwidth: BandwidthClass::VeryLow,
            latency: LatencyClass::High,
            supports_streaming: false,
            supports_large_files: false,
            supports_realtime_media: false,
        },
    }
}

/// next.md §91's "David → Bob/BLE" example: a destination isn't always
/// reachable directly — it might only be reachable *through* another
/// endpoint acting as a relay.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NextHop {
    Direct,
    Via(EndpointId),
}

/// One hop `via` has advertised it can reach `destination` over —
/// the caller-supplied signal a real routing-advertisement exchange
/// would produce. This type doesn't define or send that exchange
/// itself (see [`PathTable::compose_via_relay`]'s doc comment for why
/// building the real wire protocol is separate, later work): it's the
/// shape composition needs, so that later work doesn't also have to
/// invent this shape from scratch.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RelayAdvertisement {
    /// The neighbor making this claim — must be something this table
    /// already has a *direct* route to, or [`PathTable::compose_via_relay`]
    /// has nothing to compose it with.
    pub via: EndpointId,
    pub destination: EndpointId,
    /// The relay's own estimate of its second hop, in the same units as
    /// [`PathEntry::rtt_millis`]/[`PathEntry::reliability`] — what the
    /// relay would report about *its* path to `destination`, not
    /// anything this device measured itself.
    pub rtt_millis: Option<u32>,
    pub reliability: f32,
    pub last_seen: u64,
}

/// One candidate route to one destination, over one [`TransportLink`].
/// `last_seen`/timestamps throughout this crate are opaque `u64` ticks
/// in the caller's own monotonic units, same reasoning (and same
/// "caller supplies `now`" pattern) as `siar_dtn::bundle::MeshBundle`
/// and next.md §96.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PathEntry {
    pub link: TransportLink,
    pub next_hop: NextHop,
    pub last_seen: u64,
    pub rtt_millis: Option<u32>,
    /// Estimated delivery success rate on this path, `0.0..=1.0`. Where
    /// this number comes from (an actual measured history vs. a
    /// starting guess) is a later pass's concern; this type just holds
    /// whatever the caller has.
    pub reliability: f32,
}

/// Per-destination candidate routes, keyed by `EndpointId` — see this
/// file's top doc comment for why. One [`PathTable`] is meant to cover
/// every endpoint this device currently knows any path to — next.md
/// §92 is explicit this should NOT try to be an Internet-style global
/// route table ("mobile topology changes too quickly"), so there's no
/// attempt here at full transitive route computation across every
/// destination this table knows about. [`PathTable::compose_via_relay`]
/// (this pass) is the one deliberately bounded exception: composing
/// exactly *one* additional relay hop from a caller-supplied
/// [`RelayAdvertisement`], not walking an unbounded chain across the
/// whole table on its own — see that method's doc comment for the
/// reasoning and for what it still doesn't do.
pub struct PathTable {
    routes: std::collections::HashMap<EndpointId, Vec<PathEntry>>,
}

impl PathTable {
    pub fn new() -> Self {
        Self { routes: std::collections::HashMap::new() }
    }

    /// Records or refreshes one candidate route. Replaces an existing
    /// entry for the same `(link, next_hop)` pair for this destination
    /// rather than accumulating duplicates — re-observing the same path
    /// updates its `last_seen`/`rtt_millis`/`reliability`, it doesn't
    /// create a second candidate.
    pub fn upsert_route(&mut self, destination: EndpointId, entry: PathEntry) {
        let entries = self.routes.entry(destination).or_default();
        if let Some(existing) = entries.iter_mut().find(|existing| existing.link == entry.link && existing.next_hop == entry.next_hop) {
            *existing = entry;
        } else {
            entries.push(entry);
        }
    }

    pub fn routes_for(&self, destination: EndpointId) -> &[PathEntry] {
        self.routes.get(&destination).map(Vec::as_slice).unwrap_or(&[])
    }

    /// The single best current candidate route to `destination`, by
    /// [`TransportLink::preference_rank`] (ties broken by higher
    /// `reliability`, then lower `rtt_millis`) — added this pass for
    /// next.md §90's "BLE→Wi-Fi upgrade" decision. `None` if this table
    /// has no route to `destination` at all.
    pub fn best_route_for(&self, destination: EndpointId) -> Option<&PathEntry> {
        self.routes_for(destination).iter().min_by(|a, b| {
            a.link
                .preference_rank()
                .cmp(&b.link.preference_rank())
                .then_with(|| b.reliability.total_cmp(&a.reliability))
                .then_with(|| a.rtt_millis.unwrap_or(u32::MAX).cmp(&b.rtt_millis.unwrap_or(u32::MAX)))
        })
    }

    /// Answers next.md §90's "should we switch this peer from BLE to
    /// Wi-Fi" question — the *decision* half of a BLE→Wi-Fi upgrade,
    /// not the execution half. Returns the better link to switch to if
    /// `current_link` isn't already this table's best known route to
    /// `destination`; `None` if `current_link` is already best, or if
    /// there's no route at all.
    ///
    /// What this does NOT do, and what "BLE→Wi-Fi upgrade" fully means
    /// in next.md §90: actually establish the new connection or tear
    /// down the old one. That's real, OS-level radio control —
    /// `crates/siar-transport-wifi-direct`/`-wifi-aware`/`-ble`/
    /// `-bluetooth-classic` are all JNI bridges with no Rust-side
    /// executor consuming them, because `apps/android` (the one binary
    /// that would own real Android radios and actually call these
    /// bridges) doesn't exist in this workspace at all. This function
    /// is the pure-logic piece any future consumer of those bridges —
    /// `apps/android` or otherwise — will need, built now so that work
    /// doesn't also have to invent the ranking decision from scratch
    /// later. It's genuinely usable today by anything that already has
    /// a `PathTable` (this workspace's own `TransportManager`), even
    /// though nothing currently acts on its answer.
    pub fn recommend_upgrade(&self, destination: EndpointId, current_link: TransportLink) -> Option<TransportLink> {
        let best = self.best_route_for(destination)?;
        if best.link.preference_rank() < current_link.preference_rank() {
            Some(best.link)
        } else {
            None
        }
    }

    /// Every destination this table currently has at least one
    /// candidate route for — `TransportManager::sync_local_peers` uses
    /// this to decide which entries came from a source (like mDNS)
    /// that's expected to keep refreshing them, versus which have gone
    /// silent.
    pub fn destinations(&self) -> impl Iterator<Item = EndpointId> + '_ {
        self.routes.keys().copied()
    }

    /// Drops every route entry older than `max_age` ticks as of `now`,
    /// across every destination — next.md §92's "mobile topology
    /// changes too quickly" means a stale route is worse than no route
    /// (it looks available when it probably isn't anymore).
    pub fn remove_stale(&mut self, now: u64, max_age: u64) {
        self.routes.retain(|_, entries| {
            entries.retain(|entry| now.saturating_sub(entry.last_seen) <= max_age);
            !entries.is_empty()
        });
    }

    /// Derives a 2-hop candidate route to `advertisement.destination`
    /// by composing this table's own best *direct* route to
    /// `advertisement.via` with the second hop the relay itself
    /// advertised — next.md §91's own "David → Bob/BLE" example,
    /// generalized: this device reaches the relay directly, and the
    /// relay claims it can reach the final destination.
    ///
    /// Deliberately stops at one relay, not a fully general multi-hop
    /// search across this table — next.md §92's own line against this
    /// becoming "an Internet-style global route table" on a topology
    /// that "changes too quickly" applies just as much to a from-
    /// scratch Dijkstra/Bellman-Ford pass as it does to caching stale
    /// entries. A caller wanting a 3rd hop calls this again, upserting
    /// the resulting `PathEntry` first so the next `compose_via_relay`
    /// call finds it as its own new direct route to compose through —
    /// one deliberate, caller-driven step at a time, never an unbounded
    /// chain this function walks on its own.
    ///
    /// Returns `None` if this table has no *direct* route to
    /// `advertisement.via` at all — composing through an endpoint this
    /// device can't itself currently reach would be a route to nowhere,
    /// and composing through an *already-composed* (`Via`) entry is
    /// exactly the unbounded-chaining this function declines to do on
    /// its own (see above).
    ///
    /// The returned entry is a candidate only — same as every other
    /// `PathEntry` this crate hands back — nothing is upserted into
    /// this table automatically. A caller that wants to keep it calls
    /// `upsert_route` itself, the same as with any other entry it's
    /// separately validated.
    pub fn compose_via_relay(&self, advertisement: &RelayAdvertisement) -> Option<PathEntry> {
        let first_hop = self
            .routes_for(advertisement.via)
            .iter()
            .filter(|entry| entry.next_hop == NextHop::Direct)
            .min_by(|a, b| {
                a.link
                    .preference_rank()
                    .cmp(&b.link.preference_rank())
                    .then_with(|| b.reliability.total_cmp(&a.reliability))
            })?;

        Some(PathEntry {
            // This device's own outbound link is still whatever reaches
            // `via` — the relay's own second-hop link (BLE, Wi-Fi,
            // whatever `advertisement` was built over) is the relay's
            // problem to actually use, not something this device sends
            // over itself.
            link: first_hop.link,
            next_hop: NextHop::Via(advertisement.via),
            // Only as fresh as the staler of the two pieces composed —
            // this device's own observation of `via`, or the relay's
            // advertisement of its second hop.
            last_seen: first_hop.last_seen.min(advertisement.last_seen),
            // Two independent hops both need to succeed, so their
            // probabilities multiply — the standard reading of chained
            // reliability, and the honest one: composing never produces
            // a route *more* reliable than either hop alone.
            reliability: first_hop.reliability * advertisement.reliability,
            // Sums when both legs report a number; `None` (not a
            // fabricated partial sum) the moment either leg doesn't —
            // an RTT estimate missing one of its two components isn't
            // a real estimate.
            rtt_millis: match (first_hop.rtt_millis, advertisement.rtt_millis) {
                (Some(a), Some(b)) => Some(a.saturating_add(b)),
                _ => None,
            },
        })
    }
}

impl Default for PathTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_endpoint_id(seed: u8) -> EndpointId {
        // `EndpointId` is iroh's `PublicKey` (confirmed via
        // docs.rs/iroh/latest/iroh/type.EndpointId.html: "This is
        // equivalent to PublicKey"). Built from a hex-encoded
        // `SecretKey` via the confirmed `SecretKey::from_str` API
        // (github.com/n0-computer/iroh discussion #3900's own example
        // uses exactly this) rather than guessing at a `from_bytes`
        // method name that wasn't independently confirmed — these tests
        // only need distinct, stable identifiers to key the table by,
        // not valid signing keys from a real random source.
        use std::str::FromStr;
        let hex = format!("{seed:02x}").repeat(32);
        let secret = iroh::SecretKey::from_str(&hex).expect("valid 64-char hex test secret key");
        secret.public()
    }

    fn entry(link: TransportLink, last_seen: u64) -> PathEntry {
        PathEntry { link, next_hop: NextHop::Direct, last_seen, rtt_millis: Some(50), reliability: 0.9 }
    }

    #[test]
    fn ble_only_supports_tiny_realtime_and_large_files_never() {
        let caps = capabilities_for(TransportLink::Ble);
        assert!(!caps.supports_large_files);
        assert!(!caps.supports_realtime_media);
    }

    #[test]
    fn upsert_route_replaces_same_link_and_hop_rather_than_duplicating() {
        let mut table = PathTable::new();
        let destination = test_endpoint_id(1);
        table.upsert_route(destination, entry(TransportLink::LocalLan, 10));
        table.upsert_route(destination, entry(TransportLink::LocalLan, 20));
        assert_eq!(table.routes_for(destination).len(), 1);
        assert_eq!(table.routes_for(destination)[0].last_seen, 20);
    }

    #[test]
    fn upsert_route_keeps_distinct_links_as_separate_candidates() {
        let mut table = PathTable::new();
        let destination = test_endpoint_id(2);
        table.upsert_route(destination, entry(TransportLink::LocalLan, 10));
        table.upsert_route(destination, entry(TransportLink::Ble, 10));
        assert_eq!(table.routes_for(destination).len(), 2);
    }

    #[test]
    fn remove_stale_drops_old_entries_and_empty_destinations() {
        let mut table = PathTable::new();
        let destination = test_endpoint_id(3);
        table.upsert_route(destination, entry(TransportLink::LocalLan, 0));
        table.remove_stale(100, 50);
        assert!(table.routes_for(destination).is_empty());
    }

    #[test]
    fn remove_stale_keeps_recent_entries() {
        let mut table = PathTable::new();
        let destination = test_endpoint_id(4);
        table.upsert_route(destination, entry(TransportLink::LocalLan, 90));
        table.remove_stale(100, 50);
        assert_eq!(table.routes_for(destination).len(), 1);
    }

    #[test]
    fn destinations_lists_every_known_endpoint() {
        let mut table = PathTable::new();
        table.upsert_route(test_endpoint_id(5), entry(TransportLink::LocalLan, 0));
        table.upsert_route(test_endpoint_id(6), entry(TransportLink::Ble, 0));
        let mut ids: Vec<_> = table.destinations().collect();
        ids.sort();
        let mut expected = vec![test_endpoint_id(5), test_endpoint_id(6)];
        expected.sort();
        assert_eq!(ids, expected);
    }

    #[test]
    fn best_route_for_picks_the_highest_preference_link() {
        let mut table = PathTable::new();
        let destination = test_endpoint_id(7);
        table.upsert_route(destination, entry(TransportLink::Ble, 0));
        table.upsert_route(destination, entry(TransportLink::LocalLan, 0));
        table.upsert_route(destination, entry(TransportLink::WifiDirect, 0));
        assert_eq!(table.best_route_for(destination).unwrap().link, TransportLink::LocalLan);
    }

    #[test]
    fn best_route_for_breaks_ties_by_higher_reliability_then_lower_rtt() {
        let mut table = PathTable::new();
        let destination = test_endpoint_id(8);
        table.upsert_route(
            destination,
            PathEntry { link: TransportLink::Ble, next_hop: NextHop::Direct, last_seen: 0, rtt_millis: Some(200), reliability: 0.5 },
        );
        table.upsert_route(
            destination,
            PathEntry {
                link: TransportLink::Ble,
                next_hop: NextHop::Via(test_endpoint_id(99)),
                last_seen: 0,
                rtt_millis: Some(50),
                reliability: 0.9,
            },
        );
        let best = table.best_route_for(destination).unwrap();
        assert_eq!(best.reliability, 0.9);
        assert_eq!(best.rtt_millis, Some(50));
    }

    #[test]
    fn best_route_for_is_none_when_the_destination_is_unknown() {
        let table = PathTable::new();
        assert!(table.best_route_for(test_endpoint_id(9)).is_none());
    }

    #[test]
    fn recommend_upgrade_suggests_a_strictly_better_link() {
        let mut table = PathTable::new();
        let destination = test_endpoint_id(10);
        table.upsert_route(destination, entry(TransportLink::Ble, 0));
        table.upsert_route(destination, entry(TransportLink::WifiDirect, 0));
        assert_eq!(table.recommend_upgrade(destination, TransportLink::Ble), Some(TransportLink::WifiDirect));
    }

    #[test]
    fn recommend_upgrade_is_none_when_current_link_is_already_best() {
        let mut table = PathTable::new();
        let destination = test_endpoint_id(11);
        table.upsert_route(destination, entry(TransportLink::Ble, 0));
        table.upsert_route(destination, entry(TransportLink::WifiDirect, 0));
        assert_eq!(table.recommend_upgrade(destination, TransportLink::WifiDirect), None);
    }

    #[test]
    fn recommend_upgrade_is_none_with_no_known_route() {
        let table = PathTable::new();
        assert_eq!(table.recommend_upgrade(test_endpoint_id(12), TransportLink::Ble), None);
    }

    #[test]
    fn compose_via_relay_sums_rtt_and_multiplies_reliability_across_both_hops() {
        let mut table = PathTable::new();
        let relay = test_endpoint_id(20);
        let destination = test_endpoint_id(21);
        table.upsert_route(
            relay,
            PathEntry { link: TransportLink::LocalLan, next_hop: NextHop::Direct, last_seen: 100, rtt_millis: Some(30), reliability: 0.9 },
        );
        let advertisement =
            RelayAdvertisement { via: relay, destination, rtt_millis: Some(70), reliability: 0.8, last_seen: 90 };

        let composed = table.compose_via_relay(&advertisement).expect("relay has a direct route");
        assert_eq!(composed.link, TransportLink::LocalLan);
        assert_eq!(composed.next_hop, NextHop::Via(relay));
        assert_eq!(composed.rtt_millis, Some(100));
        assert!((composed.reliability - 0.72).abs() < 1e-6);
        // Freshness is bounded by the staler of the two pieces (90),
        // not the fresher one (100).
        assert_eq!(composed.last_seen, 90);
    }

    #[test]
    fn compose_via_relay_is_none_with_no_direct_route_to_the_relay() {
        let table = PathTable::new();
        let relay = test_endpoint_id(22);
        let advertisement = RelayAdvertisement {
            via: relay,
            destination: test_endpoint_id(23),
            rtt_millis: Some(10),
            reliability: 1.0,
            last_seen: 0,
        };
        assert!(table.compose_via_relay(&advertisement).is_none());
    }

    #[test]
    fn compose_via_relay_will_not_chain_through_an_already_composed_route() {
        // The relay itself is only reachable via *another* relay
        // (`NextHop::Via`, not `Direct`) — composing through it would
        // be exactly the unbounded chaining `compose_via_relay`
        // declines to do on its own.
        let mut table = PathTable::new();
        let relay = test_endpoint_id(24);
        let far_relay = test_endpoint_id(25);
        table.upsert_route(
            relay,
            PathEntry { link: TransportLink::Ble, next_hop: NextHop::Via(far_relay), last_seen: 0, rtt_millis: Some(50), reliability: 0.9 },
        );
        let advertisement = RelayAdvertisement {
            via: relay,
            destination: test_endpoint_id(26),
            rtt_millis: Some(10),
            reliability: 1.0,
            last_seen: 0,
        };
        assert!(table.compose_via_relay(&advertisement).is_none());
    }

    #[test]
    fn compose_via_relay_picks_the_best_direct_route_to_the_relay_when_several_exist() {
        let mut table = PathTable::new();
        let relay = test_endpoint_id(27);
        table.upsert_route(
            relay,
            PathEntry { link: TransportLink::Ble, next_hop: NextHop::Direct, last_seen: 0, rtt_millis: Some(500), reliability: 0.5 },
        );
        table.upsert_route(
            relay,
            PathEntry { link: TransportLink::LocalLan, next_hop: NextHop::Direct, last_seen: 0, rtt_millis: Some(20), reliability: 0.99 },
        );
        let advertisement = RelayAdvertisement {
            via: relay,
            destination: test_endpoint_id(28),
            rtt_millis: Some(10),
            reliability: 1.0,
            last_seen: 0,
        };
        let composed = table.compose_via_relay(&advertisement).expect("relay has a direct route");
        assert_eq!(composed.link, TransportLink::LocalLan);
    }

    #[test]
    fn compose_via_relay_does_not_mutate_the_table() {
        let mut table = PathTable::new();
        let relay = test_endpoint_id(29);
        let destination = test_endpoint_id(30);
        table.upsert_route(relay, entry(TransportLink::LocalLan, 0));
        let advertisement =
            RelayAdvertisement { via: relay, destination, rtt_millis: Some(10), reliability: 1.0, last_seen: 0 };
        table.compose_via_relay(&advertisement);
        assert!(table.routes_for(destination).is_empty());
    }

    // `classify_endpoint_addr`'s own tests — construction of a synthetic
    // `iroh::EndpointAddr` with specific `addrs` here is written against
    // the field/method shape iroh's own 0.94.0 changelog documents
    // (`EndpointAddr { id, addrs: BTreeSet<TransportAddr> }`,
    // `TransportAddr::{Relay(RelayUrl), Ip(SocketAddr)}`), not verified
    // by a real compile of this workspace's pinned iroh 1.0.3 — same
    // "found via changelog/docs.rs, not compiled" caveat every other
    // iroh-touching test in this file already carries.

    fn addr_with_ips(id: EndpointId, ips: &[std::net::SocketAddr]) -> iroh::EndpointAddr {
        let addrs = ips.iter().map(|&ip| iroh::TransportAddr::Ip(ip)).collect();
        iroh::EndpointAddr { id, addrs }
    }

    fn addr_with_relay_only(id: EndpointId) -> iroh::EndpointAddr {
        let relay_url: iroh::RelayUrl = "https://relay.example.com".parse().expect("valid relay URL");
        iroh::EndpointAddr { id, addrs: std::collections::BTreeSet::from([iroh::TransportAddr::Relay(relay_url)]) }
    }

    #[test]
    fn classify_endpoint_addr_with_a_private_ipv4_is_local_lan() {
        let addr = addr_with_ips(test_endpoint_id(40), &["192.168.1.5:4433".parse().unwrap()]);
        assert_eq!(classify_endpoint_addr(&addr), TransportLink::LocalLan);
    }

    #[test]
    fn classify_endpoint_addr_with_a_public_ipv4_is_internet_direct() {
        let addr = addr_with_ips(test_endpoint_id(41), &["8.8.8.8:4433".parse().unwrap()]);
        assert_eq!(classify_endpoint_addr(&addr), TransportLink::InternetDirect);
    }

    #[test]
    fn classify_endpoint_addr_with_only_a_relay_url_is_internet_relay() {
        let addr = addr_with_relay_only(test_endpoint_id(42));
        assert_eq!(classify_endpoint_addr(&addr), TransportLink::InternetRelay);
    }

    #[test]
    fn classify_endpoint_addr_prefers_local_lan_when_both_kinds_of_ip_are_present() {
        // A dual-stack peer reachable both on a LAN IP and a public
        // one — LAN wins, since it's the more specific/actionable
        // signal (matches this device's own likely reachability, not
        // just the peer's).
        let addr = addr_with_ips(
            test_endpoint_id(43),
            &["192.168.1.5:4433".parse().unwrap(), "8.8.8.8:4433".parse().unwrap()],
        );
        assert_eq!(classify_endpoint_addr(&addr), TransportLink::LocalLan);
    }

    #[test]
    fn classify_endpoint_addr_link_local_ipv4_is_local_lan() {
        let addr = addr_with_ips(test_endpoint_id(44), &["169.254.1.1:4433".parse().unwrap()]);
        assert_eq!(classify_endpoint_addr(&addr), TransportLink::LocalLan);
    }
}
