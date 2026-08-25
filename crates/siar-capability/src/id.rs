//! §5 "Capability Identifier" and §6 "Capability Namespace".

use serde::{Deserialize, Serialize};
use std::fmt;

/// §6: the fixed set of built-in domains from §4 ("Capability
/// Categories"), plus `Application` for third-party/custom namespaces
/// (§64, §118-119, §154). §6's sketch lists the built-in variants as
/// bare unit variants (`Core`, `Security`, ...); §4's own category list
/// additionally names `Groups`, `Presence`, `Device Resources` (here
/// `Resource`, matching §6's own naming) and `Application Extensions`
/// — all of which §6 already accounts for, so no built-in category is
/// missing relative to §4.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum CapabilityNamespace {
    Core,
    Security,
    Transport,
    Messaging,
    Files,
    Groups,
    Presence,
    Dtn,
    Media,
    Platform,
    Resource,
    Application(NamespaceId),
}

impl fmt::Display for CapabilityNamespace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Core => write!(f, "core"),
            Self::Security => write!(f, "security"),
            Self::Transport => write!(f, "transport"),
            Self::Messaging => write!(f, "messaging"),
            Self::Files => write!(f, "files"),
            Self::Groups => write!(f, "groups"),
            Self::Presence => write!(f, "presence"),
            Self::Dtn => write!(f, "dtn"),
            Self::Media => write!(f, "media"),
            Self::Platform => write!(f, "platform"),
            Self::Resource => write!(f, "resource"),
            Self::Application(id) => write!(f, "app:{id}"),
        }
    }
}

/// A registered third-party namespace identifier (§6, §64, §119).
///
/// The spec never fixes a representation for `NamespaceId` — it only
/// requires that custom extensions "define their own capability IDs"
/// (§154) without colliding with each other or the built-in namespaces
/// (§119: "Plugins must not overwrite existing IDs"). A bounded `u32`
/// mirrors §5's own choice for `CapabilityId::code` (compact numeric
/// IDs, §62) rather than an unbounded `String`, which §61's size-bound
/// requirement would otherwise be fighting against. Collision
/// prevention itself is a registry-time concern (§65, §119), not part
/// of this type — deferred to [`crate::registry::CapabilityRegistry`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct NamespaceId(pub u32);

impl fmt::Display for NamespaceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// §5: a stable typed capability identifier, unique within its
/// namespace. `code` is the compact numeric id §5 says the wire form
/// can use "after namespace agreement" — the human-readable canonical
/// names (`files.resume`, `dtn.spray_wait`, ...) are a documentation
/// and registry-lookup concern (§120-121), not encoded in this type
/// itself, so this crate doesn't guess at a canonical-name catalog
/// before any extension actually needs one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CapabilityId {
    pub namespace: CapabilityNamespace,
    pub code: u32,
}

impl CapabilityId {
    pub const fn new(namespace: CapabilityNamespace, code: u32) -> Self {
        Self { namespace, code }
    }
}

impl fmt::Display for CapabilityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.namespace, self.code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordering_is_total_and_deterministic() {
        // §27: "sort by namespace + code + version" needs Ord to be a
        // real total order, including across the derive-order of enum
        // variants — this is what canonical.rs's sort will lean on.
        let a = CapabilityId::new(CapabilityNamespace::Core, 1);
        let b = CapabilityId::new(CapabilityNamespace::Core, 2);
        let c = CapabilityId::new(CapabilityNamespace::Files, 0);
        assert!(a < b);
        assert!(b < c);

        let app1 = CapabilityId::new(CapabilityNamespace::Application(NamespaceId(1)), 0);
        let app2 = CapabilityId::new(CapabilityNamespace::Application(NamespaceId(2)), 0);
        assert!(app1 < app2);
    }
}
