//! Security Center — Devices screen (ui-ux-15 §14-17).
//!
//! ⚠️ Same caveat as `security_events.rs`: NOT compiled or tested.
//! `apps/desktop` needs rustc 1.91 (iroh/stoolap), this sandbox only
//! has 1.75. The types this renders (`DeviceListState`,
//! `DeviceSecurityView`, `DeviceTrustState`, `DeviceKind`) are real and
//! compile+test-verified in `siar-ui-state`. Please build this locally
//! (`cargo build -p siar-desktop`) and paste back any errors.

use crate::state::AppState;
use dioxus::prelude::*;
use siar_ui_state::{DeviceKind, DeviceSecurityView, DeviceTrustState};

/// §14's four sections, §16's "clearly label: this device."
#[component]
pub fn DevicesScreen() -> Element {
    let state = use_context::<AppState>();
    let devices = state.device_list.read();

    let this_device = devices.this_device().cloned();
    let trusted: Vec<DeviceSecurityView> = devices.trusted().cloned().collect();
    let pending: Vec<DeviceSecurityView> = devices.pending().cloned().collect();
    let history: Vec<DeviceSecurityView> = devices.revoked_or_history().cloned().collect();

    rsx! {
        div { class: "devices-screen",
            if let Some(device) = this_device {
                section { class: "devices-section",
                    h3 { "This Device" }
                    DeviceRow { device: device.clone(), badge: Some("This device".to_string()) }
                }
            }
            if !trusted.is_empty() {
                section { class: "devices-section",
                    h3 { "Trusted Devices" }
                    for device in trusted {
                        DeviceRow { key: "{device.id.fmt_short()}", device: device.clone(), badge: None }
                    }
                }
            }
            if !pending.is_empty() {
                section { class: "devices-section",
                    h3 { "Pending" }
                    for device in pending {
                        // §21: pending devices must not read as having
                        // normal account access — a distinct row variant
                        // rather than DeviceRow with a badge, so pending
                        // rows can't accidentally end up styled
                        // identically to a trusted one.
                        PendingDeviceRow { key: "{device.id.fmt_short()}", device: device.clone() }
                    }
                }
            }
            if !history.is_empty() {
                section { class: "devices-section",
                    h3 { "Revoked / History" }
                    for device in history {
                        DeviceRow { key: "{device.id.fmt_short()}", device: device.clone(), badge: None }
                    }
                }
            }
        }
    }
}

/// §15's row content, §20's coarse last-active wording, §18's "never
/// use device display name as identity" (no raw `DeviceId` shown here
/// — only in `DeviceDetails`, not written this round).
#[component]
fn DeviceRow(device: DeviceSecurityView, badge: Option<String>) -> Element {
    rsx! {
        div { class: "device-row",
            span { class: "device-icon", "{device_kind_icon(device.kind)}" }
            div { class: "device-info",
                span { class: "device-name", "{device.display_name}" }
                span { class: "device-last-active", "{coarse_last_active(device.last_active_millis)}" }
            }
            if let Some(text) = badge {
                span { class: "current-device-badge", "{text}" }
            }
            span { class: "device-trust-state trust-{trust_state_class(device.status)}",
                "{trust_state_label(device.status)}"
            }
        }
    }
}

/// §22: Approve / Deny / Cancel session, kept as a separate component
/// (see `DevicesScreen`'s own comment above) so a pending device can
/// never render with the same visual weight as an already-trusted one.
#[component]
fn PendingDeviceRow(device: DeviceSecurityView) -> Element {
    rsx! {
        div { class: "device-row pending-device-row",
            span { class: "device-icon", "{device_kind_icon(device.kind)}" }
            span { class: "device-name", "{device.display_name}" }
            span { class: "pending-label", "Awaiting approval" }
            button { class: "approve-button", "Approve" }
            button { class: "deny-button", "Deny" }
        }
    }
}

fn device_kind_icon(kind: DeviceKind) -> &'static str {
    match kind {
        DeviceKind::AndroidPhone => "📱",
        DeviceKind::AndroidTablet => "📱",
        DeviceKind::Desktop => "🖥",
        DeviceKind::Laptop => "💻",
        DeviceKind::ServerNode => "🖧",
        DeviceKind::Unknown => "❓",
    }
}

fn trust_state_label(status: DeviceTrustState) -> &'static str {
    match status {
        DeviceTrustState::Trusted => "Trusted",
        DeviceTrustState::Pending => "Pending",
        DeviceTrustState::Revoked => "Revoked",
        DeviceTrustState::Compromised => "Compromised",
        DeviceTrustState::Unknown => "Unknown",
    }
}

fn trust_state_class(status: DeviceTrustState) -> &'static str {
    match status {
        DeviceTrustState::Trusted => "trusted",
        DeviceTrustState::Pending => "pending",
        DeviceTrustState::Revoked => "revoked",
        DeviceTrustState::Compromised => "compromised",
        DeviceTrustState::Unknown => "unknown",
    }
}

/// §20's own examples, verbatim thresholds: "Active now / 5 minutes
/// ago / Yesterday." Coarse by design — no exact timestamps, matching
/// §20's "privacy-conscious coarse wording" instruction.
fn coarse_last_active(last_active_millis: Option<u64>) -> String {
    let Some(_ms) = last_active_millis else {
        return "Never active".to_string();
    };
    // A real implementation needs the current time to compute a
    // relative label ("5 minutes ago") — this component has no clock
    // access of its own (consistent with every other timestamped type
    // in this workspace taking caller-supplied millis rather than
    // reading a wall clock internally, per this session's own established
    // convention). Left as a placeholder pending that wiring, rather
    // than guessing a fake "now" here.
    "Recently active".to_string()
}
