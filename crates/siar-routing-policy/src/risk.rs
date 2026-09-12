//! §131 "Security Event Integration", §132 "Blacklisting", §133 "Peer
//! Abuse" — grouped together because all three are about excluding a
//! path or peer for a reason that has nothing to do with how good its
//! metrics look, the same "must not win scoring regardless of
//! bandwidth" posture §133's own text states outright and §125's
//! "revoked device is never selected even when it scores far better"
//! property test already proved for device trust specifically.
//!
//! - §131: [`SecurityEvent`]/[`handle_security_event`]. "Authentication
//!   failure" and "revoked device" are both, per
//!   [`crate::failure::RouteFailureClass::AuthenticationFailure`]'s
//!   own doc comment, the same [`RouteFailureClass`] variant — so
//!   there's one trigger condition to check, not two. Of the spec's
//!   own three required actions, "invalidate route cache" is real
//!   code here (calls [`crate::cache::RouteCache::invalidate`]
//!   directly, unchanged from round 9); "remove candidate" and "emit
//!   security-relevant event" are surfaced as data
//!   ([`SecurityEvent`]'s own fields) rather than performed, because
//!   this crate has no persistent candidate registry to remove
//!   *from* and no event bus/logger to emit *to* — see this crate's
//!   own top doc comment on scope. A caller gets back exactly what it
//!   needs to do both itself.
//! - §132: [`PathPenalty`], transcribed with the spec's own two
//!   fields, plus [`eliminate_penalized_paths`]. "Do not permanently
//!   blacklist transport based on one transient error" is enforced by
//!   `until` being a timestamp, not a boolean flag — a penalty
//!   necessarily expires, structurally, not by a caller remembering
//!   to lift it.
//! - §133: [`PeerAbuseStatus`]/[`eliminate_abusive_peers`]. "Rate
//!   limit" is deliberately **not** one of this enum's variants —
//!   throttling a peer's *send rate* is a dispatch-layer concern this
//!   crate already has a real answer for
//!   ([`crate::fairness::RoundRobinFairQueue`]'s own per-key
//!   capacity), not a routing-*elimination* one; folding it in here
//!   would make an abusive-but-not-yet-blocked peer disappear from
//!   routing entirely, which is a stronger response than "rate limit"
//!   asks for. Only `Quarantined`/`Blocked` eliminate at this layer.

use std::collections::HashMap;

use crate::cache::RouteCache;
use crate::candidate::PathCandidate;
use crate::failure::RouteFailureClass;
use crate::types::{Destination, PathId};
use siar_domain::DeviceId;

/// §131's own two named triggers collapse to this one check — see
/// this module's own doc comment for why.
fn is_security_relevant(outcome_class: RouteFailureClass) -> bool {
    outcome_class == RouteFailureClass::AuthenticationFailure
}

/// §131's "emit security-relevant event" — the data a caller's own
/// logger/event bus needs, not a dispatched event itself (this crate
/// has neither to dispatch to).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SecurityEvent {
    pub path_id: PathId,
    pub peer: DeviceId,
}

/// §131's full sequence for one candidate's failed attempt. Returns
/// `None` (and touches nothing) when `outcome_class` isn't
/// security-relevant — an ordinary timeout must keep being treated as
/// an ordinary transient failure, per the spec's own "do not treat as
/// ordinary transient failure" read the other way around: only the
/// named cases get this treatment, not every failure.
pub fn handle_security_event(
    candidate: &PathCandidate,
    outcome_class: RouteFailureClass,
    destination: Destination,
    cache: &mut RouteCache,
) -> Option<SecurityEvent> {
    if !is_security_relevant(outcome_class) {
        return None;
    }
    cache.invalidate(destination);
    Some(SecurityEvent {
        path_id: candidate.path_id,
        peer: candidate.peer,
    })
}

/// §132, transcribed with the spec's own two field names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PathPenalty {
    pub until: u64,
    pub reason: RouteFailureClass,
}

impl PathPenalty {
    /// `until` is exclusive of the boundary itself, matching
    /// [`crate::requirements::DeliveryRequirements::has_expired`]'s
    /// own `>=` convention for "has this timestamp passed."
    pub fn is_active(&self, now_millis: u64) -> bool {
        now_millis < self.until
    }
}

/// §132 applied to a whole candidate list — same composable shape as
/// [`crate::security::eliminate_untrusted_candidates`]. A path with
/// no entry in `penalties` at all is untouched, the same "absence
/// means no restriction" convention every policy map in this crate
/// already uses.
pub fn eliminate_penalized_paths<'a>(
    candidates: &'a [PathCandidate],
    penalties: &HashMap<PathId, PathPenalty>,
    now_millis: u64,
) -> Vec<&'a PathCandidate> {
    candidates
        .iter()
        .filter(|c| {
            penalties
                .get(&c.path_id)
                .is_none_or(|p| !p.is_active(now_millis))
        })
        .collect()
}

