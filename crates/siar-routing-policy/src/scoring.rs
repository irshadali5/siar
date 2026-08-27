//! §24 "Route Scoring", §25 "Hard Constraints vs Soft Preferences", §26
//! "Path Scoring Interface".

use crate::candidate::PathCandidate;
use crate::metrics::{EnergyCost, NetworkCost, StabilityScore};
use crate::policy::PolicyWeights;
use crate::requirements::DeliveryRequirements;
use crate::types::{DeliveryClass, PathId, RouteHealth, TransportKind};

/// Higher is better. `f64`, not an integer — §155 "Integer Score
/// Option" names an integer score as an *alternative* worth
/// considering, not the required choice, and a weighted sum of several
/// `[0.0, 1.0]`-ish terms (this module's own `score` function) is far
/// more natural to compose as floats than to keep rescaling into
/// integers at every step.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct RouteScore(pub f64);

/// §35's `switch_threshold` field type.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct RouteScoreDelta(pub f64);

/// §26's third parameter. Just enough state for stickiness/hysteresis
/// (§34-35) and existing-connection preference (§45) to work — this
/// crate has no live connection pool of its own to introspect (see its
/// top doc comment on scope), so `current_path` is supplied by
/// whatever caller does have one.
#[derive(Debug, Clone, Copy, Default)]
pub struct RoutingContext {
    pub current_path: Option<PathId>,
}

/// §26.
pub trait PathScorer {
    fn score(
        &self,
        candidate: &PathCandidate,
        req: &DeliveryRequirements,
        context: &RoutingContext,
    ) -> RouteScore;
}

/// §25 step 1, applied to one candidate. Real checks against fields
/// this crate actually has — not every hard constraint the spec
/// gestures at is checkable without live transport/policy state this
/// crate doesn't hold (e.g. §105 "Path Authorization"'s
/// device-trusted/operation-authorized checks belong in
/// [`crate::resolve`], upstream of this function, not duplicated here).
pub fn passes_hard_constraints(candidate: &PathCandidate, req: &DeliveryRequirements) -> bool {
    if candidate.health == RouteHealth::Unreachable {
        return false;
    }
    if req.class == DeliveryClass::Realtime && !candidate.capabilities.realtime_media {
        return false;
    }
    if !req.allow_relay && candidate.transport == TransportKind::IrohRelay {
        return false;
    }
    if !req.allow_bluetooth
        && matches!(
            candidate.transport,
            TransportKind::BluetoothClassic | TransportKind::BluetoothLe
        )
    {
        return false;
    }
    if !req.allow_dtn && candidate.transport == TransportKind::Dtn {
        return false;
    }
    if !req.allow_metered && candidate.capabilities.metered {
        return false;
    }
    // A missing bandwidth estimate does NOT eliminate the candidate —
    // §13's own rule ("missing metrics must be represented explicitly")
    // means "unknown" is a distinct state from "known to be
    // insufficient," and only the latter is a hard-constraint failure.
    if let (Some(min_bw), Some(estimated)) =
        (req.min_bandwidth, candidate.metrics.estimated_bandwidth)
    {
        if estimated < min_bw {
            return false;
        }
    }
    true
}

/// §25 step 1, applied to a whole candidate set.
pub fn eliminate_hard_constraint_violations<'a>(
    candidates: &'a [PathCandidate],
    req: &DeliveryRequirements,
) -> Vec<&'a PathCandidate> {
    candidates
        .iter()
        .filter(|c| passes_hard_constraints(c, req))
        .collect()
}

fn unit_interval(low_is_bad_high_is_good: f64) -> f64 {
    low_is_bad_high_is_good.clamp(0.0, 1.0)
}

fn stability_unit(s: StabilityScore) -> f64 {
    match s {
        StabilityScore::VeryUnstable => 0.0,
        StabilityScore::Unstable => 0.25,
        StabilityScore::Moderate => 0.5,
        StabilityScore::Stable => 0.75,
        StabilityScore::VeryStable => 1.0,
    }
}

fn energy_unit(e: EnergyCost) -> f64 {
    // Inverted — Free is the *best* outcome for a "suitability" score.
    match e {
        EnergyCost::Free => 1.0,
        EnergyCost::Low => 0.75,
        EnergyCost::Moderate => 0.5,
        EnergyCost::High => 0.25,
        EnergyCost::VeryHigh => 0.0,
    }
}

fn cost_unit(c: NetworkCost) -> f64 {
    match c {
        NetworkCost::Free => 1.0,
        NetworkCost::Low => 0.75,
        NetworkCost::Moderate => 0.5,
        NetworkCost::High => 0.25,
        NetworkCost::VeryHigh => 0.0,
    }
}

fn reachability_unit(h: RouteHealth) -> f64 {
    match h {
        RouteHealth::Healthy => 1.0,
        RouteHealth::Degraded => 0.6,
        RouteHealth::Suspect => 0.3,
        RouteHealth::Unknown => 0.4,
        RouteHealth::Unreachable => 0.0, // already eliminated by passes_hard_constraints; kept exhaustive rather than unreachable!()
    }
}

