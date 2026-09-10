//! §73 "Path Diversity", §74 "Underlay Group".

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::candidate::PathCandidate;

/// §74's own code block is literally just `pub struct UnderlayId;` — a
/// unit struct, which couldn't actually distinguish one underlay from
/// another. Read charitably (the same way this crate already reads
/// several of the spec's other bare-stub declarations, e.g.
/// [`crate::types::PathId`]'s own predecessor before it became a real
/// `Uuid` newtype), that's an illustrative placeholder for "add an id
/// type here," not a literal instruction to ship a type that can't
/// hold an identity — so this is a real newtype, following the same
/// pattern as every other id in this crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct UnderlayId(Uuid);

impl UnderlayId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for UnderlayId {
    fn default() -> Self {
        Self::new()
    }
}

/// §73's own worked example made precise: "Wi-Fi Internet + BLE mesh"
/// is diverse; "two logical streams over the same Wi-Fi path" is not.
/// The two candidates being compared are diverse when either: their
/// `underlay` fields are both present and different (the positive
/// case §73 actually names); or either side's `underlay` is unknown
/// (`None`).
///
/// Treating "unknown" as diverse rather than "assume shared" is a
/// deliberate, named limitation, not an oversight: `underlay` is
/// caller-supplied data this crate has no way to independently verify
/// (see this crate's own top doc comment on scope — no OS/network
/// introspection of its own), and a caller not populating it is the
/// ordinary case, not a distrusted one. Defaulting to "assume shared"
/// instead would silently disable diversity-aware replica selection
/// for every caller that hasn't wired up underlay detection yet —
/// worse than today's status quo of picking a replica with no
/// diversity awareness at all. The real cost of this choice is real
/// too: two candidates that *do* share a physical underlay the caller
/// simply didn't report will be treated as diverse when they aren't —
/// this function cannot detect that, and doesn't pretend to.
pub fn are_diverse(a: &PathCandidate, b: &PathCandidate) -> bool {
    match (a.underlay, b.underlay) {
        (Some(a), Some(b)) => a != b,
        _ => true,
    }
}

/// §73/§74's "candidates sharing the same underlay can be grouped" —
/// candidates with no reported underlay are grouped under `None`
/// together (not each their own singleton group), since this crate
/// has no basis to say they're *distinct* underlays any more than it
/// has basis to say they're the *same* one — see [`are_diverse`]'s own
/// doc comment on the same asymmetry.
pub fn group_by_underlay(
    candidates: &[PathCandidate],
) -> HashMap<Option<UnderlayId>, Vec<&PathCandidate>> {
    let mut groups: HashMap<Option<UnderlayId>, Vec<&PathCandidate>> = HashMap::new();
    for c in candidates {
        groups.entry(c.underlay).or_default().push(c);
    }
    groups
}

