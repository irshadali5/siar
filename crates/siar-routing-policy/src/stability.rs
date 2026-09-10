//! §80 "Route Stability Score".

use crate::metrics::{Ratio, StabilityScore};

/// A connection open at least this long, with zero recorded timeouts
/// or path changes, earns [`StabilityScore::VeryStable`] — five
/// minutes chosen the same way this crate's other heuristic
/// thresholds are (e.g. `estimate.rs`'s own per-transport setup-cost
/// milliseconds): a defensible order-of-magnitude judgment call this
/// crate names explicitly rather than hiding inside an opaque formula,
/// not a measured constant. §84's own "no need for fake precision
/// initially" instruction applies just as much here as it does to that
/// section's own battery-cost scale.
const VERY_STABLE_LIFETIME_MILLIS: u64 = 5 * 60 * 1_000;

/// §80: "Derive from: connection lifetime, failure rate, path changes,
/// timeout history. A path with slightly worse RTT but much better
/// stability may win" — the second half of that sentence is already
/// true of existing scoring ([`crate::scoring::DefaultScorer`] weights
/// `stability` and `latency` independently, per policy profile), this
/// function is the first half: turning the four raw signals §80 names
/// into the [`StabilityScore`] that formula actually consumes.
///
/// `failure_rate` dominates the other three signals deliberately — a
/// path that fails half the time is unstable regardless of how long
/// it's nominally been "open," the same way a flaky connection that
/// technically hasn't been torn down is still not what §80 means by
/// "stable." `path_change_count`/`timeout_count` only matter once
/// `failure_rate` is already low; a caller can't offset a high failure
/// rate by having zero timeouts, since a failure and a timeout aren't
/// the same event, and this function isn't the place to guess whether
/// they're correlated for a given transport.
pub fn derive_stability_score(
    connection_lifetime_millis: u64,
    failure_rate: Ratio,
    path_change_count: u32,
    timeout_count: u32,
) -> StabilityScore {
    let failure_rate = failure_rate.get();
    if failure_rate >= 0.5 {
        return StabilityScore::VeryUnstable;
    }
    if failure_rate >= 0.2 {
        return StabilityScore::Unstable;
    }
    if failure_rate >= 0.05 {
        return StabilityScore::Moderate;
    }
    if timeout_count == 0
        && path_change_count == 0
        && connection_lifetime_millis >= VERY_STABLE_LIFETIME_MILLIS
    {
        return StabilityScore::VeryStable;
    }
    StabilityScore::Stable
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_high_failure_rate_is_very_unstable_regardless_of_everything_else() {
        let score = derive_stability_score(10 * 60 * 1_000, Ratio::new(0.6), 0, 0);
        assert_eq!(score, StabilityScore::VeryUnstable);
    }

    #[test]
    fn a_long_lived_connection_with_no_timeouts_or_path_changes_is_very_stable() {
        let score = derive_stability_score(VERY_STABLE_LIFETIME_MILLIS, Ratio::new(0.0), 0, 0);
        assert_eq!(score, StabilityScore::VeryStable);
    }

    #[test]
    fn a_short_lived_but_low_failure_connection_is_merely_stable_not_very_stable() {
        let score = derive_stability_score(1_000, Ratio::new(0.0), 0, 0);
        assert_eq!(score, StabilityScore::Stable);
    }

    #[test]
    fn any_timeout_prevents_very_stable_even_with_a_long_lifetime() {
        let score = derive_stability_score(VERY_STABLE_LIFETIME_MILLIS, Ratio::new(0.0), 0, 1);
        assert_eq!(score, StabilityScore::Stable);
    }

    #[test]
    fn a_moderate_failure_rate_lands_in_the_middle_of_the_scale() {
        let score = derive_stability_score(VERY_STABLE_LIFETIME_MILLIS, Ratio::new(0.1), 0, 0);
        assert_eq!(score, StabilityScore::Moderate);
    }
}
