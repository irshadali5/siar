//! Device clone detection (Part 28 §23).
//!
//! §23's problem statement: "a restored backup must not silently create
//! two indistinguishable active device instances." A `DeviceId` alone
//! can't detect this — by design, a `DeviceId` is stable across a
//! restore (it's the same logical device). `DeviceInstanceId` is the
//! extra layer: a fresh, random value generated every time a device's
//! local state is *initialized* (first install, or a restore from
//! backup), so that two processes both claiming to be the same
//! `DeviceId` but holding different `DeviceInstanceId`s are
//! detectably not the same running instance, even though nothing about
//! `DeviceId`, `DeviceCertificate`, or `DeviceDirectory` would show
//! that on its own.

use serde::{Deserialize, Serialize};
use siar_domain::DeviceId;
use std::collections::HashMap;

/// §23's literal type. Randomly generated, not derived from anything
/// else about the device — deriving it from, say, a hash of the device
/// key would make two restores of the same backup produce the *same*
/// instance ID, defeating the entire point of this type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeviceInstanceId([u8; 16]);

impl DeviceInstanceId {
    pub fn generate() -> Self {
        let mut bytes = [0u8; 16];
        getrandom(&mut bytes);
        Self(bytes)
    }

    pub const fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }
}

// Small local RNG shim rather than pulling in a new dependency: every
// other random-ID generator in this crate goes through `rand_core`'s
// `OsRng` (see `keystore.rs`, `identity.rs`), so this reuses that same
// source instead of introducing a second one.
fn getrandom(buf: &mut [u8]) {
    use rand_core::{OsRng, RngCore};
    OsRng.fill_bytes(buf);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloneVerdict {
    /// This `(DeviceId, DeviceInstanceId)` pair matches what's on
    /// record — no clone.
    Known,
    /// This is the first time this `DeviceId` has been seen — nothing
    /// to compare against yet, so nothing to flag.
    FirstSeen,
    /// Same `DeviceId`, but a *different* `DeviceInstanceId` than the
    /// one on record — either a restored backup that hasn't yet
    /// (correctly) generated a fresh instance, or two genuinely
    /// concurrent instances of the same device. This module can't
    /// distinguish those two cases on its own — see this type's own
    /// doc and `restore_safety.rs` for how a caller is expected to
    /// respond regardless of which it turns out to be.
    ConcurrentOrRestoredClone { known_instance: DeviceInstanceId },
}

/// Tracks the single `DeviceInstanceId` currently on record per
/// `DeviceId` and flags mismatches. Deliberately minimal: this has no
/// opinion on *storage* (a real deployment would persist this map, not
/// keep it in a `HashMap` that resets on restart) and no opinion on
/// *response* (revoking, prompting the user, forcing a rekey) — see
/// `restore_safety.rs` for the one concrete response this spec asks
/// for.
#[derive(Default)]
pub struct CloneDetector {
    known_instances: HashMap<DeviceId, DeviceInstanceId>,
}

impl CloneDetector {
    pub fn new() -> Self {
        Self::default()
    }

    /// Checks `instance` against whatever's on record for `device`,
    /// recording it as the new instance-of-record whenever there's no
    /// prior record or the check doesn't detect a conflict (§23 asks
    /// this to *detect* a clone, not to unilaterally decide which
    /// instance is legitimate — a `ConcurrentOrRestoredClone` verdict
    /// still updates the record to the most recently observed instance,
    /// leaving any revocation/rekey decision to the caller via
    /// `restore_safety.rs`).
    pub fn check(&mut self, device: DeviceId, instance: DeviceInstanceId) -> CloneVerdict {
        match self.known_instances.insert(device, instance) {
            None => CloneVerdict::FirstSeen,
            Some(previous) if previous == instance => CloneVerdict::Known,
            Some(previous) => CloneVerdict::ConcurrentOrRestoredClone {
                known_instance: previous,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_sighting_of_a_device_is_not_flagged() {
        let mut detector = CloneDetector::new();
        let device = DeviceId::new();
        let instance = DeviceInstanceId::generate();
        assert_eq!(detector.check(device, instance), CloneVerdict::FirstSeen);
    }

    #[test]
    fn the_same_instance_seen_again_is_known() {
        let mut detector = CloneDetector::new();
        let device = DeviceId::new();
        let instance = DeviceInstanceId::generate();
        detector.check(device, instance);
        assert_eq!(detector.check(device, instance), CloneVerdict::Known);
    }

    #[test]
    fn a_different_instance_for_the_same_device_id_is_flagged() {
        let mut detector = CloneDetector::new();
        let device = DeviceId::new();
        let original = DeviceInstanceId::generate();
        let clone = DeviceInstanceId::generate();

        detector.check(device, original);
        let verdict = detector.check(device, clone);
        assert_eq!(
            verdict,
            CloneVerdict::ConcurrentOrRestoredClone {
                known_instance: original
            }
        );
    }

    #[test]
    fn different_device_ids_are_tracked_independently() {
        let mut detector = CloneDetector::new();
        let a = DeviceId::new();
        let b = DeviceId::new();
        assert_eq!(
            detector.check(a, DeviceInstanceId::generate()),
            CloneVerdict::FirstSeen
        );
        assert_eq!(
            detector.check(b, DeviceInstanceId::generate()),
            CloneVerdict::FirstSeen
        );
    }

    #[test]
    fn generated_instance_ids_are_not_trivially_colliding() {
        let a = DeviceInstanceId::generate();
        let b = DeviceInstanceId::generate();
        assert_ne!(a, b);
    }
}
