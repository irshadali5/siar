//! §112 "Notifications", §113 "Cross-Device Consistency", §114
//! "Reconciliation", §115 "Merkle / Hash Chain Support", §119
//! "Idempotency", §120 "Replay Protection".
//!
//! §112 gets no new type here: "the identity layer emits events; the
//! product decides presentation" is already this crate's actual split
//! — [`crate::audit_log`] constructs events, [`crate::contact_verification::IdentityNotification`]
//! already carries the informational-vs-security-significant decision
//! a product's presentation layer would need, and neither module
//! renders anything or decides how to show it. There is nothing left
//! for §112 to ask this crate to build.
//!
//! §115 also gets no new type: [`crate::state_chain::AccountStateEvent`]/
//! [`crate::state_chain::StateHash`] already are "ordered signed
//! events" plus a state hash — exactly what §115 says "the first
//! implementation can start with" — see that module's own doc comment
//! for the honest scope note on how far that goes today. What's new
//! here is [`ReconciliationPlan`], which is the thing §114 actually
//! asks for: a decision made FROM a state-hash comparison, not a new
//! hashing scheme.

use std::collections::HashSet;

use crate::directory::DeviceDirectory;
use crate::state_chain::StateHash;
use siar_domain::AccountId;

/// §113: "temporary divergence is expected offline... security rules
/// must remain conservative during divergence." A plain three-way
/// comparison of what two devices believe an account's generation is
/// — deliberately not a boolean "in sync" flag, since §113 asks for
/// *conservative* behavior specifically while diverged, and a caller
/// can only do that if "diverged, and in which direction" survives as
/// distinct information rather than collapsing to `false`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConvergenceStatus {
    Converged,
    LocalAhead {
        local_generation: u64,
        remote_generation: u64,
    },
    LocalBehind {
        local_generation: u64,
        remote_generation: u64,
    },
    /// Same generation number, different state hash — not a normal
    /// divergence direction at all; this is [`crate::error::IdentityError::IdentityForkDetected`]'s
    /// own condition, surfaced here so a reconciliation caller sees it
    /// before ever reaching directory-verification code.
    SameGenerationDifferentState {
        generation: u64,
    },
}

impl ConvergenceStatus {
    pub fn compare(
        local_generation: u64,
        local_hash: StateHash,
        remote_generation: u64,
        remote_hash: StateHash,
    ) -> Self {
        use std::cmp::Ordering;
        match local_generation.cmp(&remote_generation) {
            Ordering::Equal if local_hash == remote_hash => ConvergenceStatus::Converged,
            Ordering::Equal => ConvergenceStatus::SameGenerationDifferentState {
                generation: local_generation,
            },
            Ordering::Greater => ConvergenceStatus::LocalAhead {
                local_generation,
                remote_generation,
            },
            Ordering::Less => ConvergenceStatus::LocalBehind {
                local_generation,
                remote_generation,
            },
        }
    }

    /// §113's "security rules must remain conservative during
    /// divergence" made checkable: the only status in which a caller
    /// may treat local state as authoritative and skip reconciliation
    /// entirely.
    pub fn is_safe_to_proceed_without_reconciling(&self) -> bool {
        matches!(self, ConvergenceStatus::Converged)
    }
}

/// §114's own five-step list, turned into a decision rather than a
/// procedure this crate would need to execute: "exchange generation,
/// compare state hash, request missing events, verify chain, apply."
/// The first two steps are exactly [`ConvergenceStatus::compare`]; the
/// variants below are what a caller does in response to steps 3-5 —
/// this type never itself requests, transmits, or applies anything
/// (same "decide, don't dial" split [`crate::audit_log`]'s and
/// `siar_dtn_bundle::forwarding::decide_forwarding`'s own doc comments
/// already establish for this workspace).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReconciliationPlan {
    /// Nothing to do.
    NoActionNeeded,
    /// §114's "do not exchange entire history unnecessarily": the
    /// range to request is exactly the generations between what the
    /// peer already has and what we have, never "everything."
    RequestGenerationsFrom { since_generation: u64 },
    /// We're behind — nothing to request; wait for the peer (or a
    /// relay carrying a [`crate::state_transport::SignedDeviceStateUpdate`])
    /// to bring us forward instead.
    AwaitPeerUpdate,
    /// §57/§58's fork condition, not a normal reconciliation case at
    /// all — a caller must not attempt automatic reconciliation here.
    RequiresSecurityReview { generation: u64 },
}

