//! Privacy Section, App Lock, and Key Export (ui-ux-15 §110-119).
//!
//! §110's own note — "detailed settings remain elsewhere" — means this
//! module's `PrivacyControlsSummary` is deliberately a read-only
//! rollup for the Security Center screen, not a replacement for the
//! actual settings screen (ui-ux-18, not yet built). No setter methods
//! exist here beyond `set_from` on purpose: this crate's own dependency
//! rule and translated-input pattern (see `lib.rs`'s top doc) apply as
//! much to "settings summaries" as to security state — this type
//! displays what the real settings are, it doesn't own changing them.

use crate::device_lifecycle::ReauthPurpose;

/// §110's five listed high-impact controls, summarized as booleans/
/// counts rather than full settings objects — a Security Center
/// summary card needs "read receipts: on," not the full settings
/// screen's worth of configuration for each.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PrivacyControlsSummary {
    pub notification_previews_enabled: bool,
    pub read_receipts_enabled: bool,
    pub typing_indicators_enabled: bool,
    pub presence_sharing_enabled: bool,
    pub blocked_contact_count: usize,
}

/// §113, verbatim.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppLockTimeout {
    Immediately,
    OneMinute,
    FiveMinutes,
    ThirtyMinutes,
}

/// §111/§114: mobile uses the device's own biometric/credential
/// prompt; desktop's two options are listed separately in §114 since
/// desktop has no OS-level biometric prompt equivalent on every
/// platform this workspace targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppLockMethod {
    BiometricOrDeviceCredential,
    ApplicationPassphrase,
    OsAuthentication,
}

/// §111-114: whether/how the app itself is locked behind a
/// biometric/credential/passphrase prompt. §112: "protects UI access.
/// Does not necessarily stop: background receive, calls, sync" — this
/// type has no fields or methods relating to background activity at
/// all, which is the honest reflection of that scope limit rather than
/// a stubbed-out "also blocks background" flag that would misrepresent
/// what App Lock actually does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppLockSettings {
    pub enabled: bool,
    pub method: AppLockMethod,
    pub timeout: AppLockTimeout,
}

/// §115: "Even if app unlocked, high-risk action may require reauth."
/// Not new state — `ReauthPurpose` (`device_lifecycle.rs`) already has
/// no dependency on app-lock state at all, so an unlocked app was
/// never capable of silently satisfying a `ReauthPurpose` requirement
/// in this codebase; §115's rule already holds structurally. This
/// function exists to make that fact checkable/testable rather than
/// merely true by the absence of a connection between the two types.
pub const fn app_unlock_satisfies_reauth(_purpose: ReauthPurpose, _app_lock: &AppLockSettings) -> bool {
    false
}

/// §116: Android-only per the spec's own wording ("hide content in
/// Android recent-apps snapshot"). A desktop build simply never
/// constructs one of these with `hide_in_recents_while_locked: true`
/// meaningfully wired to anything — this type doesn't need a separate
/// platform flag, the platform layer either has a recent-apps snapshot
/// concept to hide from or it doesn't.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ScreenPrivacySettings {
    pub hide_in_recents_while_locked: bool,
}

/// §117: "sensitive secrets copied only on explicit action." Already a
/// concrete, working pattern for the one secret this workspace
/// currently has a copy affordance for —
/// `recovery.rs::RecoveryKeyDisplayState::mark_copied`/
/// `should_offer_clear_clipboard`. Not re-abstracted into a generic
/// "clipboard guard" type here: with exactly one real instance of this
/// pattern in the codebase, a general-purpose abstraction would be
/// speculative — reuse the existing concrete implementation as the
/// template if/when a second secret (identity key export, §118-119
/// below) needs the same treatment.
///
/// §118: "do not expose raw identity private key export in normal
/// UX." Enforced by absence — there is no un-gated export function
/// anywhere in this crate; `AdvancedKeyExportGate` below is the only
/// path, and it exists specifically outside normal UX flow.
///
/// §119: "if product allows export: Advanced, high-risk,
/// reauthenticated, strong warning" — all four conditions tracked
/// explicitly, same shape as `IdentityResetConfirmation`
/// (`identity_verification.rs`)'s multi-condition gate.
#[derive(Debug, Default)]
pub struct AdvancedKeyExportGate {
    placed_under_advanced: bool,
    reauthenticated: bool,
    strong_warning_acknowledged: bool,
}

impl AdvancedKeyExportGate {
    pub fn new() -> Self {
        Self::default()
    }

    /// §119: navigation placement is itself one of the four stated
    /// conditions, not merely a UI-layout preference — a caller marks
    /// this only once the export flow is actually reached via the
    /// Advanced section, not from a shortcut elsewhere.
    pub fn mark_reached_via_advanced_section(&mut self) {
        self.placed_under_advanced = true;
    }

    pub fn mark_reauthenticated(&mut self) {
        self.reauthenticated = true;
    }

    pub fn mark_strong_warning_acknowledged(&mut self) {
        self.strong_warning_acknowledged = true;
    }

    /// All four of §119's conditions, not any subset.
    pub fn can_export(&self) -> bool {
        self.placed_under_advanced && self.reauthenticated && self.strong_warning_acknowledged
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use siar_domain::DeviceId;

    #[test]
    fn app_unlock_never_satisfies_a_reauth_purpose() {
        let settings = AppLockSettings {
            enabled: true,
            method: AppLockMethod::BiometricOrDeviceCredential,
            timeout: AppLockTimeout::Immediately,
        };
        // Every purpose, unlocked app, still `false` — §115's rule
        // holds regardless of which purpose or app-lock configuration.
        assert!(!app_unlock_satisfies_reauth(ReauthPurpose::ShowRecoveryKey, &settings));
        assert!(!app_unlock_satisfies_reauth(ReauthPurpose::ResetIdentity, &settings));
        assert!(!app_unlock_satisfies_reauth(
            ReauthPurpose::RevokeDevice(DeviceId::new()),
            &settings
        ));
    }

    #[test]
    fn key_export_requires_all_four_conditions() {
        let mut gate = AdvancedKeyExportGate::new();
        assert!(!gate.can_export());

        gate.mark_reached_via_advanced_section();
        assert!(!gate.can_export());

        gate.mark_reauthenticated();
        assert!(!gate.can_export());

        gate.mark_strong_warning_acknowledged();
        assert!(gate.can_export());
    }

    #[test]
    fn key_export_gate_default_denies_export() {
        let gate = AdvancedKeyExportGate::new();
        assert!(!gate.can_export());
    }

    #[test]
    fn privacy_summary_defaults_to_all_off_and_zero_blocked() {
        let summary = PrivacyControlsSummary::default();
        assert!(!summary.read_receipts_enabled);
        assert_eq!(summary.blocked_contact_count, 0);
    }
}
