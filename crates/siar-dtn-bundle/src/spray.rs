//! §23 "Spray-and-Wait Baseline": "Spray: create a limited number of
//! copies. Wait: copies seek destination/gateway."

/// Given a bundle's current `replication_budget` and the number of
/// distinct peers encountered "at once" (e.g. several devices visible
/// in one BLE scan window), decides how many copies to spray to this
/// encounter batch versus retain for future encounters — the actual
/// "limited number of copies" decision §23 describes but doesn't spell
/// out an algorithm for (binary spray, give-half-away, is this crate's
/// own reasonable choice, matching the original Spray-and-Wait
/// research algorithm's common "binary spray" variant, not a spec
/// transcription).
///
/// Returns `(copies_to_spray_now, copies_to_retain)`, both bounded by
/// `replication_budget` and by `peers_encountered` (never spray more
/// copies than there are peers to give them to).
pub fn spray_allocation(replication_budget: u8, peers_encountered: u8) -> (u8, u8) {
    if replication_budget == 0 || peers_encountered == 0 {
        return (0, replication_budget);
    }
    // Binary spray: give away half the budget (rounded up, so a
    // budget of 1 still sprays that one copy rather than never
    // spraying at all), capped by how many peers can actually receive
    // one.
    let half = replication_budget.div_ceil(2);
    let to_spray = half.min(peers_encountered).min(replication_budget);
    let to_retain = replication_budget - to_spray;
    (to_spray, to_retain)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_budget_of_one_still_sprays_its_one_copy() {
        assert_eq!(spray_allocation(1, 3), (1, 0));
    }

    #[test]
    fn spraying_never_exceeds_the_number_of_peers_present() {
        assert_eq!(spray_allocation(8, 1), (1, 7));
    }

    #[test]
    fn a_normal_budget_sprays_half_and_retains_half() {
        assert_eq!(spray_allocation(4, 10), (2, 2));
    }

    #[test]
    fn no_peers_means_nothing_is_sprayed() {
        assert_eq!(spray_allocation(4, 0), (0, 4));
    }

    #[test]
    fn zero_budget_has_nothing_left_to_spray() {
        assert_eq!(spray_allocation(0, 5), (0, 0));
    }
}
