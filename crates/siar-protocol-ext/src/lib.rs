#![forbid(unsafe_code)]

//! siar-protocol-ext: a first slice of "Part 01 — Protocol Extension
//! System Architecture" (one of a 24-part architecture series the
//! person supplied; parts 02, "Multi-Device Identity Architecture",
//! and 03, "Transport Routing Policy Engine Architecture", were
//! supplied alongside it and are not attempted in this crate — see
//! this crate's introduction for why one coherent slice was chosen
//! over shallow coverage of all three).
//!
//! ## What's real here (implemented against the spec text, not guessed)
//!
//! - [`identifier`] — §5 "Protocol Namespace", §6 "Strong Protocol
//!   Identifiers", §7 "Major and Minor Versions": [`identifier::ProtocolId`]
//!   and its canonical-string round trip.
//! - [`capability`] — §8 "Capability Negotiation", §9 "Capability
//!   Representation": [`capability::CapabilitySet::intersect`] is
//!   `local ∩ remote`, verbatim from the spec.
//! - [`descriptor`] — §11 "Mandatory and Optional Extensions", §19
//!   "Per-Extension Resource Limits", §35 "Extension Descriptor".
//! - [`negotiation`] — §10 "Negotiation Flow": [`negotiation::negotiate`]
//!   reproduces the spec's own worked HELLO/HELLO_ACK example as a
//!   test, and enforces §11's "an unsupported optional extension must
//!   not tear down the whole session" rule.
//! - [`lifecycle`] — §21 "Traffic Priority", §23 "Extension Lifecycle"
//!   (as an enforced state machine, not just an enum — §23's own
//!   "lifecycle state must be explicit"), §27 "Typed Extension Errors".
//! - [`registry`] — §12 "Extension Registry", §13 "Runtime
//!   Construction" (the registry half of it — see that module's own
//!   doc comment for what's a placeholder), §14 "Extension Isolation"
//!   (structural: extensions only ever see [`registry::ExtensionContext`],
//!   never each other), §15 "Shared Services".
//! - [`channel`] — §16 "Logical Channel Model" (the classification
//!   enum; the actual head-of-line-blocking prevention is
//!   [`scheduler`]'s job — see [`channel::ChannelKind`]'s own doc
//!   comment for why priority, not channel identity, is the scheduling
//!   axis).
//! - [`framing`] — §18 "Framing", "never trust remote length fields"
//!   made real: [`framing::parse_frame_header`] always reads exactly
//!   [`framing::FRAME_HEADER_BYTES`] regardless of what the input
//!   claims, and [`framing::validate_frame_length`] checks a header
//!   against a real [`descriptor::ExtensionLimits`] before any
//!   allocation — §18's own four-step sequence (read bounded header →
//!   validate length → check extension limit → allocate/read safely)
//!   kept as genuinely separate steps, not collapsed into one
//!   "deserialize and hope" call.
//! - [`backpressure`] — §20 "Backpressure": [`backpressure::BoundedQueue`]
//!   really rejects on overflow (handing the item back to the
//!   producer) rather than growing without limit.
//! - [`scheduler`] — §22 "Fair Scheduling": [`scheduler::FairScheduler`]
//!   implements weighted round-robin across every
//!   [`lifecycle::TrafficPriority`] tier plus a real, *bounded*
//!   emergency override for `Critical` traffic — with a dedicated test
//!   reproducing §22's own named failure mode (naive
//!   highest-priority-first starving bulk traffic under sustained
//!   Critical load) and confirming this scheduler doesn't fall into
//!   it.
//!
//! ## What's explicitly NOT here
//!
//! - **No wire integration.** `siar-protocol::WireMessage` is
//!   untouched. This crate has a codec-agnostic frame header
//!   ([`framing`]) but no actual codec plugged into
//!   `siar-messaging`/`apps/*`'s existing call sites — it's a
//!   standalone capability-negotiation-and-scheduling layer a future
//!   pass would wire those into as "extensions" in this vocabulary.
//!   Doing that wiring blind, in the same pass as writing this crate,
//!   risked destabilizing a working system to match a speculative
//!   architecture document — a deliberate choice, not an oversight.
//! - **No runtime.** §13's `CommunicationRuntime` doesn't exist —
//!   [`registry::ExtensionRegistry`] is the registry half of that
//!   builder chain only.
//! - **No real shared-service handles.** [`registry::ExtensionContext`]'s
//!   `identity`/`session`/`scheduler`/`resources` fields are named
//!   placeholder types — see [`registry`]'s own doc comment. Notably,
//!   `scheduler` is NOT yet [`scheduler::FairScheduler`] — that wiring
//!   (giving each `ExtensionContext` a real handle to a shared
//!   scheduler instance) is real, separate follow-up work.
//! - **Not attempted at all**: §17 (session-local extension ID
//!   *negotiation* itself — the type exists via
//!   [`descriptor::SessionLocalExtensionId`], but nothing assigns one
//!   during [`negotiation::negotiate`] yet), §19's limits are defined
//!   and checkable ([`framing::validate_frame_length`]) but not
//!   enforced end-to-end against live in-flight-frame/stream/buffered-byte
//!   counts (only `max_frame_size` is), §24-§26 (lazy opening/shutdown),
//!   §28 onward (protocol violation classification, unknown
//!   extensions/capabilities, per-operation capabilities, security/
//!   authorization hooks, wire schema ownership, serialization
//!   discipline, SDK surface, transport neutrality, observability,
//!   testing strategy, upgrade/deprecation lifecycle, and more — the
//!   spec document runs to roughly 90 sections past where this crate
//!   stops). Each of these is a real, separate design surface in the
//!   spec, not a small addition to what's here.
//! - **Parts 02 and 03 of the series** (multi-device identity;
//!   transport routing policy) are entirely unstarted — neither
//!   shares code with this crate, and each is comparably large.

pub mod backpressure;
pub mod capability;
pub mod channel;
pub mod descriptor;
pub mod framing;
pub mod identifier;
pub mod lifecycle;
pub mod negotiation;
pub mod registry;
pub mod scheduler;

pub use backpressure::{BoundedQueue, QueueFull};
pub use capability::{CapabilityId, CapabilityRegistry, CapabilitySet};
pub use channel::ChannelKind;
pub use descriptor::{ExtensionDescriptor, ExtensionLimits, ExtensionRequirement, ExtensionVersion, NegotiatedExtension, SessionLocalExtensionId};
pub use framing::{parse_frame_header, validate_frame_length, FrameHeader, FramingError, FRAME_HEADER_BYTES};
pub use identifier::{IdentifierError, NamespaceId, ProtocolId, ProtocolMajor, ProtocolMinor, ProtocolName};
pub use lifecycle::{ExtensionError, ExtensionLifecycle, InvalidLifecycleTransition, TrafficPriority};
pub use negotiation::{negotiate, NegotiationError, RemoteAdvertisement};
pub use registry::{ExtensionContext, ExtensionHandler, ExtensionRegistry, ExtensionRegistryBuilder, ProtocolExtension, RegistryError};
pub use scheduler::FairScheduler;
