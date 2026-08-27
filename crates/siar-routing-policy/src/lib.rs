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
//!   network events itself).
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
//! - **Everything from roughly §43 onward that isn't listed above** —
//!   network transition event *handling* (only cache invalidation
//!   *hooks* exist, §42), transport setup cost/connection pooling
//!   (§44-47), security/privacy policy layering (§48-49, §108-116),
//!   LAN/Wi-Fi/Bluetooth/mesh-specific preference logic (§50-56) beyond
//!   what [`types::TransportKind`] names, size/deadline/expiry-aware
//!   routing (§60-62), traffic-type-specific route planning (§69-79),
//!   battery/thermal/platform integration (§82-90), multi-device route
//!   aggregation and group/broadcast routing (§171-175), storage-cost
//!   awareness (§176-178), and the remainder of this 200-section
//!   document not named above. This is a genuinely small slice of a
//!   very large spec — see §198 "Definition of Done" in the source
//!   document for the full bar this crate does not yet clear.
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
pub mod dispatch;
pub mod error;
pub mod failure;
pub mod metrics;
pub mod plan;
pub mod policy;
pub mod requirements;
pub mod resolve;
pub mod retry;
pub mod scoring;
pub mod types;

pub use cache::RouteCache;
pub use candidate::{PathCandidate, TransportEndpoint};
pub use error::RoutingError;
pub use failure::RouteFailureClass;
pub use metrics::{
    Bitrate, Confidence, EnergyCost, MeasuredValue, NetworkCost, PathMetrics, Ratio, SignalQuality,
    StabilityScore,
};
pub use plan::{plan_route, RoutePlan, RouteStrategy};
pub use policy::{HysteresisPolicy, PolicyWeights, RoutingPolicy, RoutingPolicyProfile};
pub use requirements::DeliveryRequirements;
pub use resolve::resolve_destination_devices;
pub use retry::RetryPolicy;
pub use scoring::{DefaultScorer, PathScorer, RouteScore, RouteScoreDelta, RoutingContext};
pub use types::{
    DeliveryClass, Destination, PathCapabilities, PathId, Priority, RouteHealth, TransportKind,
};
