//! Empty states, single-device escalation, and offline-labeling
//! restraint (ui-ux-15 §172-178).

use crate::accessibility::HighRiskConfirmationCopy;
use crate::recovery::RecoveryStatus;
use crate::security_center::{DeviceSecurityView, SecurityHealth};

/// §172, verbatim.
pub const SECURITY_EVENT_LIST_EMPTY_LABEL: &str = "No recent security events";

/// §173, verbatim.
pub const RECOVERY_NOT_CONFIGURED_CTA: &str = "Set Up Recovery";

/// §174, verbatim two-line copy. Links to Part 16 (ui-ux-16, still not
/// built) — this crate has nothing to link *to* yet, so this is just
/// the label pair a caller renders once that destination exists.
pub const BACKUP_MISSING_LABEL: &str = "No verified backup";
pub const BACKUP_MISSING_CTA: &str = "Create Backup";

/// §175: "if only one device: '1 trusted device,' and recovery
/// recommendation becomes more important." The label half, matching
/// §157/§158's singular-vs-plural pattern already established.
pub fn trusted_device_count_label(count: usize) -> String {
    match count {
        1 => "1 trusted device".to_string(),
        n => format!("{n} trusted devices"),
    }
}

/// §175's other half: recovery becomes *more important*, not merely
/// mentioned, when there's only one device — a single-device account
/// with no recovery configured has no fallback at all if that one
/// device is lost. This is a real escalation, not just display copy:
/// `SecurityHealth::Healthy` is not an honest read of that situation,
/// so it's bumped to `Attention`. Only escalates `Healthy` — never
/// downgrades an already-worse health state, and never touches
/// anything except this specific single-device-plus-no-recovery
/// combination.
pub fn effective_security_health(
    base_health: SecurityHealth,
    trusted_device_count: usize,
    recovery_status: RecoveryStatus,
) -> SecurityHealth {
    let single_device_no_recovery =
        trusted_device_count <= 1 && recovery_status == RecoveryStatus::NotConfigured;

    if single_device_no_recovery && base_health == SecurityHealth::Healthy {
        SecurityHealth::Attention
    } else {
        base_health
    }
}

/// §176: "last active may be stale. Do not label 'offline' unless
/// proven." There is deliberately no `Offline` variant anywhere in
/// this enum — this crate has no proof-of-disconnection signal (that
/// would come from a transport/connectivity layer this crate doesn't
/// depend on), so the vocabulary itself can't claim more than it
/// knows. `PossiblyStale` is the honest ceiling: "last active" is old
/// enough to be worth flagging, without asserting the device is
/// actually offline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceActivityDisplay {
    Unknown,
    RecentlyActive,
    PossiblyStale,
}

/// A caller-chosen staleness threshold rather than a constant baked in
/// here — how "stale" is worth flagging is a product decision this
/// crate shouldn't presume, the same reasoning as every other
/// caller-supplied timing threshold in this crate
/// (`should_offer_clipboard_clear`'s delay is at least a named
/// constant since the spec gives an exact number; this one doesn't).
pub fn device_activity_display(
    last_active_millis: Option<u64>,
    now_millis: u64,
    stale_after_millis: u64,
) -> DeviceActivityDisplay {
    match last_active_millis {
        None => DeviceActivityDisplay::Unknown,
        Some(last_active) if now_millis.saturating_sub(last_active) >= stale_after_millis => {
            DeviceActivityDisplay::PossiblyStale
        }
        Some(_) => DeviceActivityDisplay::RecentlyActive,
    }
}

/// §177: "if metadata incomplete: 'Unknown device' with DeviceId
/// detail." A blank/whitespace-only `display_name` is treated as
/// incomplete metadata — the fallback still includes the device's
/// short ID (`DeviceId::fmt_short`, already used elsewhere in this
/// crate) so the row remains distinguishable from any other unknown
/// device, matching "with DeviceId detail" rather than a single,
/// indistinguishable "Unknown device" label for every such row.
pub fn device_display_name(device: &DeviceSecurityView) -> String {
    if device.display_name.trim().is_empty() {
        format!("Unknown device ({})", device.id.fmt_short())
    } else {
        device.display_name.clone()
    }
}

