//! §151 "Route Probability", plus §152 "Metric Types Must Match
//! Reality", §153 "Scoring Normalization", §154 "Policy Weight
//! Example" — grouped here because §151 is the one concrete thing to
//! build in this cluster, and §152-154 are confirmations that what
//! this crate already does satisfies each of their own principles,
//! not new requirements.
//!
//! ## §151 "Route Probability"
//!
//! "For DTN/mesh, exact RTT may be unknown. Use: delivery likelihood,
//! expected delay class instead of fake precise latency." Round 15's
//! own `adapters.rs` named this as a gap — "delivery probability has
//! no equivalent" — on the grounds that *this crate* has no encounter
//! history to compute one from. §151 doesn't actually ask this crate
//! to compute that estimate itself, though: it asks for a place to
//! *receive* one from whatever adapter does have that history (DTN's
//! own "Part 06" peer-encounter logic, per §150's own closing line).
//! [`crate::metrics::PathMetrics::delivery_likelihood`]/
//! [`crate::metrics::PathMetrics::expected_delay_class`] (new this
//! round) are that place — round 15's gap was about computation, not
//! representation, and this closes the representation half honestly
//! without claiming to have solved the computation half too.
//!
//! [`ExpectedDelayClass`] is a coarse, four-value scale rather than a
//! duration — same "no principled use for a fake-precise number"
//! reasoning [`crate::platform::BatteryLevelClass`]'s own doc comment
//! already gives for a different quantity. `rtt_millis` staying `None`
//! for a DTN candidate is *already* "not fake precise latency" (it's
//! this crate's standing convention for "unmeasured," same as
//! [`crate::types::MeteredState::Unknown`]) — these two new fields are
//! additive, not a replacement for that convention.
//!
//! [`route_probability_signal`] is a small, deliberately narrow
//! reader — not a new scoring term wired into
//! [`crate::scoring::DefaultScorer`]'s weighted formula. Folding a
//! new weighted term into that formula is a real policy-tuning
//! decision (which existing weight should shrink to make room, what a
//! sensible default is) this round doesn't make on its own; a caller
//! wanting to fold delivery likelihood into a scoring decision today
//! reads this signal and factors it in on their own terms, the same
//! "compose, don't force onto every caller" shape several other
//! functions in this crate already take (e.g.
//! [`crate::authorization::authorize_path`]).
//!
//! ## §152 "Metric Types Must Match Reality"
//!
//! "Do not force Bluetooth RSSI, Iroh RTT, DTN encounter probability
//! into one misleading number. Use typed metric categories, then
//! derive normalized scoring." Already this crate's standing design:
//! [`crate::metrics::PathMetrics`] keeps `rtt_millis`,
//! `estimated_bandwidth`, `packet_loss`, `retransmission_rate`,
//! `congestion_state`, and now `delivery_likelihood`/
//! `expected_delay_class` as separate typed fields, never blended
//! into one opaque number before scoring sees them. Bluetooth RSSI
//! specifically has no field (round 15's `adapters.rs` already named
//! that gap under §148) — its absence is honest, not a case of it
//! being force-fit into something else.
//!
//! ## §153 "Scoring Normalization"
//!
//! "A scorer can normalize: LatencyScore, BandwidthScore, EnergyScore,
//! ReliabilityScore, CostScore, then combine with policy weights."
//! [`crate::scoring::PathScorer::score`]'s own internal
//! `*_suitability` terms (`latency_suitability`, `bandwidth_suitability`,
//! and so on) are exactly this — each independently normalized to
//! `[0.0, 1.0]` before being multiplied by its
//! [`crate::policy::PolicyWeights`] field and summed. Already true;
//! no new code.
//!
//! ## §154 "Policy Weight Example"
//!
//! The spec's own worked `RouteWeights` example names five terms
//! (latency, bandwidth, reliability, energy, cost) with weights
//! summing to 1.0. [`crate::policy::PolicyWeights`] is the real thing
//! this evolved into over many rounds — eleven terms, not required to
//! sum to 1.0 (that struct's own doc comment explains why: `RouteScore`
//! is a relative ranking, not a probability) — a superset of the
//! spec's simplified example, not a mismatch with it. "Do not expose
//! floating-point tuning directly to ordinary users. Profiles
//! configure these internally" is exactly what
//! [`crate::policy::RoutingPolicyProfile`] already does: seven named
//! profiles, no raw float ever handed to a caller who doesn't reach
//! into `.weights` on purpose.

use crate::metrics::{PathMetrics, Ratio};

/// §151's own second named signal, alongside delivery likelihood.
/// Four values — coarse on purpose, same reasoning
/// [`crate::platform::BatteryLevelClass`]'s own doc comment gives for
/// a different scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ExpectedDelayClass {
    Immediate,
    Minutes,
    Hours,
    Unknown,
}

/// The two §151 signals bundled together for a caller that wants
/// both without reading two separate `Option` fields itself.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RouteProbabilitySignal {
    pub delivery_likelihood: Option<Ratio>,
    pub expected_delay_class: Option<ExpectedDelayClass>,
}

/// Reads `metrics`'s own §151 fields into the bundled shape above — a
/// projection, not a new measurement, the same relationship
/// [`crate::quality::quality_signal_for`] already has to
/// `PathMetrics` for a different pair of fields.
pub fn route_probability_signal(metrics: &PathMetrics) -> RouteProbabilitySignal {
    RouteProbabilitySignal {
        delivery_likelihood: metrics.delivery_likelihood,
        expected_delay_class: metrics.expected_delay_class,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_dtn_candidate_with_no_rtt_can_still_carry_a_delivery_likelihood() {
        let mut metrics = PathMetrics::unknown();
        assert_eq!(metrics.rtt_millis, None);
        metrics.delivery_likelihood = Some(Ratio::new(0.7));
        metrics.expected_delay_class = Some(ExpectedDelayClass::Hours);

        let signal = route_probability_signal(&metrics);
        assert_eq!(signal.delivery_likelihood, Some(Ratio::new(0.7)));
        assert_eq!(signal.expected_delay_class, Some(ExpectedDelayClass::Hours));
        // The point of §151: RTT stays honestly unknown rather than
        // being backfilled with a fabricated number.
        assert_eq!(metrics.rtt_millis, None);
    }

    #[test]
    fn an_ordinary_candidate_with_no_probability_data_reads_as_all_none() {
        let metrics = PathMetrics::unknown();
        let signal = route_probability_signal(&metrics);
        assert_eq!(signal.delivery_likelihood, None);
        assert_eq!(signal.expected_delay_class, None);
    }
}
