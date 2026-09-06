//! §178 "Notification Integration", §179 "Device Endpoint Privacy".

use std::collections::HashMap;

use siar_domain::DeviceId;
use siar_event_log::ids::Timestamp;

use crate::directory::DeviceEndpoint;

/// §178: "push token is not identity." An opaque wrapper with no
/// method anywhere that compares it to, converts it into, or accepts
/// it in place of a [`DeviceId`]/certificate/signature — this crate
/// has no function that would let a `PushToken` authenticate or
/// authorize anything, which is the actual content of "is not
/// identity," not a comment appended to a type that could otherwise
/// be used that way.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PushToken(pub Vec<u8>);

/// §178's own mapping, `DeviceId → push endpoint`, "as
/// application/platform metadata" — deliberately not stored anywhere
/// near [`crate::directory::DeviceDirectory`] or any signed type; this
/// is a plain, unsigned, locally-held table, matching spec's own
/// "application/platform metadata" framing exactly.
#[derive(Debug, Clone, Default)]
pub struct PushEndpointRegistry {
    endpoints: HashMap<DeviceId, PushToken>,
}

impl PushEndpointRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, device_id: DeviceId, token: PushToken) {
        self.endpoints.insert(device_id, token);
    }

    pub fn endpoint_for(&self, device_id: DeviceId) -> Option<&PushToken> {
        self.endpoints.get(&device_id)
    }
}

/// §179's own three named properties, verbatim.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndpointScope {
    /// §179's default posture: nothing is visible in a public profile
    /// unless a caller explicitly says otherwise — matching this
    /// crate's established pattern
    /// ([`crate::device_privacy_presentation::remote_device_label`]'s
    /// own explicit-opt-in requirement for friendly names) of making
    /// the private choice the one that needs no special action.
    Ephemeral,
    AuthenticatedContactsOnly,
    PublicProfile,
}

/// §179: "transport/push endpoints should be rotatable, expiring,
/// scoped where possible. Do not expose them permanently in public
/// profile data." `expires_at` makes "expiring" a real, checkable
/// field rather than a policy statement; "rotatable" needs no field
/// at all — a caller rotates by constructing a new `ScopedEndpoint`
/// with a fresh [`DeviceEndpoint`], the same "no incremental mutator,
/// a new value each time" shape
/// [`crate::directory_cache::DirectoryCache`] and
/// [`crate::discovery_privacy::RotatingDiscoveryToken`] already use
/// for the identical concern.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopedEndpoint {
    pub endpoint: DeviceEndpoint,
    pub scope: EndpointScope,
    pub expires_at: Option<Timestamp>,
}

impl ScopedEndpoint {
    /// "Do not expose them permanently in public profile data": `true`
    /// only for `PublicProfile` scope AND not expired — an expired
    /// `PublicProfile` endpoint is not exposed either, since
    /// "permanently" is exactly what an expiry check prevents.
    pub fn is_visible_in_public_profile(&self, now: Timestamp) -> bool {
        let not_expired = match self.expires_at {
            Some(expires_at) => now.0 < expires_at.0,
            None => true,
        };
        matches!(self.scope, EndpointScope::PublicProfile) && not_expired
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_registry_maps_device_to_token_and_nothing_else() {
        let mut registry = PushEndpointRegistry::new();
        let device = DeviceId::new();
        registry.register(device, PushToken(vec![1, 2, 3]));
        assert_eq!(
            registry.endpoint_for(device),
            Some(&PushToken(vec![1, 2, 3]))
        );
        assert_eq!(registry.endpoint_for(DeviceId::new()), None);
    }

    #[test]
    fn only_public_profile_scope_and_unexpired_is_visible_publicly() {
        let endpoint = ScopedEndpoint {
            endpoint: DeviceEndpoint(vec![1]),
            scope: EndpointScope::PublicProfile,
            expires_at: Some(Timestamp(100)),
        };
        assert!(endpoint.is_visible_in_public_profile(Timestamp(50)));
        assert!(!endpoint.is_visible_in_public_profile(Timestamp(150)));
    }

    #[test]
    fn ephemeral_and_contacts_only_scopes_are_never_publicly_visible() {
        let ephemeral = ScopedEndpoint {
            endpoint: DeviceEndpoint(vec![1]),
            scope: EndpointScope::Ephemeral,
            expires_at: None,
        };
        let contacts_only = ScopedEndpoint {
            endpoint: DeviceEndpoint(vec![1]),
            scope: EndpointScope::AuthenticatedContactsOnly,
            expires_at: None,
        };
        assert!(!ephemeral.is_visible_in_public_profile(Timestamp(0)));
        assert!(!contacts_only.is_visible_in_public_profile(Timestamp(0)));
    }
}