impl ReconciliationPlan {
    pub fn from_convergence(status: ConvergenceStatus) -> Self {
        match status {
            ConvergenceStatus::Converged => ReconciliationPlan::NoActionNeeded,
            ConvergenceStatus::LocalAhead {
                remote_generation, ..
            } => ReconciliationPlan::RequestGenerationsFrom {
                since_generation: remote_generation,
            },
            ConvergenceStatus::LocalBehind { .. } => ReconciliationPlan::AwaitPeerUpdate,
            ConvergenceStatus::SameGenerationDifferentState { generation } => {
                ReconciliationPlan::RequiresSecurityReview { generation }
            }
        }
    }
}

/// §119: "receiving the same device event repeatedly should produce
/// one logical state change... use unique event id/hash and
/// generation constraints." [`crate::trust_store::TrustedAccountStore`]
/// already gives this for whole directory snapshots (a resend at the
/// same generation is a documented no-op — see its own tests); this
/// is the same guarantee at the level §119 actually names, a single
/// device event, keyed on the event's own hash rather than the whole
/// directory's.
#[derive(Debug, Default)]
pub struct EventDeduplicator {
    seen: HashSet<[u8; 32]>,
}

impl EventDeduplicator {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns `true` the first time a given event's bytes are seen,
    /// `false` on every repeat — the "one logical state change" §119
    /// asks for is then just "only act when this returns `true`,"
    /// pushed to the caller rather than hidden inside this type
    /// deciding what "acting" means for every possible event kind.
    pub fn record_if_new(&mut self, event_bytes: &[u8]) -> bool {
        let hash = *blake3::hash(event_bytes).as_bytes();
        self.seen.insert(hash)
    }
}

/// §120's own four named replay-protection mechanisms, restated as a
/// direct pointer to where each one already lives, rather than new
/// code — a fifth mechanism duplicating any of these would only
/// create two sources of truth for the same check:
/// - "old device-add event" / generation → [`crate::error::IdentityError::RollbackRejected`]
///   in [`crate::trust_store::TrustedAccountStore::accept`], already
///   generation-gated.
/// - "old transport endpoint" / state hash → the whole
///   [`DeviceDirectory`] is one signed snapshot per generation (§52's
///   own design, see [`crate::state_chain`]'s §58 note), so there is
///   no such thing as an individually-stale transport endpoint within
///   an accepted, current-generation directory to reject — it's
///   either part of the current signed snapshot or it isn't.
/// - "old revoked certificate" / certificate generation →
///   [`crate::revocation::verify_revocation`] and
///   [`crate::secure_storage::verify_device_session_presentation`]'s
///   own certificate+generation+trust checks.
/// - expiry → [`crate::secure_storage::PrekeyPool`]'s expiry checks
///   and [`IdentityClaim::is_expired`](crate::principal_claims::IdentityClaim::is_expired).
pub struct ReplayProtectionIndex {
    account: AccountId,
    highest_known_generation: u64,
}

impl ReplayProtectionIndex {
    pub fn new(account: AccountId, highest_known_generation: u64) -> Self {
        Self {
            account,
            highest_known_generation,
        }
    }

    pub fn account(&self) -> AccountId {
        self.account
    }

    /// The one genuinely new check this module adds on top of the
    /// pointers above: a caller tracking replay protection across
    /// several accounts needs somewhere to keep "highest generation
    /// seen so far" that isn't tied to a live, currently-trusted
    /// `DeviceDirectory` (e.g. while still fetching one) — this is
    /// that minimal piece of state, not a new rejection rule.
    pub fn observe(&mut self, generation: u64) {
        self.highest_known_generation = self.highest_known_generation.max(generation);
    }

