//! Recovery section (ui-ux-15 §60-73).
//!
//! Genuinely new subsystem — nothing in this workspace tracks recovery
//! method/status/rotation state anywhere else yet (confirmed against
//! ROADMAP.md before starting this module). §72-73 are the sections
//! that matter most here: "old recovery must remain valid until new
//! method is fully confirmed" and "never leave user with: old key
//! invalid, new key not saved." `RecoveryRotationFlow` exists
//! specifically to make that ordering a type-level guarantee rather
//! than a convention a caller has to remember — see its own doc
//! comment for how.

use siar_domain::AccountId;

/// §61's five listed possible methods, verbatim (including the
/// spec's own caveat on the last one — "Trusted contact? only if
/// designed"). "UI shows only actually supported methods" (§61's own
/// closing line) is enforced upstream, by whatever populates
/// `RecoveryOverview::configured_methods` — this enum itself lists
/// every method the *architecture* might support, not a claim that
/// all five are implemented; a deployment with only `RecoveryKey`
/// wired up simply never constructs the other four.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryMethod {
    RecoveryKey,
    RecoveryPassphrase,
    TrustedDeviceApproval,
    EncryptedBackup,
    TrustedContact,
}

/// §62, verbatim.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryStatus {
    NotConfigured,
    Configured,
    NeedsReview,
    Invalid,
}

/// §63: "Recovery configured / Last verified: 3 months ago" or
/// "No recovery method configured / Set up recovery." `last_verified_millis`
/// being `None` is exactly the second case — a component checks that
/// directly rather than this type needing a separate "has recovery"
/// bool that could disagree with `status`/`configured_methods`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryOverview {
    pub account: AccountId,
    pub status: RecoveryStatus,
    pub last_verified_millis: Option<u64>,
    pub configured_methods: Vec<RecoveryMethod>,
}

/// §65-66, §88, §160-161: the recovery-key display screen's own
/// state. §65: "sensitive screen, require reauth." §66: "never
/// auto-copied, explicit user action only." §88: "after copying,
/// offer Clear Clipboard... do not silently monitor clipboard
/// afterward" — this type tracks *that a copy happened* (so a caller
/// can offer the clear-clipboard prompt) and nothing more; it holds no
/// clipboard contents and runs no clipboard polling of its own.
/// §160-161: Show/Hide/Copy are three separate controls, and the key
/// "may initially be obscured after creation until user taps Show"
/// even once reauthenticated — `reauthenticated` and `visible` are
/// therefore two independent flags: passing reauth makes revealing
/// *possible* (`can_reveal()`), it doesn't make the key visible by
/// itself. `hide()` is deliberately reauth-free — toggling visibility
/// back off within an already-reauthenticated screen session is cheap
/// UI state, not a security boundary crossing, so it doesn't reset
/// `reauthenticated`.
#[derive(Debug, Default)]
pub struct RecoveryKeyDisplayState {
    reauthenticated: bool,
    visible: bool,
    copied: bool,
}

impl RecoveryKeyDisplayState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn mark_reauthenticated(&mut self) {
        self.reauthenticated = true;
    }

    /// §65: the key itself must not render before this is true — a
    /// caller checks `can_reveal()` before drawing the key, not after.
    pub fn can_reveal(&self) -> bool {
        self.reauthenticated
    }

    /// §160's "Show" control. No-op (stays hidden) if reauth hasn't
    /// happened yet — a caller wiring a Show button should also check
    /// `can_reveal()` to decide whether to show a reauth prompt
    /// instead, but this method itself refuses to reveal without it
    /// either way, so a UI bug that skips that check can't leak the
    /// key.
    pub fn show(&mut self) {
        if self.can_reveal() {
            self.visible = true;
        }
    }

    /// §160's "Hide" control. Always allowed — re-obscuring the key is
    /// never a security-relevant action that needs gating, only
    /// revealing it is.
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// §161: "may initially be obscured... until user taps Show" — a
    /// caller renders the obscured/placeholder state whenever this is
    /// `false`, regardless of `can_reveal()`.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// §66: called only from an explicit user "Copy" action — never
    /// from a render path. Recording this is what lets a caller know
    /// to offer §88's "Clear Clipboard" affordance.
    pub fn mark_copied(&mut self) {
        self.copied = true;
    }

    pub fn should_offer_clear_clipboard(&self) -> bool {
        self.copied
    }
}

/// §71's own six-step flow, plus a terminal `Done` — same pattern as
/// `device_lifecycle.rs`'s `DeviceLossFlowStep`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryRotationStep {
    Reauthenticate,
    GenerateNewMaterial,
    ShowAndSave,
    Verify,
    Activate,
    InvalidateOld,
    Done,
}

impl RecoveryRotationStep {
    const ORDER: [Self; 7] = [
        Self::Reauthenticate,
        Self::GenerateNewMaterial,
        Self::ShowAndSave,
        Self::Verify,
        Self::Activate,
        Self::InvalidateOld,
        Self::Done,
    ];

    fn index(self) -> usize {
        Self::ORDER
            .iter()
            .position(|s| *s == self)
            .expect("all variants are in ORDER")
    }
}

