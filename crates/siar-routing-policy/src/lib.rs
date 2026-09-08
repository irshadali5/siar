#![forbid(unsafe_code)]

//! siar-routing-policy: a first slice of "Part 03 — Transport Routing
//! Policy Engine Architecture" (the third of three architecture
//! documents supplied this pass; Part 01 has its own crate,
//! `siar-protocol-ext`; Part 02 has `siar-identity-multidevice`, which
//! this crate depends on for real — see [`resolve`]). Unlike Parts 01
//! and 02, this workspace had **no** crate built against Part 03's
//! specific spec text before this session — the existing
//! `siar-routing` crate is a related but independently-evolved system
//! built against a different, earlier document ("next.md"), and is
//! left untouched here (see this comment's own closing section for how
//! the two relate).
//!
//! ## Scope: §196 "Initial Production Scope" + §197 Phase 1/3/4, partial
//!
//! Implemented, with real tests exercising actual logic (not just type
//! shapes):
//!
//! - [`types`], [`metrics`], [`requirements`], [`candidate`], [`plan`]
//!   — §197 Phase 1 ("Types and policy"): `DeliveryRequirements`,
//!   `PathCandidate`, `PathMetrics`, `RoutingPolicy`, `RoutePlan`, named
//!   exactly as §5 "Main Abstractions" lists them.
//! - [`scoring`] — §197 Phase 3 ("Scoring"): hard constraints (§25 step
//!   1), a real weighted scorer implementing §24's formula for the
//!   terms this crate can actually compute (see that module's own doc
//!   comment for the two terms — congestion, failure penalty — it
//!   doesn't model).
//! - [`plan::plan_route`] and [`policy`] — §197 Phase 4 ("Failover"):
//!   §25's full four-step evaluation order end to end, including §34/§35
//!   stickiness/hysteresis, plus [`retry`]'s backoff policy and
//!   [`failure`]'s failure classification (§36-39).
//! - [`cache`] — §41 "Route Cache" with §42's invalidation triggers as
//!   callable methods (this crate doesn't listen for the underlying OS/
//!   network events itself), plus §43 "Route Re-Evaluation"'s targeted
//!   per-transport invalidation (`RouteCache::invalidate_transport`),
//!   narrower than §42's already-existing "clear everything."
//! - [`setup`] — §44 "Transport Setup Cost", §46 "Connection Pool
//!   Integration": an ordered `SetupCost` per transport, adjusted by a
//!   caller-reported `ConnectionPoolState`, now a real weighted term in
//!   [`scoring::DefaultScorer`] (previously that formula had nothing
//!   for setup cost at all).
//! - [`security`] — §47 "Peer Session Abstraction", §48 "Security
//!   Constraints": a smart-constructor `AuthenticatedSession` that can
//!   only be built by re-checking a candidate's peer against Part 02's
//!   `TrustedAccountStore`, plus a list-level elimination function —
//!   defense-in-depth on top of [`resolve`]'s own device-list-stage
//!   trust filtering.
//! - [`privacy`] — §49 "Privacy Policy", §50 "Direct vs Relay
//!   Preference", and §52 "Wi-Fi Direct/Aware"'s setup-threshold gate
//!   (also covers §53's Bluetooth-pairing side of that same threshold).
//!   §51/§54/§56 are accounted for in that module's own doc comment
//!   without needing new functions; §55 "Mesh Forwarding" is named
//!   there as a real, un-implemented gap rather than papered over.
//! - [`descriptor`] — §57 "Delivery Semantics" (closed via two new
//!   [`requirements::DeliveryRequirements`] constructors,
//!   `typing_indicator`/`file_chunk`, alongside the two constructors
//!   that already existed), §58 "Operation Descriptor", §59 "Content
//!   Class" — named exactly as the spec lists them.
//! - [`estimate`] — §60 "Size-Aware Routing" (a real, labeled-as-
//!   coarse `completion_time ≈ setup + bytes / bandwidth` estimate)
//!   and §61 "Deadline-Aware Routing" (hard elimination of candidates
//!   whose estimated completion exceeds `max_latency_millis`, rather
//!   than queuing them indefinitely).
//! - §62 "Expiry-Aware Routing" — [`requirements::DeliveryRequirements::has_expired`]
//!   plus [`retry::RetryPolicy::allows_attempt_at`], which combines it
//!   with the existing attempt-count cap. Not its own module — both
//!   pieces extend types that already existed.
//! - [`dispatch`] — beyond its existing priority-fair bridge to
//!   `siar-protocol-ext`, this is also where §63 "Queue Architecture",
//!   §64 "Weighted Fair Scheduling", and §65 "Backpressure" are
//!   accounted for (all three are already real one layer down, in
//!   `siar-protocol-ext`'s own `FairScheduler`/`BoundedQueue` — see
//!   this module's own doc comment), plus §66 "Per-Transport Queues"
//!   (`PerTransportDispatchQueue`, new this round): one independent
//!   dispatch queue per [`types::TransportKind`], so a stalled
//!   Bluetooth backlog cannot block a healthy Iroh one.
//! - [`fairness`] — §67 "Per-Peer Fairness", §68 "Per-Extension
//!   Fairness": one generic `RoundRobinFairQueue<K, T>` covering both,
//!   composable rather than wired into [`dispatch`] (see that module's
//!   own doc comment for why: `siar-protocol-ext`'s per-tier queue
//!   implementation is fixed and not swappable from this crate).
//! - [`resolve`] — §16/§17 "Destination Resolution"/"Account-Level
//!   Routing", the one piece of this crate that reaches into another
//!   real crate (`siar-identity-multidevice`) rather than staying
//!   self-contained, exactly because §16 says "Part 02 provides device
//!   membership" and Part 02 now has a real crate to mean that with.
//! - [`dispatch`] — a second real cross-crate integration, this time
//!   with `siar-protocol-ext`'s already-built `FairScheduler`/
//!   `BoundedQueue` (its own §21-22 "Fair Scheduling"): maps this
//!   crate's [`types::Priority`] onto that crate's `TrafficPriority`
//!   and queues `(RoutePlan, payload)` pairs for priority-fair
//!   dispatch. Not itself named by either spec (see that module's own
//!   doc comment), but a real gap this closes: previously nothing
//!   connected an application's delivery priority to actual queueing
//!   behavior once a route was chosen.
//!
//! §197's own Phase 5 ("DTN") is touched only incidentally — a `Dtn`
//! [`types::TransportKind`] variant exists and competes in scoring like
//! any other transport, but nothing here implements actual bundle
//! persistence or opportunistic forwarding (§23's own "persist
//! operation → select DTN bundle policy → wait for peer encounter →
//! forward opportunistically" pipeline).
//!
//! ## What's explicitly NOT here
//!
//! - **§197 Phase 2 (candidate collection).** Nothing here integrates
//!   Iroh, LAN, Bluetooth, or Wi-Fi discovery — [`candidate::PathCandidate`]
//!   is a value type a caller constructs from real discovery data this
//!   crate has no dependency on producing.
//! - **§197 Phase 6 (resource scheduling)** — §63-68 (queue
//!   architecture, weighted fair scheduling, backpressure, per-transport/
//!   peer/extension fairness) — [`dispatch`] now covers priority-tier
//!   fair scheduling and backpressure end to end (via
//!   `siar-protocol-ext`'s `FairScheduler`/`BoundedQueue`); per-transport
//!   and per-peer/extension fairness weighting is still not attempted.
//! - **§197 Phase 7 (diagnostics/testing)** — §95-99 (route
//!   diagnostics, path visualization, decision explainability, metrics
//!   collection/privacy), §124-127 (simulated routing/property/chaos/
//!   failover tests beyond this crate's own unit tests) — not
//!   attempted.
//! - **Everything from roughly §69 onward that isn't listed above** —
//!   traffic-type-specific route planning (§69-79), battery/thermal/
//!   platform integration (§82-90), multi-device route aggregation and
//!   group/broadcast routing (§171-175), storage-cost awareness
//!   (§176-178), and the remainder of this 200-section document not
//!   named above. §55 "Mesh Forwarding"'s richer candidate
//!   representation (next hop, route utility, hop budget, relay trust
//!   policy) also remains unimplemented — see [`privacy`]'s own doc
//!   comment; §108-116's deeper security/privacy layering beyond §48/
//!   §49's basic version here is likewise untouched. This is a
//!   genuinely small slice of a very large spec — see §198 "Definition
//!   of Done" in the source document for the full bar this crate does
//!   not yet clear.
//!
//! ## Relationship to the existing `siar-routing` crate
//!
//! `siar-routing` (this workspace, built earlier, against "next.md")
//! already has real path scoring, link health tracking, and a
//! scheduler covering similar conceptual ground — device routes, path
//! scoring, link health — under different type names and a different
//! design. Neither crate depends on or replaces the other. Reconciling
//! them (migrating one onto the other, keeping both for different
//! contexts, or retiring one) is a genuine product/architecture
//! decision this crate does not make unilaterally — the same posture
//! `siar-identity-multidevice` already takes toward the existing
//! `siar_crypto::device_cert` system, for the same reason.

