//! Closes this crate's own standing gap ("no §22 audit-trail event
//! log" — per [[resilient-mesh]] project memory) by turning this
//! crate's own real operations — linking a device, revoking one,
//! verifying a revocation — into `siar-event-log` [`NewEvent`]s a
//! caller can actually append.
//!
//! This module deliberately only *constructs* events; it never calls
//! `EventStore::append` itself. Every "decide" module elsewhere in
//! this workspace keeps the same split (`siar_dtn_bundle::forwarding::
//! decide_forwarding` decides a forwarding action but never dials a
//! transport; `siar_routing_policy::plan` picks a route but never
//! opens a connection) — this crate stays a policy layer, not an I/O
//! layer, and pulling in `async-trait`/an executor just to call
//! `append` here would break that pattern for no benefit, since the
//! caller already has to own the `EventStore` instance regardless.

use crate::directory::DeviceStatus;
use serde::{Deserialize, Serialize};
use siar_domain::{AccountId, DeviceId};
use siar_event_log::envelope::EventOrigin;
use siar_event_log::ids::{EventId, EventTypeId, StreamId, Timestamp};
use siar_event_log::store::NewEvent;

/// One stream per account's identity history — matches
/// `siar_dtn_bundle`/`siar_blob_manifest`'s own established pattern of
/// deriving a `StreamId` from a stable name via
/// [`StreamId::from_name`] rather than a random id, so every caller
/// who knows the account arrives at the same stream without needing a
/// separate lookup table.
pub fn identity_stream_id(account: AccountId) -> StreamId {
    StreamId::from_name(&format!("identity:{account}"))
}

/// Plain numeric tags, caller/module assigns constants — the same
/// choice `siar_dtn_bundle::types::PayloadTypeId` and
/// `siar_event_log`'s own doc comments already document the reasoning
/// for (a stable, versionable wire tag beats a string that can typo).
pub const EVENT_TYPE_DEVICE_LINKED: EventTypeId = EventTypeId(1);
pub const EVENT_TYPE_DEVICE_REVOKED: EventTypeId = EventTypeId(2);
pub const EVENT_TYPE_REVOCATION_VERIFIED: EventTypeId = EventTypeId(3);

/// The typed payload behind each of the three event types above —
/// postcard-serialized into [`NewEvent::payload`], mirroring
/// `siar_dtn_bundle::payload::PayloadReference`'s own "typed enum, not
/// raw bytes with a convention" choice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IdentityAuditPayload {
    DeviceLinked { device_id: DeviceId, generation: u64 },
    DeviceRevoked { device_id: DeviceId, generation: u64 },
    RevocationVerified { device_id: DeviceId, generation: u64 },
}

impl IdentityAuditPayload {
    fn event_type(&self) -> EventTypeId {
        match self {
            Self::DeviceLinked { .. } => EVENT_TYPE_DEVICE_LINKED,
            Self::DeviceRevoked { .. } => EVENT_TYPE_DEVICE_REVOKED,
            Self::RevocationVerified { .. } => EVENT_TYPE_REVOCATION_VERIFIED,
        }
    }

    fn into_new_event(self) -> NewEvent {
        let event_type = self.event_type();
        let payload = postcard::to_allocvec(&self)
            .expect("IdentityAuditPayload always postcard-serializes");
        NewEvent {
            event_id: EventId::new(),
            event_type,
            schema_version: 1,
            created_at: Timestamp::now(),
            origin: EventOrigin::LocalDevice(self.device_id()),
            correlation_id: None,
            causation_id: None,
            payload,
        }
    }

    fn device_id(&self) -> DeviceId {
        match self {
            Self::DeviceLinked { device_id, .. }
            | Self::DeviceRevoked { device_id, .. }
            | Self::RevocationVerified { device_id, .. } => *device_id,
        }
    }
}

/// A device successfully joined the account's [`crate::directory::DeviceDirectory`]
/// (§16-17/19-20's linking flow, already real elsewhere in this
/// crate) — call after that flow succeeds, with the directory's new
/// generation.
pub fn device_linked_event(device_id: DeviceId, new_generation: u64) -> NewEvent {
    IdentityAuditPayload::DeviceLinked { device_id, generation: new_generation }.into_new_event()
}

