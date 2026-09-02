//! Identity & Verification section (ui-ux-15 §48-59).
//!
//! §48: "shows user's own identity health and contact-verification
//! summary" — two distinct halves, kept as separate types below
//! (`OwnIdentityView` vs. `ContactVerificationSummary`) rather than one
//! combined struct, since a component rendering one rarely needs the
//! other in the same re-render.

use siar_domain::AccountId;

/// §51, verbatim.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentityKeyHealth {
    Healthy,
    RotationRecommended,
    RecoveryRequired,
    Compromised,
}

/// §49-50: own identity display. `short_fingerprint`/`full_fingerprint`
/// are pre-formatted strings this crate is handed, not computed here —
/// this crate's own dependency rule (`siar-domain` only, see `lib.rs`'s
/// top doc) means it never touches `siar-identity-multidevice`'s
/// `SafetyFingerprint` or any other key material directly. §50: "copy/
/// share only explicitly" — `full_fingerprint` is a separate field from
/// `short_fingerprint` specifically so a component can withhold
/// rendering/copy access to the full one behind an explicit reveal
/// action, never surfacing it automatically alongside the short one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnIdentityView {
    pub account: AccountId,
    pub short_fingerprint: String,
    pub full_fingerprint: String,
    pub created_at_millis: u64,
    pub key_health: IdentityKeyHealth,
}

impl OwnIdentityView {
    /// §52: "Normal users should see: 'Security keys updated' — not key
    /// epochs." The one piece of display logic this type owns; no raw
    /// epoch number, error code, or internal state name appears in any
    /// arm.
    pub const fn key_health_message(&self) -> &'static str {
        match self.key_health {
            IdentityKeyHealth::Healthy => "Your security keys are up to date.",
            IdentityKeyHealth::RotationRecommended => "Security keys updated.",
            IdentityKeyHealth::RecoveryRequired => {
                "Account recovery is required to restore full security."
            }
            IdentityKeyHealth::Compromised => {
                "Your identity may be compromised. Immediate action recommended."
            }
        }
    }
}

/// §58-59: contact verification summary. References ui-ux-08
/// (Contacts/Requests/Verification/Identity — not yet built in this
/// workspace, see ROADMAP.md) — this is the Security Center's own
/// summary surface over that screen's data, not a replacement for it;
/// a caller populates `ContactVerificationSummary` from whatever
/// eventually tracks per-contact verification state, the same
/// translated-input shape every other `*State`/`*Summary` type in this
/// crate already uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContactVerificationIssueKind {
    IdentityChanged,
    VerificationExpired,
    VerificationInvalid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContactVerificationIssue {
    pub contact: AccountId,
    pub kind: ContactVerificationIssueKind,
}

#[derive(Debug, Default)]
pub struct ContactVerificationSummary {
    issues: Vec<ContactVerificationIssue>,
}

impl ContactVerificationSummary {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_issues(&mut self, issues: Vec<ContactVerificationIssue>) {
        self.issues = issues;
    }

    /// §58's own worked example: "3 contacts need verification review."
    pub fn needs_review_count(&self) -> usize {
        self.issues.len()
    }

    /// §59: "high-priority list" — an identity change is a stronger
    /// signal than an expired-but-never-invalid verification, so it
    /// sorts first. Verification-invalid (actively wrong, not just
    /// stale) outranks merely-expired for the same reason.
    pub fn high_priority_first(&self) -> Vec<&ContactVerificationIssue> {
        let mut sorted: Vec<&ContactVerificationIssue> = self.issues.iter().collect();
        sorted.sort_by_key(|issue| match issue.kind {
            ContactVerificationIssueKind::IdentityChanged => 0,
            ContactVerificationIssueKind::VerificationInvalid => 1,
            ContactVerificationIssueKind::VerificationExpired => 2,
        });
        sorted
    }
}

/// §55-57: Identity Reset confirmation gate. §56 requires all three of
/// "clear consequences, reauthentication, typed/secondary confirmation"
/// — tracked as three independent flags rather than one, so a caller
/// can't accidentally allow reset after satisfying only two of the
/// three. §57 ("place under Advanced/Dangerous Actions") is a
/// navigation/placement concern this type doesn't enforce — only
/// `can_proceed`'s actual safety gate is this type's job.
#[derive(Debug, Default)]
pub struct IdentityResetConfirmation {
    consequences_reviewed: bool,
    reauthenticated: bool,
    typed_confirmation_matched: bool,
}