    pub fn is_stale(&self, generation: u64) -> bool {
        generation < self.highest_known_generation
    }
}

/// A convenience wrapper so callers don't have to hand-roll
/// `StateHash(directory.state_hash())` at every call site.
pub fn state_hash_of(directory: &DeviceDirectory) -> StateHash {
    StateHash(directory.state_hash())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::directory::DeviceDirectory;
    use crate::root_key::RootIdentityKey;

    fn directory_at(
        root: &RootIdentityKey,
        account: AccountId,
        generation: u64,
    ) -> DeviceDirectory {
        DeviceDirectory::sign(root, account, generation, vec![])
    }

    #[test]
    fn identical_generation_and_hash_is_converged() {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let dir = directory_at(&root, account, 3);
        let hash = state_hash_of(&dir);

        let status = ConvergenceStatus::compare(3, hash, 3, hash);
        assert_eq!(status, ConvergenceStatus::Converged);
        assert!(status.is_safe_to_proceed_without_reconciling());
        assert_eq!(
            ReconciliationPlan::from_convergence(status),
            ReconciliationPlan::NoActionNeeded
        );
    }

    #[test]
    fn local_ahead_requests_generations_since_the_peer() {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let local = state_hash_of(&directory_at(&root, account, 5));
        let remote = state_hash_of(&directory_at(&root, account, 2));

        let status = ConvergenceStatus::compare(5, local, 2, remote);
        assert!(!status.is_safe_to_proceed_without_reconciling());
        assert_eq!(
            ReconciliationPlan::from_convergence(status),
            ReconciliationPlan::RequestGenerationsFrom {
                since_generation: 2
            }
        );
    }

    #[test]
    fn local_behind_awaits_the_peer_rather_than_requesting_anything() {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let local = state_hash_of(&directory_at(&root, account, 1));
        let remote = state_hash_of(&directory_at(&root, account, 9));

        let status = ConvergenceStatus::compare(1, local, 9, remote);
        assert_eq!(
            ReconciliationPlan::from_convergence(status),
            ReconciliationPlan::AwaitPeerUpdate
        );
    }

    #[test]
    fn same_generation_different_hash_requires_security_review_not_auto_reconciliation() {
        let root_a = RootIdentityKey::generate();
        let root_b = RootIdentityKey::generate();
        let account = AccountId::new();
        // Two different (root key, devices) combos at the SAME
        // generation — the fork scenario §57 already names.
        let hash_a = state_hash_of(&directory_at(&root_a, account, 4));
        let hash_b = state_hash_of(&directory_at(&root_b, account, 4));
        assert_ne!(hash_a, hash_b);

        let status = ConvergenceStatus::compare(4, hash_a, 4, hash_b);
        assert!(!status.is_safe_to_proceed_without_reconciling());
        assert_eq!(
            ReconciliationPlan::from_convergence(status),
            ReconciliationPlan::RequiresSecurityReview { generation: 4 }
        );
    }

    #[test]
    fn event_deduplicator_only_acts_once_per_distinct_event() {
        let mut dedup = EventDeduplicator::new();
        let event = b"device-linked:some-fixed-encoding";

        assert!(dedup.record_if_new(event), "first delivery is new");
        assert!(
            !dedup.record_if_new(event),
            "a retransmit must not act again"
        );
        assert!(
            dedup.record_if_new(b"a-different-event"),
            "a genuinely different event is still new"
        );
    }

    #[test]
    fn replay_protection_index_flags_generations_older_than_the_highest_seen() {
        let account = AccountId::new();
        let mut index = ReplayProtectionIndex::new(account, 5);
        assert!(index.is_stale(3));
        assert!(!index.is_stale(5));
        assert!(!index.is_stale(7));

        index.observe(7);
        assert!(index.is_stale(6));
        assert_eq!(index.account(), account);
    }
}
