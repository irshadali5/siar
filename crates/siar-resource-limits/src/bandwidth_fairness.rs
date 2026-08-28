//! §54 "Bandwidth Fairness", §55 "Weighted Fair Queueing", §56
//! "Strict Priority Risks".
//!
//! §56's own warning — "Pure strict priority can starve bulk forever.
//! Use weighted fairness + critical reserve rather than absolute
//! priority for all classes" — is the actual correctness requirement
//! this module has to satisfy, not just the WFQ weight table alone. A
//! naive "give each tier `capacity * weight / total_weight`" single
//! pass would satisfy §55's ratios on paper but still leave `Bulk`
//! starved in practice whenever a higher-weighted tier requests *less*
//! than its computed share — the unclaimed leftover has to be
//! redistributed to whichever tiers still want it, or that capacity
//! just sits idle while `Bulk` gets nothing. [`allocate_bandwidth`]
//! implements the classic weighted max-min fair-share algorithm
//! specifically so that redistribution actually happens, tested
//! directly against the starvation case §56 warns about.
//!
//! Distinct from `siar-protocol-ext`'s `FairScheduler`: that type
//! decides dequeue *order* for discrete queued items (a scheduling
//! concern, already real, built in an earlier pass of this workspace's
//! Part 01 crate); this module divides a continuous *byte-rate budget*
//! among simultaneously-active flows (a bandwidth-sharing concern).
//! They solve different problems and neither is built on top of the
//! other.

use crate::admission::WorkPriority;
use std::collections::HashMap;

/// §55's own worked weight table, used verbatim as the default (the
/// spec gives concrete numbers here, unlike most of this crate's other
/// defaults). [`WorkPriority::Critical`] deliberately has no weight —
/// §55's own text excludes it from the weighted rotation entirely
/// ("Critical traffic gets special bounded preemption"), handled
/// separately by [`CriticalPreemption`] rather than folded into this
/// table. [`WorkPriority::Control`] here means *bandwidth-shaping*
/// control traffic in §55's sense, not `siar-protocol-ext`'s own
/// distinct notion of protocol control frames — the two crates reuse
/// the same six-tier enum (§19's own cross-part alignment) but this
/// weight table is specific to this module's bandwidth concern.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WfqWeights {
    pub control: u32,
    pub interactive: u32,
    pub normal: u32,
    pub bulk: u32,
    pub background: u32,
}

impl WfqWeights {
    pub const fn spec_example() -> Self {
        Self {
            control: 8,
            interactive: 6,
            normal: 4,
            bulk: 2,
            background: 1,
        }
    }

    fn weight_for(&self, tier: WorkPriority) -> Option<u32> {
        match tier {
            WorkPriority::Critical => None,
            WorkPriority::Control => Some(self.control),
            WorkPriority::Interactive => Some(self.interactive),
            WorkPriority::Normal => Some(self.normal),
            WorkPriority::Bulk => Some(self.bulk),
            WorkPriority::Background => Some(self.background),
        }
    }
}

/// §57's "bounded preemption" for Critical traffic, sized here as a
/// fraction of total link capacity rather than a fixed byte count so
/// it scales with whatever the actual link speed is. §55/§57 both
/// require the preemption to be *bounded* but neither gives a
/// concrete fraction — `0.5` (never more than half the link) is this
/// module's own reasoned choice, not a transcribed spec value: large
/// enough that a genuine SOS/emergency transfer isn't starved by its
/// own bound, small enough that Critical traffic alone can never fully
/// starve every other class the way §56 warns against for strict
/// priority in general.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CriticalPreemption {
    pub max_fraction_of_total: f64,
}

impl CriticalPreemption {
    pub const fn conservative_default() -> Self {
        Self {
            max_fraction_of_total: 0.5,
        }
    }

    fn bound_bytes(&self, total_bytes_per_sec: u64) -> u64 {
        ((total_bytes_per_sec as f64) * self.max_fraction_of_total) as u64
    }
}

