//! §97 "Route Decision Explainability", §98 "Metrics Collection", §99
//! "Privacy of Metrics", §100 "Routing State Store", §101 "Startup
//! Behavior".

use serde::{Deserialize, Serialize};

use crate::plan::{RoutePlan, RouteStrategy};
use crate::scoring::RoutingContext;
use crate::types::{PathId, RouteHealth, TransportKind};

/// §97's own enum, transcribed exactly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RouteReason {
    ExistingHealthyConnection,
    LowestLatency,
    HighestBandwidth,
    PolicyPreferred,
    DirectPreferred,
    RelayFallback,
    EmergencyRedundancy,
    DtnOnlyAvailable,
}

fn is_direct_transport(t: TransportKind) -> bool {
    !matches!(t, TransportKind::IrohRelay | TransportKind::Dtn)
}

/// §97 calls this "excellent for debugging," not "a proof" — this is
/// a first-match-wins heuristic classification of *why* `primary`
/// ended up chosen, in a defensible priority order, not a precise
/// decomposition of [`crate::scoring::DefaultScorer`]'s own weighted
/// sum (that formula blends every term at once; no single term is
/// ever "the" reason in the way a human explanation wants one). Order
/// matters: `EmergencyRedundancy`/`DtnOnlyAvailable` are checked before
/// the more generic `ExistingHealthyConnection`, since a candidate can
/// satisfy both at once and the more specific reason is the more
/// useful one to report.
pub fn infer_reason(
    primary: &crate::candidate::PathCandidate,
    context: &RoutingContext,
    strategy: RouteStrategy,
) -> RouteReason {
    if strategy == RouteStrategy::Redundant {
        return RouteReason::EmergencyRedundancy;
    }
    if primary.transport == TransportKind::Dtn {
        return RouteReason::DtnOnlyAvailable;
    }
    if context.current_path == Some(primary.path_id) && primary.health == RouteHealth::Healthy {
        return RouteReason::ExistingHealthyConnection;
    }
    if primary.transport == TransportKind::IrohRelay {
        return RouteReason::RelayFallback;
    }
    if is_direct_transport(primary.transport) {
        return RouteReason::DirectPreferred;
    }
    RouteReason::PolicyPreferred
}

/// §98's own list, transcribed as an enum of *events* rather than
/// running counters — this crate keeps no counters of its own (no
/// history; see [`crate::policy::PolicyWeights::congestion`]'s doc
/// comment on the same limitation), so it can only ever report "this
/// one call produced X," leaving the actual counting to whatever
/// caller-owned metrics system increments on each event. "Average
/// route setup latency," "queue delay," and "retry count" are
/// deliberately not represented here: they're timing measurements this
/// crate has no clock to take (setup latency lives in whatever dials
/// the transport; queue delay in [`crate::dispatch`]'s own queue,
/// which already has its own timing surface; retry count in whatever
/// caller loop calls [`crate::retry::RetryPolicy::allows_attempt_at`]
/// and can count its own attempts directly).
///
/// §99 "Privacy of Metrics" is enforced by construction, not by
/// filtering: this enum has no field of any kind, let alone one that
/// could carry peer identity, an IP, a contact graph, or a location —
/// there's nothing here *to* redact. [`crate::resilience::RouteDiagnostics`]
/// (§95, round 8) is the same story: its own fields are `PathId`
/// (an opaque, routing-internal identifier — not a `DeviceId`, an IP,
/// or anything else that identifies a peer or place) plus counts and
/// enums, deliberately never `candidate.peer`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteMetricEvent {
    RouteSelected,
    Failover,
    PathSwitch,
    DirectTransport,
    RelayTransport,
    DtnFallback,
}

