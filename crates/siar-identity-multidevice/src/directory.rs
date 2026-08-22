//! §52 "Device Directory", §53 "Device Directory Entry", §55 "Stale
//! Device Directory". A signed snapshot (§52's own "can be a compact
//! snapshot derived from events" — the alternative to a full signed
//! event log this crate takes, since a snapshot's rollback rule (§56)
//! is simpler to state and enforce correctly than event-log conflict
//! resolution, and §52 explicitly allows it). This workspace already
//! has an event-sourced alternative for a *different* device-trust
//! model — `siar_domain::device::{DeviceEvent, DeviceRegistry}` — see
//! `certificate.rs`'s own doc comment for how the two relate.

use serde::{Deserialize, Serialize};

use crate::certificate::DeviceCertificate;
use crate::error::IdentityError;
use crate::root_key::{RootIdentityKey, RootPublicKey};
use siar_domain::{AccountId, DeviceId};

/// §53's status values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceStatus {
    Active,
    Revoked,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceDirectoryEntry {
    pub device_id: DeviceId,
    pub certificate: DeviceCertificate,
    pub status: DeviceStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceDirectory {
    pub account_id: AccountId,
    pub generation: u64,
    pub devices: Vec<DeviceDirectoryEntry>,
    /// See [`crate::certificate::DeviceCertificate::signature`]'s own
    /// doc comment — identical reasoning, same fix.
    pub signature: Vec<u8>,
}

impl DeviceDirectory {
    fn signing_payload(account_id: AccountId, generation: u64, devices: &[DeviceDirectoryEntry]) -> Vec<u8> {
        #[derive(Serialize)]
        struct Payload<'a> {
            account_id: AccountId,
            generation: u64,
            devices: &'a [DeviceDirectoryEntry],
        }
        postcard::to_allocvec(&Payload { account_id, generation, devices }).expect("postcard encoding of a fixed-shape struct cannot fail")
    }

    pub fn sign(root_key: &RootIdentityKey, account_id: AccountId, generation: u64, devices: Vec<DeviceDirectoryEntry>) -> Self {
        let payload = Self::signing_payload(account_id, generation, &devices);
        let signature = root_key.sign(&payload).to_vec();
        Self { account_id, generation, devices, signature }
    }

    pub fn verify_signature(&self, root_public_key: &RootPublicKey) -> Result<(), IdentityError> {
        let payload = Self::signing_payload(self.account_id, self.generation, &self.devices);
        let signature: [u8; 64] = self.signature.as_slice().try_into().map_err(|_| IdentityError::MalformedKey)?;
        root_public_key.verify(&payload, &signature)
    }

    /// §26's fan-out rule made concrete: every device this account
    /// should currently receive new data/messages/keys through.
    pub fn active_devices(&self) -> impl Iterator<Item = &DeviceDirectoryEntry> {
        self.devices.iter().filter(|d| d.status == DeviceStatus::Active)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::DeviceCapabilitySet;

    #[test]
    fn a_signed_directory_verifies_and_filters_to_active_devices() {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let active_device = DeviceId::new();
        let revoked_device = DeviceId::new();

        let active_cert = DeviceCertificate::issue(&root, account, active_device, [1u8; 32], 0, None, DeviceCapabilitySet::SEND_MESSAGE, 1);
        let revoked_cert = DeviceCertificate::issue(&root, account, revoked_device, [2u8; 32], 0, None, DeviceCapabilitySet::SEND_MESSAGE, 1);

        let dir = DeviceDirectory::sign(
            &root,
            account,
            1,
            vec![
                DeviceDirectoryEntry { device_id: active_device, certificate: active_cert, status: DeviceStatus::Active },
                DeviceDirectoryEntry { device_id: revoked_device, certificate: revoked_cert, status: DeviceStatus::Revoked },
            ],
        );

        assert!(dir.verify_signature(&root.root_public_key()).is_ok());
        let active: Vec<DeviceId> = dir.active_devices().map(|d| d.device_id).collect();
        assert_eq!(active, vec![active_device]);
    }
}