/// §54: "One transfer must not monopolize link... weighted fair
/// scheduling across peers/extensions/priority classes." Allocates
/// `total_bytes_per_sec` among `demands` (each a `(tier, requested_bytes)`
/// pair, in any order, duplicates of the same tier allowed for
/// multiple simultaneous flows in that tier) and returns the granted
/// bytes/sec for each entry, in the same order as `demands`.
///
/// Two-phase: Critical demand is served first, up to
/// `critical_bound`'s fraction of the link (§57); the remainder is
/// then divided among every other tier by weighted max-min fair share
/// (§55's weights, §56's redistribution requirement) — never a single
/// proportional pass, so a tier that asks for less than its computed
/// share doesn't leave that capacity stranded while a needier
/// lower-weight tier goes unserved.
pub fn allocate_bandwidth(
    total_bytes_per_sec: u64,
    demands: &[(WorkPriority, u64)],
    weights: &WfqWeights,
    critical_bound: &CriticalPreemption,
) -> Vec<u64> {
    let mut allocated = vec![0u64; demands.len()];

    let critical_indices: Vec<usize> = demands
        .iter()
        .enumerate()
        .filter(|(_, (tier, _))| *tier == WorkPriority::Critical)
        .map(|(i, _)| i)
        .collect();
    let critical_bound_bytes = critical_bound.bound_bytes(total_bytes_per_sec);
    let critical_entries: Vec<(usize, u64, u32)> = critical_indices
        .iter()
        .map(|&i| (i, demands[i].1, 1u32))
        .collect();
    let critical_granted = weighted_max_min(critical_bound_bytes, &critical_entries);
    let mut critical_total = 0u64;
    for (&i, &granted) in &critical_granted {
        allocated[i] = granted;
        critical_total += granted;
    }

    let remainder = total_bytes_per_sec.saturating_sub(critical_total);
    let weighted_entries: Vec<(usize, u64, u32)> = demands
        .iter()
        .enumerate()
        .filter_map(|(i, (tier, demand))| weights.weight_for(*tier).map(|w| (i, *demand, w)))
        .collect();
    let weighted_granted = weighted_max_min(remainder, &weighted_entries);
    for (i, granted) in weighted_granted {
        allocated[i] = granted;
    }

    allocated
}

