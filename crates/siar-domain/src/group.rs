//! Group membership model (plan.md §27–28).
//!
//! This is the data shape only — durable vs. ephemeral event
//! classification, membership bookkeeping, epoch sequencing. The actual
//! group *key* cryptography (who can decrypt which epoch, how a removed
//! member is locked out of future epochs) is deliberately not here —
//! see `siar_crypto_mls::MlsGroupSession` for that, and
//! `siar_messaging::group_service`'s module doc for how the two fit
//! together (`GroupState` here stays the durable membership/epoch
//! bookkeeping either way; `MlsGroupSession` is the live, in-memory
//! crypto state layered alongside it for conversations on the MLS
//! path). plan.md §13 already sets the rule ("use a proven ratcheting
//! design rather than inventing one") for 1:1 sessions; group key
//! rotation is the same rule under higher stakes.

use crate::{AccountId, ConversationId, DeviceId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroupEpoch(u64);

impl GroupEpoch {
    pub const INITIAL: GroupEpoch = GroupEpoch(0);

    pub fn next(self) -> Self {
        Self(self.0 + 1)
    }

    pub fn number(self) -> u64 {
        self.0
    }

    /// Rebuilds an epoch from its raw number — the storage layer's
    /// counterpart to `number()`, for reading a persisted epoch back.
    pub fn from_number(n: u64) -> Self {
        Self(n)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemberRole {
    Member,
    Admin,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroupMember {
    pub account: AccountId,
    pub role: MemberRole,
    /// The epoch this member joined at — needed to know which past
    /// epochs (if any, once real MLS state exists) they were ever able
    /// to decrypt (plan.md §28: "old members must not decrypt future
    /// epochs" — the mirror fact, who *could* decrypt *past* ones,
    /// matters for correctly reasoning about forward secrecy).
    pub joined_at_epoch: GroupEpoch,
}

/// plan.md §27's durable/ephemeral split, applied at the type level so a
/// caller can't accidentally persist a `Typing` event or drop a
/// `MemberRemoved` on the floor — the two categories go into genuinely
/// different enums rather than one enum with a "durable: bool" field
/// that every match arm would have to remember to check correctly
/// (plan.md §122's illegal-states-unrepresentable rule, applied here).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DurableGroupEvent {
    MemberAdded {
        account: AccountId,
        epoch: GroupEpoch,
    },
    MemberRemoved {
        account: AccountId,
        epoch: GroupEpoch,
    },
    GroupRenamed {
        new_name: String,
    },
    /// A membership or key-material change advanced the epoch — the
    /// event itself doesn't carry the new key (that's the deferred MLS
    /// integration's job), just the fact that it happened and to which
    /// epoch, so the durable event log stays a complete audit trail even
    /// before real group crypto is wired in.
    EpochAdvanced {
        new_epoch: GroupEpoch,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EphemeralGroupEvent {
    Typing { account: AccountId },
    Online { account: AccountId },
    CallRinging { from: AccountId },
}

// `Serialize`/`Deserialize` added here: `apps/cli`'s `group-add-member`/
// `join-group` (and now the desktop group-invite UI, same mechanism)
// both postcard-(de)serialize a whole `GroupState` to hand a new
// member's `join_group_mls` call the founder's current membership
// bookkeeping (see that method's own doc comment on why this can't be
// derived automatically) — that only compiles with these two traits
// present, which this struct never actually had despite both call
// sites already relying on it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupState {
    pub conversation_id: ConversationId,
    pub epoch: GroupEpoch,
    members: Vec<GroupMember>,
}

impl GroupState {
    pub fn new(conversation_id: ConversationId, founder: AccountId) -> Self {
        Self {
            conversation_id,
            epoch: GroupEpoch::INITIAL,
            members: vec![GroupMember {
                account: founder,
                role: MemberRole::Admin,
                joined_at_epoch: GroupEpoch::INITIAL,
            }],
        }
    }

    /// Reconstructs state from already-known parts — the storage
    /// layer's job (plan.md §20's `group_state`/`group_members`
    /// tables), as opposed to `new`, which is specifically "a brand new
    /// group with one founder". Does not validate `members` against any
    /// event history; the caller (a repository reading its own rows
    /// back) is trusted to hand back what it was given.
    pub fn from_parts(
        conversation_id: ConversationId,
        epoch: GroupEpoch,
        members: Vec<GroupMember>,
    ) -> Self {
        Self {
            conversation_id,
            epoch,
            members,
        }
    }

    pub fn members(&self) -> &[GroupMember] {
        &self.members
    }

    pub fn is_member(&self, account: AccountId) -> bool {
        self.members.iter().any(|m| m.account == account)
    }

    /// plan.md §27's admin concept, made checkable rather than just
    /// stored — `siar_messaging::group_service`'s enforcement on
    /// `add_member`/`remove_member` (both paths) reads this directly.
    /// `false` for an account that isn't even a member, same as it
    /// would be for a non-admin member — this method doesn't
    /// distinguish those two cases because the caller (an
    /// authorization check) doesn't need to; either way, the answer is
    /// "no, they can't do admin things."
    pub fn is_admin(&self, account: AccountId) -> bool {
        self.members
            .iter()
            .any(|m| m.account == account && m.role == MemberRole::Admin)
    }

    /// Applies a durable event to local membership state, advancing the
    /// epoch where the event implies one (add/remove). This is *only*
    /// the bookkeeping — it does not (and, without real group crypto
    /// behind it, must not be treated as if it does) grant or revoke
    /// actual decryption capability for any epoch.
    pub fn apply(&mut self, event: &DurableGroupEvent) {
        match event {
            DurableGroupEvent::MemberAdded { account, epoch } => {
                if !self.is_member(*account) {
                    self.members.push(GroupMember {
                        account: *account,
                        role: MemberRole::Member,
                        joined_at_epoch: *epoch,
                    });
                }
            }
            DurableGroupEvent::MemberRemoved { account, .. } => {
                self.members.retain(|m| m.account != *account);
            }
            DurableGroupEvent::GroupRenamed { .. } => {}
            DurableGroupEvent::EpochAdvanced { new_epoch } => {
                self.epoch = *new_epoch;
            }
        }
    }
}

/// Which devices a durable group event actually needs to reach — kept
/// separate from `GroupState` since "who's in the group" (accounts) and
/// "which of their devices get this" (plan.md §38's multi-device fanout)
/// are different questions the sender needs to answer independently.
pub fn fanout_targets(
    members: &[GroupMember],
    devices_by_account: impl Fn(AccountId) -> Vec<DeviceId>,
) -> Vec<DeviceId> {
    members
        .iter()
        .flat_map(|m| devices_by_account(m.account))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn founder_starts_as_the_only_admin_member() {
        let founder = AccountId::new();
        let state = GroupState::new(ConversationId::new(), founder);
        assert_eq!(state.members().len(), 1);
        assert_eq!(state.members()[0].role, MemberRole::Admin);
        assert!(state.is_member(founder));
    }

    #[test]
    fn is_admin_true_for_founder_false_for_added_member() {
        let founder = AccountId::new();
        let bob = AccountId::new();
        let mut state = GroupState::new(ConversationId::new(), founder);
        state.apply(&DurableGroupEvent::MemberAdded {
            account: bob,
            epoch: GroupEpoch::INITIAL.next(),
        });

        assert!(state.is_admin(founder));
        assert!(!state.is_admin(bob));
    }

    #[test]
    fn is_admin_false_for_a_non_member() {
        let state = GroupState::new(ConversationId::new(), AccountId::new());
        assert!(!state.is_admin(AccountId::new()));
    }

    #[test]
    fn member_added_event_updates_membership() {
        let mut state = GroupState::new(ConversationId::new(), AccountId::new());
        let bob = AccountId::new();
        state.apply(&DurableGroupEvent::MemberAdded {
            account: bob,
            epoch: GroupEpoch::INITIAL.next(),
        });
        assert!(state.is_member(bob));
        assert_eq!(state.members().len(), 2);
    }

    #[test]
    fn member_removed_event_updates_membership() {
        let founder = AccountId::new();
        let bob = AccountId::new();
        let mut state = GroupState::new(ConversationId::new(), founder);
        state.apply(&DurableGroupEvent::MemberAdded {
            account: bob,
            epoch: GroupEpoch::INITIAL,
        });
        state.apply(&DurableGroupEvent::MemberRemoved {
            account: bob,
            epoch: GroupEpoch::INITIAL,
        });
        assert!(!state.is_member(bob));
        assert!(state.is_member(founder));
    }

    #[test]
    fn adding_an_existing_member_twice_is_a_no_op() {
        let mut state = GroupState::new(ConversationId::new(), AccountId::new());
        let bob = AccountId::new();
        state.apply(&DurableGroupEvent::MemberAdded {
            account: bob,
            epoch: GroupEpoch::INITIAL,
        });
        state.apply(&DurableGroupEvent::MemberAdded {
            account: bob,
            epoch: GroupEpoch::INITIAL,
        });
        assert_eq!(
            state.members().iter().filter(|m| m.account == bob).count(),
            1
        );
    }

    #[test]
    fn epoch_advances_only_on_epoch_advanced_events() {
        let mut state = GroupState::new(ConversationId::new(), AccountId::new());
        assert_eq!(state.epoch, GroupEpoch::INITIAL);
        state.apply(&DurableGroupEvent::EpochAdvanced {
            new_epoch: GroupEpoch::INITIAL.next(),
        });
        assert_eq!(state.epoch.number(), 1);
    }

    #[test]
    fn fanout_targets_covers_every_members_devices() {
        let alice = AccountId::new();
        let bob = AccountId::new();
        let members = vec![
            GroupMember {
                account: alice,
                role: MemberRole::Admin,
                joined_at_epoch: GroupEpoch::INITIAL,
            },
            GroupMember {
                account: bob,
                role: MemberRole::Member,
                joined_at_epoch: GroupEpoch::INITIAL,
            },
        ];
        let alice_devices = vec![DeviceId::new(), DeviceId::new()];
        let bob_devices = vec![DeviceId::new()];
        let alice_devices_clone = alice_devices.clone();
        let bob_devices_clone = bob_devices.clone();

        let targets = fanout_targets(&members, move |account| {
            if account == alice {
                alice_devices_clone.clone()
            } else if account == bob {
                bob_devices_clone.clone()
            } else {
                Vec::new()
            }
        });

        assert_eq!(targets.len(), 3);
        for d in alice_devices.iter().chain(bob_devices.iter()) {
            assert!(targets.contains(d));
        }
    }
}
