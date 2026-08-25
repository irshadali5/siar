//! §26 "Negotiation Transcript Hash", §27 "Canonical Encoding", §94
//! "Capability Hash".

use crate::set::CapabilitySet;
use std::fmt;

/// §94: `CapabilitySetHash([u8; 32])` — used for cache validation,
/// delta detection (§95), and transcript binding (§26). This crate
/// implements only the pure hash-of-a-set primitive; §26's fuller
/// transcript hash (local advertisement + remote advertisement +
/// selected capabilities + session nonce) is a negotiation-time
/// concern for a later pass, not built here, since it needs the
/// advertisement/session types this crate's Phase 1 deliberately
/// doesn't include yet (§161 Phase 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CapabilitySetHash([u8; 32]);

impl CapabilitySetHash {
    /// §27: "Capability ordering must be canonical before hashing...
    /// Do not hash arbitrary HashMap iteration order." [`CapabilitySet`]
    /// already maintains canonical `(namespace, code, version)` order
    /// on every insert (see `set.rs`), so postcard-encoding its
    /// iteration order directly satisfies §27 without a separate sort
    /// step here that could fall out of sync with the set's own
    /// invariant.
    pub fn of(set: &CapabilitySet) -> Self {
        let descriptors: Vec<_> = set.iter().collect();
        let bytes = postcard::to_allocvec(&descriptors)
            .expect("CapabilitySet descriptors always postcard-serialize");
        Self(*blake3::hash(&bytes).as_bytes())
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl fmt::Display for CapabilitySetHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::descriptor::{CapabilityDescriptor, CapabilityParameters, CapabilityRequirement};
    use crate::id::{CapabilityId, CapabilityNamespace};
    use crate::version::CapabilityVersion;

    fn desc(namespace: CapabilityNamespace, code: u32) -> CapabilityDescriptor {
        CapabilityDescriptor::new(
            CapabilityId::new(namespace, code),
            CapabilityVersion::new(1, 0),
            CapabilityRequirement::Optional,
            CapabilityParameters::None,
        )
    }

    #[test]
    fn hash_is_independent_of_insertion_order() {
        // The whole point of §27: two peers building the *same*
        // logical set in different insertion orders (e.g. because
        // their extension registries iterate differently) must derive
        // the *same* transcript hash, or every "both peers verify
        // equality" (§26) claim in the spec is unimplementable.
        let mut a = CapabilitySet::new();
        a.insert(desc(CapabilityNamespace::Files, 5)).unwrap();
        a.insert(desc(CapabilityNamespace::Core, 1)).unwrap();
        a.insert(desc(CapabilityNamespace::Dtn, 3)).unwrap();

        let mut b = CapabilitySet::new();
        b.insert(desc(CapabilityNamespace::Dtn, 3)).unwrap();
        b.insert(desc(CapabilityNamespace::Core, 1)).unwrap();
        b.insert(desc(CapabilityNamespace::Files, 5)).unwrap();

        assert_eq!(CapabilitySetHash::of(&a), CapabilitySetHash::of(&b));
    }

    #[test]
    fn hash_changes_when_set_content_changes() {
        let mut a = CapabilitySet::new();
        a.insert(desc(CapabilityNamespace::Core, 1)).unwrap();

        let mut b = CapabilitySet::new();
        b.insert(desc(CapabilityNamespace::Core, 2)).unwrap();

        assert_ne!(CapabilitySetHash::of(&a), CapabilitySetHash::of(&b));
    }
}