/// The core weighted max-min fair-share loop: repeatedly compute each
/// still-active entry's proportional share of whatever capacity is
/// still unclaimed, fully satisfy any entry whose share already covers
/// its full demand (removing it from further rounds and returning its
/// unused headroom to the pool), and repeat until either no capacity
/// or no active entry remains. Entries keyed by their original index
/// so callers can reassemble results in input order.
fn weighted_max_min(mut remaining: u64, entries: &[(usize, u64, u32)]) -> HashMap<usize, u64> {
    let mut granted: HashMap<usize, u64> = HashMap::new();
    let mut active: Vec<(usize, u64, u32)> = entries
        .iter()
        .copied()
        .filter(|&(_, demand, _)| demand > 0)
        .collect();

    while remaining > 0 && !active.is_empty() {
        let total_weight: u64 = active.iter().map(|&(_, _, w)| w as u64).sum();
        if total_weight == 0 {
            break;
        }

        // One pass: compute this round's proportional share per active
        // entry, satisfy anything whose share already meets its full
        // demand, and track how much of `remaining` actually got
        // claimed this round (fully-satisfied entries claim only their
        // demand, not their full share, so the difference goes back
        // into the pool for the entries still active next round).
        let mut satisfied_any = false;
        let mut still_active = Vec::new();
        let mut claimed_this_round = 0u64;

        for (idx, demand, weight) in active {
            let share = (remaining * weight as u64) / total_weight;
            if share >= demand {
                *granted.entry(idx).or_insert(0) += demand;
                claimed_this_round += demand;
                satisfied_any = true;
            } else {
                still_active.push((idx, demand, weight));
            }
        }

        if satisfied_any {
            remaining = remaining.saturating_sub(claimed_this_round);
            active = still_active;
            continue;
        }

        // No entry's share covers its full demand — every remaining
        // active entry gets its proportional share. Floor division
        // can leave a small remainder unclaimed (e.g. 95 split 6:4:2:1
        // floors to 43+29+14+7=93, two bytes short of 95) — that
        // remainder is real, unallocated capacity, not something safe
        // to just leave idle, so it's handed out one unit at a time
        // (highest weight first, for determinism) to any entry that
        // still has headroom under its own demand, until either the
        // remainder or every entry's headroom is exhausted.
        let mut shares: Vec<(usize, u64, u64)> = still_active
            .iter()
            .map(|&(idx, demand, weight)| {
                let share = ((remaining * weight as u64) / total_weight).min(demand);
                (idx, demand, share)
            })
            .collect();
        shares.sort_by(|a, b| b.2.cmp(&a.2).then(a.0.cmp(&b.0)));

        let allocated_sum: u64 = shares.iter().map(|&(_, _, s)| s).sum();
        let mut leftover = remaining.saturating_sub(allocated_sum);
        while leftover > 0 {
            let mut gave_any = false;
            for (_, demand, share) in shares.iter_mut() {
                if leftover == 0 {
                    break;
                }
                if *share < *demand {
                    *share += 1;
                    leftover -= 1;
                    gave_any = true;
                }
            }
            if !gave_any {
                break; // every entry is at its own demand cap
            }
        }

        for (idx, _, share) in shares {
            *granted.entry(idx).or_insert(0) += share;
        }
        break;
    }

    granted
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_example_weights_match_55s_own_table() {
        let w = WfqWeights::spec_example();
        assert_eq!(
            (w.control, w.interactive, w.normal, w.bulk, w.background),
            (8, 6, 4, 2, 1)
        );
    }

    #[test]
    fn unlimited_demand_splits_exactly_by_weight_ratio() {
        // Total weight 8+6+4+2+1=21; 210 bytes/sec divides evenly.
        let weights = WfqWeights::spec_example();
        let demands = [
            (WorkPriority::Control, 1_000_000),
            (WorkPriority::Interactive, 1_000_000),
            (WorkPriority::Normal, 1_000_000),
            (WorkPriority::Bulk, 1_000_000),
            (WorkPriority::Background, 1_000_000),
        ];
        let allocated = allocate_bandwidth(
            210,
            &demands,
            &weights,
            &CriticalPreemption::conservative_default(),
        );
        assert_eq!(allocated, vec![80, 60, 40, 20, 10]);
        assert_eq!(allocated.iter().sum::<u64>(), 210);
    }

    #[test]
    fn bulk_never_fully_starves_even_when_every_higher_tier_wants_everything() {
        // §56's exact warning, checked directly: strict priority would
        // give Bulk zero here; weighted fairness must not.
        let weights = WfqWeights::spec_example();
        let demands = [
            (WorkPriority::Control, u64::MAX),
            (WorkPriority::Interactive, u64::MAX),
            (WorkPriority::Normal, u64::MAX),
            (WorkPriority::Bulk, 1_000),
            (WorkPriority::Background, u64::MAX),
        ];
        let allocated = allocate_bandwidth(
            1000,
            &demands,
            &weights,
            &CriticalPreemption::conservative_default(),
        );
        assert!(allocated[3] > 0, "Bulk must receive a nonzero share");
    }

    #[test]
    fn a_tier_requesting_less_than_its_share_frees_the_remainder_for_others() {
        // Max-min property: Control only needs 5 bytes despite having
        // the highest weight — the other 95 bytes must go to the
        // still-demanding tiers, not sit unused.
        let weights = WfqWeights::spec_example();
        let demands = [
            (WorkPriority::Control, 5),
            (WorkPriority::Interactive, u64::MAX),
            (WorkPriority::Normal, u64::MAX),
            (WorkPriority::Bulk, u64::MAX),
            (WorkPriority::Background, u64::MAX),
        ];
        let allocated = allocate_bandwidth(
            100,
            &demands,
            &weights,
            &CriticalPreemption::conservative_default(),
        );
        assert_eq!(allocated[0], 5); // exactly its demand, no more
        assert_eq!(allocated.iter().sum::<u64>(), 100); // nothing left idle
    }

    #[test]
    fn critical_is_served_first_up_to_its_bound_then_remainder_is_shared() {
        let weights = WfqWeights::spec_example();
        let bound = CriticalPreemption {
            max_fraction_of_total: 0.5,
        };
        let demands = [
            (WorkPriority::Critical, u64::MAX),
            (WorkPriority::Bulk, u64::MAX),
        ];
        let allocated = allocate_bandwidth(1000, &demands, &weights, &bound);
        assert_eq!(allocated[0], 500); // capped at 50% of the link
        assert_eq!(allocated[1], 500); // the rest goes to Bulk, the only other demander
    }

    #[test]
    fn total_allocation_never_exceeds_link_capacity_across_many_shapes() {
        let weights = WfqWeights::spec_example();
        let bound = CriticalPreemption::conservative_default();
        let shapes: [&[(WorkPriority, u64)]; 3] = [
            &[
                (WorkPriority::Critical, 300),
                (WorkPriority::Bulk, 50),
                (WorkPriority::Background, 10),
            ],
            &[
                (WorkPriority::Normal, 1),
                (WorkPriority::Normal, 1),
                (WorkPriority::Normal, 1),
            ],
            &[
                (WorkPriority::Control, u64::MAX),
                (WorkPriority::Critical, u64::MAX),
            ],
        ];
        for demands in shapes {
            let allocated = allocate_bandwidth(1000, demands, &weights, &bound);
            assert!(allocated.iter().sum::<u64>() <= 1000);
        }
    }
}
