//! Metadata minimization: scoped pseudonyms and rotating proximity
//! discovery identifiers (Part 28 §34, §35, §36).
//!
//! §35's own requirement — "same peer → different pseudonym per
//! extension" — and §36's — rotating proximity discovery IDs instead
//! of stable ones — are the same underlying primitive used two ways:
//! a one-way, keyed derivation from a stable identifier plus a context
//! tag, such that two outputs derived under different contexts (or
//! different epochs, for the rotating case) cannot be linked back to
//! the same input without already knowing that input. Both are built
//! on the same `derive_scoped_value` here rather than as two unrelated
//! functions that happen to look similar.

use blake3::Hasher;
use serde::{Deserialize, Serialize};
use siar_domain::DeviceId;

use crate::epoch::SecurityEpoch;

const OUTPUT_LEN: usize = 16;

// Distinct leading tags so that `derive_scoped_pseudonym` and
// `derive_rotating_discovery_id` can never collide even if a caller's
// `context` bytes happen to equal some epoch's big-endian encoding (or
// vice versa) — each derivation lives in its own namespace before the
// caller-supplied bytes are even hashed, not just distinguished by
// what the caller happens to pass in.
const PSEUDONYM_TAG: &[u8] = b"siar-crypto/scoped-pseudonym/v1";
const DISCOVERY_TAG: &[u8] = b"siar-crypto/rotating-discovery-id/v1";

/// One-way derivation: `blake3(tag || stable_id || context)`, truncated
/// to 16 bytes. Blake3's preimage resistance is what makes this safe to
/// hand to an untrusted context (a plugin, a nearby BLE scanner) —
/// recovering `stable_id` from the output, or correlating two outputs
/// produced under different `context` values as coming from the same
/// `stable_id`, is exactly the hardness property a cryptographic hash
/// provides.
fn derive_scoped_value(tag: &[u8], stable_id: DeviceId, context: &[u8]) -> [u8; OUTPUT_LEN] {
    let mut hasher = Hasher::new();
    hasher.update(tag);
    hasher.update(stable_id.as_uuid().as_bytes());
    hasher.update(context);
    let mut output = [0u8; OUTPUT_LEN];
    output.copy_from_slice(&hasher.finalize().as_bytes()[..OUTPUT_LEN]);
    output
}

/// §34/§35: a per-context pseudonym for a device. Two different
/// `context` values (e.g. two different plugin/extension IDs) for the
/// same `DeviceId` produce unrelated-looking `ScopedPseudonym`s — §35's
/// own stated goal, "prevents easy cross-plugin correlation."
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ScopedPseudonym([u8; OUTPUT_LEN]);

/// `context` should be something stable and unique to the scope being
/// separated from every other scope — an extension/plugin ID is the
/// use case §35 names explicitly. Deterministic: the same
/// `(device, context)` pair always derives the same pseudonym, which is
/// necessary for a plugin to recognize "the same peer" across repeated
/// interactions without ever learning that peer's real `DeviceId`.
pub fn derive_scoped_pseudonym(device: DeviceId, context: &[u8]) -> ScopedPseudonym {
    ScopedPseudonym(derive_scoped_value(PSEUDONYM_TAG, device, context))
}

/// §36: an ephemeral identifier for proximity discovery (BLE/LAN
/// advertisement) that rotates with `epoch` rather than staying stable
/// like the underlying `DeviceId` would. "Identity becomes known only
/// after authenticated handshake" (§36's own text) — this type is
/// exactly what gets broadcast *before* that handshake; nothing about
/// it should let a passive nearby observer link two rotations together
/// or recover the real device identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RotatingDiscoveryId([u8; OUTPUT_LEN]);

/// Reuses `SecurityEpoch` (§22) as the rotation driver rather than
/// inventing a separate proximity-specific counter — advancing the
/// account's security epoch (e.g. on device revocation) already
/// naturally rotates every discovery ID derived from it too, which is
/// a reasonable side benefit rather than a coincidence: a revoked
/// device's *old* discovery IDs stop being derivable by anything that
/// only has the new epoch.
pub fn derive_rotating_discovery_id(device: DeviceId, epoch: SecurityEpoch) -> RotatingDiscoveryId {
    RotatingDiscoveryId(derive_scoped_value(
        DISCOVERY_TAG,
        device,
        &epoch.as_u64().to_be_bytes(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_same_device_and_context_always_derives_the_same_pseudonym() {
        let device = DeviceId::new();
        let a = derive_scoped_pseudonym(device, b"plugin-a");
        let b = derive_scoped_pseudonym(device, b"plugin-a");
        assert_eq!(a, b);
    }

    #[test]
    fn different_contexts_for_the_same_device_are_unlinkable_pseudonyms() {
        let device = DeviceId::new();
        let for_plugin_a = derive_scoped_pseudonym(device, b"plugin-a");
        let for_plugin_b = derive_scoped_pseudonym(device, b"plugin-b");
        assert_ne!(for_plugin_a, for_plugin_b);
    }

    #[test]
    fn different_devices_under_the_same_context_get_different_pseudonyms() {
        let context = b"plugin-a";
        let a = derive_scoped_pseudonym(DeviceId::new(), context);
        let b = derive_scoped_pseudonym(DeviceId::new(), context);
        assert_ne!(a, b);
    }

    #[test]
    fn discovery_id_rotates_across_epochs() {
        let device = DeviceId::new();
        let epoch0 = derive_rotating_discovery_id(device, SecurityEpoch(0));
        let epoch1 = derive_rotating_discovery_id(device, SecurityEpoch(1));
        assert_ne!(epoch0, epoch1);
    }

    #[test]
    fn discovery_id_is_stable_within_the_same_epoch() {
        let device = DeviceId::new();
        let first = derive_rotating_discovery_id(device, SecurityEpoch(5));
        let second = derive_rotating_discovery_id(device, SecurityEpoch(5));
        assert_eq!(first, second);
    }

    #[test]
    fn pseudonyms_and_discovery_ids_for_the_same_device_do_not_collide() {
        // Deliberately identical caller-supplied bytes for both calls
        // (context `[0,0,0,0,0,0,0,0]` vs. epoch 0's own big-endian
        // encoding, which is the same 8 zero bytes) — this is exactly
        // the case the tag prefixes exist to keep separate. Before the
        // tag fix, this would have been a real collision: both calls
        // reduced to `derive_scoped_value(device, [0u8; 8])`.
        let device = DeviceId::new();
        let pseudonym = derive_scoped_pseudonym(device, &0u64.to_be_bytes());
        let discovery_id = derive_rotating_discovery_id(device, SecurityEpoch(0));
        assert_ne!(pseudonym.0, discovery_id.0);
    }
}
