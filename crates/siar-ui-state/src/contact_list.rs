//! `ContactListState` — the UI-facing counterpart to `siar-storage`'s
//! `StoredContact`. Deliberately its own type rather than re-exporting
//! `siar_storage::StoredContact` directly: this crate stays
//! storage/infra-free (same rule `siar-domain`'s own Cargo.toml already
//! states for itself), and `apps/desktop`'s command loop is what
//! converts between the two, same pattern already used for
//! `GroupSummary`/`GroupState`.

use siar_domain::{AccountId, DeviceId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavedContact {
    pub device_id: DeviceId,
    pub account_id: AccountId,
    pub display_name: String,
    pub ticket_text: String,
    pub key_package_b64: Option<String>,
}

#[derive(Debug, Default)]
pub struct ContactListState {
    contacts: Vec<SavedContact>,
}

impl ContactListState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Bulk-loads the whole saved contact list — used once at startup
    /// once `siar-storage`'s repository has been read, replacing
    /// whatever was here before rather than merging, since a fresh
    /// load from disk is always authoritative.
    pub fn load(&mut self, contacts: Vec<SavedContact>) {
        self.contacts = contacts;
    }

    pub fn upsert(&mut self, contact: SavedContact) {
        if let Some(existing) = self
            .contacts
            .iter_mut()
            .find(|c| c.device_id == contact.device_id)
        {
            *existing = contact;
        } else {
            self.contacts.push(contact);
        }
    }

    pub fn remove(&mut self, device_id: DeviceId) {
        self.contacts.retain(|c| c.device_id != device_id);
    }

    pub fn ordered(&self) -> &[SavedContact] {
        &self.contacts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn contact(device_id: DeviceId) -> SavedContact {
        SavedContact {
            device_id,
            account_id: AccountId::new(),
            display_name: "Bob".to_string(),
            ticket_text: "fake-ticket".to_string(),
            key_package_b64: None,
        }
    }

    #[test]
    fn load_replaces_whatever_was_there_before() {
        let mut state = ContactListState::new();
        state.upsert(contact(DeviceId::new()));
        assert_eq!(state.ordered().len(), 1);

        state.load(vec![contact(DeviceId::new()), contact(DeviceId::new())]);
        assert_eq!(state.ordered().len(), 2);
    }

    #[test]
    fn upsert_inserts_then_updates_in_place() {
        let mut state = ContactListState::new();
        let id = DeviceId::new();
        state.upsert(contact(id));
        assert_eq!(state.ordered().len(), 1);

        state.upsert(SavedContact {
            display_name: "Bob (renamed)".to_string(),
            ..contact(id)
        });
        assert_eq!(state.ordered().len(), 1);
        assert_eq!(state.ordered()[0].display_name, "Bob (renamed)");
    }

    #[test]
    fn remove_deletes_only_the_matching_contact() {
        let mut state = ContactListState::new();
        let a = DeviceId::new();
        let b = DeviceId::new();
        state.upsert(contact(a));
        state.upsert(contact(b));

        state.remove(a);
        assert_eq!(state.ordered().len(), 1);
        assert_eq!(state.ordered()[0].device_id, b);
    }
}
