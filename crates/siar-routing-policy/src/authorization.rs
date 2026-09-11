//! §105 "Path Authorization", §106 "Extension Capability Integration",
//! §107 "Device Capability Integration".
//!
//! §105's own four-item list — "device active, identity trusted
//! enough, operation authorized, extension supported" — is a
//! conjunction over checks that, before this round, lived in three
//! different places and had never actually been composed into one
//! gate a caller could run. This module is that composition, not a
//! fifth reimplementation of any of the four:
//!
//! - "device active" + "identity trusted enough" are exactly what
//!   [`crate::security::authorize_candidate`] already checks in one
//!   step — [`siar_identity_multidevice::DeviceDirectory::is_device_trusted`]
//!   requires `DeviceStatus::Active` (see that method's own doc
//!   comment), so a revoked-but-still-listed device and an inactive
//!   device fail identically, the same "don't distinguish reasons
//!   that both mean don't send here" posture [`crate::security`]
//!   already documents for itself.
//! - "operation authorized" is a direct capability-bit check against
//!   the now-authenticated device's own certificate — the one piece
//!   of [`siar_identity_multidevice::device_authorization::DeviceAuthorizationDecision::combine`]
//!   that's actually this crate's business to enforce structurally.
//!   That function's *other* two inputs
//!   (`user_policy_allows`/`network_policy_allows`) are, per its own
//!   doc comment, "an application's own settings"/"a transport
//!   crate['s]" concern — this module has no opinion on either and
//!   doesn't fabricate one. A caller that also needs those two
//!   folded in calls `combine` itself afterward, using
//!   [`PathAuthorization::device_capabilities`] as the device input
//!   `combine` needs — this module hands back the fact rather than
//!   re-deriving it a second time.
//! - "extension supported" (§106) is the genuinely new check: nothing
//!   before this round verified a candidate's peer against
//!   `siar-protocol-ext`'s own negotiated-capability record
//!   ([`siar_protocol_ext::peer::PeerCapabilities`]). [`authorize_path`]
//!   takes it as `Option<&PeerCapabilities>` rather than requiring
//!   one on every call — an operation with
//!   [`crate::descriptor::OperationDescriptor::required_extension`]
//!   set to `None` (the common case) has nothing to check here at
//!   all, the same "optional plumbing, not forced on every caller"
//!   shape [`crate::resolve`] and [`crate::security`] already use for
//!   dependencies not every caller has on hand. When a requirement
//!   *is* present, no capability record for that peer is treated as
//!   "not supported," never as "unknown, so allow it" — the same
//!   conservative direction [`crate::types::MeteredState::Unknown`]
//!   already takes for a different resource.
//!
//! §107 "Device Capability Integration" is deliberately NOT another
//! function in the shape of [`authorize_path`] — its own worked
//! example ("phone supports video, headless relay does not... routing
//! should target appropriate device subset") is a *device-selection*
//! concern, one step upstream of authorizing a single already-built
//! candidate, so it's [`select_devices_with_capability`] instead:
//! filtering a resolved [`siar_domain::DeviceId`] list (straight out
//! of [`crate::resolve::resolve_destination_devices`]) down to the
//! subset whose own certificate carries a required
//! [`siar_identity_multidevice::DeviceCapabilitySet`] — the same
//! certificate field [`authorize_path`]'s "operation authorized"
//! check reads, applied across a device list instead of one
//! already-resolved candidate. §107's own example needed a capability
//! bit this crate's dependency didn't yet have —
//! [`siar_identity_multidevice::DeviceCapabilitySet::REALTIME_MEDIA`]
//! is new this round, added there rather than as a parallel type here
//! (see that constant's own doc comment).

use siar_domain::{AccountId, DeviceId};
use siar_identity_multidevice::{DeviceCapabilitySet, TrustedAccountStore};
use siar_protocol_ext::identifier::ProtocolId;
use siar_protocol_ext::peer::PeerCapabilities;

use crate::candidate::PathCandidate;
use crate::error::RoutingError;
use crate::security::{authorize_candidate, AuthenticatedSession};

/// §105's full witness. Unlike [`AuthenticatedSession`] (device
/// active + identity trusted only), holding one of these means a
/// candidate has passed **all four** of §105's checks, not just the
/// first two. Same smart-constructor shape as `AuthenticatedSession`
/// itself: no public fields, the only way to get one is
/// [`authorize_path`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PathAuthorization {
    session: AuthenticatedSession,
    device_capabilities: DeviceCapabilitySet,
}

impl PathAuthorization {
    pub fn session(&self) -> AuthenticatedSession {
        self.session
    }

