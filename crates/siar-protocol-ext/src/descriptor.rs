//! Extension descriptors — spec §35 "Extension Descriptor", plus the
//! pieces it's built from: §7 (version), §11 (requirement), §19
//! (limits).

use crate::capability::CapabilitySet;
use crate::identifier::{ProtocolId, ProtocolMajor, ProtocolMinor};

/// Combines [`ProtocolMajor`] and [`ProtocolMinor`] into the one field
/// spec §35's `ExtensionDescriptor.version: ExtensionVersion` names —
/// the spec doesn't spell out `ExtensionVersion`'s own fields
/// anywhere it was given to this pass, so this is the direct, obvious
/// combination of the two version concepts §7 already defines
/// separately, not a guess at unstated extra fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ExtensionVersion {
    pub major: ProtocolMajor,
    pub minor: ProtocolMinor,
}

/// spec §11 "Mandatory and Optional Extensions", verbatim enum.
/// "An unsupported optional extension must not tear down the whole
/// session" — enforced by [`crate::negotiation::negotiate`], not by
/// this type itself (this is just the declared requirement level).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ExtensionRequirement {
    Required,
    Optional,
}

/// spec §19 "Per-Extension Resource Limits", verbatim struct.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ExtensionLimits {
    pub max_frame_size: usize,
    pub max_in_flight_frames: usize,
    pub max_concurrent_streams: usize,
    pub max_buffered_bytes: usize,
}

/// spec §35's `ExtensionDescriptor`. One field renamed from what §35
/// shows verbatim: `requirements: ExtensionRequirements` (plural)
/// there — that plural type's own fields are never given anywhere in
/// this document, so rather than invent an unspecified struct this
/// uses [`ExtensionRequirement`] (singular, §11's actual verbatim
/// type) directly. If a future spec part defines what
/// `ExtensionRequirements` (plural) actually holds beyond a single
/// requirement level — e.g. per-operation requirements, per spec
/// §31 "Operation-Level Required Capabilities" — this field is the one
/// to revisit.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExtensionDescriptor {
    pub id: ProtocolId,
    pub version: ExtensionVersion,
    pub capabilities: CapabilitySet,
    pub requirement: ExtensionRequirement,
    pub limits: ExtensionLimits,
}

/// The result of negotiating one [`ExtensionDescriptor`] against a
/// remote peer's advertised capabilities for the same [`ProtocolId`] —
/// spec §10's "negotiated: messaging/1 [text, reply]" line, given
/// structure. `session_id` is spec §17's "Session-Local Extension
/// IDs" — assigned by [`crate::negotiation::negotiate`], not chosen by
/// the extension itself.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NegotiatedExtension {
    pub id: ProtocolId,
    pub session_id: SessionLocalExtensionId,
    pub capabilities: CapabilitySet,
}

/// spec §17: "The numeric mapping is session-local. This reduces
/// repeated framing overhead while retaining stable global protocol
/// identities." Deliberately not `Copy`-derived-away-from-newtype —
/// kept as a distinct type from a bare `u16` so a session-local ID is
/// never accidentally compared against or substituted for a
/// [`crate::capability::CapabilityId`] or any other small-integer
/// newtype in this crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct SessionLocalExtensionId(pub u16);
