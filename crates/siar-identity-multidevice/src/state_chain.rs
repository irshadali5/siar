//! §58 "Concurrent Device Changes", §59 "Account State Chain".
//!
//! §58 needs no new code — it's already resolved by this crate's
//! existing design choice, not something to build fresh. Spec §58
//! names three options ("single account authority, quorum, serialized
//! event chain") and recommends the simplest for an initial
//! implementation. This crate's actual model
//! ([`crate::directory::DeviceDirectory`]: one root key signs the
//! *entire* directory snapshot at each generation) already IS spec
//! §58's first option, "single account authority" — there is
//! structurally no way for two different callers to produce two
//! independently-valid directories at the same generation without
//! access to the same root key, which is exactly what makes forks
//! (§57) a signal of `bug`/`compromise`/`concurrent invalid update`
//! rather than a normal occurrence this crate needs a resolution
//! protocol for.

use crate::directory::DeviceStatus;
use siar_domain::{AccountId, DeviceId};

/// §59's `prev_hash` field's type — a plain named wrapper around
/// [`crate::directory::DeviceDirectory::state_hash`]'s output, so
/// call sites read as "a state hash" rather than an anonymous
/// `[u8; 32]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct StateHash(pub [u8; 32]);

/// §59's own `DeviceEvent` — not given a variant list anywhere in this
/// document, so this crate supplies the one it actually has real
/// operations for: every one of [`crate::rotation::rotate_device_key`]/
/// [`crate::revocation::revoke_device`]/[`crate::recovery::add_device_via_recovery`]
/// corresponds to exactly one variant here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DeviceEvent {
    Linked {
        device_id: DeviceId,
    },
    Rotated {
        device_id: DeviceId,
    },
    Revoked {
        device_id: DeviceId,
    },
    RecoveredViaPolicy {
        device_id: DeviceId,
    },
    StatusChanged {
        device_id: DeviceId,
        new_status: DeviceStatus,
    },
}

/// §59, verbatim struct shape.
///
/// **Honest scope note**: this crate's live, production data path is
/// still the signed-snapshot model in [`crate::directory`] — this type
/// is a real, usable bridging primitive (a caller building tamper-
/// evident audit history CAN construct and verify a chain of these,
/// referencing each snapshot's real [`crate::directory::DeviceDirectory::state_hash`]),
/// not a claim that this crate's core flow has switched to a live
/// event-chain architecture. `siar-event-log` (Part 04) is this
/// workspace's real event-log crate and, per its own `lib.rs` doc
/// comment, does not yet implement hash-chaining/integrity linkage
/// itself — so `prev_hash`-style tamper evidence doesn't exist
/// end-to-end anywhere in this workspace yet. This type is this
/// crate's honest contribution toward that: the identity-side shape a
/// real chain would need, not a claim the chain is wired up.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AccountStateEvent {
    pub account_id: AccountId,
    pub generation: u64,
    pub prev_hash: StateHash,
    pub event: DeviceEvent,
}

impl AccountStateEvent {
    /// A hash of this event itself, suitable as the NEXT event's
    /// `prev_hash` — deterministic (same event always hashes the same)
    /// and covers every field, so tampering with `prev_hash`, `event`,
    /// or `generation` after the fact is detectable by anyone
    /// re-deriving this hash and comparing.
    pub fn hash(&self) -> StateHash {
        let bytes = postcard::to_allocvec(self).expect("fixed-size fields never fail to serialize");
        StateHash(blake3::hash(&bytes).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::DeviceCapabilitySet;
    use crate::certificate::DeviceCertificate;
    use crate::directory::{DeviceDirectory, DeviceDirectoryEntry};
    use crate::root_key::RootIdentityKey;

    #[test]
    fn spec_59_a_chain_of_events_is_tamper_evident() {
        let account = AccountId::new();
        let device = DeviceId::new();

        let genesis_hash = StateHash([0u8; 32]); // no previous state yet
        let event1 = AccountStateEvent {
            account_id: account,
            generation: 1,
            prev_hash: genesis_hash,
            event: DeviceEvent::Linked { device_id: device },
        };
        let event2 = AccountStateEvent {
            account_id: account,
            generation: 2,
            prev_hash: event1.hash(),
            event: DeviceEvent::Rotated { device_id: device },
        };

        // The chain is intact: event2 really does reference event1's
        // real hash.
        assert_eq!(event2.prev_hash, event1.hash());

        // Tamper with event1 after the fact (e.g. claiming it was a
        // Revoked, not a Linked, event) — its hash changes, so event2's
        // stored prev_hash no longer matches, exactly the detection
        // §59 exists for.
        let tampered_event1 = AccountStateEvent {
            event: DeviceEvent::Revoked { device_id: device },
            ..event1
        };
        assert_ne!(event2.prev_hash, tampered_event1.hash());
    }

    #[test]
    fn spec_59_prev_hash_can_reference_a_real_directory_snapshot() {
        // Confirms the bridge to the crate's actual live model: a
        // real DeviceDirectory's state_hash() is exactly the kind of
        // value StateHash/prev_hash is meant to carry.
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
            0,
        );
        let directory = DeviceDirectory::sign(
            &root,
            account,
            0,
            vec![DeviceDirectoryEntry {
                device_id: device,
                certificate: cert,
                status: DeviceStatus::Active,
                transport_endpoints: vec![],
            }],
        );

        let next_event = AccountStateEvent {
            account_id: account,
            generation: 1,
            prev_hash: StateHash(directory.state_hash()),
            event: DeviceEvent::Rotated { device_id: device },
        };
        assert_eq!(next_event.prev_hash.0, directory.state_hash());
    }
}
