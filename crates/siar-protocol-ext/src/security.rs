//! spec §32 "Security Requirements Per Extension", §33 "Authorization
//! Hooks".

use crate::capability::CapabilityId;
use crate::identifier::ProtocolId;

/// spec §32, verbatim struct.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SecurityRequirements {
    pub authenticated_peer: bool,
    pub e2ee_required: bool,
    pub authorization_required: bool,
    pub allow_anonymous: bool,
}

impl SecurityRequirements {
    /// spec §32's own worked example: "Messaging likely requires:
    /// authenticated = true, E2EE = true." Kept as a named constructor
    /// (not just a comment) so a real messaging extension's descriptor
    /// has something concrete to reach for instead of writing out all
    /// four fields by hand and risking one silently defaulting wrong.
    /// `authorization_required`/`allow_anonymous` aren't specified by
    /// this example either way, so both default to `false` here — the
    /// conservative reading (nothing is authorized or anonymous unless
    /// asked for), not a guess at unstated intent.
    pub const fn messaging_default() -> Self {
        Self {
            authenticated_peer: true,
            e2ee_required: true,
            authorization_required: false,
            allow_anonymous: false,
        }
    }
}

/// spec §33: "Extensions should not invent product-specific
/// authorization internally." A minimal, deliberately opaque stand-in
/// for a device's real identity — this crate stays standalone (see
/// this crate's own lib.rs doc, "No wire integration"), so it has no
/// dependency on `siar-crypto`/`siar-domain`'s real `DeviceId`; a real
/// integration maps one onto the other rather than this crate
/// depending downward on application-layer identity types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct PeerIdentity(pub [u8; 32]);

/// spec §33's `operation` parameter — what's being authorized, in
/// terms this crate already has real types for: which extension, and
/// (per §31) optionally which specific negotiated capability the
/// operation requires.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct OperationDescriptor {
    pub extension: ProtocolId,
    pub operation_name: String,
    pub required_capability: Option<CapabilityId>,
}

/// spec §33's return type. A closed two-variant decision, not a bare
/// `bool` — `Deny` carries a reason because every one of §33's own
/// three examples (block/contact policy, organization/role policy,
/// authority/priority policy) is a decision an application will want
/// to explain back to a user or log, not just silently act on.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AuthorizationDecision {
    Allow,
    Deny { reason: String },
}

/// spec §33, verbatim trait shape. The three example implementations
/// spec §33 names — "Messenger → block/contact policy," "ERP →
/// organization/role policy," "Emergency → authority/priority policy"
/// — are exactly why this is a trait an application implements rather
/// than logic this crate bakes in: each is real product-specific
/// policy this crate has no business encoding.
pub trait ExtensionAuthorization {
    fn authorize(
        &self,
        peer: PeerIdentity,
        operation: OperationDescriptor,
    ) -> impl std::future::Future<Output = AuthorizationDecision> + Send;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identifier::{NamespaceId, ProtocolMajor, ProtocolName};

    #[test]
    fn spec_32_messaging_default_matches_the_spec_example() {
        let requirements = SecurityRequirements::messaging_default();
        assert!(requirements.authenticated_peer);
        assert!(requirements.e2ee_required);
    }

    /// A block-list authorizer, standing in for spec §33's "Messenger
    /// → block/contact policy" example — real enough to prove the
    /// trait is actually usable, not just a type that compiles.
    struct BlockListAuthorizer {
        blocked: Vec<PeerIdentity>,
    }

    impl ExtensionAuthorization for BlockListAuthorizer {
        async fn authorize(
            &self,
            peer: PeerIdentity,
            _operation: OperationDescriptor,
        ) -> AuthorizationDecision {
            if self.blocked.contains(&peer) {
                AuthorizationDecision::Deny {
                    reason: "peer is on the block list".to_string(),
                }
            } else {
                AuthorizationDecision::Allow
            }
        }
    }

    #[tokio::test]
    async fn spec_33_blocked_peer_is_denied_with_a_reason() {
        let blocked_peer = PeerIdentity([9u8; 32]);
        let authorizer = BlockListAuthorizer {
            blocked: vec![blocked_peer],
        };
        let operation = OperationDescriptor {
            extension: ProtocolId::new(
                NamespaceId::new("org.example").unwrap(),
                ProtocolName::new("messaging").unwrap(),
                ProtocolMajor(1),
            ),
            operation_name: "SendMessage".to_string(),
            required_capability: None,
        };

        let decision = authorizer.authorize(blocked_peer, operation).await;
        assert!(matches!(decision, AuthorizationDecision::Deny { .. }));
    }

    #[tokio::test]
    async fn spec_33_unblocked_peer_is_allowed() {
        let authorizer = BlockListAuthorizer { blocked: vec![] };
        let operation = OperationDescriptor {
            extension: ProtocolId::new(
                NamespaceId::new("org.example").unwrap(),
                ProtocolName::new("messaging").unwrap(),
                ProtocolMajor(1),
            ),
            operation_name: "SendMessage".to_string(),
            required_capability: None,
        };

        let decision = authorizer.authorize(PeerIdentity([1u8; 32]), operation).await;
        assert_eq!(decision, AuthorizationDecision::Allow);
    }
}
