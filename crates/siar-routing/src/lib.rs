//! The resilient routing core — next.md §4 ("Resilient Router"), §7,
//! §9–10, §39, §90–94. Phase 5 of `next.md`'s roadmap
//! ("cross-transport routing, BLE → Wi-Fi upgrades, gateway nodes").
//!
//! - [`path`][]: [`path::TransportCapabilities`]/[`path::capabilities_for`]
//!   (next.md §7), and [`path::PathTable`] (§91) — per-destination
//!   candidate routes, plus [`path::PathTable::best_route_for`]/
//!   [`path::PathTable::recommend_upgrade`] — the BLE→Wi-Fi upgrade
//!   *decision* (§90), added once `siar_domain::connectivity::
//!   TransportLink::preference_rank` existed to rank candidates by.
//!   [`path::PathTable::compose_via_relay`] (this pass) is the bounded,
//!   one-hop-at-a-time multi-hop composition primitive — see its own
//!   doc comment for what "bounded" means here and why.
//! - [`score`]: content-aware suitability (§9, §53) and
//!   [`score::route_score`]/[`score::best_route`] (§10, §39).
//! - [`scheduler`]: [`scheduler::PriorityScheduler`] (§93–94), and (this
//!   pass) [`scheduler::PriorityScheduler::congestion_ceiling`] — the
//!   self-contained, queue-occupancy half of "detecting real
//!   congestion" (§93's own dividing line: backlog in the throttled
//!   tiers).
//! - [`link_health`]: [`link_health::LinkHealth`] (this pass) — the
//!   other half: turning real send outcomes into the `rtt_millis`/
//!   `reliability` numbers `PathEntry` has always had fields for.
//! - [`device_routes`]: [`device_routes::DeviceRoutes`] — the
//!   `DeviceId -> EndpointId` join `path.rs`'s own doc comment flagged
//!   as needed but deferred when `PathTable` was corrected to key on
//!   `EndpointId`; built once `apps/emergency-node` had a real signal
//!   for it (`MailboxCheckIn`'s self-disclosure).
//!
//! `siar_connectivity::TransportManager` (a separate crate, deliberately
//! not this one — see this crate's `Cargo.toml` note on why) now exists
//! and is wired into `apps/emergency-node`, feeding this crate's
//! `PathTable` from real `SiarEndpoint::local_peers` observations. That
//! closes next.md §90's "`TransportManager` itself" gap this doc
//! comment used to list here.
//!
//! What next.md's own §90–91 describe that still ISN'T here, flagged
//! the same way this whole workspace has flagged every real-network-
//! shaped gap so far:
//!
//! - **Actually executing** a BLE→Wi-Fi upgrade `recommend_upgrade`
//!   suggests — dialing the new connection, tearing down the old one.
//!   That's real, OS-level radio control belonging to whatever consumes
//!   `siar-transport-wifi-direct`/`-wifi-aware`/`-ble`/
//!   `-bluetooth-classic`'s JNI bridges — and nothing does yet, because
//!   `apps/android` (the one binary that would own real Android radios)
//!   doesn't exist anywhere in this workspace. This crate now answers
//!   "should we upgrade, and to what" correctly; acting on that answer
//!   is real, separate, larger follow-up work.
//! - **Gateway-node bridging** (§40–43, §101–103) — a device relaying
//!   traffic between two peers that share no common transport (one
//!   reachable only via BLE, the other only via Internet). Beyond
//!   needing the same `apps/android`-owned radio control as the point
//!   above, this also needs every transport crate to implement one
//!   common send/receive interface a bridge could dispatch across —
//!   today each transport crate is an isolated JNI stub with its own
//!   shape, not a shared trait. Neither piece exists; not attempted
//!   here.
//! - **`LinkHealth::record_outcome` now has a real caller.** This pass
//!   built the computation (bounded rolling window -> reliability/RTT),
//!   `siar-connectivity::TransportManager` gained `record_send_outcome`
//!   to fold a measurement into the live `PathTable`, and a later pass
//!   gave it a genuine caller: `apps/emergency-node`'s `send_and_record`
//!   helper times every real `SiarEndpoint::send` this relay makes and
//!   reports the outcome back — see that function's own doc comment for
//!   the one honest approximation it still makes (classifying every
//!   send as `TransportLink::InternetDirect` without checking whether
//!   iroh actually negotiated direct vs. relayed). Queue-occupancy
//!   congestion (`PriorityScheduler::congestion_ceiling`) is also fully
//!   closed, called from the same relay's dequeue loop.
//! - **`PathTable::compose_via_relay` also has no real caller, and no
//!   real `RelayAdvertisement` source.** This pass closed the
//!   computation half of "multi-hop route computation" — composing one
//!   additional relay hop from this table's own known-good direct
//!   route to that relay, deliberately not a general multi-hop search
//!   (see that method's doc comment). What it does NOT include, and
//!   what would make it genuinely live: a real routing-advertisement
//!   exchange — some device on the mesh actually telling its neighbors
//!   "I can reach destination D" — which needs its own wire message
//!   type, a periodic exchange, and loop/flood-bound handling, none of
//!   which exists anywhere in this workspace. `RelayAdvertisement` is
//!   the shape that future exchange would need to produce, built now
//!   for the same reason `LinkHealth`'s shape was: so inventing the
//!   real protocol later doesn't also have to invent this from scratch.
//!
//! This crate is deliberately the "given accurate inputs, what's the
//! right decision" half — wiring real inputs into it, and acting on its
//! decisions, is transport- and platform-touching work each gap above
//! still needs.

pub mod device_routes;
pub mod link_health;
pub mod path;
pub mod scheduler;
pub mod score;
