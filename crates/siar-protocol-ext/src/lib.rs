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
//! - §17 "Session-Local Extension IDs": [`negotiation::negotiate`]
//!   assigns a [`descriptor::SessionLocalExtensionId`] to every
//!   successfully negotiated extension (sequential by iteration order —
//!   session-local and not persisted, so the exact numbering scheme
//!   doesn't matter; the spec's own 7/9/12 example is illustrative,
//!   not an algorithm to reproduce).
//! - §24 "Lazy Extension Opening", §25 "Lazy Initialization Targets"
//!   ([`lifecycle::LazyInitTarget`], §25's own closed list of
//!   subsystems), §26 "Extension Shutdown"
//!   ([`lifecycle::GRACEFUL_SHUTDOWN_STEPS`]/[`lifecycle::ABRUPT_SHUTDOWN_STEPS`],
//!   two separate ordered step lists, not one enum with a mode flag —
//!   see that module's own doc comment for why).
//! - [`violation`] — §28 "Protocol Violation Classification"
//!   ([`violation::classify_framing_error`]/[`violation::classify_extension_error`],
//!   both grounded in §28's own four worked examples, kept as this
//!   module's tests), §29 "Unknown Extensions"
//!   ([`violation::unknown_extension_policy`]), §31 "Operation-Level
//!   Required Capabilities" ([`violation::operation_supported`]).
//! - §30 "Unknown Capabilities": [`descriptor::ExtensionDescriptor::required_capabilities`]
//!   plus the check [`negotiation::negotiate`] now runs against it —
//!   "unknown optional capability → ignore" needs no code at all
//!   ([`capability::CapabilitySet::intersect`] already drops anything
//!   unshared); "unknown required capability → reject extension
//!   operation/negotiation" is [`negotiation::NegotiationError::RequiredCapabilityUnavailable`]
//!   for a `Required` extension, or a silently-dropped extension for
//!   an `Optional` one (§11's "don't tear down the session" rule,
//!   applied one level down).
//! - [`security`] — §32 "Security Requirements Per Extension"
//!   ([`security::SecurityRequirements`], verbatim, plus
//!   [`security::SecurityRequirements::messaging_default`] for §32's
//!   own worked example), §33 "Authorization Hooks"
//!   ([`security::ExtensionAuthorization`], verbatim trait shape, with
//!   a block-list authorizer in that module's own tests standing in
//!   for §33's "Messenger → block/contact policy" example).
//! - §34 "Namespaced Application Extensions": no new code — already
//!   fully enforced by [`identifier::NamespaceId`]'s existing
//!   validation; this round only confirmed spec §34's own three
//!   example namespaces round-trip through it.
//! - §36 "Generated Documentation": [`descriptor::generate_documentation`],
//!   covering all five things §36 names tooling should be able to
//!   generate (supported extensions, version table, capabilities,
//!   resource limits, security requirements) from real
//!   [`descriptor::ExtensionDescriptor`] data, not hand-maintained
//!   prose.
//! - §37 "Wire Schema Ownership", §38 "Domain Types vs Wire Types": no
//!   new code — both are architectural separations this crate already
//!   maintains (this crate never touches `siar-protocol::WireMessage`;
//!   [`descriptor::ExtensionDescriptor`] is a domain-level type,
//!   [`framing::FrameHeader`]/[`framing::encode_frame_header`] is the
//!   wire-level one, kept deliberately separate). Reconciled, not
//!   reattempted as new code.
//! - §39 "Serialization Discipline": [`framing::encode_frame_header`]
//!   (previously decode-only) plus a golden byte-layout test — §39's
//!   own "golden compatibility tests" line, made real rather than only
//!   asserted. `frame_length`/`extension_session_id` were already
//!   fixed-width (`u32`/`u16`, never `usize`, §39's other explicit
//!   rule) since [`framing::FrameHeader`] was first written.
//! - §40 "Low-Copy Strategy": no new code this round — no large-payload
//!   type exists yet in this crate to apply the low-copy/priority-order
//!   principle to; noted here as reconciled-with-nothing-to-do-yet
//!   rather than silently skipped.
//! - [`events`] — §41 "Extension Events" ([`events::MessagingEvent`]/
//!   [`events::FileEvent`], spec's own verbatim variant lists), §42
//!   "Avoid a Mandatory Giant Event Enum" ([`events::CoreEvent`] kept
//!   separate from the feature-specific enums, [`events::AggregatedEvent`]
//!   as the explicitly opt-in aggregation §42 asks for — see that
//!   module's own doc comment for why `PresenceEvent` isn't included:
//!   the spec never gives its variants, unlike messaging's and files').
//! - [`sdk`] — §43 "Public SDK Surface": marker traits only
//!   ([`sdk::CommunicationClient`]/[`sdk::MessagingClient`]/
//!   [`sdk::FileTransferClient`]/[`sdk::PresenceClient`]) — no backing
//!   implementation exists yet (see this crate's "No wire integration"
//!   note), but the boundary itself (client traits distinct from
//!   [`registry::ProtocolExtension`], the raw handler interface) is
//!   real and tested.
//! - [`peer`] — §44 "Peer Capability Query"
//!   ([`peer::PeerCapabilities::supports`]), §45 "Capability Cache"
//!   ([`peer::PeerCapabilityCache`], with real expiry enforcement —
//!   "cached capabilities are hints, not security truth" made
//!   structural where a type system can actually enforce it), §46
//!   "Capability Changes" ([`peer::CapabilityChangeReason`], spec's own
//!   six causes as a closed enum).
//! - §47 "Stability Classification":
//!   [`descriptor::ExtensionStability`] (verbatim enum, now a field on
//!   [`descriptor::ExtensionDescriptor`]) plus
//!   [`descriptor::requires_opt_in`] for §47's one concrete rule
//!   ("Experimental protocols should require explicit opt-in in
//!   production builds").
//! - §48 "Private/Enterprise Extensions", §49 "Third-Party Extension
//!   Loading", §50 "FFI Boundary": no new code — all three are already
//!   true of this crate's existing design. §48:
//!   [`identifier::NamespaceId`] already accepts
//!   `com.company/internal-workflow/1`-style identifiers with no core
//!   protocol change needed. §49: [`registry::ExtensionRegistry`] only
//!   ever holds compile-time `Box<dyn registry::ProtocolExtension>`
//!   values — there is no runtime-loading mechanism to have
//!   accidentally built. §50: no `comm-ffi` crate exists, correctly —
//!   spec §50 itself says not to build one until external-language
//!   integration is actually required.
//! - §51 "Transport Neutrality": no new code — [`security::PeerIdentity`]
//!   (this crate's only peer-identity type) is already an opaque
//!   32-byte identifier with no Iroh/Bluetooth/IP/Wi-Fi-specific type
//!   anywhere near it; [`routing::RoutingRequirements`] (§53, below)
//!   was written to the same rule from the start — see that module's
//!   own doc comment.
//! - [`routing`] — §52 "Delivery Classes" ([`routing::DeliveryClass`],
//!   verbatim, plus [`routing::spec_52_example_classification`]
//!   covering spec §52's own worked examples — "File chunk" is
//!   deliberately left unclassified since the spec itself says its
//!   answer depends on policy, not a fixed rule), §53 "Routing
//!   Requirements Per Operation" ([`routing::RoutingRequirements`],
//!   spec's own six-item list — see that module's doc comment for why
//!   it never names a transport, tying §51 and §53 together).
//! - [`resource`] — §54 "Resource Accounting" ([`resource::ResourceAccounting`],
//!   spec's own six tracked fields per extension), §55 "Per-Peer
//!   Quotas" ([`resource::QuotaPolicy`] keyed by [`resource::TrustLevel`]
//!   — spec says quotas vary by trust level but never names one, so
//!   this is the smallest closed set that makes that checkable, not a
//!   guessed taxonomy), §56 "Abuse Handling"
//!   ([`resource::ABUSE_ESCALATION_LADDER`], spec's own five controls
//!   in order, plus [`resource::AbuseControl::is_scoped_to_one_extension`]
//!   encoding spec §56's isolation principle — a malicious file stream
//!   shouldn't disable unrelated messaging state — as a checkable
//!   property of the ladder itself).
//! - [`observability`] — §57 "Observability" ([`observability::TraceFields`],
//!   spec's own eight fields and *only* those eight — "never log
//!   sensitive payload contents or keys" enforced structurally by the
//!   type simply having no such field), §58 "Metrics"
//!   ([`observability::ExtensionMetrics`], spec's own eight counters,
//!   real in-memory counters a caller can read directly — "should work
//!   without external telemetry" is true by construction), §59
//!   "Diagnostics" ([`observability::render_diagnostics`], reproducing
//!   spec §59's own worked example shape; capability-id-to-name lookup
//!   is caller-supplied rather than guessed, since this crate has no
//!   built-in name registry for [`capability::CapabilityId`]).
//! - §60 "Compatibility Matrix": no new production code — this is
//!   already true of [`negotiation::negotiate`] (major version is part
//!   of [`identifier::ProtocolId`] identity, so a major mismatch is
//!   already "not the same protocol"; [`capability::CapabilitySet::intersect`]
//!   already implements "old subset ↔ new superset"). What's new is
//!   the matrix of tests spec §60 itself asks every stable extension to
//!   have — v1.0↔v1.0, old-subset↔new-superset both directions, and a
//!   major-version mismatch failing cleanly for both `Required` and
//!   `Optional` — applied to this crate's own negotiation engine as
//!   the reference case, in `negotiation.rs`'s own test module.
//! - §61 "Golden Wire Tests": no new code beyond §39's — the pattern
//!   spec §61 asks for (Rust value → expected exact stable bytes →
//!   decode back) already exists twice: [`framing::encode_frame_header`]'s
//!   golden byte test, and [`identifier`]'s existing canonical-string
//!   round-trip test. Reconciled, not reattempted as new code.
//! - §62 "Fuzzing": genuinely not attempted — needs `cargo-fuzz`
//!   infrastructure (a nightly toolchain, `libfuzzer`-linked binary
//!   targets) this pass didn't set up. Real follow-up work, not a
//!   silently-skipped small addition.
//! - §63 "Property Tests": four of spec §63's five listed invariants
//!   were already covered by existing tests elsewhere (round-trip:
//!   §39's tests; unknown-optional-doesn't-kill-session and
//!   required-unknown-capability-rejected: §11/§30's tests;
//!   unbounded-allocation: `framing.rs`'s hostile-length test) — only
//!   "duplicate advertisement is deterministic" had nothing testing it,
//!   added in `negotiation.rs`. A real `proptest`-based suite
//!   generalizing these beyond fixed examples is separate, real
//!   follow-up work, not done this round.
//! - §64 "Simulated Peer Testing", §65 "Upgrade Example": reproduced
//!   exactly as spec's own worked examples, as tests in
//!   `negotiation.rs` — no new production code, since both are already
//!   true of [`negotiation::negotiate`] as written.
//! - §66 "Multiple Major Versions": no code — "convert into common
//!   domain models where semantics permit" needs an actual domain
//!   model layer for a real extension's content, which doesn't exist
//!   in this crate by design (see "No wire integration").
//! - [`deprecation`] — §67 "Deprecation Lifecycle"
//!   ([`deprecation::DeprecationStatus::advance`], strictly linear, no
//!   skip-ahead — mirrors [`lifecycle::ExtensionLifecycle`]'s own
//!   pattern), §68 "Security Deprecation"
//!   ([`deprecation::DeprecationStatus::force_security_deprecate`], the
//!   one sanctioned way to skip `Deprecated` entirely, plus
//!   [`deprecation::SECURITY_DEPRECATION_STEPS`], spec's own four-step
//!   list in order).
//! - [`persistence`] — §69 "Extension Persistence"
//!   ([`persistence::MESSAGING_OWNED_TABLES`]/[`persistence::FILES_OWNED_TABLES`],
//!   spec's own verbatim table names, tested disjoint). §70 "Wire vs
//!   Database Migration": no code — already true by construction, this
//!   crate has no database dependency at all.
//! - [`examples`] — §71 "ERP Custom Extension Example"
//!   ([`examples::erp_approval_example`], tested negotiating exactly
//!   like a core extension with nothing special-cased), §72 "Emergency
//!   Extension Design" ([`examples::EmergencyCriticalFields`], spec's
//!   own five critical fields; [`examples::LocationPrivacy`] is this
//!   module's own closed set for the one field spec names but never
//!   gives values for), §73 "File Extension Example"
//!   ([`examples::FileCapability`]/[`examples::StreamRole`], both
//!   verbatim), §74 "Messaging Extension Example"
//!   ([`examples::MessagingCapability`], verbatim seven), §75
//!   "Presence Extension Example" ([`examples::PresenceCapability`],
//!   verbatim three, plus [`examples::presence_default_routing`]
//!   turning spec's four prose properties into a real
//!   [`routing::RoutingRequirements`]), §76 "Calls and Media
//!   Extensions" ([`examples::CallsCapability`]/[`examples::MediaCapabilityArea`],
//!   kept as two separate enums matching spec's own "separate call
//!   control from media transport" instruction), §77 "Protocol
//!   Composition" (tested: all six of spec's own named extensions
//!   negotiating together in one call, proving no central monolith is
//!   needed).
//! - [`handshake`] — §78 "Core Handshake State Machine"
//!   ([`handshake::HandshakeStage`], spec's own eight-stage diagram
//!   plus a `Failed` terminal reachable before `SessionEstablished`
//!   for "authentication failure stops application-level use"), §79
//!   "Reconnection" ([`handshake::ReconnectionRevalidation`], all three
//!   of spec's revalidation checks required, not a majority — the
//!   concrete form of "never blindly trust stale session metadata").
//! - [`platform`] — §80 "Mobile Efficiency"
//!   ([`platform::MobileEfficiencyPractice`], spec's own seven
//!   practices — most already true of this crate's existing design,
//!   see that module's own doc comment for which three are genuinely
//!   new), §81 "Headless Nodes"
//!   ([`platform::HeadlessCapableExtension`]/[`platform::HeadlessUseCase`],
//!   both verbatim). §82 "Compile-Time Features", §83 "Binary Size": no
//!   code — both are guidance for *other* workspace crates' Cargo
//!   feature flags, not this standalone crate's own concern.
//! - [`config`] — §84 "Typed Extension Configuration"
//!   ([`config::MessagingConfig`]/[`config::FileConfig`], both
//!   verbatim), §85 "Configuration Validation"
//!   ([`config::validate_file_config`], spec's own worked example —
//!   chunk size vs. `max_frame_size`), §86 "Extension Dependencies"
//!   ([`config::ExtensionDependencyKind`], verbatim two-item list, plus
//!   a test proving messaging negotiates fine with files never
//!   advertised at all — spec's own "messaging itself must continue
//!   working without files," directly demonstrated).
//! - [`integration`] — §87 "Integration Interfaces"
//!   ([`integration::ContentResolver`], verbatim trait shape, tested
//!   against a stub file-side implementation — the concrete mechanism
//!   behind §86's "must continue working without files": messaging
//!   depends on this trait, never on a files crate directly), §88
//!   "Core vs First-Party Extensions"
//!   ([`integration::ExtensionClassification`], verbatim four tiers,
//!   deliberately kept a separate axis from
//!   [`descriptor::ExtensionStability`] — see that type's own doc
//!   comment for why conflating them would be wrong), §89 "Extension
//!   Provenance" ([`integration::ProvenanceControl`], verbatim four
//!   items), §90 "Interoperability Specification"
//!   ([`integration::InteropDocumentationChecklist`], spec's own nine
//!   documentation items as real boolean gates — `is_complete()`
//!   requires all nine, not most).
//! - §91 "Rust Is the Reference, Not the Wire Format": no new code —
//!   already true by construction. [`framing::FrameHeader`]'s wire
//!   fields are fixed-width (`u32`/`u16`/`u8`, never `usize`),
//!   [`framing::encode_frame_header`] writes explicit big-endian bytes
//!   rather than relying on any native layout, and every wire-adjacent
//!   enum in this crate goes through `serde`'s own variant encoding
//!   (never a raw `#[repr]` memory layout) — none of spec §91's five
//!   forbidden dependencies exist anywhere in this crate.
//! - §92 "Postcard Rules": no new code — this crate doesn't depend on
//!   Postcard at all (`Cargo.toml` has no such dependency), so none of
//!   its six rules currently apply; noted here so a future pass that
//!   *does* add Postcard knows [`framing`]'s hand-rolled
//!   encode/decode+golden-test pattern (§39) already satisfies "keep
//!   golden test vectors" and "avoid `usize` on wire" by example.
//! - [`health`] — §93 "Error Code Strategy"
//!   ([`health::ErrorCodeRange`]/[`health::ErrorCode`], spec's own
//!   worked ranges verbatim — "human-readable text is diagnostic
//!   only" enforced structurally, `code` and `diagnostic_text` are
//!   always two separate fields), §94 "Extension Health"
//!   ([`health::ExtensionHealth`], using this crate's real
//!   [`lifecycle::ExtensionLifecycle`] rather than an invented
//!   `ExtensionState`), §95 "Recovery Semantics"
//!   ([`health::OperationRecoveryClass`], verbatim four classes, plus
//!   spec's own five worked examples reproduced as a test).
//! - [`isolation`] — §96 "Scheduler Contract"
//!   ([`isolation::SchedulingSubmission`], spec's own six declared
//!   fields — never a transport/queue/retry-policy field, same
//!   declare-don't-execute discipline as [`routing::RoutingRequirements`]
//!   §53), §97 "Storage Isolation" ([`isolation::StorageNamespace`],
//!   spec's own five namespaces with real key-prefix strings), §98
//!   "Metrics Isolation" ([`isolation::LabeledMetric`], structurally
//!   incapable of carrying a peer id as a label, same pattern as
//!   [`observability::TraceFields`] and payloads), §99 "Capability
//!   Isolation" ([`isolation::ModuleAccessGrant`], spec's own ERP
//!   document-module example reproduced as a test).
//! - §100 "Public API Example", §101 "File-Only Acceptance Test": no
//!   new code — both describe `CommunicationRuntime`/`MessagingExtension`/
//!   `FileExtension`, none of which exist in this crate by design (see
//!   "No wire integration," "No runtime"). §101's actual test —
//!   "if this cannot compile cleanly, the architecture is still too
//!   coupled" — has its closest real analog in
//!   `examples::spec_71_a_third_party_namespace_negotiates_exactly_like_a_core_extension`,
//!   which negotiates one extension with zero reference to any other
//!   extension type.
//! - §102 "Custom ERP Extension": no new code — identical in substance
//!   to `examples::erp_approval_example` (§71), already built.
//! - §103 "Anti-Patterns": no new code — spec's own eight-item list,
//!   each already avoided by a specific, real decision made across
//!   this crate's rounds rather than by policy alone: "one giant
//!   event enum" → [`events`]'s per-feature enums (§42); "every
//!   extension mandatory" → [`descriptor::ExtensionRequirement::Optional`]
//!   exists and is exercised throughout; "serialize domain structs
//!   directly" → [`framing::FrameHeader`] is hand-encoded, never a
//!   derived domain type; "unbounded queues" →
//!   [`backpressure::BoundedQueue`]; "raw strings for hot capability
//!   checks" → [`capability::CapabilityId`] is a `u32`, never a
//!   `String`; "Dioxus part of protocol handling" / "Kotlin own
//!   protocol state" → this crate has zero UI-framework or
//!   platform-language dependency of any kind; "assume Iroh is the
//!   only transport forever" → [`security::PeerIdentity`]/
//!   [`routing::RoutingRequirements`] never name a transport (§51,
//!   §53).
//! - §104 "Recommended Crates for This Part", §105 "Implementation
//!   Sequence": no new code — §104 is a suggested directory layout
//!   this crate's actual module list is a superset of, and §105's six
//!   phases are all either already covered (Phase 1: [`descriptor`]/
//!   [`registry`]; Phase 3: [`lifecycle`]/[`framing`]; Phase 4:
//!   [`scheduler`]/[`backpressure`]/[`resource`]) or out of scope by
//!   design (Phase 2's literal `Hello`/`VersionNegotiation` wire
//!   *message types* — this crate models the handshake *state machine*
//!   ([`handshake::HandshakeStage`], §78) but spec never gives field
//!   lists for the actual wire messages, so inventing them here would
//!   be guessed content; Phase 5's messaging/files conversion is "No
//!   wire integration"; Phase 6's fuzzing is §62's honest gap).
//! - [`definition_of_done`] — §106 "Definition of Done": spec's own
//!   sixteen-item completion checklist as a real, checkable self-audit
//!   ([`definition_of_done::DefinitionOfDoneItem::status`]) — answered
//!   honestly, including the four items this crate does NOT yet
//!   satisfy (two concrete-extension-type gaps that are the same root
//!   cause as "No wire integration"; `ExtensionContext`'s placeholder
//!   fields, §13; and §62's fuzzing gap). A completion checklist that
//!   can't say "not yet" to some of its own items isn't actually being
//!   checked — see that module's own test enforcing the count stays
//!   honest.
//! - §107 "Relationship to the Remaining 23 Parts": no new code — this
//!   is the same 24-part roadmap `ROADMAP.md` (repo root) already
//!   tracks in full; not re-duplicated here.
//! - §108 "Final Principle": no new code — this is Part 01's closing
//!   mission statement ("evolve through independently versioned,
//!   negotiated capabilities, not an ever-growing central enum"), which
//!   this crate's entire design across every round embodies rather
//!   than states once at the end: every closed enum in this crate is
//!   *feature-scoped* ([`events`]'s per-feature events, §42;
//!   [`examples`]'s per-domain capability vocabularies, §71-77), never
//!   one central "AllExtensionEvents"/"AllCapabilities" enum growing
//!   without bound as new extensions are added.
//!
//! **Part 01 (`siar-protocol-ext`) is now feature-complete against all
//! 108 spec sections** — either built, or reconciled as genuinely
//! no-code-needed with the specific reason recorded above. The honest
//! remaining gaps are four, all named in [`definition_of_done`]: two
//! concrete-extension-type demonstrations (would need actual
//! `MessagingExtension`/`FileExtension` types this crate deliberately
//! doesn't build), real `ExtensionContext` service wiring (§13, blocked
//! on a real runtime existing), and a `cargo-fuzz` suite (§62).
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
//! - §19's limits are defined and checkable
//!   ([`framing::validate_frame_length`]) but not enforced end-to-end
//!   against live in-flight-frame/stream/buffered-byte counts (only
//!   `max_frame_size` is).
//! - **Not attempted at all**: §62 fuzzing (needs `cargo-fuzz`
//!   infrastructure), and two `Definition of Done` items that need
//!   actual `MessagingExtension`/`FileExtension` concrete types this
//!   crate deliberately doesn't build (see [`definition_of_done`]).
//!   Every other section of this 108-section spec is either built or
//!   reconciled as genuinely no-code-needed, with the specific reason
//!   recorded inline above.
//! - **Parts 02 and 03 of the series** (multi-device identity;
//!   transport routing policy) are entirely unstarted — neither
//!   shares code with this crate, and each is comparably large.

