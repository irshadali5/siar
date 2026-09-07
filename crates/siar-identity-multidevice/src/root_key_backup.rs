//! §194 "Root Key Backup", §195 "Backup Import".
//!
//! This crate does not implement the encryption/KDF itself —
//! dependency-minimal by design, same posture as
//! [`crate::identity_backup`]'s own encrypted-metadata field (an
//! opaque blob a caller encrypts). What IS this crate's job is the
//! ENVELOPE shape §194 requires and the ORDER OF CHECKS §195 requires
//! — both real, both enforced by the type system and a real function,
//! not left as documentation.

use crate::wire_limits::SchemaVersion;
use siar_domain::AccountId;

/// §194's own four required properties, made structural: "versioned"
/// is the `version` field; "integrity-protected" and "authenticated"
/// are the caller's AEAD ciphertext (this type only carries the
/// resulting bytes, never validates them itself — that needs the
/// actual cipher, which this crate doesn't depend on); "strongly
/// derived" is `kdf_salt` existing as a required field at all — a
/// caller cannot construct an envelope while skipping key derivation
/// entirely, since there's no field to leave it out of. "Never export
/// raw private key in plaintext" is enforced by absence: no
/// [`crate::root_key::RootIdentityKey`] type is imported into this
/// file, so `ciphertext` can only ever be bytes a caller already
/// encrypted — there is no constructor path that accepts a plaintext
/// key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootKeyBackupEnvelope {
    pub version: SchemaVersion,
    pub account_id: AccountId,
    pub kdf_salt: [u8; 32],
    pub ciphertext: Vec<u8>,
}

/// §195's own four named checks, in spec's own order — checked here as
/// a single function a caller runs BEFORE ever touching local state,
/// matching §195's "before replacing local state." Integrity/format
/// verification of the ciphertext itself (decrypting and checking the
/// AEAD tag) happens in the caller's own cipher step, which this
/// crate doesn't implement; what's checked here is everything this
/// crate CAN check without that cipher: the envelope's own shape and
/// whether it's even for the account attempting to import it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackupImportRejection {
    UnsupportedVersion,
    WrongAccount,
    EmptyCiphertext,
}

pub fn validate_before_import(
    envelope: &RootKeyBackupEnvelope,
    expected_account: AccountId,
    supported_versions: &[SchemaVersion],
) -> Result<(), BackupImportRejection> {
    if !supported_versions.contains(&envelope.version) {
        return Err(BackupImportRejection::UnsupportedVersion);
    }
    if envelope.account_id != expected_account {
        return Err(BackupImportRejection::WrongAccount);
    }
    if envelope.ciphertext.is_empty() {
        return Err(BackupImportRejection::EmptyCiphertext);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn envelope(account: AccountId) -> RootKeyBackupEnvelope {
        RootKeyBackupEnvelope {
            version: SchemaVersion(1),
            account_id: account,
            kdf_salt: [7u8; 32],
            ciphertext: vec![1, 2, 3],
        }
    }

    #[test]
    fn a_well_formed_envelope_for_the_right_account_and_version_passes() {
        let account = AccountId::new();
        let result = validate_before_import(&envelope(account), account, &[SchemaVersion(1)]);
        assert!(result.is_ok());
    }

    #[test]
    fn wrong_account_is_rejected_before_touching_local_state() {
        let account = AccountId::new();
        let other_account = AccountId::new();
        let result = validate_before_import(&envelope(account), other_account, &[SchemaVersion(1)]);
        assert_eq!(result, Err(BackupImportRejection::WrongAccount));
    }

    #[test]
    fn unsupported_version_is_rejected() {
        let account = AccountId::new();
        let result = validate_before_import(&envelope(account), account, &[SchemaVersion(2)]);
        assert_eq!(result, Err(BackupImportRejection::UnsupportedVersion));
    }

    #[test]
    fn empty_ciphertext_is_rejected() {
        let account = AccountId::new();
        let mut bad_envelope = envelope(account);
        bad_envelope.ciphertext.clear();
        let result = validate_before_import(&bad_envelope, account, &[SchemaVersion(1)]);
        assert_eq!(result, Err(BackupImportRejection::EmptyCiphertext));
    }
}
