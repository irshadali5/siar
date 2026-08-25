//! Part 07 — Capability Negotiation Architecture.
//!
//! This crate is being built the same way as the workspace's other
//! Part-01–06 crates (see [[resilient-mesh]] in project memory): one
//! real, deliberately-scoped, tested slice per pass, tracked honestly
//! against the spec's own section numbers rather than a shallow first
//! pass across all 164 sections.
//!
//! ## This pass — Phase 1, Core Types (§161)
//!
//! - [`id`] — §5 "Capability Identifier", §6 "Capability Namespace":
//!   [`id::CapabilityId`], [`id::CapabilityNamespace`], [`id::NamespaceId`]
//! - [`version`] — §9-10 "Capability Versioning": [`version::CapabilityVersion`],
//!   with real major/minor negotiation logic (§19's "effective max =
//!   min(...)" rule specialized to versions)
//! - [`descriptor`] — §7-8 "Capability Descriptor" / "Required vs
//!   Optional", §11-12 "Parameterized Capabilities", §17 "Durable vs
//!   Ephemeral", §76 "Direction": [`descriptor::CapabilityDescriptor`],
//!   [`descriptor::CapabilityRequirement`], [`descriptor::CapabilityParameters`]
//!   (a closed typed enum, never `HashMap<String, String>`, per §12's
//!   explicit instruction), [`descriptor::CapabilityLifetime`],
//!   [`descriptor::CapabilityDirection`], [`descriptor::BoundedBytes`],
//!   [`descriptor::CapabilityBits`] (with a real bitwise-AND
//!   intersection, §19)
//! - [`set`] — §27 "Canonical Encoding", §61 "Capability Advertisement
//!   Size": [`set::CapabilitySet`], a bounded collection that
//!   maintains canonical `(namespace, code, version)` order on every
//!   insert rather than sorting lazily before use
//! - [`hash`] — §26, §27, §94: [`hash::CapabilitySetHash`], a real
//!   blake3-over-canonical-postcard hash, tested for insertion-order
//!   independence (the property §26's "both peers verify equality"
//!   actually depends on)
//! - [`registry`] — §65-69 "Capability Registry" / "Definition" /
//!   "Security Class" / "Mandatory Unknown Critical Capability" /
//!   "Capability Dependency": [`registry::CapabilityRegistry`] with
//!   real validation — §68's unknown-required rejection and §69's
//!   dependency-consistency check, both tested against both the
//!   pass and fail case
//! - [`error`] — §159 "Error Model": [`error::CapabilityNegotiationError`],
//!   reconciled against §103's overlapping sketch (see that module's
//!   doc comment)
//!
//! ## Deliberately not attempted this pass
//!
//! Everything in §161 Phase 2 onward: `CapabilityAdvertisement` (§13)
//! and the authenticated-session binding it needs (§13-15, needs Part
//! 02 identity types this crate doesn't depend on yet), the actual
//! `negotiate()` intersection/policy-filter pipeline (§18-24, §157's
//! `CapabilityClient`/`CapabilityPolicy`/`CapabilityQuery`), the
//! two-phase confirmation and transcript-hash handshake (§25-26), the
//! extension negotiator trait and its files/DTN/media integrations
//! (§33-41, needs Parts 01/05/06 wiring), dynamic capability updates
//! and epochs (§45-53), the capability cache (§49-52), and the wire
//! format (§100-102). None of these are guessed at here — the module
//! list above is genuinely everything built and verified this pass.

pub mod descriptor;
pub mod error;
pub mod hash;
pub mod id;
pub mod registry;
pub mod set;
pub mod version;

pub use descriptor::{
    BoundedBytes, BoundedBytesError, CapabilityBits, CapabilityDescriptor, CapabilityDirection,
    CapabilityLifetime, CapabilityParameters, CapabilityRequirement, MAX_PARAMETER_BYTES,
};
pub use error::CapabilityNegotiationError;
pub use hash::CapabilitySetHash;
pub use id::{CapabilityId, CapabilityNamespace, NamespaceId};
pub use registry::{
    CapabilityDefinition, CapabilityDependency, CapabilityRegistry, ParameterSchema, SecurityClass,
};
pub use set::{CapabilitySet, CapabilitySetError, MAX_CAPABILITIES_PER_SET};
pub use version::CapabilityVersion;
