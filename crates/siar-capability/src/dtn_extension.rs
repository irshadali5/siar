//! §37 "DTN Capabilities", and §11's own `dtn.max_bundle_size = 1 MiB`
//! parameterized-capability example.
//!
//! A second concrete [`ExtensionNegotiator`], alongside
//! `files_extension.rs`'s, for the `dtn/1` namespace. Unlike
//! `files/1`, the spec gives no worked dependency example for any
//! `dtn/1` capability (§69's only worked example is
//! `files.parallel_chunks` → `files.chunking`) — so, deliberately,
//! this module registers no [`crate::registry::CapabilityDependency`]
//! edge between any of the seven `dtn/1` capabilities. An edge like
//! "`gateway_handoff` requires `relay_ack`" might well be true of the
//! real protocol, but nothing in Parts 06 or 07 states it, and adding
//! it here would be inventing protocol semantics rather than
//! transcribing them.

use crate::descriptor::{CapabilityDescriptor, CapabilityParameters, CapabilityRequirement};
use crate::extension::ExtensionNegotiator;
use crate::id::{CapabilityId, CapabilityNamespace};
use crate::registry::{CapabilityDefinition, CapabilityRegistry, ParameterSchema, SecurityClass};
use crate::set::CapabilitySet;
use crate::version::CapabilityVersion;

/// §37's own list, assigned stable codes in listed order — same
/// convention as `files_extension.rs`'s constants, for the same
/// reason (a future caller wiring this into [[iroh-messenger]]'s
/// `siar-dtn-bundle` shouldn't have to re-guess the numbering).
pub const DIRECT_ONLY: u32 = 0;
pub const RELAY_ACK: u32 = 1;
pub const SPRAY_WAIT: u32 = 2;
pub const GATEWAY_HANDOFF: u32 = 3;
pub const BLOB_CHUNK: u32 = 4;
pub const LOCAL_BROADCAST: u32 = 5;
pub const MAX_BUNDLE_SIZE: u32 = 6;

fn id(code: u32) -> CapabilityId {
    CapabilityId::new(CapabilityNamespace::Dtn, code)
}

/// A `dtn/1` capability advertiser/negotiator with a caller-chosen max
/// bundle size (§11's own example value is 1 MiB).
///
/// `direct_only` and `max_bundle_size` are advertised as
/// [`CapabilityRequirement::Required`]: `siar-dtn-bundle`'s already-
/// built `decide_forwarding` (per [[resilient-mesh]] project memory)
/// treats direct delivery as always winning when available, even
/// preempting spray-and-wait, so a peer that can't do at least direct
/// delivery isn't meaningfully speaking `dtn/1` at all; a bundle size
/// limit is likewise a precondition for bounding any transfer, not an
/// optional feature. `relay_ack`, `spray_wait`, `gateway_handoff`,
/// `blob_chunk`, and `local_broadcast` are advertised
/// [`CapabilityRequirement::Optional`], matching §37's framing of them
/// as a feature list layered on top of the baseline.
#[derive(Debug, Clone)]
pub struct DtnExtensionNegotiator {
    max_bundle_size: u32,
    registry: CapabilityRegistry,
}

impl DtnExtensionNegotiator {
    pub fn new(max_bundle_size: u32) -> Self {
        let mut registry = CapabilityRegistry::new();
        for (code, name, schema) in [
            (DIRECT_ONLY, "dtn.direct_only", ParameterSchema::None),
            (RELAY_ACK, "dtn.relay_ack", ParameterSchema::None),
            (SPRAY_WAIT, "dtn.spray_wait", ParameterSchema::None),
            (GATEWAY_HANDOFF, "dtn.gateway_handoff", ParameterSchema::None),
            (BLOB_CHUNK, "dtn.blob_chunk", ParameterSchema::None),
            (LOCAL_BROADCAST, "dtn.local_broadcast", ParameterSchema::None),
            (MAX_BUNDLE_SIZE, "dtn.max_bundle_size", ParameterSchema::U32),
        ] {
            registry.register(CapabilityDefinition {
                id: id(code),
                name,
                max_version: CapabilityVersion::new(1, 0),
                parameter_schema: schema,
                security_class: SecurityClass::Functional,
            });
        }

        Self {
            max_bundle_size,
            registry,
        }
    }
}

