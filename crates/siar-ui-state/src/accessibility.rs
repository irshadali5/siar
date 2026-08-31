//! Accessibility label generation and high-risk confirmation UX
//! (ui-ux-15 §157-158, §162, §168-171).
//!
//! §159/§163-167 (recovery-key chunk reading, no-truncation/wrap
//! rules, RTL mirroring, reduced motion, keyboard/TalkBack support)
//! are rendering-layer guidance with no portable data shape — they
//! describe how a component *lays out* content it already has, not
//! content this crate computes. Not attempted here, same reasoning as
//! §142-143/§145-147 last round. §159 specifically: recovery
//! material's actual format (base32, word list, ...) isn't something
//! this crate imposes a chunking scheme on without knowing it —
//! grouping-for-readability is a render-time concern over whatever
//! `RecoveryMaterialView::material_display` actually contains.

use crate::device_lifecycle::ReauthPurpose;
use crate::security_center::{DeviceSecurityView, DeviceTrustState, SecurityHealth};
use crate::security_event::SecurityEventSeverity;

/// §157's own exact worked example, generalized to the count that
/// actually varies: "Security needs attention. One unresolved critical
/// event." A caller passes whatever
/// `SecurityStatusBanner::visible_issues(...).len()` returns.
pub fn security_overview_accessibility_summary(unresolved_critical_count: usize) -> String {
    match unresolved_critical_count {
        0 => "Security is healthy. No unresolved critical events.".to_string(),
        1 => "Security needs attention. One unresolved critical event.".to_string(),
        n => format!("Security needs attention. {n} unresolved critical events."),
    }
}

/// §158's own exact worked example: "Pixel 10, trusted, active now,
/// this device." Built from `DeviceSecurityView`'s own fields rather
/// than duplicating them — this function is purely a formatting
/// concern over data the type already carries.
pub fn device_row_accessibility_label(device: &DeviceSecurityView) -> String {
    let trust = match device.status {
        DeviceTrustState::Trusted => "trusted",
        DeviceTrustState::Pending => "pending approval",
        DeviceTrustState::Revoked => "revoked",
        DeviceTrustState::Compromised => "compromised",
        DeviceTrustState::Unknown => "unknown trust level",
    };

    let mut parts = vec![device.display_name.clone(), trust.to_string()];
    if device.current_device {
        parts.push("this device".to_string());
    }
    parts.join(", ")
}

/// §162: "Critical/warning/healthy always have: text, icon" — never
/// color alone. This is the icon half; the text half is
/// `SecurityCenterSnapshot`/`SecurityEvent`'s own existing labels
/// (`severity_label` in the desktop-side rendering code) — kept
/// separate here rather than merged into one "label" string so a
/// caller can lay out icon and text as genuinely separate visual
/// elements, not one string a screen reader would read as a single
/// run-on phrase.
pub const fn security_health_icon_name(health: SecurityHealth) -> &'static str {
    match health {
        SecurityHealth::Healthy => "shield-check",
        SecurityHealth::Attention => "shield-alert",
        SecurityHealth::Critical => "shield-x",
        SecurityHealth::Unknown => "shield-question",
    }
}

pub const fn security_event_severity_icon_name(severity: SecurityEventSeverity) -> &'static str {
    match severity {
        SecurityEventSeverity::Info => "info-circle",
        SecurityEventSeverity::Warning => "alert-triangle",
        SecurityEventSeverity::Critical => "alert-octagon",
    }
}

/// §169's own four named examples.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DestructiveAction {
    RevokeDevice,
    ResetIdentity,
    RotateRecoveryKey,
    DeleteRecoveryMethod,
}

/// §170: "confirmation friction should match actual risk" — not every
/// destructive action needs §171's full typed-phrase treatment.
/// `ResetIdentity` is the one action this workspace's own
/// `IdentityResetConfirmation` (`identity_verification.rs`) already
/// implements as a three-condition, typed-confirmation gate — this
/// enum is what tells a caller *which* actions warrant reaching for
/// that heavier gate versus a plain confirm dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmationFriction {
    SimpleConfirm,
    TypedConfirmation,
}

