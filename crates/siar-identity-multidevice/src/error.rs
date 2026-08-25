//! Typed errors for this crate — Part 01's own §27/§93 precedent
//! (`siar-protocol-ext`'s `ExtensionError`, stable
//! extension-scoped error codes) applied here: no `anyhow`, matching
//! §190 "No `anyhow` in Public Domain API" from this crate's own spec
//! document.

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum IdentityError {
    #[error("malformed key bytes")]
    MalformedKey,
    #[error("invalid signature")]
    InvalidSignature,
    #[error("device certificate's account_id does not match the expected account")]
    AccountMismatch,
    /// §56 "Rollback Protection": a directory generation at or below one
    /// already trusted for this account.
    #[error("directory generation {given} is not newer than the highest trusted generation {highest} for this account — rejected to prevent rollback")]
    RollbackRejected { given: u64, highest: u64 },
    #[error("directory signature did not verify against the account's trusted root public key")]
    DirectorySignatureInvalid,
    /// Used by [`crate::revocation::verify_revocation`] — a directory
    /// claims to reflect a specific revocation but its actual contents
    /// don't match (the target device isn't `Revoked`, or an unrelated
    /// device's status changed too). Deliberately its own variant
    /// rather than reusing `AccountMismatch` — that one means something
    /// different (a certificate's `account_id` field doesn't match),
    /// and conflating the two would make error handling silently wrong
    /// for a caller matching on this enum.
    #[error("directory does not correctly reflect the claimed revocation")]
    RevocationMismatch,
}
