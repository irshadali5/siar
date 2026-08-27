//! Multi-device model (plan.md §38–40, §130).
//!
//! plan.md §7 draws the key distinction: an account is not one key, it's
//! several devices each with their own identity. This module models
//! trust between devices via linking (plan.md §42's QR pairing: an
//! already-trusted device vouches for a new one), not via a separate
//! always-present account root key — that mirrors how Signal/WhatsApp
//! linked devices actually work, and avoids inventing a key-hierarchy
//! this plan never specified.

use crate::{AccountId, DeviceId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationState {
    /// Freshly linked, not yet out-of-band confirmed (plan.md §42's QR
    /// flow gets a device to here, not further — actual fingerprint
    /// comparison is a UI flow this type doesn't perform itself).
    Unverified,
    /// A human confirmed the device's key fingerprint out-of-band.
    Verified,
    Revoked,
}

/// plan.md §40's device change events — durable, and every device that
/// learns of one applies it to its local `DeviceRegistry`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeviceEvent {
    Added {
        account: AccountId,
        device: DeviceId,
        /// Raw Ed25519 verifying-key bytes — kept as bytes here (not an
        /// `ed25519_dalek::VerifyingKey`) so this crate stays
        /// infra/crypto-free (plan.md §86); `siar-crypto` is what turns
        /// this back into a usable key when actually verifying a
        /// certificate or a signature.
        verifying_key: [u8; 32],
        signed_by: DeviceId,
    },
    Revoked {
        device: DeviceId,
    },
    Renamed {
        device: DeviceId,
        name: String,
    },
    KeyRotated {
        device: DeviceId,
        new_verifying_key: [u8; 32],
        signed_by: DeviceId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceDescriptor {
    pub account: AccountId,
    pub device: DeviceId,
    pub verifying_key: [u8; 32],
    pub trust: VerificationState,
}

/// Local view of every device a given account has (plan.md §40). Applies
/// `DeviceEvent`s — the actual signature verification that makes an
/// `Added`/`KeyRotated` event trustworthy in the first place happens
/// before the event reaches here (see `siar-crypto`'s `verify_device_certificate`)
/// — this type's job is bookkeeping, not cryptographic judgment.
#[derive(Debug, Default)]
pub struct DeviceRegistry {
    devices: Vec<DeviceDescriptor>,
}

impl DeviceRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn apply(&mut self, event: &DeviceEvent) {
        match event {
            DeviceEvent::Added {
                account,
                device,
                verifying_key,
                ..
            } => {
                if !self.devices.iter().any(|d| d.device == *device) {
                    self.devices.push(DeviceDescriptor {
                        account: *account,
                        device: *device,
                        verifying_key: *verifying_key,
                        trust: VerificationState::Unverified,
                    });
                }
            }
            DeviceEvent::Revoked { device } => {
                if let Some(d) = self.devices.iter_mut().find(|d| d.device == *device) {
                    d.trust = VerificationState::Revoked;
                }
            }
            DeviceEvent::Renamed { .. } => {
                // Display name isn't modeled on `DeviceDescriptor` yet —
                // this arm exists so the wire format is stable once it
                // is, not because it does anything today.
            }
            DeviceEvent::KeyRotated {
                device,
                new_verifying_key,
                ..
            } => {
                if let Some(d) = self.devices.iter_mut().find(|d| d.device == *device) {
                    d.verifying_key = *new_verifying_key;
                    // plan.md §40: a rotation resets trust — the new key
                    // hasn't itself been out-of-band verified yet, even
                    // if the old one was.
                    d.trust = VerificationState::Unverified;
                }
            }
        }
    }

    pub fn mark_verified(&mut self, device: DeviceId) {
        if let Some(d) = self.devices.iter_mut().find(|d| d.device == device) {
            if d.trust != VerificationState::Revoked {
                d.trust = VerificationState::Verified;
            }
        }
    }

    /// plan.md §38–39: fanout targets are every non-revoked device
    /// belonging to the account — revoked devices must not receive
    /// future messages/keys.
    pub fn active_devices(&self, account: AccountId) -> Vec<DeviceId> {
        self.devices
            .iter()
            .filter(|d| d.account == account && d.trust != VerificationState::Revoked)
            .map(|d| d.device)
            .collect()
    }

    pub fn trust_of(&self, device: DeviceId) -> Option<VerificationState> {
        self.devices
            .iter()
            .find(|d| d.device == device)
            .map(|d| d.trust)
    }
}

/// plan.md §20's `sync_cursor`: how far a given device has caught up on
/// another device's message stream (plan.md §39: sender syncs to their
/// own other devices too, not just the recipient's).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncCursor {
    pub device: DeviceId,
    pub last_sequence: u64,
}

impl SyncCursor {
    pub fn advance(self, sequence: u64) -> Self {
        Self {
            device: self.device,
            last_sequence: self.last_sequence.max(sequence),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn added_device_starts_unverified_and_active() {
        let mut reg = DeviceRegistry::new();
        let account = AccountId::new();
        let device = DeviceId::new();
        let signer = DeviceId::new();
        reg.apply(&DeviceEvent::Added {
            account,
            device,
            verifying_key: [1u8; 32],
            signed_by: signer,
        });
        assert_eq!(reg.trust_of(device), Some(VerificationState::Unverified));
        assert!(reg.active_devices(account).contains(&device));
    }

    #[test]
    fn revoked_device_is_excluded_from_fanout() {
        let mut reg = DeviceRegistry::new();
        let account = AccountId::new();
        let device = DeviceId::new();
        reg.apply(&DeviceEvent::Added {
            account,
            device,
            verifying_key: [1u8; 32],
            signed_by: DeviceId::new(),
        });
        reg.apply(&DeviceEvent::Revoked { device });

        assert_eq!(reg.trust_of(device), Some(VerificationState::Revoked));
        assert!(!reg.active_devices(account).contains(&device));
    }

    #[test]
    fn key_rotation_resets_trust_to_unverified() {
        let mut reg = DeviceRegistry::new();
        let account = AccountId::new();
        let device = DeviceId::new();
        reg.apply(&DeviceEvent::Added {
            account,
            device,
            verifying_key: [1u8; 32],
            signed_by: DeviceId::new(),
        });
        reg.mark_verified(device);
        assert_eq!(reg.trust_of(device), Some(VerificationState::Verified));

        reg.apply(&DeviceEvent::KeyRotated {
            device,
            new_verifying_key: [2u8; 32],
            signed_by: DeviceId::new(),
        });
        assert_eq!(reg.trust_of(device), Some(VerificationState::Unverified));
    }

    #[test]
    fn revoked_device_cannot_be_marked_verified_again() {
        let mut reg = DeviceRegistry::new();
        let account = AccountId::new();
        let device = DeviceId::new();
        reg.apply(&DeviceEvent::Added {
            account,
            device,
            verifying_key: [1u8; 32],
            signed_by: DeviceId::new(),
        });
        reg.apply(&DeviceEvent::Revoked { device });
        reg.mark_verified(device);
        assert_eq!(reg.trust_of(device), Some(VerificationState::Revoked));
    }

    #[test]
    fn sync_cursor_only_moves_forward() {
        let device = DeviceId::new();
        let cursor = SyncCursor {
            device,
            last_sequence: 10,
        };
        assert_eq!(cursor.advance(15).last_sequence, 15);
        assert_eq!(cursor.advance(5).last_sequence, 10); // doesn't go backward
    }
}
