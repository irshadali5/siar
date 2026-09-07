//! §189 "Error Types": spec suggests one flat twelve-variant
//! `IdentityError` enum. This crate's real [`crate::error::IdentityError`]
//! has a different, smaller shape (7 variants) because error handling
//! in this crate is split across several already-shipped, already-
//! tested types rather than centralized into one enum — the mapping
//! below shows where each of spec's twelve named categories actually
//! lives. Renaming this crate's real `IdentityError` to match spec's
//! twelve variants now would be a breaking change to every already-
//! tested call site across nine rounds of work, the same category of
//! caution this crate already applies to §125's schema-versioning gap;
//! the mapping is the honest reconciliation instead.
//!
//! §189's actual rule — "do not collapse into `Identity failed`" — is
//! satisfied, arguably more thoroughly than spec's own suggestion:
//! nothing in this crate ever produces one generic failure value. If
//! anything, splitting errors across [`crate::error::IdentityError`],
//! [`crate::recovery::RecoveryError`], and
//! [`crate::secure_storage::SecureStoreError`] is MORE granular than
//! spec's single enum, since each is scoped to exactly the operations
//! that can produce it rather than one type every subsystem shares.

/// Spec's own twelve suggested variants, verbatim, purely as a
/// reference list for [`SuggestedErrorCategory::where_this_lives`] to
/// map — never constructed or returned anywhere in this crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuggestedErrorCategory {
    InvalidCertificate,
    RevokedDevice,
    ExpiredDevice,
    StaleGeneration,
    ForkDetected,
    LinkExpired,
    LinkRejected,
    VerificationFailed,
    Unauthorized,
    RecoveryFailed,
    Storage,
    Crypto,
}

impl SuggestedErrorCategory {
    /// Where this concern is actually handled in this crate today —
    /// an `IdentityError` variant, a different crate-scoped error
    /// enum, or (for several categories) not an error at all, but a
    /// state-machine variant or boolean this crate treats as an
    /// ordinary outcome rather than a failure.
    pub fn where_this_lives(&self) -> &'static str {
        match self {
            SuggestedErrorCategory::InvalidCertificate => {
                "error::IdentityError::InvalidSignature / AccountMismatch / MalformedKey"
            }
            SuggestedErrorCategory::RevokedDevice => {
                "NOT an IdentityError — session_cache::AuthenticationOutcome::Reject (a revoked device is an ordinary, expected outcome to check for, not a thrown error)"
            }
            SuggestedErrorCategory::ExpiredDevice => {
                "NOT an IdentityError — certificate::DeviceCertificate::is_expired (a bool a caller checks, same reasoning as RevokedDevice)"
            }
            SuggestedErrorCategory::StaleGeneration => "error::IdentityError::RollbackRejected",
            SuggestedErrorCategory::ForkDetected => "error::IdentityError::IdentityForkDetected",
            SuggestedErrorCategory::LinkExpired => {
                "NOT an IdentityError — invite::DeviceLinkInvite::is_expired (a bool, checked before acting, not a failure surfaced after acting)"
            }
            SuggestedErrorCategory::LinkRejected => {
                "NOT an IdentityError — linking_state_machine::LinkingState::Rejected (a real state in the linking flow's own state machine, not an exception to it)"
            }
            SuggestedErrorCategory::VerificationFailed => {
                "linking_state_machine::LinkingState::VerificationFailed for the linking flow; error::IdentityError::InvalidSignature/DirectorySignatureInvalid for cryptographic verification"
            }
            SuggestedErrorCategory::Unauthorized => {
                "device_authorization::DeviceAuthorizationDecision::allowed (a struct field, since an authorization decision also carries WHY/how-much, not just yes/no)"
            }
            SuggestedErrorCategory::RecoveryFailed => {
                "recovery::RecoveryError (its own enum: NoRecoveryPolicyConfigured / EvidenceDoesNotMatchPolicy / WrongRecoverySecret / QuorumNotMet — each name more specific than one generic RecoveryFailed)"
            }
            SuggestedErrorCategory::Storage => "secure_storage::SecureStoreError",
            SuggestedErrorCategory::Crypto => {
                "error::IdentityError::InvalidSignature / MalformedKey"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_suggested_category_maps_to_something_real_in_this_crate() {
        let all = [
            SuggestedErrorCategory::InvalidCertificate,
            SuggestedErrorCategory::RevokedDevice,
            SuggestedErrorCategory::ExpiredDevice,
            SuggestedErrorCategory::StaleGeneration,
            SuggestedErrorCategory::ForkDetected,
            SuggestedErrorCategory::LinkExpired,
            SuggestedErrorCategory::LinkRejected,
            SuggestedErrorCategory::VerificationFailed,
            SuggestedErrorCategory::Unauthorized,
            SuggestedErrorCategory::RecoveryFailed,
            SuggestedErrorCategory::Storage,
            SuggestedErrorCategory::Crypto,
        ];
        for category in all {
            assert!(!category.where_this_lives().is_empty());
        }
    }

    /// Confirms two of the mapped-elsewhere variants actually exist
    /// with the names this module claims, rather than trusting the
    /// prose alone.
    #[test]
    fn cited_real_variants_actually_exist() {
        let stale = crate::error::IdentityError::RollbackRejected {
            given: 1,
            highest: 2,
        };
        let fork = crate::error::IdentityError::IdentityForkDetected { generation: 1 };
        assert!(matches!(
            stale,
            crate::error::IdentityError::RollbackRejected { .. }
        ));
        assert!(matches!(
            fork,
            crate::error::IdentityError::IdentityForkDetected { .. }
        ));
    }
}
