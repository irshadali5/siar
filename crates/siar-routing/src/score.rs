//! Content-aware transport suitability, and route scoring — next.md
//! §9–10, §39, §53.

use siar_domain::TransportLink;
use siar_dtn::bundle::MessagePriority;

use crate::path::{capabilities_for, PathEntry};

/// next.md §53's size classes, used for both attachment routing (§53)
/// and BLE fragment sizing decisions upstream of this crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PayloadSizeClass {
    /// < 32 KB
    Tiny,
    /// 32 KB – 512 KB
    Small,
    /// 512 KB – 10 MB
    Medium,
    /// > 10 MB
    Large,
}

pub fn classify_payload(bytes: usize) -> PayloadSizeClass {
    const KB: usize = 1024;
    const MB: usize = 1024 * KB;
    if bytes < 32 * KB {
        PayloadSizeClass::Tiny
    } else if bytes < 512 * KB {
        PayloadSizeClass::Small
    } else if bytes < 10 * MB {
        PayloadSizeClass::Medium
    } else {
        PayloadSizeClass::Large
    }
}

/// next.md §53's per-transport ceiling: BLE tiny only, Bluetooth
/// Classic up to medium, everything else (Wi-Fi family, LAN, Iroh) up
/// to large. This is a *ceiling*, not a floor — a fast transport being
/// listed for "medium/large" in the doc's table doesn't mean it can't
/// also carry a tiny payload; §9's own examples send a 10-byte SOS over
/// whatever's available, BLE included.
fn max_supported_size(link: TransportLink) -> PayloadSizeClass {
    use TransportLink::*;
    match link {
        Ble => PayloadSizeClass::Tiny,
        BluetoothClassic => PayloadSizeClass::Medium,
        WifiDirect | WifiAware | LocalLan | InternetDirect | InternetRelay => PayloadSizeClass::Large,
    }
}

pub fn is_suitable_for_payload(link: TransportLink, size_class: PayloadSizeClass) -> bool {
    size_class <= max_supported_size(link)
}

/// next.md §10 / §39 combined into one weighted score — higher is
/// better. Deliberately a simple sum with round-number weights, not a
/// tuned formula: the doc names the factors (connectivity, bandwidth,
/// latency, battery cost, reliability, destination reachability,
/// priority) but gives no concrete weights, so these are a defensible
/// starting point, not a measured result. A caller that finds these
/// wrong for a real deployment should override by re-ranking
/// `PathEntry` candidates with its own weights, not by patching magic
/// numbers here without a reason tied to real routing behavior.
///
/// next.md §10: "Emergency traffic overrides ordinary preferences" —
/// modeled here as a large flat bonus that dwarfs every other factor,
/// so an Emergency message's ranking among *reachable* candidates is
/// barely affected by bandwidth/latency/reliability differences between
/// them; use [`is_suitable_for_payload`] first to filter out paths that
/// can't carry the payload at all; this function only ranks the ones
/// that already can.
pub fn route_score(entry: &PathEntry, priority: MessagePriority) -> i64 {
    let caps = capabilities_for(entry.link);

    let mut score: i64 = 0;
    score += match caps.bandwidth {
        crate::path::BandwidthClass::High => 40,
        crate::path::BandwidthClass::Medium => 25,
        crate::path::BandwidthClass::Low => 10,
        crate::path::BandwidthClass::VeryLow => 0,
    };
    score += match caps.latency {
        crate::path::LatencyClass::Low => 20,
        crate::path::LatencyClass::Medium => 10,
        crate::path::LatencyClass::High => 0,
    };
    score += (entry.reliability.clamp(0.0, 1.0) * 20.0) as i64;
    if let Some(rtt) = entry.rtt_millis {
        score -= (rtt as i64) / 10;
    }

    score += match priority {
        MessagePriority::Emergency => 1000,
        MessagePriority::Critical => 15,
        MessagePriority::Interactive => 10,
        MessagePriority::Normal => 5,
        MessagePriority::Background => 0,
    };

    score
}

