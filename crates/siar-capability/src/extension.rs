//! §33 "Extension Capability Negotiation", §34 "Extension Negotiator
//! Trait".

use crate::error::CapabilityNegotiationError;
use crate::policy::CapabilityPolicy;
use crate::registry::CapabilityRegistry;
use crate::set::CapabilitySet;

/// §34's sketch types `advertise() -> ExtensionCapabilitySet` and
/// `negotiate(...) -> Result<NegotiatedExtensionCapabilities, ...>`.
/// Both of those spec types are, per every example in §35/§37 (a flat
/// list of named capabilities under one namespace), exactly what
/// [`CapabilitySet`] already is — a bounded, canonically-ordered
/// collection of [`crate::descriptor::CapabilityDescriptor`]s. Neither
/// `ExtensionCapabilitySet` nor `NegotiatedExtensionCapabilities` is
/// given any field or behavior anywhere in the spec beyond "a set of
/// this extension's capabilities" and "the negotiated subset of
/// those" respectively, so introducing two more newtypes here would
/// only be indirection over the type this crate already has, not a
/// distinct concept — this trait uses [`CapabilitySet`] directly for
/// both.
///
/// §33: "Each protocol extension from Part 01 owns its capability
/// schema... delegated to extension-specific logic" — so
/// [`registry`](ExtensionNegotiator::registry) is per-implementor,
/// not shared, matching that ownership.
pub trait ExtensionNegotiator {
    /// This extension's own advertised capabilities (§34's `advertise`).
    fn advertise(&self) -> CapabilitySet;

    /// This extension's [`CapabilityRegistry`] — its schema, security
    /// classes, and internal dependency edges (§65-69), scoped to just
    /// this extension's own namespace, per §33.
    fn registry(&self) -> &CapabilityRegistry;

    /// §34's `negotiate`: intersect this extension's own advertisement
    /// against a remote peer's advertised set for the same extension,
    /// under the given policy. A provided method built directly on
    /// [`crate::negotiate::negotiate`], so an implementor only has to
    /// supply [`advertise`](ExtensionNegotiator::advertise) and
    /// [`registry`](ExtensionNegotiator::registry) — not re-implement
    /// §19's intersection rules or §24's pipeline itself.
    fn negotiate(
        &self,
        remote: &CapabilitySet,
        policy: &CapabilityPolicy,
    ) -> Result<CapabilitySet, CapabilityNegotiationError> {
        crate::negotiate::negotiate(&self.advertise(), remote, self.registry(), policy)
    }
}