    /// The authorized device's own certificate capabilities — exposed
    /// so a caller that also needs
    /// [`siar_identity_multidevice::device_authorization::DeviceAuthorizationDecision::combine`]'s
    /// user-policy/network-policy inputs folded in doesn't have to
    /// re-fetch the certificate a second time just to get this.
    pub fn device_capabilities(&self) -> DeviceCapabilitySet {
        self.device_capabilities
    }
}

/// §105's full gate, run in the order the spec lists: device
/// active/identity trusted, then operation authorized, then extension
/// supported. Fails on the first check a candidate doesn't pass
/// rather than collecting every failure — §48's "must never be
/// selected" is a hard stop, not something worth a multi-error report
/// for what is, per candidate, a single yes/no admission decision.
#[allow(clippy::result_large_err)]
pub fn authorize_path(
    candidate: &PathCandidate,
    account_id: AccountId,
    trust_store: &TrustedAccountStore,
    required_capability: DeviceCapabilitySet,
    required_extension: Option<&ProtocolId>,
    peer_capabilities: Option<&PeerCapabilities>,
) -> Result<PathAuthorization, RoutingError> {
    // §105 items 1-2: device active + identity trusted.
    let session = authorize_candidate(candidate, account_id, trust_store)?;

    // Item 3: operation authorized — the device's own certificate
    // must carry the capability this operation needs. Re-fetches the
    // directory rather than trusting `candidate` to carry the
    // certificate itself, for the same reason `authorize_candidate`
    // re-verifies trust at the candidate stage instead of relying on
    // resolve-time filtering: a caller could be holding a candidate
    // built before this device's certificate changed.
    let directory = trust_store
        .directory_for(account_id)
        .ok_or(RoutingError::UnknownDestination)?;
    let device_capabilities = directory
        .devices
        .iter()
        .find(|entry| entry.device_id == session.device_id())
        .map(|entry| entry.certificate.capabilities)
        .ok_or(RoutingError::UnknownDestination)?;

    if !device_capabilities.contains(required_capability) {
        return Err(RoutingError::OperationNotAuthorized);
    }

    // Item 4: extension supported — §106's own worked example
    // ("files/1 required, remote only supports messaging → path
    // invalid for file operation"). No record for this peer is
    // treated the same as a record that doesn't include the
    // extension, never as "unknown, so allow it."
    if let Some(extension) = required_extension {
        let supported = peer_capabilities
            .map(|caps| caps.supports(extension))
            .unwrap_or(false);
        if !supported {
            return Err(RoutingError::ExtensionNotSupported);
        }
    }

    Ok(PathAuthorization {
        session,
        device_capabilities,
    })
}

/// §105 applied across a candidate list, same "composable, caller
/// opts in" shape as [`crate::security::eliminate_untrusted_candidates`].
/// `peer_capabilities_for` is a caller-supplied lookup rather than a
/// map this function takes ownership of — a
/// [`siar_protocol_ext::peer::PeerCapabilityCache`]-backed closure is
/// the expected real caller, but this function doesn't need to know
/// that, the same loose coupling [`crate::dispatch`] already has to
/// `siar-protocol-ext`'s scheduler internals.
pub fn eliminate_paths_lacking_authorization<'a>(
    candidates: &'a [PathCandidate],
    account_id: AccountId,
    trust_store: &TrustedAccountStore,
    required_capability: DeviceCapabilitySet,
    required_extension: Option<&ProtocolId>,
    peer_capabilities_for: impl Fn(DeviceId) -> Option<PeerCapabilities>,
) -> Vec<&'a PathCandidate> {
    candidates
        .iter()
        .filter(|c| {
            let caps = peer_capabilities_for(c.peer);
            authorize_path(
                c,
                account_id,
                trust_store,
                required_capability,
                required_extension,
                caps.as_ref(),
            )
            .is_ok()
        })
        .collect()
}

