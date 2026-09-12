//! §145 "Transport Adapter Contract", §146 "Iroh Adapter", §147 "LAN
//! Adapter", §148 "Bluetooth Adapter", §149 "Wi-Fi Direct/Aware
//! Adapter", §150 "DTN Adapter".
//!
//! This module has almost no code in it — deliberately. §145's own
//! "report" half (availability, capabilities, health, metrics, setup
//! cost, current session state) is already exactly what
//! [`crate::candidate::PathCandidate`] carries: a `PathCandidate` *is*
//! one adapter's reported snapshot of one path, field for field. §145's
//! "support" half (connect/acquire, close, send/stream) is the same
//! kind of trait §120 "Transport Manager API" already declined to
//! define here, for the same reason: it needs a real session handle
//! type, and this crate has no transport dependency to define one
//! against (see this crate's own top doc comment on scope, and
//! [`crate::engine`]'s own doc comment on §120 for the fuller
//! reasoning). §146-150's own per-adapter field lists are checked
//! below one by one against what already exists, with the genuine
//! gaps named rather than papered over with a field that doesn't
//! actually mean anything yet.
//!
//! ## §145 "Transport Adapter Contract" — the report half
//!
//! | Spec's field | This crate's equivalent |
//! |---|---|
//! | availability | [`crate::acquisition::CandidateState`] |
//! | capabilities | [`crate::types::PathCapabilities`] |
//! | health | [`crate::types::RouteHealth`] |
//! | metrics | [`crate::metrics::PathMetrics`] |
//! | setup cost | [`crate::setup::SetupCost`] |
//! | current session state | [`crate::setup::ConnectionPoolState`] (via [`crate::metrics::PathMetrics::pool_state`]) |
//!
//! ## §146 "Iroh Adapter"
//!
//! "Direct vs relay path" is [`crate::types::TransportKind::IrohDirect`]/
//! [`crate::types::TransportKind::IrohRelay`] — already two plain enum
//! variants, not an Iroh-specific type. "RTT" is
//! [`crate::metrics::PathMetrics::rtt_millis`]. "Session availability"
//! is [`crate::acquisition::CandidateState`]. "Address info" is
//! [`crate::candidate::TransportEndpoint`] — already opaque bytes, per
//! that type's own doc comment. §146's own closing line — "routing API
//! should not expose Iroh-specific types upward" — is therefore
//! already true structurally, not merely by convention: there is no
//! Iroh-specific type anywhere in this crate's public API to expose.
//!
//! ## §147 "LAN Adapter"
//!
//! "Local reachability" is [`crate::types::TransportKind::LocalLan`]
//! plus [`crate::types::RouteHealth`]. "Estimated throughput class" is
//! [`crate::metrics::PathMetrics::estimated_bandwidth`]. "Interface"
//! (e.g. which network interface a LAN path is bound to) has **no
//! equivalent** — a genuine gap, left open deliberately rather than
//! bolted onto [`crate::candidate::TransportEndpoint`], which is
//! opaque by design (see that type's own doc comment); an interface
//! name is exactly the kind of transport-specific detail that opacity
//! exists to keep out of this crate.
//!
//! ## §148 "Bluetooth Adapter"
//!
//! "BLE vs Classic" is
//! [`crate::types::TransportKind::BluetoothLe`]/[`crate::types::TransportKind::BluetoothClassic`].
//! "Estimated bandwidth" is
//! [`crate::metrics::PathMetrics::estimated_bandwidth`]. "Do not use
//! Bluetooth MAC as identity" is already true structurally: every
//! candidate's identity field is
//! [`crate::candidate::PathCandidate::peer`], a
//! [`siar_domain::DeviceId`] — there is no MAC-shaped field anywhere
//! in this crate for a caller to reach for instead, even by mistake.
//! "Proximity" and "paired/available state" have **no equivalent** —
//! [`crate::acquisition::CandidateState`] covers *reachability*
//! (`RequiresDiscovery`/`RequiresSetup`/`Active`) but not "paired," a
//! distinct Bluetooth-specific pairing-bond concept this crate has no
//! field for, and proximity (RSSI/distance) has never had a home in
//! any metric here. Both are named gaps, not silently folded into an
//! existing field that doesn't actually mean the same thing.
//!
//! ## §149 "Wi-Fi Direct/Aware Adapter"
//!
//! "Setup cost" is [`crate::setup::SetupCost`] (already `Expensive`
//! for both, per that module's own worked example). "Available
//! bandwidth class" is
//! [`crate::metrics::PathMetrics::estimated_bandwidth`]. "Background
//! restrictions" is
//! [`crate::types::PathCapabilities::requires_foreground`] combined
//! with [`crate::platform::DeviceState::foreground`] — already real
//! since round 7. "Current group/session" (which Wi-Fi Direct group a
//! path belongs to) has **no equivalent** — the same shape of gap as
//! §147's "interface": a transport-specific session handle this
//! crate's opaque [`crate::candidate::TransportEndpoint`] deliberately
//! doesn't surface.
//!
//! ## §150 "DTN Adapter"
//!
//! "Store-and-forward path available" is
//! [`crate::types::TransportKind::Dtn`] plus
//! [`crate::types::PathCapabilities::store_and_forward`]. "Replication
//! policy" is
//! [`crate::requirements::DeliveryRequirements::dtn_replication_budget`]
//! (§56, already real). "Uncertain latency" has no dedicated
//! field, but it's already expressed the honest way this crate
//! expresses any unmeasured quantity:
//! [`crate::metrics::PathMetrics::rtt_millis`] simply being `None` for
//! a DTN candidate *is* "uncertain," the same convention
//! [`crate::types::MeteredState::Unknown`] and
//! [`crate::types::RoamingState::Unknown`] already use for a different
//! kind of uncertainty — inventing a second, DTN-specific
//! "uncertainty" type on top of that would just be encoding the same
//! fact twice. "Delivery probability" — **updated since this was first
//! written**: [`crate::probability`] (§151, a later round) added
//! [`crate::metrics::PathMetrics::delivery_likelihood`]/
//! [`crate::metrics::PathMetrics::expected_delay_class`] as the place
//! an adapter *reports* this from its own encounter history; this
//! crate still doesn't compute that estimate itself (same "no clock,
//! no history" boundary [`crate::explain::RouteMetricEvent`]'s own doc
//! comment already draws) — see [`crate::probability`]'s own doc
//! comment for the fuller distinction between representation and
//! computation. "Detailed
//! peer encounter logic remains in Part 06" is the spec's own
//! confirmation that this crate isn't where that estimate should live
//! anyway.
