//! §40 "Path Memory", §41 "Route Cache", §42 "Network Transition
//! Events" (the invalidation triggers this cache's `invalidate_*`
//! methods are meant to be called from).

use std::collections::HashMap;

use crate::plan::RoutePlan;
use crate::types::{Destination, TransportKind};

struct CachedRoute {
    plan: RoutePlan,
    observed_at_millis: u64,
    ttl_millis: u64,
}

/// §41: "Cache: Destination, Best Known Path, Fallbacks, Observed At,
/// TTL." `RoutePlan` already carries best-path-plus-fallbacks (§18), so
/// this wraps a whole plan per destination rather than duplicating its
/// fields. No wall clock of its own — every method that needs "now"
/// takes it as a parameter, same posture [`crate::metrics::Confidence`]
/// and [`crate::scoring`]'s `recent_success` term already take toward
/// time.
#[derive(Default)]
pub struct RouteCache {
    entries: HashMap<Destination, CachedRoute>,
}

impl RouteCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn put(
        &mut self,
        destination: Destination,
        plan: RoutePlan,
        now_millis: u64,
        ttl_millis: u64,
    ) {
        self.entries.insert(
            destination,
            CachedRoute {
                plan,
                observed_at_millis: now_millis,
                ttl_millis,
            },
        );
    }

    /// Returns the cached plan only if it hasn't outlived its TTL as of
    /// `now_millis` — an expired entry is treated as absent (not
    /// returned, not automatically evicted; call [`RouteCache::invalidate`]
    /// explicitly, or let the next successful [`RouteCache::put`]
    /// overwrite it) rather than something the caller has to separately
    /// check staleness on.
    pub fn get(&self, destination: Destination, now_millis: u64) -> Option<&RoutePlan> {
        let entry = self.entries.get(&destination)?;
        if now_millis.saturating_sub(entry.observed_at_millis) > entry.ttl_millis {
            return None;
        }
        Some(&entry.plan)
    }

    /// §41's invalidation triggers ("device directory update, network
    /// transition, transport shutdown, authentication failure") all
    /// funnel through this one method or [`RouteCache::invalidate_all`]
    /// — this crate doesn't listen for those events itself (no wire/OS
    /// integration; see its top doc comment), a caller that does own
    /// that integration calls in.
    pub fn invalidate(&mut self, destination: Destination) {
        self.entries.remove(&destination);
    }

    /// §42 "Network Transition Events": a network change can invalidate
    /// every cached route at once, not just one destination's.
    pub fn invalidate_all(&mut self) {
        self.entries.clear();
    }

    /// §43 "Route Re-Evaluation": "Do not recompute all routes on
    /// every small event. Use targeted invalidation: Wi-Fi changed →
    /// reevaluate Wi-Fi-related candidates." [`Self::invalidate_all`]
    /// is the blunt tool §42 sometimes calls for; this is the targeted
    /// one §43 asks for instead — only entries whose *primary* path
    /// uses `transport` are dropped. A cached route whose primary uses
    /// a different transport is left untouched even if one of its
    /// fallbacks happens to use `transport`, since that fallback isn't
    /// the path actually carrying traffic right now — re-evaluating it
    /// can wait for its own turn as primary rather than paying the
    /// recompute cost §43 says to avoid.
    pub fn invalidate_transport(&mut self, transport: TransportKind) {
        self.entries
            .retain(|_, cached| cached.plan.primary.transport != transport);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::candidate::{PathCandidate, TransportEndpoint};
    use crate::metrics::PathMetrics;
    use crate::plan::RouteStrategy;
    use crate::types::{PathCapabilities, PathId, RouteHealth, TransportKind};
    use siar_domain::{AccountId, DeviceId};

    fn dummy_plan() -> RoutePlan {
        let candidate = PathCandidate {
            path_id: PathId::new(),
            transport: TransportKind::IrohDirect,
            peer: DeviceId::new(),
            endpoint: TransportEndpoint(vec![]),
            metrics: PathMetrics::unknown(),
            capabilities: PathCapabilities {
                reliable_stream: true,
                datagram: true,
                large_files: true,
                realtime_media: false,
                peer_discovery: true,
                store_and_forward: false,
                metered: false,
            },
            health: RouteHealth::Healthy,
            underlay: None,
        };
        RoutePlan {
            primary: candidate,
            fallbacks: vec![],
            replicas: vec![],
            strategy: RouteStrategy::Single,
        }
    }

    #[test]
    fn a_fresh_entry_is_returned_within_ttl() {
        let mut cache = RouteCache::new();
        let dest = Destination::Account(AccountId::new());
        cache.put(dest, dummy_plan(), 1_000, 5_000);
        assert!(cache.get(dest, 3_000).is_some());
    }

    #[test]
    fn an_entry_past_its_ttl_is_treated_as_absent() {
        let mut cache = RouteCache::new();
        let dest = Destination::Account(AccountId::new());
        cache.put(dest, dummy_plan(), 1_000, 5_000);
        assert!(cache.get(dest, 10_000).is_none());
    }

    #[test]
    fn invalidate_all_clears_every_destination() {
        let mut cache = RouteCache::new();
        let dest_a = Destination::Account(AccountId::new());
        let dest_b = Destination::Account(AccountId::new());
        cache.put(dest_a, dummy_plan(), 1_000, 5_000);
        cache.put(dest_b, dummy_plan(), 1_000, 5_000);
        cache.invalidate_all();
        assert!(cache.get(dest_a, 1_000).is_none());
        assert!(cache.get(dest_b, 1_000).is_none());
    }

    #[test]
    fn invalidate_transport_only_drops_entries_whose_primary_uses_that_transport() {
        let mut cache = RouteCache::new();
        let wifi_dest = Destination::Account(AccountId::new());
        let iroh_dest = Destination::Account(AccountId::new());

        let mut wifi_plan = dummy_plan();
        wifi_plan.primary.transport = TransportKind::WifiDirect;
        cache.put(wifi_dest, wifi_plan, 1_000, 5_000);
        cache.put(iroh_dest, dummy_plan(), 1_000, 5_000); // primary is IrohDirect

        cache.invalidate_transport(TransportKind::WifiDirect);

        assert!(cache.get(wifi_dest, 1_000).is_none());
        assert!(cache.get(iroh_dest, 1_000).is_some());
    }
}
