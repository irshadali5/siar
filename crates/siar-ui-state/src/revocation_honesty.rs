//! Revocation and recovery honesty (ui-ux-15 §179-183).
//!
//! This whole section is one theme: don't let the UI claim more than
//! the backend actually guarantees. Every type/function here exists to
//! make an honest claim the *only* claim available, the same
//! "capability-gated wording" pattern `revocation_lifecycle.rs`'s
//! `WipeCapability`/`lost_device_action_label` already established for
//! wipe — applied here to session invalidation and to what recovery
//! can and can't restore.

/// §179: "if architecture has session tokens: revocation invalidates
/// them. UI can say 'This signs the device out' only if true." A
/// caller-supplied fact about the real architecture, never assumed —
/// assuming `true` by default would be exactly the overclaim §179
/// warns against.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RevocationCapabilities {
    pub invalidates_session_tokens: bool,
}

impl RevocationCapabilities {
    /// Returns the "signs the device out" phrase only when it's
    /// actually true — `None` otherwise, so a caller has to
    /// deliberately handle the false case rather than a default string
    /// silently rendering an overclaim.
    pub const fn sign_out_copy(self) -> Option<&'static str> {
        if self.invalidates_session_tokens {
            Some("This signs the device out.")
        } else {
            None
        }
    }
}

/// §180-181: "revocation cannot erase copies already exported/
/// decrypted on another device... do not claim 'all data is erased
/// remotely' unless guaranteed." Always the same honest text — this
/// isn't conditional on any capability flag because §180's underlying
/// fact (revocation can't reach data already copied elsewhere) is true
/// regardless of architecture; there's no configuration under which a
/// stronger claim would become accurate.
pub const REVOCATION_DATA_DISCLAIMER: &str =
    "Revoking this device stops future access. Any data already copied or decrypted elsewhere by that device cannot be erased remotely.";

/// §182: "explain what recovery can restore: account identity,
/// encrypted backup, contacts, history — depending on the actual
/// architecture." A caller-supplied fact set, same reasoning as
/// `RecoveryCapabilities` (`presentation_api.rs`, §136) — this is a
/// different type from that one deliberately: `RecoveryCapabilities`
/// is about which *actions* (create/rotate/test/export) are available,
/// `RecoveryScope` is about which *data categories* a successful
/// recovery actually restores. Conflating the two would make "can I
/// rotate my recovery key" and "will recovery bring back my message
/// history" the same question when they're not.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecoveryScope {
    pub restores_account_identity: bool,
    pub restores_encrypted_backup: bool,
    pub restores_contacts: bool,
    pub restores_history: bool,
}

impl RecoveryScope {
    /// §183: "if no backup/archive exists: recovery may restore
    /// account access but not missing message history." The one piece
    /// of conditional honesty this type owns — when history isn't
    /// restorable, say so explicitly rather than staying silent about
    /// it (silence would read as "recovery restores everything," the
    /// same overclaim §182-183 exist to prevent).
    pub fn history_caveat(self) -> Option<&'static str> {
        if self.restores_history {
            None
        } else {
            Some("Recovery may restore account access but not missing message history.")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_out_copy_is_none_when_tokens_are_not_actually_invalidated() {
        let capabilities = RevocationCapabilities {
            invalidates_session_tokens: false,
        };
        assert_eq!(capabilities.sign_out_copy(), None);
    }

    #[test]
    fn sign_out_copy_is_present_only_when_true() {
        let capabilities = RevocationCapabilities {
            invalidates_session_tokens: true,
        };
        assert_eq!(
            capabilities.sign_out_copy(),
            Some("This signs the device out.")
        );
    }

    #[test]
    fn history_caveat_appears_only_when_history_is_not_restorable() {
        let full_scope = RecoveryScope {
            restores_account_identity: true,
            restores_encrypted_backup: true,
            restores_contacts: true,
            restores_history: true,
        };
        assert_eq!(full_scope.history_caveat(), None);

        let no_history = RecoveryScope {
            restores_history: false,
            ..full_scope
        };
        assert!(no_history.history_caveat().is_some());
        assert!(no_history
            .history_caveat()
            .unwrap()
            .contains("not missing message history"));
    }

    #[test]
    fn revocation_disclaimer_never_claims_remote_erasure() {
        assert!(!REVOCATION_DATA_DISCLAIMER
            .to_lowercase()
            .contains("all data is erased"));
    }
}
