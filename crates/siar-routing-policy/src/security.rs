//! §47 "Peer Session Abstraction", §48 "Security Constraints".
//!
//! §47: "Routing should route through authenticated sessions, not raw
//! transport addresses. DeviceId → Path → Transport Connection →
//! Authenticated Session. This ties routing back to Part 02
//! identity." §48 (immediately following): a device that fails that
//! chain — untrusted, revoked, unknown to the destination account's
//! directory — "must never be selected," full stop, regardless of how
//! well it scores.
//!
//! [`crate::resolve::resolve_destination_devices`] already applies
//! Part 02 trust at the *device-list* stage (§16/§17) — the set of
//! `DeviceId`s a caller is even allowed to build candidates for is
//! already trust-filtered before this module ever runs, the same way
//! `siar-identity-multidevice`'s own `routing_integration.rs` filters
//! stale endpoints out of `resolve_account_endpoints` before a router
//! ever sees them. What this module adds is the second, independent
//! check §48 actually asks for: a **defense-in-depth**
//! re-verification at the *candidate* stage, for exactly the case §48
//! exists to guard against — a device that was trusted when the
//! candidate list was built and was revoked in the time since. A
//! caller that already re-resolves on every revocation doesn't
//! strictly need this; a caller that doesn't gets the guarantee
//! anyway, because this module makes it structurally hard to route to
//! an unauthenticated peer at all rather than relying on every caller
//! remembering to check.

use siar_domain::{AccountId, DeviceId};
use siar_identity_multidevice::TrustedAccountStore;

use crate::candidate::PathCandidate;
use crate::error::RoutingError;

/// §47's "Authenticated Session" as a real type, not just spec prose
/// — a witness that `device_id` was, at construction time, a trusted
/// device on `account_id`'s directory. Deliberately has no public
/// constructor other than [`authorize_candidate`]: the only way to
/// get one is to pass the trust check, which is the whole point of a
/// "smart constructor" — a caller cannot accidentally skip §48's
/// check and still end up holding a value that claims to represent
/// one, since the type can't be built any other way. This is the
/// same "make the illegal state unrepresentable" reasoning
/// [`crate::requirements::DeliveryRequirements`]'s own constructors
/// already lean on for the class/priority pairings §7 pins down —
/// applied here to a trust boundary instead of a data-consistency one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuthenticatedSession {
    device_id: DeviceId,
    account_id: AccountId,
}

impl AuthenticatedSession {
    pub fn device_id(&self) -> DeviceId {
        self.device_id
    }

    pub fn account_id(&self) -> AccountId {
        self.account_id
    }
}

/// §47+§48 together: re-verify `candidate.peer` against
/// `account_id`'s current directory in `trust_store`, and only on
/// success hand back the [`AuthenticatedSession`] witness. This is the
/// crate's only path to constructing one.
///
/// Fails (§48's "must never be selected") when: the account has no
/// directory in `trust_store` at all (nothing to authenticate
/// against); or the directory exists but doesn't currently trust
/// `candidate.peer` — covers "not this account's device at all" and
/// "was a device, got revoked/suspended since" identically, since
/// [`siar_identity_multidevice::DeviceDirectory::is_device_trusted`]
/// already collapses both to `false` (see that method's own doc
/// comment on why a wrong-account query and a revoked-device query
/// aren't distinguished — from the routing side, both mean "do not
/// send here" for the same reason).
#[allow(clippy::result_large_err)]
pub fn authorize_candidate(
    candidate: &PathCandidate,
    account_id: AccountId,
    trust_store: &TrustedAccountStore,
) -> Result<AuthenticatedSession, RoutingError> {
    let directory = trust_store
        .directory_for(account_id)
        .ok_or(RoutingError::UnknownDestination)?;

    if !directory.is_device_trusted(candidate.peer) {
        return Err(RoutingError::UnauthorizedDevice);
    }

    Ok(AuthenticatedSession {
        device_id: candidate.peer,
        account_id,
    })
}

/// §48 applied across a whole candidate list, mirroring
/// [`crate::scoring::passes_hard_constraints`]'s "hard constraint,
/// checked before scoring, not folded into the weighted sum" shape —
/// trust is exactly that kind of constraint: a candidate that fails
/// it isn't merely a worse choice, it "must never be selected" at any
/// score. Composable, like [`crate::privacy::eliminate_privacy_violations`]:
/// a caller that has a `TrustedAccountStore` on hand calls this before
/// [`crate::scoring::eliminate_hard_constraint_violations`]/
/// [`crate::plan::plan_route`]; a caller that doesn't (this crate's own
/// [`crate::dispatch`]/[`crate::plan`] tests, for instance, which never
/// touch `siar-identity-multidevice`) simply doesn't call it, exactly
/// as [`crate::resolve`] is already optional plumbing rather than
/// something [`crate::plan::plan_route`] forces on every caller.
pub fn eliminate_untrusted_candidates<'a>(
    candidates: &'a [PathCandidate],
    account_id: AccountId,
    trust_store: &TrustedAccountStore,
) -> Vec<&'a PathCandidate> {
    candidates
        .iter()
        .filter(|c| authorize_candidate(c, account_id, trust_store).is_ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::candidate::TransportEndpoint;
    use crate::metrics::PathMetrics;
    use crate::types::{PathCapabilities, PathId, RouteHealth, TransportKind};
    use siar_identity_multidevice::{
        DeviceCapabilitySet, DeviceCertificate, DeviceDirectory, DeviceDirectoryEntry,
        DeviceStatus, RootIdentityKey,
    };

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
            },
            health: RouteHealth::Healthy,
            underlay: None,
        }
    }

    /// Same construction pattern [`crate::resolve`]'s own tests
    /// already use — a real signed directory with one active device,
    /// accepted into a real `TrustedAccountStore`.
    fn trusted_store_with_one_active_device() -> (TrustedAccountStore, AccountId, DeviceId) {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let device = DeviceId::new();

        let cert = DeviceCertificate::issue(
            &root,
            account,
            device,
            [1u8; 32],
            0,
            None,
            DeviceCapabilitySet::SEND_MESSAGE,
            1,
        );
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

    #[test]
    fn a_trusted_device_authorizes_successfully() {
        let (store, account_id, trusted_device) = trusted_store_with_one_active_device();
        let candidate = candidate_for(trusted_device);

        let session = authorize_candidate(&candidate, account_id, &store).unwrap();
        assert_eq!(session.device_id(), trusted_device);
        assert_eq!(session.account_id(), account_id);
    }

    #[test]
    fn an_unknown_device_is_rejected_not_merely_scored_lower() {
        let (store, account_id, _trusted_device) = trusted_store_with_one_active_device();
        let stranger = candidate_for(DeviceId::new());

        assert!(authorize_candidate(&stranger, account_id, &store).is_err());
    }

    #[test]
    fn an_unknown_account_id_is_rejected() {
        let (store, _account_id, trusted_device) = trusted_store_with_one_active_device();
        let candidate = candidate_for(trusted_device);

        let err = authorize_candidate(&candidate, AccountId::new(), &store).unwrap_err();
        assert_eq!(err, RoutingError::UnknownDestination);
    }

    #[test]
    fn eliminate_untrusted_candidates_keeps_only_the_trusted_ones() {
        let (store, account_id, trusted_device) = trusted_store_with_one_active_device();
        let candidates = vec![
            candidate_for(trusted_device),
            candidate_for(DeviceId::new()),
        ];

        let kept = eliminate_untrusted_candidates(&candidates, account_id, &store);
        assert_eq!(kept.len(), 1);
        assert_eq!(kept[0].peer, trusted_device);
    }
}
