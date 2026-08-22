//! Durable group repository (plan.md §20's `group_state`/would-be
//! `conversation_members` tables), replacing `siar-messaging`'s
//! previous `InMemoryGroupRepository` placeholder.
//!
//! Same "not yet verified against a real stoolap build" status as the
//! rest of this crate (see `lib.rs`'s module doc) — written against the
//! same `execute`/`query`/`row.get::<T>(index)` shapes already used by
//! `message_repo.rs`/`outbox_repo.rs`, not run through a compiler here.

use crate::StorageError;
use siar_domain::{AccountId, ConversationId, GroupEpoch, GroupMember, GroupState, MemberRole};
use stoolap::Database;
use std::sync::Arc;
use uuid::Uuid;

pub trait GroupRepository {
    /// Replaces the persisted state for `state.conversation_id` with
    /// `state` in full — group membership changes (add/remove/epoch
    /// advance) are infrequent enough that a full replace-under-
    /// transaction is simpler and safer than diffing member rows, and
    /// avoids a stale row lingering if a member was removed.
    fn upsert(&self, state: &GroupState) -> Result<(), StorageError>;

    fn get(&self, conversation: ConversationId) -> Result<Option<GroupState>, StorageError>;
}

pub struct StoolapGroupRepository {
    db: Arc<Database>,
}

impl StoolapGroupRepository {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }
}

impl GroupRepository for StoolapGroupRepository {
    fn upsert(&self, state: &GroupState) -> Result<(), StorageError> {
        let conversation_id = state.conversation_id.as_uuid().to_string();

        let run = || -> Result<(), StorageError> {
            self.db
                .execute(
                    "DELETE FROM group_state WHERE conversation_id = $1",
                    (conversation_id.clone(),),
                )
                .map_err(StorageError::from_stoolap)?;
            self.db
                .execute(
                    "INSERT INTO group_state (conversation_id, epoch) VALUES ($1, $2)",
                    (conversation_id.clone(), state.epoch.number() as i64),
                )
                .map_err(StorageError::from_stoolap)?;

            self.db
                .execute(
                    "DELETE FROM group_members WHERE conversation_id = $1",
                    (conversation_id.clone(),),
                )
                .map_err(StorageError::from_stoolap)?;
            for member in state.members() {
                self.db
                    .execute(
                        "INSERT INTO group_members
                            (conversation_id, account_id, role, joined_at_epoch)
                         VALUES ($1, $2, $3, $4)",
                        (
                            conversation_id.clone(),
                            member.account.as_uuid().to_string(),
                            role_to_str(member.role).to_string(),
                            member.joined_at_epoch.number() as i64,
                        ),
                    )
                    .map_err(StorageError::from_stoolap)?;
            }
            Ok(())
        };

        // plan.md §16's transactional-write rule, applied to group
        // state: a crash mid-replace must not leave `group_state` and
        // `group_members` disagreeing with each other.
        self.db.execute("BEGIN", ()).map_err(StorageError::from_stoolap)?;
        match run() {
            Ok(()) => {
                self.db.execute("COMMIT", ()).map_err(StorageError::from_stoolap)?;
                Ok(())
            }
            Err(e) => {
                let _ = self.db.execute("ROLLBACK", ());
                Err(e)
            }
        }
    }

    fn get(&self, conversation: ConversationId) -> Result<Option<GroupState>, StorageError> {
        let conversation_id = conversation.as_uuid().to_string();

        let epoch = {
            let rows = self
                .db
                .query(
                    "SELECT epoch FROM group_state WHERE conversation_id = $1",
                    (conversation_id.clone(),),
                )
                .map_err(StorageError::from_stoolap)?;
            let mut found = None;
            for row in rows {
                let row = row.map_err(StorageError::from_stoolap)?;
                let epoch: i64 = row.get(0).map_err(StorageError::from_stoolap)?;
                found = Some(GroupEpoch::from_number(epoch as u64));
                break;
            }
            found
        };
        let Some(epoch) = epoch else {
            return Ok(None);
        };

        let rows = self
            .db
            .query(
                "SELECT account_id, role, joined_at_epoch
                 FROM group_members
                 WHERE conversation_id = $1",
                (conversation_id,),
            )
            .map_err(StorageError::from_stoolap)?;

        let mut members = Vec::new();
        for row in rows {
            let row = row.map_err(StorageError::from_stoolap)?;
            let account_id: String = row.get(0).map_err(StorageError::from_stoolap)?;
            let role: String = row.get(1).map_err(StorageError::from_stoolap)?;
            let joined_at_epoch: i64 = row.get(2).map_err(StorageError::from_stoolap)?;

            members.push(GroupMember {
                account: AccountId::from_uuid(parse_uuid(&account_id)?),
                role: str_to_role(&role)?,
                joined_at_epoch: GroupEpoch::from_number(joined_at_epoch as u64),
            });
        }

        Ok(Some(GroupState::from_parts(conversation, epoch, members)))
    }
}

fn parse_uuid(s: &str) -> Result<Uuid, StorageError> {
    Uuid::parse_str(s).map_err(|e| StorageError::MalformedId(e.to_string()))
}

fn role_to_str(role: MemberRole) -> &'static str {
    match role {
        MemberRole::Member => "member",
        MemberRole::Admin => "admin",
    }
}

fn str_to_role(s: &str) -> Result<MemberRole, StorageError> {
    Ok(match s {
        "member" => MemberRole::Member,
        "admin" => MemberRole::Admin,
        other => return Err(StorageError::MalformedId(format!("unknown group role '{other}'"))),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use siar_domain::AccountId;

    #[test]
    fn upsert_then_get_round_trips() {
        let db = crate::open_in_memory().unwrap();
        let repo = StoolapGroupRepository::new(db);

        let founder = AccountId::new();
        let conversation = ConversationId::new();
        let state = GroupState::new(conversation, founder);
        repo.upsert(&state).unwrap();

        let round_tripped = repo.get(conversation).unwrap().unwrap();
        assert_eq!(round_tripped.epoch, state.epoch);
        assert_eq!(round_tripped.members().len(), 1);
        assert_eq!(round_tripped.members()[0].account, founder);
        assert_eq!(round_tripped.members()[0].role, MemberRole::Admin);
    }

    #[test]
    fn get_returns_none_for_unknown_conversation() {
        let db = crate::open_in_memory().unwrap();
        let repo = StoolapGroupRepository::new(db);
        assert!(repo.get(ConversationId::new()).unwrap().is_none());
    }

    #[test]
    fn upsert_replaces_membership_on_removal() {
        let db = crate::open_in_memory().unwrap();
        let repo = StoolapGroupRepository::new(db);

        let founder = AccountId::new();
        let bob = AccountId::new();
        let conversation = ConversationId::new();
        let mut state = GroupState::new(conversation, founder);
        state.apply(&siar_domain::DurableGroupEvent::MemberAdded {
            account: bob,
            epoch: GroupEpoch::INITIAL.next(),
        });
        repo.upsert(&state).unwrap();
        assert_eq!(repo.get(conversation).unwrap().unwrap().members().len(), 2);

        state.apply(&siar_domain::DurableGroupEvent::MemberRemoved { account: bob, epoch: state.epoch });
        repo.upsert(&state).unwrap();
        let round_tripped = repo.get(conversation).unwrap().unwrap();
        assert_eq!(round_tripped.members().len(), 1);
        assert_eq!(round_tripped.members()[0].account, founder);
    }
}