pub mod cache;
pub mod candidate;
pub mod descriptor;
pub mod dispatch;
pub mod error;
pub mod estimate;
pub mod failure;
pub mod fairness;
pub mod metrics;
pub mod plan;
pub mod policy;
pub mod privacy;
pub mod requirements;
pub mod resolve;
pub mod retry;
pub mod scoring;
pub mod security;
pub mod setup;
pub mod types;

pub use cache::RouteCache;
pub use candidate::{PathCandidate, TransportEndpoint};
pub use descriptor::{ByteCount, ContentClass, OperationDescriptor, OperationId};
pub use error::RoutingError;
pub use estimate::{
    completion_time_millis, eliminate_deadline_exceeding_candidates, exceeds_deadline,
};
pub use failure::RouteFailureClass;
pub use metrics::{
    Bitrate, Confidence, EnergyCost, MeasuredValue, NetworkCost, PathMetrics, Ratio, SignalQuality,
    StabilityScore,
};
pub use plan::{plan_route, RoutePlan, RouteStrategy};
pub use policy::{HysteresisPolicy, PolicyWeights, RoutingPolicy, RoutingPolicyProfile};
pub use privacy::{
    direct_preference_bonus, eliminate_privacy_violations, eliminate_unjustified_expensive_setup,
    justifies_expensive_setup, passes_privacy_policy, PrivacyPolicy,
};
pub use requirements::DeliveryRequirements;
pub use resolve::resolve_destination_devices;
pub use retry::RetryPolicy;
pub use scoring::{DefaultScorer, PathScorer, RouteScore, RouteScoreDelta, RoutingContext};
pub use security::{authorize_candidate, eliminate_untrusted_candidates, AuthenticatedSession};
pub use setup::{effective_setup_cost, static_setup_cost, ConnectionPoolState, SetupCost};
pub use types::{
    DeliveryClass, Destination, PathCapabilities, PathId, Priority, RouteHealth, TransportKind,
};
