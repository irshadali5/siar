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
//! - §69 "Route Planning for Messaging", §70 "Route Planning for
//!   Files", §72 "Route Planning for Emergency" — verified, not
//!   separately implemented: this crate's own `plan` module test suite
//!   has integration tests
//!   checking that [`plan::plan_route`]'s existing scoring actually
//!   produces the compositions/orderings these sections describe
//!   (where that's robustly true without fabricated measurements —
//!   see those tests' own doc comments for the one case, DTN's
//!   relative position in §69, this crate deliberately does *not*
//!   claim to get right without real caller-supplied data).
//! - [`quality`] — §71 "Route Planning for Calls"'s "routing supplies
//!   path quality signals to media adaptation" half (the video →
//!   audio → voice-message fallback ladder itself is a media-
//!   adaptation decision, out of this crate's scope; the *selection*
//!   half of §71 was already covered by existing scoring before this
//!   round).
//! - [`diversity`] — §73 "Path Diversity", §74 "Underlay Group": a
//!   real `UnderlayId` newtype (not the bare unit-struct stub §74's
//!   own code block shows — see this module's own doc comment),
//!   `are_diverse`/`group_by_underlay`, and `most_diverse_fallback`,
//!   now actually used by [`plan::plan_route`] to pick which fallback
//!   becomes a `Redundant` plan's replica.
//! - §75-78 "Multipath Chunk Scheduler"/"Path Collapse"/"Duplicate
//!   Chunk Handling"/"Realtime Multipath" — read and deliberately not
//!   implemented; the spec itself names all four as future/optional/
//!   advanced work, not v1, matching [`plan::plan_route`]'s own doc
//!   comment on why it never produces `RouteStrategy::Multipath`.
//! - [`metrics::CongestionState`] plus two new [`metrics::PathMetrics`]
//!   fields — §79 "Congestion Signals," closing a gap
//!   [`scoring::DefaultScorer`]'s own doc comment named since this
//!   crate's first phase ("the two it doesn't: congestion, failure
//!   penalty"). Only congestion; failure penalty still needs failure
//!   *history* this crate keeps no record of. "RTT trend" (also named
//!   by §79) is deliberately not a field — a trend needs samples over
//!   time this crate has no clock to collect (see this crate's own top
//!   doc comment on scope).
//! - [`stability`] — §80 "Route Stability Score":
//!   `derive_stability_score` turns connection lifetime/failure rate/
//!   path changes/timeout count into the [`metrics::StabilityScore`]
//!   [`scoring::DefaultScorer`] already consumes — the half of §80
//!   that didn't already exist (the other half, "a path with slightly
//!   worse RTT but much better stability may win," was already true of
//!   existing per-term weighting).
//! - §81 "Cold vs Warm Path" — already covered before this round:
//!   `warm` is exactly [`setup::ConnectionPoolState::Active`], which
//!   [`setup::effective_setup_cost`] already collapses to
//!   [`setup::SetupCost::Cheap`] regardless of the transport's own
//!   static cost (§46, this crate's own round-2 work) — precisely
//!   §81's "warm path gets a setup-cost advantage."
//! - [`types::MeteredState`], [`types::RoamingState`] — §82 "Metered
//!   Networks," §83 "Roaming": real tri-state enums (not the `bool`
//!   [`candidate::PathCandidate::capabilities`] used to carry), each
//!   with an `Unknown` case a platform that hasn't reported yet can
//!   express, wired into [`scoring::passes_hard_constraints`] via two
//!   new functions that both treat `Unknown` conservatively (same
//!   direction as [`metrics::Confidence::Stale`]'s own "don't trust
//!   what you can't currently verify" reasoning) — `Unknown` metered
//!   state is blocked exactly like confirmed-metered; `Unknown`
//!   roaming state is blocked for bulk transfers exactly like
//!   confirmed-roaming. §82's own "do not use one global allow/deny"
//!   was already satisfied structurally by `allow_metered` being a
//!   per-`DeliveryRequirements` field, not a crate-wide setting — this
//!   round's real fix was that [`requirements::DeliveryRequirements::file_chunk`]
//!   had `allow_metered: true`, contradicting §82's own "block bulk"
//!   worked example; now corrected, alongside a new
//!   `allow_roaming_bulk` field for §83's own "allow cellular but
//!   forbid roaming bulk transfer" example specifically.
//! - §84 "Battery Cost" — already covered before this round by
//!   [`metrics::EnergyCost`] (see that type's own doc comment for why
//!   its variant names differ cosmetically from §84's own list without
//!   being a gap).
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
//! - [`platform`] — §85 "Battery-Aware Inputs" (`DeviceState`), §87
//!   "Platform Policy Integration" (see that module's own doc comment
//!   for how §87's six named signals map across this crate — three
//!   already existed, three land in this module and [`acquisition`]).
//! - [`acquisition`] — §86 "Background Restrictions"
//!   (`is_currently_usable`/`eliminate_background_restricted`, now
//!   actually called by [`plan::plan_route`]), §89 "Passive vs Active
//!   Candidates" (`CandidateState`, transcribed exactly from that
//!   section's own code block, now a real [`policy::PolicyWeights::candidate_state`]
//!   scoring term).
//! - [`discovery`] — §88 "Path Acquisition", §90 "Discovery Budget":
//!   a real, stateful `DiscoveryBudget` (sliding window + cooldown, a
//!   caller persists it across calls) plus `discovery_permitted`,
//!   which combines the budget with [`platform::DeviceState`] —
//!   thermal-critical blocks unconditionally (a hardware safety
//!   margin, no priority override); battery-saver blocks unless
//!   `Priority::Critical`, the same override shape
//!   [`privacy::justifies_expensive_setup`] (§52) already uses.
//! - [`resilience`] — §91 "Route Escalation Ladder"
//!   (`EscalationStage`/`escalation_stage_of`, built entirely from
//!   existing types: [`acquisition::CandidateState`] for stages 1-2,
//!   [`setup::SetupCost`] for the 3-vs-4 split, `TransportKind::Dtn`
//!   for stage 5), §92 "Timeout by Stage"
//!   (`timeout_millis_for_stage`), §93 "Hedged Requests"
//!   (`HedgePolicy`/`hedge_policy_for`, now a real
//!   [`plan::RouteStrategy::Hedged`] [`plan::plan_route`] can actually
//!   produce, with `RoutePlan::hedge_delay_millis` carrying the
//!   delay), §95 "Route Diagnostics" (`RouteDiagnostics`/`diagnose`,
//!   scoped to the two rejection checks this crate can run without an
//!   external policy object — see that module's own doc comment for
//!   why security/privacy checks aren't included). §94
//!   "Deduplication Requirement" has no function of its own — see
//!   [`descriptor::OperationId`]'s own doc comment, extended this
//!   round to cover `Hedged` alongside `Redundant`. §96 "Path
//!   Visualization" is explicitly deferred by the spec itself to
//!   "Part 18" — nothing to implement yet.
//! - [`explain`] — §97 "Route Decision Explainability"
//!   (`RouteReason`, transcribed exactly, now a real
//!   `RoutePlan::reason` field), §98 "Metrics Collection"
//!   (`RouteMetricEvent`/`metric_events_for` — event classification
//!   only, no counters kept, since this crate has no history of its
//!   own; see that module's own doc comment for which of §98's named
//!   metrics have no equivalent here at all, and why), §100 "Routing
//!   State Store" (`RouteHint`/`hint_from_plan`), §101 "Startup
//!   Behavior" (`revalidate_hint` — the one step of that section's own
//!   three-step startup sequence that's actually this crate's to
//!   implement). §99 "Privacy of Metrics" needed no function: it's
//!   enforced by construction, not filtering — see [`explain::RouteMetricEvent`]'s
//!   own doc comment for why there's no field to redact in the first
//!   place. §102 "Suspend/Resume" turned out to already be covered
//!   entirely by composing existing pieces: [`cache::RouteCache::invalidate_all`]
//!   (invalidate stale metrics), a fresh [`plan::plan_route`] call
//!   (reassess active sessions), [`retry::RetryPolicy`] (retry durable
//!   operations), and [`discovery::DiscoveryBudget`]'s own cooldown
//!   (do not immediately launch every discovery mechanism) — nothing
//!   new needed. §103 "Process Death" is likewise already true by
//!   design, not by this round's addition — see [`plan::RoutePlan`]'s
//!   own doc comment for why it deliberately has no
//!   `Serialize`/`Deserialize` derive. §104 "Route Plan Lifetime" adds
//!   real fields: `RoutePlan::created_at_millis`/`valid_until_millis`,
//!   the latter derived from `RoutingPolicy::hysteresis.minimum_hold_millis`
//!   rather than a new invented duration.
//! - **Everything from roughly §105 onward that isn't listed above** —
//!   §105-107 (Path Authorization, Extension Capability Integration,
//!   Device Capability Integration) and beyond, multi-device route
//!   aggregation and group/broadcast routing (§171-175), storage-cost
//!   awareness (§176-178), and the remainder
//!   of this 200-section document not named above. §55 "Mesh
//!   Forwarding"'s richer candidate representation (next hop, route
//!   utility, hop budget, relay trust policy) also remains
//!   unimplemented — see [`privacy`]'s own doc comment. §108-116's
//!   deeper security/privacy layering beyond
//!   §48/§49's basic version here is likewise untouched. This is a
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

