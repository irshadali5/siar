//! Capability negotiation — spec §8 "Capability Negotiation", §9
//! "Capability Representation".
//!
//! "The effective feature set is: `local capabilities ∩ remote
//! capabilities`" (§8) — [`CapabilitySet::intersect`] is exactly that.
//! Deliberately `Vec<CapabilityId>`, matching the struct spec §9 gives
//! verbatim, not `HashSet` — §9's note that `SmallVec` "may reduce
//! allocation" for very small sets is left as a future swap (see this
//! module's own `CapabilitySet` doc comment), not pulled in as a new
//! workspace dependency for a "may".

use std::collections::HashMap;

/// spec §9's `CapabilityId(pub u32)`, exactly as given there. A stable
/// compact identifier — "Avoid hot-path capability checks based on
/// arbitrary strings" (§9) — with human-readable names kept out of
/// this type entirely and looked up via [`CapabilityRegistry`] instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub struct CapabilityId(pub u32);

/// spec §9's `CapabilitySet` struct, exactly as given there
/// (`pub values: Vec<CapabilityId>`). Small sets in practice (a
/// handful of capabilities per extension per the spec's own messaging/
/// file examples in §8) — if profiling ever shows this allocation
/// matters, `values` swapping to a `SmallVec` is a private
/// implementation change this type's public API doesn't need to
/// change for.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CapabilitySet {
    pub values: Vec<CapabilityId>,
}

impl CapabilitySet {
    pub fn new(values: impl IntoIterator<Item = CapabilityId>) -> Self {
        let mut values: Vec<CapabilityId> = values.into_iter().collect();
        values.sort_unstable();
        values.dedup();
        Self { values }
    }

    pub fn contains(&self, id: CapabilityId) -> bool {
        self.values.binary_search(&id).is_ok()
    }

    /// `local capabilities ∩ remote capabilities` — spec §8, verbatim.
    /// This is the negotiation itself: whatever survives the
    /// intersection is the effective feature set for the session, not
    /// a preference or a suggestion.
    pub fn intersect(&self, other: &CapabilitySet) -> CapabilitySet {
        let values: Vec<CapabilityId> = self.values.iter().filter(|id| other.contains(**id)).copied().collect();
        CapabilitySet { values }
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }
}

/// "Maintain a registry mapping IDs to human-readable names for
/// diagnostics" — spec §9, verbatim purpose. Deliberately separate
/// from [`CapabilityId`] itself (which stays a bare `u32` newtype on
/// the hot path) — this is a diagnostics/documentation-time lookup,
/// not something a per-message capability check ever touches.
#[derive(Debug, Default)]
pub struct CapabilityRegistry {
    names: HashMap<CapabilityId, &'static str>,
}

impl CapabilityRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Panics on a duplicate id — a capability registry with two names
    /// for one id is a programming error at registration time (a
    /// build-time/startup-time mistake), not a runtime condition
    /// callers need to recover from, matching how this crate treats
    /// every other "this should have been caught during development"
    /// case (see [`crate::registry::ExtensionRegistryBuilder::register_extension`]'s
    /// own doc comment for the same reasoning applied there).
    pub fn register(&mut self, id: CapabilityId, name: &'static str) {
        if let Some(existing) = self.names.insert(id, name) {
            panic!("CapabilityId({}) already registered as {existing:?}, cannot also register as {name:?}", id.0);
        }
    }

    pub fn name(&self, id: CapabilityId) -> Option<&'static str> {
        self.names.get(&id).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intersect_keeps_only_shared_capabilities() {
        let text = CapabilityId(1);
        let reply = CapabilityId(2);
        let edit = CapabilityId(3);
        let local = CapabilitySet::new([text, reply, edit]);
        let remote = CapabilitySet::new([text, reply]);
        let negotiated = local.intersect(&remote);
        assert_eq!(negotiated, CapabilitySet::new([text, reply]));
    }

    #[test]
    fn intersect_is_order_independent() {
        let a = CapabilitySet::new([CapabilityId(3), CapabilityId(1)]);
        let b = CapabilitySet::new([CapabilityId(1), CapabilityId(2)]);
        assert_eq!(a.intersect(&b), CapabilitySet::new([CapabilityId(1)]));
        assert_eq!(a.intersect(&b), b.intersect(&a));
    }

    #[test]
    fn registry_rejects_duplicate_id() {
        let mut reg = CapabilityRegistry::new();
        reg.register(CapabilityId(1), "text");
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            reg.register(CapabilityId(1), "reply");
        }));
        assert!(result.is_err());
    }
}
