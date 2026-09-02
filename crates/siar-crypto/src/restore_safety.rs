//! Backup restore safety (Part 28 §24).
//!
//! §24's own reasoning: "restoring stale ratchet state can cause
//! message-key reuse, replay confusion, nonce reuse. Therefore restore
//! must force: new device incarnation, or fresh authenticated rekey —
//! rather than blindly continuing old ratchet state." This crate has no
//! ratchet yet (§11-13, still unbuilt), so there's no "old ratchet
//! state" to blindly continue *today* — but the enforcement rule §24
//! asks for doesn't depend on the ratchet existing; it depends on
//! restore-from-backup being detectable, which `clone_detection.rs`'s
//! `DeviceInstanceId` already makes possible. This module is the
//! decision point: whatever the ratchet eventually is, this is where a
//! restore gets forced onto one of §24's two safe paths instead of
//! resuming whatever it was reading before.

use crate::clone_detection::{CloneDetector, CloneVerdict, DeviceInstanceId};
use siar_domain::DeviceId;

/// §24's own two options, verbatim — not a third "resume anyway" option,
/// because that's precisely the behavior §24 says restore must never
/// fall back to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestoreDecision {
    /// Start this restored device as a brand-new instance: a fresh
    /// `DeviceInstanceId`, and — once a ratchet exists — no
    /// continuation of any prior ratchet state under the old instance.
    /// The lower-friction option: no re-authentication needed, since
    /// the device's own long-term identity key is presumably still
    /// intact in the backup.
    NewIncarnation { instance: DeviceInstanceId },
    /// Require the device to go through enrollment again as if it were
    /// new — full re-authentication via the existing linking flow
    /// (`siar_identity_multidevice`'s `DeviceLinkInvite` /
    /// `LinkingApprovalPrompt`) and a freshly negotiated session key,
    /// discarding the restored key material entirely rather than trying
    /// to safely resume any part of it.
    FreshAuthenticatedRekey,
}

/// Decides how a restore should proceed, given what `CloneDetector`
/// observed. This function's whole job is to make "just resume" an
/// unreachable outcome — every arm below returns one of §24's two safe
/// paths, never a pass-through.
///
/// `detector` is mutated as a side effect (the restored instance gets
/// recorded), matching `CloneDetector::check`'s own behavior — a
/// restore this function processes becomes the new instance-of-record
/// for `device` regardless of which decision comes back, since the
/// alternative (not recording it) would make the *next* restore of the
/// same backup look like a first sighting again instead of a second,
/// more suspicious repeat.
pub fn decide_restore(detector: &mut CloneDetector, device: DeviceId) -> RestoreDecision {
    let fresh_instance = DeviceInstanceId::generate();
    match detector.check(device, fresh_instance) {
        // No prior instance on record — nothing to have conflicted
        // with. Still a restore (a caller only calls this function for
        // one), so it still gets a fresh instance rather than reusing
        // whatever instance ID was in the backup itself; only the
        // *decision* differs by verdict, not whether an instance ID is
        // freshly generated.
        CloneVerdict::FirstSeen => RestoreDecision::NewIncarnation {
            instance: fresh_instance,
        },
        // Should not occur in practice (the instance generated above is
        // fresh every call, so it can only equal `previous` by an
        // astronomically unlikely collision) — handled the same as
        // `FirstSeen` rather than treated as an error, since either way
        // a fresh instance was just recorded.
        CloneVerdict::Known => RestoreDecision::NewIncarnation {
            instance: fresh_instance,
        },
        // A different instance was already on record for this device —
        // this restore is happening while another instance might still
        // be active (concurrent) or the backup is simply stale
        // (sequential). §24 doesn't ask this module to tell those apart
        // before deciding; both are exactly the situation "restoring
        // stale state" describes, so both get the stronger of the two
        // safe paths: force full re-authentication rather than silently
        // minting a new incarnation next to a possibly-still-live one.
        CloneVerdict::ConcurrentOrRestoredClone { .. } => RestoreDecision::FreshAuthenticatedRekey,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_first_time_restore_gets_a_new_incarnation() {
        let mut detector = CloneDetector::new();
        let device = DeviceId::new();
        let decision = decide_restore(&mut detector, device);
        assert!(matches!(decision, RestoreDecision::NewIncarnation { .. }));
    }

    #[test]
    fn restoring_the_same_backup_twice_forces_a_fresh_rekey_the_second_time() {
        let mut detector = CloneDetector::new();
        let device = DeviceId::new();

        let first = decide_restore(&mut detector, device);
        assert!(matches!(first, RestoreDecision::NewIncarnation { .. }));

        // Same backup restored again (e.g. the user re-runs the restore
        // flow, or a second physical device restores the same backup
        // file) — this must not be treated as another clean first-time
        // restore.
        let second = decide_restore(&mut detector, device);
        assert_eq!(second, RestoreDecision::FreshAuthenticatedRekey);
    }

    #[test]
    fn a_restore_updates_the_instance_of_record() {
        let mut detector = CloneDetector::new();
        let device = DeviceId::new();

        let RestoreDecision::NewIncarnation { instance } = decide_restore(&mut detector, device)
        else {
            panic!("expected NewIncarnation for a first-time restore");
        };

        // The instance minted by the restore is now what a subsequent,
        // ordinary (non-restore) sighting of this device should match.
        assert_eq!(detector.check(device, instance), CloneVerdict::Known);
    }

    #[test]
    fn different_devices_restoring_do_not_interfere_with_each_other() {
        let mut detector = CloneDetector::new();
        let device_a = DeviceId::new();
        let device_b = DeviceId::new();

        assert!(matches!(
            decide_restore(&mut detector, device_a),
            RestoreDecision::NewIncarnation { .. }
        ));
        assert!(matches!(
            decide_restore(&mut detector, device_b),
            RestoreDecision::NewIncarnation { .. }
        ));
    }
}