/// §178's own exact worked example, parameterized only on the device
/// name — the consequence text is fixed, matching the spec's literal
/// wording rather than a templated "may lose access" softened
/// version.
pub fn lost_device_revocation_confirmation(device_name: &str) -> HighRiskConfirmationCopy {
    HighRiskConfirmationCopy {
        what_will_happen: format!(
            "Revoke {device_name}? It will no longer be able to access your account or receive future messages."
        ),
        what_will_not_happen: "Your other trusted devices keep full access.".to_string(),
        is_undoable: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security_center::DeviceKind;
    use siar_domain::DeviceId;

    fn sample_device(display_name: &str) -> DeviceSecurityView {
        DeviceSecurityView {
            id: DeviceId::new(),
            display_name: display_name.to_string(),
            kind: DeviceKind::Desktop,
            status: crate::security_center::DeviceTrustState::Trusted,
            added_at_millis: 0,
            last_active_millis: None,
            current_device: false,
            security_flags: Vec::new(),
        }
    }

    #[test]
    fn trusted_device_count_label_matches_the_spec_worked_example() {
        assert_eq!(trusted_device_count_label(1), "1 trusted device");
        assert_eq!(trusted_device_count_label(3), "3 trusted devices");
    }

    #[test]
    fn single_device_with_no_recovery_escalates_healthy_to_attention() {
        let health = effective_security_health(SecurityHealth::Healthy, 1, RecoveryStatus::NotConfigured);
        assert_eq!(health, SecurityHealth::Attention);
    }

    #[test]
    fn single_device_with_recovery_configured_stays_healthy() {
        let health = effective_security_health(SecurityHealth::Healthy, 1, RecoveryStatus::Configured);
        assert_eq!(health, SecurityHealth::Healthy);
    }

    #[test]
    fn multiple_devices_with_no_recovery_does_not_trigger_the_escalation() {
        let health = effective_security_health(SecurityHealth::Healthy, 3, RecoveryStatus::NotConfigured);
        assert_eq!(health, SecurityHealth::Healthy);
    }

    #[test]
    fn escalation_never_downgrades_an_already_worse_health_state() {
        let health = effective_security_health(SecurityHealth::Critical, 1, RecoveryStatus::NotConfigured);
        assert_eq!(health, SecurityHealth::Critical);
    }

    #[test]
    fn device_activity_display_never_claims_offline() {
        // Structural check as much as a behavioral one: there is no
        // `DeviceActivityDisplay::Offline` variant to even construct.
        let stale = device_activity_display(Some(0), 1_000_000, 60_000);
        assert_eq!(stale, DeviceActivityDisplay::PossiblyStale);
    }

    #[test]
    fn recently_active_device_is_not_flagged_stale() {
        let recent = device_activity_display(Some(999_000), 1_000_000, 60_000);
        assert_eq!(recent, DeviceActivityDisplay::RecentlyActive);
    }

    #[test]
    fn missing_last_active_is_unknown_not_stale() {
        assert_eq!(device_activity_display(None, 1_000_000, 60_000), DeviceActivityDisplay::Unknown);
    }

    #[test]
    fn blank_display_name_falls_back_to_unknown_device_with_id() {
        let device = sample_device("   ");
        let label = device_display_name(&device);
        assert!(label.starts_with("Unknown device ("));
    }

    #[test]
    fn a_real_display_name_is_used_as_is() {
        let device = sample_device("Pixel 10");
        assert_eq!(device_display_name(&device), "Pixel 10");
    }

    #[test]
    fn lost_device_confirmation_matches_the_spec_worked_example() {
        let copy = lost_device_revocation_confirmation("Pixel 10");
        assert_eq!(
            copy.what_will_happen,
            "Revoke Pixel 10? It will no longer be able to access your account or receive future messages."
        );
        assert!(!copy.is_undoable);
    }
}
