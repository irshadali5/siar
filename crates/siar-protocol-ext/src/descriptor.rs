//! Extension descriptors — spec §35 "Extension Descriptor", plus the
//! pieces it's built from: §7 (version), §11 (requirement), §19
//! (limits).

use crate::capability::CapabilitySet;
use crate::identifier::{ProtocolId, ProtocolMajor, ProtocolMinor};
use crate::security::SecurityRequirements;

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

/// spec §47, verbatim enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ExtensionStability {
    Stable,
    Experimental,
    Internal,
}

/// spec §47: "Experimental protocols should require explicit opt-in in
/// production builds." The one concrete, checkable rule §47 states —
/// `Stable` and `Internal` never require opt-in (an internal-only
/// extension isn't reachable by ordinary negotiation in the first
/// place, so the production-build gate is specifically about
/// `Experimental`).
pub fn requires_opt_in(stability: ExtensionStability, production_build: bool) -> bool {
    matches!(stability, ExtensionStability::Experimental) && production_build
}

/// spec §35's `ExtensionDescriptor`. One field renamed from what §35
/// shows verbatim: `requirements: ExtensionRequirements` (plural)
/// there — that plural type's own fields are never given anywhere in
/// this document, so rather than invent an unspecified struct this
/// uses [`ExtensionRequirement`] (singular, §11's actual verbatim
/// type) directly.
///
/// `required_capabilities` is this pass's answer to that same
/// revisit note, now that §30/§31 are implemented: spec §30
/// distinguishes "unknown optional capability → ignore" (already true
/// for free — [`CapabilitySet::intersect`] silently drops anything the
/// remote doesn't share) from "unknown required capability → reject
/// extension operation/negotiation", which needs the extension to be
/// able to say *which* of its capabilities aren't optional in the
/// first place. Not part of `capabilities` itself — a capability can
/// be advertised (offered, negotiable) without being required for the
/// extension to be usable at all; `required_capabilities` should
/// always be a subset of `capabilities`, though this type doesn't
/// enforce that itself (see [`crate::negotiation::negotiate`], which
/// is where a required capability actually gets checked against what
/// negotiation produced).
///
/// `security` (§32) and `stability` (§47) are this round's two further
/// additions to the same struct — both genuinely per-extension
/// declared properties §35's own "used for: negotiation, documentation,
/// diagnostics, testing, compatibility tooling" list already implies
/// belong here, not bolted on separately. [`generate_documentation`]
/// (§36) is the first real consumer of both.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExtensionDescriptor {
    pub id: ProtocolId,
    pub version: ExtensionVersion,
    pub capabilities: CapabilitySet,
    pub required_capabilities: CapabilitySet,
    pub requirement: ExtensionRequirement,
    pub limits: ExtensionLimits,
    pub security: SecurityRequirements,
    pub stability: ExtensionStability,
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

/// spec §36: "Typed descriptors allow tooling to generate: supported
/// extensions, version table, capabilities, resource limits, security
/// requirements." One human-readable string covering exactly those
/// five things, generated from real [`ExtensionDescriptor`] data
/// rather than hand-maintained prose — §36's own stated point ("this
/// reduces documentation drift") only holds if the generator reads the
/// same struct negotiation itself reads, which this does.
pub fn generate_documentation(descriptors: &[ExtensionDescriptor]) -> String {
    let mut out = String::from("# Supported Extensions\n\n");
    for d in descriptors {
        out.push_str(&format!(
            "## {}\n\n\
             - version: {}.{}\n\
             - stability: {:?}\n\
             - requirement: {:?}\n\
             - capabilities: {:?}\n\
             - required capabilities: {:?}\n\
             - resource limits: max_frame_size={}, max_in_flight_frames={}, max_concurrent_streams={}, max_buffered_bytes={}\n\
             - security: authenticated_peer={}, e2ee_required={}, authorization_required={}, allow_anonymous={}\n\n",
            d.id.canonical_name(),
            d.version.major.0,
            d.version.minor.0,
            d.stability,
            d.requirement,
            d.capabilities.values,
            d.required_capabilities.values,
            d.limits.max_frame_size,
            d.limits.max_in_flight_frames,
            d.limits.max_concurrent_streams,
            d.limits.max_buffered_bytes,
            d.security.authenticated_peer,
            d.security.e2ee_required,
            d.security.authorization_required,
            d.security.allow_anonymous,
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::CapabilityId;
    use crate::identifier::{NamespaceId, ProtocolName};

    fn sample_descriptor() -> ExtensionDescriptor {
        ExtensionDescriptor {
            id: ProtocolId::new(
                NamespaceId::new("org.example").unwrap(),
                ProtocolName::new("messaging").unwrap(),
                ProtocolMajor(1),
            ),
            version: ExtensionVersion {
                major: ProtocolMajor(1),
                minor: ProtocolMinor(0),
            },
            capabilities: CapabilitySet::new([CapabilityId(1), CapabilityId(2)]),
            required_capabilities: CapabilitySet::new([CapabilityId(1)]),
            requirement: ExtensionRequirement::Required,
            limits: ExtensionLimits {
                max_frame_size: 1024,
                max_in_flight_frames: 16,
                max_concurrent_streams: 4,
                max_buffered_bytes: 65536,
            },
            security: SecurityRequirements::messaging_default(),
            stability: ExtensionStability::Stable,
        }
    }

    #[test]
    fn spec_47_experimental_in_production_requires_opt_in() {
        assert!(requires_opt_in(ExtensionStability::Experimental, true));
    }

    #[test]
    fn spec_47_experimental_outside_production_does_not_require_opt_in() {
        assert!(!requires_opt_in(ExtensionStability::Experimental, false));
    }

    #[test]
    fn spec_47_stable_never_requires_opt_in() {
        assert!(!requires_opt_in(ExtensionStability::Stable, true));
        assert!(!requires_opt_in(ExtensionStability::Internal, true));
    }

    #[test]
    fn spec_36_generated_documentation_covers_all_five_named_things() {
        let doc = generate_documentation(&[sample_descriptor()]);
        // "supported extensions"
        assert!(doc.contains("org.example/messaging/1"));
        // "version table"
        assert!(doc.contains("1.0"));
        // "capabilities"
        assert!(doc.contains("capabilities"));
        // "resource limits"
        assert!(doc.contains("max_frame_size=1024"));
        // "security requirements"
        assert!(doc.contains("e2ee_required=true"));
    }
}
