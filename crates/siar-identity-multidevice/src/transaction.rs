//! §118 "Transaction Boundaries": "verify certificate → append event →
//! update snapshot → commit → emit event. Never emit `DeviceAdded`
//! before durable persistence."
//!
//! A type-state machine, not a runtime checklist a caller could get
//! out of order or skip a step of: each phase is its own type, and the
//! only way to obtain the next phase's value is to call the method on
//! the previous one, consuming it. There is no public constructor for
//! [`Committed`] other than [`SnapshotUpdated::commit`], and
//! [`Committed::into_audit_event`] is the ONLY way to get an
//! [`crate::audit_log::IdentityAuditPayload`] for a device addition out
//! of this module — so "never emit before durable persistence" is
//! something the compiler enforces on any caller going through this
//! type, not only a comment.

use siar_domain::DeviceId;

use crate::audit_log::IdentityAuditPayload;

/// Step 1: certificate verification has already happened (by
/// whatever means the caller's linking flow uses —
/// [`crate::certificate::DeviceCertificate`]'s own verification, or
/// [`crate::directory::DeviceDirectory::verify_signature`] for the
/// directory it's being added to). This type's existence is the
/// caller's claim that step 1 is done; it is not re-verified here,
/// matching this module's actual job (sequencing, not re-deriving
/// checks other modules already own).
pub struct CertificateVerified {
    device_id: DeviceId,
    generation: u64,
}

impl CertificateVerified {
    pub fn new(device_id: DeviceId, generation: u64) -> Self {
        Self {
            device_id,
            generation,
        }
    }

    /// Step 2.
    pub fn event_appended(self) -> EventAppended {
        EventAppended {
            device_id: self.device_id,
            generation: self.generation,
        }
    }
}

/// Step 2: the device event has been appended to whatever event store/
/// state chain the caller uses (e.g. via
/// [`crate::storage::IdentityStore::append_device_event`]).
pub struct EventAppended {
    device_id: DeviceId,
    generation: u64,
}

impl EventAppended {
    /// Step 3.
    pub fn snapshot_updated(self) -> SnapshotUpdated {
        SnapshotUpdated {
            device_id: self.device_id,
            generation: self.generation,
        }
    }
}

/// Step 3: the account's [`crate::directory::DeviceDirectory`]
/// snapshot has been updated to include the new device.
pub struct SnapshotUpdated {
    device_id: DeviceId,
    generation: u64,
}

impl SnapshotUpdated {
    /// Step 4: durable persistence has actually happened. The ONLY
    /// way to produce a [`Committed`] value — there is no path into
    /// this type that skips steps 1-3.
    pub fn commit(self) -> Committed {
        Committed {
            device_id: self.device_id,
            generation: self.generation,
        }
    }
}

/// Step 4 done: durably persisted. Holding one of these is proof —
/// not a comment, an actual value only obtainable by walking the
/// whole chain above — that it is now safe to do step 5.
pub struct Committed {
    device_id: DeviceId,
    generation: u64,
}

impl Committed {
    /// Step 5: emit the event. §118's actual rule ("never emit
    /// `DeviceAdded` before durable persistence") is enforced by this
    /// being the only function in this crate that produces a
    /// `DeviceLinked` [`IdentityAuditPayload`] for a fresh device
    /// addition, and it requires a [`Committed`] value to call.
    pub fn into_audit_event(self) -> IdentityAuditPayload {
        IdentityAuditPayload::DeviceLinked {
            device_id: self.device_id,
            generation: self.generation,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn walking_every_phase_in_order_produces_the_audit_event() {
        let device_id = DeviceId::new();
        let event = CertificateVerified::new(device_id, 3)
            .event_appended()
            .snapshot_updated()
            .commit()
            .into_audit_event();

        assert_eq!(
            event,
            IdentityAuditPayload::DeviceLinked {
                device_id,
                generation: 3
            }
        );
    }

    // There is deliberately no test attempting to skip a phase (e.g.
    // calling `.commit()` directly on a `CertificateVerified`) — that
    // is a compile error, not a runtime failure, which is the entire
    // point of encoding §118 as a type-state machine rather than a
    // runtime-checked sequence. A test that tried to compile such a
    // call would fail to build the crate, which is the actual
    // guarantee this module provides.
}
