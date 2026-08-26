//! Priority-aware dispatch queueing, bridging this crate's own
//! [`Priority`]/[`RoutePlan`] to `siar-protocol-ext`'s already-real
//! `FairScheduler`/`BoundedQueue` (per [[resilient-mesh]] project
//! memory: "no queueing/backpressure integration with
//! siar-protocol-ext's now-real BoundedQueue/FairScheduler" was the
//! standing gap this module closes).
//!
//! Neither crate's spec text names this integration directly — Part
//! 03 stops at producing a [`RoutePlan`] (this crate's own lib.rs doc
//! comment: "this crate stops at producing a `RoutePlan`, it doesn't
//! dial anything"), and Part 01's §21-22 fair-scheduling text is
//! written generically ("traffic priority"), not in terms of Part
//! 03's specific [`Priority`] enum. The integration is nonetheless
//! real, not speculative: both types already exist as concrete,
//! tested code in this workspace, and a caller that has both a
//! [`RoutePlan`] (from [`crate::plan::plan_route`]) and a payload to
//! send has no way today to actually queue that payload for
//! priority-fair dispatch — this module is the missing connective
//! tissue, not a new abstraction layered on top of either crate's own
//! design.

use crate::plan::RoutePlan;
use crate::types::Priority;
use siar_protocol_ext::backpressure::QueueFull;
use siar_protocol_ext::lifecycle::TrafficPriority;
use siar_protocol_ext::scheduler::FairScheduler;

/// Maps this crate's application-facing [`Priority`] (§8 "Priority
/// Levels" from `requirements.rs`'s own doc comment) onto
/// `siar-protocol-ext`'s six-tier [`TrafficPriority`] (§21).
///
/// [`TrafficPriority::Control`] has no counterpart in [`Priority`] —
/// it names *protocol-internal* traffic (HELLO/HELLO_ACK,
/// negotiation frames), never something an application-level
/// [`DeliveryRequirements`](crate::requirements::DeliveryRequirements)
/// would ask for, so this mapping is necessarily onto a strict
/// 5-of-6 subset of the target tiers rather than all six — that gap
/// belongs to whichever code emits the protocol's own control
/// frames, not to this function.
pub fn traffic_priority_for(priority: Priority) -> TrafficPriority {
    match priority {
        Priority::Critical => TrafficPriority::Critical,
        Priority::High => TrafficPriority::Interactive,
        Priority::Normal => TrafficPriority::Normal,
        Priority::Low => TrafficPriority::Bulk,
        Priority::Background => TrafficPriority::Background,
    }
}

/// A priority-fair dispatch queue for `(RoutePlan, payload)` pairs,
/// built directly on [`FairScheduler`] rather than reimplementing its
/// weighted-round-robin/bounded-emergency-override logic (see that
/// type's own doc comment for why naive highest-priority-first
/// scheduling is wrong here).
pub struct RouteDispatchQueue<T> {
    scheduler: FairScheduler<(RoutePlan, T)>,
}

impl<T> RouteDispatchQueue<T> {
    pub fn new(per_tier_capacity: usize) -> Self {
        Self {
            scheduler: FairScheduler::with_default_weights(per_tier_capacity),
        }
    }

    /// Queues `payload` for dispatch over `plan`, at the tier
    /// [`traffic_priority_for`] derives from `priority`. Returns the
    /// rejected `(plan, payload)` pair, unchanged, if that tier's
    /// queue is already at [`FairScheduler`]'s bound (§20: every
    /// queue in `siar-protocol-ext` is bounded, no exceptions) —
    /// never silently drops or blocks.
    pub fn enqueue(
        &mut self,
        priority: Priority,
        plan: RoutePlan,
        payload: T,
    ) -> Result<(), ((RoutePlan, T), QueueFull)> {
        self.scheduler.enqueue(traffic_priority_for(priority), (plan, payload))
    }

    /// Pops the next `(plan, payload)` pair to actually dial/send, per
    /// [`FairScheduler::next`]'s weighted-fair + bounded-critical-
    /// override selection.
    pub fn dispatch_next(&mut self) -> Option<(RoutePlan, T)> {
        self.scheduler.next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::candidate::{PathCandidate, TransportEndpoint};
    use crate::plan::RouteStrategy;
    use crate::types::{PathCapabilities, PathId, RouteHealth, TransportKind};

    fn plan_for(_label: &str) -> RoutePlan {
        let primary = PathCandidate {
            path_id: PathId::new(),
            transport: TransportKind::IrohDirect,
            peer: siar_domain::DeviceId::new(),
            endpoint: TransportEndpoint(Vec::new()),
            metrics: crate::metrics::PathMetrics::unknown(),
            capabilities: PathCapabilities {
                reliable_stream: true,
                datagram: false,
                large_files: false,
                realtime_media: false,
                peer_discovery: false,
                store_and_forward: false,
                metered: false,
            },
            health: RouteHealth::Healthy,
        };
        RoutePlan {
            primary,
            fallbacks: Vec::new(),
            replicas: Vec::new(),
            strategy: RouteStrategy::Single,
        }
    }

    #[test]
    fn every_application_priority_maps_to_a_distinct_traffic_tier() {
        // Not strictly required by any spec text, but a mapping that
        // collapsed two distinct application priorities onto the same
        // wire tier would silently discard the distinction the caller
        // asked for — this guards against ever introducing that bug.
        let mapped: Vec<TrafficPriority> = [
            Priority::Critical,
            Priority::High,
            Priority::Normal,
            Priority::Low,
            Priority::Background,
        ]
        .into_iter()
        .map(traffic_priority_for)
        .collect();

        let mut deduped = mapped.clone();
        deduped.sort_by_key(|t| format!("{t:?}"));
        deduped.dedup();
        assert_eq!(mapped.len(), deduped.len());
    }

    #[test]
    fn critical_dispatches_before_background_regardless_of_enqueue_order() {
        let mut queue: RouteDispatchQueue<&str> = RouteDispatchQueue::new(16);
        queue.enqueue(Priority::Background, plan_for("bg"), "bulk-sync").unwrap();
        queue.enqueue(Priority::Critical, plan_for("sos"), "emergency-alert").unwrap();

        let (_, payload) = queue.dispatch_next().unwrap();
        assert_eq!(payload, "emergency-alert");
    }

    #[test]
    fn enqueue_past_capacity_returns_the_rejected_pair_rather_than_dropping() {
        let mut queue: RouteDispatchQueue<u32> = RouteDispatchQueue::new(1);
        queue.enqueue(Priority::Normal, plan_for("a"), 1).unwrap();

        let err = queue.enqueue(Priority::Normal, plan_for("b"), 2).unwrap_err();
        let ((_, rejected_payload), _) = err;
        assert_eq!(rejected_payload, 2);
    }
}