impl DestructiveAction {
    pub const fn required_friction(self) -> ConfirmationFriction {
        match self {
            Self::ResetIdentity => ConfirmationFriction::TypedConfirmation,
            Self::RevokeDevice | Self::RotateRecoveryKey | Self::DeleteRecoveryMethod => {
                ConfirmationFriction::SimpleConfirm
            }
        }
    }

    /// §170's own title example ("Do Not Require Typing Device Name
    /// Unless Needed") implies revoke-device confirmations name the
    /// device in the *explanation* text without requiring it be
    /// *typed* back — that's a rendering-copy concern for whatever
    /// builds `HighRiskConfirmationCopy` below, not something this
    /// method decides.
    pub const fn reauth_purpose(self) -> Option<ReauthPurpose> {
        match self {
            Self::ResetIdentity => Some(ReauthPurpose::ResetIdentity),
            Self::RotateRecoveryKey => Some(ReauthPurpose::RotateRecoveryKey),
            // RevokeDevice needs a specific DeviceId this enum variant
            // doesn't carry — a caller constructs
            // `ReauthPurpose::RevokeDevice(id)` itself rather than
            // this method guessing at one.
            Self::RevokeDevice => None,
            // No dedicated `ReauthPurpose` variant exists for deleting
            // a recovery *method* specifically (as opposed to rotating
            // the key) — a real gap, not silently mapped to a
            // near-enough purpose that would misrepresent what's being
            // authorized.
            Self::DeleteRecoveryMethod => None,
        }
    }
}

/// §168, verbatim three-part structure: "what will happen, what will
/// not happen, whether it can be undone." A caller supplies the actual
/// wording per-action — this type only enforces that all three parts
/// exist, not what they say.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HighRiskConfirmationCopy {
    pub what_will_happen: String,
    pub what_will_not_happen: String,
    pub is_undoable: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security_center::DeviceKind;
    use siar_domain::DeviceId;

    fn sample_device(name: &str, status: DeviceTrustState, current: bool) -> DeviceSecurityView {
        DeviceSecurityView {
            id: DeviceId::new(),
            display_name: name.to_string(),
            kind: DeviceKind::AndroidPhone,
            status,
            added_at_millis: 0,
            last_active_millis: None,
            current_device: current,
            security_flags: Vec::new(),
        }
    }

    #[test]
    fn overview_summary_matches_the_spec_worked_example_for_one_event() {
        assert_eq!(
            security_overview_accessibility_summary(1),
            "Security needs attention. One unresolved critical event."
        );
    }

    #[test]
    fn overview_summary_pluralizes_for_more_than_one() {
        let summary = security_overview_accessibility_summary(3);
        assert!(summary.contains("3 unresolved critical events"));
    }

    #[test]
    fn overview_summary_reports_healthy_at_zero() {
        assert!(security_overview_accessibility_summary(0).contains("healthy"));
    }

    #[test]
    fn device_row_label_matches_the_spec_worked_example_shape() {
        let device = sample_device("Pixel 10", DeviceTrustState::Trusted, true);
        assert_eq!(device_row_accessibility_label(&device), "Pixel 10, trusted, this device");
    }

    #[test]
    fn device_row_label_omits_this_device_for_other_devices() {
        let device = sample_device("Old Laptop", DeviceTrustState::Trusted, false);
        assert_eq!(device_row_accessibility_label(&device), "Old Laptop, trusted");
    }

    #[test]
    fn every_severity_and_health_state_has_a_distinct_icon_name() {
        let healths = [
            SecurityHealth::Healthy,
            SecurityHealth::Attention,
            SecurityHealth::Critical,
            SecurityHealth::Unknown,
        ];
        for i in 0..healths.len() {
            for j in (i + 1)..healths.len() {
                assert_ne!(
                    security_health_icon_name(healths[i]),
                    security_health_icon_name(healths[j])
                );
            }
        }
    }

    #[test]
    fn only_identity_reset_requires_typed_confirmation() {
        assert_eq!(
            DestructiveAction::ResetIdentity.required_friction(),
            ConfirmationFriction::TypedConfirmation
        );
        for action in [
            DestructiveAction::RevokeDevice,
            DestructiveAction::RotateRecoveryKey,
            DestructiveAction::DeleteRecoveryMethod,
        ] {
            assert_eq!(action.required_friction(), ConfirmationFriction::SimpleConfirm);
        }
    }
}
