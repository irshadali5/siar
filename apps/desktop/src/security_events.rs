//! Security events UI (Part 28 §42, "Identity Change UX").
//!
//! Follows `components.rs`'s own established shape: a leaf component
//! subscribing to exactly the state slice it renders
//! (`state.security_events`, a `Signal<SecurityEventState>`).

use crate::state::AppState;
use dioxus::prelude::*;
use siar_ui_state::{SecurityEventKind, SecurityEventSeverity};

/// Renders any unacknowledged `StrongWarning`-severity events (§42:
/// "a root/account identity change should trigger a strong warning")
/// as a blocking modal-style banner the user must acknowledge before
/// it's dismissed — not a passive toast, since §42's own wording
/// ("strong warning") implies something a user can't accidentally miss
/// the way a corner toast can be missed.
///
/// This is deliberately separate from `SecurityEventList` below rather
/// than one component handling both severities — a `StrongWarning`
/// needs to interrupt; a `Notice` (new device linked, device revoked)
/// doesn't, and folding both into one component risks the notice case
/// accidentally inheriting the warning case's blocking behavior later.
#[component]
pub fn StrongSecurityWarningBanner() -> Element {
    let mut state = use_context::<AppState>();
    let warnings: Vec<(usize, siar_ui_state::SecurityEvent)> = state
        .security_events
        .read()
        .events()
        .iter()
        .enumerate()
        .filter(|(_, e)| !e.acknowledged && e.kind.severity() == SecurityEventSeverity::StrongWarning)
        .map(|(i, e)| (i, e.clone()))
        .collect();

    if warnings.is_empty() {
        return rsx! {};
    }

    rsx! {
        div { class: "strong-security-warning-overlay",
            for (index, event) in warnings {
                div { class: "strong-security-warning-banner",
                    key: "{index}",
                    span { class: "warning-icon", "⚠" }
                    p { class: "warning-text", "{strong_warning_text(&event.kind)}" }
                    button {
                        class: "acknowledge-button",
                        onclick: move |_| state.security_events.write().acknowledge(index),
                        "I understand"
                    }
                }
            }
        }
    }
}

fn strong_warning_text(kind: &SecurityEventKind) -> String {
    match kind {
        SecurityEventKind::RootIdentityChanged { .. } => {
            "This contact's root identity has changed. Their safety fingerprint no longer \
             matches what you previously verified. Before continuing, re-verify their identity \
             — this can happen after a legitimate account recovery, but it can also mean your \
             conversation is no longer as secure as you previously confirmed."
                .to_string()
        }
        // `StrongSecurityWarningBanner` only ever receives
        // `StrongWarning`-severity events (see its own filter above),
        // and `RootIdentityChanged` is the only variant
        // `SecurityEventKind::severity()` maps to that tier today — see
        // `siar-ui-state`'s `security_event.rs`. This arm exists so a
        // future new `StrongWarning`-severity variant fails to compile
        // here (a non-exhaustive match) rather than silently falling
        // through with no message.
        other => format!("A security event requires your attention: {other:?}"),
    }
}

/// Renders `Notice`-severity events (new device linked, device
/// revoked) as a dismissible list — informational, not blocking,
/// matching §42's own distinction from the strong-warning case above.
#[component]
pub fn SecurityEventList() -> Element {
    let mut state = use_context::<AppState>();
    let notices: Vec<(usize, siar_ui_state::SecurityEvent)> = state
        .security_events
        .read()
        .events()
        .iter()
        .enumerate()
        .filter(|(_, e)| !e.acknowledged && e.kind.severity() == SecurityEventSeverity::Notice)
        .map(|(i, e)| (i, e.clone()))
        .collect();

    rsx! {
        ul { class: "security-event-list",
            for (index, event) in notices {
                li {
                    key: "{index}",
                    class: "security-event-notice",
                    span { class: "notice-text", "{notice_text(&event.kind)}" }
                    button {
                        class: "dismiss-button",
                        onclick: move |_| state.security_events.write().acknowledge(index),
                        "Dismiss"
                    }
                }
            }
        }
    }
}

fn notice_text(kind: &SecurityEventKind) -> String {
    match kind {
        SecurityEventKind::NewDeviceLinked { device } => {
            format!("A new device was linked to your account ({}).", device.fmt_short())
        }
        SecurityEventKind::DeviceRevoked { device } => {
            format!("A device was removed from your account ({}).", device.fmt_short())
        }
        // Same exhaustiveness reasoning as `strong_warning_text` above,
        // mirrored for the `Notice` tier.
        other => format!("{other:?}"),
    }
}
