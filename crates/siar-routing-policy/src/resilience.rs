//! §91 "Route Escalation Ladder", §92 "Timeout by Stage", §93 "Hedged
//! Requests", §95 "Route Diagnostics".

use serde::{Deserialize, Serialize};

use crate::candidate::PathCandidate;
use crate::plan::{RoutePlan, RouteStrategy};
use crate::platform::DeviceState;
use crate::requirements::DeliveryRequirements;
use crate::setup::{static_setup_cost, SetupCost};
use crate::types::{DeliveryClass, PathId, Priority, RouteHealth, TransportKind};

/// §91's own five stages, transcribed in the order it lists them.
/// `PartialOrd`/`Ord` follow declaration order deliberately — "cheaper
/// to reach" is exactly the ladder's own ordering, the same as every
/// other small ordered scale in this crate
/// ([`crate::metrics::EnergyCost`], [`crate::setup::SetupCost`], etc).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EscalationStage {
    ActiveConnections,
    KnownEndpoints,
    LightweightDiscovery,
    ExpensiveProximitySetup,
    DtnFallback,
}

/// Classifies `candidate` into §91's ladder. `Dtn` is checked first and
/// unconditionally lands on [`EscalationStage::DtnFallback`] — §91's
/// own stage 5 names the transport directly, not a state — after
/// which [`crate::acquisition::CandidateState`] (§89, round 7) covers
/// stages 1-2 exactly (`Active`→`ActiveConnections`,
/// `PassiveKnown`→`KnownEndpoints`), and [`crate::setup::static_setup_cost`]
/// (§44, round 2) splits `RequiresDiscovery`/`RequiresSetup` between
/// stages 3 and 4 by *cost* rather than by state alone — §91's own
/// stage 3/4 split is explicitly about cost ("lightweight" vs
/// "expensive"), which `CandidateState` alone doesn't carry (a
/// `RequiresSetup` candidate on a cheap transport is still cheaper
/// than a `RequiresDiscovery` one on an expensive transport).
pub fn escalation_stage_of(candidate: &PathCandidate) -> EscalationStage {
    if candidate.transport == TransportKind::Dtn {
        return EscalationStage::DtnFallback;
    }
    match candidate.state {
        crate::acquisition::CandidateState::Active => EscalationStage::ActiveConnections,
        crate::acquisition::CandidateState::PassiveKnown => EscalationStage::KnownEndpoints,
        crate::acquisition::CandidateState::RequiresDiscovery
        | crate::acquisition::CandidateState::RequiresSetup => {
            match static_setup_cost(candidate.transport) {
                SetupCost::Cheap | SetupCost::Moderate => EscalationStage::LightweightDiscovery,
                SetupCost::Expensive => EscalationStage::ExpensiveProximitySetup,
            }
        }
    }
}

/// §92: "Each stage has bounded timeout. Do not wait forever." Coarse,
/// named heuristic milliseconds — same "order-of-magnitude judgment
/// call, not a measured constant" posture as
/// [`crate::estimate`]'s own private `setup_cost_millis_estimate`/
/// [`crate::stability`]'s own private `VERY_STABLE_LIFETIME_MILLIS`.
/// `DtnFallback` gets the *shortest* timeout of the five despite being
/// the "last resort" stage — handing a bundle to the DTN subsystem is
/// a fast, local operation (§56 "DTN Routing Boundary": this crate
/// only decides whether DTN is *allowed*, the DTN subsystem's own
/// delivery timeline is out of scope), not a wait for delivery
/// confirmation.
pub fn timeout_millis_for_stage(stage: EscalationStage) -> u64 {
    match stage {
        EscalationStage::ActiveConnections => 2_000,
        EscalationStage::KnownEndpoints => 3_000,
        EscalationStage::LightweightDiscovery => 5_000,
        EscalationStage::ExpensiveProximitySetup => 8_000,
        EscalationStage::DtnFallback => 1_000,
    }
}

/// §91's actual point: "This controls resource cost." A caller's
/// discovery orchestrator — not [`crate::plan::plan_route`] itself,
/// which only ever scores candidates it's already been handed, per
/// this crate's own top doc comment on scope — calls this to decide
/// whether it's worth paying for the *next* stage's acquisition cost
/// at all, rather than always running every stage regardless of
/// whether an earlier one already succeeded.
pub fn should_escalate_beyond(stage: EscalationStage, candidates: &[PathCandidate]) -> bool {
    !candidates
        .iter()
        .any(|c| c.health == RouteHealth::Healthy && escalation_stage_of(c) <= stage)
}