/// Maps one produced `plan` to the §98 events it represents.
/// `previous_primary` (the prior call's `plan.primary.path_id`, if any
/// — a caller's own job to remember, same as [`RoutingContext::current_path`]
/// already is) is what makes "path switch count" possible to detect at
/// all; without it, this function can't tell "chose the same path
/// again" from "switched."
pub fn metric_events_for(
    plan: &RoutePlan,
    previous_primary: Option<PathId>,
) -> Vec<RouteMetricEvent> {
    let mut events = vec![RouteMetricEvent::RouteSelected];
    if plan.strategy == RouteStrategy::Failover {
        events.push(RouteMetricEvent::Failover);
    }
    if let Some(previous) = previous_primary {
        if previous != plan.primary.path_id {
            events.push(RouteMetricEvent::PathSwitch);
        }
    }
    match plan.primary.transport {
        TransportKind::Dtn => events.push(RouteMetricEvent::DtnFallback),
        TransportKind::IrohRelay => events.push(RouteMetricEvent::RelayTransport),
        _ => events.push(RouteMetricEvent::DirectTransport),
    }
    events
}

/// §100's own three named fields, nothing more — "do not persist every
/// packet metric indefinitely" is satisfied by this type simply having
/// no field for one. This crate does no persistence of its own (no
/// storage I/O; see its top doc comment on scope) — this is the value
/// shape a caller's own persistence layer reads and writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteHint {
    pub last_successful_transport: TransportKind,
    pub recent_gateway: Option<PathId>,
    pub known_endpoint_freshness_millis: Option<u64>,
}

/// What a caller should persist after `plan` succeeds — `recent_gateway`
/// is `plan.primary.path_id` only when the primary is actually a relay
/// (§100's own word, "gateway," matches a relay path far better than a
/// direct one; a direct connection isn't gatewaying anything).
pub fn hint_from_plan(plan: &RoutePlan, now_millis: u64) -> RouteHint {
    RouteHint {
        last_successful_transport: plan.primary.transport,
        recent_gateway: (plan.primary.transport == TransportKind::IrohRelay)
            .then_some(plan.primary.path_id),
        known_endpoint_freshness_millis: Some(now_millis),
    }
}

