//! §196 "Identity Reset", §197 "Account Deletion", §198 "Organization
//! Offboarding".
//!
//! §196/§197 both follow the same shape
//! [`crate::client_api::RemovalPresentation`]'s `backup_caveat`
//! already established: a required disclaimer field, not a comment a
//! UI author might skip — "the system must not claim otherwise" (§158)
//! and "must not promise impossible remote deletion" (§197) are the
//! same category of honesty requirement, so they get the same
//! structural fix.

use siar_domain::{AccountId, DeviceId};

/// §196: "resetting identity means new account identity, not clear app
/// cache." Checkable directly: a real reset is [`AccountId::new`]
/// producing a genuinely different id, never the same account with
/// state cleared — this function exists so that claim is testable
/// rather than only asserted in prose.
pub fn is_a_real_identity_reset(old_account: AccountId, new_account: AccountId) -> bool {
    old_account != new_account
}

/// §196's own two required UI disclosures, as required fields — a UI
/// built on this type cannot construct a reset confirmation screen
/// that omits either one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentityResetPresentation {
    pub contacts_may_see_new_identity: &'static str,
    pub old_encrypted_relationships_may_not_carry_over: &'static str,
}

impl Default for IdentityResetPresentation {
    fn default() -> Self {
        Self {
            contacts_may_see_new_identity:
                "Your contacts will see you as a new, unverified identity after this reset.",
            old_encrypted_relationships_may_not_carry_over:
                "Message history and verified relationships tied to your old identity may not carry over.",
        }
    }
}

/// §197's own four real actions, verbatim — and, just as importantly,
/// nothing resembling "delete everywhere," since that's exactly what
/// §197 says cannot be promised.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountDeletionAction {
    RevokeDevices,
    PublishTombstone,
    DeleteLocalSecrets,
    RequestServerCleanup,
}

/// §197's own required caveat, same required-field pattern as
/// [`IdentityResetPresentation`]/[`crate::client_api::RemovalPresentation`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AccountDeletionPresentation {
    pub actions_taken: &'static [AccountDeletionAction],
    pub cannot_guarantee_remote_deletion_caveat: &'static str,
}

impl Default for AccountDeletionPresentation {
    fn default() -> Self {
        Self {
            actions_taken: &[
                AccountDeletionAction::RevokeDevices,
                AccountDeletionAction::PublishTombstone,
                AccountDeletionAction::DeleteLocalSecrets,
                AccountDeletionAction::RequestServerCleanup,
            ],
            cannot_guarantee_remote_deletion_caveat:
                "This cannot delete copies already stored on other people's devices or outside our servers.",
        }
    }
}

/// §198's own four-step flow, verbatim order. A guarded state machine
/// matching [`crate::linking_state_machine::LinkingState`]'s own
/// precedent — "personal identity may remain unaffected if separated
/// properly" is why this machine only ever operates on an
/// organization-scoped [`DeviceId`], never on an [`AccountId`]; there
/// is no field or variant here that could reach a person's personal
/// account at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OffboardingStep {
    EmployeeLeft,
    OrganizationDeviceCapabilityRevoked,
    RemovedFromFutureKeys,
    FutureSessionsBlocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("cannot advance offboarding from {from:?} to {to:?}")]
pub struct InvalidOffboardingTransition {
    pub from: OffboardingStep,
    pub to: OffboardingStep,
}

impl OffboardingStep {
    pub fn new() -> Self {
        OffboardingStep::EmployeeLeft
    }

    pub fn advance(
        self,
        to: OffboardingStep,
    ) -> Result<OffboardingStep, InvalidOffboardingTransition> {
        use OffboardingStep::*;
        let valid = matches!(
            (self, to),
            (EmployeeLeft, OrganizationDeviceCapabilityRevoked)
                | (OrganizationDeviceCapabilityRevoked, RemovedFromFutureKeys)
                | (RemovedFromFutureKeys, FutureSessionsBlocked)
        );
        if valid {
            Ok(to)
        } else {
            Err(InvalidOffboardingTransition { from: self, to })
        }
    }
}

impl Default for OffboardingStep {
    fn default() -> Self {
        Self::new()
    }
}

/// The organization-scoped device this offboarding flow applies to —
/// deliberately not an [`AccountId`], since §198's "personal identity
/// may remain unaffected" only holds if the type this whole flow
/// operates on structurally cannot be a person's own account id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OffboardingTarget {
    pub organization_device_id: DeviceId,
    pub step: OffboardingStep,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_reset_producing_the_same_account_id_is_not_a_real_reset() {
        let account = AccountId::new();
        assert!(!is_a_real_identity_reset(account, account));
    }

    #[test]
    fn a_reset_producing_a_different_account_id_is_real() {
        let old = AccountId::new();
        let new = AccountId::new();
        assert!(is_a_real_identity_reset(old, new));
    }

    #[test]
    fn identity_reset_presentation_always_has_both_disclosures() {
        let presentation = IdentityResetPresentation::default();
        assert!(!presentation.contacts_may_see_new_identity.is_empty());
        assert!(!presentation
            .old_encrypted_relationships_may_not_carry_over
            .is_empty());
    }

    #[test]
    fn account_deletion_presentation_always_carries_the_remote_deletion_caveat() {
        let presentation = AccountDeletionPresentation::default();
        assert!(!presentation
            .cannot_guarantee_remote_deletion_caveat
            .is_empty());
        assert_eq!(presentation.actions_taken.len(), 4);
    }

    #[test]
    fn offboarding_cannot_skip_a_step() {
        let result = OffboardingStep::new().advance(OffboardingStep::FutureSessionsBlocked);
        assert!(result.is_err());
    }

    #[test]
    fn offboarding_walked_in_order_reaches_sessions_blocked() {
        let step = OffboardingStep::new()
            .advance(OffboardingStep::OrganizationDeviceCapabilityRevoked)
            .unwrap()
            .advance(OffboardingStep::RemovedFromFutureKeys)
            .unwrap()
            .advance(OffboardingStep::FutureSessionsBlocked)
            .unwrap();
        assert_eq!(step, OffboardingStep::FutureSessionsBlocked);
    }
}
