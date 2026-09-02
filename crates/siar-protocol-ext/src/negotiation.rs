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

    /// spec §30: "unknown required capability → reject extension
    /// operation/negotiation." A `Required` extension whose
    /// [`ExtensionDescriptor::required_capabilities`] don't all survive
    /// intersection with what the remote advertised — the remote is
    /// missing a capability this extension cannot function without, so
    /// (matching `RequiredExtensionUnavailable`'s own reasoning) there
    /// is no valid session to return.
    #[error("required capability {missing:?} of required extension {extension} was not shared by the remote peer")]
    RequiredCapabilityUnavailable {
        extension: String,
        missing: crate::capability::CapabilityId,
    },
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
/// spec §30 applied at the same granularity, one level down: if any of
/// a negotiated extension's [`ExtensionDescriptor::required_capabilities`]
/// didn't survive [`CapabilitySet::intersect`] (the remote doesn't
/// share it), that's the exact "unknown required capability" case —
/// for a `Required` extension this fails the whole session
/// ([`NegotiationError::RequiredCapabilityUnavailable`], same reasoning
/// as a missing `Required` extension entirely); for an `Optional`
/// extension it's treated like the extension itself being unavailable
/// — silently absent from the result, per spec §11's "don't tear down
/// the whole session" rule extended to capability granularity.
/// "Unknown optional capability → ignore" (spec §30's other half) needs
/// no extra code at all: [`CapabilitySet::intersect`] already drops
/// anything not shared, for capabilities that were never in
/// `required_capabilities` to begin with.
///
/// `session_ids` are assigned by iteration order over `local` — spec
/// §17 "Session-Local Extension IDs" ("the numeric mapping is
/// session-local... this reduces repeated framing overhead while
/// retaining stable global protocol identities") — see
/// [`SessionLocalExtensionId`]'s own doc comment for why sequential
/// assignment is fine (session-local, not persisted, not compared
/// across sessions; the spec's own example numbers, 7/9/12, aren't a
/// numbering *algorithm* to reproduce, just an illustration that the
/// mapping is compact and session-scoped).
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

                let missing_required = descriptor
                    .required_capabilities
                    .values
                    .iter()
                    .find(|id| !capabilities.contains(**id));

                if let Some(&missing) = missing_required {
                    if matches!(descriptor.requirement, ExtensionRequirement::Required) {
                        return Err(NegotiationError::RequiredCapabilityUnavailable {
                            extension: descriptor.id.canonical_name(),
                            missing,
                        });
                    }
                    // Optional extension, missing one of its own
                    // required capabilities — spec §11's principle
                    // applied one level down: this extension can't
                    // function, but it's optional, so it's just
                    // absent from the result, not a session failure.
                    continue;
                }

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
        descriptor_with_required(protocol, requirement, caps, &[])
    }

    fn descriptor_with_required(
        protocol: &str,
        requirement: ExtensionRequirement,
        caps: &[u32],
        required: &[u32],
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
            required_capabilities: CapabilitySet::new(required.iter().map(|n| CapabilityId(*n))),
            requirement,
            limits: ExtensionLimits {
                max_frame_size: 65536,
                max_in_flight_frames: 32,
                max_concurrent_streams: 4,
                max_buffered_bytes: 1 << 20,
            },
            security: crate::security::SecurityRequirements::messaging_default(),
            stability: crate::descriptor::ExtensionStability::Stable,
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

    #[test]
    fn spec_30_required_capability_missing_from_a_required_extension_fails_the_session() {
        // messaging is Required overall, and messaging.edit (id 3) is
        // required within it — the remote only advertises [1, 2], so
        // this must be NegotiationError, not a silently-degraded
        // negotiated extension missing capability 3.
        let local = vec![descriptor_with_required(
            "messaging",
            ExtensionRequirement::Required,
            &[1, 2, 3],
            &[3],
        )];
        let remote = vec![advertisement("messaging", &[1, 2])];
        let err = negotiate(&local, &remote).unwrap_err();
        assert_eq!(
            err,
            NegotiationError::RequiredCapabilityUnavailable {
                extension: "org.example.comm/messaging/1".to_string(),
                missing: CapabilityId(3),
            }
        );
    }

    #[test]
    fn spec_30_required_capability_missing_from_an_optional_extension_just_drops_it() {
        // Same shape, but the extension itself is Optional — spec §11's
        // "don't tear down the whole session" rule applied one level
        // down: the session still negotiates fine, this one extension
        // is just absent.
        let local = vec![
            descriptor("messaging", ExtensionRequirement::Required, &[1]),
            descriptor_with_required("files", ExtensionRequirement::Optional, &[1, 2], &[2]),
        ];
        let remote = vec![
            advertisement("messaging", &[1]),
            advertisement("files", &[1]), // no capability 2
        ];
        let result = negotiate(&local, &remote).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id.protocol.as_str(), "messaging");
    }

    #[test]
    fn spec_30_unknown_optional_capability_is_silently_ignored() {
        // messaging.formatting (id 4) is advertised but never marked
        // required — the remote not having it must not affect
        // negotiation at all, it's just absent from the negotiated set.
        let local = vec![descriptor_with_required(
            "messaging",
            ExtensionRequirement::Optional,
            &[1, 2, 4],
            &[1],
        )];
        let remote = vec![advertisement("messaging", &[1, 2])]; // no capability 4
        let result = negotiate(&local, &remote).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].capabilities,
            CapabilitySet::new([CapabilityId(1), CapabilityId(2)])
        );
    }

    // --- spec §60 "Compatibility Matrix": "each stable extension must
    // test: v1.0 ↔ v1.0, v1.0 ↔ v1.1, v1.1 ↔ v1.2, old subset ↔ new
    // superset. Major incompatibility should fail cleanly and
    // predictably." Applied to this crate's own negotiation engine as
    // the reference case — no new production code needed, since
    // `ProtocolId` already carries the major version as part of
    // identity (so a major mismatch is already just "not the same
    // protocol, not negotiated" per `missing_optional_extension_does_not_fail_session`
    // / `missing_required_extension_fails_the_session` above) and
    // `CapabilitySet::intersect` already implements "old subset ↔ new
    // superset" for minor-version capability growth. These tests exist
    // to prove that claim against the real matrix, not to add behavior.

    fn advertisement_with_major(protocol: &str, major: u16, caps: &[u32]) -> RemoteAdvertisement {
        RemoteAdvertisement {
            id: ProtocolId::new(
                NamespaceId::new("org.example.comm").unwrap(),
                ProtocolName::new(protocol).unwrap(),
                ProtocolMajor(major),
            ),
            capabilities: CapabilitySet::new(caps.iter().map(|n| CapabilityId(*n))),
        }
    }

    #[test]
    fn spec_60_v1_0_local_against_v1_0_remote_negotiates_the_shared_set() {
        let local = vec![descriptor("messaging", ExtensionRequirement::Optional, &[1, 2])];
        let remote = vec![advertisement_with_major("messaging", 1, &[1, 2])];
        let result = negotiate(&local, &remote).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].capabilities, CapabilitySet::new([CapabilityId(1), CapabilityId(2)]));
    }

    #[test]
    fn spec_60_old_v1_0_local_against_newer_v1_1_style_superset_remote() {
        // "old subset ↔ new superset": local only knows about
        // capability 1 (an "old" build); remote (a newer minor
        // version) additionally advertises capability 3. Negotiation
        // must still succeed with just the shared subset — the old
        // side is never broken by the new side knowing more.
        let local = vec![descriptor("messaging", ExtensionRequirement::Optional, &[1])];
        let remote = vec![advertisement_with_major("messaging", 1, &[1, 3])];
        let result = negotiate(&local, &remote).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].capabilities, CapabilitySet::new([CapabilityId(1)]));
    }

    #[test]
    fn spec_60_newer_v1_1_local_against_old_v1_0_style_subset_remote() {
        // The mirror image: local (newer) knows about capabilities
        // [1, 3]; remote (older) only has [1]. Must still negotiate
        // cleanly down to the shared subset, not fail.
        let local = vec![descriptor("messaging", ExtensionRequirement::Optional, &[1, 3])];
        let remote = vec![advertisement_with_major("messaging", 1, &[1])];
        let result = negotiate(&local, &remote).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].capabilities, CapabilitySet::new([CapabilityId(1)]));
    }

    #[test]
    fn spec_60_major_version_incompatibility_fails_cleanly_for_a_required_extension() {
        // A v2 remote for a v1-required local extension is a genuine
        // major incompatibility — spec §60: "should fail cleanly and
        // predictably," not silently negotiate something wrong.
        // ProtocolId includes ProtocolMajor, so this is already just
        // "no matching advertisement" from negotiate()'s point of view.
        let local = vec![descriptor("messaging", ExtensionRequirement::Required, &[1])];
        let remote = vec![advertisement_with_major("messaging", 2, &[1])];
        let err = negotiate(&local, &remote).unwrap_err();
        assert_eq!(
            err,
            NegotiationError::RequiredExtensionUnavailable(
                "org.example.comm/messaging/1".to_string()
            )
        );
    }

    #[test]
    fn spec_60_major_version_incompatibility_is_a_clean_no_op_for_an_optional_extension() {
        let local = vec![descriptor("messaging", ExtensionRequirement::Optional, &[1])];
        let remote = vec![advertisement_with_major("messaging", 2, &[1])];
        let result = negotiate(&local, &remote).unwrap();
        assert!(result.is_empty(), "no matching major version negotiated, cleanly");
    }

    // --- spec §63 "Property Tests": "duplicate advertisement is
    // deterministic." Not a proptest-suite addition this round (that's
    // real, separate follow-up work — see this crate's lib.rs "Not
    // attempted" note) but the one specific claim among §63's five
    // listed invariants that wasn't already covered by an existing
    // test elsewhere: encode/decode round trip is `framing.rs`'s
    // spec_39 tests, unknown-optional/required-capability-rejected are
    // this file's spec_11/spec_30 tests, unbounded-allocation is
    // `framing.rs`'s hostile-length test. Only "duplicate
    // advertisement" had nothing testing it yet.
    #[test]
    fn spec_63_a_duplicate_remote_advertisement_resolves_deterministically() {
        // Two advertisements for the same protocol id, differing
        // capability sets — remote_by_id's HashMap construction means
        // the LAST one in iteration order wins, which is deterministic
        // (not random/HashMap-iteration-order-dependent, since
        // .collect() processes the input Vec in its given order every
        // time). This test pins that behavior down explicitly so a
        // future refactor can't silently make it nondeterministic.
        let local = vec![descriptor("messaging", ExtensionRequirement::Optional, &[1, 2, 3])];
        let remote = vec![
            advertisement("messaging", &[1]),
            advertisement("messaging", &[1, 2, 3]), // duplicate id, listed second
        ];
        let result = negotiate(&local, &remote).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].capabilities,
            CapabilitySet::new([CapabilityId(1), CapabilityId(2), CapabilityId(3)]),
            "the second (later) advertisement for the duplicated id must win"
        );

        // Running it again must produce the exact same result — the
        // actual meaning of "deterministic."
        let result_again = negotiate(&local, &remote).unwrap();
        assert_eq!(result, result_again);
    }

    // --- spec §64 "Simulated Peer Testing" — reproduced exactly.
    #[test]
    fn spec_64_peer_b_missing_files_leaves_messaging_working_and_session_valid() {
        let local = vec![
            descriptor("messaging", ExtensionRequirement::Required, &[1]),
            descriptor("files", ExtensionRequirement::Optional, &[1]),
        ];
        let remote = vec![advertisement("messaging", &[1])]; // Peer B: messaging/1 only

        let result = negotiate(&local, &remote).unwrap(); // "session remains valid"
        let negotiated_ids: Vec<_> = result.iter().map(|e| e.id.protocol.as_str().to_string()).collect();
        assert!(negotiated_ids.contains(&"messaging".to_string())); // "messaging works"
        assert!(!negotiated_ids.contains(&"files".to_string())); // "files unsupported"
    }

    // --- spec §65 "Upgrade Example" — reproduced exactly.
    #[test]
    fn spec_65_peer_a_and_peer_b_negotiate_the_only_version_b_has() {
        // Peer A: files/1, files/2. Peer B: files/1 only. -> files/1.
        let local = vec![
            descriptor("files", ExtensionRequirement::Optional, &[1]),
            {
                let mut files_v2 = descriptor("files", ExtensionRequirement::Optional, &[1]);
                files_v2.id = ProtocolId::new(
                    NamespaceId::new("org.example.comm").unwrap(),
                    ProtocolName::new("files").unwrap(),
                    ProtocolMajor(2),
                );
                files_v2
            },
        ];
        let remote = vec![advertisement_with_major("files", 1, &[1])];

        let result = negotiate(&local, &remote).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id.major, ProtocolMajor(1));
    }

    #[test]
    fn spec_65_peer_a_and_peer_c_negotiate_the_only_version_c_has() {
        // Peer A: files/1, files/2 (same local descriptors as above).
        // Peer C: files/2 only. -> files/2. "This enables gradual
        // protocol migration": A didn't have to drop files/1 support
        // to also speak files/2 with a newer peer.
        let local = vec![
            descriptor("files", ExtensionRequirement::Optional, &[1]),
            {
                let mut files_v2 = descriptor("files", ExtensionRequirement::Optional, &[1]);
                files_v2.id = ProtocolId::new(
                    NamespaceId::new("org.example.comm").unwrap(),
                    ProtocolName::new("files").unwrap(),
                    ProtocolMajor(2),
                );
                files_v2
            },
        ];
        let remote = vec![advertisement_with_major("files", 2, &[1])];

        let result = negotiate(&local, &remote).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id.major, ProtocolMajor(2));
    }
}