/// §101: "Never assume persisted route is still valid." The one part
/// of §101's own startup sequence ("load hints → start transport →
/// revalidate candidates") that's actually this crate's to implement —
/// loading hints and starting a transport are a caller's own
/// persistence/wire concerns. Real revalidation, not a rubber stamp: a
/// hint is only still trustworthy if its own transport currently
/// appears among `current_candidates` *and* that candidate is healthy
/// right now — a hint naming a transport that's simply absent from the
/// current candidate list (radio off, no longer in range, etc.) is
/// exactly the stale-persisted-route case this section warns against.
pub fn revalidate_hint(
    hint: &RouteHint,
    current_candidates: &[crate::candidate::PathCandidate],
) -> bool {
    current_candidates
        .iter()
        .any(|c| c.transport == hint.last_successful_transport && c.health == RouteHealth::Healthy)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::acquisition::CandidateState;
    use crate::candidate::{PathCandidate, TransportEndpoint};
    use crate::metrics::PathMetrics;
    use crate::plan::RouteStrategy;
    use crate::types::{MeteredState, PathCapabilities, RoamingState};
    use siar_domain::DeviceId;

    fn candidate(transport: TransportKind, health: RouteHealth) -> PathCandidate {
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
                metered: MeteredState::Unknown,
                roaming: RoamingState::Unknown,
                requires_foreground: false,
            },
            health,
            underlay: None,
            state: CandidateState::Active,
        }
    }

    #[test]
    fn dtn_only_available_takes_priority_over_existing_connection() {
        let dtn = candidate(TransportKind::Dtn, RouteHealth::Healthy);
        let context = RoutingContext {
            current_path: Some(dtn.path_id),
            device: None,
        };
        assert_eq!(
            infer_reason(&dtn, &context, RouteStrategy::Single),
            RouteReason::DtnOnlyAvailable
        );
    }

    #[test]
    fn an_existing_healthy_connection_is_reported_as_such() {
        let iroh = candidate(TransportKind::IrohDirect, RouteHealth::Healthy);
        let context = RoutingContext {
            current_path: Some(iroh.path_id),
            device: None,
        };
        assert_eq!(
            infer_reason(&iroh, &context, RouteStrategy::Single),
            RouteReason::ExistingHealthyConnection
        );
    }

    #[test]
    fn a_fresh_relay_pick_is_relay_fallback_not_existing_connection() {
        let relay = candidate(TransportKind::IrohRelay, RouteHealth::Healthy);
        let context = RoutingContext::default(); // no current path
        assert_eq!(
            infer_reason(&relay, &context, RouteStrategy::Single),
            RouteReason::RelayFallback
        );
    }

    #[test]
    fn redundant_strategy_is_always_emergency_redundancy() {
        let iroh = candidate(TransportKind::IrohDirect, RouteHealth::Healthy);
        let context = RoutingContext::default();
        assert_eq!(
            infer_reason(&iroh, &context, RouteStrategy::Redundant),
            RouteReason::EmergencyRedundancy
        );
    }

    #[test]
    fn metric_events_detect_a_path_switch() {
        let plan = RoutePlan {
            primary: candidate(TransportKind::LocalLan, RouteHealth::Healthy),
            fallbacks: vec![],
            replicas: vec![],
            strategy: RouteStrategy::Single,
            hedge_delay_millis: None,
            reason: RouteReason::PolicyPreferred,
            created_at_millis: 0,
            valid_until_millis: 0,
        };
        let previous = PathId::new(); // definitely different from the fresh primary above
        let events = metric_events_for(&plan, Some(previous));
        assert!(events.contains(&RouteMetricEvent::PathSwitch));
        assert!(events.contains(&RouteMetricEvent::DirectTransport));
    }

    #[test]
    fn metric_events_report_dtn_fallback() {
        let plan = RoutePlan {
            primary: candidate(TransportKind::Dtn, RouteHealth::Healthy),
            fallbacks: vec![],
            replicas: vec![],
            strategy: RouteStrategy::Single,
            hedge_delay_millis: None,
            reason: RouteReason::PolicyPreferred,
            created_at_millis: 0,
            valid_until_millis: 0,
        };
        assert!(metric_events_for(&plan, None).contains(&RouteMetricEvent::DtnFallback));
    }

    #[test]
    fn hint_from_plan_records_gateway_only_for_a_relay_primary() {
        let direct_plan = RoutePlan {
            primary: candidate(TransportKind::IrohDirect, RouteHealth::Healthy),
            fallbacks: vec![],
            replicas: vec![],
            strategy: RouteStrategy::Single,
            hedge_delay_millis: None,
            reason: RouteReason::PolicyPreferred,
            created_at_millis: 0,
            valid_until_millis: 0,
        };
        assert_eq!(hint_from_plan(&direct_plan, 1_000).recent_gateway, None);

        let relay = candidate(TransportKind::IrohRelay, RouteHealth::Healthy);
        let relay_path_id = relay.path_id;
        let relay_plan = RoutePlan {
            primary: relay,
            fallbacks: vec![],
            replicas: vec![],
            strategy: RouteStrategy::Single,
            hedge_delay_millis: None,
            reason: RouteReason::PolicyPreferred,
            created_at_millis: 0,
            valid_until_millis: 0,
        };
        assert_eq!(
            hint_from_plan(&relay_plan, 1_000).recent_gateway,
            Some(relay_path_id)
        );
    }

    #[test]
    fn a_hint_naming_a_transport_no_longer_present_does_not_revalidate() {
        let hint = RouteHint {
            last_successful_transport: TransportKind::WifiDirect,
            recent_gateway: None,
            known_endpoint_freshness_millis: Some(0),
        };
        let current = vec![candidate(TransportKind::IrohDirect, RouteHealth::Healthy)];
        assert!(!revalidate_hint(&hint, &current));
    }

    #[test]
    fn a_hint_whose_transport_is_still_healthy_revalidates() {
        let hint = RouteHint {
            last_successful_transport: TransportKind::IrohDirect,
            recent_gateway: None,
            known_endpoint_freshness_millis: Some(0),
        };
        let current = vec![candidate(TransportKind::IrohDirect, RouteHealth::Healthy)];
        assert!(revalidate_hint(&hint, &current));
    }

    #[test]
    fn a_hint_whose_transport_is_present_but_unhealthy_does_not_revalidate() {
        let hint = RouteHint {
            last_successful_transport: TransportKind::IrohDirect,
            recent_gateway: None,
            known_endpoint_freshness_millis: Some(0),
        };
        let current = vec![candidate(
            TransportKind::IrohDirect,
            RouteHealth::Unreachable,
        )];
        assert!(!revalidate_hint(&hint, &current));
    }
}
