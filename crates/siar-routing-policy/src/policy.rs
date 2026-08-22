//! §27 "Policy Profiles" through §35 "Path Hysteresis".

use crate::scoring::RouteScoreDelta;

/// The tunable inputs to [`crate::scoring::DefaultScorer`] — §24's
/// formula terms this crate actually models (see that module's own doc
/// comment for the two it doesn't: congestion, failure penalty). Not
/// required to sum to 1.0 — [`crate::scoring::RouteScore`] is a
/// relative ranking value, not a probability, so un-normalized weights
/// are fine as long as they're consistent within one comparison.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PolicyWeights {
    pub reachability: f64,
    pub latency: f64,
    pub bandwidth: f64,
    pub stability: f64,
    pub energy: f64,
    pub cost: f64,
    pub recent_success: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct HysteresisPolicy {
    pub switch_threshold: RouteScoreDelta,
    pub minimum_hold_millis: u64,
    pub degraded_override: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct RoutingPolicy {
    pub weights: PolicyWeights,
    pub hysteresis: HysteresisPolicy,
}

/// §27's seven named defaults. `BulkTransfer` is listed in §27's own
/// enumeration but, unlike the other six, has no dedicated numbered
/// section spelling out its rules — its preset below is inferred from
/// §7's own `file transfer → Bulk/Reliable` example and §31's
/// "Low-Cost Policy" (large-file guidance), not transcribed from spec
/// text the way the other six are, and is flagged here as such rather
/// than presented with the same confidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutingPolicyProfile {
    Balanced,
    LowLatency,
    LowPower,
    LowCost,
    HighReliability,
    Emergency,
    BulkTransfer,
}

impl RoutingPolicyProfile {
    pub fn policy(self) -> RoutingPolicy {
        match self {
            // §28: "prefer direct, prefer unmetered, avoid high battery
            // cost, use relay if needed" — a moderate, roughly even
            // weighting with a mild energy lean.
            Self::Balanced => RoutingPolicy {
                weights: PolicyWeights {
                    reachability: 1.0,
                    latency: 0.6,
                    bandwidth: 0.5,
                    stability: 0.7,
                    energy: 0.6,
                    cost: 0.4,
                    recent_success: 0.5,
                },
                hysteresis: HysteresisPolicy {
                    switch_threshold: RouteScoreDelta(0.3),
                    minimum_hold_millis: 10_000,
                    degraded_override: true,
                },
            },
            // §29: "minimize RTT, penalize jitter/loss, avoid path
            // switching, avoid DTN, avoid BLE" — latency dominates;
            // energy/cost barely matter; hysteresis threshold raised
            // ("avoid path switching") since a call mid-flight
            // shouldn't churn paths for a marginal gain.
            Self::LowLatency => RoutingPolicy {
                weights: PolicyWeights {
                    reachability: 1.0,
                    latency: 1.5,
                    bandwidth: 0.8,
                    stability: 1.0,
                    energy: 0.1,
                    cost: 0.1,
                    recent_success: 0.4,
                },
                hysteresis: HysteresisPolicy {
                    switch_threshold: RouteScoreDelta(0.6),
                    minimum_hold_millis: 30_000,
                    degraded_override: true,
                },
            },
            // §30: "prefer existing connection, avoid active discovery,
            // avoid Wi-Fi Direct setup" — energy dominates; latency and
            // bandwidth matter far less.
            Self::LowPower => RoutingPolicy {
                weights: PolicyWeights {
                    reachability: 1.0,
                    latency: 0.2,
                    bandwidth: 0.2,
                    stability: 0.5,
                    energy: 1.5,
                    cost: 0.4,
                    recent_success: 0.8, // "prefer existing connection"
                },
                hysteresis: HysteresisPolicy {
                    switch_threshold: RouteScoreDelta(0.5),
                    minimum_hold_millis: 60_000,
                    degraded_override: true,
                },
            },
            // §31: "prefer LAN/Wi-Fi, delay bulk traffic" — monetary
            // cost dominates.
            Self::LowCost => RoutingPolicy {
                weights: PolicyWeights {
                    reachability: 1.0,
                    latency: 0.2,
                    bandwidth: 0.4,
                    stability: 0.5,
                    energy: 0.3,
                    cost: 1.5,
                    recent_success: 0.4,
                },
                hysteresis: HysteresisPolicy {
                    switch_threshold: RouteScoreDelta(0.3),
                    minimum_hold_millis: 10_000,
                    degraded_override: true,
                },
            },
            // §32: "prefer proven paths, allow retry" — stability and
            // recent success dominate over raw latency/energy.
            Self::HighReliability => RoutingPolicy {
                weights: PolicyWeights {
                    reachability: 1.2,
                    latency: 0.4,
                    bandwidth: 0.4,
                    stability: 1.3,
                    energy: 0.3,
                    cost: 0.3,
                    recent_success: 1.2,
                },
                hysteresis: HysteresisPolicy {
                    switch_threshold: RouteScoreDelta(0.4),
                    minimum_hold_millis: 15_000,
                    degraded_override: true,
                },
            },
            // §33: "allow mesh, allow DTN, increase discovery, allow
            // redundancy... ignore some cost preferences... but should
            // still remain battery-aware" — reachability dominates
            // everything else; cost weight near zero (ignored, not
            // literally zero — "battery-aware" keeps a nonzero energy
            // term).
            Self::Emergency => RoutingPolicy {
                weights: PolicyWeights {
                    reachability: 2.0,
                    latency: 0.3,
                    bandwidth: 0.2,
                    stability: 0.6,
                    energy: 0.3,
                    cost: 0.05,
                    recent_success: 0.5,
                },
                hysteresis: HysteresisPolicy {
                    switch_threshold: RouteScoreDelta(0.2), // switch readily — reachability matters more than stability here
                    minimum_hold_millis: 2_000,
                    degraded_override: true,
                },
            },
            // Inferred, not transcribed — see this enum's own doc
            // comment. Bandwidth and cost dominate; latency barely
            // matters for a background bulk transfer.
            Self::BulkTransfer => RoutingPolicy {
                weights: PolicyWeights {
                    reachability: 1.0,
                    latency: 0.1,
                    bandwidth: 1.3,
                    stability: 0.8,
                    energy: 0.5,
                    cost: 0.9,
                    recent_success: 0.5,
                },
                hysteresis: HysteresisPolicy {
                    switch_threshold: RouteScoreDelta(0.3),
                    minimum_hold_millis: 20_000,
                    degraded_override: true,
                },
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn low_latency_weighs_latency_more_than_low_power_does() {
        let low_latency = RoutingPolicyProfile::LowLatency.policy();
        let low_power = RoutingPolicyProfile::LowPower.policy();
        assert!(low_latency.weights.latency > low_power.weights.latency);
    }

    #[test]
    fn low_power_weighs_energy_more_than_low_latency_does() {
        let low_latency = RoutingPolicyProfile::LowLatency.policy();
        let low_power = RoutingPolicyProfile::LowPower.policy();
        assert!(low_power.weights.energy > low_latency.weights.energy);
    }

    #[test]
    fn emergency_weighs_reachability_above_every_other_profile() {
        let emergency = RoutingPolicyProfile::Emergency.policy();
        for other in [
            RoutingPolicyProfile::Balanced,
            RoutingPolicyProfile::LowLatency,
            RoutingPolicyProfile::LowPower,
            RoutingPolicyProfile::LowCost,
            RoutingPolicyProfile::HighReliability,
            RoutingPolicyProfile::BulkTransfer,
        ] {
            assert!(emergency.weights.reachability >= other.policy().weights.reachability);
        }
    }
}
