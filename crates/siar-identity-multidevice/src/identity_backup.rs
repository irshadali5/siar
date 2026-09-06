//! §155 "Backup Relationship": identity backup "should contain:
//! recovery material, public account state, encrypted necessary
//! metadata... not blindly copy: all active session state, all device
//! private keys — unless explicitly designed."

use crate::directory::DeviceDirectory;
use crate::recovery::DerivedRecoveryKey;

/// §155's own three named contents, verbatim — and, just as
/// importantly, nothing else. There is no field here for a session
/// key, a [`crate::root_key::RootIdentityKey`], or a
/// [`crate::secure_storage::SecretBytes`] — none of those types are
/// even imported into this file, so `IdentityBackup` structurally
/// cannot carry them, the same "enforced by absence" pattern
/// [`crate::platform_boundary`]'s view models already use for the
/// identical concern.
///
/// `recovery_material` is [`DerivedRecoveryKey`], not
/// [`crate::recovery::RecoverySecret`] — that type's own doc comment
/// already says it must never appear "on the wire or in a persisted
/// blob," and a backup is exactly a persisted blob. `DerivedRecoveryKey`
/// is that same module's own answer to "the ONE thing allowed to leave
/// the device."
#[derive(Debug, Clone)]
pub struct IdentityBackup {
    pub recovery_material: DerivedRecoveryKey,
    pub public_account_state: DeviceDirectory,
    /// Opaque bytes — this crate performs no encryption of its own
    /// (dependency-minimal, same posture as
    /// [`crate::recovery::RecoveryKeyDerivation`]'s KDF boundary); a
    /// caller encrypts whatever metadata it decides is "necessary"
    /// before this field is ever populated. What counts as necessary
    /// is a product decision spec doesn't make here, so this crate
    /// doesn't guess at one either.
    pub encrypted_metadata: Vec<u8>,
}

impl IdentityBackup {
    pub fn new(
        recovery_material: DerivedRecoveryKey,
        public_account_state: DeviceDirectory,
        encrypted_metadata: Vec<u8>,
    ) -> Self {
        Self {
            recovery_material,
            public_account_state,
            encrypted_metadata,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::root_key::RootIdentityKey;
    use siar_domain::AccountId;

    #[test]
    fn a_backup_carries_only_its_three_named_fields() {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let directory = DeviceDirectory::sign(&root, account, 1, vec![]);
        let backup = IdentityBackup::new(
            DerivedRecoveryKey([7u8; 32]),
            directory.clone(),
            vec![1, 2, 3],
        );

        assert_eq!(backup.recovery_material, DerivedRecoveryKey([7u8; 32]));
        assert_eq!(backup.public_account_state.account_id, directory.account_id);
        assert_eq!(backup.encrypted_metadata, vec![1, 2, 3]);
    }

    // There is deliberately no test attempting to add a session-key or
    // root-private-key field to `IdentityBackup` — that would be a
    // compile error (no such type is even imported into this file),
    // which is the actual guarantee §155's "not blindly copy... device
    // private keys" asks for, the same reasoning
    // `platform_boundary.rs`'s own tests module gives for the
    // identical kind of structural property.
}