/// §107's own worked example, generalized: narrow a resolved device
/// list (typically [`crate::resolve::resolve_destination_devices`]'s
/// output) down to the subset whose certificate carries
/// `required_capability` — "call to Bob: phone supports video,
/// headless relay does not" is exactly
/// `required_capability = DeviceCapabilitySet::REALTIME_MEDIA`
/// filtering `[phone, headless_relay]` down to `[phone]`. A device
/// this directory doesn't currently trust (never certified, or
/// revoked/expired since) is excluded the same way
/// [`crate::security::authorize_candidate`] excludes it — not a
/// separate reason, the same one.
pub fn select_devices_with_capability(
    device_ids: &[DeviceId],
    account_id: AccountId,
    trust_store: &TrustedAccountStore,
    required_capability: DeviceCapabilitySet,
) -> Vec<DeviceId> {
    let Some(directory) = trust_store.directory_for(account_id) else {
        return Vec::new();
    };

    device_ids
        .iter()
        .filter(|device_id| {
            directory
                .devices
                .iter()
                .find(|entry| entry.device_id == **device_id)
                .is_some_and(|entry| entry.certificate.capabilities.contains(required_capability))
        })
        .copied()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::candidate::TransportEndpoint;
    use crate::metrics::PathMetrics;
    use crate::types::{PathCapabilities, PathId, RouteHealth, TransportKind};
    use siar_identity_multidevice::{
        DeviceCertificate, DeviceDirectory, DeviceDirectoryEntry, DeviceStatus, RootIdentityKey,
    };
    use siar_protocol_ext::capability::CapabilityId;
    use siar_protocol_ext::descriptor::{NegotiatedExtension, SessionLocalExtensionId};
    use siar_protocol_ext::identifier::{NamespaceId, ProtocolMajor, ProtocolName};

    fn files_v1() -> ProtocolId {
        ProtocolId::new(
            NamespaceId::new("org.siar").unwrap(),
            ProtocolName::new("files").unwrap(),
            ProtocolMajor(1),
        )
    }

    fn messaging_v1() -> ProtocolId {
        ProtocolId::new(
            NamespaceId::new("org.siar").unwrap(),
            ProtocolName::new("messaging").unwrap(),
            ProtocolMajor(1),
        )
    }

    fn candidate_for(peer: DeviceId) -> PathCandidate {
        PathCandidate {
            path_id: PathId::new(),
            transport: TransportKind::IrohDirect,
            peer,
            endpoint: TransportEndpoint(Vec::new()),
            metrics: PathMetrics::unknown(),
            capabilities: PathCapabilities {
                reliable_stream: true,
                datagram: false,
                large_files: false,
                realtime_media: false,
                peer_discovery: false,
                store_and_forward: false,
                metered: crate::types::MeteredState::Unknown,
                roaming: crate::types::RoamingState::Unknown,
                requires_foreground: false,
            },
            health: RouteHealth::Healthy,
            underlay: None,
            state: crate::acquisition::CandidateState::Active,
        }
    }

    /// Same construction pattern [`crate::resolve`]/[`crate::security`]'s
    /// own tests already use, but with a caller-chosen capability set
    /// so tests can exercise both a device that has
    /// `required_capability` and one that doesn't.
    fn trusted_store_with_one_device(
        capabilities: DeviceCapabilitySet,
    ) -> (TrustedAccountStore, AccountId, DeviceId) {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let device = DeviceId::new();

        let cert =
            DeviceCertificate::issue(&root, account, device, [1u8; 32], 0, None, capabilities, 1);
        let directory = DeviceDirectory::sign(
            &root,
            account,
            1,
            vec![DeviceDirectoryEntry {
                device_id: device,
                certificate: cert,
                status: DeviceStatus::Active,
                transport_endpoints: vec![],
            }],
        );

        let mut store = TrustedAccountStore::new();
        store.accept(directory, &root.root_public_key()).unwrap();
        (store, account, device)
    }

    fn peer_caps_supporting(protocol: ProtocolId) -> PeerCapabilities {
        PeerCapabilities::from_negotiated(&[NegotiatedExtension {
            id: protocol,
            session_id: SessionLocalExtensionId(1),
            capabilities: siar_protocol_ext::capability::CapabilitySet::new([CapabilityId(1)]),
        }])
    }

    #[test]
    fn a_device_with_capability_and_no_extension_requirement_is_authorized() {
        let (store, account, device) =
            trusted_store_with_one_device(DeviceCapabilitySet::SEND_MESSAGE);
        let candidate = candidate_for(device);

        let auth = authorize_path(
            &candidate,
            account,
            &store,
            DeviceCapabilitySet::SEND_MESSAGE,
            None,
            None,
        )
        .unwrap();

        assert_eq!(auth.session().device_id(), device);
        assert_eq!(
            auth.device_capabilities(),
            DeviceCapabilitySet::SEND_MESSAGE
        );
    }

    #[test]
    fn a_device_lacking_the_required_capability_is_rejected() {
        let (store, account, device) =
            trusted_store_with_one_device(DeviceCapabilitySet::SEND_MESSAGE);
        let candidate = candidate_for(device);

        let err = authorize_path(
            &candidate,
            account,
            &store,
            DeviceCapabilitySet::REALTIME_MEDIA,
            None,
            None,
        )
        .unwrap_err();

        assert_eq!(err, RoutingError::OperationNotAuthorized);
    }

    #[test]
    fn an_untrusted_device_fails_before_capability_is_even_checked() {
        let (store, account, _device) = trusted_store_with_one_device(DeviceCapabilitySet::ALL);
        let stranger = candidate_for(DeviceId::new());

        let err = authorize_path(
            &stranger,
            account,
            &store,
            DeviceCapabilitySet::SEND_MESSAGE,
            None,
            None,
        )
        .unwrap_err();

        assert_eq!(err, RoutingError::UnauthorizedDevice);
    }

    #[test]
    fn required_extension_actually_negotiated_by_the_peer_passes() {
        let (store, account, device) =
            trusted_store_with_one_device(DeviceCapabilitySet::SEND_MESSAGE);
        let candidate = candidate_for(device);
        let caps = peer_caps_supporting(files_v1());

        let auth = authorize_path(
            &candidate,
            account,
            &store,
            DeviceCapabilitySet::SEND_MESSAGE,
            Some(&files_v1()),
            Some(&caps),
        )
        .unwrap();

        assert_eq!(auth.session().device_id(), device);
    }

    #[test]
    fn required_extension_the_peer_never_negotiated_fails_this_spec_106_example() {
        // §106's own worked example: "files/1 required, remote only
        // supports messaging → path invalid for file operation."
        let (store, account, device) =
            trusted_store_with_one_device(DeviceCapabilitySet::SEND_MESSAGE);
        let candidate = candidate_for(device);
        let caps = peer_caps_supporting(messaging_v1());

        let err = authorize_path(
            &candidate,
            account,
            &store,
            DeviceCapabilitySet::SEND_MESSAGE,
            Some(&files_v1()),
            Some(&caps),
        )
        .unwrap_err();

        assert_eq!(err, RoutingError::ExtensionNotSupported);
    }

    #[test]
    fn required_extension_with_no_capability_record_at_all_fails_not_passes_by_default() {
        let (store, account, device) =
            trusted_store_with_one_device(DeviceCapabilitySet::SEND_MESSAGE);
        let candidate = candidate_for(device);

        let err = authorize_path(
            &candidate,
            account,
            &store,
            DeviceCapabilitySet::SEND_MESSAGE,
            Some(&files_v1()),
            None,
        )
        .unwrap_err();

        assert_eq!(err, RoutingError::ExtensionNotSupported);
    }

    #[test]
    fn eliminate_paths_lacking_authorization_keeps_only_fully_authorized_candidates() {
        let (store, account, capable_device) =
            trusted_store_with_one_device(DeviceCapabilitySet::REALTIME_MEDIA);
        let incapable_device = DeviceId::new();
        let candidates = vec![
            candidate_for(capable_device),
            candidate_for(incapable_device),
        ];

        let kept = eliminate_paths_lacking_authorization(
            &candidates,
            account,
            &store,
            DeviceCapabilitySet::REALTIME_MEDIA,
            None,
            |_| None,
        );

        assert_eq!(kept.len(), 1);
        assert_eq!(kept[0].peer, capable_device);
    }

    #[test]
    fn select_devices_with_capability_matches_spec_107s_phone_vs_relay_example() {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let phone = DeviceId::new();
        let headless_relay = DeviceId::new();

        let phone_cert = DeviceCertificate::issue(
            &root,
            account,
            phone,
            [1u8; 32],
            0,
            None,
            DeviceCapabilitySet::SEND_MESSAGE.union(DeviceCapabilitySet::REALTIME_MEDIA),
            1,
        );
        let relay_cert = DeviceCertificate::issue(
            &root,
            account,
            headless_relay,
            [2u8; 32],
            0,
            None,
            DeviceCapabilitySet::SEND_MESSAGE.union(DeviceCapabilitySet::RELAY),
            1,
        );
        let directory = DeviceDirectory::sign(
            &root,
            account,
            1,
            vec![
                DeviceDirectoryEntry {
                    device_id: phone,
                    certificate: phone_cert,
                    status: DeviceStatus::Active,
                    transport_endpoints: vec![],
                },
                DeviceDirectoryEntry {
                    device_id: headless_relay,
                    certificate: relay_cert,
                    status: DeviceStatus::Active,
                    transport_endpoints: vec![],
                },
            ],
        );
        let mut store = TrustedAccountStore::new();
        store.accept(directory, &root.root_public_key()).unwrap();

        let targets = select_devices_with_capability(
            &[phone, headless_relay],
            account,
            &store,
            DeviceCapabilitySet::REALTIME_MEDIA,
        );

        assert_eq!(targets, vec![phone]);
    }

    #[test]
    fn select_devices_with_capability_returns_empty_for_an_unknown_account() {
        let store = TrustedAccountStore::new();
        let targets = select_devices_with_capability(
            &[DeviceId::new()],
            AccountId::new(),
            &store,
            DeviceCapabilitySet::REALTIME_MEDIA,
        );
        assert!(targets.is_empty());
    }
}