/// §71-73: a recovery-material rotation in progress. The one property
/// this type exists to guarantee: **the old method cannot be reported
/// invalid before the new one has been verified and activated.**
/// `advance()` enforces §71's exact step order (no skipping ahead),
/// and `old_method_still_valid()` — the answer a caller must check
/// before ever telling the backend "the old recovery key/passphrase no
/// longer works" — only flips to `false` once `InvalidateOld` has
/// actually completed, never earlier. §73's failure mode ("old key
/// invalid, new key not saved") is exactly what this ordering makes
/// structurally unreachable: there is no path from `Reauthenticate` to
/// `old_method_still_valid() == false` that doesn't pass through
/// `Verify` and `Activate` first.
#[derive(Debug)]
pub struct RecoveryRotationFlow {
    step: RecoveryRotationStep,
}

impl RecoveryRotationFlow {
    pub fn start() -> Self {
        Self {
            step: RecoveryRotationStep::Reauthenticate,
        }
    }

    pub fn step(&self) -> RecoveryRotationStep {
        self.step
    }

    /// Advances exactly one step in §71's fixed order. There is no
    /// "jump to `InvalidateOld`" method anywhere on this type — the
    /// only way to reach it is by calling `advance()` five times from
    /// the start, each of which requires having actually been at the
    /// preceding step.
    pub fn advance(&mut self) {
        let next_index = self.step.index() + 1;
        if let Some(next) = RecoveryRotationStep::ORDER.get(next_index) {
            self.step = *next;
        }
    }

    /// §72: "old recovery must remain valid until new method is fully
    /// confirmed" — true for every step up through `Activate`, false
    /// only once `InvalidateOld` (or `Done`, which is unreachable
    /// without having passed through it) has been reached.
    pub fn old_method_still_valid(&self) -> bool {
        self.step.index() < RecoveryRotationStep::InvalidateOld.index()
    }

    pub fn is_done(&self) -> bool {
        self.step == RecoveryRotationStep::Done
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overview_with_no_last_verified_reads_as_not_yet_configured_or_stale() {
        let overview = RecoveryOverview {
            account: AccountId::new(),
            status: RecoveryStatus::NotConfigured,
            last_verified_millis: None,
            configured_methods: Vec::new(),
        };
        assert!(overview.last_verified_millis.is_none());
        assert!(overview.configured_methods.is_empty());
    }

    #[test]
    fn recovery_key_cannot_reveal_before_reauth() {
        let state = RecoveryKeyDisplayState::new();
        assert!(!state.can_reveal());
    }

    #[test]
    fn recovery_key_reveals_after_reauth() {
        let mut state = RecoveryKeyDisplayState::new();
        state.mark_reauthenticated();
        assert!(state.can_reveal());
    }

    #[test]
    fn recovery_key_stays_hidden_by_default_even_after_reauth() {
        // §161: "may initially be obscured after creation until user
        // taps Show" — reauth alone must not make it visible.
        let mut state = RecoveryKeyDisplayState::new();
        state.mark_reauthenticated();
        assert!(state.can_reveal());
        assert!(!state.is_visible());
    }

    #[test]
    fn show_reveals_only_after_reauth() {
        let mut state = RecoveryKeyDisplayState::new();
        state.show();
        assert!(
            !state.is_visible(),
            "show() before reauth must not reveal the key"
        );

        state.mark_reauthenticated();
        state.show();
        assert!(state.is_visible());
    }

    #[test]
    fn hide_works_without_needing_reauth_again() {
        let mut state = RecoveryKeyDisplayState::new();
        state.mark_reauthenticated();
        state.show();
        assert!(state.is_visible());

        state.hide();
        assert!(!state.is_visible());

        // Showing again within the same reauthenticated session
        // doesn't require a fresh reauth.
        state.show();
        assert!(state.is_visible());
    }

    #[test]
    fn clear_clipboard_only_offered_after_an_explicit_copy() {
        let mut state = RecoveryKeyDisplayState::new();
        assert!(!state.should_offer_clear_clipboard());
        state.mark_copied();
        assert!(state.should_offer_clear_clipboard());
    }

    #[test]
    fn old_method_stays_valid_through_verify_and_activate() {
        let mut flow = RecoveryRotationFlow::start();
        assert!(flow.old_method_still_valid());

        flow.advance(); // GenerateNewMaterial
        assert!(flow.old_method_still_valid());
        flow.advance(); // ShowAndSave
        assert!(flow.old_method_still_valid());
        flow.advance(); // Verify
        assert!(flow.old_method_still_valid());
        flow.advance(); // Activate
        assert!(flow.old_method_still_valid());
    }

    #[test]
    fn old_method_becomes_invalid_only_after_invalidate_old_step() {
        let mut flow = RecoveryRotationFlow::start();
        for _ in 0..5 {
            flow.advance();
        }
        assert_eq!(flow.step(), RecoveryRotationStep::InvalidateOld);
        assert!(!flow.old_method_still_valid());
    }

    #[test]
    fn flow_reaches_done_after_all_six_advances() {
        let mut flow = RecoveryRotationFlow::start();
        for _ in 0..6 {
            flow.advance();
        }
        assert!(flow.is_done());
        assert!(!flow.old_method_still_valid());
    }

    #[test]
    fn advancing_past_done_is_a_harmless_no_op() {
        let mut flow = RecoveryRotationFlow::start();
        for _ in 0..10 {
            flow.advance();
        }
        assert!(flow.is_done());
    }
}