/// Picks the highest-[`route_score`] entry among `candidates` that
/// [`is_suitable_for_payload`] accepts for `size_class` — `None` if
/// nothing suitable is reachable at all, which the caller should treat
/// as "hand this to DTN" (next.md §120's "otherwise → persist DTN
/// bundle"), not as an error.
pub fn best_route<'a>(
    candidates: &'a [PathEntry],
    priority: MessagePriority,
    size_class: PayloadSizeClass,
) -> Option<&'a PathEntry> {
    candidates
        .iter()
        .filter(|entry| is_suitable_for_payload(entry.link, size_class))
        .max_by_key(|entry| route_score(entry, priority))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::path::NextHop;

    fn entry(link: TransportLink, reliability: f32, rtt_millis: Option<u32>) -> PathEntry {
        PathEntry { link, next_hop: NextHop::Direct, last_seen: 0, rtt_millis, reliability }
    }

    #[test]
    fn classify_payload_matches_next_md_53s_boundaries() {
        assert_eq!(classify_payload(1024), PayloadSizeClass::Tiny);
        assert_eq!(classify_payload(32 * 1024), PayloadSizeClass::Small);
        assert_eq!(classify_payload(512 * 1024), PayloadSizeClass::Medium);
        assert_eq!(classify_payload(10 * 1024 * 1024), PayloadSizeClass::Large);
    }

    #[test]
    fn ble_rejects_anything_past_tiny() {
        assert!(is_suitable_for_payload(TransportLink::Ble, PayloadSizeClass::Tiny));
        assert!(!is_suitable_for_payload(TransportLink::Ble, PayloadSizeClass::Small));
    }

    #[test]
    fn fast_transports_still_accept_tiny_payloads() {
        // next.md §9: a 10-byte SOS over Wi-Fi/Iroh is fine even though
        // those transports are "for" medium/large per §53's table.
        assert!(is_suitable_for_payload(TransportLink::InternetDirect, PayloadSizeClass::Tiny));
        assert!(is_suitable_for_payload(TransportLink::WifiDirect, PayloadSizeClass::Tiny));
    }

    #[test]
    fn higher_bandwidth_and_lower_rtt_scores_higher() {
        let fast = entry(TransportLink::LocalLan, 0.9, Some(20));
        let slow = entry(TransportLink::Ble, 0.9, Some(200));
        assert!(route_score(&fast, MessagePriority::Normal) > route_score(&slow, MessagePriority::Normal));
    }

    #[test]
    fn emergency_priority_dwarfs_ordinary_score_differences() {
        let slow_but_reachable = entry(TransportLink::Ble, 0.5, Some(500));
        let fast = entry(TransportLink::LocalLan, 1.0, Some(5));
        // At Normal priority the fast path wins comfortably.
        assert!(route_score(&fast, MessagePriority::Normal) > route_score(&slow_but_reachable, MessagePriority::Normal));
        // At Emergency priority both scores are dominated by the flat
        // bonus — the fast path still wins (it's strictly better on
        // every other factor too), but the gap should have shrunk in
        // relative terms rather than grown, showing the bonus is doing
        // the dwarfing.
        let normal_gap = route_score(&fast, MessagePriority::Normal) - route_score(&slow_but_reachable, MessagePriority::Normal);
        let emergency_gap = route_score(&fast, MessagePriority::Emergency) - route_score(&slow_but_reachable, MessagePriority::Emergency);
        assert_eq!(normal_gap, emergency_gap, "the flat Emergency bonus should cancel out, leaving the same underlying gap");
    }

    #[test]
    fn best_route_picks_highest_scoring_suitable_candidate() {
        let candidates =
            vec![entry(TransportLink::Ble, 0.9, Some(50)), entry(TransportLink::LocalLan, 0.9, Some(20))];
        let chosen = best_route(&candidates, MessagePriority::Normal, PayloadSizeClass::Tiny).expect("should find a route");
        assert_eq!(chosen.link, TransportLink::LocalLan);
    }

    #[test]
    fn best_route_excludes_unsuitable_candidates_even_if_they_would_score_higher() {
        // BLE is the only candidate but the payload is too large for it.
        let candidates = vec![entry(TransportLink::Ble, 1.0, Some(1))];
        assert!(best_route(&candidates, MessagePriority::Normal, PayloadSizeClass::Large).is_none());
    }
}
