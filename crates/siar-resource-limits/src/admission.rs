//! §22 "Bounded Queue Principle" (backpressure never blocks forever),
//! §23 "Backpressure Semantics", §24 "Admission Control", §25
//! "Admission Controller", §26 "Resource Request", §27 "Resource
//! Owner".
//!
//! §23 names `AdmissionResult` with three reason payloads
//! (`DeferredReason`/`RejectReason`/`DropReason`) but never defines
//! any of the three enums — this module does, grounded in the
//! concrete guidance elsewhere in Part 08's own text rather than
//! invented from nothing (each variant's doc comment cites where).
//! §25's `AdmissionController` trait is sketched against a
//! `&ResourceSnapshot` (§80) this pass doesn't build — deferred there
//! for now (see `lib.rs`'s own gap list) in favor of a function that
//! decides against the [`crate::types::ResourceBudget`] this crate
//! already has: [`admit`], real and tested, not a trait stub.

use crate::types::ResourceBudget;
use serde::{Deserialize, Serialize};
use siar_domain::DeviceId;
use siar_protocol_ext::identifier::ProtocolId;
use siar_protocol_ext::lifecycle::TrafficPriority;

/// §19's `WorkPriority` is never given its own variant list separate
/// from `siar-protocol-ext`'s six-tier `TrafficPriority` — this spec's
/// own §19 text explicitly frames it as aligning with the other parts'
/// priority schemes rather than introducing a new one, so this crate
/// reuses that type directly instead of duplicating an identical enum
/// under a different name (the same reasoning `siar_capability::
/// extension::ExtensionNegotiator` already used for reusing
/// `CapabilitySet` instead of inventing a parallel type).
pub type WorkPriority = TrafficPriority;

/// §27, verbatim.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceOwner {
    Core,
    Messaging,
    Files,
    Dtn,
    Calls,
    Extension(ProtocolId),
    Peer(DeviceId),
}

/// Not defined anywhere in Part 08's text beyond being named as a
/// `ResourceRequest` field (§26) — `siar_routing_policy::requirements::
/// DeliveryRequirements.allow_metered` is the closest existing concept
/// in this workspace, a plain bool. This crate widens that to a
/// tri-state rather than reusing the bool directly, because §3
/// elsewhere in this very spec explicitly warns against collapsing
/// network conditions into "one generic flag" — and "don't know
/// whether the current connection is metered" (common enough on
/// mobile handoff) is a real third state a bool can't represent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BandwidthClass {
    Unmetered,
    Metered,
    Unknown,
}

/// §26, verbatim field list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceRequest {
    pub owner: ResourceOwner,
    pub priority: WorkPriority,
    pub memory: u64,
    pub storage: u64,
    pub streams: u32,
    pub connections: u32,
    pub bandwidth_class: BandwidthClass,
    pub durable: bool,
}

/// Grounded in §58 ("Realtime media frames become useless after
/// deadline... drop stale frame, not block and accumulate seconds of
/// latency").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DropReason {
    /// §58's own case: the resource wasn't available in time and the
    /// work has no value once delayed.
    Stale,
    /// §59-60 "Coalescing": a newer update for the same slot made this
    /// one moot before it was ever admitted.
    Superseded,
}

/// Grounded in §22's "bounded queue... reject or shed, not grow
/// forever" framing, applied to durable (queueable) work that doesn't
/// fit right now but isn't disposable the way §58's realtime frames
/// are.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeferredReason {
    /// Fits within the resource's hard total (§8's budget), just not
    /// within what's currently free — worth waiting for.
    AwaitingBudget,
}

/// Grounded in §24 "Admission Control"'s own worked pipeline (global
/// budget, then peer quota, then extension quota, in that order —
/// §28's hierarchical accounting) and §30's "never make trusted peers
/// unlimited."
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RejectReason {
    /// The request alone exceeds the resource's hard total (§8) —
    /// it could never be satisfied no matter how much else is freed,
    /// so retrying later is pointless; the caller must shrink the
    /// request instead.
    OverGlobalBudget,
}

/// §23's own three-variant sketch, with each reason enum now real
/// rather than left as a bare name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AdmissionResult {
    Accepted,
    Deferred(DeferredReason),
    Rejected(RejectReason),
    Dropped(DropReason),
}