/// [`crate::revocation::revoke_device`] succeeded — call with its
/// returned [`crate::directory::DeviceDirectory`]'s new generation.
pub fn device_revoked_event(device_id: DeviceId, new_generation: u64) -> NewEvent {
    IdentityAuditPayload::DeviceRevoked { device_id, generation: new_generation }.into_new_event()
}

/// [`crate::revocation::verify_revocation`] succeeded on a remote
/// peer's directory — a distinct event from `DeviceRevoked` because
/// it can happen on a *different* device than the one that issued the
/// revocation (§25-27's whole point: every device independently
/// verifies a revocation it receives, it doesn't just trust that the
/// issuer did it correctly).
pub fn revocation_verified_event(device_id: DeviceId, new_generation: u64) -> NewEvent {
    IdentityAuditPayload::RevocationVerified { device_id, generation: new_generation }.into_new_event()
}

/// Reconstructs the audit payload from a [`siar_event_log::store::StoredEvent`]'s
/// raw bytes — the read-side counterpart to the three constructors
/// above, so a caller building an actual audit-trail view doesn't have
/// to know the postcard encoding itself.
pub fn decode_audit_payload(payload: &[u8]) -> Result<IdentityAuditPayload, postcard::Error> {
    postcard::from_bytes(payload)
}

/// True if `status` is the kind of status transition this module
/// bothers auditing at all — [`DeviceStatus::Expired`] has no
/// constructor function above because nothing in this crate currently
/// produces that transition (see this crate's own `lib.rs` gap list);
/// this function exists so a future caller adding that transition has
/// one obvious place to extend, rather than the omission being silent.
pub fn is_audited_status(status: DeviceStatus) -> bool {
    matches!(status, DeviceStatus::Active | DeviceStatus::Revoked)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_account_always_derives_the_same_stream_id() {
        let account = AccountId::new();
        assert_eq!(identity_stream_id(account), identity_stream_id(account));
    }

    #[test]
    fn different_accounts_derive_different_stream_ids() {
        assert_ne!(identity_stream_id(AccountId::new()), identity_stream_id(AccountId::new()));
    }

    #[test]
    fn device_linked_event_round_trips_through_postcard() {
        let device = DeviceId::new();
        let event = device_linked_event(device, 3);
        assert_eq!(event.event_type, EVENT_TYPE_DEVICE_LINKED);

        let decoded = decode_audit_payload(&event.payload).unwrap();
        assert_eq!(decoded, IdentityAuditPayload::DeviceLinked { device_id: device, generation: 3 });
    }

    #[test]
    fn device_revoked_event_round_trips_through_postcard() {
        let device = DeviceId::new();
        let event = device_revoked_event(device, 5);
        assert_eq!(event.event_type, EVENT_TYPE_DEVICE_REVOKED);

        let decoded = decode_audit_payload(&event.payload).unwrap();
        assert_eq!(decoded, IdentityAuditPayload::DeviceRevoked { device_id: device, generation: 5 });
    }

    #[test]
    fn revocation_verified_event_round_trips_through_postcard() {
        let device = DeviceId::new();
        let event = revocation_verified_event(device, 5);
        assert_eq!(event.event_type, EVENT_TYPE_REVOCATION_VERIFIED);

        let decoded = decode_audit_payload(&event.payload).unwrap();
        assert_eq!(decoded, IdentityAuditPayload::RevocationVerified { device_id: device, generation: 5 });
    }

    #[test]
    fn each_event_type_gets_a_distinct_tag() {
        let tags = [EVENT_TYPE_DEVICE_LINKED, EVENT_TYPE_DEVICE_REVOKED, EVENT_TYPE_REVOCATION_VERIFIED];
        for (i, a) in tags.iter().enumerate() {
            for b in &tags[i + 1..] {
                assert_ne!(a, b);
            }
        }
    }

    #[test]
    fn active_and_revoked_are_audited_but_expired_is_flagged_as_not_yet() {
        assert!(is_audited_status(DeviceStatus::Active));
        assert!(is_audited_status(DeviceStatus::Revoked));
        assert!(!is_audited_status(DeviceStatus::Expired));
    }
}
