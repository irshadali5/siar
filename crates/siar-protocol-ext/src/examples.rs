//! spec §71-77: the six worked extension examples (ERP, emergency,
//! files, messaging, presence, calls/media) plus §77 "Protocol
//! Composition". These are vocabulary — capability enums an extension
//! author would use when building a real `ExtensionDescriptor` for
//! that domain — not implementations of messaging/files/calls
//! themselves, which live in `siar-messaging`/other crates entirely
//! outside this one (see this crate's own "No wire integration" note).

use crate::identifier::{NamespaceId, ProtocolId, ProtocolMajor, ProtocolName};

/// spec §71: `com.example.erp/approval/1` — a third-party namespace
/// extension needs no core protocol change to exist at all. This
/// function exists only so that claim is a real, checkable value
/// rather than prose — see this module's own test for the actual
/// "negotiates exactly like a core extension" proof.
pub fn erp_approval_example() -> ProtocolId {
    ProtocolId::new(
        NamespaceId::new("com.example.erp").unwrap(),
        ProtocolName::new("approval").unwrap(),
        ProtocolMajor(1),
    )
}

/// spec §72's own three named protocol families, verbatim.
pub fn emergency_sos() -> ProtocolId {
    emergency_family("emergency-sos")
}
pub fn emergency_alert() -> ProtocolId {
    emergency_family("emergency-alert")
}
pub fn emergency_resource() -> ProtocolId {
    emergency_family("emergency-resource")
}
fn emergency_family(name: &str) -> ProtocolId {
    ProtocolId::new(
        NamespaceId::new("org.siar").unwrap(),
        ProtocolName::new(name).unwrap(),
        ProtocolMajor(1),
    )
}

/// spec §72's own five "critical fields," verbatim — `dtn_permission`
/// stays a plain `bool` (spec gives no richer shape for it here, and
/// `siar-dtn`/`siar-dtn-bundle` already own the real DTN permission
/// model elsewhere in this workspace — see this crate's documented
/// reconciliation note in `/areas/resilient-mesh.md` about those two
/// existing DTN models; this field is not a third one, just a flag an
/// emergency extension would set when constructing a real bundle).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EmergencyCriticalFields {
    pub priority: crate::lifecycle::TrafficPriority,
    pub expiry_millis: u64,
    pub signature_authenticity_required: bool,
    pub dtn_permission: bool,
    pub location_privacy: LocationPrivacy,
}

/// spec §72 names "location privacy semantics" as a critical field but
/// never gives its own values — the smallest closed set that makes an
/// emergency extension's location handling an explicit choice rather
/// than an unstated default, without inventing detail spec §72 doesn't
/// give.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LocationPrivacy {
    Precise,
    Approximate,
    Withheld,
}

/// spec §73's own six file capabilities, verbatim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FileCapability {
    Manifest,
    RangeRequest,
    Resume,
    ParallelChunks,
    ContentAddressing,
    EncryptedMetadata,
}

/// spec §73's own stream layout: one `Control` stream plus numbered
/// `data-N` streams — `StreamRole::Data(0)` renders as "data-0" etc.,
/// matching the spec's literal naming rather than an opaque index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum StreamRole {
    Control,
    Data(u32),
}

/// spec §74's own seven messaging capabilities, verbatim.
/// `CustomContentReference` is what §74's own "attachments are
/// represented as content/blob references; actual transfer belongs to
/// files/1" line refers to — deliberately not a payload-carrying
/// variant, just the vocabulary marker; the real reference type lives
/// in `siar-messaging`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MessagingCapability {
    Text,
    Reply,
    Edit,
    Reaction,
    DeliveryReceipt,
    ReadReceipt,
    CustomContentReference,
}

/// spec §75's own three presence capabilities, verbatim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PresenceCapability {
    BasicPresence,
    Typing,
    ActivityHint,
}

/// spec §75's own four listed properties ("small frames, short
/// expiry, usually not DTN, loss tolerant"), turned into a real
/// [`crate::routing::RoutingRequirements`] rather than left as
/// separate prose bullets — this is what those properties actually
/// mean in terms this crate already has code for.
pub fn presence_default_routing() -> crate::routing::RoutingRequirements {
    crate::routing::RoutingRequirements {
        realtime_requirement: true,
        maximum_age_millis: Some(30_000), // "short expiry"
        durability: crate::routing::DeliveryClass::Realtime, // "loss tolerant" -> not Durable
        forwarding_permission: false, // "usually not DTN"
        size_class: crate::routing::SizeClass::Small, // "small frames"
        priority: crate::lifecycle::TrafficPriority::Interactive,
    }
}

