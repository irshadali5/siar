//! Persistent contact book (this crate's `contacts` table — see
//! `schema.rs`'s own section). Closes a real, previously-flagged gap:
//! `apps/desktop`/`apps/cli` both regenerated their local identity and
//! opened an in-memory database on every launch, so every paired
//! peer's ticket/key-package had to be re-pasted by hand each session
//! — this repository, plus `apps/desktop/src/main.rs`'s switch to a
//! real on-disk `DeviceIdentity`/database, is what makes a saved
//! contact actually survive a restart.
//!
//! Deliberately not yet next.md §41's real contact-discovery system —
//! this still only stores what the user manually pasted in once
//! (ticket text, optionally a key package); nothing here *discovers*
//! a stranger's device. That gap was already flagged in earlier
//! sessions and stays open.

use crate::StorageError;
use siar_domain::{AccountId, DeviceId};
use stoolap::Database;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredContact {
    pub device_id: DeviceId,
    pub account_id: AccountId,
    pub display_name: String,
    pub ticket_text: String,
    /// Not every saved contact has a recorded key package — it's only
    /// captured when the add-member form (or a future "save contact"
    /// flow) was given one at save time.
    pub key_package_b64: Option<String>,
    pub added_at_millis: u64,
}

pub trait ContactRepository {
    /// Insert or fully replace the row for `contact.device_id` —
    /// re-saving an already-known device (e.g. updating its display
    /// name, or attaching a key package it didn't have before) is
    /// expected, not an error case.
    fn upsert(&self, contact: &StoredContact) -> Result<(), StorageError>;

    fn list(&self) -> Result<Vec<StoredContact>, StorageError>;

    fn remove(&self, device_id: DeviceId) -> Result<(), StorageError>;
}

pub struct StoolapContactRepository {
    db: Arc<Database>,
}

impl StoolapContactRepository {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }
}

impl ContactRepository for StoolapContactRepository {
    fn upsert(&self, contact: &StoredContact) -> Result<(), StorageError> {
        let device_id = contact.device_id.as_uuid().to_string();

        self.db
            .execute("DELETE FROM contacts WHERE device_id = $1", (device_id.clone(),))
            .map_err(StorageError::from_stoolap)?;
        self.db
            .execute(
                "INSERT INTO contacts
                    (device_id, account_id, display_name, ticket_text, key_package_b64, added_at_millis)
                 VALUES ($1, $2, $3, $4, $5, $6)",
                (
                    device_id,
                    contact.account_id.as_uuid().to_string(),
                    contact.display_name.clone(),
                    contact.ticket_text.clone(),
                    contact.key_package_b64.clone(),
                    contact.added_at_millis as i64,
                ),
            )
            .map_err(StorageError::from_stoolap)?;
        Ok(())
    }

    fn list(&self) -> Result<Vec<StoredContact>, StorageError> {
        let rows = self
            .db
            .query(
                "SELECT device_id, account_id, display_name, ticket_text, key_package_b64, added_at_millis
                 FROM contacts",
                (),
            )
            .map_err(StorageError::from_stoolap)?;

        let mut contacts = Vec::new();
        for row in rows {
            let row = row.map_err(StorageError::from_stoolap)?;
            let device_id: String = row.get(0).map_err(StorageError::from_stoolap)?;
            let account_id: String = row.get(1).map_err(StorageError::from_stoolap)?;
            let display_name: String = row.get(2).map_err(StorageError::from_stoolap)?;
            let ticket_text: String = row.get(3).map_err(StorageError::from_stoolap)?;
            let key_package_b64: Option<String> = row.get(4).map_err(StorageError::from_stoolap)?;
            let added_at_millis: i64 = row.get(5).map_err(StorageError::from_stoolap)?;

            contacts.push(StoredContact {
                device_id: DeviceId::from_uuid(parse_uuid(&device_id)?),
                account_id: AccountId::from_uuid(parse_uuid(&account_id)?),
                display_name,
                ticket_text,
                key_package_b64,
                added_at_millis: added_at_millis as u64,
            });
        }
        Ok(contacts)
    }

    fn remove(&self, device_id: DeviceId) -> Result<(), StorageError> {
        self.db
            .execute(
                "DELETE FROM contacts WHERE device_id = $1",
                (device_id.as_uuid().to_string(),),
            )
            .map_err(StorageError::from_stoolap)?;
        Ok(())
    }
}

fn parse_uuid(s: &str) -> Result<Uuid, StorageError> {
    Uuid::parse_str(s).map_err(|e| StorageError::MalformedId(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn contact(device_id: DeviceId) -> StoredContact {
        StoredContact {
            device_id,
            account_id: AccountId::new(),
            display_name: "Bob".to_string(),
            ticket_text: "fake-ticket".to_string(),
            key_package_b64: None,
            added_at_millis: 1_000,
        }
    }

    #[test]
    fn upsert_then_list_round_trips() {
        let db = crate::open_in_memory().unwrap();
        let repo = StoolapContactRepository::new(db);

        let device_id = DeviceId::new();
        repo.upsert(&contact(device_id)).unwrap();

        let contacts = repo.list().unwrap();
        assert_eq!(contacts.len(), 1);
        assert_eq!(contacts[0].device_id, device_id);
        assert_eq!(contacts[0].display_name, "Bob");
        assert_eq!(contacts[0].key_package_b64, None);
    }

    #[test]
    fn upsert_replaces_the_existing_row_for_the_same_device() {
        let db = crate::open_in_memory().unwrap();
        let repo = StoolapContactRepository::new(db);

        let device_id = DeviceId::new();
        repo.upsert(&contact(device_id)).unwrap();
        repo.upsert(&StoredContact {
            display_name: "Bob (renamed)".to_string(),
            key_package_b64: Some("YWJj".to_string()),
            ..contact(device_id)
        })
        .unwrap();

        let contacts = repo.list().unwrap();
        assert_eq!(contacts.len(), 1);
        assert_eq!(contacts[0].display_name, "Bob (renamed)");
        assert_eq!(contacts[0].key_package_b64, Some("YWJj".to_string()));
    }

    #[test]
    fn remove_deletes_the_matching_row_only() {
        let db = crate::open_in_memory().unwrap();
        let repo = StoolapContactRepository::new(db);

        let a = DeviceId::new();
        let b = DeviceId::new();
        repo.upsert(&contact(a)).unwrap();
        repo.upsert(&contact(b)).unwrap();

        repo.remove(a).unwrap();
        let contacts = repo.list().unwrap();
        assert_eq!(contacts.len(), 1);
        assert_eq!(contacts[0].device_id, b);
    }

    #[test]
    fn list_on_an_empty_store_is_an_empty_vec_not_an_error() {
        let db = crate::open_in_memory().unwrap();
        let repo = StoolapContactRepository::new(db);
        assert!(repo.list().unwrap().is_empty());
    }
}