/// §93: whether hedging is worth its own cost for this operation, and
/// how long to wait before firing the hedge. §93's own words — "useful
/// for: control messages, important text... potentially expensive:
/// use selectively" — read as two conditions: high enough priority to
/// be worth the duplicate send, and *not* [`DeliveryClass::Bulk`]
/// (doubling a file transfer's bandwidth cost is exactly the
/// "expensive" this section warns against) or
/// [`DeliveryClass::DelayTolerant`] (already served by
/// [`crate::plan::RouteStrategy::Redundant`]'s simultaneous-send
/// semantics, not a delayed hedge).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HedgePolicy {
    pub should_hedge: bool,
    pub delay_millis: u64,
}

pub fn hedge_policy_for(req: &DeliveryRequirements) -> HedgePolicy {
    let should_hedge = req.priority >= Priority::High
        && req.class != DeliveryClass::Bulk
        && req.class != DeliveryClass::DelayTolerant;
    // A quarter of the stated deadline, floored at 50ms so a very
    // tight `max_latency_millis` still leaves the primary a real
    // chance before the hedge fires; a fixed default when no deadline
    // is stated at all.
    let delay_millis = match req.max_latency_millis {
        Some(max) => (max as u64 / 4).max(50),
        None => 150,
    };
    HedgePolicy {
        should_hedge,
        delay_millis,
    }
}

/// §95's own list: "chosen path + reason, rejected candidates +
/// reason, retry history, escalation stage reached." Retry history
/// isn't included — this crate keeps no attempt history of its own
/// (see [`crate::policy::PolicyWeights::congestion`]'s own doc comment
/// on the same "no history" limitation for `failure_penalty`); a
/// caller that retries already knows its own retry history without
/// this crate reconstructing it.
#[derive(Debug, Clone)]
pub struct RouteDiagnostics {
    pub chosen_path_id: PathId,
    pub chosen_transport: TransportKind,
    pub strategy: RouteStrategy,
    pub escalation_stage_reached: EscalationStage,
    pub fallback_count: usize,
    pub replica_count: usize,
    pub rejected: Vec<(PathId, RejectionReason)>,
}

/// Deliberately scoped to the two checks this function can run without
/// an external dependency ([`crate::security::authorize_candidate`]
/// needs a `TrustedAccountStore`; [`crate::privacy::passes_privacy_policy`]
/// needs a `PrivacyPolicy` a caller may or may not have configured) —
/// a caller composing those checks itself already knows which one
/// rejected a given candidate, since it called them by name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RejectionReason {
    HardConstraintViolation,
    BackgroundRestricted,
}

fn diagnose_candidate(
    candidate: &PathCandidate,
    req: &DeliveryRequirements,
    device: Option<&DeviceState>,
) -> Option<RejectionReason> {
    if !crate::scoring::passes_hard_constraints(candidate, req) {
        return Some(RejectionReason::HardConstraintViolation);
    }
    if !crate::acquisition::is_currently_usable(candidate, device) {
        return Some(RejectionReason::BackgroundRestricted);
    }
    None
}

