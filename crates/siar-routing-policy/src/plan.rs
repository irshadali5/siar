//! §18 "Route Plan" through §23 "Delay-Tolerant Route", tying together
//! §25's evaluation order and §34's stickiness.

use crate::candidate::PathCandidate;
use crate::error::RoutingError;
use crate::policy::RoutingPolicy;
use crate::requirements::DeliveryRequirements;
use crate::scoring::{
    eliminate_hard_constraint_violations, PathScorer, RouteScore, RoutingContext,
};
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
/// `device` (§85, this round) is the same idea for battery/thermal/
/// foreground state — `None` when a caller has none to report, which
/// changes nothing (see [`crate::acquisition::eliminate_background_restricted`],
/// the only place this function actually reads it, for what `None`
/// resolves to: every candidate kept, since nothing is confirmed
/// backgrounded).
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
    device: Option<&crate::platform::DeviceState>,
) -> Result<RoutePlan, RoutingError> {
    let mut eligible = eliminate_hard_constraint_violations(candidates, req);
    // §86 "Background Restrictions": a second, independent elimination
    // pass — kept separate from `eliminate_hard_constraint_violations`
    // rather than folded into it, the same "composable, not baked into
    // the core hard-constraint function" posture already used for
    // [`crate::privacy::eliminate_privacy_violations`]/
    // [`crate::security::eliminate_untrusted_candidates`].
    eligible.retain(|c| crate::acquisition::is_currently_usable(c, device));
    if eligible.is_empty() {
        return Err(RoutingError::NoEligibleCandidates);
    }

    let context = RoutingContext {
        current_path: current.map(|c| c.path_id),
        device: device.copied(),
    };
    let mut scored: Vec<(&PathCandidate, RouteScore)> = eligible
        .into_iter()
        .map(|c| (c, scorer.score(c, req, &context)))
        .collect();
    // Deterministic tie-break by `path_id` (§123 "Deterministic
    // Scoring") — `f64` doesn't implement `Ord`, and two genuinely
    // equal scores should still produce a stable, repeatable pick
    // rather than depending on input order.
    scored.sort_by(|(a, a_score), (b, b_score)| {
        b_score
            .0
            .partial_cmp(&a_score.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.path_id.cmp(&b.path_id))
    });

    // §34/§35 stickiness: keep the current path unless a candidate
    // beats it by more than the policy's switch threshold, or the
    // current path has degraded (§35's `degraded_override`).
    let best = if let Some(current) = current {
        let current_still_eligible = scored.iter().find(|(c, _)| c.path_id == current.path_id);
        match current_still_eligible {
            Some((_, current_score)) => {
                let top = scored[0];
                let current_is_degraded = matches!(
                    current.health,
                    crate::types::RouteHealth::Degraded | crate::types::RouteHealth::Suspect
                );
                let should_switch = (top.1 .0 - current_score.0
                    > policy.hysteresis.switch_threshold.0)
                    || (policy.hysteresis.degraded_override
                        && current_is_degraded
                        && top.0.path_id != current.path_id);
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
    let mut fallbacks: Vec<PathCandidate> = scored
        .iter()
        .map(|(c, _)| (*c).clone())
        .filter(|c| c.path_id != primary.path_id)
        .collect();

    // §21 "Redundant Route": "Use redundancy sparingly" — reserved for
    // the spec's own named case, Critical + DelayTolerant (its SOS
    // example). Everything else with at least one fallback is
    // Failover (§20); a lone eligible candidate is Single (§19). §23
    // DelayTolerant is not separately produced here since a DTN
    // candidate competes on its own merits via [`crate::scoring`]
    // rather than this function special-casing "no other path exists"
    // — a real DTN candidate reaching this function already passed
    // §25 step 1 like any other transport.
    let strategy = if req.priority == Priority::Critical
        && req.class == DeliveryClass::DelayTolerant
        && !fallbacks.is_empty()
    {
        RouteStrategy::Redundant
    } else if fallbacks.is_empty() {
        RouteStrategy::Single
    } else {
        RouteStrategy::Failover
    };

    // §73 "Path Diversity": prefer a replica on a different underlay
    // from the primary rather than blindly taking the next-best-scored
    // fallback (which could easily share the primary's own underlay —
    // see [`crate::diversity`]'s own doc comment for §73's worked
    // "same Wi-Fi path twice" non-example).
    let replicas = if strategy == RouteStrategy::Redundant {
        match crate::diversity::most_diverse_fallback(&primary, &fallbacks) {
            Some(chosen) => {
                let chosen_id = chosen.path_id;
                let index = fallbacks
                    .iter()
                    .position(|f| f.path_id == chosen_id)
                    .expect("most_diverse_fallback only ever returns an item from `fallbacks`");
                vec![fallbacks.remove(index)]
            }
            // Unreachable in practice — `strategy == Redundant` is
            // only ever set when `!fallbacks.is_empty()` — but handled
            // rather than assumed, per this crate's usual "don't panic
            // on a state that shouldn't happen" posture.
            None => vec![],
        }
    } else {
        vec![]
    };

    Ok(RoutePlan {
        primary,
        fallbacks,
        replicas,
        strategy,
    })
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
                metered: crate::types::MeteredState::Unknown,
                roaming: crate::types::RoamingState::Unknown,
                requires_foreground: false,
            },
            health,
            underlay: None,
            state: crate::acquisition::CandidateState::Active,
        }
    }

    #[test]
    fn a_single_eligible_candidate_produces_a_single_strategy_plan() {
        let candidates = vec![candidate(TransportKind::IrohDirect, RouteHealth::Healthy)];
        let req = DeliveryRequirements::interactive_message();
        let policy = RoutingPolicyProfile::Balanced.policy();
        let scorer = DefaultScorer {
            weights: policy.weights,
        };

        let plan = plan_route(&candidates, &req, &policy, &scorer, None, None).unwrap();
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
        let scorer = DefaultScorer {
            weights: policy.weights,
        };

        let plan = plan_route(&candidates, &req, &policy, &scorer, None, None).unwrap();
        assert_eq!(plan.strategy, RouteStrategy::Failover);
        assert_eq!(plan.primary.path_id, healthy.path_id);
        assert_eq!(plan.fallbacks.len(), 1);
    }

    #[test]
    fn no_eligible_candidates_is_a_real_error_not_a_panic() {
        let unreachable = candidate(TransportKind::IrohDirect, RouteHealth::Unreachable);
        let req = DeliveryRequirements::interactive_message();
        let policy = RoutingPolicyProfile::Balanced.policy();
        let scorer = DefaultScorer {
            weights: policy.weights,
        };

        let result = plan_route(&[unreachable], &req, &policy, &scorer, None, None);
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
        let scorer = DefaultScorer {
            weights: policy.weights,
        };

        let plan = plan_route(&candidates, &req, &policy, &scorer, Some(&current), None).unwrap();
        assert_eq!(plan.primary.path_id, current.path_id);
    }

    #[test]
    fn a_degraded_current_path_is_switched_away_from_even_without_beating_the_threshold() {
        let current = candidate(TransportKind::IrohDirect, RouteHealth::Degraded);
        let healthy_alternative = candidate(TransportKind::LocalLan, RouteHealth::Healthy);
        let candidates = vec![current.clone(), healthy_alternative.clone()];
        let req = DeliveryRequirements::interactive_message();
        let policy = RoutingPolicyProfile::Balanced.policy();
        let scorer = DefaultScorer {
            weights: policy.weights,
        };

        let plan = plan_route(&candidates, &req, &policy, &scorer, Some(&current), None).unwrap();
        assert_eq!(plan.primary.path_id, healthy_alternative.path_id);
    }

    #[test]
    fn critical_delay_tolerant_traffic_with_a_fallback_uses_redundant_strategy() {
        let a = candidate(TransportKind::Dtn, RouteHealth::Healthy);
        let b = candidate(TransportKind::MeshRelay, RouteHealth::Healthy);
        let candidates = vec![a, b];
        let req = DeliveryRequirements::emergency();
        let policy = RoutingPolicyProfile::Emergency.policy();
        let scorer = DefaultScorer {
            weights: policy.weights,
        };

        let plan = plan_route(&candidates, &req, &policy, &scorer, None, None).unwrap();
        assert_eq!(plan.strategy, RouteStrategy::Redundant);
        assert_eq!(plan.replicas.len(), 1);
    }

    #[test]
    fn redundant_strategy_prefers_an_underlay_diverse_replica_over_the_next_best_score() {
        // Three eligible candidates: a healthy primary, a same-
        // underlay fallback that would score marginally higher than
        // the diverse one (via `recent_success`), and an underlay-
        // diverse fallback that scores lower but is what §73 actually
        // wants picked as the replica.
        let underlay = crate::diversity::UnderlayId::new();
        let mut primary = candidate(TransportKind::IrohDirect, RouteHealth::Healthy);
        primary.underlay = Some(underlay);

        let mut same_underlay_but_better_scored =
            candidate(TransportKind::IrohRelay, RouteHealth::Healthy);
        same_underlay_but_better_scored.underlay = Some(underlay);
        same_underlay_but_better_scored.metrics.last_success_millis = Some(1_000);

        let diverse_but_lower_scored = candidate(TransportKind::MeshRelay, RouteHealth::Healthy);

        let candidates = vec![
            primary.clone(),
            same_underlay_but_better_scored.clone(),
            diverse_but_lower_scored.clone(),
        ];
        let req = DeliveryRequirements::emergency();
        let policy = RoutingPolicyProfile::Emergency.policy();
        let scorer = DefaultScorer {
            weights: policy.weights,
        };

        let plan = plan_route(&candidates, &req, &policy, &scorer, Some(&primary), None).unwrap();
        assert_eq!(plan.strategy, RouteStrategy::Redundant);
        assert_eq!(plan.replicas.len(), 1);
        assert_eq!(plan.replicas[0].path_id, diverse_but_lower_scored.path_id);
        // The same-underlay candidate is still available as an
        // ordinary fallback — diversity only changes which one
        // becomes the *replica*, it doesn't discard the other.
        assert!(plan
            .fallbacks
            .iter()
            .any(|f| f.path_id == same_underlay_but_better_scored.path_id));
    }

    /// §69 "Route Planning for Messaging": "1. existing authenticated
    /// direct route... 5. DTN," with the spec's own caveat "actual
    /// ordering depends on policy and measured health" taken
    /// seriously rather than glossed over — this only asserts the
    /// parts of that ordering [`plan_route`]'s existing scoring
    /// robustly produces without fabricated measurements: an
    /// already-connected candidate stays primary (already covered by
    /// this file's own stickiness tests, not re-tested here), and a
    /// cheap-setup transport (LAN) outranks an expensive-setup one
    /// (Bluetooth pairing) when nothing else differs, straight from
    /// §44's `setup_cost` term. Where a candidate would land relative
    /// to DTN specifically depends on real measured latency/reliability
    /// data this crate has no basis to invent for a healthy-but-
    /// otherwise-unmeasured DTN candidate — see [`crate::setup::static_setup_cost`]'s
    /// own doc comment on why DTN's *setup* cost being cheap doesn't by
    /// itself say anything about whether it's a *good* choice for
    /// message delivery right now.
    #[test]
    fn messaging_prefers_cheap_setup_transports_when_all_else_is_equal() {
        let lan = candidate(TransportKind::LocalLan, RouteHealth::Healthy);
        let bluetooth_pairing = candidate(TransportKind::BluetoothClassic, RouteHealth::Healthy);
        let candidates = vec![bluetooth_pairing.clone(), lan.clone()];
        let req = DeliveryRequirements::interactive_message();
        let policy = RoutingPolicyProfile::Balanced.policy();
        let scorer = DefaultScorer {
            weights: policy.weights,
        };

        let plan = plan_route(&candidates, &req, &policy, &scorer, None, None).unwrap();
        assert_eq!(plan.primary.transport, TransportKind::LocalLan);
    }

    /// §70 "Route Planning for Files": "1. high-bandwidth direct...
    /// 5. Bluetooth only if small/allowed." Using realistic
    /// differentiated measurements (the kind a real caller would
    /// actually supply for a file transfer, unlike §69's test above)
    /// rather than leaving metrics unknown — with a `min_bandwidth`
    /// floor actually set, both §44's setup-cost term and §24's own
    /// bandwidth-suitability term agree: a high-bandwidth direct path
    /// beats a low-bandwidth, expensive-setup Bluetooth one.
    #[test]
    fn files_prefer_high_bandwidth_direct_over_small_only_bluetooth() {
        let mut high_bandwidth_direct = candidate(TransportKind::IrohDirect, RouteHealth::Healthy);
        high_bandwidth_direct.metrics.estimated_bandwidth =
            Some(crate::metrics::Bitrate(50_000_000));
        high_bandwidth_direct.capabilities.metered = crate::types::MeteredState::Unmetered;
        high_bandwidth_direct.capabilities.roaming = crate::types::RoamingState::NotRoaming;

        let mut bluetooth_small_only =
            candidate(TransportKind::BluetoothClassic, RouteHealth::Healthy);
        bluetooth_small_only.metrics.estimated_bandwidth = Some(crate::metrics::Bitrate(500_000));
        bluetooth_small_only.capabilities.metered = crate::types::MeteredState::Unmetered;
        bluetooth_small_only.capabilities.roaming = crate::types::RoamingState::NotRoaming;

        let candidates = vec![bluetooth_small_only.clone(), high_bandwidth_direct.clone()];
        let mut req = DeliveryRequirements::file_chunk();
        req.min_bandwidth = Some(crate::metrics::Bitrate(5_000_000));
        let policy = RoutingPolicyProfile::BulkTransfer.policy();
        let scorer = DefaultScorer {
            weights: policy.weights,
        };

        let plan = plan_route(&candidates, &req, &policy, &scorer, None, None).unwrap();
        assert_eq!(plan.primary.transport, TransportKind::IrohDirect);
    }

    /// §72 "Route Planning for Emergency": "SOS may choose: Internet
    /// direct + nearby mesh copy... depending on connectivity." This
    /// verifies the Emergency policy profile actually produces exactly
    /// that composition — an Internet-direct primary plus a mesh
    /// replica — when both are available, rather than asserting it as
    /// prose without checking.
    #[test]
    fn emergency_composes_internet_direct_plus_a_nearby_mesh_replica() {
        let internet_direct = candidate(TransportKind::IrohDirect, RouteHealth::Healthy);
        let nearby_mesh = candidate(TransportKind::MeshRelay, RouteHealth::Healthy);
        let candidates = vec![internet_direct.clone(), nearby_mesh.clone()];
        let req = DeliveryRequirements::emergency();
        let policy = RoutingPolicyProfile::Emergency.policy();
        let scorer = DefaultScorer {
            weights: policy.weights,
        };

        let plan = plan_route(&candidates, &req, &policy, &scorer, None, None).unwrap();
        assert_eq!(plan.strategy, RouteStrategy::Redundant);
        assert_eq!(plan.primary.transport, TransportKind::IrohDirect);
        assert_eq!(plan.replicas[0].transport, TransportKind::MeshRelay);
    }
}
