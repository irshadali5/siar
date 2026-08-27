//! Negotiation flow — spec §10 "Negotiation Flow", §11 "Mandatory and
//! Optional Extensions".
//!
//! This module is the local computation both sides of spec §10's
//! `HELLO`/`HELLO_ACK` diagram perform — not the wire encoding of
//! `HELLO`/`HELLO_ACK` itself (that's a `siar-protocol` framing
//! concern this crate deliberately doesn't reach into; see this
//! crate's top-level doc comment). Given what this side advertised and
//! what the remote side advertised, [`negotiate`] computes exactly
//! what spec §10's `HELLO_ACK`/`negotiated:` block shows.

use crate::capability::CapabilitySet;
use crate::descriptor::{
    ExtensionDescriptor, ExtensionRequirement, NegotiatedExtension, SessionLocalExtensionId,
};
use crate::identifier::ProtocolId;
use std::collections::HashMap;

/// What a remote peer advertised for one [`ProtocolId`] in its own
/// `HELLO` — just the capability set, since major-version compatibility
/// is decided by which `ProtocolId` (namespace/protocol/major, spec
/// §7) the remote even sent a capability set for at all: no shared
/// major version means no entry for that protocol, not a mismatched
/// entry to reconcile.
#[derive(Debug, Clone)]
pub struct RemoteAdvertisement {
    pub id: ProtocolId,
    pub capabilities: CapabilitySet,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NegotiationError {
    /// spec §11: a `Required` extension with no matching advertisement
    /// from the remote side (no shared major version, or the remote
    /// simply doesn't implement it) — the one case where negotiation
    /// genuinely fails the whole session, per spec §11's own
    /// "files = required" example ("a file-only application" has
    /// nothing to fall back to without files).
    #[error("required extension {0} was not advertised by the remote peer")]
    RequiredExtensionUnavailable(String),
}

/// Negotiates a full session: every `local` [`ExtensionDescriptor`]
/// against whatever the `remote` side advertised for the same
/// [`ProtocolId`]. Returns one [`NegotiatedExtension`] per successfully
/// negotiated extension — spec §11's rule applied exactly: "An
/// unsupported optional extension must not tear down the whole
/// session," so a missing `Optional` extension is silently absent from
/// the result, not an error; a missing `Required` one fails the whole
/// call via [`NegotiationError::RequiredExtensionUnavailable`], since
/// at that point there is no valid session to return at all (matching
/// spec §11's "files = required" example: there is no meaningful
/// partial session for a file-only application with no file
/// extension).
///
/// `session_ids` are assigned by iteration order over `local` — see
/// [`SessionLocalExtensionId`]'s own doc comment for why that's fine
/// (session-local, not persisted, not compared across sessions).
pub fn negotiate(
    local: &[ExtensionDescriptor],
    remote: &[RemoteAdvertisement],
) -> Result<Vec<NegotiatedExtension>, NegotiationError> {
    let remote_by_id: HashMap<&ProtocolId, &CapabilitySet> = remote
        .iter()
        .map(|advertisement| (&advertisement.id, &advertisement.capabilities))
        .collect();

    let mut negotiated = Vec::new();
    let mut next_session_id: u16 = 1;

    for descriptor in local {
        match remote_by_id.get(&descriptor.id) {
            Some(remote_capabilities) => {
                let capabilities = descriptor.capabilities.intersect(remote_capabilities);
                let session_id = SessionLocalExtensionId(next_session_id);
                next_session_id += 1;
                negotiated.push(NegotiatedExtension {
                    id: descriptor.id.clone(),
                    session_id,
                    capabilities,
                });
            }
            None => {
                if matches!(descriptor.requirement, ExtensionRequirement::Required) {
                    return Err(NegotiationError::RequiredExtensionUnavailable(
                        descriptor.id.canonical_name(),
                    ));
                }
                // Optional and unavailable — spec §11: silently
                // absent from the result, session continues.
            }
        }
    }

    Ok(negotiated)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::CapabilityId;
    use crate::descriptor::{ExtensionLimits, ExtensionVersion};
    use crate::identifier::{NamespaceId, ProtocolMajor, ProtocolMinor, ProtocolName};

    fn descriptor(
        protocol: &str,
        requirement: ExtensionRequirement,
        caps: &[u32],
    ) -> ExtensionDescriptor {
        ExtensionDescriptor {
            id: ProtocolId::new(
                NamespaceId::new("org.example.comm").unwrap(),
                ProtocolName::new(protocol).unwrap(),
                ProtocolMajor(1),
            ),
            version: ExtensionVersion {
                major: ProtocolMajor(1),
                minor: ProtocolMinor(0),
            },
            capabilities: CapabilitySet::new(caps.iter().map(|n| CapabilityId(*n))),
            requirement,
            limits: ExtensionLimits {
                max_frame_size: 65536,
                max_in_flight_frames: 32,
                max_concurrent_streams: 4,
                max_buffered_bytes: 1 << 20,
            },
        }
    }

    fn advertisement(protocol: &str, caps: &[u32]) -> RemoteAdvertisement {
        RemoteAdvertisement {
            id: ProtocolId::new(
                NamespaceId::new("org.example.comm").unwrap(),
                ProtocolName::new(protocol).unwrap(),
                ProtocolMajor(1),
            ),
            capabilities: CapabilitySet::new(caps.iter().map(|n| CapabilityId(*n))),
        }
    }

    #[test]
    fn matches_spec_10_example() {
        // spec §10's own worked example: messaging/1 [text, reply,
        // edit] locally, [text, reply] remotely -> negotiated
        // [text, reply].
        let local = vec![descriptor(
            "messaging",
            ExtensionRequirement::Optional,
            &[1, 2, 3],
        )];
        let remote = vec![advertisement("messaging", &[1, 2])];
        let result = negotiate(&local, &remote).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].capabilities,
            CapabilitySet::new([CapabilityId(1), CapabilityId(2)])
        );
        assert_eq!(result[0].session_id.0, 1);
    }

    #[test]
    fn missing_optional_extension_does_not_fail_session() {
        let local = vec![
            descriptor("messaging", ExtensionRequirement::Required, &[1]),
            descriptor("presence", ExtensionRequirement::Optional, &[1]),
        ];
        let remote = vec![advertisement("messaging", &[1])]; // no presence
        let result = negotiate(&local, &remote).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id.protocol.as_str(), "messaging");
    }

    #[test]
    fn missing_required_extension_fails_the_session() {
        let local = vec![descriptor("files", ExtensionRequirement::Required, &[1])];
        let remote: Vec<RemoteAdvertisement> = vec![]; // remote doesn't speak files at all
        let err = negotiate(&local, &remote).unwrap_err();
        assert_eq!(
            err,
            NegotiationError::RequiredExtensionUnavailable("org.example.comm/files/1".to_string())
        );
    }
}
