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
//!
//! This is also where Part 03's own §63-65 "Queue Architecture"/
//! "Weighted Fair Scheduling"/"Backpressure" are accounted for: all
//! three are already real, just one layer down, in
//! `siar-protocol-ext`'s [`FairScheduler`]/[`siar_protocol_ext::backpressure::BoundedQueue`]
//! — per-class bounded queues (§63), weighted round-robin with a
//! strict-bounded Critical override (§64), and real rejection rather
//! than unbounded growth (§65's `QueueFull`, surfaced unchanged
//! through [`RouteDispatchQueue::enqueue`]'s own `Result`). §65's
//! alternative "or async wait" signaling isn't modeled — a caller
//! gets `QueueFull` and decides for itself whether to wait, retry, or
//! drop, same as [`siar_protocol_ext::backpressure::BoundedQueue::try_push`]'s
//! own doc comment already says of its callers.
//!
//! §66 "Per-Transport Queues" is new this round:
//! [`PerTransportDispatchQueue`] keeps one independent
//! [`RouteDispatchQueue`] (and so one independent [`FairScheduler`])
//! per [`crate::types::TransportKind`] — a stalled Bluetooth queue
//! genuinely cannot block a healthy Iroh one, because they aren't the
//! same queue.

use std::collections::HashMap;

use crate::plan::RoutePlan;
use crate::types::{Priority, TransportKind};
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
    #[allow(clippy::result_large_err)]
    pub fn enqueue(
        &mut self,
        priority: Priority,
        plan: RoutePlan,
        payload: T,
    ) -> Result<(), ((RoutePlan, T), QueueFull)> {
        self.scheduler
            .enqueue(traffic_priority_for(priority), (plan, payload))
    }

    /// Pops the next `(plan, payload)` pair to actually dial/send, per
    /// [`FairScheduler::next`]'s weighted-fair + bounded-critical-
    /// override selection.
    pub fn dispatch_next(&mut self) -> Option<(RoutePlan, T)> {
        self.scheduler.next()
    }

    pub fn is_empty(&self) -> bool {
        self.scheduler.is_empty()
    }
}

/// §66 "Per-Transport Queues": "Maintain separate bounded queues per
/// transport/session. This prevents stalled Bluetooth from blocking
/// healthy Iroh." One whole [`RouteDispatchQueue`] — priority tiers
/// and all — per [`TransportKind`], keyed off each plan's own
/// `primary.transport` at enqueue time. A caller running one consumer
/// task per transport calls [`Self::dispatch_next`] with that task's
/// own transport and never sees another transport's backlog at all,
/// which is the actual isolation §66 asks for (not just "keep the
/// data structures separate" but "a stall in one cannot be observed
/// from the other").
pub struct PerTransportDispatchQueue<T> {
    per_tier_capacity: usize,
    queues: HashMap<TransportKind, RouteDispatchQueue<T>>,
}

impl<T> PerTransportDispatchQueue<T> {
    pub fn new(per_tier_capacity: usize) -> Self {
        Self {
            per_tier_capacity,
            queues: HashMap::new(),
        }
    }

    /// Routes to `plan.primary.transport`'s own queue, creating it on
    /// first use — a transport that has never been enqueued to yet
    /// doesn't pre-allocate a queue (and so doesn't pre-allocate
    /// [`FairScheduler`]'s own six per-tier [`siar_protocol_ext::backpressure::BoundedQueue`]s)
    /// until something actually needs it.
    #[allow(clippy::result_large_err)]
    pub fn enqueue(
        &mut self,
        priority: Priority,
        plan: RoutePlan,
        payload: T,
    ) -> Result<(), ((RoutePlan, T), QueueFull)> {
        let transport = plan.primary.transport;
        let capacity = self.per_tier_capacity;
        self.queues
            .entry(transport)
            .or_insert_with(|| RouteDispatchQueue::new(capacity))
            .enqueue(priority, plan, payload)
    }