impl IdentityResetConfirmation {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn mark_consequences_reviewed(&mut self) {
        self.consequences_reviewed = true;
    }

    pub fn mark_reauthenticated(&mut self) {
        self.reauthenticated = true;
    }

    /// §56's "typed/secondary confirmation." Only ever records whether
    /// `typed` matched `expected` — neither string is retained, since
    /// there's nothing this type needs them for once compared.
    pub fn check_typed_confirmation(&mut self, typed: &str, expected: &str) {
        self.typed_confirmation_matched = typed == expected;
    }

    /// §56's actual safety gate: all three requirements met, not just
    /// some.
    pub fn can_proceed(&self) -> bool {
        self.consequences_reviewed && self.reauthenticated && self.typed_confirmation_matched
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_identity(health: IdentityKeyHealth) -> OwnIdentityView {
        OwnIdentityView {
            account: AccountId::new(),
            short_fingerprint: "AB12 CD34".to_string(),
            full_fingerprint: "AB12 CD34 EF56 ...".to_string(),
            created_at_millis: 1_700_000_000_000,
            key_health: health,
        }
    }

    #[test]
    fn key_health_message_never_mentions_epochs() {
        for health in [
            IdentityKeyHealth::Healthy,
            IdentityKeyHealth::RotationRecommended,
            IdentityKeyHealth::RecoveryRequired,
            IdentityKeyHealth::Compromised,
        ] {
            let message = sample_identity(health).key_health_message();
            assert!(!message.to_lowercase().contains("epoch"));
        }
    }

    #[test]
    fn rotation_recommended_shows_the_spec_exact_wording() {
        let identity = sample_identity(IdentityKeyHealth::RotationRecommended);
        assert_eq!(identity.key_health_message(), "Security keys updated.");
    }

    #[test]
    fn contact_verification_needs_review_count_matches_issue_count() {
        let mut summary = ContactVerificationSummary::new();
        summary.set_issues(vec![
            ContactVerificationIssue {
                contact: AccountId::new(),
                kind: ContactVerificationIssueKind::IdentityChanged,
            },
            ContactVerificationIssue {
                contact: AccountId::new(),
                kind: ContactVerificationIssueKind::VerificationExpired,
            },
            ContactVerificationIssue {
                contact: AccountId::new(),
                kind: ContactVerificationIssueKind::VerificationInvalid,
            },
        ]);
        assert_eq!(summary.needs_review_count(), 3);
    }

    #[test]
    fn identity_changed_issues_sort_before_expired_ones() {
        let mut summary = ContactVerificationSummary::new();
        summary.set_issues(vec![
            ContactVerificationIssue {
                contact: AccountId::new(),
                kind: ContactVerificationIssueKind::VerificationExpired,
            },
            ContactVerificationIssue {
                contact: AccountId::new(),
                kind: ContactVerificationIssueKind::IdentityChanged,
            },
        ]);
        let ordered = summary.high_priority_first();
        assert_eq!(
            ordered[0].kind,
            ContactVerificationIssueKind::IdentityChanged
        );
        assert_eq!(
            ordered[1].kind,
            ContactVerificationIssueKind::VerificationExpired
        );
    }

    #[test]
    fn identity_reset_requires_all_three_conditions() {
        let mut confirmation = IdentityResetConfirmation::new();
        assert!(!confirmation.can_proceed());

        confirmation.mark_consequences_reviewed();
        assert!(!confirmation.can_proceed());

        confirmation.mark_reauthenticated();
        assert!(!confirmation.can_proceed());

        confirmation.check_typed_confirmation("wrong", "RESET");
        assert!(!confirmation.can_proceed());

        confirmation.check_typed_confirmation("RESET", "RESET");
        assert!(confirmation.can_proceed());
    }

    #[test]
    fn a_mismatched_retyped_confirmation_revokes_the_gate() {
        let mut confirmation = IdentityResetConfirmation::new();
        confirmation.mark_consequences_reviewed();
        confirmation.mark_reauthenticated();
        confirmation.check_typed_confirmation("RESET", "RESET");
        assert!(confirmation.can_proceed());

        // User edits the field again and it no longer matches — the
        // gate must close again, not remain open from the earlier
        // correct attempt.
        confirmation.check_typed_confirmation("RES", "RESET");
        assert!(!confirmation.can_proceed());
    }
}
