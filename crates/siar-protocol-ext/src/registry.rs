//! Extension registry and runtime construction — spec §12 "Extension
//! Registry", §13 "Runtime Construction", §14 "Extension Isolation",
//! §15 "Shared Services".
//!
//! What's real here: the registry data structure, its builder-style
//! construction (§13), and [`ExtensionContext`]'s shape as constrained
//! handles rather than `Arc<EntireRuntime>` (§15's explicit warning).
//! What's a deliberate stub, named as such rather than hidden:
//! [`IdentityHandle`], [`SessionHandle`], [`SchedulerHandle`], and
//! [`ResourcePolicy`] are placeholder marker types with no behavior —
//! the real `identity`/`session`/`scheduler`/`resource policy`
//! subsystems spec §15 lists don't exist in this codebase in the
//! shape this document describes (this workspace's actual identity
//! and scheduling code predates this spec and doesn't yet match its
//! vocabulary — see `siar-domain`'s `DeviceId`/`AccountId` and
//! `apps/desktop`'s `bootstrap_messaging` for what exists today). Wiring
//! real handles to real subsystems is a separate, later integration
//! pass, not attempted here.

use crate::descriptor::{ExtensionDescriptor, NegotiatedExtension};
use crate::identifier::ProtocolId;
use crate::lifecycle::ExtensionError;
use std::collections::HashMap;
use std::sync::Arc;

/// Placeholder for the real identity subsystem handle spec §15 lists —
/// see this module's own top doc comment.
#[derive(Debug, Clone)]
pub struct IdentityHandle;

/// Placeholder for the real session handle spec §15 lists — see this
/// module's own top doc comment.
#[derive(Debug, Clone)]
pub struct SessionHandle;

/// Placeholder for the real scheduler handle spec §15 lists — see this
/// module's own top doc comment.
#[derive(Debug, Clone)]
pub struct SchedulerHandle;

/// Placeholder for the real resource policy spec §15 lists — see this
/// module's own top doc comment.
#[derive(Debug, Clone, Default)]
pub struct ResourcePolicy;

/// spec §15's `ExtensionContext`, exactly as given there. "Expose
/// constrained handles, not the entire runtime" / "Avoid:
/// `Arc<EntireRuntime>` inside every extension" — the whole reason
/// this type exists rather than every [`ProtocolExtension`] just
/// taking a runtime reference.
#[derive(Debug, Clone)]
pub struct ExtensionContext {
    pub identity: IdentityHandle,
    pub session: SessionHandle,
    pub scheduler: SchedulerHandle,
    pub resources: ResourcePolicy,
}

/// spec §12's `ProtocolExtension` trait, exactly as given there
/// (`create_handler`'s `negotiated` parameter is spec §12's
/// `NegotiatedExtension`, defined in [`crate::descriptor`] rather than
/// here since it's also `negotiate`'s own return type — see
/// [`crate::negotiation`]).
pub trait ProtocolExtension: Send + Sync {
    fn descriptor(&self) -> ExtensionDescriptor;

    fn create_handler(
        &self,
        negotiated: NegotiatedExtension,
        ctx: ExtensionContext,
    ) -> Result<Box<dyn ExtensionHandler>, ExtensionError>;
}

/// Not given a concrete method set anywhere in this document — spec
/// §12 references `Box<dyn ExtensionHandler>` as `create_handler`'s
/// return type without ever defining the trait's own methods. Left
/// as a minimal marker trait (object-safe, `Send + Sync` matching
/// [`ProtocolExtension`]'s own bound) rather than inventing an
/// unspecified method set — spec §23 "Extension Lifecycle" and §26
/// "Extension Shutdown" both describe behavior a real
/// `ExtensionHandler` would need to expose (`open`/`close`/lifecycle
/// transitions), so this is exactly the kind of "revisit once a later
/// spec part or an actual extension implementation says more" gap
/// this crate's top-level doc comment names rather than guesses past.
pub trait ExtensionHandler: Send + Sync {}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RegistryError {
    #[error("extension {0} is already registered")]
    AlreadyRegistered(String),
}

/// spec §12's `ExtensionRegistry` (`// protocol id -> implementation`)
/// plus spec §13's builder-style construction
/// (`CommunicationRuntime::builder().register_extension(...).build()`).
/// This crate stops at the registry itself — `CommunicationRuntime`
/// (the thing spec §13 actually shows `.build()`ing) doesn't exist
/// here; building one would mean threading this registry's negotiated
/// extensions into real session/transport machinery, which is exactly
/// the "separate, later integration pass" this module's own top doc
/// comment already names for [`ExtensionContext`]'s placeholder
/// handles.
///
/// "Avoid mandatory global static registries" (§12) — enforced simply
/// by this being an ordinary owned value with no `static`/`OnceLock`
/// anywhere in this crate, built explicitly by whoever constructs one,
/// matching spec §12's own stated reasoning.
pub struct ExtensionRegistry {
    extensions: HashMap<ProtocolId, Arc<dyn ProtocolExtension>>,
}

