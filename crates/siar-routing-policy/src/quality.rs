//! §71 "Route Planning for Calls".
//!
//! §71 names two things: a *selection* criterion ("low RTT, low
//! jitter, sufficient bandwidth, stable, realtime") and a *feedback*
//! one ("routing supplies path quality signals to media adaptation").
//! The selection half is already real —
//! [`crate::scoring::eliminate_hard_constraint_violations`]'s
//! `realtime_media` capability check plus [`crate::scoring::DefaultScorer`]'s
//! `latency`/`stability`/`bandwidth` terms are precisely those
//! criteria, already wired into [`crate::plan::plan_route`]. What's
//! new this round is the feedback half: §71's own video → lower
//! bitrate → audio → voice-message fallback ladder is a *media
//! adaptation* decision, not a routing one (this crate stops at
//! producing a [`crate::plan::RoutePlan`], same scope line its own top
//! doc comment already draws) — but "supplies path quality signals"
//! is squarely this crate's job, and until now there was no purpose-
//! built type for it; a caller doing that adaptation had nothing but
//! [`crate::metrics::PathMetrics`]'s own broader shape to reach into
//! and guess which fields were the relevant ones.

use crate::candidate::PathCandidate;
use crate::metrics::{Bitrate, Ratio, StabilityScore};

/// The subset of [`PathCandidate`]'s own data that §71's fallback
/// ladder actually needs to make its video/audio/voice-message
/// decision, named for that purpose rather than reused wholesale —
/// deliberately narrower than [`crate::metrics::PathMetrics`] itself
/// (no `monetary_cost`/`signal_quality`/`last_success_millis` here;
/// none of those inform a live media-adaptation decision the way RTT,
/// jitter, bandwidth, and stability do).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PathQualitySignal {
    pub rtt_millis: Option<u32>,
    pub jitter_millis: Option<u32>,
    pub estimated_bandwidth: Option<Bitrate>,
    pub packet_loss: Option<Ratio>,
    pub stability: StabilityScore,
}

/// Reads `candidate`'s own already-real metrics into the narrower
/// shape §71's own media-adaptation consumer needs — a projection, not
/// a new measurement; this crate still has no way to measure any of
/// these itself (see this crate's own top doc comment on scope).
pub fn quality_signal_for(candidate: &PathCandidate) -> PathQualitySignal {
    let m = &candidate.metrics;
    PathQualitySignal {
        rtt_millis: m.rtt_millis,
        jitter_millis: m.jitter_millis,
        estimated_bandwidth: m.estimated_bandwidth,
        packet_loss: m.packet_loss,
        stability: m.stability,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::candidate::TransportEndpoint;
    use crate::metrics::PathMetrics;
    use crate::types::{PathCapabilities, PathId, RouteHealth, TransportKind};
    use siar_domain::DeviceId;

    #[test]
    fn quality_signal_reflects_the_candidates_own_measured_metrics() {
        let mut metrics = PathMetrics::unknown();
        metrics.rtt_millis = Some(40);
        metrics.jitter_millis = Some(5);
        metrics.estimated_bandwidth = Some(Bitrate(2_000_000));

        let candidate = PathCandidate {
            path_id: PathId::new(),
            transport: TransportKind::IrohDirect,
            peer: DeviceId::new(),
            endpoint: TransportEndpoint(Vec::new()),
            metrics,
            capabilities: PathCapabilities {
                reliable_stream: false,
                datagram: true,
                large_files: false,
                realtime_media: true,
                peer_discovery: false,
                store_and_forward: false,
                metered: false,
            },
            health: RouteHealth::Healthy,
            underlay: None,
        };

        let signal = quality_signal_for(&candidate);
        assert_eq!(signal.rtt_millis, Some(40));
        assert_eq!(signal.jitter_millis, Some(5));
        assert_eq!(signal.estimated_bandwidth, Some(Bitrate(2_000_000)));
    }
}
