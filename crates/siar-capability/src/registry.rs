//! §65 "Capability Registry", §66 "Capability Definition", §67
//! "Security Class", §68 "Mandatory Unknown Critical Capability", §69
//! "Capability Dependency".

use crate::descriptor::CapabilityParameters;
use crate::error::CapabilityNegotiationError;
use crate::id::CapabilityId;
use crate::set::CapabilitySet;
use crate::version::CapabilityVersion;
use std::collections::HashMap;

/// §67: unknown-critical handling (§68) is conservative in proportion
/// to this classification — the registry itself only enforces the
/// binary required/unknown case (§68); using the finer-grained class
/// to modulate policy (e.g. auto-reject `SecuritySensitive` from
/// unverified peers) is a policy-layer concern for Phase 4 (§161),
/// not implemented here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SecurityClass {
    Informational,
    Functional,
    SecuritySensitive,
    Critical,
}

/// §66: shape validation for a capability's parameters, checked
/// against the *variant* of an advertised [`CapabilityParameters`]
/// (not its value — range/limit checking is §19's negotiation-time
/// concern, deferred). Not in the spec's own §66 sketch by name, but
/// §7/§12 both insist on typed parameters over a stringly-typed map,
/// and a definition with no way to check "does this advertisement even
/// have the right *kind* of parameter for this capability" leaves that
/// insistence unenforced — so it's added here as the real missing
/// piece, the same way `mark_eligible` was added to close a gap in the
/// [[resilient-mesh]] Part 06 crate's own trait sketch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ParameterSchema {
    None,
    U32,
    U64,
    RangeU32,
    BitSet,
    Bytes,
}

impl ParameterSchema {
    fn matches(self, params: &CapabilityParameters) -> bool {
        matches!(
            (self, params),
            (ParameterSchema::None, CapabilityParameters::None)
                | (ParameterSchema::U32, CapabilityParameters::U32(_))
                | (ParameterSchema::U64, CapabilityParameters::U64(_))
                | (
                    ParameterSchema::RangeU32,
                    CapabilityParameters::RangeU32 { .. }
                )
                | (ParameterSchema::BitSet, CapabilityParameters::BitSet(_))
                | (ParameterSchema::Bytes, CapabilityParameters::Bytes(_))
        )
    }
}

/// §66: the registry's canonical entry for one [`CapabilityId`] —
/// name, max supported version, expected parameter shape, and
/// security classification.
#[derive(Debug, Clone)]
pub struct CapabilityDefinition {
    pub id: CapabilityId,
    pub name: &'static str,
    pub max_version: CapabilityVersion,
    pub parameter_schema: ParameterSchema,
    pub security_class: SecurityClass,
}

/// §69: "Some capabilities depend on others" — e.g.
/// `files.parallel_chunks requires files.chunking`. Modeled here as
/// one dependency edge (`capability` requires the single `requires`
/// id) rather than the spec's own sketch of `requires: CapabilitySet`
/// (a full set of descriptors): a dependency only needs to assert
/// *presence* of the prerequisite capability id in the same
/// advertisement (§69's own example is presence-shaped, "requires
/// files.chunking", not "requires files.chunking at exactly version
/// X") — requiring full descriptor equality would reject an
/// advertisement that both peers agree is dependency-consistent purely
/// because of an unrelated field like version. Multiple dependency
/// edges from the same capability are just multiple registered
/// [`CapabilityDependency`] values, which [`CapabilityRegistry::validate`]
/// already checks against without needing a set-of-sets on this type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CapabilityDependency {
    pub capability: CapabilityId,
    pub requires: CapabilityId,
}

/// §65: "canonical names, validation, version semantics, parameter
/// codec, security classification... No global singleton required" —
/// this registry is a plain owned value, constructed and passed by the
/// caller rather than reached for through global state.
#[derive(Debug, Clone, Default)]
pub struct CapabilityRegistry {
    definitions: HashMap<CapabilityId, CapabilityDefinition>,
    dependencies: Vec<CapabilityDependency>,
}

