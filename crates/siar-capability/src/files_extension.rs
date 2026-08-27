//! §35 "File Capabilities", §36 "File Limit Negotiation".
//!
//! A concrete, real [`ExtensionNegotiator`] for the `files/1`
//! extension namespace — proof that the trait built in `extension.rs`
//! actually composes with everything built in earlier passes
//! (registry validation, dependency checking, intersection, policy
//! filtering), not just a type-checks-in-isolation sketch. "Part 05
//! consumes the negotiated set" (§35) — this module produces exactly
//! that negotiated [`CapabilitySet`] and stops; it does not reach into
//! `siar-blob-manifest` itself, which stays a downstream integration
//! concern for whichever crate owns that wiring.

use crate::descriptor::{CapabilityDescriptor, CapabilityParameters, CapabilityRequirement};
use crate::extension::ExtensionNegotiator;
use crate::id::{CapabilityId, CapabilityNamespace};
use crate::registry::{
    CapabilityDefinition, CapabilityDependency, CapabilityRegistry, ParameterSchema, SecurityClass,
};
use crate::set::CapabilitySet;
use crate::version::CapabilityVersion;

/// §35's own list, assigned stable codes in listed order. Kept as
/// `pub const`s rather than a private convention so a caller wiring
/// this into Part 05 later can refer to the same ids this crate
/// negotiates, without re-guessing the numbering.
pub const FIXED_CHUNKING: u32 = 0;
pub const RESUME: u32 = 1;
pub const PARALLEL_CHUNKS: u32 = 2;
pub const CIPHERTEXT_ADDRESSING: u32 = 3;
pub const PARTIAL_READ: u32 = 4;
pub const MULTI_SOURCE: u32 = 5;
pub const MAX_CHUNK_SIZE: u32 = 6;

fn id(code: u32) -> CapabilityId {
    CapabilityId::new(CapabilityNamespace::Files, code)
}

/// A `files/1` capability advertiser/negotiator with a caller-chosen
/// max chunk size (§36's own worked example: "Sender: max chunk = 4
/// MiB. Receiver: max chunk = 1 MiB. Effective: 1 MiB.").
///
/// `fixed_chunking` and `ciphertext_addressing` are advertised as
/// [`CapabilityRequirement::Required`] — without chunking there is no
/// `files/1` transfer at all, and `siar_blob_manifest`'s whole design
/// (per [[resilient-mesh]] project memory) is ciphertext-only
/// addressing by construction, so a peer that can't do that isn't
/// really speaking `files/1`. Every other listed capability is
/// advertised [`CapabilityRequirement::Optional`], matching §35's own
/// framing of them as a feature list rather than a minimum bar.
#[derive(Debug, Clone)]
pub struct FilesExtensionNegotiator {
    max_chunk_size: u32,
    registry: CapabilityRegistry,
}

impl FilesExtensionNegotiator {
    pub fn new(max_chunk_size: u32) -> Self {
        let mut registry = CapabilityRegistry::new();
        for (code, name, schema, class) in [
            (
                FIXED_CHUNKING,
                "files.fixed_chunking",
                ParameterSchema::None,
                SecurityClass::Functional,
            ),
            (
                RESUME,
                "files.resume",
                ParameterSchema::None,
                SecurityClass::Functional,
            ),
            (
                PARALLEL_CHUNKS,
                "files.parallel_chunks",
                ParameterSchema::None,
                SecurityClass::Functional,
            ),
            (
                CIPHERTEXT_ADDRESSING,
                "files.ciphertext_addressing",
                ParameterSchema::None,
                SecurityClass::SecuritySensitive,
            ),
            (
                PARTIAL_READ,
                "files.partial_read",
                ParameterSchema::None,
                SecurityClass::Functional,
            ),
            (
                MULTI_SOURCE,
                "files.multi_source",
                ParameterSchema::None,
                SecurityClass::Functional,
            ),
            (
                MAX_CHUNK_SIZE,
                "files.max_chunk_size",
                ParameterSchema::U32,
                SecurityClass::Functional,
            ),
        ] {
            registry.register(CapabilityDefinition {
                id: id(code),
                name,
                max_version: CapabilityVersion::new(1, 0),
                parameter_schema: schema,
                security_class: class,
            });
        }
        // §69's own worked example: parallel_chunks requires chunking.
        registry.register_dependency(CapabilityDependency {
            capability: id(PARALLEL_CHUNKS),
            requires: id(FIXED_CHUNKING),
        });

        Self {
            max_chunk_size,
            registry,
        }
    }
}