/// spec §76: "Separate call control from media transport where
/// useful" — two enums, not one, mirroring that separation in the
/// type system itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CallsCapability {
    Initiate,
    Ring,
    Answer,
    Hangup,
}

/// spec §76's own five media capability areas, verbatim. "Codec
/// implementation remains outside the core protocol" — this variant
/// names *that a codec is negotiable*, never encodes/decodes one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MediaCapabilityArea {
    AudioCodecs,
    VideoCodecs,
    Resolution,
    FrameRate,
    HardwareOrSoftwareCapability,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::{CapabilityId, CapabilitySet};
    use crate::descriptor::{
        ExtensionDescriptor, ExtensionLimits, ExtensionRequirement, ExtensionStability,
        ExtensionVersion,
    };
    use crate::identifier::ProtocolMinor;
    use crate::negotiation::{negotiate, RemoteAdvertisement};
    use crate::security::SecurityRequirements;

    fn minimal_descriptor(id: ProtocolId, requirement: ExtensionRequirement) -> ExtensionDescriptor {
        ExtensionDescriptor {
            id,
            version: ExtensionVersion {
                major: ProtocolMajor(1),
                minor: ProtocolMinor(0),
            },
            capabilities: CapabilitySet::new([CapabilityId(1)]),
            required_capabilities: CapabilitySet::default(),
            requirement,
            limits: ExtensionLimits {
                max_frame_size: 65536,
                max_in_flight_frames: 32,
                max_concurrent_streams: 4,
                max_buffered_bytes: 1 << 20,
            },
            security: SecurityRequirements::messaging_default(),
            stability: ExtensionStability::Stable,
        }
    }

    #[test]
    fn spec_71_a_third_party_namespace_negotiates_exactly_like_a_core_extension() {
        // The actual claim §71 makes: no core protocol change needed.
        // Proven by running the same negotiate() call any core
        // extension would go through, with nothing special-cased for
        // this namespace anywhere in negotiate()'s own implementation.
        let id = erp_approval_example();
        let local = vec![minimal_descriptor(id.clone(), ExtensionRequirement::Optional)];
        let remote = vec![RemoteAdvertisement {
            id,
            capabilities: CapabilitySet::new([CapabilityId(1)]),
        }];
        let result = negotiate(&local, &remote).unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn spec_72_emergency_protocol_families_are_distinct_ids() {
        assert_ne!(emergency_sos(), emergency_alert());
        assert_ne!(emergency_alert(), emergency_resource());
    }

    #[test]
    fn spec_75_presence_routing_reflects_all_four_named_properties() {
        let routing = presence_default_routing();
        assert!(routing.realtime_requirement); // implied by "short expiry"/loss tolerant
        assert_eq!(routing.maximum_age_millis, Some(30_000)); // "short expiry"
        assert!(!routing.forwarding_permission); // "usually not DTN"
        assert_eq!(routing.size_class, crate::routing::SizeClass::Small); // "small frames"
    }

    #[test]
    fn spec_77_six_distinct_extensions_compose_in_one_negotiation_with_no_central_special_casing() {
        // spec §77's own six-item list, verbatim namespaces —
        // negotiated together in a single negotiate() call to prove
        // composition doesn't require a monolith: nothing about
        // negotiate()'s implementation branches on which of these six
        // it's looking at.
        fn descriptor(name: &str) -> ExtensionDescriptor {
            minimal_descriptor(
                ProtocolId::new(
                    NamespaceId::new("org.siar").unwrap(),
                    ProtocolName::new(name).unwrap(),
                    ProtocolMajor(1),
                ),
                ExtensionRequirement::Optional,
            )
        }
        fn advertisement(name: &str) -> RemoteAdvertisement {
            RemoteAdvertisement {
                id: ProtocolId::new(
                    NamespaceId::new("org.siar").unwrap(),
                    ProtocolName::new(name).unwrap(),
                    ProtocolMajor(1),
                ),
                capabilities: CapabilitySet::new([CapabilityId(1)]),
            }
        }

        let names = ["messaging", "files", "presence", "calls", "emergency", "customapp"];
        let local: Vec<_> = names.iter().map(|n| descriptor(n)).collect();
        let remote: Vec<_> = names.iter().map(|n| advertisement(n)).collect();

        let result = negotiate(&local, &remote).unwrap();
        assert_eq!(result.len(), names.len());
    }
}