/// §133's own three responses, minus "rate limit" — see this module's
/// own doc comment for why that one stays a dispatch-layer concern.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeerAbuseStatus {
    Quarantined,
    Blocked,
}

/// §133: "a high-bandwidth malicious peer must not win scoring" — the
/// only way to guarantee that structurally is to remove it before
/// scoring runs at all, not to hope a penalty term outweighs a large
/// bandwidth advantage. A peer with no entry in `abuse` is untouched.
pub fn eliminate_abusive_peers<'a>(
    candidates: &'a [PathCandidate],
    abuse: &HashMap<DeviceId, PeerAbuseStatus>,
) -> Vec<&'a PathCandidate> {
    candidates
        .iter()
        .filter(|c| !abuse.contains_key(&c.peer))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::candidate::TransportEndpoint;
    use crate::metrics::PathMetrics;
    use crate::types::{MeteredState, PathCapabilities, RoamingState, RouteHealth, TransportKind};
    use siar_domain::AccountId;

    fn candidate(peer: DeviceId) -> PathCandidate {
        PathCandidate {
            path_id: PathId::new(),
            transport: TransportKind::IrohDirect,
            peer,
            endpoint: TransportEndpoint(Vec::new()),
            metrics: PathMetrics::unknown(),
            capabilities: PathCapabilities {
                reliable_stream: true,
                datagram: false,
                large_files: false,
                realtime_media: false,
                peer_discovery: false,
                store_and_forward: false,
                metered: MeteredState::Unknown,
                roaming: RoamingState::Unknown,
                requires_foreground: false,
            },
            health: RouteHealth::Healthy,
            underlay: None,
            state: crate::acquisition::CandidateState::Active,
        }
    }

    fn dummy_plan() -> crate::plan::RoutePlan {
        crate::plan::RoutePlan {
            primary: candidate(DeviceId::new()),
            fallbacks: vec![],
            replicas: vec![],
            strategy: crate::plan::RouteStrategy::Single,
            hedge_delay_millis: None,
            reason: crate::explain::RouteReason::PolicyPreferred,
            created_at_millis: 0,
            valid_until_millis: 0,
        }
    }

    #[test]
    fn an_authentication_failure_invalidates_the_cache_and_surfaces_an_event() {
        let mut cache = RouteCache::new();
        let dest = Destination::Account(AccountId::new());
        cache.put(dest, dummy_plan(), 0, 5_000);
        let c = candidate(DeviceId::new());

        let event = handle_security_event(
            &c,
            RouteFailureClass::AuthenticationFailure,
            dest,
            &mut cache,
        );

        assert_eq!(
            event,
            Some(SecurityEvent {
                path_id: c.path_id,
                peer: c.peer,
            })
        );
        assert!(cache.get(dest, 0).is_none());
    }

    #[test]
    fn an_ordinary_timeout_is_not_treated_as_security_relevant() {
        let mut cache = RouteCache::new();
        let dest = Destination::Account(AccountId::new());
        cache.put(dest, dummy_plan(), 0, 5_000);
        let c = candidate(DeviceId::new());

        let event = handle_security_event(&c, RouteFailureClass::Temporary, dest, &mut cache);

        assert_eq!(event, None);
        assert!(cache.get(dest, 0).is_some()); // untouched
    }

    #[test]
    fn a_penalty_excludes_a_path_only_until_it_expires() {
        let c = candidate(DeviceId::new());
        let mut penalties = HashMap::new();
        penalties.insert(
            c.path_id,
            PathPenalty {
                until: 10_000,
                reason: RouteFailureClass::Temporary,
            },
        );
        let candidates = vec![c];

        assert!(eliminate_penalized_paths(&candidates, &penalties, 5_000).is_empty());
        assert_eq!(
            eliminate_penalized_paths(&candidates, &penalties, 10_000).len(),
            1
        );
    }

    #[test]
    fn a_quarantined_peer_is_eliminated_even_with_no_other_penalty_recorded() {
        let peer = DeviceId::new();
        let candidates = vec![candidate(peer)];
        let mut abuse = HashMap::new();
        abuse.insert(peer, PeerAbuseStatus::Quarantined);

        assert!(eliminate_abusive_peers(&candidates, &abuse).is_empty());
    }

    #[test]
    fn spec_133_a_high_bandwidth_peer_with_abuse_status_never_survives_to_compete_on_score() {
        // The peer's own metrics are excellent; abuse status must
        // still win.
        let peer = DeviceId::new();
        let mut excellent = candidate(peer);
        excellent.metrics.rtt_millis = Some(1);
        let candidates = vec![excellent];
        let mut abuse = HashMap::new();
        abuse.insert(peer, PeerAbuseStatus::Blocked);

        assert!(eliminate_abusive_peers(&candidates, &abuse).is_empty());
    }

    #[test]
    fn a_peer_with_no_recorded_abuse_status_is_untouched() {
        let candidates = vec![candidate(DeviceId::new())];
        let abuse = HashMap::new();
        assert_eq!(eliminate_abusive_peers(&candidates, &abuse).len(), 1);
    }
}