/// Builds a [`RouteDiagnostics`] from a produced `plan` plus the full
/// candidate list that went into producing it — `plan` alone doesn't
/// carry what got rejected along the way, only what survived.
pub fn diagnose(
    plan: &RoutePlan,
    all_candidates: &[PathCandidate],
    req: &DeliveryRequirements,
    device: Option<&DeviceState>,
) -> RouteDiagnostics {
    let kept = |id: PathId| {
        plan.primary.path_id == id
            || plan.fallbacks.iter().any(|c| c.path_id == id)
            || plan.replicas.iter().any(|c| c.path_id == id)
    };
    let rejected = all_candidates
        .iter()
        .filter(|c| !kept(c.path_id))
        .filter_map(|c| diagnose_candidate(c, req, device).map(|reason| (c.path_id, reason)))
        .collect();

    RouteDiagnostics {
        chosen_path_id: plan.primary.path_id,
        chosen_transport: plan.primary.transport,
        strategy: plan.strategy,
        escalation_stage_reached: escalation_stage_of(&plan.primary),
        fallback_count: plan.fallbacks.len(),
        replica_count: plan.replicas.len(),
        rejected,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::acquisition::CandidateState;
    use crate::candidate::TransportEndpoint;
    use crate::metrics::PathMetrics;
    use crate::types::{MeteredState, PathCapabilities, RoamingState};
    use siar_domain::DeviceId;

    fn candidate_at(
        transport: TransportKind,
        state: CandidateState,
        health: RouteHealth,
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
                metered: MeteredState::Unknown,
                roaming: RoamingState::Unknown,
                requires_foreground: false,
            },
            health,
            underlay: None,
            state,
        }
    }

    #[test]
    fn dtn_always_lands_on_the_fallback_stage_regardless_of_state() {
        let dtn = candidate_at(
            TransportKind::Dtn,
            CandidateState::Active,
            RouteHealth::Healthy,
        );
        assert_eq!(escalation_stage_of(&dtn), EscalationStage::DtnFallback);
    }

    #[test]
    fn requires_setup_on_an_expensive_transport_is_the_expensive_stage() {
        let bt_pairing = candidate_at(
            TransportKind::BluetoothClassic,
            CandidateState::RequiresSetup,
            RouteHealth::Healthy,
        );
        assert_eq!(
            escalation_stage_of(&bt_pairing),
            EscalationStage::ExpensiveProximitySetup
        );
    }

    #[test]
    fn requires_discovery_on_a_cheap_transport_is_the_lightweight_stage() {
        let iroh_discovery = candidate_at(
            TransportKind::IrohDirect,
            CandidateState::RequiresDiscovery,
            RouteHealth::Healthy,
        );
        assert_eq!(
            escalation_stage_of(&iroh_discovery),
            EscalationStage::LightweightDiscovery
        );
    }

    #[test]
    fn should_not_escalate_when_a_healthy_active_candidate_already_exists() {
        let active = candidate_at(
            TransportKind::IrohDirect,
            CandidateState::Active,
            RouteHealth::Healthy,
        );
        assert!(!should_escalate_beyond(
            EscalationStage::ActiveConnections,
            &[active]
        ));
    }

    #[test]
    fn should_escalate_when_nothing_healthy_is_available_yet() {
        let degraded = candidate_at(
            TransportKind::IrohDirect,
            CandidateState::Active,
            RouteHealth::Unreachable,
        );
        assert!(should_escalate_beyond(
            EscalationStage::ActiveConnections,
            &[degraded]
        ));
    }

    #[test]
    fn bulk_transfers_are_never_hedged_even_at_critical_priority() {
        let mut req = DeliveryRequirements::file_chunk();
        req.priority = Priority::Critical;
        assert!(!hedge_policy_for(&req).should_hedge);
    }

    #[test]
    fn an_ordinary_low_priority_message_is_not_hedged() {
        let req = DeliveryRequirements::interactive_message();
        assert!(!hedge_policy_for(&req).should_hedge); // default priority is Normal
    }

    #[test]
    fn a_high_priority_interactive_message_is_hedged() {
        let mut req = DeliveryRequirements::interactive_message();
        req.priority = Priority::High;
        assert!(hedge_policy_for(&req).should_hedge);
    }

    #[test]
    fn hedge_delay_is_a_quarter_of_the_stated_deadline() {
        let mut req = DeliveryRequirements::interactive_message();
        req.priority = Priority::High;
        req.max_latency_millis = Some(400);
        assert_eq!(hedge_policy_for(&req).delay_millis, 100);
    }

    #[test]
    fn hedge_delay_floors_at_50ms_for_a_very_tight_deadline() {
        let mut req = DeliveryRequirements::interactive_message();
        req.priority = Priority::High;
        req.max_latency_millis = Some(40); // a quarter of this is 10ms
        assert_eq!(hedge_policy_for(&req).delay_millis, 50);
    }

    #[test]
    fn diagnose_reports_the_chosen_path_and_reasons_for_what_was_rejected() {
        let primary = candidate_at(
            TransportKind::IrohDirect,
            CandidateState::Active,
            RouteHealth::Healthy,
        );
        let unreachable = candidate_at(
            TransportKind::IrohRelay,
            CandidateState::Active,
            RouteHealth::Unreachable,
        );
        let candidates = vec![primary.clone(), unreachable.clone()];
        let req = DeliveryRequirements::interactive_message();
        let policy = crate::policy::RoutingPolicyProfile::Balanced.policy();
        let scorer = crate::scoring::DefaultScorer {
            weights: policy.weights,
        };

        let plan =
            crate::plan::plan_route(&candidates, &req, &policy, &scorer, None, None, 0).unwrap();
        let diagnostics = diagnose(&plan, &candidates, &req, None);

        assert_eq!(diagnostics.chosen_path_id, primary.path_id);
        assert_eq!(
            diagnostics.escalation_stage_reached,
            EscalationStage::ActiveConnections
        );
        assert_eq!(diagnostics.rejected.len(), 1);
        assert_eq!(diagnostics.rejected[0].0, unreachable.path_id);
        assert_eq!(
            diagnostics.rejected[0].1,
            RejectionReason::HardConstraintViolation
        );
    }
}