    /// `None` both when `transport` has no queue at all yet (nothing
    /// has ever been enqueued for it) and when its queue exists but is
    /// currently empty — indistinguishable to a caller that only cares
    /// "is there something to dispatch right now," which is the only
    /// question this method answers.
    pub fn dispatch_next(&mut self, transport: TransportKind) -> Option<(RoutePlan, T)> {
        self.queues.get_mut(&transport)?.dispatch_next()
    }

    pub fn is_empty(&self) -> bool {
        self.queues.values().all(RouteDispatchQueue::is_empty)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::candidate::{PathCandidate, TransportEndpoint};
    use crate::plan::RouteStrategy;
    use crate::types::{PathCapabilities, PathId, RouteHealth, TransportKind};

    fn plan_for(_label: &str) -> RoutePlan {
        plan_with_transport(TransportKind::IrohDirect)
    }

    fn plan_with_transport(transport: TransportKind) -> RoutePlan {
        let primary = PathCandidate {
            path_id: PathId::new(),
            transport,
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
                metered: crate::types::MeteredState::Unknown,
                roaming: crate::types::RoamingState::Unknown,
            },
            health: RouteHealth::Healthy,
            underlay: None,
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
        queue
            .enqueue(Priority::Background, plan_for("bg"), "bulk-sync")
            .unwrap();
        queue
            .enqueue(Priority::Critical, plan_for("sos"), "emergency-alert")
            .unwrap();

        let (_, payload) = queue.dispatch_next().unwrap();
        assert_eq!(payload, "emergency-alert");
    }

    #[test]
    fn enqueue_past_capacity_returns_the_rejected_pair_rather_than_dropping() {
        let mut queue: RouteDispatchQueue<u32> = RouteDispatchQueue::new(1);
        queue.enqueue(Priority::Normal, plan_for("a"), 1).unwrap();

        let err = queue
            .enqueue(Priority::Normal, plan_for("b"), 2)
            .unwrap_err();
        let ((_, rejected_payload), _) = err;
        assert_eq!(rejected_payload, 2);
    }

    #[test]
    fn a_stalled_transports_full_queue_does_not_affect_a_different_transport() {
        let mut queue: PerTransportDispatchQueue<u32> = PerTransportDispatchQueue::new(1);
        queue
            .enqueue(
                Priority::Normal,
                plan_with_transport(TransportKind::BluetoothClassic),
                1,
            )
            .unwrap();
        // Bluetooth's Normal tier is now at capacity (1) — a second
        // enqueue for the *same* transport is rejected...
        assert!(queue
            .enqueue(
                Priority::Normal,
                plan_with_transport(TransportKind::BluetoothClassic),
                2,
            )
            .is_err());
        // ...but Iroh's own queue is untouched and accepts normally.
        assert!(queue
            .enqueue(
                Priority::Normal,
                plan_with_transport(TransportKind::IrohDirect),
                3
            )
            .is_ok());
    }

    #[test]
    fn dispatch_next_only_returns_items_enqueued_for_that_transport() {
        let mut queue: PerTransportDispatchQueue<&str> = PerTransportDispatchQueue::new(16);
        queue
            .enqueue(
                Priority::Normal,
                plan_with_transport(TransportKind::IrohDirect),
                "iroh-item",
            )
            .unwrap();
        queue
            .enqueue(
                Priority::Normal,
                plan_with_transport(TransportKind::BluetoothClassic),
                "bt-item",
            )
            .unwrap();

        let (_, iroh_payload) = queue.dispatch_next(TransportKind::IrohDirect).unwrap();
        assert_eq!(iroh_payload, "iroh-item");
        // Iroh's queue is now empty, but Bluetooth's own item is
        // still there, untouched.
        assert!(queue.dispatch_next(TransportKind::IrohDirect).is_none());
        let (_, bt_payload) = queue
            .dispatch_next(TransportKind::BluetoothClassic)
            .unwrap();
        assert_eq!(bt_payload, "bt-item");
    }

    #[test]
    fn a_transport_that_was_never_enqueued_to_has_no_queue_and_dispatches_nothing() {
        let mut queue: PerTransportDispatchQueue<u32> = PerTransportDispatchQueue::new(16);
        assert!(queue.dispatch_next(TransportKind::Dtn).is_none());
        assert!(queue.is_empty());
    }
}