/// The default, policy-weighted implementation of §24's conceptual
/// formula. Deterministic (§123 "Deterministic Scoring" — no RNG, no
/// wall-clock reads inside this function itself) and real, but a
/// genuinely partial reading of §24's formula: `congestion` (§79,
/// unimplemented — no live congestion signal type exists in this crate
/// yet) and `failure_penalty` beyond what [`RouteHealth`] already
/// folds into `reachability` are both treated as zero rather than
/// modeled, named explicitly here rather than silently dropped.
pub struct DefaultScorer {
    pub weights: PolicyWeights,
}

impl PathScorer for DefaultScorer {
    fn score(
        &self,
        candidate: &PathCandidate,
        req: &DeliveryRequirements,
        _context: &RoutingContext,
    ) -> RouteScore {
        let w = &self.weights;
        let m = &candidate.metrics;

        let reachability = reachability_unit(candidate.health);

        let latency_suitability = match (req.max_latency_millis, m.rtt_millis) {
            (Some(max), Some(rtt)) => {
                unit_interval(1.0 - (rtt as f64 / max as f64 - 1.0).max(0.0) / 2.0)
            }
            _ => 0.5, // unknown or unconstrained — neutral, not penalized (§13)
        };

        let bandwidth_suitability = match (req.min_bandwidth, m.estimated_bandwidth) {
            (Some(min), Some(est)) => unit_interval(est.0 as f64 / min.0.max(1) as f64).min(1.0),
            _ => 0.5,
        };

        let stability = stability_unit(m.stability);
        let energy_suitability = energy_unit(m.energy_cost);
        let cost_suitability = cost_unit(m.monetary_cost);
        // §40 "Path Memory": a hint, not proof — a present
        // `last_success_millis` counts for something, but this
        // function has no clock of its own to judge *how* recent, so
        // it's binary (present/absent) rather than decayed by age.
        let recent_success = if m.last_success_millis.is_some() {
            1.0
        } else {
            0.5
        };

        let total = w.reachability * reachability
            + w.latency * latency_suitability
            + w.bandwidth * bandwidth_suitability
            + w.stability * stability
            + w.energy * energy_suitability
            + w.cost * cost_suitability
            + w.recent_success * recent_success;

        RouteScore(total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::candidate::TransportEndpoint;
    use crate::metrics::PathMetrics;
    use crate::policy::RoutingPolicyProfile;
    use crate::types::PathCapabilities;
    use siar_domain::DeviceId;

    fn candidate(transport: TransportKind, health: RouteHealth, realtime: bool) -> PathCandidate {
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
                realtime_media: realtime,
                peer_discovery: true,
                store_and_forward: false,
                metered: false,
            },
            health,
        }
    }

    #[test]
    fn realtime_media_requires_a_realtime_capable_path() {
        let req = DeliveryRequirements::realtime_media();
        let capable = candidate(TransportKind::IrohDirect, RouteHealth::Healthy, true);
        let incapable = candidate(TransportKind::BluetoothLe, RouteHealth::Healthy, false);
        assert!(passes_hard_constraints(&capable, &req));
        assert!(!passes_hard_constraints(&incapable, &req));
    }

    #[test]
    fn disallowed_relay_is_eliminated_even_when_otherwise_healthy() {
        let mut req = DeliveryRequirements::interactive_message();
        req.allow_relay = false;
        let relay = candidate(TransportKind::IrohRelay, RouteHealth::Healthy, false);
        assert!(!passes_hard_constraints(&relay, &req));
    }

    #[test]
    fn unreachable_health_is_always_a_hard_elimination() {
        let req = DeliveryRequirements::interactive_message();
        let dead = candidate(TransportKind::IrohDirect, RouteHealth::Unreachable, false);
        assert!(!passes_hard_constraints(&dead, &req));
    }

    #[test]
    fn a_missing_bandwidth_estimate_does_not_eliminate_a_candidate() {
        let mut req = DeliveryRequirements::interactive_message();
        req.min_bandwidth = Some(crate::metrics::Bitrate(1_000_000));
        let unknown_bw = candidate(TransportKind::IrohDirect, RouteHealth::Healthy, false);
        assert!(passes_hard_constraints(&unknown_bw, &req)); // metrics.estimated_bandwidth is None from ::unknown()
    }

    #[test]
    fn a_healthier_candidate_scores_higher_all_else_equal() {
        let req = DeliveryRequirements::interactive_message();
        let policy = RoutingPolicyProfile::Balanced.policy();
        let scorer = DefaultScorer {
            weights: policy.weights,
        };
        let context = RoutingContext::default();

        let healthy = candidate(TransportKind::IrohDirect, RouteHealth::Healthy, false);
        let degraded = candidate(TransportKind::IrohDirect, RouteHealth::Degraded, false);

        let healthy_score = scorer.score(&healthy, &req, &context);
        let degraded_score = scorer.score(&degraded, &req, &context);
        assert!(healthy_score.0 > degraded_score.0);
    }
}