pub mod acquisition;
pub mod cache;
pub mod candidate;
pub mod descriptor;
pub mod discovery;
pub mod dispatch;
pub mod diversity;
pub mod error;
pub mod estimate;
pub mod explain;
pub mod failure;
pub mod fairness;
pub mod metrics;
pub mod plan;
pub mod platform;
pub mod policy;
pub mod privacy;
pub mod quality;
pub mod requirements;
pub mod resilience;
pub mod resolve;
pub mod retry;
pub mod scoring;
pub mod security;
pub mod setup;
pub mod stability;
pub mod types;

pub use cache::RouteCache;
pub use candidate::{PathCandidate, TransportEndpoint};
pub use descriptor::{ByteCount, ContentClass, OperationDescriptor, OperationId};
pub use diversity::{are_diverse, group_by_underlay, most_diverse_fallback, UnderlayId};
pub use error::RoutingError;
pub use estimate::{
    completion_time_millis, eliminate_deadline_exceeding_candidates, exceeds_deadline,
};
pub use explain::{
    hint_from_plan, infer_reason, metric_events_for, revalidate_hint, RouteHint, RouteMetricEvent,
    RouteReason,
};
pub use failure::RouteFailureClass;
pub use metrics::{
    Bitrate, Confidence, CongestionState, EnergyCost, MeasuredValue, NetworkCost, PathMetrics,
    Ratio, SignalQuality, StabilityScore,
};
pub use plan::{plan_route, RoutePlan, RouteStrategy};
pub use policy::{HysteresisPolicy, PolicyWeights, RoutingPolicy, RoutingPolicyProfile};
pub use privacy::{
    direct_preference_bonus, eliminate_privacy_violations, eliminate_unjustified_expensive_setup,
    justifies_expensive_setup, passes_privacy_policy, PrivacyPolicy,
};
pub use quality::{quality_signal_for, PathQualitySignal};
pub use requirements::DeliveryRequirements;
pub use resilience::{
    diagnose, escalation_stage_of, hedge_policy_for, should_escalate_beyond,
    timeout_millis_for_stage, EscalationStage, HedgePolicy, RejectionReason, RouteDiagnostics,
};
pub use resolve::resolve_destination_devices;
pub use retry::RetryPolicy;
pub use scoring::{DefaultScorer, PathScorer, RouteScore, RouteScoreDelta, RoutingContext};
pub use security::{authorize_candidate, eliminate_untrusted_candidates, AuthenticatedSession};
pub use setup::{effective_setup_cost, static_setup_cost, ConnectionPoolState, SetupCost};
pub use stability::derive_stability_score;
pub use types::{
    DeliveryClass, Destination, MeteredState, PathCapabilities, PathId, Priority, RoamingState,
    RouteHealth, TransportKind,
};