impl ExtensionNegotiator for DtnExtensionNegotiator {
    fn advertise(&self) -> CapabilitySet {
        let mut set = CapabilitySet::new();
        let v1 = CapabilityVersion::new(1, 0);
        let descriptors = [
            CapabilityDescriptor::new(id(DIRECT_ONLY), v1, CapabilityRequirement::Required, CapabilityParameters::None),
            CapabilityDescriptor::new(id(RELAY_ACK), v1, CapabilityRequirement::Optional, CapabilityParameters::None),
            CapabilityDescriptor::new(id(SPRAY_WAIT), v1, CapabilityRequirement::Optional, CapabilityParameters::None),
            CapabilityDescriptor::new(
                id(GATEWAY_HANDOFF),
                v1,
                CapabilityRequirement::Optional,
                CapabilityParameters::None,
            ),
            CapabilityDescriptor::new(id(BLOB_CHUNK), v1, CapabilityRequirement::Optional, CapabilityParameters::None),
            CapabilityDescriptor::new(
                id(LOCAL_BROADCAST),
                v1,
                CapabilityRequirement::Optional,
                CapabilityParameters::None,
            ),
            CapabilityDescriptor::new(
                id(MAX_BUNDLE_SIZE),
                v1,
                CapabilityRequirement::Required,
                CapabilityParameters::U32(self.max_bundle_size),
            ),
        ];
        for descriptor in descriptors {
            set.insert(descriptor).expect("7 descriptors is well under MAX_CAPABILITIES_PER_SET");
        }
        set
    }

    fn registry(&self) -> &CapabilityRegistry {
        &self.registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::CapabilityPolicy;

    #[test]
    fn effective_max_bundle_size_is_the_minimum_of_both_sides() {
        // §11's own example value (1 MiB) used as one side; the other
        // is deliberately larger, so the negotiated result exercises
        // the min() rule rather than just echoing equal inputs.
        let generous = DtnExtensionNegotiator::new(4 * 1024 * 1024);
        let spec_example = DtnExtensionNegotiator::new(1024 * 1024);

        let negotiated = generous
            .negotiate(&spec_example.advertise(), &CapabilityPolicy::new())
            .unwrap();

        assert_eq!(
            negotiated.get(&id(MAX_BUNDLE_SIZE)).unwrap().parameters,
            CapabilityParameters::U32(1024 * 1024)
        );
    }

    #[test]
    fn negotiation_is_symmetric_between_two_dtn_peers() {
        let a = DtnExtensionNegotiator::new(4 * 1024 * 1024);
        let b = DtnExtensionNegotiator::new(1024 * 1024);

        let from_a = a.negotiate(&b.advertise(), &CapabilityPolicy::new()).unwrap();
        let from_b = b.negotiate(&a.advertise(), &CapabilityPolicy::new()).unwrap();

        let a_ids: Vec<_> = from_a.iter().map(|d| d.id).collect();
        let b_ids: Vec<_> = from_b.iter().map(|d| d.id).collect();
        assert_eq!(a_ids, b_ids);
    }

    #[test]
    fn peer_lacking_direct_delivery_fails_negotiation() {
        // §8: an unsupported Required capability is a negotiation
        // failure, not a silent drop — checked here against the
        // concrete dtn/1 baseline rather than only the generic case
        // already covered in negotiate.rs's own tests.
        let full = DtnExtensionNegotiator::new(1024 * 1024);

        let mut relay_only = CapabilitySet::new();
        relay_only
            .insert(CapabilityDescriptor::new(
                id(RELAY_ACK),
                CapabilityVersion::new(1, 0),
                CapabilityRequirement::Optional,
                CapabilityParameters::None,
            ))
            .unwrap();
        relay_only
            .insert(CapabilityDescriptor::new(
                id(MAX_BUNDLE_SIZE),
                CapabilityVersion::new(1, 0),
                CapabilityRequirement::Required,
                CapabilityParameters::U32(1024 * 1024),
            ))
            .unwrap();
        // direct_only deliberately omitted.

        let err = full.negotiate(&relay_only, &CapabilityPolicy::new()).unwrap_err();
        assert_eq!(
            err,
            crate::error::CapabilityNegotiationError::MissingRequired(id(DIRECT_ONLY))
        );
    }

    #[test]
    fn optional_features_negotiate_present_when_both_sides_advertise_them() {
        let a = DtnExtensionNegotiator::new(1024 * 1024);
        let b = DtnExtensionNegotiator::new(1024 * 1024);

        let negotiated = a.negotiate(&b.advertise(), &CapabilityPolicy::new()).unwrap();
        for optional in [RELAY_ACK, SPRAY_WAIT, GATEWAY_HANDOFF, BLOB_CHUNK, LOCAL_BROADCAST] {
            assert!(negotiated.contains(&id(optional)), "expected {optional} to be negotiated");
        }
    }
}
