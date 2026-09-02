//! spec §44 "Peer Capability Query", §45 "Capability Cache", §46
//! "Capability Changes".

use crate::capability::CapabilitySet;
use crate::identifier::ProtocolId;
use crate::security::PeerIdentity;
use std::collections::HashMap;

/// spec §44's `caps.supports(MESSAGING_V1)` pattern, given a real
/// backing type: what [`crate::negotiation::negotiate`] actually
/// produced for one peer, keyed by [`ProtocolId`] so `supports` can
/// answer "did we negotiate this exact protocol+major version" without
/// the caller re-deriving that from a `Vec<NegotiatedExtension>` at
/// every call site.
#[derive(Debug, Clone, Default)]
pub struct PeerCapabilities {
    negotiated: HashMap<ProtocolId, CapabilitySet>,
}

impl PeerCapabilities {
    pub fn from_negotiated(extensions: &[crate::descriptor::NegotiatedExtension]) -> Self {
        Self {
            negotiated: extensions
                .iter()
                .map(|e| (e.id.clone(), e.capabilities.clone()))
                .collect(),
        }
    }

    /// spec §44: "if caps.supports(MESSAGING_V1) { enable messaging
    /// UX }" — true iff this exact [`ProtocolId`] (protocol name *and*
    /// major version) was actually negotiated with this peer, not
    /// merely locally supported.
    pub fn supports(&self, protocol: &ProtocolId) -> bool {
        self.negotiated.contains_key(protocol)
    }

    pub fn capabilities_for(&self, protocol: &ProtocolId) -> Option<&CapabilitySet> {
        self.negotiated.get(protocol)
    }
}

/// spec §45's own field list, verbatim: "peer, protocol versions,
/// capabilities, last observed, expiry/source."
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CapabilityCacheEntry {
    pub peer: PeerIdentity,
    pub protocol_versions: Vec<ProtocolId>,
    pub capabilities: CapabilitySet,
    pub last_observed_millis: u64,
    pub expiry_millis: Option<u64>,
    pub source: CacheSource,
}

/// The "source" half of §45's "expiry/source" field — kept as its own
/// small enum rather than a free-text string so "cached capabilities
/// are hints, not security truth" (§45's own closing line) has
/// somewhere real to be enforced from: [`PeerCapabilityCache::get`]
/// only ever returns a hint annotated with where it came from, never
/// something a caller could mistake for a fresh, authenticated result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CacheSource {
    NegotiatedThisSession,
    PersistedHint,
}

/// spec §45: "persist a hint cache... but authenticate and renegotiate
/// on a fresh session. Cached capabilities are hints, not security
/// truth." The type itself can't stop a caller from misusing a hint —
/// no type system can enforce "renegotiate on a fresh session" — but
/// [`PeerCapabilityCache::get`] returning `None` once an entry is past
/// its own `expiry_millis` is the one part of that rule this crate
/// *can* enforce structurally, so it does.
#[derive(Debug, Clone, Default)]
pub struct PeerCapabilityCache {
    entries: HashMap<PeerIdentity, CapabilityCacheEntry>,
}

impl PeerCapabilityCache {
    pub fn insert(&mut self, entry: CapabilityCacheEntry) {
        self.entries.insert(entry.peer, entry);
    }

    /// Returns `None` for an unknown peer, or for one whose cached
    /// entry's `expiry_millis` has already passed as of `now_millis` —
    /// an expired hint is treated exactly like no hint at all, never
    /// silently returned as though it were still current.
    pub fn get(&self, peer: PeerIdentity, now_millis: u64) -> Option<&CapabilityCacheEntry> {
        self.entries
            .get(&peer)
            .filter(|entry| entry.expiry_millis.is_none_or(|expiry| now_millis < expiry))
    }
}

/// spec §46's own six causes, verbatim, as a closed set — "therefore
/// they are not immutable identity data" is the actual point of §46,
/// and a closed enum of *why* a capability changed is what makes that
/// point checkable (e.g. in a UI that wants to tell a user why a
/// feature just disappeared) rather than only assertable in prose.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CapabilityChangeReason {
    ApplicationUpgrade,
    FeatureDisablement,
    Permissions,
    HardwareAvailability,
    PlatformChanges,
    PolicyChanges,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::CapabilityId;
    use crate::descriptor::{NegotiatedExtension, SessionLocalExtensionId};
    use crate::identifier::{NamespaceId, ProtocolMajor, ProtocolName};

    fn messaging_v1() -> ProtocolId {
        ProtocolId::new(
            NamespaceId::new("org.example").unwrap(),
            ProtocolName::new("messaging").unwrap(),
            ProtocolMajor(1),
        )
    }

    #[test]
    fn spec_44_supports_is_true_only_for_actually_negotiated_protocols() {
        let negotiated = vec![NegotiatedExtension {
            id: messaging_v1(),
            session_id: SessionLocalExtensionId(1),
            capabilities: CapabilitySet::new([CapabilityId(1)]),
        }];
        let caps = PeerCapabilities::from_negotiated(&negotiated);

        assert!(caps.supports(&messaging_v1()));

        let files_v1 = ProtocolId::new(
            NamespaceId::new("org.example").unwrap(),
            ProtocolName::new("files").unwrap(),
            ProtocolMajor(1),
        );
        assert!(!caps.supports(&files_v1));
    }

    #[test]
    fn spec_45_expired_cache_entry_is_not_returned() {
        let peer = PeerIdentity([1u8; 32]);
        let mut cache = PeerCapabilityCache::default();
        cache.insert(CapabilityCacheEntry {
            peer,
            protocol_versions: vec![messaging_v1()],
            capabilities: CapabilitySet::new([CapabilityId(1)]),
            last_observed_millis: 1_000,
            expiry_millis: Some(2_000),
            source: CacheSource::PersistedHint,
        });

        assert!(cache.get(peer, 1_500).is_some(), "not expired yet");
        assert!(cache.get(peer, 2_500).is_none(), "past its own expiry");
    }

    #[test]
    fn spec_45_entry_with_no_expiry_never_expires() {
        let peer = PeerIdentity([2u8; 32]);
        let mut cache = PeerCapabilityCache::default();
        cache.insert(CapabilityCacheEntry {
            peer,
            protocol_versions: vec![],
            capabilities: CapabilitySet::default(),
            last_observed_millis: 1_000,
            expiry_millis: None,
            source: CacheSource::NegotiatedThisSession,
        });

        assert!(cache.get(peer, u64::MAX).is_some());
    }

    #[test]
    fn spec_45_unknown_peer_returns_none() {
        let cache = PeerCapabilityCache::default();
        assert!(cache.get(PeerIdentity([9u8; 32]), 0).is_none());
    }
}