impl ExtensionNegotiator for FilesExtensionNegotiator {
    fn advertise(&self) -> CapabilitySet {
        let mut set = CapabilitySet::new();
        let v1 = CapabilityVersion::new(1, 0);
        let descriptors = [
            CapabilityDescriptor::new(
                id(FIXED_CHUNKING),
                v1,
                CapabilityRequirement::Required,
                CapabilityParameters::None,
            ),
            CapabilityDescriptor::new(
                id(RESUME),
                v1,
                CapabilityRequirement::Optional,
                CapabilityParameters::None,
            ),
            CapabilityDescriptor::new(
                id(PARALLEL_CHUNKS),
                v1,
                CapabilityRequirement::Optional,
                CapabilityParameters::None,
            ),
            CapabilityDescriptor::new(
                id(CIPHERTEXT_ADDRESSING),
                v1,
                CapabilityRequirement::Required,
                CapabilityParameters::None,
            ),
            CapabilityDescriptor::new(
                id(PARTIAL_READ),
                v1,
                CapabilityRequirement::Optional,
                CapabilityParameters::None,
            ),
            CapabilityDescriptor::new(
                id(MULTI_SOURCE),
                v1,
                CapabilityRequirement::Optional,
                CapabilityParameters::None,
            ),
            CapabilityDescriptor::new(
                id(MAX_CHUNK_SIZE),
                v1,
                CapabilityRequirement::Required,
                CapabilityParameters::U32(self.max_chunk_size),
            ),
        ];
        for descriptor in descriptors {
            set.insert(descriptor)
                .expect("7 descriptors is well under MAX_CAPABILITIES_PER_SET");
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
    fn effective_max_chunk_size_is_the_minimum_of_both_sides() {
        // §36's exact worked example.
        let sender = FilesExtensionNegotiator::new(4 * 1024 * 1024);
        let receiver = FilesExtensionNegotiator::new(1024 * 1024);

        let negotiated = sender
            .negotiate(&receiver.advertise(), &CapabilityPolicy::new())
            .unwrap();

        assert_eq!(
            negotiated.get(&id(MAX_CHUNK_SIZE)).unwrap().parameters,
            CapabilityParameters::U32(1024 * 1024)
        );
    }

    #[test]
    fn negotiation_is_symmetric_between_two_files_peers() {
        let a = FilesExtensionNegotiator::new(4 * 1024 * 1024);
        let b = FilesExtensionNegotiator::new(1024 * 1024);

        let from_a = a
            .negotiate(&b.advertise(), &CapabilityPolicy::new())
            .unwrap();
        let from_b = b
            .negotiate(&a.advertise(), &CapabilityPolicy::new())
            .unwrap();

        let a_ids: Vec<_> = from_a.iter().map(|d| d.id).collect();
        let b_ids: Vec<_> = from_b.iter().map(|d| d.id).collect();
        assert_eq!(a_ids, b_ids);
    }

    #[test]
    fn all_required_capabilities_negotiate_successfully_between_matching_peers() {
        let a = FilesExtensionNegotiator::new(4 * 1024 * 1024);
        let b = FilesExtensionNegotiator::new(4 * 1024 * 1024);

        let negotiated = a
            .negotiate(&b.advertise(), &CapabilityPolicy::new())
            .unwrap();
        assert!(negotiated.contains(&id(FIXED_CHUNKING)));
        assert!(negotiated.contains(&id(CIPHERTEXT_ADDRESSING)));
        assert!(negotiated.contains(&id(MAX_CHUNK_SIZE)));
    }

    #[test]
    fn dependency_violation_surfaces_through_the_trait_negotiate_path() {
        // parallel_chunks with no fixed_chunking should never occur
        // from `advertise()` itself (both are always advertised
        // together here), so this test builds the inconsistent case
        // directly to prove `ExtensionNegotiator::negotiate`'s
        // provided method really does run registry.validate() (§69),
        // not just the intersection step.
        let negotiator = FilesExtensionNegotiator::new(1024 * 1024);

        let mut inconsistent = CapabilitySet::new();
        inconsistent
            .insert(CapabilityDescriptor::new(
                id(PARALLEL_CHUNKS),
                CapabilityVersion::new(1, 0),
                CapabilityRequirement::Optional,
                CapabilityParameters::None,
            ))
            .unwrap();
        // fixed_chunking deliberately omitted.

        let err = negotiator
            .negotiate(&inconsistent, &CapabilityPolicy::new())
            .unwrap_err();
        assert_eq!(
            err,
            crate::error::CapabilityNegotiationError::DependencyViolation {
                capability: id(PARALLEL_CHUNKS),
                missing: id(FIXED_CHUNKING),
            }
        );
    }
}
