//! §60 "Size-Aware Routing", §61 "Deadline-Aware Routing".

use crate::candidate::PathCandidate;
use crate::descriptor::ByteCount;
use crate::requirements::DeliveryRequirements;
use crate::setup::{static_setup_cost, SetupCost};

/// §60's own formula names four inputs — "setup cost, transfer time,
/// energy, reliability" — of which `energy`/`reliability` are already
/// real, separately-weighted [`crate::scoring::DefaultScorer`] terms
/// (`energy`/`stability`). This is the other two, combined exactly as
/// §60's own worked formula states: `completion_time ≈ setup + bytes /
/// bandwidth`.
///
/// A coarse, clearly-labeled *estimate* in milliseconds per
/// [`SetupCost`] tier — order-of-magnitude figures (a LAN/Iroh
/// handshake is tens of milliseconds; a Wi-Fi Direct group creation or
/// Bluetooth pairing is on the order of seconds), not measured values
/// this crate has no way to measure (see its own top doc comment on
/// scope). Kept as its own named function, not folded silently into
/// [`completion_time_millis`], so a caller can see and override the
/// assumption directly.
fn setup_cost_millis_estimate(cost: SetupCost) -> u64 {
    match cost {
        SetupCost::Cheap => 50,
        SetupCost::Moderate => 500,
        SetupCost::Expensive => 3_000,
    }
}

/// §60's formula, `None` standing in for §60's own "with uncertainty"
/// clause: rather than inventing a numeric confidence interval this
/// crate has no principled way to compute, a completion time is either
/// estimable (a bandwidth figure exists) or it isn't — the same
/// binary-not-fabricated honesty [`crate::scoring::DefaultScorer`]'s
/// own `recent_success` term already applies to
/// `last_success_millis`. Ignores `candidate.metrics.pool_state` — an
/// already-`Active` pooled connection would make the *real* setup cost
/// closer to zero than even [`SetupCost::Cheap`]'s estimate, but that
/// nuance belongs to [`crate::setup::effective_setup_cost`]'s own
/// scoring-time use, not to a size/deadline estimate a caller might
/// compute well before any specific connection is chosen.
pub fn completion_time_millis(candidate: &PathCandidate, size: ByteCount) -> Option<u64> {
    let bandwidth_bps = candidate.metrics.estimated_bandwidth?.0;
    if bandwidth_bps == 0 {
        return None;
    }
    let setup_millis = setup_cost_millis_estimate(static_setup_cost(candidate.transport));
    let transfer_millis = (size.0.saturating_mul(8))
        .saturating_div(bandwidth_bps)
        .saturating_mul(1_000);
    Some(setup_millis.saturating_add(transfer_millis))
}

/// §61: "If estimated arrival exceeds deadline: drop rather than queue
/// indefinitely." `deadline_millis` of `None` (no deadline stated)
/// never exceeds — matches this crate's usual "unconstrained, not
/// automatically failing" reading of an absent `Option` field (§13,
/// applied identically to [`crate::requirements::DeliveryRequirements::has_expired`]'s
/// own `None` case).
pub fn exceeds_deadline(estimated_completion_millis: u64, deadline_millis: Option<u32>) -> bool {
    match deadline_millis {
        Some(deadline) => estimated_completion_millis > deadline as u64,
        None => false,
    }
}