impl CapabilityRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, definition: CapabilityDefinition) {
        self.definitions.insert(definition.id, definition);
    }

    pub fn register_dependency(&mut self, dependency: CapabilityDependency) {
        self.dependencies.push(dependency);
    }

    pub fn definition(&self, id: &CapabilityId) -> Option<&CapabilityDefinition> {
        self.definitions.get(id)
    }

    /// Runs the registry-level checks this crate can make without a
    /// remote peer's advertisement or a policy object (both later
    /// pieces, §161 Phase 2/4): §68's unknown-required rejection, and
    /// §69's dependency-consistency rejection. Negotiation-time
    /// concerns (intersection, version negotiation, security-floor
    /// enforcement) are deliberately not attempted here.
    pub fn validate(&self, set: &CapabilitySet) -> Result<(), CapabilityNegotiationError> {
        for descriptor in set.iter() {
            // §68: "If remote marks: unknown critical capability
            // required → reject... Never silently ignore." A required
            // capability this registry has no definition for is
            // exactly that case (§8 restates the same rule for the
            // general required/unknown case).
            if descriptor.requirement == crate::descriptor::CapabilityRequirement::Required
                && !self.definitions.contains_key(&descriptor.id)
            {
                return Err(CapabilityNegotiationError::MissingRequired(descriptor.id));
            }

            if let Some(def) = self.definitions.get(&descriptor.id) {
                if !def.parameter_schema.matches(&descriptor.parameters) {
                    return Err(CapabilityNegotiationError::Malformed);
                }
            }
        }

        for dependency in &self.dependencies {
            if set.contains(&dependency.capability) && !set.contains(&dependency.requires) {
                return Err(CapabilityNegotiationError::DependencyViolation {
                    capability: dependency.capability,
                    missing: dependency.requires,
                });
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::descriptor::{CapabilityDescriptor, CapabilityRequirement};
    use crate::id::CapabilityNamespace;

    fn files_chunking() -> CapabilityId {
        CapabilityId::new(CapabilityNamespace::Files, 1)
    }

    fn files_parallel() -> CapabilityId {
        CapabilityId::new(CapabilityNamespace::Files, 2)
    }

    fn desc(id: CapabilityId, requirement: CapabilityRequirement) -> CapabilityDescriptor {
        CapabilityDescriptor::new(
            id,
            CapabilityVersion::new(1, 0),
            requirement,
            CapabilityParameters::None,
        )
    }

    #[test]
    fn unknown_required_capability_is_rejected() {
        // §68 / §8: never silently ignore an unrecognized *required*
        // capability.
        let registry = CapabilityRegistry::new();
        let mut set = CapabilitySet::new();
        set.insert(desc(files_chunking(), CapabilityRequirement::Required))
            .unwrap();

        assert_eq!(
            registry.validate(&set),
            Err(CapabilityNegotiationError::MissingRequired(files_chunking()))
        );
    }

    #[test]
    fn unknown_optional_capability_is_ignored_safely() {
        // §8: the mirror-image rule — unknown *optional* capabilities
        // must not break validation.
        let registry = CapabilityRegistry::new();
        let mut set = CapabilitySet::new();
        set.insert(desc(files_chunking(), CapabilityRequirement::Optional))
            .unwrap();

        assert!(registry.validate(&set).is_ok());
    }

    #[test]
    fn dependency_violation_is_rejected() {
        // §69's own example: files.parallel_chunks requires
        // files.chunking.
        let mut registry = CapabilityRegistry::new();
        registry.register_dependency(CapabilityDependency {
            capability: files_parallel(),
            requires: files_chunking(),
        });

        let mut set = CapabilitySet::new();
        set.insert(desc(files_parallel(), CapabilityRequirement::Optional))
            .unwrap();
        // files_chunking deliberately not inserted.

        assert_eq!(
            registry.validate(&set),
            Err(CapabilityNegotiationError::DependencyViolation {
                capability: files_parallel(),
                missing: files_chunking(),
            })
        );
    }

    #[test]
    fn dependency_satisfied_when_prerequisite_present() {
        let mut registry = CapabilityRegistry::new();
        registry.register_dependency(CapabilityDependency {
            capability: files_parallel(),
            requires: files_chunking(),
        });

        let mut set = CapabilitySet::new();
        set.insert(desc(files_parallel(), CapabilityRequirement::Optional))
            .unwrap();
        set.insert(desc(files_chunking(), CapabilityRequirement::Optional))
            .unwrap();

        assert!(registry.validate(&set).is_ok());
    }

    #[test]
    fn parameter_shape_mismatch_is_malformed() {
        let mut registry = CapabilityRegistry::new();
        registry.register(CapabilityDefinition {
            id: files_chunking(),
            name: "files.chunking",
            max_version: CapabilityVersion::new(1, 0),
            parameter_schema: ParameterSchema::U32,
            security_class: SecurityClass::Functional,
        });

        let mut set = CapabilitySet::new();
        // Advertised with no parameters, but the registry expects U32.
        set.insert(desc(files_chunking(), CapabilityRequirement::Optional))
            .unwrap();

        assert_eq!(
            registry.validate(&set),
            Err(CapabilityNegotiationError::Malformed)
        );
    }
}
