//! Retry backoff schedule (plan.md §33).
//!
//! Kept pure and dependency-free (no RNG, no `Instant`) specifically so
//! it's unit-testable without pulling in `siar-transport`/`siar-storage`
//! — this is the one piece of Phase 2's retry scheduler that can be
//! verified in a sandbox that can't compile `iroh`/`stoolap`.

/// plan.md §33's example sequence: 1s, 2s, 5s, 10s, 30s, 1m, 5m, 15m, then
/// holds at the ceiling. `attempts` is the count of prior failed
/// attempts (0 on the first retry after the initial failure).
pub fn backoff_millis(attempts: u32) -> u64 {
    const SCHEDULE_MS: &[u64] = &[
        1_000,
        2_000,
        5_000,
        10_000,
        30_000,
        60_000,
        300_000,
        900_000,
    ];
    let index = (attempts as usize).min(SCHEDULE_MS.len() - 1);
    SCHEDULE_MS[index]
}

/// Applies +/-`jitter_fraction` jitter to `base_millis`, using a caller
/// supplied `unit_random` in `[0.0, 1.0)` rather than an RNG type, so the
/// function itself stays pure and deterministic under test. Callers pass
/// `rand::random::<f64>()` (or similar) at the call site.
pub fn with_jitter(base_millis: u64, unit_random: f64, jitter_fraction: f64) -> u64 {
    debug_assert!((0.0..1.0).contains(&unit_random));
    debug_assert!((0.0..=1.0).contains(&jitter_fraction));
    let span = (base_millis as f64) * jitter_fraction;
    let offset = (unit_random * 2.0 - 1.0) * span; // in [-span, +span)
    (base_millis as f64 + offset).max(0.0) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn follows_the_documented_schedule() {
        assert_eq!(backoff_millis(0), 1_000);
        assert_eq!(backoff_millis(1), 2_000);
        assert_eq!(backoff_millis(2), 5_000);
        assert_eq!(backoff_millis(3), 10_000);
        assert_eq!(backoff_millis(4), 30_000);
        assert_eq!(backoff_millis(5), 60_000);
        assert_eq!(backoff_millis(6), 300_000);
        assert_eq!(backoff_millis(7), 900_000);
    }

    #[test]
    fn holds_at_the_ceiling_past_the_schedule() {
        assert_eq!(backoff_millis(8), 900_000);
        assert_eq!(backoff_millis(1000), 900_000);
    }

    #[test]
    fn jitter_stays_within_the_requested_fraction() {
        let base = 10_000u64;
        let low = with_jitter(base, 0.0, 0.2);
        let high = with_jitter(base, 0.999_999, 0.2);
        assert!(low >= 8_000 && low <= base, "low={low}");
        assert!(high >= base && high <= 12_000, "high={high}");
    }

    #[test]
    fn zero_jitter_fraction_is_a_no_op() {
        assert_eq!(with_jitter(5_000, 0.5, 0.0), 5_000);
    }
}
