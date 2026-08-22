//! §18 "Route Plan" through §23 "Delay-Tolerant Route", tying together
//! §25's evaluation order and §34's stickiness.

use crate::candidate::PathCandidate;
use crate::error::RoutingError;
use crate::policy::RoutingPolicy;
use crate::requirements::DeliveryRequirements;
use crate::scoring::{eliminate_hard_constraint_violations, PathScorer, RouteScore, RoutingContext};
use crate::types::{DeliveryClass, Priority};

/// §18's strategies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteStrategy {
    Single,
    Failover,
    Redundant,
    Multipath,
    DelayTolerant,
}

/// §18.
#[derive(Debug, Clone)]
pub struct RoutePlan {
    pub primary: PathCandidate,
    pub fallbacks: Vec<PathCandidate>,
    pub replicas: Vec<PathCandidate>,
    pub strategy: RouteStrategy,
}

/// §25's four-step evaluation order, run end to end:
/// 1. eliminate paths violating hard constraints
/// 2. score remaining paths
/// 3. apply stickiness/hysteresis
/// 4. produce route plan
///
/// `current` is the currently-active candidate, if any, matching §34's
/// worked example ("Iroh direct stable, Wi-Fi briefly appears — do not
/// churn"); pass `None` for a fresh destination with no existing route.
///
/// §22 "Multipath Route" is explicitly named in the spec as "a later
/// optimization, not required for v1 routing" — this function never
/// produces [`RouteStrategy::Multipath`] for that reason, not because
/// the type doesn't exist.
pub fn plan_route(
    candidates: &[PathCandidate],
    req: &DeliveryRequirements,
    policy: &RoutingPolicy,
    scorer: &dyn PathScorer,
    current: Option<&PathCandidate>,
) -> Result<RoutePlan, RoutingError> {
    let eligible = eliminate_hard_constraint_violations(candidates, req);
    if eligible.is_empty() {
        return Err(RoutingError::NoEligibleCandidates);
    }

    let context = RoutingContext { current_path: current.map(|c| c.path_id) };
    let mut scored: Vec<(&PathCandidate, RouteScore)> =
        eligible.into_iter().map(|c| (c, scorer.score(c, req, &context))).collect();
    // Deterministic tie-break by `path_id` (§123 "Deterministic
    // Scoring") — `f64` doesn't implement `Ord`, and two genuinely
    // equal scores should still produce a stable, repeatable pick
    // rather than depending on input order.
    scored.sort_by(|(a, a_score), (b, b_score)| {
        b_score.0.partial_cmp(&a_score.0).unwrap_or(std::cmp::Ordering::Equal).then_with(|| a.path_id.cmp(&b.path_id))
    });

    // §34/§35 stickiness: keep the current path unless a candidate
    // beats it by more than the policy's switch threshold, or the
    // current path has degraded (§35's `degraded_override`).
    let best = if let Some(current) = current {
        let current_still_eligible = scored.iter().find(|(c, _)| c.path_id == current.path_id);
        match current_still_eligible {
            Some((_, current_score)) => {
                let top = scored[0];
                let current_is_degraded =
                    matches!(current.health, crate::types::RouteHealth::Degraded | crate::types::RouteHealth::Suspect);
                let should_switch = (top.1 .0 - current_score.0 > policy.hysteresis.switch_threshold.0)
                    || (policy.hysteresis.degraded_override && current_is_degraded && top.0.path_id != current.path_id);
                if should_switch {
                    top
                } else {
                    *current_still_eligible.unwrap()
                }
            }
            // The current path didn't survive hard-constraint
            // elimination this round (e.g. it just went Unreachable) —
            // there's nothing to stick to.
            None => scored[0],
        }
    } else {
        scored[0]
    };

    let primary = best.0.clone();
    let mut fallbacks: Vec<PathCandidate> =
        scored.iter().map(|(c, _)| (*c).clone()).filter(|c| c.path_id != primary.path_id).collect();

    // §21 "Redundant Route": "Use redundancy sparingly" — reserved for
    // the spec's own named case, Critical + DelayTolerant (its SOS
    // example). Everything else with at least one fallback is
    // Failover (§20); a lone eligible candidate is Single (§19). §23
    // DelayTolerant is not separately produced here since a DTN
    // candidate competes on its own merits via [`crate::scoring`]
    // rather than this function special-casing "no other path exists"
    // — a real DTN candidate reaching this function already passed
    // §25 step 1 like any other transport.
    let strategy = if req.priority == Priority::Critical && req.class == DeliveryClass::DelayTolerant && !fallbacks.is_empty() {
        RouteStrategy::Redundant
    } else if fallbacks.is_empty() {
        RouteStrategy::Single
    } else {
        RouteStrategy::Failover
    };

    let replicas = if strategy == RouteStrategy::Redundant { fallbacks.drain(..1.min(fallbacks.len())).collect() } else { vec![] };

    Ok(RoutePlan { primary, fallbacks, replicas, strategy })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::candidate::TransportEndpoint;
    use crate::metrics::PathMetrics;
    use crate::policy::RoutingPolicyProfile;
    use crate::scoring::DefaultScorer;
    use crate::types::{PathCapabilities, PathId, RouteHealth, TransportKind};
    use siar_domain::DeviceId;

    fn candidate(transport: TransportKind, health: RouteHealth) -> PathCandidate {
        PathCandidate {
            path_id: PathId::new(),
            transport,
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
            health,
        }
    }

    #[test]
    fn a_single_eligible_candidate_produces_a_single_strategy_plan() {
        let candidates = vec![candidate(TransportKind::IrohDirect, RouteHealth::Healthy)];
        let req = DeliveryRequirements::interactive_message();
        let policy = RoutingPolicyProfile::Balanced.policy();
        let scorer = DefaultScorer { weights: policy.weights };

        let plan = plan_route(&candidates, &req, &policy, &scorer, None).unwrap();
        assert_eq!(plan.strategy, RouteStrategy::Single);
        assert!(plan.fallbacks.is_empty());
    }

    #[test]
    fn multiple_eligible_candidates_produce_a_failover_plan_with_the_healthiest_primary() {
        let healthy = candidate(TransportKind::IrohDirect, RouteHealth::Healthy);
        let degraded = candidate(TransportKind::IrohRelay, RouteHealth::Degraded);
        let candidates = vec![degraded.clone(), healthy.clone()];
        let req = DeliveryRequirements::interactive_message();
        let policy = RoutingPolicyProfile::Balanced.policy();
        let scorer = DefaultScorer { weights: policy.weights };

        let plan = plan_route(&candidates, &req, &policy, &scorer, None).unwrap();
        assert_eq!(plan.strategy, RouteStrategy::Failover);
        assert_eq!(plan.primary.path_id, healthy.path_id);
        assert_eq!(plan.fallbacks.len(), 1);
    }

    #[test]
    fn no_eligible_candidates_is_a_real_error_not_a_panic() {
        let unreachable = candidate(TransportKind::IrohDirect, RouteHealth::Unreachable);
        let req = DeliveryRequirements::interactive_message();
        let policy = RoutingPolicyProfile::Balanced.policy();
        let scorer = DefaultScorer { weights: policy.weights };

        let result = plan_route(&[unreachable], &req, &policy, &scorer, None);
        assert!(matches!(result, Err(RoutingError::NoEligibleCandidates)));
    }

    #[test]
    fn stickiness_keeps_the_current_path_when_a_new_candidate_is_only_marginally_better() {
        let current = candidate(TransportKind::IrohDirect, RouteHealth::Healthy);
        let marginally_better = candidate(TransportKind::LocalLan, RouteHealth::Healthy);
        let candidates = vec![current.clone(), marginally_better];
        let req = DeliveryRequirements::interactive_message();
        let policy = RoutingPolicyProfile::Balanced.policy();
        // Both candidates score identically under DefaultScorer (same
        // health, same unknown metrics) — well under the switch
        // threshold, so stickiness should keep `current`.
        let scorer = DefaultScorer { weights: policy.weights };

        let plan = plan_route(&candidates, &req, &policy, &scorer, Some(&current)).unwrap();
        assert_eq!(plan.primary.path_id, current.path_id);
    }

    #[test]
    fn a_degraded_current_path_is_switched_away_from_even_without_beating_the_threshold() {
        let current = candidate(TransportKind::IrohDirect, RouteHealth::Degraded);
        let healthy_alternative = candidate(TransportKind::LocalLan, RouteHealth::Healthy);
        let candidates = vec![current.clone(), healthy_alternative.clone()];
        let req = DeliveryRequirements::interactive_message();
        let policy = RoutingPolicyProfile::Balanced.policy();
        let scorer = DefaultScorer { weights: policy.weights };

        let plan = plan_route(&candidates, &req, &policy, &scorer, Some(&current)).unwrap();
        assert_eq!(plan.primary.path_id, healthy_alternative.path_id);
    }

    #[test]
    fn critical_delay_tolerant_traffic_with_a_fallback_uses_redundant_strategy() {
        let a = candidate(TransportKind::Dtn, RouteHealth::Healthy);
        let b = candidate(TransportKind::MeshRelay, RouteHealth::Healthy);
        let candidates = vec![a, b];
        let req = DeliveryRequirements::emergency();
        let policy = RoutingPolicyProfile::Emergency.policy();
        let scorer = DefaultScorer { weights: policy.weights };

        let plan = plan_route(&candidates, &req, &policy, &scorer, None).unwrap();
        assert_eq!(plan.strategy, RouteStrategy::Redundant);
        assert_eq!(plan.replicas.len(), 1);
    }
}
