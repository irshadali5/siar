//! Reauth challenge, the security UI event stream, sensitive UI
//! effects, and recovery-material handling (ui-ux-15 §141, §144,
//! §148-156).
//!
//! §142-143 (the actual Android `BiometricPrompt`/desktop credential-
//! store adapters) and §145-147 (ViewModel/Presenter ownership split,
//! "no security decisions in UI") are platform/architecture guidance
//! with nothing to encode as a portable Rust type — they describe how
//! `apps/android`/`apps/desktop` should be *structured*, not a data
//! shape this crate can represent. Not attempted here, same reasoning
//! as skipping §142-143's own two adapter sections in the last round's
//! `ReauthProof` doc comment.

use siar_domain::DeviceId;
use zeroize::Zeroize;

use crate::device_lifecycle::ReauthPurpose;
use crate::presentation_api::RecoveryMaterialView;
use crate::recovery::{RecoveryMethod, RecoveryOverview};
use crate::recovery_advanced::BackupSecurityState;
use crate::security_center::{DeviceSecurityView, SecurityHealth};
use crate::security_event::SecurityEvent;

/// Not one of §141's fields — a locally-assigned identifier for one
/// outstanding challenge, same reasoning and shape as
/// `SecurityEventId` (`security_event.rs`): only needs to be unique
/// within whatever's tracking outstanding challenges, not globally.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReauthChallengeId(pub u64);

/// §141, verbatim. "Platform returns a short-lived proof/token" is
/// `ReauthProof` (`presentation_api.rs`, already built last round) —
/// this is the request half, that was the response half.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReauthChallenge {
    pub id: ReauthChallengeId,
    pub purpose: ReauthPurpose,
}

/// §144, verbatim — the event stream a real Rust "security truth"
/// layer emits to drive UI updates. `SecurityEventView`/
/// `RecoveryStatusView` in the spec's own sketch are, as in
/// `presentation_api.rs`, this crate's existing `SecurityEvent`/
/// `RecoveryOverview` — reused, not duplicated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecurityUiEvent {
    HealthChanged(SecurityHealth),
    DeviceChanged(DeviceSecurityView),
    DeviceRemoved(DeviceId),
    EventAdded(SecurityEvent),
    EventUpdated(SecurityEvent),
    RecoveryChanged(RecoveryOverview),
    BackupSecurityChanged(BackupSecurityState),
}

/// §148's own four named examples. "Examples:" in the spec's own
/// wording, not an exhaustive list — same honesty as
/// `DeviceSecurityFlag`/`UiError` elsewhere in this crate: these four,
/// not a guess at every effect a real implementation might eventually
/// need.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SensitiveUiEffect {
    RequestReauth(ReauthChallenge),
    EnableSecureWindow,
    CopySensitiveSecret,
    OpenSystemSecuritySettings,
}

/// §150's own four release triggers, verbatim. All four cause the same
/// release behavior (`SensitiveRecoveryMaterialHandle::release`
/// doesn't branch on which reason fired) — the spec doesn't ask for
/// different handling per trigger, only that release happens for any
/// of them; this exists so a caller can record *why* release
/// happened (useful for the kind of diagnostic logging §133-134's
/// Diagnostics view might show) without this type needing to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReleaseReason {
    ScreenExit,
    Timeout,
    AppLock,
    Background,
}

/// §149-152: "avoid exposing raw secret broadly, use a dedicated
/// short-lived sensitive model... release/clear after: screen exit,
/// timeout, app lock, background." This is that dedicated model — the
/// scoped, drop-early wrapper §151/§152 ask both Compose and Dioxus to
/// use in place of holding `RecoveryMaterialView` in an ordinary
/// long-lived `StateFlow`/`Signal`. `release()` doesn't just drop the
/// reference — it actively zeroizes the material's text in place
/// first, since a `String`'s heap buffer isn't guaranteed to be
/// cleared by an ordinary drop.
#[derive(Debug)]
pub struct SensitiveRecoveryMaterialHandle {
    material: Option<RecoveryMaterialView>,
}

impl SensitiveRecoveryMaterialHandle {
    pub fn new(material: RecoveryMaterialView) -> Self {
        Self {
            material: Some(material),
        }
    }