/// The real decision §25's `AdmissionController` trait leaves as a
/// method signature. Takes both the resource's hard ceiling (`total`)
/// and its currently-free capacity (`remaining`) — distinguishing "can
/// never fit" (§24/§8: reject outright, no retry) from "doesn't fit
/// *right now*" (§22/§58: queue it if durable, drop it if not) is the
/// whole reason this function takes two [`ResourceBudget`]s instead of
/// one.
///
/// Bandwidth (`request.bandwidth_class`) is deliberately not checked
/// against `ResourceBudget::network_bytes_per_sec` here: a *rate*
/// isn't a one-shot pool to admit against the way memory/storage/
/// streams/connections are — that's exactly what
/// [`crate::token_bucket::TokenBucket`] already exists for, and
/// duplicating a weaker version of that check here would give the
/// same resource two different, potentially-disagreeing gatekeepers.
pub fn admit(
    request: &ResourceRequest,
    remaining: &ResourceBudget,
    total: &ResourceBudget,
) -> AdmissionResult {
    let exceeds_total = request.memory > total.memory_bytes
        || request.storage > total.storage_bytes
        || request.streams > total.max_streams
        || request.connections > total.max_connections;
    if exceeds_total {
        return AdmissionResult::Rejected(RejectReason::OverGlobalBudget);
    }

    let fits_remaining = request.memory <= remaining.memory_bytes
        && request.storage <= remaining.storage_bytes
        && request.streams <= remaining.max_streams
        && request.connections <= remaining.max_connections;
    if fits_remaining {
        return AdmissionResult::Accepted;
    }

    if request.durable {
        AdmissionResult::Deferred(DeferredReason::AwaitingBudget)
    } else {
        // §58: don't block and accumulate latency for disposable work
        // — shed it now instead of holding it in a queue that only
        // grows.
        AdmissionResult::Dropped(DropReason::Stale)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generous_budget() -> ResourceBudget {
        ResourceBudget {
            memory_bytes: 1_000_000,
            storage_bytes: 1_000_000,
            network_bytes_per_sec: None,
            max_connections: 100,
            max_streams: 100,
            max_tasks: 100,
        }
    }

    fn request(memory: u64, durable: bool) -> ResourceRequest {
        ResourceRequest {
            owner: ResourceOwner::Messaging,
            priority: WorkPriority::Normal,
            memory,
            storage: 0,
            streams: 0,
            connections: 0,
            bandwidth_class: BandwidthClass::Unknown,
            durable,
        }
    }

    #[test]
    fn admits_a_request_that_comfortably_fits_remaining_capacity() {
        let budget = generous_budget();
        let result = admit(&request(1_000, true), &budget, &budget);
        assert_eq!(result, AdmissionResult::Accepted);
    }

    #[test]
    fn rejects_a_request_that_exceeds_the_hard_total_outright() {
        let total = generous_budget();
        let result = admit(&request(2_000_000, true), &total, &total);
        assert_eq!(
            result,
            AdmissionResult::Rejected(RejectReason::OverGlobalBudget)
        );
    }

    #[test]
    fn defers_a_durable_request_that_fits_total_but_not_remaining() {
        let total = generous_budget();
        let mut remaining = total.clone();
        remaining.memory_bytes = 500; // almost exhausted right now
        let result = admit(&request(1_000, true), &remaining, &total);
        assert_eq!(
            result,
            AdmissionResult::Deferred(DeferredReason::AwaitingBudget)
        );
    }

    #[test]
    fn drops_a_non_durable_request_that_fits_total_but_not_remaining() {
        // §58's exact policy: disposable/realtime work gets shed, not
        // queued, when it can't be admitted right now.
        let total = generous_budget();
        let mut remaining = total.clone();
        remaining.memory_bytes = 500;
        let result = admit(&request(1_000, false), &remaining, &total);
        assert_eq!(result, AdmissionResult::Dropped(DropReason::Stale));
    }

    #[test]
    fn each_dimension_is_checked_independently_not_just_memory() {
        let total = generous_budget();
        let mut remaining = total.clone();
        remaining.max_streams = 0;

        let mut req = request(0, true);
        req.streams = 1;
        assert_eq!(
            admit(&req, &remaining, &total),
            AdmissionResult::Deferred(DeferredReason::AwaitingBudget)
        );
    }

    #[test]
    fn work_priority_is_the_same_type_as_traffic_priority() {
        // Compiles only if the type alias really does point at
        // `siar_protocol_ext::lifecycle::TrafficPriority` rather than
        // a look-alike local enum.
        let _: WorkPriority = siar_protocol_ext::lifecycle::TrafficPriority::Critical;
    }
}