/// §73's actual point applied to redundant delivery: given a `primary`
/// already chosen and a list of scored `fallbacks` (assumed sorted
/// best-first, matching [`crate::plan::plan_route`]'s own ordering),
/// pick the highest-scored one that's [`are_diverse`] from `primary`
/// to serve as the redundant replica — rather than blindly taking
/// `fallbacks[0]`, which could easily be "the same Wi-Fi path,
/// slightly worse-scored," §73's own named non-example of diversity.
/// Falls back to `fallbacks[0]` when *no* fallback is diverse from the
/// primary — a same-underlay replica is still strictly better than no
/// replica at all, and §21's own "use redundancy sparingly" doesn't
/// say "refuse redundancy entirely when diversity isn't available."
pub fn most_diverse_fallback<'a>(
    primary: &PathCandidate,
    fallbacks: &'a [PathCandidate],
) -> Option<&'a PathCandidate> {
    fallbacks
        .iter()
        .find(|f| are_diverse(primary, f))
        .or_else(|| fallbacks.first())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::candidate::TransportEndpoint;
    use crate::metrics::PathMetrics;
    use crate::types::{PathCapabilities, PathId, RouteHealth, TransportKind};
    use siar_domain::DeviceId;

    fn candidate_with_underlay(
        transport: TransportKind,
        underlay: Option<UnderlayId>,
    ) -> PathCandidate {
        PathCandidate {
            path_id: PathId::new(),
            transport,
            peer: DeviceId::new(),
            endpoint: TransportEndpoint(Vec::new()),
            metrics: PathMetrics::unknown(),
            capabilities: PathCapabilities {
                reliable_stream: true,
                datagram: false,
                large_files: false,
                realtime_media: false,
                peer_discovery: false,
                store_and_forward: false,
                metered: crate::types::MeteredState::Unknown,
                roaming: crate::types::RoamingState::Unknown,
                requires_foreground: false,
            },
            health: RouteHealth::Healthy,
            underlay,
            state: crate::acquisition::CandidateState::Active,
        }
    }

    #[test]
    fn two_candidates_on_different_known_underlays_are_diverse() {
        let wifi = UnderlayId::new();
        let ble = UnderlayId::new();
        let a = candidate_with_underlay(TransportKind::IrohDirect, Some(wifi));
        let b = candidate_with_underlay(TransportKind::MeshRelay, Some(ble));
        assert!(are_diverse(&a, &b));
    }

    #[test]
    fn two_candidates_on_the_same_known_underlay_are_not_diverse() {
        let wifi = UnderlayId::new();
        let a = candidate_with_underlay(TransportKind::IrohDirect, Some(wifi));
        let b = candidate_with_underlay(TransportKind::IrohRelay, Some(wifi));
        assert!(!are_diverse(&a, &b));
    }

    #[test]
    fn unknown_underlay_on_either_side_is_treated_as_diverse() {
        let wifi = UnderlayId::new();
        let known = candidate_with_underlay(TransportKind::IrohDirect, Some(wifi));
        let unknown = candidate_with_underlay(TransportKind::IrohRelay, None);
        assert!(are_diverse(&known, &unknown));
        assert!(are_diverse(&unknown, &known));
    }

    #[test]
    fn group_by_underlay_groups_shared_underlays_together() {
        let wifi = UnderlayId::new();
        let a = candidate_with_underlay(TransportKind::IrohDirect, Some(wifi));
        let b = candidate_with_underlay(TransportKind::IrohRelay, Some(wifi));
        let c = candidate_with_underlay(TransportKind::MeshRelay, None);
        let candidates = [a, b, c];
        let groups = group_by_underlay(&candidates);
        assert_eq!(groups.get(&Some(wifi)).unwrap().len(), 2);
        assert_eq!(groups.get(&None).unwrap().len(), 1);
    }

    #[test]
    fn most_diverse_fallback_skips_a_same_underlay_candidate_for_a_diverse_one() {
        let wifi = UnderlayId::new();
        let primary = candidate_with_underlay(TransportKind::IrohDirect, Some(wifi));
        let same_underlay = candidate_with_underlay(TransportKind::IrohRelay, Some(wifi));
        let diverse = candidate_with_underlay(TransportKind::MeshRelay, Some(UnderlayId::new()));
        let fallbacks = vec![same_underlay.clone(), diverse.clone()];

        let chosen = most_diverse_fallback(&primary, &fallbacks).unwrap();
        assert_eq!(chosen.path_id, diverse.path_id);
    }

    #[test]
    fn most_diverse_fallback_falls_back_to_the_best_scored_when_nothing_is_diverse() {
        let wifi = UnderlayId::new();
        let primary = candidate_with_underlay(TransportKind::IrohDirect, Some(wifi));
        let only_option = candidate_with_underlay(TransportKind::IrohRelay, Some(wifi));
        let fallbacks = vec![only_option.clone()];

        let chosen = most_diverse_fallback(&primary, &fallbacks).unwrap();
        assert_eq!(chosen.path_id, only_option.path_id);
    }
}