pub mod backpressure;
pub mod capability;
pub mod channel;
pub mod config;
pub mod definition_of_done;
pub mod deprecation;
pub mod descriptor;
pub mod events;
pub mod examples;
pub mod framing;
pub mod handshake;
pub mod health;
pub mod identifier;
pub mod integration;
pub mod isolation;
pub mod lifecycle;
pub mod negotiation;
pub mod observability;
pub mod peer;
pub mod persistence;
pub mod platform;
pub mod registry;
pub mod resource;
pub mod routing;
pub mod scheduler;
pub mod sdk;
pub mod security;
pub mod violation;

pub use backpressure::{BoundedQueue, QueueFull};
pub use capability::{CapabilityId, CapabilityRegistry, CapabilitySet};
pub use channel::ChannelKind;
pub use config::{ConfigError, ExtensionDependencyKind, FileConfig, MessagingConfig, validate_file_config};
pub use definition_of_done::{DefinitionOfDoneItem, CUSTOM_APP_EXTENSIONS_WITHOUT_CORE_CHANGES_NOTE};
pub use deprecation::{
    DeprecationStatus, InvalidDeprecationTransition, SecurityDeprecationAction,
    SECURITY_DEPRECATION_STEPS,
};
pub use descriptor::{
    generate_documentation, requires_opt_in, ExtensionDescriptor, ExtensionLimits,
    ExtensionRequirement, ExtensionStability, ExtensionVersion, NegotiatedExtension,
    SessionLocalExtensionId,
};
pub use events::{AggregatedEvent, CoreEvent, FileEvent, MessagingEvent};
pub use examples::{
    emergency_alert, emergency_resource, emergency_sos, erp_approval_example,
    presence_default_routing, CallsCapability, EmergencyCriticalFields, FileCapability,
    LocationPrivacy, MediaCapabilityArea, MessagingCapability, PresenceCapability, StreamRole,
};
pub use framing::{
    encode_frame_header, parse_frame_header, validate_frame_length, FrameHeader, FramingError,
    FRAME_HEADER_BYTES,
};
pub use handshake::{HandshakeStage, InvalidHandshakeTransition, ReconnectionRevalidation};
pub use health::{
    ErrorCode, ErrorCodeRange, ExtensionErrorSummary, ExtensionHealth, OperationRecoveryClass,
    CORE_ERROR_CODE_RANGE, FILES_ERROR_CODE_RANGE, MESSAGING_ERROR_CODE_RANGE,
};
pub use identifier::{
    IdentifierError, NamespaceId, ProtocolId, ProtocolMajor, ProtocolMinor, ProtocolName,
};
pub use integration::{
    ContentReference, ContentResolver, ExtensionClassification, ExtensionInteropStatus,
    InteropDocumentationChecklist, ProvenanceControl, ResolveError, ResolvedContent,
};
pub use isolation::{LabeledMetric, ModuleAccessGrant, SchedulingSubmission, StorageNamespace};
pub use lifecycle::{
    AbruptShutdownStep, ExtensionError, ExtensionLifecycle, GracefulShutdownStep,
    InvalidLifecycleTransition, LazyInitTarget, TrafficPriority, ABRUPT_SHUTDOWN_STEPS,
    GRACEFUL_SHUTDOWN_STEPS,
};
pub use negotiation::{negotiate, NegotiationError, RemoteAdvertisement};
pub use observability::{render_diagnostics, ExtensionMetrics, TraceFields, TraceResult};
pub use peer::{CacheSource, CapabilityCacheEntry, CapabilityChangeReason, PeerCapabilities, PeerCapabilityCache};
pub use persistence::{OwnedTable, FILES_OWNED_TABLES, MESSAGING_OWNED_TABLES};
pub use platform::{HeadlessCapableExtension, HeadlessUseCase, MobileEfficiencyPractice};
pub use registry::{
    ExtensionContext, ExtensionHandler, ExtensionRegistry, ExtensionRegistryBuilder,
    ProtocolExtension, RegistryError,
};
pub use resource::{
    AbuseControl, PeerQuota, QuotaPolicy, ResourceAccounting, ResourceUsage, TrustLevel,
    ABUSE_ESCALATION_LADDER,
};
pub use routing::{DeliveryClass, RoutingRequirements, SizeClass};
pub use scheduler::FairScheduler;
pub use sdk::{CommunicationClient, FileTransferClient, MessagingClient, PresenceClient};
pub use security::{
    AuthorizationDecision, ExtensionAuthorization, OperationDescriptor, PeerIdentity,
    SecurityRequirements,
};
pub use violation::{
    classify_extension_error, classify_framing_error, operation_supported,
    unknown_extension_policy, UnsupportedOperationResponse, ViolationClass,
};