impl ExtensionRegistry {
    pub fn builder() -> ExtensionRegistryBuilder {
        ExtensionRegistryBuilder {
            extensions: HashMap::new(),
        }
    }

    pub fn descriptors(&self) -> impl Iterator<Item = ExtensionDescriptor> + '_ {
        self.extensions.values().map(|ext| ext.descriptor())
    }

    pub fn get(&self, id: &ProtocolId) -> Option<&Arc<dyn ProtocolExtension>> {
        self.extensions.get(id)
    }
}

/// spec §13's `CommunicationRuntime::builder()` chain, scoped to just
/// what this crate owns (the registry, not a full runtime — see
/// [`ExtensionRegistry`]'s own doc comment).
pub struct ExtensionRegistryBuilder {
    extensions: HashMap<ProtocolId, Arc<dyn ProtocolExtension>>,
}

impl ExtensionRegistryBuilder {
    /// Panics on a duplicate [`ProtocolId`] registration — a
    /// programming error caught at startup, matching
    /// [`crate::capability::CapabilityRegistry::register`]'s own
    /// choice and reasoning. Use [`ExtensionRegistryBuilder::try_register_extension`]
    /// if the set of extensions to register isn't known statically at
    /// compile time (e.g. built from configuration) and a duplicate is
    /// a real runtime condition to handle rather than a bug.
    pub fn register_extension(self, extension: impl ProtocolExtension + 'static) -> Self {
        match self.try_register_extension(extension) {
            Ok(next) => next,
            Err(RegistryError::AlreadyRegistered(id)) => {
                panic!("extension {id} is already registered")
            }
        }
    }

    pub fn try_register_extension(
        mut self,
        extension: impl ProtocolExtension + 'static,
    ) -> Result<Self, RegistryError> {
        let id = extension.descriptor().id;
        if self.extensions.contains_key(&id) {
            return Err(RegistryError::AlreadyRegistered(id.canonical_name()));
        }
        self.extensions.insert(id, Arc::new(extension));
        Ok(self)
    }

    pub fn build(self) -> ExtensionRegistry {
        ExtensionRegistry {
            extensions: self.extensions,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::CapabilitySet;
    use crate::descriptor::{ExtensionLimits, ExtensionRequirement, ExtensionVersion};
    use crate::identifier::{NamespaceId, ProtocolMajor, ProtocolMinor, ProtocolName};

    struct DummyExtension(ProtocolId);

    impl ProtocolExtension for DummyExtension {
        fn descriptor(&self) -> ExtensionDescriptor {
            ExtensionDescriptor {
                id: self.0.clone(),
                version: ExtensionVersion {
                    major: ProtocolMajor(1),
                    minor: ProtocolMinor(0),
                },
                capabilities: CapabilitySet::default(),
                required_capabilities: CapabilitySet::default(),
                requirement: ExtensionRequirement::Optional,
                limits: ExtensionLimits {
                    max_frame_size: 1024,
                    max_in_flight_frames: 1,
                    max_concurrent_streams: 1,
                    max_buffered_bytes: 1024,
                },
                security: crate::security::SecurityRequirements::messaging_default(),
                stability: crate::descriptor::ExtensionStability::Stable,
            }
        }

        fn create_handler(
            &self,
            _negotiated: NegotiatedExtension,
            _ctx: ExtensionContext,
        ) -> Result<Box<dyn ExtensionHandler>, ExtensionError> {
            struct Dummy;
            impl ExtensionHandler for Dummy {}
            Ok(Box::new(Dummy))
        }
    }

    fn protocol_id(name: &str) -> ProtocolId {
        ProtocolId::new(
            NamespaceId::new("org.example.comm").unwrap(),
            ProtocolName::new(name).unwrap(),
            ProtocolMajor(1),
        )
    }

    #[test]
    fn registers_and_looks_up_by_protocol_id() {
        let registry = ExtensionRegistry::builder()
            .register_extension(DummyExtension(protocol_id("messaging")))
            .register_extension(DummyExtension(protocol_id("files")))
            .build();
        assert!(registry.get(&protocol_id("messaging")).is_some());
        assert!(registry.get(&protocol_id("presence")).is_none());
        assert_eq!(registry.descriptors().count(), 2);
    }

    #[test]
    fn duplicate_registration_is_an_error_not_a_silent_overwrite() {
        let result = ExtensionRegistry::builder()
            .register_extension(DummyExtension(protocol_id("messaging")))
            .try_register_extension(DummyExtension(protocol_id("messaging")));
        assert!(matches!(result, Err(RegistryError::AlreadyRegistered(_))));
    }

    #[test]
    #[should_panic(expected = "already registered")]
    fn register_extension_panics_on_duplicate() {
        ExtensionRegistry::builder()
            .register_extension(DummyExtension(protocol_id("messaging")))
            .register_extension(DummyExtension(protocol_id("messaging")));
    }
}