    /// `None` once released — a caller checks this rather than the
    /// material silently becoming stale/wrong to display.
    pub fn peek(&self) -> Option<&RecoveryMaterialView> {
        self.material.as_ref()
    }

    pub fn release(&mut self, _reason: ReleaseReason) {
        if let Some(mut material) = self.material.take() {
            material.material_display.zeroize();
        }
    }

    pub fn is_released(&self) -> bool {
        self.material.is_none()
    }
}

/// §153: "if platform supports controlled clipboard clearing, offer
/// clear after 1 minute — do not promise guaranteed deletion from
/// external clipboard managers." The 60-second threshold, made
/// checkable against a caller-supplied current time rather than a
/// wall-clock read inside this crate (same convention as every other
/// timestamped type here).
pub const CLIPBOARD_CLEAR_OFFER_DELAY_MILLIS: u64 = 60_000;

pub fn should_offer_clipboard_clear(copied_at_millis: u64, now_millis: u64) -> bool {
    now_millis.saturating_sub(copied_at_millis) >= CLIPBOARD_CLEAR_OFFER_DELAY_MILLIS
}

/// §154, verbatim two options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryExportFormat {
    EncryptedFile,
    PrintedText,
}

impl RecoveryExportFormat {
    /// §155: "Desktop optional. Warn about physical security." Only
    /// the printed-paper path carries a physical-security risk — an
    /// encrypted file has its own risk profile (where it's saved,
    /// whether the disk is encrypted) but isn't "someone can just pick
    /// this up off the printer tray."
    pub const fn requires_physical_security_warning(self) -> bool {
        matches!(self, Self::PrintedText)
    }
}

/// §156: "never include real recovery material in screenshot tests,
/// use fake fixtures." A ready-made, obviously-fake fixture —
/// `"REAL_RECOVERY_MATERIAL"`-shaped enough to visually stand in for a
/// real key in a rendered screenshot, but textually unmistakable as a
/// fixture to anyone reading the raw string (a test author reaching
/// for a plausible-looking-but-real string instead of this function is
/// exactly the mistake §156 warns against).
pub fn fake_recovery_material_for_screenshot_tests() -> RecoveryMaterialView {
    RecoveryMaterialView {
        method: RecoveryMethod::RecoveryKey,
        material_display: "TEST-FIXTURE-NOT-REAL-0000-0000".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_clears_the_material_and_zeroizes_the_string() {
        let mut handle = SensitiveRecoveryMaterialHandle::new(RecoveryMaterialView {
            method: RecoveryMethod::RecoveryKey,
            material_display: "AAAA-BBBB-CCCC-DDDD".to_string(),
        });
        assert!(handle.peek().is_some());

        handle.release(ReleaseReason::ScreenExit);
        assert!(handle.peek().is_none());
        assert!(handle.is_released());
    }

    #[test]
    fn releasing_twice_is_a_harmless_no_op() {
        let mut handle = SensitiveRecoveryMaterialHandle::new(RecoveryMaterialView {
            method: RecoveryMethod::RecoveryKey,
            material_display: "AAAA-BBBB-CCCC-DDDD".to_string(),
        });
        handle.release(ReleaseReason::Timeout);
        handle.release(ReleaseReason::AppLock);
        assert!(handle.is_released());
    }

    #[test]
    fn clipboard_clear_is_not_offered_before_the_threshold() {
        assert!(!should_offer_clipboard_clear(0, 59_999));
    }

    #[test]
    fn clipboard_clear_is_offered_at_exactly_one_minute() {
        assert!(should_offer_clipboard_clear(0, 60_000));
    }

    #[test]
    fn printed_text_requires_the_physical_security_warning_but_encrypted_file_does_not() {
        assert!(RecoveryExportFormat::PrintedText.requires_physical_security_warning());
        assert!(!RecoveryExportFormat::EncryptedFile.requires_physical_security_warning());
    }

    #[test]
    fn screenshot_fixture_is_textually_distinguishable_from_a_real_key() {
        let fixture = fake_recovery_material_for_screenshot_tests();
        assert!(fixture.material_display.contains("TEST-FIXTURE-NOT-REAL"));
    }
}