/// §61 applied across a candidate list, using §58's `estimated_size`
/// and `req.max_latency_millis` as the deadline (§61's own "video
/// frame useful for 100 ms" example is exactly a `max_latency_millis`
/// value read as a hard deadline rather than a soft scoring input —
/// [`crate::scoring::DefaultScorer`] already uses this same field as a
/// *soft* latency-suitability term; this is the *hard* "drop it
/// instead" reading §61 separately calls for). A candidate with no
/// bandwidth estimate (so [`completion_time_millis`] returns `None`)
/// is kept rather than dropped — §61's "estimated arrival exceeds
/// deadline" presupposes an actual estimate to compare; an unknown
/// completion time isn't evidence the deadline will be missed, any
/// more than [`crate::scoring::DefaultScorer`]'s own neutral `0.5`
/// treats an unknown latency as a known-bad one.
pub fn eliminate_deadline_exceeding_candidates<'a>(
    candidates: &'a [PathCandidate],
    size: ByteCount,
    req: &DeliveryRequirements,
) -> Vec<&'a PathCandidate> {
    candidates
        .iter()
        .filter(|c| match completion_time_millis(c, size) {
            Some(estimate) => !exceeds_deadline(estimate, req.max_latency_millis),
            None => true,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::candidate::TransportEndpoint;
    use crate::metrics::{Bitrate, PathMetrics};
    use crate::types::{PathCapabilities, PathId, RouteHealth, TransportKind};
    use siar_domain::DeviceId;

    fn candidate(transport: TransportKind, bandwidth_bps: Option<u64>) -> PathCandidate {
        let mut metrics = PathMetrics::unknown();
        metrics.estimated_bandwidth = bandwidth_bps.map(Bitrate);
        PathCandidate {
            path_id: PathId::new(),
            transport,
            peer: DeviceId::new(),
            endpoint: TransportEndpoint(Vec::new()),
            metrics,
            capabilities: PathCapabilities {
                reliable_stream: true,
                datagram: false,
                large_files: true,
                realtime_media: false,
                peer_discovery: false,
                store_and_forward: false,
                metered: false,
            },
            health: RouteHealth::Healthy,
            underlay: None,
        }
    }

    #[test]
    fn no_bandwidth_estimate_means_no_completion_time_estimate() {
        let c = candidate(TransportKind::IrohDirect, None);
        assert_eq!(completion_time_millis(&c, ByteCount(1_000)), None);
    }

    #[test]
    fn a_larger_payload_over_the_same_path_takes_longer() {
        let c = candidate(TransportKind::IrohDirect, Some(1_000_000)); // 1 Mbps
        let small = completion_time_millis(&c, ByteCount(1_000)).unwrap();
        let large = completion_time_millis(&c, ByteCount(1_000_000)).unwrap();
        assert!(large > small);
    }

    #[test]
    fn an_expensive_setup_transport_costs_more_even_for_the_same_bandwidth() {
        let cheap = candidate(TransportKind::LocalLan, Some(1_000_000));
        let expensive = candidate(TransportKind::WifiDirect, Some(1_000_000));
        let size = ByteCount(0); // isolate setup cost from transfer time
        assert!(
            completion_time_millis(&expensive, size).unwrap()
                > completion_time_millis(&cheap, size).unwrap()
        );
    }

    #[test]
    fn no_deadline_never_exceeds() {
        assert!(!exceeds_deadline(u64::MAX, None));
    }

    #[test]
    fn exceeding_the_stated_deadline_is_detected() {
        assert!(exceeds_deadline(200, Some(100)));
        assert!(!exceeds_deadline(50, Some(100)));
    }

    #[test]
    fn a_realtime_frame_over_a_too_slow_path_is_dropped_not_queued() {
        let mut req = DeliveryRequirements::realtime_media();
        req.max_latency_millis = Some(100); // §61's own "100 ms" example
        let slow = candidate(TransportKind::BluetoothClassic, Some(1_000)); // very slow + expensive setup
        let fast = candidate(TransportKind::IrohDirect, Some(10_000_000));
        let candidates = vec![slow, fast];

        let kept = eliminate_deadline_exceeding_candidates(&candidates, ByteCount(10_000), &req);
        assert_eq!(kept.len(), 1);
        assert_eq!(kept[0].transport, TransportKind::IrohDirect);
    }

    #[test]
    fn a_candidate_with_no_bandwidth_estimate_is_kept_rather_than_dropped() {
        let req = DeliveryRequirements::realtime_media();
        let unknown = candidate(TransportKind::IrohDirect, None);
        let candidates = vec![unknown];
        assert_eq!(
            eliminate_deadline_exceeding_candidates(&candidates, ByteCount(1_000), &req).len(),
            1
        );
    }
}
